use std::collections::HashMap;
use std::sync::Arc;

use async_graphql::{Context, Object, SimpleObject};

use crate::http_server::graphql_error::GraphqlResult;
use crate::http_server::state::AppState;
use crate::import_track;
use crate::soulseek::{FileAttribute, SingleFileResult, Track};

use std::path::Path;

#[derive(Debug, Clone, SimpleObject)]
pub struct SoulSeekSearchResult {
    pub username: String,
    pub token: String,
    pub filename: String,
    pub size: u64,
    pub slots_free: bool,
    pub avg_speed: f64,
    pub queue_length: u32,
    pub attributes: Vec<SoulSeekFileAttributeValue>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct SoulSeekFileAttributeValue {
    pub attribute: SoulSeekFileAttribute,
    pub value: u32,
}

#[derive(Debug, Clone, Copy, async_graphql::Enum, PartialEq, Eq)]
pub enum SoulSeekFileAttribute {
    Bitrate,
    Duration,
    VariableBitRate,
    Encoder,
    SampleRate,
    BitDepth,
}

impl From<FileAttribute> for SoulSeekFileAttribute {
    fn from(attr: FileAttribute) -> Self {
        match attr {
            FileAttribute::Bitrate => SoulSeekFileAttribute::Bitrate,
            FileAttribute::Duration => SoulSeekFileAttribute::Duration,
            FileAttribute::VariableBitRate => SoulSeekFileAttribute::VariableBitRate,
            FileAttribute::Encoder => SoulSeekFileAttribute::Encoder,
            FileAttribute::SampleRate => SoulSeekFileAttribute::SampleRate,
            FileAttribute::BitDepth => SoulSeekFileAttribute::BitDepth,
        }
    }
}

#[derive(Debug, Clone, SimpleObject)]
pub struct DownloadStatus {
    pub success: bool,
    pub message: String,
}

impl From<SingleFileResult> for SoulSeekSearchResult {
    fn from(result: SingleFileResult) -> Self {
        let attributes = result
            .attrs
            .into_iter()
            .map(|(attr, value)| SoulSeekFileAttributeValue {
                attribute: attr.into(),
                value,
            })
            .collect();

        SoulSeekSearchResult {
            username: result.username,
            token: result.token,
            filename: result.filename,
            size: result.size,
            slots_free: result.slots_free,
            avg_speed: result.avg_speed,
            queue_length: result.queue_length,
            attributes,
        }
    }
}

#[derive(Default)]
pub struct SoulseekMutation;

#[Object]
impl SoulseekMutation {
    async fn search_soulseek(
        &self,
        ctx: &Context<'_>,
        track_title: String,
        album_name: Option<String>,
        artists: Option<Vec<String>>,
        duration: Option<i32>,
    ) -> GraphqlResult<Vec<SoulSeekSearchResult>> {
        let app_state = ctx
            .data::<Arc<AppState>>()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get app state: {:?}", e))?;

        // Build Track struct from mutation arguments
        let track = Track {
            title: track_title,
            album: album_name.unwrap_or_default(),
            artists: artists.unwrap_or_default(),
            length: duration.map(|d| d as u32),
        };

        // Perform search
        let results = app_state
            .soulseek_context
            .search_for_track(&track)
            .await
            .map_err(|e| {
                log::error!("SoulSeek search error: {}", e);
                color_eyre::eyre::eyre!("SoulSeek search failed: {}", e)
            })?;

        // Convert results to GraphQL types
        let graphql_results: Vec<SoulSeekSearchResult> = results
            .into_iter()
            .map(SoulSeekSearchResult::from)
            .collect();

        Ok(graphql_results)
    }

    async fn download_soulseek_file(
        &self,
        ctx: &Context<'_>,
        username: String,
        filename: String,
        size: u64,
        token: String,
    ) -> GraphqlResult<DownloadStatus> {
        let app_state = ctx
            .data::<Arc<AppState>>()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get app state: {:?}", e))?;

        // Build SingleFileResult from mutation arguments
        let result = SingleFileResult {
            username,
            token,
            filename,
            size,
            slots_free: true,
            avg_speed: 0.0,
            queue_length: 0,
            attrs: HashMap::new(),
        };

        // Initiate download
        let receiver = app_state
            .soulseek_context
            .download_file(&result, &app_state.download_directory)
            .await?;

        let filename = result.filename.clone();
        let filename_for_path = result.filename.clone();

        // Spawn a blocking task to iterate the receiver and return the final result
        let download_result = tokio::task::spawn_blocking(move || {
            for status in receiver {
                match status {
                    soulseek_rs::DownloadStatus::Queued => {
                        log::info!("Download queued: {}", filename);
                    }
                    soulseek_rs::DownloadStatus::InProgress {
                        bytes_downloaded,
                        total_bytes,
                        speed_bytes_per_sec: _,
                    } => {
                        log::info!(
                            "Download in progress: {} ({} bytes downloaded, {} bytes total)",
                            filename,
                            bytes_downloaded,
                            total_bytes
                        );
                    }
                    soulseek_rs::DownloadStatus::Completed => {
                        log::info!("Download completed: {}", filename);
                        return Ok(DownloadStatus {
                            success: true,
                            message: format!("Download completed: {}", filename),
                        });
                    }
                    soulseek_rs::DownloadStatus::Failed => {
                        log::error!("Download failed: {}", filename);
                        return Err(color_eyre::eyre::eyre!("Download failed: {}", filename));
                    }
                    soulseek_rs::DownloadStatus::TimedOut => {
                        log::error!("Download timed out: {}", filename);
                        return Err(color_eyre::eyre::eyre!("Download timed out: {}", filename));
                    }
                }
            }
            // If the loop ends without a terminal status, return an error
            Err(color_eyre::eyre::eyre!("Download failed: {}", filename))
        })
        .await
        .map_err(|e| color_eyre::eyre::eyre!("Blocking task panicked: {}", e))?;

        match download_result {
            Ok(status) => {
                let file_path = Path::new(&filename_for_path)
                    .file_name()
                    .to_owned()
                    .map(|file_name| Path::new(&app_state.download_directory).join(file_name));
                if let Some(file_path) = file_path {
                    import_track::import_track(
                        &file_path,
                        &app_state.api_key,
                        &app_state.config,
                        &app_state.db,
                    )
                    .await
                    .map_err(|e| color_eyre::eyre::eyre!("Failed to import track: {}", e))?;
                }
                Ok(status)
            }
            Err(e) => Err(e.into()),
        }
    }
}
