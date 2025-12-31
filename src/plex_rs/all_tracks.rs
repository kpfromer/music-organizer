use color_eyre::eyre::{OptionExt, Result, WrapErr};
use reqwest::Client;
use serde::Deserialize;
use url::Url;

/* ---------- Shared container ---------- */

/// A minimal Plex JSON envelope for list style endpoints that return `MediaContainer.Metadata`.
///
/// Notes
/// - Plex responses are wrapped in a top level `MediaContainer`.
/// - Many fields are optional or omitted depending on endpoint and server version.
/// - `metadata` defaults to an empty vec when missing.
#[derive(Debug, Clone, Deserialize)]
pub struct PlexResponse<T> {
    #[serde(rename = "MediaContainer")]
    pub media_container: PlexMediaContainer<T>,
}

/// The inner Plex MediaContainer payload.
///
/// Notes
/// - `size`, `totalSize`, and `offset` commonly appear on list endpoints.
/// - For paged requests, you will typically use `offset` and `total_size` for looping.
#[derive(Debug, Clone, Deserialize)]
pub struct PlexMediaContainer<T> {
    #[serde(default)]
    pub size: Option<u32>,

    #[serde(rename = "totalSize", default)]
    pub total_size: Option<u32>,

    #[serde(default)]
    pub offset: Option<u32>,

    #[serde(rename = "Metadata", default = "Vec::new")]
    pub metadata: Vec<T>,
}

/* ---------- Library sections ---------- */

/// Response type for `/library/sections`.
#[derive(Debug, Deserialize)]
pub struct PlexLibrarySectionsResponse {
    #[serde(rename = "MediaContainer")]
    pub media_container: PlexLibrarySectionsContainer,
}

/// `MediaContainer` for `/library/sections` which returns a `Directory` list.
#[derive(Debug, Deserialize)]
pub struct PlexLibrarySectionsContainer {
    #[serde(rename = "Directory", default)]
    pub directories: Vec<PlexLibrarySection>,
}

/// A Plex library section.
///
/// Notes
/// - `key` is the library section id.
/// - `section_type` is commonly `movie`, `show`, or for music libraries `artist`.
#[derive(Debug, Deserialize)]
pub struct PlexLibrarySection {
    pub key: String,
    pub title: String,
    #[serde(rename = "type")]
    pub section_type: String,
}

/// Fetch all Plex library sections.
///
/// Endpoint
/// - `GET /library/sections`
///
/// Returns
/// - A list of sections. For music, pick the one where `section_type == "artist"`.
pub async fn get_library_sections(
    client: &Client,
    base_url: &Url,
    user_token: &str,
) -> Result<Vec<PlexLibrarySection>> {
    let url = base_url.join("library/sections")?;

    let res = client
        .get(url)
        .header("Accept", "application/json")
        .header("X-Plex-Token", user_token)
        .send()
        .await?
        .error_for_status()?
        .json::<PlexLibrarySectionsResponse>()
        .await
        .wrap_err("Failed to deserialize library sections")?;

    Ok(res.media_container.directories)
}

/* ---------- Tracks ---------- */

/// A music track item returned from `/library/sections/{id}/all?type=10`.
#[derive(Debug, Deserialize)]
pub struct PlexLibraryTrack {
    #[serde(rename = "ratingKey")]
    pub rating_key: String,

    pub title: String,

    #[serde(rename = "grandparentTitle", default)]
    pub artist: Option<String>,

    #[serde(rename = "parentTitle", default)]
    pub album: Option<String>,

    #[serde(rename = "index", default)]
    pub track_number: Option<u32>,

    #[serde(default)]
    pub duration: Option<u64>,

    #[serde(rename = "Media", default)]
    pub media: Vec<PlexMedia>,
}

/// Media element containing Part information with file paths
#[derive(Debug, Deserialize)]
pub struct PlexMedia {
    #[serde(rename = "Part", default)]
    pub parts: Vec<PlexPart>,
}

/// Part element containing the actual file path
#[derive(Debug, Deserialize)]
pub struct PlexPart {
    /// Absolute path to the media file on disk
    pub file: String,
}

impl PlexLibraryTrack {
    /// Returns the file path from the first media part.
    ///
    /// # Errors
    /// - Returns an error if there are 0 or more than 1 media items
    /// - Returns an error if the media item has no parts or the part has no file
    pub fn file_path(&self) -> Result<&str> {
        match self.media.len() {
            0 => color_eyre::eyre::bail!("Track '{}' has no media items", self.title),
            1 => {
                let media = &self.media[0];
                let part = media
                    .parts
                    .first()
                    .ok_or_eyre(format!("Track '{}' has no parts in media", self.title))?;
                Ok(part.file.as_str())
            }
            n => color_eyre::eyre::bail!(
                "Track '{}' has {} media items, expected exactly 1",
                self.title,
                n
            ),
        }
    }
}

/// Response type for `/library/sections/{id}/all?type=10`.
#[derive(Debug, Deserialize)]
pub struct PlexLibraryTracksResponse {
    #[serde(rename = "MediaContainer")]
    pub media_container: PlexMediaContainer<PlexLibraryTrack>,
}

/// Fetch one page of tracks from a music section.
///
/// Pagination
/// - Pass `start` as the offset (`X-Plex-Container-Start`).
/// - Pass `size` as the page size (`X-Plex-Container-Size`).
///
/// Endpoint
/// - `GET /library/sections/{id}/all?type=10`
///
/// Returns
/// - The full decoded `PlexMediaContainer` so the caller can read `total_size` and `offset`.
pub async fn get_tracks_page(
    client: &Client,
    base_url: &Url,
    user_token: &str,
    music_section_id: &str,
    start: u32,
    size: u32,
) -> Result<PlexMediaContainer<PlexLibraryTrack>> {
    let url = base_url.join(&format!(
        "library/sections/{}/all?type=10",
        music_section_id
    ))?;

    let res = client
        .get(url)
        .header("Accept", "application/json")
        .header("X-Plex-Token", user_token)
        .header("X-Plex-Container-Start", start.to_string())
        .header("X-Plex-Container-Size", size.to_string())
        .send()
        .await?
        .error_for_status()?
        .json::<PlexLibraryTracksResponse>()
        .await
        .wrap_err("Failed to deserialize library tracks page")?;

    Ok(res.media_container)
}

/// Fetch all tracks from a music section, handling Plex pagination.
///
/// Pagination strategy
/// - Requests are made in pages of `page_size`.
/// - Stops when we have retrieved `totalSize` tracks, or when an empty page is returned.
/// - If `totalSize` is missing, falls back to the empty page stop condition.
///
/// Endpoint
/// - `GET /library/sections/{id}/all?type=10`
///
/// Parameters
/// - `music_section_id`: The library section id (from `/library/sections`).
/// - `page_size`: How many tracks per request. Typical values: 200 to 2000.
///
/// Returns
/// - All decoded tracks.
pub async fn get_all_tracks_paginated(
    client: &Client,
    base_url: &Url,
    user_token: &str,
    music_section_id: &str,
    page_size: u32,
) -> Result<Vec<PlexLibraryTrack>> {
    let mut start: u32 = 0;
    let mut out: Vec<PlexLibraryTrack> = Vec::new();

    loop {
        let container = get_tracks_page(
            client,
            base_url,
            user_token,
            music_section_id,
            start,
            page_size,
        )
        .await?;

        // Defensive stop if server returns an empty page.
        if container.metadata.is_empty() {
            break;
        }

        // Append results and advance offset.
        out.extend(container.metadata);
        start = out.len() as u32;

        // If Plex tells us the total size, stop exactly at the end.
        if let Some(total) = container.total_size
            && start >= total
        {
            break;
        }
    }

    Ok(out)
}

/// Convenience helper: find the first music library section id.
///
/// Notes
/// - Plex music libraries typically have `section_type == "artist"`.
pub fn find_music_section_id(sections: &[PlexLibrarySection]) -> Option<&str> {
    sections
        .iter()
        .find(|s| s.section_type == "artist")
        .map(|s| s.key.as_str())
}
