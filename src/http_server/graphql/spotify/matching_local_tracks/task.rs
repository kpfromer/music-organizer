use std::sync::Arc;
use tracing::{self, Instrument, instrument};

use crate::http_server::graphql::spotify::matching_local_tracks::matcher::MatchConfidence;
use crate::http_server::graphql::spotify::matching_local_tracks::similarity_filter::match_spotify_track_to_local_track;
use crate::http_server::graphql::spotify::matching_local_tracks::task_db::{
    create_spotify_to_local_matcher_task, mark_spotify_to_local_matcher_task_as_completed,
    mark_spotify_to_local_matcher_task_as_failed,
    mark_spotify_to_local_matcher_task_as_in_progress, update_spotify_to_local_matcher_task,
};
use crate::{database::Database, entities};
use color_eyre::eyre::{OptionExt, Result};
use sea_orm::ActiveModelTrait;
use sea_orm::{ColumnTrait, EntityTrait};
use sea_orm::{QueryFilter, Set};

fn is_spotify_track_already_matched(spotify_track: &entities::spotify_track::Model) -> bool {
    spotify_track.local_track_id.is_some()
}

async fn update_database_spotify_track_with_local_track(
    db: &Database,
    spotify_track: &entities::spotify_track::Model,
    local_track: &entities::track::Model,
) -> Result<()> {
    if spotify_track.local_track_id.is_some() {
        // This should not happen, but if it does, we should log a failure.
        return Err(color_eyre::eyre::eyre!(
            "Spotify track already has a local track"
        ));
    }
    let mut spotify_track: entities::spotify_track::ActiveModel = spotify_track.clone().into();
    spotify_track.local_track_id = Set(Some(local_track.id));
    spotify_track.update(&db.conn).await?;
    Ok(())
}

#[instrument(skip(db, task, spotify_tracks), fields(num_spotify_tracks = ?spotify_tracks.len()))]
async fn match_existing_spotify_tracks_with_local(
    db: &Database,
    task: &entities::spotify_to_local_matcher_tasks::Model,
    spotify_tracks: Vec<entities::spotify_track::Model>,
) -> Result<()> {
    let unmatched_spotify_tracks = spotify_tracks
        .into_iter()
        .filter(|spotify_track| !is_spotify_track_already_matched(spotify_track))
        .collect::<Vec<_>>();
    let local_tracks = entities::track::Entity::find().all(&db.conn).await?;

    let matches =
        match_spotify_track_to_local_track(db, &unmatched_spotify_tracks[..], &local_tracks[..])
            .await?;
    tracing::debug!("Best local matches: {:?}", matches);
    let mut matched_tracks = 0;
    let mut failed_tracks = 0;
    for (spotify_track, matches) in matches {
        let best_local_match = matches.first();

        if let Some(best_local_match) = best_local_match
            && matches!(best_local_match.1.confidence, MatchConfidence::High)
        {
            let best_local_match = entities::track::Entity::find()
                .filter(entities::track::Column::Id.eq(best_local_match.0.id))
                .one(&db.conn)
                .await?
                .ok_or_eyre("No local track found for best local match")?;

            tracing::info!(
                "Best local match found for spotify track: {:?}",
                best_local_match
            );

            update_database_spotify_track_with_local_track(db, spotify_track, &best_local_match)
                .await?;
            matched_tracks += 1;
        } else {
            tracing::error!(
                spotify_track = ?spotify_track,
                best_local_match = ?best_local_match,
                "No best local match found for spotify track",
            );
            failed_tracks += 1;
        }
        if let Err(e) =
            update_spotify_to_local_matcher_task(db, task, matched_tracks, failed_tracks).await
        {
            tracing::error!(error = ?e, "Failed to update spotify to local matcher task");
        }
    }

    Ok(())
}

#[instrument(skip(db, spotify_tracks), fields(num_spotify_tracks = ?spotify_tracks.len()))]
pub async fn match_existing_spotify_tracks_with_local_task(
    db: Arc<Database>,
    spotify_tracks: Vec<entities::spotify_track::Model>,
) -> Result<entities::spotify_to_local_matcher_tasks::Model> {
    let task = create_spotify_to_local_matcher_task(&db, spotify_tracks.len() as i64).await?;
    let task_clone = task.clone();
    tokio::task::spawn(async move {
        if let Err(e) = mark_spotify_to_local_matcher_task_as_in_progress(&db, &task_clone).await {
            tracing::error!(error = ?e, "Failed to mark spotify to local matcher task as in progress");
        }

        match match_existing_spotify_tracks_with_local(&db, &task_clone, spotify_tracks).await {
            Ok(()) => {
                tracing::info!("Successfully matched existing spotify tracks with local");
                if let Err(e) =
                    mark_spotify_to_local_matcher_task_as_completed(&db, &task_clone).await
                {
                    tracing::error!(error = ?e, "Failed to mark spotify to local matcher task as completed");
                }
            }
            Err(e) => {
                if let Err(e) =
                    mark_spotify_to_local_matcher_task_as_failed(&db, &task_clone, e.to_string())
                        .await
                {
                    tracing::error!(error = ?e, "Failed to mark spotify to local matcher task as failed");
                }
                tracing::error!(
                    "Failed to match existing spotify tracks with local: {:?}",
                    e
                );
            }
        }
    }.in_current_span());
    Ok(task)
}
