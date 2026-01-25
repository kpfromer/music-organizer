use crate::database::Database;
use crate::entities;
use async_graphql::SimpleObject;
use color_eyre::eyre::OptionExt;
use color_eyre::eyre::Result;
use color_eyre::eyre::WrapErr;
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;
use tracing;

use super::add_tracks_to_playlist::add_tracks_to_local_playlist;

#[derive(Debug, Clone, SimpleObject)]
pub struct SyncSpotifyPlaylistToLocalLibraryResult {
    pub matched_track_count: u32,
    pub missing_track_count: u32,
}

/// Finds a spotify playlist by id and adds all tracks to a local playlist.
/// This **does not** download missing tracks or match them to local tracks.
pub async fn sync_spotify_playlist_to_local_library(
    db: &Database,
    spotify_playlist: entities::spotify_playlist::Model,
    local_playlist: entities::playlist::Model,
) -> Result<SyncSpotifyPlaylistToLocalLibraryResult> {
    tracing::info!(
        "Starting sync of spotify playlist to local library: {:?}",
        &spotify_playlist
    );

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

    let local_tracks_for_local_playlist = spotify_playlist_with_tracks
        .spotify_tracks
        .iter()
        .filter_map(|spotify_track| spotify_track.local_track_id)
        .collect::<Vec<_>>();

    let matched_track_count = local_tracks_for_local_playlist.len();
    let total_track_count = spotify_playlist_with_tracks.spotify_tracks.len();
    let missing_track_count = total_track_count - matched_track_count;

    // Add all successfully processed tracks to the local playlist
    add_tracks_to_local_playlist(db, &local_playlist, local_tracks_for_local_playlist).await?;

    // Mark sync as completed
    tracing::info!(
        "Completed sync of spotify playlist to local library: {:?}",
        &spotify_playlist
    );

    Ok(SyncSpotifyPlaylistToLocalLibraryResult {
        matched_track_count: matched_track_count as u32,
        missing_track_count: missing_track_count as u32,
    })
}
