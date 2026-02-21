use crate::config::Config;
use crate::database::Database;
use crate::entities;
use crate::soulseek::SoulSeekClientContext;
use color_eyre::eyre::OptionExt;
use color_eyre::eyre::Result;
use color_eyre::eyre::WrapErr;
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;
use sea_orm::{EntityTrait, Set};
use tracing;

use super::add_tracks_to_playlist::add_tracks_to_local_playlist;
use super::process_track::process_spotify_track;

/// Main sync function that orchestrates the process of syncing a Spotify playlist
/// to the local music library.
///
/// This function:
/// 1. Loads the Spotify playlist with all its tracks
/// 2. Processes each track (downloads/matches if needed)
/// 3. Updates sync state progress after each track
/// 4. Adds all successfully processed tracks to the local playlist
/// 5. Marks the sync as completed
///
/// The sync state is updated incrementally throughout the process so that
/// progress can be monitored even if the sync is interrupted.
pub async fn sync_spotify_playlist_to_local_library(
    db: &Database,
    soulseek_context: &SoulSeekClientContext,
    api_key: String,
    config: Config,
    sync_state: entities::spotify_playlist_sync_state::Model,
    spotify_playlist: entities::spotify_playlist::Model,
    local_playlist: entities::playlist::Model,
) -> Result<()> {
    tracing::info!(
        "Starting sync of spotify playlist to local library: {:?}",
        &spotify_playlist
    );

    let mut sync_state: entities::spotify_playlist_sync_state::ActiveModel = sync_state.into();

    // Load the Spotify playlist with all its tracks and their local track relationships
    let spotify_playlist_with_tracks = entities::spotify_playlist::Entity::load()
        .filter(entities::spotify_playlist::Column::Id.eq(spotify_playlist.id))
        .with((
            // https://www.sea-ql.org/SeaORM/docs/relation/entity-loader/
            entities::spotify_track::Entity,
            entities::track::Entity,
        ))
        .one(&db.conn)
        .await
        .wrap_err("Failed to fetch spotify tracks for spotify playlist")?
        .ok_or_eyre("Spotify playlist not found")?;

    tracing::info!(
        "Found {} tracks in spotify playlist: {:?}",
        spotify_playlist_with_tracks.spotify_tracks.len(),
        &spotify_playlist
    );

    // Process each track and collect successfully processed track IDs
    let mut local_tracks_for_local_playlist = Vec::new();
    let mut tracks_downloaded = 0;
    let mut tracks_failed = 0;

    for spotify_track in spotify_playlist_with_tracks.spotify_tracks {
        let result = process_spotify_track(
            db,
            soulseek_context,
            &api_key,
            &config,
            spotify_playlist.id,
            spotify_track,
        )
        .await?;

        if result.success {
            if let Some(local_track_id) = result.local_track_id {
                local_tracks_for_local_playlist.push(local_track_id);
            }
            tracks_downloaded += 1;
        } else {
            tracks_failed += 1;
        }

        // Update sync state progress after each track so progress is visible
        // even if the sync is interrupted
        sync_state.tracks_downloaded = Set(tracks_downloaded);
        sync_state.tracks_failed = Set(tracks_failed);
        entities::spotify_playlist_sync_state::Entity::update(sync_state.clone())
            .exec(&db.conn)
            .await?;
    }

    // Add all successfully processed tracks to the local playlist
    add_tracks_to_local_playlist(db, &local_playlist, local_tracks_for_local_playlist).await?;

    // Mark sync as completed
    tracing::info!(
        "Completed sync of spotify playlist to local library: {:?}",
        &spotify_playlist
    );
    sync_state.sync_status = Set("completed".to_string());
    entities::spotify_playlist_sync_state::Entity::update(sync_state)
        .exec(&db.conn)
        .await?;

    Ok(())
}
