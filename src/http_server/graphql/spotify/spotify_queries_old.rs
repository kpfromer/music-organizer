use std::sync::Arc;

use async_graphql::{Context, Object};
use chrono::{DateTime, Utc};
use color_eyre::eyre::OptionExt;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::entities;
use crate::http_server::graphql_error::GraphqlResult;
use crate::http_server::state::AppState;

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

#[Object]
impl SpotifyQuery {
    /// Get all Spotify accounts
    async fn spotify_accounts(&self, ctx: &Context<'_>) -> GraphqlResult<Vec<SpotifyAccount>> {
        let app_state = ctx
            .data::<Arc<AppState>>()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get app state: {:?}", e))?;
        let db = &app_state.db;

        let accounts = entities::spotify_account::Entity::find()
            .all(&db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to fetch spotify accounts: {}", e))?;

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
        let app_state = ctx
            .data::<Arc<AppState>>()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get app state: {:?}", e))?;
        let db = &app_state.db;

        let playlists = entities::spotify_playlist::Entity::find()
            .filter(entities::spotify_playlist::Column::AccountId.eq(account_id))
            .all(&db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to fetch spotify playlists: {}", e))?;

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
        let app_state = ctx
            .data::<Arc<AppState>>()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get app state: {:?}", e))?;
        let db = &app_state.db;

        let sync_state = entities::spotify_playlist_sync_state::Entity::find()
            .filter(
                entities::spotify_playlist_sync_state::Column::SpotifyPlaylistId
                    .eq(spotify_playlist_id),
            )
            .one(&db.conn)
            .await
            .map_err(|e| {
                color_eyre::eyre::eyre!("Failed to fetch spotify playlist sync state: {}", e)
            })?;

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
        let app_state = ctx
            .data::<Arc<AppState>>()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get app state: {:?}", e))?;
        let db = &app_state.db;

        let failures = entities::spotify_track_download_failure::Entity::find()
            .filter(
                entities::spotify_track_download_failure::Column::SpotifyPlaylistId
                    .eq(spotify_playlist_id),
            )
            .all(&db.conn)
            .await
            .map_err(|e| {
                color_eyre::eyre::eyre!("Failed to fetch spotify track download failures: {}", e)
            })?;

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
}
