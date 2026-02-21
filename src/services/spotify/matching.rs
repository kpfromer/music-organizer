use std::sync::Arc;

use color_eyre::eyre::{OptionExt, Result, WrapErr};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

use crate::database::Database;
use crate::entities;

pub struct SpotifyMatchingService {
    db: Arc<Database>,
}

impl SpotifyMatchingService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Accept a match candidate: link the Spotify track to the local track,
    /// mark the candidate as accepted, and dismiss all other pending candidates.
    pub async fn accept_candidate(&self, candidate_id: i64) -> Result<()> {
        let candidate = entities::spotify_match_candidate::Entity::find_by_id(candidate_id)
            .one(&self.db.conn)
            .await
            .wrap_err("Failed to fetch match candidate")?
            .ok_or_eyre("Match candidate not found")?;

        self.link_spotify_to_local(&candidate.spotify_track_id, candidate.local_track_id)
            .await?;

        // Mark the accepted candidate
        let mut candidate_active: entities::spotify_match_candidate::ActiveModel =
            candidate.clone().into();
        candidate_active.status = Set(entities::spotify_match_candidate::CandidateStatus::Accepted);
        candidate_active
            .update(&self.db.conn)
            .await
            .wrap_err("Failed to update match candidate")?;

        self.dismiss_pending_candidates(&candidate.spotify_track_id)
            .await
    }

    /// Manually match a Spotify track to a local track and dismiss pending candidates.
    pub async fn manually_match(&self, spotify_track_id: &str, local_track_id: i64) -> Result<()> {
        self.link_spotify_to_local(spotify_track_id, local_track_id)
            .await?;
        self.dismiss_pending_candidates(spotify_track_id).await
    }

    /// Dismiss all pending candidates for a Spotify track without matching.
    pub async fn dismiss_track(&self, spotify_track_id: &str) -> Result<()> {
        self.dismiss_pending_candidates(spotify_track_id).await
    }

    async fn link_spotify_to_local(
        &self,
        spotify_track_id: &str,
        local_track_id: i64,
    ) -> Result<()> {
        let spotify_track = entities::spotify_track::Entity::find()
            .filter(entities::spotify_track::Column::SpotifyTrackId.eq(spotify_track_id))
            .one(&self.db.conn)
            .await
            .wrap_err("Failed to fetch spotify track")?
            .ok_or_eyre("Spotify track not found")?;

        let mut spotify_track_active: entities::spotify_track::ActiveModel = spotify_track.into();
        spotify_track_active.local_track_id = Set(Some(local_track_id));
        spotify_track_active
            .update(&self.db.conn)
            .await
            .wrap_err("Failed to update spotify track")?;

        Ok(())
    }

    async fn dismiss_pending_candidates(&self, spotify_track_id: &str) -> Result<()> {
        let pending_candidates = entities::spotify_match_candidate::Entity::find()
            .filter(entities::spotify_match_candidate::Column::SpotifyTrackId.eq(spotify_track_id))
            .filter(
                entities::spotify_match_candidate::Column::Status
                    .eq(entities::spotify_match_candidate::CandidateStatus::Pending),
            )
            .all(&self.db.conn)
            .await
            .wrap_err("Failed to fetch pending candidates")?;

        for candidate in pending_candidates {
            let mut candidate_active: entities::spotify_match_candidate::ActiveModel =
                candidate.into();
            candidate_active.status =
                Set(entities::spotify_match_candidate::CandidateStatus::Dismissed);
            candidate_active
                .update(&self.db.conn)
                .await
                .wrap_err("Failed to dismiss candidate")?;
        }

        Ok(())
    }
}
