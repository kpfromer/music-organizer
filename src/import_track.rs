use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::acoustid::{AcoustIdRecording, lookup_fingerprint};
use crate::musicbrainz::{fetch_recording_with_details, fetch_release_with_details};
use crate::{chromaprint, file_hash};
use color_eyre::Result;
use reqwest::Client;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use serde::{Deserialize, Serialize};
use tokio::time::interval;
use tracing::instrument;

use crate::{config::Config, database::Database};

pub const SUPPORTED_FILE_TYPES: &[&str] = &["mp3", "flac", "m4a", "aac", "ogg", "wav"];

#[derive(Debug, Clone, thiserror::Error, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ImportError {
    #[error("Already tried to import this file and failed originally")]
    AlreadyTriedToImport,

    #[error("Unsupported file type: {extension}")]
    UnsupportedFileType { extension: String },

    #[error(
        "Duplicate track detected! This track (MusicBrainz ID: {musicbrainz_id}) already exists in the database at: {existing_path}"
    )]
    DuplicateTrack {
        musicbrainz_id: String,
        existing_path: String,
    },

    #[error("File system error during {operation} on {path}: {error_message}")]
    FileSystemError {
        operation: String,
        path: String,
        error_message: String,
    },

    #[error("Hash computation error: {message}")]
    HashComputationError { message: String },

    #[error("Chromaprint error: {reason}")]
    ChromaprintError { reason: String },

    #[error("AcoustID error: {reason}")]
    AcoustIdError { reason: String },

    #[error("MusicBrainz error: {reason}")]
    MusicBrainzError { reason: String },

    #[error("Database error during {operation}: {error_message}")]
    DatabaseError {
        operation: String,
        error_message: String,
    },
}

impl ImportError {
    /// Serialize the error to a JSON string for database storage
    pub fn to_db_string(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| self.to_string())
    }
}

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
async fn gather_track_metadata(
    file_path: &Path,
    api_key: &str,
) -> Result<TrackMetadata, ImportError> {
    log::debug!("Gathering metadata for file: {}", file_path.display());

    let client = Client::new();

    // Compute SHA-256 hash
    log::debug!("Computing SHA-256 hash");
    let sha256 =
        file_hash::compute_sha256(file_path).map_err(|e| ImportError::HashComputationError {
            message: e.to_string(),
        })?;

    // Get fingerprint for AcoustID lookup
    log::debug!("Getting fingerprint from chromaprint");
    let (fingerprint, duration) = chromaprint::chromaprint_from_file(file_path).map_err(|e| {
        ImportError::ChromaprintError {
            reason: e.to_string(),
        }
    })?;

    // Lookup via AcoustID
    log::debug!("Initiating AcoustID lookup (duration: {}s)", duration);
    let resp = lookup_fingerprint(&client, api_key, &fingerprint, duration)
        .await
        .map_err(|e| ImportError::AcoustIdError {
            reason: format!("Failed to lookup fingerprint: {}", e),
        })?;

    let result = resp
        .results
        .into_iter()
        .max_by(|a, b| {
            a.score
                .partial_cmp(&b.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .ok_or(ImportError::AcoustIdError {
            reason: "No AcoustID results found".to_string(),
        })?;

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
        .ok_or(ImportError::AcoustIdError {
            reason: "No recordings found".to_string(),
        })?;

    // Fetch from MusicBrainz
    log::debug!(
        "Fetching recording details from MusicBrainz (ID: {})",
        best_recording.id
    );
    let recording_from_musicbrainz = fetch_recording_with_details(&best_recording.id)
        .await
        .map_err(|e| ImportError::MusicBrainzError {
            reason: format!("Failed to fetch recording: {}", e),
        })?;

    let release = recording_from_musicbrainz
        .releases
        .as_ref()
        .ok_or(ImportError::MusicBrainzError {
            reason: "No releases found".to_string(),
        })?
        .first()
        .ok_or(ImportError::MusicBrainzError {
            reason: "No releases found".to_string(),
        })?;

    log::debug!(
        "Fetching release details from MusicBrainz (ID: {})",
        release.id
    );
    let release_from_musicbrainz = fetch_release_with_details(&release.id).await.map_err(|e| {
        ImportError::MusicBrainzError {
            reason: format!("Failed to fetch release: {}", e),
        }
    })?;

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
        // Find the track number using global ordering across all discs
        // If a track is on disc 2 position 3, and disc 1 has 10 tracks, result is 13
        release_from_musicbrainz.media.as_ref().and_then(|media| {
            let mut offset = 0;
            media.iter().find_map(|medium| {
                let tracks = medium.tracks.as_deref().unwrap_or(&[]);

                // Check if our recording is in this medium
                if let Some(track) = tracks.iter().find(|t| {
                    t.recording
                        .as_ref()
                        .is_some_and(|r| r.id == best_recording.id)
                }) {
                    // Found it! Return the global position
                    Some((offset + track.position) as i32)
                } else {
                    // Not in this medium, add its track count to offset
                    offset += tracks.len() as u32;
                    None
                }
            })
        })
    }
    .ok_or(ImportError::MusicBrainzError {
        reason: "No track number found".to_string(),
    })?;

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
#[instrument(skip(api_key, config, database))]
pub async fn import_track(
    file_path: &Path,
    api_key: &str,
    config: &Config,
    database: &Database,
) -> Result<crate::entities::track::Model, ImportError> {
    log::debug!("Starting import for file: {}", file_path.display());

    let extension = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if !SUPPORTED_FILE_TYPES.contains(&extension) {
        return Err(ImportError::UnsupportedFileType {
            extension: extension.to_string(),
        });
    }

    let unimportable_file = database
        .get_unimportable_file_by_file_path(file_path.to_str().unwrap())
        .await
        .map_err(|e| ImportError::DatabaseError {
            operation: "get unimportable file by file path".to_string(),
            error_message: e.to_string(),
        })?;

    if unimportable_file.is_some() {
        log::debug!(
            "Track {} already exists in unimportable database skipping",
            file_path.display()
        );
        return Err(ImportError::AlreadyTriedToImport);
    }

    // Gather all metadata
    let metadata = gather_track_metadata(file_path, api_key).await?;

    // Check for duplicate by MusicBrainz ID
    log::debug!("Checking for duplicate by MusicBrainz ID");
    if let Some(track_mbid) = &metadata.track_musicbrainz_id {
        let is_duplicate = database
            .is_duplicate_by_musicbrainz_id(track_mbid)
            .await
            .map_err(|e| ImportError::DatabaseError {
                operation: "check duplicate".to_string(),
                error_message: e.to_string(),
            })?;

        if is_duplicate {
            let existing = database
                .get_track_by_sha256(&metadata.sha256)
                .await
                .map_err(|e| ImportError::DatabaseError {
                    operation: "get track by sha256".to_string(),
                    error_message: e.to_string(),
                })?;
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

            return Err(ImportError::DuplicateTrack {
                musicbrainz_id: track_mbid.clone(),
                existing_path: existing_path.to_string(),
            });
        }
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
        std::fs::create_dir_all(parent).map_err(|e| ImportError::FileSystemError {
            operation: "create directory".to_string(),
            path: parent.display().to_string(),
            error_message: e.to_string(),
        })?;
    }

    // Try rename first (fast for same filesystem)
    log::debug!(
        "Moving file from {} to {}",
        metadata.source_path.display(),
        organized_path.display()
    );
    if std::fs::rename(&metadata.source_path, &organized_path).is_err() {
        // If rename fails, assume it's a cross-filesystem move
        log::debug!("Rename failed, copying file across filesystems");
        std::fs::copy(&metadata.source_path, &organized_path).map_err(|e| {
            ImportError::FileSystemError {
                operation: "copy file".to_string(),
                path: format!(
                    "{} -> {}",
                    metadata.source_path.display(),
                    organized_path.display()
                ),
                error_message: e.to_string(),
            }
        })?;

        log::debug!("Removing original file: {}", metadata.source_path.display());
        std::fs::remove_file(&metadata.source_path).map_err(|e| ImportError::FileSystemError {
            operation: "remove file".to_string(),
            path: metadata.source_path.display().to_string(),
            error_message: e.to_string(),
        })?;
    }

    // Now update database
    log::debug!("Updating database with track information");

    // Upsert artists
    let mut album_artist_ids = Vec::new();
    for (idx, (name, mbid)) in metadata.album_artists.iter().enumerate() {
        let artist_id = database
            .upsert_artist(name, mbid.as_deref())
            .await
            .map_err(|e| ImportError::DatabaseError {
                operation: format!("upsert artist: {}", name),
                error_message: e.to_string(),
            })?;
        album_artist_ids.push((artist_id, idx == 0)); // First is primary
    }

    let mut track_artist_ids = Vec::new();
    for (idx, (name, mbid)) in metadata.track_artists.iter().enumerate() {
        let artist_id = database
            .upsert_artist(name, mbid.as_deref())
            .await
            .map_err(|e| ImportError::DatabaseError {
                operation: format!("upsert artist: {}", name),
                error_message: e.to_string(),
            })?;
        track_artist_ids.push((artist_id, idx == 0)); // First is primary
    }

    // Upsert album
    let album_id = database
        .upsert_album(
            &metadata.album_title,
            metadata.album_musicbrainz_id.as_deref(),
            metadata.album_year,
        )
        .await
        .map_err(|e| ImportError::DatabaseError {
            operation: format!("upsert album: {}", metadata.album_title),
            error_message: e.to_string(),
        })?;

    // Link album artists
    for (artist_id, is_primary) in album_artist_ids {
        database
            .add_album_artist(album_id, artist_id, is_primary)
            .await
            .map_err(|e| ImportError::DatabaseError {
                operation: "add album artist".to_string(),
                error_message: e.to_string(),
            })?;
    }

    // Upsert track
    let track_id = database
        .upsert_track(
            album_id,
            &metadata.track_title,
            Some(metadata.track_number),
            metadata.duration,
            metadata.track_musicbrainz_id.as_deref(),
            &organized_path,
            &metadata.sha256,
        )
        .await
        .map_err(|e| ImportError::DatabaseError {
            operation: format!("upsert track: {}", metadata.track_title),
            error_message: e.to_string(),
        })?;

    // Link track artists
    for (artist_id, is_primary) in track_artist_ids {
        database
            .add_track_artist(track_id, artist_id, is_primary)
            .await
            .map_err(|e| ImportError::DatabaseError {
                operation: "add track artist".to_string(),
                error_message: e.to_string(),
            })?;
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
    let track = crate::entities::track::Entity::find()
        .filter(crate::entities::track::Column::FilePath.eq(organized_path.to_str().unwrap()))
        .one(&database.conn)
        .await
        .map_err(|e| ImportError::DatabaseError {
            operation: "get track by file path".to_string(),
            error_message: e.to_string(),
        })?
        .ok_or_else(|| ImportError::DatabaseError {
            operation: "get track by file path".to_string(),
            error_message: "Track not found".to_string(),
        })?;

    Ok(track)
}

#[instrument(skip(api_key, config, database))]
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
        match result {
            Ok(_) => {
                success_count += 1;
            }
            Err(ImportError::AlreadyTriedToImport) => {
                // We already tried to import this file and failed originally, so we don't need to do anything
                // or log anything
            }
            Err(e) => {
                error_count += 1;
                log::warn!("Error importing track {}: {}", path.display(), e);
                println!("Error importing track {}: {}", path.display(), e);
                // Optionally save to unimportable_files here if desired
            }
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

// TODO: I need to way to get rejected files and save them
/// Watch a directory for new music files and import them automatically
#[instrument(skip(api_key, config, database))]
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
                    // Skip files we've already tried to import
                    if matches!(e, ImportError::AlreadyTriedToImport) {
                        continue;
                    }

                    log::warn!("Error importing track {}: {}", path.display(), e);
                    println!("Error importing track {}: {}", path.display(), e);

                    // Compute SHA-256 for the unimportable file record
                    let sha256 = match file_hash::compute_sha256(path) {
                        Ok(hash) => hash,
                        Err(hash_err) => {
                            log::error!(
                                "Failed to compute SHA-256 for unimportable file {}: {}",
                                path.display(),
                                hash_err
                            );
                            // Use a placeholder if hash computation fails
                            "unknown".to_string()
                        }
                    };

                    // Save to unimportable_files
                    if let Err(db_err) = database.insert_unimportable_file(path, &sha256, &e).await
                    {
                        log::error!(
                            "Failed to insert unimportable file {}: {}",
                            path.display(),
                            db_err
                        );
                    } else {
                        log::debug!("Saved unimportable file to database: {}", path.display());
                    }
                }
            }
        }
    }
}
