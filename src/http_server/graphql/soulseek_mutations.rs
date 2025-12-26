use std::collections::HashMap;
use std::sync::Arc;

use async_graphql::{Context, Object, SimpleObject};

use crate::http_server::graphql_error::GraphqlResult;
use crate::http_server::state::AppState;
use crate::soulseek::{FileAttribute, SingleFileResult, Track};

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

pub struct Mutation;

#[Object]
impl Mutation {
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

        // Acquire lock on SoulSeek client context
        let mut context = app_state.soulseek_context.lock().await;

        // Perform search
        let results = crate::soulseek::search_for_track(&track, &mut context)
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
        size: i64,
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
            size: size as u64,
            slots_free: true,
            avg_speed: 0.0,
            queue_length: 0,
            attrs: HashMap::new(),
        };

        // Acquire lock on SoulSeek client context
        let context = app_state.soulseek_context.lock().await;

        // Initiate download
        match crate::soulseek::download_file(&result, &app_state.download_directory, &context).await
        {
            Ok(receiver) => {
                for status in receiver {
                    match status {
                        soulseek_rs::DownloadStatus::Queued => {
                            log::info!("Download queued: {}", result.filename);
                        }
                        soulseek_rs::DownloadStatus::InProgress {
                            bytes_downloaded,
                            total_bytes,
                            speed_bytes_per_sec: _,
                        } => {
                            log::info!(
                                "Download in progress: {} ({} bytes downloaded, {} bytes total)",
                                result.filename,
                                bytes_downloaded,
                                total_bytes
                            );
                        }
                        soulseek_rs::DownloadStatus::Completed => {
                            log::info!("Download completed: {}", result.filename);
                            return Ok(DownloadStatus {
                                success: true,
                                message: format!("Download completed: {}", result.filename),
                            });
                        }
                        soulseek_rs::DownloadStatus::Failed => {
                            log::error!("Download failed: {}", result.filename);
                            return Err(color_eyre::eyre::eyre!(
                                "Download failed: {}",
                                result.filename
                            )
                            .into());
                        }
                        soulseek_rs::DownloadStatus::TimedOut => {
                            log::error!("Download timed out: {}", result.filename);
                            return Err(color_eyre::eyre::eyre!(
                                "Download timed out: {}",
                                result.filename
                            )
                            .into());
                        }
                    }
                }
                Err(color_eyre::eyre::eyre!("Download failed: {}", result.filename).into())
            }
            Err(e) => {
                log::error!("SoulSeek download error: {}", e);
                Ok(DownloadStatus {
                    success: false,
                    message: format!("Download failed: {}", e),
                })
            }
        }
    }
}
