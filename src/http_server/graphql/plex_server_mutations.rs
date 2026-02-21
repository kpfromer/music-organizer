use async_graphql::{Context, Object};

use crate::http_server::graphql::context::get_app_state;
use crate::http_server::graphql::plex_server_queries::{AuthResponse, PlexServer};
use crate::http_server::graphql_error::GraphqlResult;
use crate::services::plex::PlexService;
use crate::services::plex::client::PlexHttpAdapter;

fn map_server(m: crate::entities::plex_server::Model) -> PlexServer {
    PlexServer {
        id: m.id,
        name: m.name,
        server_url: m.server_url,
        has_access_token: m.access_token.is_some(),
        created_at: m.created_at,
        updated_at: m.updated_at,
    }
}

#[derive(Default)]
pub struct PlexServerMutation;

#[Object]
impl PlexServerMutation {
    async fn create_plex_server(
        &self,
        ctx: &Context<'_>,
        name: String,
        server_url: String,
    ) -> GraphqlResult<PlexServer> {
        let app_state = get_app_state(ctx)?;
        let service = PlexService::new(app_state.db.clone(), PlexHttpAdapter::new());
        let server_model = service.create_server(name, server_url).await?;
        Ok(map_server(server_model))
    }

    async fn authenticate_plex_server(
        &self,
        ctx: &Context<'_>,
        server_id: i64,
    ) -> GraphqlResult<AuthResponse> {
        let app_state = get_app_state(ctx)?;
        let service = PlexService::new(app_state.db.clone(), PlexHttpAdapter::new());
        let (auth_url, pin_id) = service
            .start_authentication(server_id, &app_state.base_url)
            .await?;
        Ok(AuthResponse { auth_url, pin_id })
    }

    async fn complete_plex_server_authentication(
        &self,
        ctx: &Context<'_>,
        server_id: i64,
        pin_id: i32,
    ) -> GraphqlResult<PlexServer> {
        let app_state = get_app_state(ctx)?;
        let service = PlexService::new(app_state.db.clone(), PlexHttpAdapter::new());
        let updated_server = service.complete_authentication(server_id, pin_id).await?;
        Ok(map_server(updated_server))
    }
}
