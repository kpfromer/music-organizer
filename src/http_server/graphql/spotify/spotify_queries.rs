use async_graphql::{Context, Object};
use chrono::{DateTime, Utc};
use color_eyre::eyre::OptionExt;

use crate::http_server::graphql::context::get_app_state;
use crate::http_server::graphql::map_track_with_relations;
use crate::http_server::graphql::track_queries::Track;
use crate::http_server::graphql_error::GraphqlResult;
use crate::services::spotify::matching::SpotifyMatchingService;
use crate::services::spotify::sync::SpotifySyncQueryService;

#[derive(Default)]
pub struct SpotifyQuery;

#[derive(async_graphql::SimpleObject)]
pub struct SpotifyAccount {
    pub id: i64,
    pub user_id: String,
    pub display_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(async_graphql::SimpleObject)]
pub struct SpotifyPlaylist {
    pub id: i64,
    pub spotify_id: String,
    pub name: String,
    pub description: Option<String>,
    pub track_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(async_graphql::SimpleObject)]
pub struct SpotifyPlaylistSyncState {
    pub id: i64,
    pub spotify_playlist_id: i64,
    pub local_playlist_id: Option<i64>,
    pub last_sync_at: Option<i64>,
    pub sync_status: String,
    pub tracks_downloaded: i32,
    pub tracks_failed: i32,
    pub error_log: Option<String>,
}

#[derive(async_graphql::SimpleObject)]
pub struct SpotifyTrackDownloadFailure {
    pub id: i64,
    pub spotify_playlist_id: i64,
    pub spotify_track_id: String,
    pub track_name: String,
    pub artist_name: String,
    pub album_name: Option<String>,
    pub isrc: Option<String>,
    pub reason: String,
    pub attempts_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(async_graphql::SimpleObject)]
pub struct SpotifyMatchedTrack {
    pub spotify_track_id: String,
    pub spotify_title: String,
    pub spotify_artists: Vec<String>,
    pub spotify_album: String,
    pub spotify_isrc: Option<String>,
    pub spotify_duration: Option<i32>,
    pub spotify_created_at: DateTime<Utc>,
    pub spotify_updated_at: DateTime<Utc>,
    pub local_track: Track,
}

#[derive(async_graphql::SimpleObject)]
pub struct SpotifyMatchedTracksResponse {
    pub matched_tracks: Vec<SpotifyMatchedTrack>,
    pub total_count: i64,
    pub page: i32,
    pub page_size: i32,
}

#[derive(async_graphql::SimpleObject)]
pub struct SpotifyMatchCandidate {
    pub id: i64,
    pub local_track: Track,
    pub score: f64,
    pub confidence: String,
    pub title_similarity: f64,
    pub artist_similarity: f64,
    pub album_similarity: f64,
    pub duration_match: String,
    pub version_match: String,
}

#[derive(async_graphql::SimpleObject)]
pub struct SpotifyUnmatchedTrack {
    pub spotify_track_id: String,
    pub spotify_title: String,
    pub spotify_artists: Vec<String>,
    pub spotify_album: String,
    pub spotify_isrc: Option<String>,
    pub spotify_duration: Option<i32>,
    pub candidates: Vec<SpotifyMatchCandidate>,
}

#[derive(async_graphql::SimpleObject)]
pub struct SpotifyUnmatchedTracksResponse {
    pub unmatched_tracks: Vec<SpotifyUnmatchedTrack>,
    pub total_count: i64,
    pub page: i32,
    pub page_size: i32,
}

#[derive(async_graphql::SimpleObject)]
pub struct SearchLocalTracksResponse {
    pub tracks: Vec<Track>,
    pub total_count: i64,
    pub page: i32,
    pub page_size: i32,
}

#[Object]
impl SpotifyQuery {
    /// Get all Spotify accounts
    async fn spotify_accounts(&self, ctx: &Context<'_>) -> GraphqlResult<Vec<SpotifyAccount>> {
        let app_state = get_app_state(ctx)?;
        let service =
            crate::services::spotify::account::SpotifyAccountService::new(app_state.db.clone());
        let accounts = service.list_accounts().await?;

        accounts
            .into_iter()
            .map(|account| {
                Ok(SpotifyAccount {
                    id: account.id,
                    user_id: account.user_id,
                    display_name: account.display_name,
                    created_at: DateTime::from_timestamp(account.created_at, 0)
                        .ok_or_eyre("Failed to convert created_at to DateTime<Utc>")?,
                    updated_at: DateTime::from_timestamp(account.updated_at, 0)
                        .ok_or_eyre("Failed to convert updated_at to DateTime<Utc>")?,
                })
            })
            .collect::<GraphqlResult<Vec<SpotifyAccount>>>()
    }

    /// Get playlists for a Spotify account
    async fn spotify_playlists(
        &self,
        ctx: &Context<'_>,
        account_id: i64,
    ) -> GraphqlResult<Vec<SpotifyPlaylist>> {
        let app_state = get_app_state(ctx)?;
        let service = SpotifySyncQueryService::new(app_state.db.clone());
        let playlists = service.list_playlists(account_id).await?;

        playlists
            .into_iter()
            .map(|playlist| {
                Ok(SpotifyPlaylist {
                    id: playlist.id,
                    spotify_id: playlist.spotify_id,
                    name: playlist.name,
                    description: playlist.description,
                    track_count: playlist.track_count,
                    created_at: DateTime::from_timestamp(playlist.created_at, 0)
                        .ok_or_eyre("Failed to convert created_at to DateTime<Utc>")?,
                    updated_at: DateTime::from_timestamp(playlist.updated_at, 0)
                        .ok_or_eyre("Failed to convert updated_at to DateTime<Utc>")?,
                })
            })
            .collect::<GraphqlResult<Vec<SpotifyPlaylist>>>()
    }

    /// Get sync state for a Spotify playlist
    async fn spotify_playlist_sync_state(
        &self,
        ctx: &Context<'_>,
        spotify_playlist_id: i64,
    ) -> GraphqlResult<Option<SpotifyPlaylistSyncState>> {
        let app_state = get_app_state(ctx)?;
        let service = SpotifySyncQueryService::new(app_state.db.clone());
        let sync_state = service.get_playlist_sync_state(spotify_playlist_id).await?;

        Ok(sync_state.map(|state| SpotifyPlaylistSyncState {
            id: state.id,
            spotify_playlist_id: state.spotify_playlist_id,
            local_playlist_id: state.local_playlist_id,
            last_sync_at: state.last_sync_at,
            sync_status: state.sync_status,
            tracks_downloaded: state.tracks_downloaded,
            tracks_failed: state.tracks_failed,
            error_log: state.error_log,
        }))
    }

    /// Get download failures for a Spotify playlist
    async fn spotify_track_download_failures(
        &self,
        ctx: &Context<'_>,
        spotify_playlist_id: i64,
    ) -> GraphqlResult<Vec<SpotifyTrackDownloadFailure>> {
        let app_state = get_app_state(ctx)?;
        let service = SpotifySyncQueryService::new(app_state.db.clone());
        let failures = service.list_download_failures(spotify_playlist_id).await?;

        failures
            .into_iter()
            .map(|failure| {
                Ok(SpotifyTrackDownloadFailure {
                    id: failure.id,
                    spotify_playlist_id: failure.spotify_playlist_id,
                    spotify_track_id: failure.spotify_track_id,
                    track_name: failure.track_name,
                    artist_name: failure.artist_name,
                    album_name: failure.album_name,
                    isrc: failure.isrc,
                    reason: failure.reason,
                    attempts_count: failure.attempts_count,
                    created_at: DateTime::from_timestamp(failure.created_at, 0)
                        .ok_or_eyre("Failed to convert created_at to DateTime<Utc>")?,
                    updated_at: DateTime::from_timestamp(failure.updated_at, 0)
                        .ok_or_eyre("Failed to convert updated_at to DateTime<Utc>")?,
                })
            })
            .collect::<GraphqlResult<Vec<SpotifyTrackDownloadFailure>>>()
    }

    /// Get matched Spotify tracks with their local track information
    async fn spotify_matched_tracks(
        &self,
        ctx: &Context<'_>,
        page: Option<i32>,
        page_size: Option<i32>,
        search: Option<String>,
    ) -> GraphqlResult<SpotifyMatchedTracksResponse> {
        let app_state = get_app_state(ctx)?;
        let service = SpotifyMatchingService::new(app_state.db.clone());

        let page = page.unwrap_or(1).max(1) as usize;
        let page_size = page_size.unwrap_or(25).clamp(1, 100) as usize;

        let result = service
            .list_matched_tracks(search.as_deref(), page, page_size)
            .await?;

        let mut matched_tracks = Vec::new();
        for item in result.items {
            let local_track = map_track_with_relations(item.local_track)?;
            matched_tracks.push(SpotifyMatchedTrack {
                spotify_track_id: item.spotify_track.spotify_track_id,
                spotify_title: item.spotify_track.title,
                spotify_artists: item.spotify_track.artists.0,
                spotify_album: item.spotify_track.album,
                spotify_isrc: item.spotify_track.isrc,
                spotify_duration: item.spotify_track.duration.map(|duration| duration / 1000),
                spotify_created_at: DateTime::from_timestamp(item.spotify_track.created_at, 0)
                    .ok_or_eyre("Failed to convert spotify created_at to DateTime<Utc>")?,
                spotify_updated_at: DateTime::from_timestamp(item.spotify_track.updated_at, 0)
                    .ok_or_eyre("Failed to convert spotify updated_at to DateTime<Utc>")?,
                local_track,
            });
        }

        Ok(SpotifyMatchedTracksResponse {
            matched_tracks,
            total_count: result.total_count as i64,
            page: result.page as i32,
            page_size: result.page_size as i32,
        })
    }

    /// Get unmatched Spotify tracks with their match candidates for review
    async fn spotify_unmatched_tracks(
        &self,
        ctx: &Context<'_>,
        page: Option<i32>,
        page_size: Option<i32>,
        search: Option<String>,
    ) -> GraphqlResult<SpotifyUnmatchedTracksResponse> {
        let app_state = get_app_state(ctx)?;
        let service = SpotifyMatchingService::new(app_state.db.clone());

        let page = page.unwrap_or(1).max(1) as usize;
        let page_size = page_size.unwrap_or(25).clamp(1, 100) as usize;

        let result = service
            .list_unmatched_tracks(search.as_deref(), page, page_size)
            .await?;

        let mut unmatched_tracks = Vec::new();
        for item in result.items {
            let mut candidates = Vec::new();
            for c in item.candidates {
                let local_track = map_track_with_relations(c.local_track)?;
                candidates.push(SpotifyMatchCandidate {
                    id: c.candidate.id,
                    local_track,
                    score: c.candidate.score,
                    confidence: format!("{:?}", c.candidate.confidence),
                    title_similarity: c.candidate.title_similarity,
                    artist_similarity: c.candidate.artist_similarity,
                    album_similarity: c.candidate.album_similarity,
                    duration_match: format!("{:?}", c.candidate.duration_match),
                    version_match: format!("{:?}", c.candidate.version_match),
                });
            }

            unmatched_tracks.push(SpotifyUnmatchedTrack {
                spotify_track_id: item.spotify_track.spotify_track_id,
                spotify_title: item.spotify_track.title,
                spotify_artists: item.spotify_track.artists.0,
                spotify_album: item.spotify_track.album,
                spotify_isrc: item.spotify_track.isrc,
                spotify_duration: item.spotify_track.duration.map(|d| d / 1000),
                candidates,
            });
        }

        Ok(SpotifyUnmatchedTracksResponse {
            unmatched_tracks,
            total_count: result.total_count as i64,
            page: result.page as i32,
            page_size: result.page_size as i32,
        })
    }

    /// Search local tracks for manual matching
    async fn search_local_tracks_for_matching(
        &self,
        ctx: &Context<'_>,
        search: String,
        page: Option<i32>,
        page_size: Option<i32>,
    ) -> GraphqlResult<SearchLocalTracksResponse> {
        let app_state = get_app_state(ctx)?;
        let service = SpotifyMatchingService::new(app_state.db.clone());

        let page = page.unwrap_or(1).max(1) as usize;
        let page_size = page_size.unwrap_or(25).clamp(1, 100) as usize;

        let result = service
            .search_local_tracks(&search, page, page_size)
            .await?;

        let tracks: Vec<Track> = result
            .items
            .into_iter()
            .map(map_track_with_relations)
            .collect::<color_eyre::Result<Vec<_>>>()?;

        Ok(SearchLocalTracksResponse {
            tracks,
            total_count: result.total_count as i64,
            page: result.page as i32,
            page_size: result.page_size as i32,
        })
    }
}
