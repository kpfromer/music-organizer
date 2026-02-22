use color_eyre::eyre::WrapErr;
use spotify_rs::AuthCodeClient;
use spotify_rs::AuthCodeFlow;
use spotify_rs::RedirectUrl;
use spotify_rs::Token;
use spotify_rs::Unauthenticated;
use spotify_rs::UnknownFlow;
use spotify_rs::client::Client as SpotifyRsClient;
use url::Url;

use crate::ports::spotify::{SpotifyApiPlaylist, SpotifyApiTrack, SpotifyClient};

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
) -> (SpotifyRsClient<Unauthenticated, AuthCodeFlow>, Url) {
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

/// Adapter that implements `SpotifyClient` port using the `spotify_rs` crate.
pub struct SpotifyRsAdapter {
    client: SpotifyRsClient<Token, UnknownFlow>,
}

impl SpotifyRsAdapter {
    pub async fn from_refresh_token(
        credentials: &SpotifyApiCredentials,
        refresh_token: String,
    ) -> color_eyre::eyre::Result<Self> {
        let client = SpotifyRsClient::from_refresh_token(
            credentials.client_id(),
            credentials.client_secret(),
            Some(SPOTIFY_SCOPES.to_vec().into()),
            true,
            refresh_token,
        )
        .await
        .wrap_err("Failed to create spotify client")?;

        Ok(Self { client })
    }
}

#[async_trait::async_trait]
impl SpotifyClient for SpotifyRsAdapter {
    async fn current_user_playlists(&self) -> color_eyre::eyre::Result<Vec<SpotifyApiPlaylist>> {
        let pages = spotify_rs::current_user_playlists()
            .get(&self.client)
            .await
            .wrap_err("Failed to fetch user spotify playlists")?
            .get_all(&self.client)
            .await
            .wrap_err("Unable to get all user spotify playlists")?;

        Ok(pages
            .into_iter()
            .flatten()
            .map(|p| SpotifyApiPlaylist {
                id: p.id,
                name: p.name,
                description: p.description,
                snapshot_id: p.snapshot_id,
                total_tracks: p.tracks.map(|t| t.total).unwrap_or(0) as i32,
            })
            .collect())
    }

    async fn playlist_tracks(
        &self,
        playlist_id: &str,
    ) -> color_eyre::eyre::Result<Vec<SpotifyApiTrack>> {
        let pages = spotify_rs::playlist_items(playlist_id)
            .get(&self.client)
            .await
            .wrap_err("Failed to fetch spotify tracks from api")?
            .get_all(&self.client)
            .await
            .wrap_err("Unable to get all spotify playlist items")?;

        Ok(pages
            .into_iter()
            .flatten()
            .filter_map(|item| {
                if let spotify_rs::model::PlayableItem::Track(track) = item.track {
                    Some(SpotifyApiTrack {
                        id: track.id,
                        name: track.name,
                        duration_ms: track.duration_ms as i32,
                        artists: track.artists.iter().map(|a| a.name.clone()).collect(),
                        album_name: track.album.name,
                        isrc: track.external_ids.isrc,
                        upc: track.external_ids.upc,
                    })
                } else {
                    None
                }
            })
            .collect())
    }
}
