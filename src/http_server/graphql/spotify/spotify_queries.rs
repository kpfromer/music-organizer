use async_graphql::{Context, Object};
use chrono::{DateTime, Utc};
use color_eyre::eyre::OptionExt;
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};

use crate::entities;
use crate::http_server::graphql::context::get_app_state;
use crate::http_server::graphql::track_queries::{Album, Artist, Track};
use crate::http_server::graphql_error::GraphqlResult;
use color_eyre::eyre::WrapErr;

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

#[Object]
impl SpotifyQuery {
    /// Get all Spotify accounts
    async fn spotify_accounts(&self, ctx: &Context<'_>) -> GraphqlResult<Vec<SpotifyAccount>> {
        let db = &get_app_state(ctx)?.db;

        let accounts = entities::spotify_account::Entity::find()
            .all(&db.conn)
            .await
            .wrap_err("Failed to fetch spotify accounts")?;

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
        let db = &get_app_state(ctx)?.db;

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
        let db = &get_app_state(ctx)?.db;

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
        let db = &get_app_state(ctx)?.db;

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

    /// Get matched Spotify tracks with their local track information
    async fn spotify_matched_tracks(
        &self,
        ctx: &Context<'_>,
        page: Option<i32>,
        page_size: Option<i32>,
        search: Option<String>,
    ) -> GraphqlResult<SpotifyMatchedTracksResponse> {
        let db = &get_app_state(ctx)?.db;

        let page = page.unwrap_or(1).max(1) as usize;
        let page_size = page_size.unwrap_or(25).clamp(1, 100) as usize;

        // Build base query for matched Spotify tracks (where local_track_id is not null)
        let mut base_query = entities::spotify_track::Entity::find()
            .filter(entities::spotify_track::Column::LocalTrackId.is_not_null());

        // Apply search filter if provided
        if let Some(search_term) = &search {
            if !search_term.is_empty() {
                let condition = Condition::any()
                    .add(entities::spotify_track::Column::Title.contains(search_term))
                    .add(entities::spotify_track::Column::Album.contains(search_term));
                base_query = base_query.filter(condition);
            }
        }

        // Get total count (build count query separately with same filters)
        let mut count_query = entities::spotify_track::Entity::find()
            .filter(entities::spotify_track::Column::LocalTrackId.is_not_null());

        if let Some(search_term) = &search {
            if !search_term.is_empty() {
                let condition = Condition::any()
                    .add(entities::spotify_track::Column::Title.contains(search_term))
                    .add(entities::spotify_track::Column::Album.contains(search_term));
                count_query = count_query.filter(condition);
            }
        }

        let total_count = count_query.count(&db.conn).await.map_err(|e| {
            color_eyre::eyre::eyre!("Failed to count matched spotify tracks: {}", e)
        })?;

        // Apply pagination
        let offset = (page.saturating_sub(1)) * page_size;
        let spotify_tracks = base_query
            .limit(page_size as u64)
            .offset(offset as u64)
            .order_by_desc(entities::spotify_track::Column::UpdatedAt)
            .all(&db.conn)
            .await
            .map_err(|e| {
                color_eyre::eyre::eyre!("Failed to fetch matched spotify tracks: {}", e)
            })?;

        let mut matched_tracks = Vec::new();

        for spotify_track in spotify_tracks {
            let local_track_id = spotify_track
                .local_track_id
                .ok_or_eyre("Spotify track should have local_track_id")?;

            // Fetch local track with album
            let (local_track_model, album_model) =
                entities::track::Entity::find_by_id(local_track_id)
                    .find_also_related(entities::album::Entity)
                    .one(&db.conn)
                    .await
                    .map_err(|e| color_eyre::eyre::eyre!("Failed to fetch local track: {}", e))?
                    .ok_or_eyre("Local track not found")?;

            let album_model = album_model.ok_or_eyre("Local track has no album")?;

            // Fetch artists for local track
            let track_artists = db
                .get_track_artists(local_track_id)
                .await
                .map_err(|e| color_eyre::eyre::eyre!("Failed to fetch track artists: {}", e))?;

            let artists: Vec<Artist> = track_artists
                .into_iter()
                .map(|(artist, _)| Artist {
                    id: artist.id,
                    name: artist.name,
                })
                .collect();

            #[cfg(debug_assertions)]
            let base_url = "http://localhost:3000".to_string();
            #[cfg(not(debug_assertions))]
            let base_url = "";

            let local_track = Track {
                id: local_track_model.id,
                title: local_track_model.title,
                track_number: local_track_model.track_number,
                duration: local_track_model.duration,
                created_at: DateTime::<Utc>::from_timestamp_secs(local_track_model.created_at)
                    .ok_or_eyre("Failed to convert created_at to DateTime<Utc>")?,
                album: Album {
                    id: album_model.id,
                    title: album_model.title,
                    year: album_model.year,
                    artwork_url: Some(format!("{}/album-art-image/{}", base_url, local_track_id)),
                },
                artists,
            };

            matched_tracks.push(SpotifyMatchedTrack {
                spotify_track_id: spotify_track.spotify_track_id,
                spotify_title: spotify_track.title,
                spotify_artists: spotify_track.artists.0,
                spotify_album: spotify_track.album,
                spotify_isrc: spotify_track.isrc,
                spotify_duration: spotify_track.duration.map(|duration| duration / 1000),
                spotify_created_at: DateTime::from_timestamp(spotify_track.created_at, 0)
                    .ok_or_eyre("Failed to convert spotify created_at to DateTime<Utc>")?,
                spotify_updated_at: DateTime::from_timestamp(spotify_track.updated_at, 0)
                    .ok_or_eyre("Failed to convert spotify updated_at to DateTime<Utc>")?,
                local_track,
            });
        }

        Ok(SpotifyMatchedTracksResponse {
            matched_tracks,
            total_count: total_count as i64,
            page: page as i32,
            page_size: page_size as i32,
        })
    }
}
