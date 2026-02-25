use std::sync::Arc;

use color_eyre::eyre::{OptionExt, Result, WrapErr};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use tracing::instrument;

use crate::database::Database;
use crate::entities;
use crate::services::playlist::PlaylistService;
use crate::services::spotify::matching_local_tracks::match_existing_spotify_tracks_with_local_task;

#[derive(Debug)]
pub struct SyncSpotifyPlaylistToLocalResult {
    pub total_tracks: i64,
    pub matched_tracks: i64,
    pub unmatched_tracks: i64,
    pub new_matches_found: i64,
}

/// Sync a Spotify playlist to a local playlist by matching tracks.
/// Does NOT download anything — only matches against existing local tracks.
#[instrument(skip(db))]
pub async fn sync_spotify_playlist_to_local(
    db: Arc<Database>,
    spotify_playlist_id: i64,
    local_playlist_name: String,
) -> Result<SyncSpotifyPlaylistToLocalResult> {
    // 1. Verify the spotify playlist exists
    let spotify_playlist = entities::spotify_playlist::Entity::find_by_id(spotify_playlist_id)
        .one(&db.conn)
        .await
        .wrap_err("Failed to fetch spotify playlist")?
        .ok_or_eyre("Spotify playlist not found")?;

    // 2. Fetch all spotify tracks in this playlist
    let spotify_track_links = entities::spotify_track_playlist::Entity::find()
        .filter(entities::spotify_track_playlist::Column::SpotifyPlaylistId.eq(spotify_playlist.id))
        .all(&db.conn)
        .await
        .wrap_err("Failed to fetch spotify track playlist links")?;

    let mut spotify_tracks = Vec::new();
    for link in &spotify_track_links {
        let track = entities::spotify_track::Entity::find()
            .filter(entities::spotify_track::Column::SpotifyTrackId.eq(&link.spotify_track_id))
            .one(&db.conn)
            .await
            .wrap_err("Failed to fetch spotify track")?
            .ok_or_eyre("Spotify track not found")?;
        spotify_tracks.push(track);
    }

    let total_tracks = spotify_tracks.len() as i64;

    // 3. Count how many were already matched before this run
    let previously_matched = spotify_tracks
        .iter()
        .filter(|t| t.local_track_id.is_some())
        .count() as i64;

    // 4. Run the matcher on unmatched tracks
    let unmatched_tracks: Vec<_> = spotify_tracks
        .iter()
        .filter(|t| t.local_track_id.is_none())
        .cloned()
        .collect();

    if !unmatched_tracks.is_empty() {
        // Run matching synchronously (not as background task) since we want immediate results
        let _task =
            match_existing_spotify_tracks_with_local_task(db.clone(), unmatched_tracks).await?;

        // Wait briefly for the background task to complete
        // The task runs in a tokio::spawn, so we need to give it time
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    // 5. Re-fetch spotify tracks to get updated local_track_id values
    let mut spotify_tracks_updated = Vec::new();
    for link in &spotify_track_links {
        let track = entities::spotify_track::Entity::find()
            .filter(entities::spotify_track::Column::SpotifyTrackId.eq(&link.spotify_track_id))
            .one(&db.conn)
            .await
            .wrap_err("Failed to re-fetch spotify track")?
            .ok_or_eyre("Spotify track not found")?;
        spotify_tracks_updated.push(track);
    }

    let now_matched = spotify_tracks_updated
        .iter()
        .filter(|t| t.local_track_id.is_some())
        .count() as i64;
    let new_matches_found = now_matched - previously_matched;

    // 6. Find or create the local playlist, link it to the spotify playlist
    let playlist_service = PlaylistService::new(db.clone());
    let local_playlist = find_or_create_linked_playlist(
        &db,
        &playlist_service,
        &local_playlist_name,
        spotify_playlist.id,
    )
    .await?;

    // 7. Add all matched tracks to the local playlist
    for spotify_track in &spotify_tracks_updated {
        if let Some(local_track_id) = spotify_track.local_track_id {
            playlist_service
                .add_track(local_playlist.id, local_track_id)
                .await?;
        }
    }

    let unmatched = total_tracks - now_matched;

    Ok(SyncSpotifyPlaylistToLocalResult {
        total_tracks,
        matched_tracks: now_matched,
        unmatched_tracks: unmatched,
        new_matches_found,
    })
}

/// Find an existing local playlist linked to the spotify playlist, or create a new one.
async fn find_or_create_linked_playlist(
    db: &Database,
    playlist_service: &PlaylistService,
    name: &str,
    spotify_playlist_id: i64,
) -> Result<entities::playlist::Model> {
    // Check if there's already a playlist linked to this spotify playlist
    let existing = entities::playlist::Entity::find()
        .filter(entities::playlist::Column::SpotifyPlaylistId.eq(spotify_playlist_id))
        .one(&db.conn)
        .await
        .wrap_err("Failed to search for linked playlist")?;

    if let Some(playlist) = existing {
        // Update name if different
        if playlist.name != name {
            let mut active: entities::playlist::ActiveModel = playlist.into();
            active.name = Set(name.to_string());
            let updated = active
                .update(&db.conn)
                .await
                .wrap_err("Failed to update playlist name")?;
            return Ok(updated);
        }
        return Ok(playlist);
    }

    // Create new playlist
    let playlist = playlist_service.create(name.to_string(), None).await?;

    // Set the spotify_playlist_id FK
    let mut active: entities::playlist::ActiveModel = playlist.into();
    active.spotify_playlist_id = Set(Some(spotify_playlist_id));
    let updated = active
        .update(&db.conn)
        .await
        .wrap_err("Failed to link playlist to spotify playlist")?;

    Ok(updated)
}
