use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use crate::config::Config;
use crate::database::Database;
use crate::import_track;
use song_rs::{Client as SongDownloader, SongQuery, SongResult, WantedFileTypes};

pub struct SoulseekService {
    db: Arc<Database>,
    song_downloader: SongDownloader,
    download_directory: PathBuf,
    api_key: String,
    config: Config,
}

impl SoulseekService {
    pub fn new(
        db: Arc<Database>,
        song_downloader: SongDownloader,
        download_directory: PathBuf,
        api_key: String,
        config: Config,
    ) -> Self {
        Self {
            db,
            song_downloader,
            download_directory,
            api_key,
            config,
        }
    }

    pub async fn search(&self, query: &SongQuery) -> color_eyre::Result<Vec<SongResult>> {
        self.song_downloader
            .search(query, Duration::from_secs(120), &WantedFileTypes::all())
            .await
            .map_err(|e| {
                tracing::error!("SoulSeek search error: {}", e);
                color_eyre::eyre::eyre!("SoulSeek search failed: {}", e)
            })
    }

    pub async fn download_and_import(&self, result: &SongResult) -> color_eyre::Result<String> {
        let download_dir = self
            .download_directory
            .as_os_str()
            .to_str()
            .ok_or_else(|| color_eyre::eyre::eyre!("Download directory is not valid UTF-8"))?
            .to_string();

        let (_download, mut receiver) =
            { self.song_downloader.download(result, &download_dir).await }.map_err(|e| {
                tracing::error!("SoulSeek download error: {}", e);
                color_eyre::eyre::eyre!("SoulSeek download failed: {}", e)
            })?;

        let filename = result.filename.filename().to_string();

        while let Some(status) = receiver.recv().await {
            match status {
                song_rs::DownloadStatus::Queued => {
                    tracing::info!("Download queued: {}", filename);
                }
                song_rs::DownloadStatus::InProgress {
                    bytes_downloaded,
                    total_bytes,
                    ..
                } => {
                    tracing::info!(
                        "Download in progress: {} ({} bytes downloaded, {} bytes total)",
                        filename,
                        bytes_downloaded,
                        total_bytes
                    );
                }
                song_rs::DownloadStatus::Completed => {
                    tracing::info!("Download completed: {}", filename);
                    break;
                }
                song_rs::DownloadStatus::Failed => {
                    tracing::error!("Download failed: {}", filename);
                    return Err(color_eyre::eyre::eyre!("Download failed: {}", filename));
                }
                song_rs::DownloadStatus::TimedOut => {
                    tracing::error!("Download timed out: {}", filename);
                    return Err(color_eyre::eyre::eyre!("Download timed out: {}", filename));
                }
            }
        }

        let file_path = self.download_directory.join(&filename);
        if file_path.exists() {
            import_track::import_track(&file_path, &self.api_key, &self.config, &self.db)
                .await
                .map_err(|e| color_eyre::eyre::eyre!("Failed to import track: {}", e))?;
        }

        Ok(format!("Download completed: {}", filename))
    }
}
