use std::sync::Arc;

use async_graphql::http::GraphiQLSource;
use async_graphql::{EmptySubscription, MergedObject, Object, Schema};
use axum::response::{Html, IntoResponse};

use async_graphql::Context;
use chrono::{DateTime, Utc};
use color_eyre::eyre::OptionExt;

use crate::http_server::graphql::plex_library_refresh_queries::PlexLibraryRefreshQuery;
use crate::http_server::graphql::query_builder::{
    PaginationInput, SortInput, TextSearchInput, TrackSortField, TrackSortInput,
};
use crate::http_server::graphql::spotify::spotify_mutations::SpotifyMutation;
use crate::http_server::graphql::spotify::spotify_queries::SpotifyQuery;
use crate::http_server::graphql::wishlist::WishlistMutation;
use crate::http_server::graphql::wishlist::WishlistQuery;
use crate::http_server::graphql::youtube_mutations::YoutubeMutation;
use crate::http_server::graphql::youtube_queries::YoutubeQuery;
use crate::http_server::graphql_error::GraphqlResult;
use crate::http_server::state::AppState;
use crate::services::plex::PlexService;
use crate::services::plex::client::PlexHttpAdapter;
use crate::services::track::{TrackService, TrackWithRelations};

mod context;
pub mod playlist_mutations;
pub mod playlist_queries;
pub mod plex_library_refresh_mutations;
pub mod plex_library_refresh_queries;
pub mod plex_playlist_mutations;
pub mod plex_playlist_queries;
pub mod plex_server_mutations;
pub mod plex_server_queries;
pub mod plex_track_queries;
pub mod query_builder;
pub mod soulseek_mutations;
mod spotify;
pub mod track_queries;
pub mod unimportable_file_queries;
mod wishlist;
mod youtube_mutations;
mod youtube_queries;

use context::get_app_state;
use playlist_mutations::PlaylistMutation;
use playlist_queries::{Playlist, PlaylistsResponse};
use plex_library_refresh_mutations::PlexLibraryRefreshMutation;
use plex_playlist_mutations::PlexPlaylistMutation;
use plex_playlist_queries::PlexPlaylistsResponse;
use plex_server_mutations::PlexServerMutation;
use plex_server_queries::PlexServer;
use plex_track_queries::PlexTracksResult;
use soulseek_mutations::SoulseekMutation;
use track_queries::{Album, Artist, Track, TracksResponse};
use unimportable_file_queries::{UnimportableFile, UnimportableFilesResponse};

pub(crate) fn map_track_with_relations(twr: TrackWithRelations) -> color_eyre::Result<Track> {
    #[cfg(debug_assertions)]
    let base_url = "http://localhost:3000";
    #[cfg(not(debug_assertions))]
    let base_url = "";

    Ok(Track {
        id: twr.track.id,
        title: twr.track.title,
        track_number: twr.track.track_number,
        duration: twr.track.duration,
        created_at: DateTime::<Utc>::from_timestamp_secs(twr.track.created_at)
            .ok_or_eyre("Failed to convert created_at to DateTime<Utc>")?,
        album: Album {
            id: twr.album.id,
            title: twr.album.title,
            year: twr.album.year,
            artwork_url: Some(format!("{}/album-art-image/{}", base_url, twr.track.id)),
        },
        artists: twr
            .artists
            .into_iter()
            .map(|(artist, _)| Artist {
                id: artist.id,
                name: artist.name,
            })
            .collect(),
    })
}

/// Compute how many spotify tracks in the linked spotify playlist are unmatched (no local_track_id).
/// Returns None if the playlist isn't linked to a spotify playlist.
async fn compute_unmatched_count(
    db: &crate::database::Database,
    spotify_playlist_id: Option<i64>,
) -> color_eyre::Result<Option<i64>> {
    use sea_orm::{
        ColumnTrait, EntityTrait, JoinType, PaginatorTrait, QueryFilter, QuerySelect, RelationTrait,
    };

    let Some(sp_id) = spotify_playlist_id else {
        return Ok(None);
    };

    use crate::entities;

    // Count spotify tracks in this playlist that have no local_track_id.
    // Start from spotify_track, join to spotify_track_playlist (reverse relation),
    // then filter by playlist id and unmatched.
    let count = entities::spotify_track::Entity::find()
        .join(
            JoinType::InnerJoin,
            entities::spotify_track_playlist::Relation::SpotifyTrack
                .def()
                .rev(),
        )
        .filter(entities::spotify_track_playlist::Column::SpotifyPlaylistId.eq(sp_id))
        .filter(entities::spotify_track::Column::LocalTrackId.is_null())
        .count(&db.conn)
        .await?;

    Ok(Some(count as i64))
}

// TODO: Remove this once we have a proper query object.
#[derive(Default)]
pub struct LegacyQuery;

#[Object]
impl LegacyQuery {
    async fn tracks(
        &self,
        ctx: &Context<'_>,
        pagination: Option<PaginationInput>,
        search: Option<TextSearchInput>,
        sort: Option<Vec<TrackSortInput>>,
    ) -> GraphqlResult<TracksResponse> {
        let app_state = get_app_state(ctx)?;
        let service = TrackService::new(app_state.db.clone());

        let search_str = search.as_ref().and_then(|s| s.search.as_deref());
        let sort_inputs: Vec<SortInput<TrackSortField>> = sort
            .unwrap_or_default()
            .into_iter()
            .map(Into::into)
            .collect();

        let result = service
            .list_tracks(
                search_str,
                &sort_inputs,
                pagination.as_ref().and_then(|p| p.page),
                pagination.as_ref().and_then(|p| p.page_size),
            )
            .await?;

        let tracks: Vec<Track> = result
            .items
            .into_iter()
            .map(map_track_with_relations)
            .collect::<color_eyre::Result<Vec<_>>>()?;

        Ok(TracksResponse {
            tracks,
            total_count: result.total_count as i64,
            page: result.page as i32,
            page_size: result.page_size as i32,
        })
    }

    async fn unimportable_files(
        &self,
        ctx: &Context<'_>,
        page: Option<i32>,
        page_size: Option<i32>,
    ) -> GraphqlResult<UnimportableFilesResponse> {
        let app_state = get_app_state(ctx)?;
        let service = TrackService::new(app_state.db.clone());

        let result = service.list_unimportable_files(page, page_size).await?;

        let mut unimportable_files = Vec::new();
        for file in result.items {
            let created_at = DateTime::<Utc>::from_timestamp_secs(file.created_at)
                .ok_or_eyre("Failed to convert created_at to DateTime<Utc>")?;
            unimportable_files.push(UnimportableFile {
                id: file.id,
                file_path: file.file_path,
                sha256: file.sha256,
                created_at,
                reason: file.reason,
            });
        }

        Ok(UnimportableFilesResponse {
            files: unimportable_files,
            total_count: result.total_count as i64,
            page: result.page as i32,
            page_size: result.page_size as i32,
        })
    }

    async fn playlists(
        &self,
        ctx: &Context<'_>,
        page: Option<i32>,
        page_size: Option<i32>,
        search: Option<String>,
        sort_by: Option<String>,
        sort_order: Option<String>,
    ) -> GraphqlResult<PlaylistsResponse> {
        let app_state = get_app_state(ctx)?;
        let service = TrackService::new(app_state.db.clone());

        let result = service
            .list_playlists(
                search.as_deref(),
                sort_by.as_deref(),
                sort_order.as_deref(),
                page,
                page_size,
            )
            .await?;

        let mut playlists = Vec::new();
        for (playlist_model, track_count) in result.items {
            let unmatched_count =
                compute_unmatched_count(&app_state.db, playlist_model.spotify_playlist_id).await?;
            playlists.push(Playlist {
                id: playlist_model.id,
                name: playlist_model.name,
                description: playlist_model.description,
                spotify_playlist_id: playlist_model.spotify_playlist_id,
                unmatched_spotify_track_count: unmatched_count,
                created_at: playlist_model.created_at,
                updated_at: playlist_model.updated_at,
                track_count: track_count as i64,
            });
        }

        Ok(PlaylistsResponse {
            playlists,
            total_count: result.total_count as i64,
            page: result.page as i32,
            page_size: result.page_size as i32,
        })
    }

    async fn playlist(&self, ctx: &Context<'_>, id: i64) -> GraphqlResult<Option<Playlist>> {
        let app_state = get_app_state(ctx)?;
        let service = TrackService::new(app_state.db.clone());

        let result = service.get_playlist(id).await?;

        match result {
            Some((playlist_model, track_count)) => {
                let unmatched_count =
                    compute_unmatched_count(&app_state.db, playlist_model.spotify_playlist_id)
                        .await?;
                Ok(Some(Playlist {
                    id: playlist_model.id,
                    name: playlist_model.name,
                    description: playlist_model.description,
                    spotify_playlist_id: playlist_model.spotify_playlist_id,
                    unmatched_spotify_track_count: unmatched_count,
                    created_at: playlist_model.created_at,
                    updated_at: playlist_model.updated_at,
                    track_count: track_count as i64,
                }))
            }
            None => Ok(None),
        }
    }

    async fn playlist_tracks(
        &self,
        ctx: &Context<'_>,
        playlist_id: i64,
        page: Option<i32>,
        page_size: Option<i32>,
    ) -> GraphqlResult<TracksResponse> {
        let app_state = get_app_state(ctx)?;
        let service = TrackService::new(app_state.db.clone());

        let result = service
            .list_playlist_tracks(playlist_id, page, page_size)
            .await?;

        let tracks: Vec<Track> = result
            .items
            .into_iter()
            .map(map_track_with_relations)
            .collect::<color_eyre::Result<Vec<_>>>()?;

        Ok(TracksResponse {
            tracks,
            total_count: result.total_count as i64,
            page: result.page as i32,
            page_size: result.page_size as i32,
        })
    }

    async fn plex_servers(&self, ctx: &Context<'_>) -> GraphqlResult<Vec<PlexServer>> {
        let app_state = get_app_state(ctx)?;
        let service = PlexService::new(app_state.db.clone(), PlexHttpAdapter::new());
        let servers = service.list_servers().await?;

        Ok(servers
            .into_iter()
            .map(|server| PlexServer {
                id: server.id,
                name: server.name,
                server_url: server.server_url,
                has_access_token: server.access_token.is_some(),
                created_at: server.created_at,
                updated_at: server.updated_at,
            })
            .collect())
    }

    async fn plex_tracks(&self, ctx: &Context<'_>) -> GraphqlResult<PlexTracksResult> {
        plex_track_queries::plex_tracks(ctx).await
    }

    async fn plex_playlists(&self, ctx: &Context<'_>) -> GraphqlResult<PlexPlaylistsResponse> {
        plex_playlist_queries::plex_playlists(ctx).await
    }
}

#[derive(Default, MergedObject)]
pub struct Query(
    LegacyQuery,
    PlexLibraryRefreshQuery,
    SpotifyQuery,
    YoutubeQuery,
    WishlistQuery,
);

#[derive(Default, MergedObject)]
pub struct Mutation(
    PlaylistMutation,
    SoulseekMutation,
    PlexServerMutation,
    PlexPlaylistMutation,
    PlexLibraryRefreshMutation,
    SpotifyMutation,
    YoutubeMutation,
    WishlistMutation,
);

pub async fn graphql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

pub fn create_schema(app_state: Arc<AppState>) -> Schema<Query, Mutation, EmptySubscription> {
    Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .data(app_state)
        .finish()
}
