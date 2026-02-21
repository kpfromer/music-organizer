use async_graphql::{Context, Object, SimpleObject};

use crate::http_server::graphql::context::get_app_state;
use crate::http_server::graphql_error::GraphqlResult;
use crate::services::plex::PlexService;
use crate::services::plex::client::PlexHttpAdapter;

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
        let app_state = get_app_state(ctx)?;
        let service = PlexService::new(app_state.db.clone(), PlexHttpAdapter::new());
        let music_section_id = service.refresh_music_library(plex_server_id).await?;

        Ok(RefreshLibraryResult {
            success: true,
            message: format!("Library refresh started for section {}", music_section_id),
            section_id: music_section_id,
        })
    }
}
