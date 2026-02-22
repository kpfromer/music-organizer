use std::collections::HashMap;

use async_graphql::{Context, Object, SimpleObject};

use crate::http_server::graphql::context::get_app_state;
use crate::http_server::graphql_error::GraphqlResult;
use crate::services::soulseek_service::SoulseekService;
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
        let app_state = get_app_state(ctx)?;
        let service = SoulseekService::new(
            app_state.db.clone(),
            app_state.soulseek_context.clone(),
            app_state.download_directory.clone(),
            app_state.api_key.clone(),
            app_state.config.clone(),
        );

        let track = Track {
            title: track_title,
            album: album_name.unwrap_or_default(),
            artists: artists.unwrap_or_default(),
            length: duration.map(|d| d as u32),
        };

        let results = service.search(&track).await?;
        Ok(results
            .into_iter()
            .map(SoulSeekSearchResult::from)
            .collect())
    }

    async fn download_soulseek_file(
        &self,
        ctx: &Context<'_>,
        username: String,
        filename: String,
        size: u64,
        token: String,
    ) -> GraphqlResult<DownloadStatus> {
        let app_state = get_app_state(ctx)?;
        let service = SoulseekService::new(
            app_state.db.clone(),
            app_state.soulseek_context.clone(),
            app_state.download_directory.clone(),
            app_state.api_key.clone(),
            app_state.config.clone(),
        );

        let file_result = SingleFileResult {
            username,
            token,
            filename,
            size,
            slots_free: true,
            avg_speed: 0.0,
            queue_length: 0,
            attrs: HashMap::new(),
        };

        let message = service.download_and_import(&file_result).await?;
        Ok(DownloadStatus {
            success: true,
            message,
        })
    }
}
