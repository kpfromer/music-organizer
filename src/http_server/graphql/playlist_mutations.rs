use async_graphql::{Context, Object};

use crate::http_server::graphql::context::get_app_state;
use crate::http_server::graphql::playlist_queries::Playlist;
use crate::http_server::graphql_error::GraphqlResult;
use crate::services::playlist::PlaylistService;

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
        let db = &get_app_state(ctx)?.db;
        let service = PlaylistService::new(db.clone());
        let model = service.create(name, description).await?;

        Ok(Playlist {
            id: model.id,
            name: model.name,
            description: model.description,
            created_at: model.created_at,
            updated_at: model.updated_at,
            track_count: 0,
        })
    }

    async fn add_track_to_playlist(
        &self,
        ctx: &Context<'_>,
        playlist_id: i64,
        track_id: i64,
    ) -> GraphqlResult<bool> {
        let db = &get_app_state(ctx)?.db;
        let service = PlaylistService::new(db.clone());
        service.add_track(playlist_id, track_id).await?;
        Ok(true)
    }
}
