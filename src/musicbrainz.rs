// TODO: Remove this once we have a proper API
#![allow(dead_code)]

use anyhow::Result;
use musicbrainz_rs::Fetch;
use musicbrainz_rs::entity::recording::Recording;
use musicbrainz_rs::entity::release::Release;
use musicbrainz_rs::entity::release_group::ReleaseGroupPrimaryType;

pub async fn fetch_recording_with_details(recording_id: &str) -> Result<Recording> {
    let recording = Recording::fetch()
        .id(recording_id)
        .with_artists()
        .with_releases()
        .with_release_group_relations()
        .execute()
        .await?;

    Ok(recording)
}

pub async fn fetch_release_with_details(release_id: &str) -> Result<Release> {
    // For rate limiting
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let release = Release::fetch()
        .id(release_id)
        .with_release_groups()
        .with_artists()
        .execute()
        .await
        .map_err(|_e| anyhow::anyhow!("Failed to fetch release from MusicBrainz"))?;

    Ok(release)
}

pub struct TrackInfo {
    pub artist_name: String,
    pub track_title: String,
    pub release_title: String,
    pub album_title: String,
    pub album_type: String,
}

pub fn extract_track_info(recording: &Recording, release: &Release) -> Result<TrackInfo> {
    let artist = recording
        .artist_credit
        .as_ref()
        .ok_or(anyhow::anyhow!("No artist found"))?
        .first()
        .ok_or(anyhow::anyhow!("No artist found"))?;

    let track_title = &recording.title;

    let release_title = &release.title;

    let release_group = release
        .release_group
        .as_ref()
        .ok_or(anyhow::anyhow!("No release group found"))?;

    let release_group_title = &release_group.title;
    let release_group_type = release_group
        .primary_type
        .as_ref()
        .ok_or(anyhow::anyhow!("No release group type found"))?;

    let album_type = match release_group_type {
        ReleaseGroupPrimaryType::Album => "Album",
        ReleaseGroupPrimaryType::Single => "Single",
        ReleaseGroupPrimaryType::Ep => "EP",
        ReleaseGroupPrimaryType::Other => "Other",
        ReleaseGroupPrimaryType::UnrecognizedReleaseGroupPrimaryType => "Unknown",
        _ => "Unknown",
    };

    Ok(TrackInfo {
        artist_name: artist.name.clone(),
        track_title: track_title.clone(),
        release_title: release_title.clone(),
        album_title: release_group_title.clone(),
        album_type: album_type.to_string(),
    })
}
