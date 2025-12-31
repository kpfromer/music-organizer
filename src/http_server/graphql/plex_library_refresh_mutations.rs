use std::sync::Arc;

use async_graphql::{Context, Object, SimpleObject};
use color_eyre::eyre::OptionExt;
use reqwest::Client;
use sea_orm::EntityTrait;

use crate::entities;
use crate::http_server::graphql_error::GraphqlResult;
use crate::http_server::state::AppState;
use crate::plex_rs::all_tracks::{find_music_section_id, get_library_sections};
use crate::plex_rs::library_refresh::refresh_library_section;

#[derive(Debug, Clone, SimpleObject)]
pub struct RefreshLibraryResult {
    pub success: bool,
    pub message: String,
    pub section_id: String,
}

#[derive(Default)]
pub struct PlexLibraryRefreshMutation;

#[Object]
impl PlexLibraryRefreshMutation {
    /// Trigger a refresh/rescan of the music library on a Plex server
    async fn refresh_music_library(
        &self,
        ctx: &Context<'_>,
        plex_server_id: i64,
    ) -> GraphqlResult<RefreshLibraryResult> {
        let app_state = ctx
            .data::<Arc<AppState>>()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get app state: {:?}", e))?;
        let db = &app_state.db;

        // Get Plex server from database
        let server = entities::plex_server::Entity::find_by_id(plex_server_id)
            .one(&db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to fetch Plex server: {}", e))?
            .ok_or_eyre("Plex server not found")?;

        let access_token = server.access_token.as_ref().ok_or_eyre(
            "Plex server does not have an access token. Please authenticate the server first.",
        )?;

        let server_url = url::Url::parse(&server.server_url)
            .map_err(|e| color_eyre::eyre::eyre!("Invalid server URL: {}", e))?;

        // Get library sections and find music section
        let client = Client::new();
        let sections = get_library_sections(&client, &server_url, access_token).await?;
        let music_section_id = find_music_section_id(&sections)
            .ok_or_eyre("No music library section found on Plex server")?;

        // Trigger refresh
        refresh_library_section(&client, &server_url, access_token, &music_section_id).await?;

        Ok(RefreshLibraryResult {
            success: true,
            message: format!("Library refresh started for section {}", music_section_id),
            section_id: music_section_id.to_string(),
        })
    }
}
