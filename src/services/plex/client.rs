use color_eyre::eyre::Result;
use reqwest::Client;
use url::Url;

use crate::plex_rs::all_tracks::{
    PlexLibrarySection, PlexLibraryTrack, PlexMediaContainer, find_music_section_id,
    get_library_sections, get_tracks_page,
};
use crate::plex_rs::auth::{
    PlexAuthResponse, PlexPinResponse, PlexResource, construct_auth_app_url, create_plex_pin,
    get_plex_resources, poll_for_plex_auth,
};
use crate::plex_rs::library_refresh::{
    PlexActivity, get_library_scan_status, refresh_library_section,
};
use crate::plex_rs::playlist::{PlexPlaylist, get_playlists};
use crate::ports::plex::PlexClient;

pub struct PlexHttpAdapter {
    client: Client,
}

impl PlexHttpAdapter {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl PlexClient for PlexHttpAdapter {
    async fn create_pin(&self) -> Result<PlexPinResponse> {
        create_plex_pin(&self.client).await
    }

    fn construct_auth_url(&self, pin_code: &str, forward_url: &str) -> Result<String> {
        construct_auth_app_url(pin_code, forward_url)
    }

    async fn poll_for_auth(&self, pin_id: i32) -> Result<PlexAuthResponse> {
        poll_for_plex_auth(&self.client, pin_id).await
    }

    async fn get_resources(&self, user_token: &str) -> Result<Vec<PlexResource>> {
        get_plex_resources(&self.client, user_token).await
    }

    async fn get_library_sections(
        &self,
        server_url: &Url,
        token: &str,
    ) -> Result<Vec<PlexLibrarySection>> {
        get_library_sections(&self.client, server_url, token).await
    }

    fn find_music_section_id<'a>(&self, sections: &'a [PlexLibrarySection]) -> Option<&'a str> {
        find_music_section_id(sections)
    }

    async fn get_tracks_page(
        &self,
        server_url: &Url,
        token: &str,
        section_id: &str,
        start: u32,
        size: u32,
    ) -> Result<PlexMediaContainer<PlexLibraryTrack>> {
        get_tracks_page(&self.client, server_url, token, section_id, start, size).await
    }

    async fn refresh_library_section(
        &self,
        server_url: &Url,
        token: &str,
        section_id: &str,
    ) -> Result<()> {
        refresh_library_section(&self.client, server_url, token, section_id).await
    }

    async fn get_library_scan_status(
        &self,
        server_url: &Url,
        token: &str,
        section_id: &str,
    ) -> Result<Option<PlexActivity>> {
        get_library_scan_status(&self.client, server_url, token, section_id).await
    }

    async fn get_playlists(&self, server_url: &Url, token: &str) -> Result<Vec<PlexPlaylist>> {
        get_playlists(&self.client, server_url, token).await
    }
}
