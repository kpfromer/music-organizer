use async_graphql::{Context, Object};
use color_eyre::eyre::WrapErr;

use crate::{
    entities,
    http_server::{graphql::context::get_app_state, graphql_error::GraphqlResult},
    services,
};
use sea_orm::Set;
use sea_orm::{ActiveModelTrait, EntityTrait};

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
        let db = &app_state.db;
        let youtube_id = services::youtube::feed::get_channel_id(&name).await?;
        if let Some(youtube_id) = youtube_id {
            let subscription = entities::youtube_subscription::ActiveModel {
                name: Set(name),
                youtube_id: Set(youtube_id),
                ..Default::default()
            };
            let subscription = subscription
                .insert(&db.conn)
                .await
                .wrap_err("Failed to add youtube subscription")?;
            services::background::youtube::add_new_videos_for_subscription(
                &app_state.db,
                &subscription,
            )
            .await?;
            Ok(true)
        } else {
            Err(color_eyre::eyre::eyre!("Failed to get youtube channel id").into())
        }
    }

    async fn remove_youtube_subscription(&self, ctx: &Context<'_>, id: i64) -> GraphqlResult<bool> {
        let app_state = get_app_state(ctx)?;
        let db = &app_state.db;
        entities::youtube_subscription::Entity::delete_by_id(id)
            .exec(&db.conn)
            .await
            .wrap_err("Failed to remove youtube subscription")?;
        Ok(true)
    }

    async fn mark_youtube_video_as_watched(
        &self,
        ctx: &Context<'_>,
        id: i64,
    ) -> GraphqlResult<bool> {
        let app_state = get_app_state(ctx)?;
        let db = &app_state.db;
        let video = entities::youtube_video::Entity::find_by_id(id)
            .one(&db.conn)
            .await
            .wrap_err("Failed to find youtube video")?;
        if let Some(video) = video {
            let mut video: entities::youtube_video::ActiveModel = video.into();
            video.watched = Set(true);
            video
                .update(&db.conn)
                .await
                .wrap_err("Failed to mark youtube video as watched")?;
            Ok(true)
        } else {
            Err(color_eyre::eyre::eyre!("Youtube video not found").into())
        }
    }
}
