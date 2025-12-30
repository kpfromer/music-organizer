use std::sync::Arc;

use async_graphql::{Context, Object};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, Set};

use crate::entities;
use crate::http_server::graphql::playlist_queries::Playlist;
use crate::http_server::graphql_error::GraphqlResult;
use crate::http_server::state::AppState;

#[derive(Default)]
pub struct PlaylistMutation;

#[Object]
impl PlaylistMutation {
    async fn create_playlist(
        &self,
        ctx: &Context<'_>,
        name: String,
        description: Option<String>,
    ) -> GraphqlResult<Playlist> {
        let app_state = ctx
            .data::<Arc<AppState>>()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get app state: {:?}", e))?;
        let db = &app_state.db;

        let playlist = entities::playlist::ActiveModel {
            name: Set(name),
            description: Set(description),
            ..Default::default()
        };

        let playlist_model = playlist
            .insert(&db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to create playlist: {}", e))?;

        Ok(Playlist {
            id: playlist_model.id,
            name: playlist_model.name,
            description: playlist_model.description,
            created_at: playlist_model.created_at,
            updated_at: playlist_model.updated_at,
            track_count: 0,
        })
    }

    async fn add_track_to_playlist(
        &self,
        ctx: &Context<'_>,
        playlist_id: i64,
        track_id: i64,
    ) -> GraphqlResult<bool> {
        let app_state = ctx
            .data::<Arc<AppState>>()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get app state: {:?}", e))?;
        let db = &app_state.db;

        // Check if playlist exists
        let playlist = entities::playlist::Entity::find_by_id(playlist_id)
            .one(&db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to find playlist: {}", e))?;

        if playlist.is_none() {
            return Err(color_eyre::eyre::eyre!("Playlist not found").into());
        }

        // Check if track exists
        let track = entities::track::Entity::find_by_id(track_id)
            .one(&db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to find track: {}", e))?;

        if track.is_none() {
            return Err(color_eyre::eyre::eyre!("Track not found").into());
        }

        // Check if track is already in playlist
        let existing = entities::playlist_track::Entity::find()
            .filter(
                Condition::all()
                    .add(entities::playlist_track::Column::PlaylistId.eq(playlist_id))
                    .add(entities::playlist_track::Column::TrackId.eq(track_id)),
            )
            .one(&db.conn)
            .await
            .map_err(|e| {
                color_eyre::eyre::eyre!("Failed to check existing playlist track: {}", e)
            })?;

        if existing.is_some() {
            // Track is already in playlist, return success
            return Ok(true);
        }

        // Add track to playlist
        let now = Utc::now();
        let playlist_track = entities::playlist_track::ActiveModel {
            playlist_id: Set(playlist_id),
            track_id: Set(track_id),
            created_at: Set(now),
            updated_at: Set(now),
        };

        playlist_track
            .insert(&db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to add track to playlist: {}", e))?;

        // Update playlist's updated_at timestamp
        let mut playlist_model: entities::playlist::ActiveModel = playlist.unwrap().into();
        playlist_model.updated_at = Set(Utc::now());
        playlist_model
            .update(&db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to update playlist: {}", e))?;

        Ok(true)
    }
}
