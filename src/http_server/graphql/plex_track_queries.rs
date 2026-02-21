use async_graphql::{Context, Union};

use crate::http_server::graphql::context::get_app_state;
use crate::http_server::graphql_error::GraphqlResult;
use crate::services::plex::client::PlexHttpAdapter;
use crate::services::plex::{PlexService, PlexTracksOutcome};

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
    let app_state = get_app_state(ctx)?;
    let service = PlexService::new(app_state.db.clone(), PlexHttpAdapter::new());

    let outcome = service.get_tracks().await?;

    Ok(match outcome {
        PlexTracksOutcome::NoServer => PlexTracksResult::NoPlexServer(NoPlexServerError {
            message: "No Plex server configured. Please add a Plex server first.".to_string(),
        }),
        PlexTracksOutcome::MultipleServers(count) => {
            PlexTracksResult::MultiplePlexServers(MultiplePlexServersError {
                message: format!(
                    "Multiple Plex servers found ({}). Only one server is supported at a time.",
                    count
                ),
                server_count: count as i32,
            })
        }
        PlexTracksOutcome::NoToken => PlexTracksResult::Error(PlexTracksError {
            message:
                "Plex server does not have an access token. Please authenticate the server first."
                    .to_string(),
        }),
        PlexTracksOutcome::Error(msg) => PlexTracksResult::Error(PlexTracksError { message: msg }),
        PlexTracksOutcome::Success(container) => {
            let tracks: Vec<PlexTrack> = container
                .metadata
                .into_iter()
                .map(|track| PlexTrack {
                    title: track.title,
                    album: track.album,
                    artist: track.artist,
                })
                .collect();
            PlexTracksResult::Success(PlexTracksSuccess { tracks })
        }
    })
}
