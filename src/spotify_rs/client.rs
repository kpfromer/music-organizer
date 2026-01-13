#![allow(dead_code)]
use std::time::Duration;

use color_eyre::Result;
use serde::Deserialize;

use crate::spotify_rs::types::{SpotifyPlaylist, SpotifyTrack, SpotifyUser};

/// Spotify API client
pub struct SpotifyClient {
    access_token: String,
    client: reqwest::Client,
}

impl SpotifyClient {
    pub fn new(access_token: String) -> Self {
        Self {
            access_token,
            client: reqwest::Client::new(),
        }
    }

    /// Get the current user's profile
    pub async fn get_current_user(&self) -> Result<SpotifyUser> {
        let response = self
            .client
            .get("https://api.spotify.com/v1/me")
            .bearer_auth(&self.access_token)
            .timeout(Duration::from_secs(10))
            .send()
            .await?
            .error_for_status()?;

        let user: SpotifyUser = response.json().await?;
        Ok(user)
    }

    /// Get all playlists for the current user
    pub async fn get_user_playlists(&self) -> Result<Vec<SpotifyPlaylist>> {
        let mut all_playlists = Vec::new();
        let mut next_url = Some("https://api.spotify.com/v1/me/playlists?limit=50".to_string());

        while let Some(url) = next_url {
            let response = self
                .client
                .get(&url)
                .bearer_auth(&self.access_token)
                .timeout(Duration::from_secs(10))
                .send()
                .await?
                .error_for_status()?;

            #[derive(Deserialize)]
            struct PlaylistsResponse {
                items: Vec<SpotifyPlaylist>,
                next: Option<String>,
            }

            let page: PlaylistsResponse = response.json().await?;
            all_playlists.extend(page.items);
            next_url = page.next;
        }

        Ok(all_playlists)
    }

    /// Get all tracks in a playlist
    pub async fn get_playlist_tracks(&self, playlist_id: &str) -> Result<Vec<SpotifyTrack>> {
        let mut all_tracks = Vec::new();
        let mut next_url = Some(format!(
            "https://api.spotify.com/v1/playlists/{}/tracks?limit=100",
            playlist_id
        ));

        while let Some(url) = next_url {
            let response = self
                .client
                .get(&url)
                .bearer_auth(&self.access_token)
                .timeout(Duration::from_secs(10))
                .send()
                .await?
                .error_for_status()?;

            #[derive(Deserialize)]
            struct PlaylistTrackObject {
                track: Option<SpotifyTrack>,
            }

            #[derive(Deserialize)]
            struct TracksResponse {
                items: Vec<PlaylistTrackObject>,
                next: Option<String>,
            }

            let page: TracksResponse = response.json().await?;
            for item in page.items {
                if let Some(track) = item.track {
                    all_tracks.push(track);
                }
            }
            next_url = page.next;
        }

        Ok(all_tracks)
    }
}
