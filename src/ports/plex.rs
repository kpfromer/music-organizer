use color_eyre::eyre::Result;
use url::Url;

use crate::plex_rs::all_tracks::{PlexLibrarySection, PlexLibraryTrack, PlexMediaContainer};
use crate::plex_rs::auth::{PlexAuthResponse, PlexPinResponse, PlexResource};
use crate::plex_rs::library_refresh::PlexActivity;
use crate::plex_rs::playlist::PlexPlaylist;

/// Port trait wrapping the Plex API capabilities used by business logic.
///
/// Implementations live in `services::plex::client` (production) or test mocks.
#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait PlexClient: Send + Sync {
    async fn create_pin(&self) -> Result<PlexPinResponse>;

    fn construct_auth_url(&self, pin_code: &str, forward_url: &str) -> Result<String>;

    async fn poll_for_auth(&self, pin_id: i32) -> Result<PlexAuthResponse>;

    async fn get_resources(&self, user_token: &str) -> Result<Vec<PlexResource>>;

    async fn get_library_sections(
        &self,
        server_url: &Url,
        token: &str,
    ) -> Result<Vec<PlexLibrarySection>>;

    fn find_music_section_id(&self, sections: &[PlexLibrarySection]) -> Option<String>;

    async fn get_tracks_page(
        &self,
        server_url: &Url,
        token: &str,
        section_id: &str,
        start: u32,
        size: u32,
    ) -> Result<PlexMediaContainer<PlexLibraryTrack>>;

    async fn refresh_library_section(
        &self,
        server_url: &Url,
        token: &str,
        section_id: &str,
    ) -> Result<()>;

    async fn get_library_scan_status(
        &self,
        server_url: &Url,
        token: &str,
        section_id: &str,
    ) -> Result<Option<PlexActivity>>;

    async fn get_playlists(&self, server_url: &Url, token: &str) -> Result<Vec<PlexPlaylist>>;
}
