use std::sync::Arc;
use tracing::{self, Instrument, instrument};

use crate::services::spotify::matching_local_tracks::matcher::{
    DurationMatch, MatchConfidence, VersionMatch,
};
use crate::services::spotify::matching_local_tracks::similarity_filter::match_spotify_track_to_local_track;
use crate::services::spotify::matching_local_tracks::task_db::{
    create_spotify_to_local_matcher_task, mark_spotify_to_local_matcher_task_as_completed,
    mark_spotify_to_local_matcher_task_as_failed,
    mark_spotify_to_local_matcher_task_as_in_progress, update_spotify_to_local_matcher_task,
};
use crate::{database::Database, entities};
use color_eyre::eyre::{OptionExt, Result};
use sea_orm::ActiveModelBehavior;
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

fn confidence_to_candidate(
    confidence: &MatchConfidence,
) -> Option<entities::spotify_match_candidate::CandidateConfidence> {
    match confidence {
        MatchConfidence::High => Some(entities::spotify_match_candidate::CandidateConfidence::High),
        MatchConfidence::Medium => {
            Some(entities::spotify_match_candidate::CandidateConfidence::Medium)
        }
        MatchConfidence::Low => Some(entities::spotify_match_candidate::CandidateConfidence::Low),
        MatchConfidence::NoMatch => None,
    }
}

fn duration_match_to_candidate(
    dm: &DurationMatch,
) -> entities::spotify_match_candidate::CandidateDurationMatch {
    match dm {
        DurationMatch::Exact => entities::spotify_match_candidate::CandidateDurationMatch::Exact,
        DurationMatch::Close => entities::spotify_match_candidate::CandidateDurationMatch::Close,
        DurationMatch::Mismatch => {
            entities::spotify_match_candidate::CandidateDurationMatch::Mismatch
        }
    }
}

fn version_match_to_candidate(
    vm: &VersionMatch,
) -> entities::spotify_match_candidate::CandidateVersionMatch {
    match vm {
        VersionMatch::Match => entities::spotify_match_candidate::CandidateVersionMatch::Match,
        VersionMatch::Mismatch => {
            entities::spotify_match_candidate::CandidateVersionMatch::Mismatch
        }
        VersionMatch::Ambiguous => {
            entities::spotify_match_candidate::CandidateVersionMatch::Ambiguous
        }
    }
}

async fn store_match_candidates(
    db: &Database,
    spotify_track: &entities::spotify_track::Model,
    candidates: &[(
        entities::track::Model,
        crate::services::spotify::matching_local_tracks::matcher::MatchResult,
    )],
) -> Result<()> {
    // Delete existing pending candidates for this spotify track
    entities::spotify_match_candidate::Entity::delete_many()
        .filter(
            entities::spotify_match_candidate::Column::SpotifyTrackId
                .eq(&spotify_track.spotify_track_id),
        )
        .filter(
            entities::spotify_match_candidate::Column::Status
                .eq(entities::spotify_match_candidate::CandidateStatus::Pending),
        )
        .exec(&db.conn)
        .await?;

    // Store top 5 candidates
    for (local_track, match_result) in candidates.iter().take(5) {
        let candidate_confidence = match confidence_to_candidate(&match_result.confidence) {
            Some(c) => c,
            None => continue,
        };

        let candidate = entities::spotify_match_candidate::ActiveModel {
            spotify_track_id: Set(spotify_track.spotify_track_id.clone()),
            local_track_id: Set(local_track.id),
            score: Set(match_result.score),
            confidence: Set(candidate_confidence),
            title_similarity: Set(match_result.title_similarity),
            artist_similarity: Set(match_result.artist_similarity),
            album_similarity: Set(match_result.album_similarity),
            duration_match: Set(duration_match_to_candidate(&match_result.duration_match)),
            version_match: Set(version_match_to_candidate(&match_result.version_match)),
            ..entities::spotify_match_candidate::ActiveModel::new()
        };

        candidate.insert(&db.conn).await?;
    }

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
            tracing::warn!(
                spotify_track = ?spotify_track,
                best_local_match = ?best_local_match,
                "No high-confidence match found for spotify track, storing candidates",
            );

            // Store non-high-confidence candidates for manual review
            if !matches.is_empty()
                && let Err(e) = store_match_candidates(db, spotify_track, &matches).await
            {
                tracing::error!(error = ?e, "Failed to store match candidates");
            }

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
    let unmatched_spotify_tracks_count = spotify_tracks
        .iter()
        .filter(|&spotify_track| !is_spotify_track_already_matched(spotify_track))
        .count() as i64;

    let task = create_spotify_to_local_matcher_task(&db, unmatched_spotify_tracks_count).await?;
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
