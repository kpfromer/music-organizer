use std::sync::Arc;

use async_graphql::{Context, Union};
use reqwest::Client;
use url::Url;

use crate::entities;
use crate::http_server::graphql_error::GraphqlResult;
use crate::http_server::state::AppState;
use crate::plex_rs::all_tracks::{find_music_section_id, get_library_sections, get_tracks_page};
use sea_orm::EntityTrait;

#[derive(Debug, Clone, async_graphql::SimpleObject)]
pub struct PlexTrack {
    pub title: String,
    pub album: Option<String>,
    pub artist: Option<String>,
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
pub struct PlexTracksSuccess {
    pub tracks: Vec<PlexTrack>,
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
pub struct NoPlexServerError {
    pub message: String,
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
pub struct MultiplePlexServersError {
    pub message: String,
    pub server_count: i32,
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
pub struct PlexTracksError {
    pub message: String,
}

#[derive(Union)]
pub enum PlexTracksResult {
    Success(PlexTracksSuccess),
    NoPlexServer(NoPlexServerError),
    MultiplePlexServers(MultiplePlexServersError),
    Error(PlexTracksError),
}

/// Fetch up to 50 tracks from the configured Plex server.
/// Returns a union type that can be either success or one of several error types.
pub async fn plex_tracks(ctx: &Context<'_>) -> GraphqlResult<PlexTracksResult> {
    let app_state = ctx
        .data::<Arc<AppState>>()
        .map_err(|e| color_eyre::eyre::eyre!("Failed to get app state: {:?}", e))?;
    let db = &app_state.db;

    // Fetch all plex servers
    let servers = entities::plex_server::Entity::find()
        .all(&db.conn)
        .await
        .map_err(|e| color_eyre::eyre::eyre!("Failed to fetch plex servers: {}", e))?;

    // Check for no servers
    if servers.is_empty() {
        return Ok(PlexTracksResult::NoPlexServer(NoPlexServerError {
            message: "No Plex server configured. Please add a Plex server first.".to_string(),
        }));
    }

    // Check for multiple servers
    if servers.len() > 1 {
        return Ok(PlexTracksResult::MultiplePlexServers(
            MultiplePlexServersError {
                message: format!(
                    "Multiple Plex servers found ({}). Only one server is supported at a time.",
                    servers.len()
                ),
                server_count: servers.len() as i32,
            },
        ));
    }

    // Get the single server
    let server = servers.into_iter().next().unwrap();

    // Check if server has access token
    let access_token = match &server.access_token {
        Some(token) => token.clone(),
        None => {
            return Ok(PlexTracksResult::Error(PlexTracksError {
                    message: "Plex server does not have an access token. Please authenticate the server first.".to_string(),
                }));
        }
    };

    // Parse server URL
    let server_url = match Url::parse(&server.server_url) {
        Ok(url) => url,
        Err(e) => {
            return Ok(PlexTracksResult::Error(PlexTracksError {
                message: format!("Invalid server URL: {}", e),
            }));
        }
    };

    // Create HTTP client
    let client = Client::new();

    // Get library sections
    let sections = match get_library_sections(&client, &server_url, &access_token).await {
        Ok(sections) => sections,
        Err(e) => {
            return Ok(PlexTracksResult::Error(PlexTracksError {
                message: format!("Failed to fetch library sections: {}", e),
            }));
        }
    };

    // Find music section
    let music_section_id = match find_music_section_id(&sections) {
        Some(id) => id,
        None => {
            return Ok(PlexTracksResult::Error(PlexTracksError {
                message: "No music library section found on Plex server.".to_string(),
            }));
        }
    };

    // Fetch tracks (limit to 50)
    let container =
        match get_tracks_page(&client, &server_url, &access_token, music_section_id, 0, 50).await {
            Ok(container) => container,
            Err(e) => {
                return Ok(PlexTracksResult::Error(PlexTracksError {
                    message: format!("Failed to fetch tracks: {}", e),
                }));
            }
        };

    // Convert to GraphQL types
    let tracks: Vec<PlexTrack> = container
        .metadata
        .into_iter()
        .map(|track| PlexTrack {
            title: track.title,
            album: track.album,
            artist: track.artist,
        })
        .collect();

    Ok(PlexTracksResult::Success(PlexTracksSuccess { tracks }))
}
