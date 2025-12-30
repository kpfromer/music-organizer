use color_eyre::eyre::{OptionExt, Result, WrapErr};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use url::Url;

/* ---------- Core response envelope ---------- */

#[derive(Debug, Clone, Deserialize)]
pub struct PlexResponse<T> {
    #[serde(rename = "MediaContainer")]
    pub media_container: PlexMediaContainer<T>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlexMediaContainer<T> {
    #[serde(default)]
    pub size: Option<u32>,
    #[serde(rename = "totalSize")]
    pub total_size: Option<u32>,
    #[serde(default)]
    pub offset: Option<u32>,

    #[serde(rename = "Metadata")]
    pub metadata: Vec<T>,
}

/* ---------- Identity (machineIdentifier) ---------- */

#[derive(Debug, Clone, Deserialize)]
pub struct PlexIdentity {
    #[serde(rename = "machineIdentifier")]
    pub machine_identifier: String,
}

pub async fn get_machine_identifier(
    client: &Client,
    base_url: &Url,
    user_token: &str,
) -> Result<String> {
    let url = base_url.join("identity")?;

    let res = client
        .get(url)
        .header("Accept", "application/json")
        .header("X-Plex-Token", user_token)
        .send()
        .await?
        .error_for_status()?
        .json::<PlexResponse<PlexIdentity>>()
        .await
        .wrap_err("Failed to deserialize Plex identity response")?;

    let identity = res
        .media_container
        .metadata
        .into_iter()
        .next()
        .ok_or_eyre("Plex identity response had no Metadata")?;

    Ok(identity.machine_identifier)
}

/* ---------- Playlists ---------- */

#[derive(Debug, Clone, Deserialize)]
pub struct PlexPlaylist {
    #[serde(rename = "ratingKey")]
    pub rating_key: String,

    pub title: String,

    #[serde(rename = "playlistType")]
    pub playlist_type: String,

    #[serde(default)]
    pub smart: Option<bool>,

    #[serde(rename = "leafCount", default)]
    pub leaf_count: Option<u32>,

    #[serde(default)]
    pub duration: Option<u64>,

    #[serde(default)]
    pub summary: Option<String>,

    #[serde(default)]
    pub key: Option<String>,

    #[serde(default)]
    pub composite: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlexPlaylistList {
    #[serde(rename = "MediaContainer")]
    pub media_container: PlexMediaContainer<PlexPlaylist>,
}

pub async fn get_playlists(
    client: &Client,
    base_url: &Url,
    user_token: &str,
) -> Result<Vec<PlexPlaylist>> {
    let url = base_url.join("playlists?type=15")?;

    let res = client
        .get(url)
        .header("Accept", "application/json")
        .header("X-Plex-Token", user_token)
        .send()
        .await?
        .error_for_status()?
        .json::<PlexPlaylistList>()
        .await
        .wrap_err("Failed to deserialize Plex playlists response")?;

    Ok(res.media_container.metadata)
}

/* ---------- Create playlist ---------- */

#[derive(Debug, Clone, Deserialize)]
pub struct PlexCreatePlaylistResponse {
    #[serde(rename = "MediaContainer")]
    pub media_container: PlexMediaContainer<PlexPlaylist>,
}

pub async fn create_music_playlist(
    client: &Client,
    base_url: &Url,
    user_token: &str,
    title: &str,
) -> Result<PlexPlaylist> {
    let mut url = base_url.join("playlists")?;
    url.query_pairs_mut()
        .append_pair("title", title)
        .append_pair("type", "audio")
        .append_pair("smart", "0");

    let res = client
        .post(url)
        .header("Accept", "application/json")
        .header("X-Plex-Token", user_token)
        .send()
        .await?
        .error_for_status()?
        .json::<PlexCreatePlaylistResponse>()
        .await
        .wrap_err("Failed to deserialize create playlist response")?;

    res.media_container
        .metadata
        .into_iter()
        .next()
        .ok_or_eyre("Create playlist response had no Metadata")
}

/* ---------- Playlist items (tracks) ---------- */

#[derive(Debug, Clone, Deserialize)]
pub struct PlexTrack {
    #[serde(rename = "ratingKey")]
    pub rating_key: String,

    #[serde(rename = "playlistItemID")]
    pub playlist_item_id: Option<u64>,

    pub title: String,

    #[serde(default)]
    pub duration: Option<u64>,

    #[serde(default)]
    pub key: Option<String>,

    #[serde(rename = "grandparentTitle", default)]
    pub artist: Option<String>,

    #[serde(rename = "parentTitle", default)]
    pub album: Option<String>,

    #[serde(rename = "originalTitle", default)]
    pub original_title: Option<String>,

    #[serde(rename = "index", default)]
    pub track_number: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlexPlaylistItemsResponse {
    #[serde(rename = "MediaContainer")]
    pub media_container: PlexMediaContainer<PlexTrack>,
}

pub async fn get_playlist_tracks(
    client: &Client,
    base_url: &Url,
    user_token: &str,
    playlist_id: &str,
) -> Result<Vec<PlexTrack>> {
    let url = base_url.join(&format!("playlists/{}/items?type=10", playlist_id))?;

    let res = client
        .get(url)
        .header("Accept", "application/json")
        .header("X-Plex-Token", user_token)
        .send()
        .await?
        .error_for_status()?
        .json::<PlexPlaylistItemsResponse>()
        .await
        .wrap_err("Failed to deserialize playlist items response")?;

    Ok(res.media_container.metadata)
}

/* ---------- Add and remove ---------- */

pub async fn add_track_to_playlist(
    client: &Client,
    base_url: &Url,
    user_token: &str,
    playlist_id: &str,
    machine_identifier: &str,
    track_rating_key: &str,
) -> Result<()> {
    let track_uri = format!(
        "server://{}/com.plexapp.plugins.library/library/metadata/{}",
        machine_identifier, track_rating_key
    );

    let mut url = base_url.join(&format!("playlists/{}/items", playlist_id))?;
    url.query_pairs_mut().append_pair("uri", &track_uri);

    client
        .put(url)
        .header("X-Plex-Token", user_token)
        .send()
        .await?
        .error_for_status()
        .wrap_err("Failed to add track to playlist")?;

    Ok(())
}

pub async fn remove_track_from_playlist(
    client: &Client,
    base_url: &Url,
    user_token: &str,
    playlist_id: &str,
    playlist_item_id: u64,
) -> Result<()> {
    let url = base_url.join(&format!(
        "playlists/{}/items/{}",
        playlist_id, playlist_item_id
    ))?;

    client
        .delete(url)
        .header("X-Plex-Token", user_token)
        .send()
        .await?
        .error_for_status()
        .wrap_err("Failed to remove track from playlist")?;

    Ok(())
}

pub async fn clear_playlist(
    client: &Client,
    base_url: &Url,
    user_token: &str,
    playlist_id: &str,
) -> Result<()> {
    let url = base_url.join(&format!("playlists/{}/items", playlist_id))?;

    client
        .delete(url)
        .header("X-Plex-Token", user_token)
        .send()
        .await?
        .error_for_status()
        .wrap_err("Failed to clear playlist")?;

    Ok(())
}

/* ---------- Optional helpers ---------- */

pub fn is_music_playlist(p: &PlexPlaylist) -> bool {
    p.playlist_type == "audio"
}
