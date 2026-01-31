use std::sync::Arc;
use tracing;

use crate::database::Database;
use crate::entities;
use crate::http_server::graphql::spotify::sync_spotify_playlist_to_local_library::sync_task::SyncSpotifyPlaylistToLocalLibraryResult;
use color_eyre::eyre::OptionExt;
use color_eyre::eyre::Result;
use color_eyre::eyre::WrapErr;
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;
use sea_orm::{EntityTrait, Set};

use super::sync_task::sync_spotify_playlist_to_local_library;

/// Public entry point for syncing a Spotify playlist to the local music library.
///
/// This function:
/// 1. Validates that the Spotify playlist exists
/// 2. Finds or creates the local playlist
/// 3. Creates a sync state to track progress
/// 4. Spawns a background task to perform the actual sync
///
/// The sync runs asynchronously in the background, and the sync state can be
/// queried to check progress. The function returns immediately with the sync state.
///
/// # Arguments
/// * `db` - Database connection
/// * `soulseek_context` - SoulSeek client context for downloading tracks
/// * `api_key` - API key for track import
/// * `config` - Application configuration
/// * `spotify_account_id` - ID of the Spotify account that owns the playlist
/// * `spotify_playlist_id` - ID of the Spotify playlist to sync
/// * `local_playlist_name` - Name of the local playlist (created if it doesn't exist)
pub async fn sync_spotify_playlist_to_local_library_task(
    db: Arc<Database>,
    spotify_account_id: i64,
    spotify_playlist_id: i64,
    local_playlist_name: String,
) -> Result<SyncSpotifyPlaylistToLocalLibraryResult> {
    // Fetch the Spotify playlist
    let spotify_playlist = entities::spotify_playlist::Entity::find()
        .filter(entities::spotify_playlist::Column::AccountId.eq(spotify_account_id))
        .filter(entities::spotify_playlist::Column::Id.eq(spotify_playlist_id))
        .one(&db.conn)
        .await
        .wrap_err("Failed to fetch spotify playlist")?
        .ok_or_eyre("Spotify playlist not found")?;

    // Find or create the local playlist
    let local_playlist = entities::playlist::Entity::find()
        .filter(entities::playlist::Column::Name.eq(&local_playlist_name))
        .one(&db.conn)
        .await
        .wrap_err("Failed to fetch local playlist")?;

    let local_playlist = match local_playlist {
        Some(playlist) => playlist,
        None => {
            // Create the local playlist if it doesn't exist
            let playlist = entities::playlist::ActiveModel {
                name: Set(local_playlist_name),
                ..Default::default()
            };
            entities::playlist::Entity::insert(playlist)
                .exec_with_returning(&db.conn)
                .await
                .wrap_err("Failed to create local playlist")?
        }
    };

    match sync_spotify_playlist_to_local_library(&db, spotify_playlist, local_playlist).await {
        Ok(result) => {
            tracing::info!(result = ?result, "Successfully completed sync");
            Ok(result)
        }
        Err(e) => {
            tracing::error!("Failed to sync spotify playlist to local library: {:?}", e);
            Err(e)
        }
    }
}
