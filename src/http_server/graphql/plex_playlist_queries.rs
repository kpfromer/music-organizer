use async_graphql::{Context, SimpleObject};

use crate::http_server::graphql::context::get_app_state;
use crate::http_server::graphql_error::GraphqlResult;
use crate::plex_rs::playlist::is_music_playlist;
use crate::services::plex::PlexService;
use crate::services::plex::client::PlexHttpAdapter;

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
    let app_state = get_app_state(ctx)?;
    let service = PlexService::new(app_state.db.clone(), PlexHttpAdapter::new());
    let plex_playlists = service.get_playlists().await?;

    let music_playlists: Vec<PlexPlaylist> = plex_playlists
        .into_iter()
        .filter(is_music_playlist)
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
