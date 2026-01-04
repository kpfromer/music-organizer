use std::sync::Arc;

use async_graphql::{Context, Object, SimpleObject};
use color_eyre::eyre::OptionExt;
use reqwest::Client;
use sea_orm::EntityTrait;

use crate::entities;
use crate::http_server::graphql_error::GraphqlResult;
use crate::http_server::state::AppState;
use crate::plex_rs::all_tracks::{find_music_section_id, get_library_sections};
use crate::plex_rs::library_refresh::get_library_scan_status;

#[derive(Debug, Clone, SimpleObject)]
pub struct LibraryScanStatus {
    pub is_scanning: bool,
    pub progress: Option<f64>,
    pub title: Option<String>,
    pub subtitle: Option<String>,
}

/// Get the current scan status for the music library on a Plex server
pub async fn music_library_scan_status(
    ctx: &Context<'_>,
    plex_server_id: i64,
) -> GraphqlResult<LibraryScanStatus> {
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

    // Check scan status
    let activity =
        get_library_scan_status(&client, &server_url, access_token, music_section_id).await?;

    Ok(match activity {
        Some(activity) => {
            // Plex returns progress as 0-100, convert to 0-1 for consistency
            let progress = activity.progress.map(|p| p / 100.0);
            LibraryScanStatus {
                is_scanning: true,
                progress,
                title: Some(activity.title),
                subtitle: activity.subtitle,
            }
        }
        None => LibraryScanStatus {
            is_scanning: false,
            progress: None,
            title: None,
            subtitle: None,
        },
    })
}

#[derive(Default)]
pub struct PlexLibraryRefreshQuery;

#[Object]
impl PlexLibraryRefreshQuery {
    /// Get the current scan status for the music library on a Plex server
    async fn music_library_scan_status(
        &self,
        ctx: &Context<'_>,
        plex_server_id: i64,
    ) -> GraphqlResult<LibraryScanStatus> {
        music_library_scan_status(ctx, plex_server_id).await
    }
}
