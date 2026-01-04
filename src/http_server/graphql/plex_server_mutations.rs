use std::sync::Arc;

use async_graphql::{Context, Object};
use reqwest::Client;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use url::Url;

use crate::entities;
use crate::http_server::graphql::plex_server_queries::{AuthResponse, PlexServer};
use crate::http_server::graphql_error::GraphqlResult;
use crate::http_server::state::AppState;
use crate::plex_rs::{
    construct_auth_app_url, create_plex_pin, get_plex_resources, poll_for_plex_auth,
};
use sea_orm::ActiveModelBehavior;

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
        let app_state = ctx
            .data::<Arc<AppState>>()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get app state: {:?}", e))?;
        let db = &app_state.db;

        // Validate server_url is a valid URL
        Url::parse(&server_url)
            .map_err(|e| color_eyre::eyre::eyre!("Invalid server URL: {}", e))?;

        let server = entities::plex_server::ActiveModel {
            name: Set(name),
            server_url: Set(server_url),
            ..entities::plex_server::ActiveModel::new()
        };

        let server_model = server
            .insert(&db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to create plex server: {}", e))?;

        Ok(PlexServer {
            id: server_model.id,
            name: server_model.name,
            server_url: server_model.server_url,
            has_access_token: server_model.access_token.is_some(),
            created_at: server_model.created_at,
            updated_at: server_model.updated_at,
        })
    }

    async fn authenticate_plex_server(
        &self,
        ctx: &Context<'_>,
        server_id: i64,
    ) -> GraphqlResult<AuthResponse> {
        let app_state = ctx
            .data::<Arc<AppState>>()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get app state: {:?}", e))?;
        let db = &app_state.db;

        // Verify server exists
        let _server = entities::plex_server::Entity::find_by_id(server_id)
            .one(&db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to find plex server: {}", e))?;

        _server.ok_or_else(|| {
            color_eyre::eyre::eyre!("Plex server with id {} not found", server_id)
        })?;

        // Create reqwest client
        let client = Client::new();

        // Create Plex PIN
        let pin = create_plex_pin(&client)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to create plex pin: {}", e))?;

        // Construct auth URL with forward URL
        let forward_url = format!("{}/plex-auth/callback", app_state.base_url);
        let auth_url = construct_auth_app_url(&pin.code, &forward_url)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to construct auth URL: {}", e))?;

        Ok(AuthResponse {
            auth_url,
            pin_id: pin.id,
        })
    }

    async fn complete_plex_server_authentication(
        &self,
        ctx: &Context<'_>,
        server_id: i64,
        pin_id: i32,
    ) -> GraphqlResult<PlexServer> {
        let app_state = ctx
            .data::<Arc<AppState>>()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get app state: {:?}", e))?;
        let db = &app_state.db;

        // Verify server exists
        let server = entities::plex_server::Entity::find_by_id(server_id)
            .one(&db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to find plex server: {}", e))?;

        let server_model = server.ok_or_else(|| {
            color_eyre::eyre::eyre!("Plex server with id {} not found", server_id)
        })?;

        // Create reqwest client
        let client = Client::new();

        // Poll for auth token with timeout/retry logic
        let mut user_token: Option<String> = None;
        for _ in 0..30 {
            // Poll up to 30 times (30 seconds total)
            let auth_response = poll_for_plex_auth(&client, pin_id)
                .await
                .map_err(|e| color_eyre::eyre::eyre!("Failed to poll for plex auth: {}", e))?;

            if let Some(token) = auth_response.auth_token {
                user_token = Some(token);
                break;
            }

            // Wait 1 second before next poll
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }

        let user_token = user_token.ok_or_else(|| {
            color_eyre::eyre::eyre!("Authentication timeout: PIN was not claimed within 30 seconds")
        })?;

        // Get Plex resources to find server access token
        let resources = get_plex_resources(&client, &user_token)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get plex resources: {}", e))?;

        // TODO: match by server_url instead of name
        // Find matching resource by server name
        let matching_resource = resources
            .into_iter()
            .find(|resource| resource.name == server_model.name);

        let access_token = matching_resource
            .and_then(|r| r.access_token)
            .ok_or_else(|| {
                color_eyre::eyre::eyre!(
                    "No matching Plex server found or server has no access token"
                )
            })?;

        // Update server with access token
        let mut server_active: entities::plex_server::ActiveModel = server_model.into();
        server_active.access_token = Set(Some(access_token));

        let updated_server = server_active
            .update(&db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to update plex server: {}", e))?;

        Ok(PlexServer {
            id: updated_server.id,
            name: updated_server.name,
            server_url: updated_server.server_url,
            has_access_token: updated_server.access_token.is_some(),
            created_at: updated_server.created_at,
            updated_at: updated_server.updated_at,
        })
    }
}
