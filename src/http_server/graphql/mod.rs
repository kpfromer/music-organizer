use std::sync::Arc;

use async_graphql::http::GraphiQLSource;
use async_graphql::{EmptySubscription, Object, Schema};
use axum::response::{Html, IntoResponse};

use async_graphql::Context;
use chrono::{DateTime, Utc};
use color_eyre::eyre::OptionExt;
use sea_orm::{EntityTrait, PaginatorTrait, QueryOrder, QuerySelect};

use crate::entities;
use crate::http_server::graphql_error::GraphqlResult;
use crate::http_server::state::AppState;

pub mod soulseek_mutations;
pub mod track_queries;
pub mod unimportable_file_queries;

use soulseek_mutations::Mutation;
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
}

pub async fn graphql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

pub fn create_schema(app_state: Arc<AppState>) -> Schema<Query, Mutation, EmptySubscription> {
    Schema::build(Query, Mutation, EmptySubscription)
        .data(app_state)
        .finish()
}
