use spotify_rs::AuthCodeClient;
use spotify_rs::AuthCodeFlow;
use spotify_rs::RedirectUrl;
use spotify_rs::Unauthenticated;
use spotify_rs::client::Client as SpotifyClient;
use url::Url;

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
