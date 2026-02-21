use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::config::Config;
use crate::database::Database;
use crate::import_track;
use crate::soulseek::{SingleFileResult, SoulSeekClientContext, Track};

pub struct SoulseekService {
    db: Arc<Database>,
    soulseek_context: Arc<SoulSeekClientContext>,
    download_directory: PathBuf,
    api_key: String,
    config: Config,
}

impl SoulseekService {
    pub fn new(
        db: Arc<Database>,
        soulseek_context: Arc<SoulSeekClientContext>,
        download_directory: PathBuf,
        api_key: String,
        config: Config,
    ) -> Self {
        Self {
            db,
            soulseek_context,
            download_directory,
            api_key,
            config,
        }
    }

    pub async fn search(&self, track: &Track) -> color_eyre::Result<Vec<SingleFileResult>> {
        self.soulseek_context
            .search_for_track(track)
            .await
            .map_err(|e| {
                tracing::error!("SoulSeek search error: {}", e);
                color_eyre::eyre::eyre!("SoulSeek search failed: {}", e)
            })
    }

    pub async fn download_and_import(
        &self,
        file_result: &SingleFileResult,
    ) -> color_eyre::Result<String> {
        let mut receiver = self
            .soulseek_context
            .download_file(file_result, &self.download_directory)
            .await?;

        let filename = file_result.filename.clone();
        let soulseek_context = self.soulseek_context.clone();

        // Download state machine
        let completed_filename: String = {
            let filename = filename.clone();
            let result: Result<String, color_eyre::Report> = async {
                while let Some(status) = receiver.recv().await {
                    match status {
                        soulseek_rs::DownloadStatus::Queued => {
                            tracing::info!("Download queued: {}", filename);
                        }
                        soulseek_rs::DownloadStatus::InProgress {
                            bytes_downloaded,
                            total_bytes,
                            speed_bytes_per_sec: _,
                        } => {
                            tracing::info!(
                                "Download in progress: {} ({} bytes downloaded, {} bytes total)",
                                filename,
                                bytes_downloaded,
                                total_bytes
                            );
                        }
                        soulseek_rs::DownloadStatus::Completed => {
                            tracing::info!("Download completed: {}", filename);
                            return Ok(format!("Download completed: {}", filename));
                        }
                        soulseek_rs::DownloadStatus::Failed => {
                            tracing::error!("Download failed: {}", filename);
                            soulseek_context
                                .report_session_error("Download failed")
                                .await;
                            return Err(color_eyre::eyre::eyre!("Download failed: {}", filename));
                        }
                        soulseek_rs::DownloadStatus::TimedOut => {
                            tracing::error!("Download timed out: {}", filename);
                            soulseek_context
                                .report_session_error("Download timed out")
                                .await;
                            return Err(color_eyre::eyre::eyre!(
                                "Download timed out: {}",
                                filename
                            ));
                        }
                    }
                }
                Err(color_eyre::eyre::eyre!("Download failed: {}", filename))
            }
            .await;
            result?
        };

        // Post-download import
        let file_path = Path::new(&filename)
            .file_name()
            .map(|file_name| Path::new(&self.download_directory).join(file_name));

        if let Some(file_path) = file_path {
            import_track::import_track(&file_path, &self.api_key, &self.config, &self.db)
                .await
                .map_err(|e| color_eyre::eyre::eyre!("Failed to import track: {}", e))?;
        }

        Ok(completed_filename)
    }
}
