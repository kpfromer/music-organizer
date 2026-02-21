use async_graphql::{Context, Object};

use crate::http_server::{graphql::context::get_app_state, graphql_error::GraphqlResult};
use crate::services::youtube::service::YoutubeService;

#[derive(Default)]
pub struct YoutubeMutation;

#[Object]
impl YoutubeMutation {
    async fn add_youtube_subscription(
        &self,
        ctx: &Context<'_>,
        name: String,
    ) -> GraphqlResult<bool> {
        let app_state = get_app_state(ctx)?;
        let service = YoutubeService::new(app_state.db.clone());
        service.add_subscription(name).await?;
        Ok(true)
    }

    async fn remove_youtube_subscription(&self, ctx: &Context<'_>, id: i64) -> GraphqlResult<bool> {
        let app_state = get_app_state(ctx)?;
        let service = YoutubeService::new(app_state.db.clone());
        service.remove_subscription(id).await?;
        Ok(true)
    }

    async fn mark_youtube_video_as_unwatched(
        &self,
        ctx: &Context<'_>,
        id: i64,
    ) -> GraphqlResult<bool> {
        let app_state = get_app_state(ctx)?;
        let service = YoutubeService::new(app_state.db.clone());
        service.set_video_watched(id, false).await?;
        Ok(true)
    }

    async fn mark_youtube_video_as_watched(
        &self,
        ctx: &Context<'_>,
        id: i64,
    ) -> GraphqlResult<bool> {
        let app_state = get_app_state(ctx)?;
        let service = YoutubeService::new(app_state.db.clone());
        service.set_video_watched(id, true).await?;
        Ok(true)
    }
}
