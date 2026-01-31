use crate::{database::Database, entities, services};
use chrono::{DateTime, Utc};
use color_eyre::eyre::{Context, Result};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectOptions, Database as SeaDatabase,
    DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect, Set,
};
use tracing::instrument;

pub async fn add_new_videos_for_subscription(
    db: &Database,
    subscription: &entities::youtube_subscription::Model,
) -> Result<()> {
    let feed = services::youtube::feed::fetch_feed(&subscription.youtube_id).await?;
    for entry in feed.entries {
        if entities::youtube_video::Entity::find()
            .filter(entities::youtube_video::Column::YoutubeId.eq(&entry.id))
            .one(&db.conn)
            .await?
            .is_some()
        {
            continue;
        }

        tracing::info!(
            entry = ?entry,
            "Adding new youtube video",
        );

        if let Some(published_at) = entry.published.parse::<DateTime<Utc>>().ok() {
            let video = entities::youtube_video::ActiveModel {
                youtube_id: Set(entry.id),
                title: Set(entry.title),
                channel_name: Set(entry.author.name),
                published_at: Set(published_at),
                thumbnail_url: Set(entry.media_group.thumbnail.url),
                video_url: Set(entry.link.href),
                watched: Set(false),
                ..Default::default()
            };
            entities::youtube_video::Entity::insert(video)
                .exec(&db.conn)
                .await
                .wrap_err("Failed to insert youtube video")?;
        }
    }

    Ok(())
}

#[instrument(skip(db))]
pub async fn add_new_videos(db: &Database) -> Result<()> {
    let subscriptions = entities::youtube_subscription::Entity::find()
        .all(&db.conn)
        .await
        .wrap_err("Failed to fetch youtube subscriptions")?;
    for subscription in subscriptions {
        match add_new_videos_for_subscription(&db, &subscription).await {
            Ok(_) => {}
            Err(e) => tracing::error!(
                subscription = ?subscription,
                error = ?e,
                "Failed to add new videos for subscription",
            ),
        }
    }

    Ok(())
}
