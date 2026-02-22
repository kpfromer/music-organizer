use async_graphql::{Context, Object, SimpleObject};

use crate::http_server::graphql::context::get_app_state;
use crate::http_server::graphql_error::GraphqlResult;
use crate::services::plex::PlexService;
use crate::services::plex::client::PlexHttpAdapter;

#[derive(Debug, Clone, SimpleObject)]
pub struct LibraryScanStatus {
    pub is_scanning: bool,
    pub progress: Option<f64>,
    pub title: Option<String>,
    pub subtitle: Option<String>,
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
        let app_state = get_app_state(ctx)?;
        let service = PlexService::new(app_state.db.clone(), PlexHttpAdapter::new());
        let activity = service.get_scan_status(plex_server_id).await?;

        Ok(match activity {
            Some(activity) => {
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
}
