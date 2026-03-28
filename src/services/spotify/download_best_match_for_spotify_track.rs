use std::path::PathBuf;
use std::time::Duration;

use crate::entities;
use color_eyre::eyre::Result;
use song_rs::{Client as SongDownloader, SongQuery, WantedFileTypes};
use tempfile::TempDir;
use tracing::instrument;

/// Downloads the best match for a Spotify track to a temporary directory.
/// Uses `song-rs` which performs search, ranking, and download; no local scoring is needed.
#[instrument(skip(song_downloader))]
pub async fn download_best_match_for_spotify_track(
    song_downloader: &SongDownloader,
    spotify_track: entities::spotify_track::Model,
) -> Result<Option<(TempDir, PathBuf)>> {
    tracing::debug!(
        "Downloading best match for spotify track: {:?}",
        &spotify_track
    );

    let temp_dir = tempfile::tempdir()?;
    let temp_dir_path_str = temp_dir
        .path()
        .as_os_str()
        .to_str()
        .ok_or_else(|| color_eyre::eyre::eyre!("Temp dir path is not valid UTF-8"))?
        .to_string();

    let query = SongQuery {
        title: spotify_track.title.clone(),
        artist: spotify_track.artists.0.first().cloned().unwrap_or_default(),
        album: Some(spotify_track.album.clone()),
        duration_secs: spotify_track.duration.unwrap_or(0) as u32 / 1000,
    };

    let (best, _download, mut status_rx) = match song_downloader
        .download_best(
            &query,
            Duration::from_secs(10),
            &temp_dir_path_str,
            &WantedFileTypes::all(),
        )
        .await
    {
        Ok(triple) => triple,
        Err(song_rs::Error::NoResults) => {
            tracing::warn!(
                "No SoulSeek results for spotify track: {:?}",
                &spotify_track
            );
            return Ok(None);
        }
        Err(e) => {
            return Err(color_eyre::eyre::eyre!("SoulSeek error: {}", e));
        }
    };

    while let Some(status) = status_rx.recv().await {
        match status {
            song_rs::DownloadStatus::Queued => {
                tracing::debug!("Download queued");
            }
            song_rs::DownloadStatus::InProgress { .. } => {
                tracing::debug!("Download in progress");
                continue;
            }
            song_rs::DownloadStatus::Completed => {
                tracing::debug!("Download completed");
                break;
            }
            song_rs::DownloadStatus::Failed => {
                tracing::error!("Download failed");
                return Err(color_eyre::eyre::eyre!("Download failed"));
            }
            song_rs::DownloadStatus::TimedOut => {
                tracing::error!("Download timed out");
                return Err(color_eyre::eyre::eyre!("Download timed out"));
            }
        }
    }

    let file_path = temp_dir.path().join(best.filename.filename());
    if !file_path.exists() {
        return Err(color_eyre::eyre::eyre!(
            "Download completed but file not found at {}",
            file_path.display()
        ));
    }

    Ok(Some((temp_dir, file_path)))
}
