use color_eyre::eyre::Result;
use spotify_rs::AuthCodeClient;
use spotify_rs::AuthCodeFlow;
use spotify_rs::RedirectUrl;
use spotify_rs::Token;
use spotify_rs::Unauthenticated;
use spotify_rs::UnknownFlow;
use spotify_rs::client::Client as SpotifyClient;
use url::Url;

use crate::entities;

pub const SPOTIFY_SCOPES: [&str; 4] = [
    "user-read-email",
    "user-read-private",
    "playlist-read-private",
    "playlist-read-collaborative",
];

#[derive(Debug, Clone)]
pub struct SpotifyApiCredentials {
    client_id: String,
    client_secret: String,
    redirect_uri: RedirectUrl,
}

impl SpotifyApiCredentials {
    pub fn new(client_id: String, client_secret: String, redirect_uri: RedirectUrl) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_uri,
        }
    }

    pub fn client_secret(&self) -> Option<&str> {
        Some(&self.client_secret)
    }

    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    pub fn redirect_uri(&self) -> &RedirectUrl {
        &self.redirect_uri
    }
}

pub fn start_spotify_auth_flow(
    credentials: SpotifyApiCredentials,
) -> (SpotifyClient<Unauthenticated, AuthCodeFlow>, Url) {
    // Whether or not to automatically refresh the token when it expires.
    let auto_refresh = true;

    // You will need to redirect the user to this URL.
    AuthCodeClient::new(
        credentials.client_id,
        credentials.client_secret,
        SPOTIFY_SCOPES.to_vec(),
        credentials.redirect_uri,
        auto_refresh,
    )
}

pub async fn complete_spotify_auth_flow(
    client: SpotifyClient<Unauthenticated, AuthCodeFlow>,
    auth_code: String,
    csrf_state: String,
) -> Result<SpotifyClient<Token, AuthCodeFlow>, spotify_rs::Error> {
    // After the user was redirected to `url`, they will be redirected *again*, to
    // your `redirect_uri`, with the "auth_code" and "csrf_state" parameters in the URL.
    // You will need to get those parameters from the URL.

    // Finally, you will be able to authenticate the client.
    client.authenticate(auth_code, csrf_state).await
}

pub async fn get_existing_spotify_client_from_db(
    credentials: SpotifyApiCredentials,
    spotify_account: entities::spotify_account::Model,
) -> Result<SpotifyClient<Token, UnknownFlow>, spotify_rs::Error> {
    let auto_refresh = true;
    let refresh_token_string = spotify_account.refresh_token;
    SpotifyClient::from_refresh_token(
        credentials.client_id,
        Some(&credentials.client_secret),
        Some(SPOTIFY_SCOPES.to_vec().into()),
        auto_refresh,
        refresh_token_string,
    )
    .await
}
