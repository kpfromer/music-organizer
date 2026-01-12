use serde::{Deserialize, Serialize};

/// Spotify OAuth token response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: Option<String>,
    pub scope: String,
}

/// Spotify user profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyUser {
    pub id: String,
    pub display_name: Option<String>,
}

/// Spotify playlist from API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyPlaylist {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub snapshot_id: String,
    pub tracks: SpotifyPlaylistTracks,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyPlaylistTracks {
    pub total: i32,
}

/// Spotify track from API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyTrack {
    pub id: String,
    pub name: String,
    pub artists: Vec<SpotifyArtist>,
    pub album: SpotifyAlbum,
    pub duration_ms: Option<i64>,
    pub external_ids: Option<SpotifyExternalIds>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyArtist {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyAlbum {
    pub id: String,
    pub name: String,
    pub external_ids: Option<SpotifyExternalIds>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyExternalIds {
    pub isrc: Option<String>,
    pub ean: Option<String>,
    pub upc: Option<String>,
}

/// Status of playlist sync operation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SyncStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

/// Result of a single track download attempt
#[derive(Debug, Clone)]
pub enum DownloadResult {
    AlreadyExists { track_id: i64 },
    Downloaded { track_id: i64 },
    Failed { reason: String },
}

/// Result of a playlist sync operation
#[derive(Debug, Clone)]
pub struct PlaylistSyncResult {
    pub total_tracks: i32,
    pub already_downloaded: i32,
    pub newly_downloaded: i32,
    pub failed: i32,
    pub failed_tracks: Vec<FailedTrack>,
}

#[derive(Debug, Clone)]
pub struct FailedTrack {
    pub spotify_track: SpotifyTrack,
    pub reason: String,
}

/// PKCE OAuth session data
#[derive(Debug, Clone)]
pub struct OAuthSession {
    pub code_verifier: String,
    pub state: String,
    pub created_at: i64,
}

/// Response for authentication initiation
#[derive(Debug, Clone)]
pub struct SpotifyAuthResponse {
    pub auth_url: String,
    pub state: String,
}
