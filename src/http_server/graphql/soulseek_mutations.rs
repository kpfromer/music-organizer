use async_graphql::{Context, Object, SimpleObject};
use song_rs::{FileType, SongQuery, SongResult};

use crate::http_server::graphql::context::get_app_state;
use crate::http_server::graphql_error::GraphqlResult;
use crate::services::soulseek_service::SoulseekService;

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

#[derive(Debug, Clone, SimpleObject)]
pub struct DownloadStatus {
    pub success: bool,
    pub message: String,
}

impl From<SongResult> for SoulSeekSearchResult {
    fn from(result: SongResult) -> Self {
        let mut attributes = Vec::new();
        if let Some(v) = result.bitrate {
            attributes.push(SoulSeekFileAttributeValue {
                attribute: SoulSeekFileAttribute::Bitrate,
                value: v,
            });
        }
        if let Some(v) = result.duration {
            attributes.push(SoulSeekFileAttributeValue {
                attribute: SoulSeekFileAttribute::Duration,
                value: v,
            });
        }
        if let Some(v) = result.vbr {
            attributes.push(SoulSeekFileAttributeValue {
                attribute: SoulSeekFileAttribute::VariableBitRate,
                value: if v { 1 } else { 0 },
            });
        }
        if let Some(v) = result.sample_rate {
            attributes.push(SoulSeekFileAttributeValue {
                attribute: SoulSeekFileAttribute::SampleRate,
                value: v,
            });
        }
        if let Some(v) = result.bit_depth {
            attributes.push(SoulSeekFileAttributeValue {
                attribute: SoulSeekFileAttribute::BitDepth,
                value: v,
            });
        }

        SoulSeekSearchResult {
            username: result.username,
            token: String::new(),
            filename: result.filename.as_str().to_string(),
            size: result.size,
            slots_free: true,
            avg_speed: 0.0,
            queue_length: 0,
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
            app_state.song_downloader.clone(),
            app_state.download_directory.clone(),
            app_state.api_key.clone(),
            app_state.config.clone(),
        );

        let query = SongQuery {
            title: track_title,
            artist: artists
                .unwrap_or_default()
                .into_iter()
                .next()
                .unwrap_or_default(),
            album: album_name,
            duration_secs: duration.unwrap_or(0) as u32,
        };

        let results = service.search(&query).await?;
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
        _token: String,
    ) -> GraphqlResult<DownloadStatus> {
        let app_state = get_app_state(ctx)?;
        let service = SoulseekService::new(
            app_state.db.clone(),
            app_state.song_downloader.clone(),
            app_state.download_directory.clone(),
            app_state.api_key.clone(),
            app_state.config.clone(),
        );

        let ext = filename.rsplit('.').next_back().unwrap_or("");
        let result = SongResult {
            username: username.clone(),
            filename: filename.clone().into(),
            file_type: FileType::from_extension(ext),
            size,
            bitrate: None,
            duration: None,
            sample_rate: None,
            bit_depth: None,
            vbr: None,
            score: 0.0,
        };

        let message = service.download_and_import(&result).await?;
        Ok(DownloadStatus {
            success: true,
            message,
        })
    }
}
