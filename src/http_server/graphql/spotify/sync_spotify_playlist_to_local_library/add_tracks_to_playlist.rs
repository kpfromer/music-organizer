use crate::database::Database;
use crate::entities;
use color_eyre::eyre::Result;
use color_eyre::eyre::WrapErr;
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;
use sea_orm::{EntityTrait, Set};

/// Adds tracks to a local playlist, skipping tracks that are already in the playlist.
///
/// This function:
/// 1. Fetches all local tracks by their IDs
/// 2. Checks which tracks are already in the playlist (to avoid duplicates)
/// 3. Inserts only the new tracks
///
/// Note: Currently uses N+1 queries for checking existing playlist tracks.
/// This could be optimized with a single query using a LEFT JOIN or NOT EXISTS.
pub async fn add_tracks_to_local_playlist(
    db: &Database,
    local_playlist: &entities::playlist::Model,
    local_track_ids: Vec<i64>,
) -> Result<()> {
    if local_track_ids.is_empty() {
        return Ok(());
    }

    // Fetch all local tracks that we want to add
    let local_playlist_tracks = entities::track::Entity::find()
        .filter(entities::track::Column::Id.is_in(local_track_ids))
        .all(&db.conn)
        .await
        .wrap_err("Failed to fetch local tracks")?;

    // Check which tracks are already in the playlist and only add new ones
    // TODO: Optimize this to use a single query instead of N+1 queries
    // Could use: SELECT track_id FROM playlist_track WHERE playlist_id = ? AND track_id IN (?)
    // Then filter out those IDs before inserting
    for local_track in local_playlist_tracks {
        let has_existing_playlist_track = entities::playlist_track::Entity::find()
            .filter(entities::playlist_track::Column::PlaylistId.eq(local_playlist.id))
            .filter(entities::playlist_track::Column::TrackId.eq(local_track.id))
            .one(&db.conn)
            .await
            .wrap_err("Failed to check if track is already in playlist")?;

        // Skip tracks that are already in the playlist
        if has_existing_playlist_track.is_some() {
            continue;
        }

        // Add the track to the playlist
        let local_playlist_track = entities::playlist_track::ActiveModel {
            playlist_id: Set(local_playlist.id),
            track_id: Set(local_track.id),
            ..Default::default()
        };
        entities::playlist_track::Entity::insert(local_playlist_track)
            .exec(&db.conn)
            .await?;
    }

    Ok(())
}
