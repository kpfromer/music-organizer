use color_eyre::eyre::OptionExt;

use crate::http_server::graphql_error::GraphqlError;
use crate::http_server::state::AppState;
use crate::services::spotify::client::SpotifyRsAdapter;

pub async fn get_spotify_adapter(
    app_state: &AppState,
    spotify_account: crate::entities::spotify_account::Model,
) -> Result<SpotifyRsAdapter, GraphqlError> {
    let credentials = app_state
        .spotify_credentials
        .as_ref()
        .ok_or_eyre("Spotify credentials not found")?;

    let adapter =
        SpotifyRsAdapter::from_refresh_token(credentials, spotify_account.refresh_token).await?;

    Ok(adapter)
}
