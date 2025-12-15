use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Duration;

use color_eyre::eyre::OptionExt;
use color_eyre::{Result, eyre::Context};
use reqwest::Client;
use tokio::time::interval;

use crate::acoustid::{AcoustIdRecording, lookup_fingerprint};
use crate::musicbrainz::{fetch_recording_with_details, fetch_release_with_details};
use crate::{chromaprint, file_hash};

use crate::{config::Config, database::Database};

pub const SUPPORTED_FILE_TYPES: &[&str] = &["mp3", "flac", "m4a", "aac", "ogg", "wav"];

#[derive(Debug)]
struct TrackMetadata {
    // File info
    source_path: PathBuf,
    sha256: String,

    // Track info
    track_title: String,
    track_number: i32,
    duration: Option<i32>,
    track_musicbrainz_id: Option<String>,

    // Album info
    album_title: String,
    album_musicbrainz_id: Option<String>,
    album_year: Option<i32>,

    // Artists (primary first) - (name, musicbrainz_id)
    track_artists: Vec<(String, Option<String>)>,
    album_artists: Vec<(String, Option<String>)>,
}

/// Gather all metadata needed for database insertion from a file
async fn gather_track_metadata(file_path: &Path, api_key: &str) -> Result<TrackMetadata> {
    log::debug!("Gathering metadata for file: {}", file_path.display());

    let client = Client::new();

    // Compute SHA-256 hash
    log::debug!("Computing SHA-256 hash");
    let sha256 = file_hash::compute_sha256(file_path)?;

    // Get fingerprint for AcoustID lookup
    log::debug!("Getting fingerprint from chromaprint");
    let (fingerprint, duration) = chromaprint::chromaprint_from_file(file_path)?;

    // Lookup via AcoustID
    log::debug!("Initiating AcoustID lookup (duration: {}s)", duration);
    let resp = lookup_fingerprint(&client, api_key, &fingerprint, duration)
        .await
        .wrap_err("Failed to lookup fingerprint")?;

    let result = resp
        .results
        .into_iter()
        .max_by(|a, b| {
            a.score
                .partial_cmp(&b.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .ok_or(color_eyre::eyre::eyre!("No AcoustID results found"))?;

    log::debug!(
        "Best AcoustID result selected with score: {:.2}",
        result.score
    );

    let best_recording: AcoustIdRecording = result
        .recordings
        .into_iter()
        .min_by(|a, b| match (a.duration, b.duration) {
            (Some(a_duration), Some(b_duration)) => (a_duration - duration as f64)
                .abs()
                .partial_cmp(&(b_duration - duration as f64).abs())
                .unwrap_or(std::cmp::Ordering::Equal),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        })
        .ok_or(color_eyre::eyre::eyre!("No recordings found"))?;

    // Fetch from MusicBrainz
    log::debug!(
        "Fetching recording details from MusicBrainz (ID: {})",
        best_recording.id
    );
    let recording_from_musicbrainz = fetch_recording_with_details(&best_recording.id).await?;

    let release = recording_from_musicbrainz
        .releases
        .as_ref()
        .ok_or(color_eyre::eyre::eyre!("No releases found"))?
        .first()
        .ok_or(color_eyre::eyre::eyre!("No releases found"))?;

    log::debug!(
        "Fetching release details from MusicBrainz (ID: {})",
        release.id
    );
    let release_from_musicbrainz = fetch_release_with_details(&release.id).await?;

    // Extract track artists from MusicBrainz recording
    let mut track_artists = Vec::new();
    if let Some(artist_credit) = &recording_from_musicbrainz.artist_credit {
        for credit in artist_credit {
            track_artists.push((credit.name.clone(), Some(credit.artist.id.clone())));
        }
    }

    // Extract album artists from MusicBrainz release group
    let mut album_artists = Vec::new();
    if let Some(release_group) = &release_from_musicbrainz.release_group
        && let Some(artist_credit) = &release_group.artist_credit
    {
        for credit in artist_credit {
            album_artists.push((credit.name.clone(), Some(credit.artist.id.clone())));
        }
    }

    // Get album title from release group
    let album_title = release_from_musicbrainz
        .release_group
        .as_ref()
        .map(|rg| rg.title.clone())
        .unwrap_or_else(|| release_from_musicbrainz.title.clone());

    // Get album year from release date (parse from DateString format like "2018-11-16")
    let album_year = release_from_musicbrainz.date.as_ref().and_then(|d| {
        // DateString is a tuple struct with String field
        // Format is typically "YYYY-MM-DD" or "YYYY"
        d.0.split('-')
            .next()
            .and_then(|year_str| year_str.parse::<i32>().ok())
    });

    let track_number = {
        // Find the track number in the media tracks
        release_from_musicbrainz
            .media
            .as_ref()
            .map(|media| {
                media
                    .iter()
                    .flat_map(|medium| medium.tracks.as_deref().unwrap_or(&[]))
                    .filter(|t| {
                        t.recording
                            .as_ref()
                            .is_some_and(|r| r.id == best_recording.id)
                    })
                    .map(|t| t.position)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
            .first()
            .map(|i| *i as i32)
    }
    .ok_or_eyre("No track number found")?;

    log::info!(
        "Metadata gathered successfully: '{}' by '{}' from album '{}'",
        recording_from_musicbrainz.title,
        track_artists
            .first()
            .map(|(name, _)| name.as_str())
            .unwrap_or("Unknown"),
        album_title
    );

    Ok(TrackMetadata {
        source_path: file_path.to_path_buf(),
        sha256,
        track_title: recording_from_musicbrainz.title.clone(),
        track_number,
        duration: duration.try_into().ok(),
        track_musicbrainz_id: Some(best_recording.id),
        album_title,
        album_musicbrainz_id: release_from_musicbrainz
            .release_group
            .as_ref()
            .map(|rg| rg.id.clone()),
        album_year,
        track_artists,
        album_artists,
    })
}

/// Sanitize filename for filesystem (remove invalid characters)
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}

/// Import a track: check for duplicates, move file, and update database
pub async fn import_track(
    file_path: &Path,
    api_key: &str,
    config: &Config,
    database: &Database,
) -> Result<()> {
    log::debug!("Starting import for file: {}", file_path.display());

    if !SUPPORTED_FILE_TYPES.contains(&file_path.extension().and_then(|e| e.to_str()).unwrap_or(""))
    {
        return Err(color_eyre::eyre::eyre!(
            "Unsupported file type: {}",
            file_path.extension().and_then(|e| e.to_str()).unwrap_or("")
        ));
    }

    // Gather all metadata
    let metadata = gather_track_metadata(file_path, api_key).await?;

    // Check for duplicate by MusicBrainz ID
    log::debug!("Checking for duplicate by MusicBrainz ID");
    if let Some(track_mbid) = &metadata.track_musicbrainz_id
        && database.is_duplicate_by_musicbrainz_id(track_mbid).await?
    {
        let existing = database.get_track_by_sha256(&metadata.sha256).await?;
        let existing_path = existing
            .as_ref()
            .map(|t| t.file_path.as_str())
            .unwrap_or("unknown");

        log::warn!(
            "Duplicate track detected: '{}' (MusicBrainz ID: {}) already exists at: {}",
            metadata.track_title,
            track_mbid,
            existing_path
        );

        return Err(color_eyre::eyre::eyre!(
            "Duplicate track detected! This track (MusicBrainz ID: {}) already exists in the database at: {}",
            track_mbid,
            existing_path
        ));
    }

    // Get primary album artist for folder structure
    let primary_album_artist = if let Some((artist, _)) = metadata.album_artists.first() {
        artist.clone()
    } else {
        "Unknown Artist".to_string()
    };

    // Construct organized file path: DIRECTORY/ArtistName/AlbumName/TrackNumber - TrackName.ext
    let extension = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("mp3");

    let track_number_str = format!("{:02} ", metadata.track_number);

    let sanitized_artist = sanitize_filename(&primary_album_artist);
    let sanitized_album = sanitize_filename(&metadata.album_title);
    let sanitized_track = sanitize_filename(&metadata.track_title);

    let organized_path = config
        .directory_path()
        .join(&sanitized_artist)
        .join(&sanitized_album)
        .join(format!(
            "{}{}.{}",
            track_number_str, sanitized_track, extension
        ));

    log::debug!("Organized path: {}", organized_path.display());

    // Create directories if needed
    if let Some(parent) = organized_path.parent() {
        log::debug!("Creating directory structure: {}", parent.display());
        std::fs::create_dir_all(parent)
            .context(format!("Failed to create directory: {}", parent.display()))?;
    }

    // Try rename first (fast for same filesystem)
    log::debug!(
        "Moving file from {} to {}",
        metadata.source_path.display(),
        organized_path.display()
    );
    if std::fs::rename(&metadata.source_path, &organized_path)
        .context(format!(
            "Failed to move file from {} to {}",
            metadata.source_path.display(),
            organized_path.display()
        ))
        .is_err()
    {
        // If rename fails, assume it's a cross-filesystem move
        log::debug!("Rename failed, copying file across filesystems");
        std::fs::copy(&metadata.source_path, &organized_path).context(format!(
            "Failed to copy file from {} to {}",
            metadata.source_path.display(),
            organized_path.display()
        ))?;

        log::debug!("Removing original file: {}", metadata.source_path.display());
        std::fs::remove_file(&metadata.source_path).context(format!(
            "Failed to remove original file: {}",
            metadata.source_path.display()
        ))?;
    };

    // Now update database
    log::debug!("Updating database with track information");

    // Upsert artists
    let mut album_artist_ids = Vec::new();
    for (idx, (name, mbid)) in metadata.album_artists.iter().enumerate() {
        let artist_id = database.upsert_artist(name, mbid.as_deref()).await?;
        album_artist_ids.push((artist_id, idx == 0)); // First is primary
    }

    let mut track_artist_ids = Vec::new();
    for (idx, (name, mbid)) in metadata.track_artists.iter().enumerate() {
        let artist_id = database.upsert_artist(name, mbid.as_deref()).await?;
        track_artist_ids.push((artist_id, idx == 0)); // First is primary
    }

    // Upsert album
    let album_id = database
        .upsert_album(
            &metadata.album_title,
            metadata.album_musicbrainz_id.as_deref(),
            metadata.album_year,
        )
        .await?;

    // Link album artists
    for (artist_id, is_primary) in album_artist_ids {
        database
            .add_album_artist(album_id, artist_id, is_primary)
            .await?;
    }

    // Upsert track
    let track_id = database
        .upsert_track(
            album_id,
            &metadata.track_title,
            Some(metadata.track_number),
            metadata.duration,
            metadata.track_musicbrainz_id.as_deref(),
            &organized_path.to_string_lossy(),
            &metadata.sha256,
        )
        .await?;

    // Link track artists
    for (artist_id, is_primary) in track_artist_ids {
        database
            .add_track_artist(track_id, artist_id, is_primary)
            .await?;
    }

    log::info!(
        "Track imported successfully: '{}' by '{}' -> {}",
        metadata.track_title,
        metadata
            .track_artists
            .first()
            .map(|(name, _)| name.as_str())
            .unwrap_or("Unknown"),
        organized_path.display()
    );

    println!("Successfully imported: {}", organized_path.display());
    Ok(())
}

pub async fn import_folder(
    folder_path: &Path,
    api_key: &str,
    config: &Config,
    database: &Database,
) -> Result<()> {
    log::debug!("Starting folder import from: {}", folder_path.display());

    let mut success_count = 0;
    let mut error_count = 0;
    let mut total_count = 0;

    for entry in walkdir::WalkDir::new(folder_path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            e.file_type().is_file()
                && SUPPORTED_FILE_TYPES
                    .contains(&e.path().extension().and_then(|e| e.to_str()).unwrap_or(""))
        })
    {
        let path = entry.path();
        total_count += 1;
        log::info!("Processing file ({}/...): {}", total_count, path.display());

        let result = import_track(path, api_key, config, database).await;
        if let Err(e) = result {
            error_count += 1;
            log::warn!("Error importing track {}: {}", path.display(), e);
            println!("Error importing track {}: {}", path.display(), e);
        } else {
            success_count += 1;
        }
    }

    log::info!(
        "Folder import complete: {} successful, {} errors, {} total",
        success_count,
        error_count,
        total_count
    );

    Ok(())
}

/// Watch a directory for new music files and import them automatically
pub async fn watch_directory(
    directory: &Path,
    api_key: &str,
    config: &Config,
    database: &Database,
) -> Result<()> {
    log::info!("Starting watch mode for directory: {}", directory.display());
    println!("Watching directory: {}", directory.display());
    println!("Press Ctrl+C to stop watching...");

    let mut seen_files = HashSet::new();
    let mut interval = interval(Duration::from_secs(5));

    loop {
        interval.tick().await;
        log::debug!("Scanning directory for new files");

        // Scan for new files
        for entry in walkdir::WalkDir::new(directory)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| {
                e.file_type().is_file()
                    && SUPPORTED_FILE_TYPES
                        .contains(&e.path().extension().and_then(|e| e.to_str()).unwrap_or(""))
            })
        {
            let path = entry.path();
            if let Ok(canonical) = path.canonicalize()
                && !seen_files.contains(&canonical)
            {
                seen_files.insert(canonical.clone());
                log::info!("New file detected: {}", path.display());
                println!("New file detected: {}", path.display());
                let result = import_track(path, api_key, config, database).await;
                if let Err(e) = result {
                    log::warn!("Error importing track {}: {}", path.display(), e);
                    println!("Error importing track {}: {}", path.display(), e);
                }
            }
        }
    }
}
