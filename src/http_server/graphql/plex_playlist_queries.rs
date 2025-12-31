use std::sync::Arc;

use async_graphql::{Context, SimpleObject};
use color_eyre::eyre::OptionExt;
use reqwest::Client;
use sea_orm::EntityTrait;
use url::Url;

use crate::entities;
use crate::http_server::graphql_error::GraphqlResult;
use crate::http_server::state::AppState;
use crate::plex_rs::playlist::{get_playlists, is_music_playlist};

#[derive(Debug, Clone, SimpleObject)]
pub struct PlexPlaylist {
    pub rating_key: String,
    pub title: String,
    pub playlist_type: String,
    pub leaf_count: Option<u32>,
    pub duration: Option<u64>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct PlexPlaylistsResponse {
    pub playlists: Vec<PlexPlaylist>,
}

/// Fetch all music playlists from Plex
pub async fn plex_playlists(ctx: &Context<'_>) -> GraphqlResult<PlexPlaylistsResponse> {
    let app_state = ctx
        .data::<Arc<AppState>>()
        .map_err(|e| color_eyre::eyre::eyre!("Failed to get app state: {:?}", e))?;
    let db = &app_state.db;

    // Fetch all plex servers
    let servers = entities::plex_server::Entity::find()
        .all(&db.conn)
        .await
        .map_err(|e| color_eyre::eyre::eyre!("Failed to fetch plex servers: {}", e))?;

    if servers.is_empty() {
        return Err(color_eyre::eyre::eyre!(
            "No Plex server configured. Please add a Plex server first."
        )
        .into());
    }

    if servers.len() > 1 {
        return Err(color_eyre::eyre::eyre!(
            "Multiple Plex servers found ({}). Only one server is supported at a time.",
            servers.len()
        )
        .into());
    }

    let server = servers.into_iter().next().unwrap();

    let access_token = server.access_token.as_ref().ok_or_eyre(
        "Plex server does not have an access token. Please authenticate the server first.",
    )?;

    let server_url = Url::parse(&server.server_url)
        .map_err(|e| color_eyre::eyre::eyre!("Invalid server URL: {}", e))?;

    let client = Client::new();
    let plex_playlists = get_playlists(&client, &server_url, access_token).await?;

    let music_playlists: Vec<PlexPlaylist> = plex_playlists
        .into_iter()
        .filter(|p| is_music_playlist(p))
        .map(|p| PlexPlaylist {
            rating_key: p.rating_key,
            title: p.title,
            playlist_type: p.playlist_type,
            leaf_count: p.leaf_count,
            duration: p.duration,
        })
        .collect();

    Ok(PlexPlaylistsResponse {
        playlists: music_playlists,
    })
}
