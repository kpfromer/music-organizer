use std::sync::Arc;

use async_graphql::http::GraphiQLSource;
use async_graphql::{EmptySubscription, MergedObject, Object, Schema};
use axum::response::{Html, IntoResponse};

use async_graphql::Context;
use chrono::{DateTime, Utc};
use color_eyre::eyre::OptionExt;
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};

use crate::entities;
use crate::http_server::graphql_error::GraphqlResult;
use crate::http_server::state::AppState;

pub mod playlist_mutations;
pub mod playlist_queries;
pub mod soulseek_mutations;
pub mod track_queries;
pub mod unimportable_file_queries;

use playlist_mutations::PlaylistMutation;
use playlist_queries::{Playlist, PlaylistsResponse};
use soulseek_mutations::SoulseekMutation;
use track_queries::{Album, Artist, Track, TracksResponse};
use unimportable_file_queries::{UnimportableFile, UnimportableFilesResponse};

pub struct Query;

#[Object]
impl Query {
    async fn howdy(&self) -> &'static str {
        "partner"
    }

    async fn error_example(&self) -> GraphqlResult<&'static str> {
        Err(color_eyre::eyre::eyre!("This is a test error from the graphql schema").into())
    }

    async fn tracks(
        &self,
        ctx: &Context<'_>,
        page: Option<i32>,
        page_size: Option<i32>,
    ) -> GraphqlResult<TracksResponse> {
        // TODO: Performance issue
        // N+1 query problem: Fetch all track artists in a single query.
        // The current implementation fetches artists individually for each track, resulting in N+1 database queries when there are N tracks.
        // This can severely impact performance with many tracks.

        let app_state = ctx
            .data::<Arc<AppState>>()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get app state: {:?}", e))?;
        let db = &app_state.db;

        let page = page.unwrap_or(1).max(1) as usize;
        let page_size = page_size.unwrap_or(25).clamp(1, 100) as usize;

        // Get total count
        let total_count = entities::track::Entity::find()
            .count(&db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to count tracks: {}", e))?;

        // Calculate offset
        let offset = (page.saturating_sub(1)) * page_size;

        // Fetch paginated tracks with their albums
        let track_models = entities::track::Entity::find()
            .order_by_desc(entities::track::Column::CreatedAt)
            .find_also_related(entities::album::Entity)
            .limit(page_size as u64)
            .offset(offset as u64)
            .all(&db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to fetch tracks: {}", e))?;

        let mut tracks = Vec::new();

        for (track_model, album_model) in track_models {
            let album_model = album_model.ok_or_else(|| {
                color_eyre::eyre::eyre!("Track {} has no associated album", track_model.id)
            })?;

            // Fetch artists for this track
            let track_artists = db
                .get_track_artists(track_model.id)
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

            tracks.push(Track {
                id: track_model.id,
                title: track_model.title,
                track_number: track_model.track_number,
                duration: track_model.duration,
                created_at: DateTime::<Utc>::from_timestamp_secs(track_model.created_at)
                    .ok_or_eyre("Failed to convert created_at to DateTime<Utc>")?,
                album: Album {
                    id: album_model.id,
                    title: album_model.title,
                    year: album_model.year,
                    artwork_url: Some(format!("{}/album-art-image/{}", base_url, track_model.id)),
                },
                artists,
            });
        }

        Ok(TracksResponse {
            tracks,
            total_count: total_count as i64,
            page: page as i32,
            page_size: page_size as i32,
        })
    }

    async fn unimportable_files(
        &self,
        ctx: &Context<'_>,
        page: Option<i32>,
        page_size: Option<i32>,
    ) -> GraphqlResult<UnimportableFilesResponse> {
        let app_state = ctx
            .data::<Arc<AppState>>()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get app state: {:?}", e))?;
        let db = &app_state.db;

        let page = page.unwrap_or(1).max(1) as usize;
        let page_size = page_size.unwrap_or(25).clamp(1, 100) as usize;

        let (files, total_count) = db
            .get_unimportable_files(page, page_size)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to fetch unimportable files: {}", e))?;

        let mut unimportable_files = Vec::new();
        for file in files {
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
            total_count: total_count as i64,
            page: page as i32,
            page_size: page_size as i32,
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
        let app_state = ctx
            .data::<Arc<AppState>>()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get app state: {:?}", e))?;
        let db = &app_state.db;

        let page = page.unwrap_or(1).max(1) as usize;
        let page_size = page_size.unwrap_or(25).clamp(1, 100) as usize;

        // Build query with search filter
        let mut query = entities::playlist::Entity::find();
        if let Some(search_term) = &search
            && !search_term.is_empty()
        {
            let condition = Condition::any()
                .add(entities::playlist::Column::Name.contains(search_term))
                .add(entities::playlist::Column::Description.contains(search_term));
            query = query.filter(condition);
        }

        // Get total count with search filter applied
        let total_count = query
            .clone()
            .count(&db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to count playlists: {}", e))?;

        // Apply sorting
        let sort_by = sort_by.as_deref().unwrap_or("created_at");
        let sort_order = sort_order.as_deref().unwrap_or("desc");
        let is_desc = sort_order == "desc";

        query = match sort_by {
            "name" => {
                if is_desc {
                    query.order_by_desc(entities::playlist::Column::Name)
                } else {
                    query.order_by_asc(entities::playlist::Column::Name)
                }
            }
            "updated_at" => {
                if is_desc {
                    query.order_by_desc(entities::playlist::Column::UpdatedAt)
                } else {
                    query.order_by_asc(entities::playlist::Column::UpdatedAt)
                }
            }
            _ => {
                // Default to created_at
                if is_desc {
                    query.order_by_desc(entities::playlist::Column::CreatedAt)
                } else {
                    query.order_by_asc(entities::playlist::Column::CreatedAt)
                }
            }
        };

        // Calculate offset
        let offset = (page.saturating_sub(1)) * page_size;

        // Fetch paginated playlists
        let playlist_models = query
            .limit(page_size as u64)
            .offset(offset as u64)
            .all(&db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to fetch playlists: {}", e))?;

        let mut playlists = Vec::new();

        for playlist_model in playlist_models {
            // TODO: fix n+1 query issue
            // Count tracks for this playlist
            let track_count = entities::playlist_track::Entity::find()
                .filter(entities::playlist_track::Column::PlaylistId.eq(playlist_model.id))
                .count(&db.conn)
                .await
                .map_err(|e| {
                    color_eyre::eyre::eyre!("Failed to count tracks for playlist: {}", e)
                })?;

            playlists.push(Playlist {
                id: playlist_model.id,
                name: playlist_model.name,
                description: playlist_model.description,
                created_at: playlist_model.created_at,
                updated_at: playlist_model.updated_at,
                track_count: track_count as i64,
            });
        }

        Ok(PlaylistsResponse {
            playlists,
            total_count: total_count as i64,
            page: page as i32,
            page_size: page_size as i32,
        })
    }

    async fn playlist(&self, ctx: &Context<'_>, id: i64) -> GraphqlResult<Option<Playlist>> {
        let app_state = ctx
            .data::<Arc<AppState>>()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get app state: {:?}", e))?;
        let db = &app_state.db;

        let playlist_model = entities::playlist::Entity::find_by_id(id)
            .one(&db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to find playlist: {}", e))?;

        if let Some(playlist_model) = playlist_model {
            // Count tracks for this playlist
            let track_count = entities::playlist_track::Entity::find()
                .filter(entities::playlist_track::Column::PlaylistId.eq(playlist_model.id))
                .count(&db.conn)
                .await
                .map_err(|e| {
                    color_eyre::eyre::eyre!("Failed to count tracks for playlist: {}", e)
                })?;

            Ok(Some(Playlist {
                id: playlist_model.id,
                name: playlist_model.name,
                description: playlist_model.description,
                created_at: playlist_model.created_at,
                updated_at: playlist_model.updated_at,
                track_count: track_count as i64,
            }))
        } else {
            Ok(None)
        }
    }

    async fn playlist_tracks(
        &self,
        ctx: &Context<'_>,
        playlist_id: i64,
        page: Option<i32>,
        page_size: Option<i32>,
    ) -> GraphqlResult<TracksResponse> {
        let app_state = ctx
            .data::<Arc<AppState>>()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get app state: {:?}", e))?;
        let db = &app_state.db;

        // Verify playlist exists
        let playlist = entities::playlist::Entity::find_by_id(playlist_id)
            .one(&db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to find playlist: {}", e))?;

        if playlist.is_none() {
            return Err(color_eyre::eyre::eyre!("Playlist not found").into());
        }

        let page = page.unwrap_or(1).max(1) as usize;
        let page_size = page_size.unwrap_or(25).clamp(1, 100) as usize;

        // Get track IDs for this playlist
        let playlist_track_models = entities::playlist_track::Entity::find()
            .filter(entities::playlist_track::Column::PlaylistId.eq(playlist_id))
            .all(&db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to fetch playlist tracks: {}", e))?;

        let track_ids: Vec<i64> = playlist_track_models.iter().map(|pt| pt.track_id).collect();

        let total_count = track_ids.len() as i64;

        // Calculate offset
        let offset = (page.saturating_sub(1)) * page_size;
        let paginated_track_ids: Vec<i64> =
            track_ids.into_iter().skip(offset).take(page_size).collect();

        if paginated_track_ids.is_empty() {
            return Ok(TracksResponse {
                tracks: Vec::new(),
                total_count,
                page: page as i32,
                page_size: page_size as i32,
            });
        }

        // Fetch paginated tracks with their albums
        let track_models = entities::track::Entity::find()
            .filter(entities::track::Column::Id.is_in(paginated_track_ids))
            .find_also_related(entities::album::Entity)
            .all(&db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to fetch tracks: {}", e))?;

        let mut tracks = Vec::new();

        for (track_model, album_model) in track_models {
            let album_model = album_model.ok_or_else(|| {
                color_eyre::eyre::eyre!("Track {} has no associated album", track_model.id)
            })?;

            // Fetch artists for this track
            let track_artists = db
                .get_track_artists(track_model.id)
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

            tracks.push(Track {
                id: track_model.id,
                title: track_model.title,
                track_number: track_model.track_number,
                duration: track_model.duration,
                created_at: DateTime::<Utc>::from_timestamp_secs(track_model.created_at)
                    .ok_or_eyre("Failed to convert created_at to DateTime<Utc>")?,
                album: Album {
                    id: album_model.id,
                    title: album_model.title,
                    year: album_model.year,
                    artwork_url: Some(format!("{}/album-art-image/{}", base_url, track_model.id)),
                },
                artists,
            });
        }

        Ok(TracksResponse {
            tracks,
            total_count,
            page: page as i32,
            page_size: page_size as i32,
        })
    }
}

#[derive(Default, MergedObject)]
pub struct Mutation(PlaylistMutation, SoulseekMutation);

pub async fn graphql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

pub fn create_schema(app_state: Arc<AppState>) -> Schema<Query, Mutation, EmptySubscription> {
    Schema::build(Query, Mutation::default(), EmptySubscription)
        .data(app_state)
        .finish()
}
