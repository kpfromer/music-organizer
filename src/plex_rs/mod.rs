use color_eyre::eyre::Context;
use reqwest::Client;
use url::Url;

pub mod all_tracks;
pub mod auth;
pub mod playlist;
pub mod sync_playlist;

// Re-export authentication items for backward compatibility
pub use auth::{
    APP_IDENTIFIER, APP_NAME, PlexAuthResponse, PlexPinResponse, PlexResource,
    construct_auth_app_url, create_plex_pin, get_plex_resources, poll_for_plex_auth,
};

/// Docs:
/// https://developer.plex.tv/pms/#section/API-Info/Authenticating-with-Plex
struct PlexConfig {
    server_url: Url,
}

struct PlexClient {
    client: Client,
}

/// Fetches Plex playlists (type=15) from a running Plex Media Server instance using a user's Plex token.
///
/// # Arguments
/// * `client` - reqwest::Client instance to use for the request
/// * `base_url` - Base URL for the PMS, e.g., "http://127.0.0.1:32400"
/// * `user_token` - Plex user access token
///
/// # Returns
/// The raw JSON response as serde_json::Value
pub async fn get_plex_playlists(
    client: &Client,
    base_url: &Url,
    user_token: &str,
) -> color_eyre::Result<serde_json::Value> {
    let url = base_url.join("playlists?type=15")?;
    let res = client
        .get(url)
        .header("Accept", "application/json")
        .header("X-Plex-Token", user_token)
        .send()
        .await?
        .error_for_status()?
        .json::<serde_json::Value>()
        .await
        .wrap_err("Failed to deserialize Plex playlists response")?;
    Ok(res)
}
