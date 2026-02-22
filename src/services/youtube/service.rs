use std::sync::Arc;

use color_eyre::eyre::WrapErr;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};

use crate::database::Database;
use crate::entities;
use crate::services;

pub struct YoutubeService {
    db: Arc<Database>,
}

impl YoutubeService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub async fn add_subscription(
        &self,
        name: String,
    ) -> color_eyre::Result<youtube_subscription::Model> {
        let youtube_id = services::youtube::feed::get_channel_id(&name).await?;
        let youtube_id = youtube_id
            .ok_or_else(|| color_eyre::eyre::eyre!("Failed to get youtube channel id"))?;

        let subscription = entities::youtube_subscription::ActiveModel {
            name: Set(name),
            youtube_id: Set(youtube_id),
            ..Default::default()
        };
        let subscription = subscription
            .insert(&self.db.conn)
            .await
            .wrap_err("Failed to add youtube subscription")?;

        services::background::youtube::add_new_videos_for_subscription(&self.db, &subscription)
            .await?;

        Ok(subscription)
    }

    pub async fn remove_subscription(&self, id: i64) -> color_eyre::Result<()> {
        entities::youtube_subscription::Entity::delete_by_id(id)
            .exec(&self.db.conn)
            .await
            .wrap_err("Failed to remove youtube subscription")?;
        Ok(())
    }

    pub async fn set_video_watched(&self, id: i64, watched: bool) -> color_eyre::Result<()> {
        let video = entities::youtube_video::Entity::find_by_id(id)
            .one(&self.db.conn)
            .await
            .wrap_err("Failed to find youtube video")?;

        let video = video.ok_or_else(|| color_eyre::eyre::eyre!("Youtube video not found"))?;

        let mut video: entities::youtube_video::ActiveModel = video.into();
        video.watched = Set(watched);
        video
            .update(&self.db.conn)
            .await
            .wrap_err("Failed to update youtube video watched status")?;

        Ok(())
    }

    pub async fn list_subscriptions(
        &self,
    ) -> color_eyre::Result<Vec<entities::youtube_subscription::Model>> {
        let subscriptions = entities::youtube_subscription::Entity::find()
            .order_by_asc(entities::youtube_subscription::Column::Name)
            .all(&self.db.conn)
            .await
            .wrap_err("Failed to fetch youtube subscriptions")?;
        Ok(subscriptions)
    }

    pub async fn list_videos(
        &self,
        watched: Option<bool>,
    ) -> color_eyre::Result<Vec<entities::youtube_video::Model>> {
        let mut query = entities::youtube_video::Entity::find()
            .order_by_desc(entities::youtube_video::Column::PublishedAt);
        if let Some(watched) = watched {
            query = query.filter(entities::youtube_video::Column::Watched.eq(watched));
        }

        let videos = query
            .all(&self.db.conn)
            .await
            .wrap_err("Failed to fetch youtube videos")?;
        Ok(videos)
    }
}

use crate::entities::youtube_subscription;
