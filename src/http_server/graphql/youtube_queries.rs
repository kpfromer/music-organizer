use async_graphql::{Context, Object};
use chrono::{DateTime, Utc};
use tracing::{info, instrument};

use crate::http_server::graphql::context::get_app_state;
use crate::http_server::graphql_error::GraphqlResult;
use crate::{entities, services};
use color_eyre::eyre::WrapErr;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::QueryOrder;

#[derive(Default)]
pub struct YoutubeQuery;

#[derive(async_graphql::SimpleObject, Debug)]
pub struct YoutubeSubscription {
    pub id: i64,
    pub name: String,
    pub youtube_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(async_graphql::SimpleObject, Debug)]
pub struct Video {
    pub id: i64,
    pub title: String,
    pub channel_name: String,
    pub published_at: Option<DateTime<Utc>>,
    pub thumbnail_url: String,
    pub video_url: String,
    pub watched: bool,
}

#[Object]
impl YoutubeQuery {
    #[instrument(skip(self, ctx))]
    async fn youtube_subscriptions(
        &self,
        ctx: &Context<'_>,
    ) -> GraphqlResult<Vec<YoutubeSubscription>> {
        let app_state = get_app_state(ctx)?;
        let db = &app_state.db;
        let subscriptions = entities::youtube_subscription::Entity::find()
            .order_by_asc(entities::youtube_subscription::Column::Name)
            .all(&db.conn)
            .await
            .wrap_err("Failed to fetch youtube subscriptions")?;
        Ok(subscriptions
            .into_iter()
            .map(|subscription| YoutubeSubscription {
                id: subscription.id,
                name: subscription.name,
                youtube_id: subscription.youtube_id,
                created_at: subscription.created_at,
                updated_at: subscription.updated_at,
            })
            .collect())
    }
    /// Get all videos from subscribed channels
    /// Cache for 3 minutes
    #[graphql(cache_control(max_age = 180))]
    #[instrument(skip(self, ctx))]
    async fn youtube_videos(
        &self,
        ctx: &Context<'_>,
        watched: Option<bool>,
    ) -> GraphqlResult<Vec<Video>> {
        let db = &get_app_state(ctx)?.db;
        // TODO: get subscriptions from db
        // TODO: filter for watched/unwatched videos from db
        // TODO: don't use cache, use db for youtube videos

        let mut qs = entities::youtube_video::Entity::find()
            .order_by_desc(entities::youtube_video::Column::PublishedAt);
        if let Some(watched) = watched {
            qs = qs.filter(entities::youtube_video::Column::Watched.eq(watched));
        }

        let videos = qs
            .all(&db.conn)
            .await
            .wrap_err("Failed to fetch youtube videos")?;

        info!(videos = ?videos, "Found videos");
        Ok(videos
            .into_iter()
            .map(|video| Video {
                id: video.id,
                title: video.title,
                channel_name: video.channel_name,
                published_at: Some(video.published_at),
                thumbnail_url: video.thumbnail_url,
                video_url: video.video_url,
                watched: video.watched,
            })
            .collect())
    }
}
