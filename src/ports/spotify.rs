use color_eyre::eyre::Result;

/// Decoupled representation of a Spotify playlist from the API.
#[derive(Debug, Clone)]
pub struct SpotifyApiPlaylist {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub snapshot_id: String,
    pub total_tracks: i32,
}

/// Decoupled representation of a Spotify track from the API.
#[derive(Debug, Clone)]
pub struct SpotifyApiTrack {
    pub id: String,
    pub name: String,
    pub duration_ms: i32,
    pub artists: Vec<String>,
    pub album_name: String,
    pub isrc: Option<String>,
    pub upc: Option<String>,
}

/// Port trait wrapping the Spotify API capabilities used by business logic.
///
/// Implementations live in `services::spotify::client` (production) or test mocks.
pub trait SpotifyClient: Send + Sync {
    fn current_user_playlists(
        &self,
    ) -> impl Future<Output = Result<Vec<SpotifyApiPlaylist>>> + Send;
    fn playlist_tracks(
        &self,
        playlist_id: &str,
    ) -> impl Future<Output = Result<Vec<SpotifyApiTrack>>> + Send;
}
