use std::sync::Arc;

use async_graphql::http::GraphiQLSource;
use async_graphql::{EmptySubscription, Object, Schema};
use axum::response::{Html, IntoResponse};

use async_graphql::Context;
use sea_orm::{EntityTrait, QueryOrder};

use crate::entities;
use crate::http_server::graphql_error::GraphqlResult;
use crate::http_server::state::AppState;

pub mod soulseek_mutations;
pub mod track_queries;

use soulseek_mutations::Mutation;
use track_queries::{Album, Artist, Track};

pub struct Query;

#[Object]
impl Query {
    async fn howdy(&self) -> &'static str {
        "partner"
    }

    async fn error_example(&self) -> GraphqlResult<&'static str> {
        Err(color_eyre::eyre::eyre!("This is a test error from the graphql schema").into())
    }

    async fn tracks(&self, ctx: &Context<'_>) -> GraphqlResult<Vec<Track>> {
        let app_state = ctx
            .data::<AppState>()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get app state: {:?}", e))?;
        let db = &app_state.db;

        // Fetch all tracks with their albums
        let track_models = entities::track::Entity::find()
            .order_by_desc(entities::track::Column::CreatedAt)
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

            tracks.push(Track {
                id: track_model.id,
                title: track_model.title,
                track_number: track_model.track_number,
                duration: track_model.duration,
                created_at: track_model.created_at,
                album: Album {
                    id: album_model.id,
                    title: album_model.title,
                    year: album_model.year,
                    artwork_url: None, // TODO: Add artwork URL support
                },
                artists,
            });
        }

        Ok(tracks)
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
