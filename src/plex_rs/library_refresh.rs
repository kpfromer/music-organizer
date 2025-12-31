use color_eyre::eyre::{OptionExt, Result, WrapErr};
use reqwest::Client;
use serde::Deserialize;
use url::Url;

/// Trigger a refresh/rescan of a Plex library section
///
/// Endpoint: `GET /library/sections/{section_id}/refresh`
///
/// This will start a background scan of the library section.
pub async fn refresh_library_section(
    client: &Client,
    base_url: &Url,
    user_token: &str,
    section_id: &str,
) -> Result<()> {
    let mut url = base_url.join(&format!("library/sections/{}/refresh", section_id))?;
    url.query_pairs_mut()
        .append_pair("X-Plex-Token", user_token);

    client
        .get(url)
        .header("Accept", "application/json")
        .header("X-Plex-Token", user_token)
        .send()
        .await?
        .error_for_status()
        .wrap_err("Failed to refresh library section")?;

    Ok(())
}

/// Response type for `/activities` endpoint
#[derive(Debug, Clone, Deserialize)]
pub struct PlexActivitiesResponse {
    #[serde(rename = "MediaContainer")]
    pub media_container: PlexActivitiesContainer,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlexActivitiesContainer {
    #[serde(rename = "Activity", default)]
    pub activities: Vec<PlexActivity>,
}

/// Represents a Plex background activity (like library scanning)
#[derive(Debug, Clone, Deserialize)]
pub struct PlexActivity {
    #[serde(rename = "uuid", default)]
    pub uuid: Option<String>,

    #[serde(rename = "type")]
    pub activity_type: String,

    #[serde(rename = "title")]
    pub title: String,

    #[serde(rename = "subtitle", default)]
    pub subtitle: Option<String>,

    #[serde(rename = "progress", default)]
    pub progress: Option<f64>,

    #[serde(rename = "Context", default)]
    pub context: Option<PlexActivityContext>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlexActivityContext {
    #[serde(rename = "librarySectionID", default)]
    pub library_section_id: Option<String>,

    #[serde(rename = "librarySectionTitle", default)]
    pub library_section_title: Option<String>,
}

/// Get current activities/status from Plex server
///
/// Endpoint: `GET /activities`
///
/// Returns all current background activities, including library scans.
pub async fn get_activities(
    client: &Client,
    base_url: &Url,
    user_token: &str,
) -> Result<Vec<PlexActivity>> {
    let mut url = base_url.join("activities")?;
    url.query_pairs_mut()
        .append_pair("X-Plex-Token", user_token);

    let res = client
        .get(url)
        .header("Accept", "application/json")
        .header("X-Plex-Token", user_token)
        .send()
        .await?
        .error_for_status()?
        .json::<PlexActivitiesResponse>()
        .await
        .wrap_err("Failed to deserialize activities response")?;

    Ok(res.media_container.activities)
}

/// Check if a library section is currently being scanned
///
/// Returns the activity if found, None otherwise.
pub async fn get_library_scan_status(
    client: &Client,
    base_url: &Url,
    user_token: &str,
    section_id: &str,
) -> Result<Option<PlexActivity>> {
    let activities = get_activities(client, base_url, user_token).await?;

    // Look for activities related to this section
    // Plex API uses "library.update.section" for library scans
    let scan_activity = activities.into_iter().find(|activity| {
        // Check if activity is related to library scanning/refreshing
        let is_scan_type = activity.activity_type == "library.update.section"
            || activity.activity_type == "library.refresh"
            || activity.activity_type == "library.refresh.section"
            || activity.title.to_lowercase().contains("scan")
            || activity.title.to_lowercase().contains("refresh");

        // Check if it's for our section
        let is_our_section = activity
            .context
            .as_ref()
            .and_then(|ctx| ctx.library_section_id.as_ref())
            .map(|id| id == section_id)
            .unwrap_or(false);

        is_scan_type && is_our_section
    });

    Ok(scan_activity)
}
