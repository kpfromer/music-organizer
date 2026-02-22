use std::sync::Arc;

use color_eyre::eyre::{OptionExt, Result, WrapErr};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};

use crate::database::Database;
use crate::entities;
use crate::services::track::{PaginatedResult, TrackService, TrackWithRelations};

pub struct MatchedTrackResult {
    pub spotify_track: entities::spotify_track::Model,
    pub local_track: TrackWithRelations,
}

pub struct MatchCandidateWithTrack {
    pub candidate: entities::spotify_match_candidate::Model,
    pub local_track: TrackWithRelations,
}

pub struct UnmatchedTrackWithCandidates {
    pub spotify_track: entities::spotify_track::Model,
    pub candidates: Vec<MatchCandidateWithTrack>,
}

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

    pub async fn list_matched_tracks(
        &self,
        search: Option<&str>,
        page: usize,
        page_size: usize,
    ) -> Result<PaginatedResult<MatchedTrackResult>> {
        let track_service = TrackService::new(self.db.clone());

        let mut base_condition =
            Condition::all().add(entities::spotify_track::Column::LocalTrackId.is_not_null());

        if let Some(search_term) = search
            && !search_term.is_empty()
        {
            let search_condition = Condition::any()
                .add(entities::spotify_track::Column::Title.contains(search_term))
                .add(entities::spotify_track::Column::Album.contains(search_term));
            base_condition = base_condition.add(search_condition);
        }

        let total_count = entities::spotify_track::Entity::find()
            .filter(base_condition.clone())
            .count(&self.db.conn)
            .await
            .wrap_err("Failed to count matched spotify tracks")?;

        let offset = (page.saturating_sub(1)) * page_size;
        let spotify_tracks = entities::spotify_track::Entity::find()
            .filter(base_condition)
            .limit(page_size as u64)
            .offset(offset as u64)
            .order_by_desc(entities::spotify_track::Column::UpdatedAt)
            .all(&self.db.conn)
            .await
            .wrap_err("Failed to fetch matched spotify tracks")?;

        let mut items = Vec::new();
        for spotify_track in spotify_tracks {
            let local_track_id = spotify_track
                .local_track_id
                .ok_or_eyre("Spotify track should have local_track_id")?;
            let local_track = track_service.get_track_by_id(local_track_id).await?;
            items.push(MatchedTrackResult {
                spotify_track,
                local_track,
            });
        }

        Ok(PaginatedResult {
            items,
            total_count,
            page,
            page_size,
        })
    }

    pub async fn list_unmatched_tracks(
        &self,
        search: Option<&str>,
        page: usize,
        page_size: usize,
    ) -> Result<PaginatedResult<UnmatchedTrackWithCandidates>> {
        let track_service = TrackService::new(self.db.clone());

        let mut base_condition =
            Condition::all().add(entities::spotify_track::Column::LocalTrackId.is_null());

        if let Some(search_term) = search
            && !search_term.is_empty()
        {
            let search_condition = Condition::any()
                .add(entities::spotify_track::Column::Title.contains(search_term))
                .add(entities::spotify_track::Column::Album.contains(search_term));
            base_condition = base_condition.add(search_condition);
        }

        let total_count = entities::spotify_track::Entity::find()
            .filter(base_condition.clone())
            .count(&self.db.conn)
            .await
            .wrap_err("Failed to count unmatched spotify tracks")?;

        let offset = (page.saturating_sub(1)) * page_size;
        let spotify_tracks = entities::spotify_track::Entity::find()
            .filter(base_condition)
            .limit(page_size as u64)
            .offset(offset as u64)
            .order_by_desc(entities::spotify_track::Column::UpdatedAt)
            .all(&self.db.conn)
            .await
            .wrap_err("Failed to fetch unmatched spotify tracks")?;

        let mut items = Vec::new();
        for spotify_track in spotify_tracks {
            let candidate_models = entities::spotify_match_candidate::Entity::find()
                .filter(
                    entities::spotify_match_candidate::Column::SpotifyTrackId
                        .eq(&spotify_track.spotify_track_id),
                )
                .filter(
                    entities::spotify_match_candidate::Column::Status
                        .eq(entities::spotify_match_candidate::CandidateStatus::Pending),
                )
                .order_by_desc(entities::spotify_match_candidate::Column::Score)
                .all(&self.db.conn)
                .await
                .wrap_err("Failed to fetch match candidates")?;

            let mut candidates = Vec::new();
            for candidate in candidate_models {
                let local_track = track_service
                    .get_track_by_id(candidate.local_track_id)
                    .await?;
                candidates.push(MatchCandidateWithTrack {
                    candidate,
                    local_track,
                });
            }

            items.push(UnmatchedTrackWithCandidates {
                spotify_track,
                candidates,
            });
        }

        Ok(PaginatedResult {
            items,
            total_count,
            page,
            page_size,
        })
    }

    pub async fn search_local_tracks(
        &self,
        search: &str,
        page: usize,
        page_size: usize,
    ) -> Result<PaginatedResult<TrackWithRelations>> {
        let track_service = TrackService::new(self.db.clone());

        let search_condition =
            Condition::any().add(entities::track::Column::Title.contains(search));

        let total_count = entities::track::Entity::find()
            .filter(search_condition.clone())
            .count(&self.db.conn)
            .await
            .wrap_err("Failed to count local tracks")?;

        let offset = (page.saturating_sub(1)) * page_size;
        let track_models = entities::track::Entity::find()
            .filter(search_condition)
            .limit(page_size as u64)
            .offset(offset as u64)
            .order_by_asc(entities::track::Column::Title)
            .all(&self.db.conn)
            .await
            .wrap_err("Failed to fetch local tracks")?;

        let mut items = Vec::new();
        for track_model in track_models {
            items.push(track_service.get_track_by_id(track_model.id).await?);
        }

        Ok(PaginatedResult {
            items,
            total_count,
            page,
            page_size,
        })
    }

    pub async fn list_unmatched_spotify_tracks(
        &self,
    ) -> Result<Vec<entities::spotify_track::Model>> {
        entities::spotify_track::Entity::find()
            .filter(entities::spotify_track::Column::LocalTrackId.is_null())
            .all(&self.db.conn)
            .await
            .wrap_err("Failed to fetch unmatched spotify tracks")
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
