use std::sync::Arc;

use super::llm_reranker::pick_best_local_match;
use super::similarity_filter::filter_for_best_local_matches;
use crate::{database::Database, entities};
use color_eyre::eyre::{Context, OptionExt, Result};
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

async fn match_existing_spotify_tracks_with_local(
    db: &Database,
    spotify_tracks: Vec<entities::spotify_track::Model>,
) -> Result<()> {
    let unmatched_spotify_tracks = spotify_tracks
        .into_iter()
        .filter(|spotify_track| !is_spotify_track_already_matched(spotify_track))
        .collect::<Vec<_>>();

    let values = filter_for_best_local_matches(db, &unmatched_spotify_tracks[..]).await?;
    log::debug!("Best local matches: {:?}", values);
    for (spotify_track, local_tracks) in values {
        let best_local_match = pick_best_local_match(spotify_track, local_tracks).await?;

        if let Some(best_local_match) = best_local_match {
            let best_local_match = entities::track::Entity::find()
                .filter(entities::track::Column::Id.eq(best_local_match))
                .one(&db.conn)
                .await?
                .ok_or_eyre("No local track found for best local match")?;

            log::info!(
                "Best local match found for spotify track: {:?}",
                best_local_match
            );

            update_database_spotify_track_with_local_track(db, spotify_track, &best_local_match)
                .await?;
        } else {
            log::error!(
                "No best local match found for spotify track: {:?}",
                spotify_track
            );
            // TODO: log a failure in db?
        }
    }

    Ok(())
}

pub async fn match_existing_spotify_tracks_with_local_task(
    db: Arc<Database>,
    spotify_tracks: Vec<entities::spotify_track::Model>,
) -> Result<()> {
    tokio::task::spawn(async move {
        match match_existing_spotify_tracks_with_local(&db, spotify_tracks).await {
            Ok(()) => {
                log::info!("Successfully matched existing spotify tracks with local");
            }
            Err(e) => {
                log::error!(
                    "Failed to match existing spotify tracks with local: {:?}",
                    e
                );
                // TODO: log a failure in db?
            }
        }
    });
    Ok(())
}
