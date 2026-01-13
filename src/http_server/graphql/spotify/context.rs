use async_graphql::Context;
use color_eyre::eyre::{OptionExt, WrapErr};

use crate::{
    http_server::{graphql::context::get_app_state, graphql_error::GraphqlError},
    services::spotify::client::SPOTIFY_SCOPES,
};
use spotify_rs::{Token, UnknownFlow, client::Client as SpotifyClient};

pub async fn get_spotify_client<'a>(
    ctx: &Context<'a>,
    spotify_account: crate::entities::spotify_account::Model,
) -> Result<SpotifyClient<Token, UnknownFlow>, GraphqlError> {
    let app_state = get_app_state(ctx)?;
    let refresh_token = spotify_account.refresh_token;
    let credentials = app_state
        .spotify_credentials
        .as_ref()
        .ok_or_eyre("Spotify credentials not found")?;

    let spotify_client = spotify_rs::client::Client::from_refresh_token(
        credentials.client_id(),
        credentials.client_secret(),
        Some(SPOTIFY_SCOPES.to_vec().into()),
        true,
        refresh_token,
    )
    .await
    .wrap_err("Failed to create spotify client")?;

    Ok(spotify_client)
}
