use std::path::PathBuf;

use crate::{
    entities,
    soulseek::{FileAttribute, SingleFileResult, SoulSeekClientContext, Track},
};
use color_eyre::eyre::Result;
use futures::TryStreamExt;
use tempfile::TempDir;
use tokio::fs::DirEntry;
use tokio_stream::wrappers::ReadDirStream;

use super::matching_local_tracks::matcher::{
    self, MatchConfidence, MatchResult, Track as MatcherTrack,
};

/// Minimum score threshold for accepting a SoulSeek result.
///
/// This is intentionally lower than the Spotify-to-local matching threshold
/// (which requires High confidence / ~0.85 for auto-matching) for two reasons:
///
/// 1. **SoulSeek filenames are unreliable metadata sources.** Artist/title are parsed
///    from file paths (e.g. `@@user\Music\Artist\Album\01 - Title.mp3`) which are
///    inconsistently structured. A strict threshold would reject valid results that
///    simply have oddly-named files or unconventional folder structures.
///
/// 2. **The `import_track()` pipeline provides a second validation layer.** After
///    downloading, AcoustID fingerprinting verifies the audio content matches the
///    expected track, catching any false positives from filename-based scoring.
///
/// Unlike the Spotify-to-local matcher (`find_matches()` in `matcher.rs`), we also
/// skip artist and duration pre-filters. The local matcher pre-filters by primary
/// artist similarity and excludes duration mismatches before scoring. We skip these
/// because SoulSeek path-parsed artists are too unreliable for pre-filtering — a
/// folder named "Various" or "Downloads" would be wrongly compared against the real
/// artist. Instead, we let `compare_tracks()` handle artist/duration as weighted
/// score components, where a poor artist match naturally produces a low overall score.
const MIN_SCORE_THRESHOLD: f64 = 0.50;

#[derive(Debug, Clone)]
struct ParsedSoulseekMetadata {
    title: String,
    artist: String,
    album: String,
}

/// Parse artist, album, and title metadata from a SoulSeek file path.
///
/// SoulSeek filenames are full paths with backslash or forward-slash separators.
/// Common patterns:
/// - `@@user\Music\Artist\Album\01 - Title.mp3` → artist from path, title from filename
/// - `@@user\Music\Artist - Title.mp3` → parse "Artist - Title" from filename
/// - `@@user\Downloads\Title.mp3` → title only, no reliable artist/album info
fn parse_soulseek_filename(filename: &str) -> ParsedSoulseekMetadata {
    // Split by both backslash and forward-slash
    let parts: Vec<&str> = filename
        .split(['\\', '/'])
        .filter(|s| !s.is_empty())
        .collect();

    let raw_filename = parts.last().copied().unwrap_or("");

    // Strip file extension
    let without_ext = raw_filename
        .rsplit_once('.')
        .map(|(name, _)| name)
        .unwrap_or(raw_filename);

    // Strip leading track numbers like "01 - ", "01. ", "1 ", "01-", "1."
    let title_from_filename = strip_track_number(without_ext).trim().to_string();

    // Try to extract artist/album from path components
    // Path typically: [user_prefix, ..., artist, album, filename]
    let num_parts = parts.len();

    if num_parts >= 4 {
        // Have at least: prefix / artist / album / filename
        let artist = parts[num_parts - 3].to_string();
        let album = parts[num_parts - 2].to_string();
        ParsedSoulseekMetadata {
            title: title_from_filename,
            artist,
            album,
        }
    } else if num_parts == 3 {
        // Could be: prefix / artist / filename  OR  prefix / album / filename
        // Assume it's artist
        let artist = parts[num_parts - 2].to_string();
        ParsedSoulseekMetadata {
            title: title_from_filename,
            artist,
            album: String::new(),
        }
    } else {
        // Short path — try "Artist - Title" pattern in filename
        if let Some((artist, title)) = parse_artist_title_from_filename(&title_from_filename) {
            ParsedSoulseekMetadata {
                title,
                artist,
                album: String::new(),
            }
        } else {
            ParsedSoulseekMetadata {
                title: title_from_filename,
                artist: String::new(),
                album: String::new(),
            }
        }
    }
}

/// Strip leading track number patterns from a filename.
/// Handles: "01 - Title", "01. Title", "1 Title", "01-Title", "1.Title"
fn strip_track_number(s: &str) -> &str {
    let bytes = s.as_bytes();
    let len = bytes.len();

    // Find how many leading digits there are
    let digit_end = bytes.iter().take_while(|b| b.is_ascii_digit()).count();
    if digit_end == 0 || digit_end > 3 || digit_end >= len {
        return s;
    }

    let rest = &s[digit_end..];

    // Match separator patterns after digits: " - ", ". ", "- ", "-", ". ", " "
    for sep in &[" - ", ". ", "- ", " . ", ".", "-", " "] {
        if let Some(after) = rest.strip_prefix(sep)
            && !after.is_empty()
        {
            return after;
        }
    }

    s
}

/// Try to parse "Artist - Title" pattern from a filename.
fn parse_artist_title_from_filename(filename: &str) -> Option<(String, String)> {
    // Try " - " separator first (most common)
    if let Some((artist, title)) = filename.split_once(" - ") {
        let artist = artist.trim().to_string();
        let title = title.trim().to_string();
        if !artist.is_empty() && !title.is_empty() {
            return Some((artist, title));
        }
    }
    None
}

/// Score a single SoulSeek result against a Spotify track using the matcher.
fn score_soulseek_result(
    result: &SingleFileResult,
    spotify_track: &entities::spotify_track::Model,
) -> MatchResult {
    let parsed = parse_soulseek_filename(&result.filename);

    // Get duration from SoulSeek attrs (in seconds) and convert to ms
    let duration_ms = result
        .attrs
        .get(&FileAttribute::Duration)
        .map(|&secs| secs * 1000)
        .unwrap_or(0);

    let soulseek_track = MatcherTrack {
        title: parsed.title,
        primary_artist: parsed.artist,
        secondary_artists: vec![],
        album: parsed.album,
        duration_ms,
    };

    let spotify_matcher_track = MatcherTrack {
        title: spotify_track.title.clone(),
        primary_artist: spotify_track.artists.0.first().cloned().unwrap_or_default(),
        secondary_artists: spotify_track
            .artists
            .0
            .get(1..)
            .unwrap_or_default()
            .to_vec(),
        album: spotify_track.album.clone(),
        duration_ms: spotify_track.duration.unwrap_or(0) as u32,
    };

    let soulseek_normalized = matcher::normalize_track(&soulseek_track);
    let spotify_normalized = matcher::normalize_track(&spotify_matcher_track);
    matcher::compare_tracks(&soulseek_normalized, &spotify_normalized)
}

/// Select the best SoulSeek result for a Spotify track by scoring each result
/// using the track matcher and returning the highest-scoring result above the
/// confidence threshold.
fn score_and_pick_best_match<'a>(
    search_results: &'a [SingleFileResult],
    spotify_track: &entities::spotify_track::Model,
) -> Option<&'a SingleFileResult> {
    let mut scored: Vec<(&SingleFileResult, MatchResult)> = search_results
        .iter()
        .map(|r| {
            let match_result = score_soulseek_result(r, spotify_track);
            (r, match_result)
        })
        .collect();

    // Sort by score descending
    scored.sort_by(|a, b| {
        b.1.score
            .partial_cmp(&a.1.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Log top results for debugging
    for (i, (result, match_result)) in scored.iter().take(3).enumerate() {
        tracing::debug!(
            "SoulSeek result #{} for '{}' by '{}': score={:.3}, confidence={}, title_sim={:.3}, artist_sim={:.3}, album_sim={:.3}, duration={}, version={}, filename={}",
            i + 1,
            spotify_track.title,
            spotify_track.artists.0.join(", "),
            match_result.score,
            match_result.confidence,
            match_result.title_similarity,
            match_result.artist_similarity,
            match_result.album_similarity,
            match_result.duration_match,
            match_result.version_match,
            result.filename,
        );
    }

    // Return best result if it meets the threshold
    scored
        .into_iter()
        .next()
        .and_then(|(result, match_result)| {
            if match_result.score >= MIN_SCORE_THRESHOLD
                && !matches!(match_result.confidence, MatchConfidence::NoMatch)
            {
                Some(result)
            } else {
                tracing::debug!(
                    "No SoulSeek result met threshold ({}) for '{}' by '{}'{}",
                    MIN_SCORE_THRESHOLD,
                    spotify_track.title,
                    spotify_track.artists.0.join(", "),
                    scored_summary_suffix(search_results.len(), match_result.score),
                );
                None
            }
        })
}

fn scored_summary_suffix(total: usize, best_score: f64) -> String {
    if total == 0 {
        " (no results)".to_string()
    } else {
        format!(" (best score: {:.3} from {} results)", best_score, total)
    }
}

/// Downloads the best match for a spotify track to the local library.
/// This performs a search for the track on SoulSeek.
/// Then filters and ranks the results based on the track metadata.
/// Finally, it downloads the best match to a temporary directory.
pub async fn download_best_match_for_spotify_track(
    soulseek_context: &SoulSeekClientContext,
    spotify_track: entities::spotify_track::Model,
) -> Result<Option<(TempDir, PathBuf)>> {
    tracing::debug!(
        "Downloading best match for spotify track: {:?}",
        &spotify_track
    );

    let soulseek_search_results = soulseek_context
        .search_for_track(&Track {
            title: spotify_track.title.clone(),
            album: spotify_track.album.clone(),
            artists: spotify_track.artists.0.clone(),
            length: spotify_track.duration.map(|d| d as u32),
        })
        .await?;
    let best_match = score_and_pick_best_match(&soulseek_search_results, &spotify_track);
    let best_match = match best_match {
        Some(best_match) => best_match,
        None => {
            tracing::warn!(
                "No best match found for spotify track: {:?}",
                &spotify_track
            );
            return Ok(None);
        }
    };
    tracing::debug!("Best match found for spotify track: {:?}", best_match);

    let temp_dir = tempfile::tempdir()?;

    // TODO: retries? backoff?
    // TODO: handle no progress on download?
    // We will error if the download fails, so we don't need to handle the result here.
    let mut download_receiver = soulseek_context
        .download_file(best_match, temp_dir.path())
        .await?;

    tracing::debug!("Downloading best match for spotify track: {:?}", best_match);

    while let Some(status) = download_receiver.recv().await {
        match status {
            soulseek_rs::DownloadStatus::Completed => {
                tracing::debug!("Download completed for spotify track: {:?}", best_match);
                break;
            }
            soulseek_rs::DownloadStatus::Failed => {
                return Err(color_eyre::eyre::eyre!("Download failed"));
            }
            soulseek_rs::DownloadStatus::TimedOut => {
                return Err(color_eyre::eyre::eyre!("Download timed out"));
            }
            soulseek_rs::DownloadStatus::InProgress {
                bytes_downloaded,
                total_bytes,
                speed_bytes_per_sec: _,
            } => {
                tracing::debug!(
                    "Download in progress for spotify track: {:?} ({} bytes downloaded, {} bytes total)",
                    best_match,
                    bytes_downloaded,
                    total_bytes
                );
                continue;
            }
            soulseek_rs::DownloadStatus::Queued => {
                continue;
            }
        }
    }

    let files: Vec<DirEntry> = ReadDirStream::new(tokio::fs::read_dir(temp_dir.path()).await?)
        .try_collect()
        .await?;
    if files.len() != 1 {
        return Err(color_eyre::eyre::eyre!(
            "Expected 1 file in temp directory, got {}",
            files.len()
        ));
    }
    let file = &files[0];
    let file_path = file.path();

    Ok(Some((temp_dir, file_path)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_spotify_track(
        title: &str,
        artists: Vec<&str>,
        album: &str,
        duration_ms: Option<i32>,
    ) -> entities::spotify_track::Model {
        entities::spotify_track::Model {
            spotify_track_id: "test_id".to_string(),
            title: title.to_string(),
            artists: entities::spotify_track::StringVec(
                artists.into_iter().map(String::from).collect(),
            ),
            album: album.to_string(),
            duration: duration_ms,
            isrc: None,
            barcode: None,
            created_at: 0,
            updated_at: 0,
            local_track_id: None,
        }
    }

    fn make_soulseek_result(filename: &str, duration_secs: Option<u32>) -> SingleFileResult {
        let mut attrs = HashMap::new();
        if let Some(dur) = duration_secs {
            attrs.insert(FileAttribute::Duration, dur);
        }
        SingleFileResult {
            username: "testuser".to_string(),
            token: "token".to_string(),
            filename: filename.to_string(),
            size: 5_000_000,
            slots_free: true,
            avg_speed: 100.0,
            queue_length: 0,
            attrs,
        }
    }

    // =========================================================================
    // parse_soulseek_filename tests
    // =========================================================================

    #[test]
    fn test_parse_full_path_with_artist_album() {
        let parsed = parse_soulseek_filename(
            "@@user\\Music\\Queen\\A Night at the Opera\\01 - Bohemian Rhapsody.mp3",
        );
        assert_eq!(parsed.title, "Bohemian Rhapsody");
        assert_eq!(parsed.artist, "Queen");
        assert_eq!(parsed.album, "A Night at the Opera");
    }

    #[test]
    fn test_parse_forward_slash_path() {
        let parsed = parse_soulseek_filename(
            "@@user/Music/Radiohead/OK Computer/03 - Subterranean Homesick Alien.flac",
        );
        assert_eq!(parsed.title, "Subterranean Homesick Alien");
        assert_eq!(parsed.artist, "Radiohead");
        assert_eq!(parsed.album, "OK Computer");
    }

    #[test]
    fn test_parse_track_number_dot_separator() {
        let parsed = parse_soulseek_filename("@@user\\Music\\Artist\\Album\\05. Song Title.mp3");
        assert_eq!(parsed.title, "Song Title");
    }

    #[test]
    fn test_parse_no_track_number() {
        let parsed = parse_soulseek_filename("@@user\\Music\\Artist\\Album\\Song Title.mp3");
        assert_eq!(parsed.title, "Song Title");
        assert_eq!(parsed.artist, "Artist");
    }

    #[test]
    fn test_parse_short_path_artist_title_pattern() {
        let parsed = parse_soulseek_filename("@@user\\Queen - Bohemian Rhapsody.mp3");
        assert_eq!(parsed.title, "Bohemian Rhapsody");
        assert_eq!(parsed.artist, "Queen");
    }

    #[test]
    fn test_parse_short_path_no_artist() {
        let parsed = parse_soulseek_filename("@@user\\Some Song.mp3");
        assert_eq!(parsed.title, "Some Song");
        assert_eq!(parsed.artist, "");
    }

    #[test]
    fn test_parse_three_component_path() {
        let parsed = parse_soulseek_filename("@@user\\Queen\\Bohemian Rhapsody.mp3");
        assert_eq!(parsed.title, "Bohemian Rhapsody");
        assert_eq!(parsed.artist, "Queen");
        assert_eq!(parsed.album, "");
    }

    // =========================================================================
    // strip_track_number tests
    // =========================================================================

    #[test]
    fn test_strip_track_number_dash() {
        assert_eq!(strip_track_number("01 - Title"), "Title");
    }

    #[test]
    fn test_strip_track_number_dot() {
        assert_eq!(strip_track_number("01. Title"), "Title");
    }

    #[test]
    fn test_strip_track_number_space() {
        assert_eq!(strip_track_number("01 Title"), "Title");
    }

    #[test]
    fn test_strip_no_track_number() {
        assert_eq!(strip_track_number("Title"), "Title");
    }

    #[test]
    fn test_strip_track_number_single_digit() {
        assert_eq!(strip_track_number("1 - Title"), "Title");
    }

    // =========================================================================
    // score_and_pick_best_match tests
    // =========================================================================

    #[test]
    fn test_matching_track_scores_high() {
        let spotify = make_spotify_track(
            "Bohemian Rhapsody",
            vec!["Queen"],
            "A Night at the Opera",
            Some(354000),
        );
        let results = vec![make_soulseek_result(
            "@@user\\Music\\Queen\\A Night at the Opera\\01 - Bohemian Rhapsody.mp3",
            Some(354), // 354 seconds
        )];

        let best = score_and_pick_best_match(&results, &spotify);
        assert!(best.is_some());
    }

    #[test]
    fn test_wrong_artist_scores_low() {
        let spotify = make_spotify_track(
            "Bohemian Rhapsody",
            vec!["Queen"],
            "A Night at the Opera",
            Some(354000),
        );
        let result = make_soulseek_result(
            "@@user\\Music\\Metallica\\Master of Puppets\\01 - Bohemian Rhapsody.mp3",
            Some(354),
        );

        let match_result = score_soulseek_result(&result, &spotify);
        // Artist mismatch should drag the score down significantly
        assert!(
            match_result.artist_similarity < 0.5,
            "artist_similarity should be low for wrong artist, got {}",
            match_result.artist_similarity
        );
    }

    #[test]
    fn test_duration_mismatch_reduces_score() {
        let spotify = make_spotify_track(
            "Short Song",
            vec!["Artist"],
            "Album",
            Some(180_000), // 3 minutes
        );

        let good_duration = make_soulseek_result(
            "@@user\\Music\\Artist\\Album\\01 - Short Song.mp3",
            Some(180),
        );
        let bad_duration = make_soulseek_result(
            "@@user\\Music\\Artist\\Album\\01 - Short Song.mp3",
            Some(600), // 10 minutes — way off
        );

        let good_score = score_soulseek_result(&good_duration, &spotify).score;
        let bad_score = score_soulseek_result(&bad_duration, &spotify).score;

        assert!(
            good_score > bad_score,
            "Good duration score ({}) should beat bad duration score ({})",
            good_score,
            bad_score
        );
    }

    #[test]
    fn test_no_results_returns_none() {
        let spotify = make_spotify_track("Song", vec!["Artist"], "Album", Some(180_000));
        let results: Vec<SingleFileResult> = vec![];

        let best = score_and_pick_best_match(&results, &spotify);
        assert!(best.is_none());
    }

    #[test]
    fn test_all_below_threshold_returns_none() {
        let spotify = make_spotify_track(
            "Bohemian Rhapsody",
            vec!["Queen"],
            "A Night at the Opera",
            Some(354000),
        );
        // Completely unrelated track
        let results = vec![make_soulseek_result(
            "@@user\\Music\\Metallica\\Master of Puppets\\01 - Battery.mp3",
            Some(312),
        )];

        let best = score_and_pick_best_match(&results, &spotify);
        assert!(best.is_none(), "Unrelated track should not match");
    }

    #[test]
    fn test_picks_best_from_multiple_results() {
        let spotify = make_spotify_track(
            "Bohemian Rhapsody",
            vec!["Queen"],
            "A Night at the Opera",
            Some(354000),
        );
        let results = vec![
            // Wrong track entirely
            make_soulseek_result(
                "@@user\\Music\\Metallica\\Master of Puppets\\01 - Battery.mp3",
                Some(312),
            ),
            // Correct track
            make_soulseek_result(
                "@@user\\Music\\Queen\\A Night at the Opera\\01 - Bohemian Rhapsody.mp3",
                Some(354),
            ),
            // Same artist, wrong track
            make_soulseek_result(
                "@@user\\Music\\Queen\\Greatest Hits\\01 - We Will Rock You.mp3",
                Some(122),
            ),
        ];

        let best = score_and_pick_best_match(&results, &spotify);
        assert!(best.is_some());
        assert!(best.unwrap().filename.contains("Bohemian Rhapsody"));
    }
}
