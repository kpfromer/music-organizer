use crate::{
    database::Database,
    entities::{self, spotify_to_local_matcher_tasks::MatchTaskStatus},
};
use color_eyre::eyre::{Context, Result};
use sea_orm::ActiveModelTrait;
use sea_orm::EntityTrait;
use sea_orm::Set;

pub async fn create_spotify_to_local_matcher_task(
    db: &Database,
    total_tracks: i64,
) -> Result<entities::spotify_to_local_matcher_tasks::Model> {
    let task = entities::spotify_to_local_matcher_tasks::ActiveModel {
        status: Set(MatchTaskStatus::Pending),
        total_tracks: Set(total_tracks),
        matched_tracks: Set(0),
        failed_tracks: Set(0),
        ..Default::default()
    };
    let task = entities::spotify_to_local_matcher_tasks::Entity::insert(task)
        .exec_with_returning(&db.conn)
        .await
        .wrap_err("Failed to create spotify to local matcher task")?;
    Ok(task)
}
pub async fn update_spotify_to_local_matcher_task(
    db: &Database,
    task: &entities::spotify_to_local_matcher_tasks::Model,
    matched_tracks: i64,
    failed_tracks: i64,
) -> Result<()> {
    let mut task: entities::spotify_to_local_matcher_tasks::ActiveModel = task.clone().into();
    task.matched_tracks = Set(matched_tracks);
    task.failed_tracks = Set(failed_tracks);
    task.update(&db.conn).await?;
    Ok(())
}

pub async fn mark_spotify_to_local_matcher_task_as_in_progress(
    db: &Database,
    task: &entities::spotify_to_local_matcher_tasks::Model,
) -> Result<()> {
    let mut task: entities::spotify_to_local_matcher_tasks::ActiveModel = task.clone().into();
    task.status = Set(MatchTaskStatus::InProgress);
    task.update(&db.conn).await?;
    Ok(())
}

pub async fn mark_spotify_to_local_matcher_task_as_failed(
    db: &Database,
    task: &entities::spotify_to_local_matcher_tasks::Model,
    error_message: String,
) -> Result<()> {
    let mut task: entities::spotify_to_local_matcher_tasks::ActiveModel = task.clone().into();
    task.status = Set(MatchTaskStatus::Failed);
    task.error_message = Set(Some(error_message));
    task.update(&db.conn).await?;
    Ok(())
}

pub async fn mark_spotify_to_local_matcher_task_as_completed(
    db: &Database,
    task: &entities::spotify_to_local_matcher_tasks::Model,
) -> Result<()> {
    let mut task: entities::spotify_to_local_matcher_tasks::ActiveModel = task.clone().into();
    task.status = Set(MatchTaskStatus::Completed);
    task.update(&db.conn).await?;
    Ok(())
}
