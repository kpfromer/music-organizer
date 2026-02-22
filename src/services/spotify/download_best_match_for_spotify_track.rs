use std::path::PathBuf;
use tracing;

use crate::{
    entities,
    soulseek::{SingleFileResult, SoulSeekClientContext, Track},
};
use color_eyre::eyre::Result;
use futures::TryStreamExt;
use tempfile::TempDir;
use tokio::fs::DirEntry;
use tokio_stream::wrappers::ReadDirStream;

fn pick_best_match(search_results: &[SingleFileResult]) -> Result<Option<&SingleFileResult>> {
    // TODO: use ollama to rank
    Ok(search_results.first())
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
    let best_match = pick_best_match(&soulseek_search_results)?;
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
