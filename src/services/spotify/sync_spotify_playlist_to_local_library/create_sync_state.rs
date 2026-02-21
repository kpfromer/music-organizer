use crate::database::Database;
use crate::entities;
use color_eyre::eyre::Result;
use color_eyre::eyre::WrapErr;
use sea_orm::{EntityTrait, Set};

/// Creates a new sync state record to track the progress of syncing a Spotify playlist
/// to a local playlist.
///
/// The sync state tracks:
/// - Which Spotify playlist is being synced
/// - Which local playlist it's being synced to
/// - Progress counters (tracks downloaded, tracks failed)
/// - Sync status (pending, in_progress, completed, error)
pub async fn create_sync_state(
    db: &Database,
    spotify_playlist: &crate::entities::spotify_playlist::Model,
    local_playlist: &crate::entities::playlist::Model,
) -> Result<entities::spotify_playlist_sync_state::Model> {
    let sync_state = entities::spotify_playlist_sync_state::ActiveModel {
        spotify_playlist_id: Set(spotify_playlist.id),
        local_playlist_id: Set(Some(local_playlist.id)),
        ..Default::default()
    };

    entities::spotify_playlist_sync_state::Entity::insert(sync_state)
        .exec_with_returning(&db.conn)
        .await
        .wrap_err("Failed to create sync state")
}
