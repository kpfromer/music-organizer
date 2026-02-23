use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use color_eyre::eyre::{OptionExt, Result, WrapErr};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder, Set,
};
use tokio::sync::Notify;

use crate::config::Config;
use crate::database::Database;
use crate::entities;
use crate::entities::wishlist_item::WishlistStatus;
use crate::import_track::import_track;
use crate::services::playlist::PlaylistService;
use crate::services::spotify::download_best_match_for_spotify_track::download_best_match_for_spotify_track;
use crate::soulseek::SoulSeekClientContext;

/// Spawn the wishlist background task. Returns a Notify handle that can be used
/// to wake the task immediately (e.g. when a new item is added).
pub fn spawn_wishlist_background_task(
    db: Arc<Database>,
    soulseek_context: Arc<SoulSeekClientContext>,
    api_key: String,
    config: Config,
) -> Arc<Notify> {
    let notify = Arc::new(Notify::new());
    let notify_clone = notify.clone();

    tokio::spawn(async move {
        tracing::info!("Wishlist background task started");
        loop {
            // Wait for notification or timeout (5 minutes)
            tokio::select! {
                _ = notify_clone.notified() => {
                    tracing::debug!("Wishlist task woken by notification");
                }
                _ = tokio::time::sleep(Duration::from_secs(300)) => {
                    tracing::debug!("Wishlist task woken by timer");
                }
            }

            if let Err(e) = process_pending_items(&db, &soulseek_context, &api_key, &config).await {
                tracing::error!("Wishlist background task error: {}", e);
            }
        }
    });

    notify
}

async fn process_pending_items(
    db: &Arc<Database>,
    soulseek_context: &SoulSeekClientContext,
    api_key: &str,
    config: &Config,
) -> Result<()> {
    let now = Utc::now().timestamp();

    // Find items that are pending or failed with next_retry_at <= now
    let condition = Condition::any()
        .add(entities::wishlist_item::Column::Status.eq(WishlistStatus::Pending))
        .add(
            Condition::all()
                .add(entities::wishlist_item::Column::Status.eq(WishlistStatus::Failed))
                .add(
                    Condition::any()
                        .add(entities::wishlist_item::Column::NextRetryAt.is_null())
                        .add(entities::wishlist_item::Column::NextRetryAt.lte(now)),
                ),
        );

    let items = entities::wishlist_item::Entity::find()
        .filter(condition)
        .order_by_asc(entities::wishlist_item::Column::CreatedAt)
        .all(&db.conn)
        .await
        .wrap_err("Failed to fetch pending wishlist items")?;

    for item in items {
        if let Err(e) = process_single_item(db, soulseek_context, api_key, config, item).await {
            tracing::error!("Failed to process wishlist item: {}", e);
        }
    }

    Ok(())
}

async fn process_single_item(
    db: &Arc<Database>,
    soulseek_context: &SoulSeekClientContext,
    api_key: &str,
    config: &Config,
    item: entities::wishlist_item::Model,
) -> Result<()> {
    let spotify_track = entities::spotify_track::Entity::find()
        .filter(entities::spotify_track::Column::SpotifyTrackId.eq(&item.spotify_track_id))
        .one(&db.conn)
        .await
        .wrap_err("Failed to fetch spotify track")?
        .ok_or_eyre("Spotify track not found for wishlist item")?;

    // Skip if already matched to a local track
    if spotify_track.local_track_id.is_some() {
        set_status(db, &item, WishlistStatus::Completed, None).await?;
        return Ok(());
    }

    // Set status to searching
    set_status(db, &item, WishlistStatus::Searching, None).await?;

    // Search and download
    let download_result =
        download_best_match_for_spotify_track(soulseek_context, spotify_track.clone()).await;

    match download_result {
        Ok(Some((_temp_dir, temp_file))) => {
            // Set status to downloading -> importing
            set_status(db, &item, WishlistStatus::Importing, None).await?;

            // Import the track
            match import_track(&temp_file, api_key, config, db).await {
                Ok(local_track) => {
                    // Link spotify track to local track
                    let mut spotify_active: entities::spotify_track::ActiveModel =
                        spotify_track.into();
                    spotify_active.local_track_id = Set(Some(local_track.id));
                    spotify_active
                        .update(&db.conn)
                        .await
                        .wrap_err("Failed to link spotify track to local track")?;

                    // Add track to linked local playlists (Phase 3)
                    if let Err(e) =
                        add_to_linked_playlists(db, &item.spotify_track_id, local_track.id).await
                    {
                        tracing::error!("Failed to add track to linked playlists: {}", e);
                    }

                    set_status(db, &item, WishlistStatus::Completed, None).await?;
                    tracing::info!(
                        "Wishlist item completed: {} -> local track {}",
                        item.spotify_track_id,
                        local_track.id
                    );
                }
                Err(e) => {
                    let reason = format!("Import failed: {}", e);
                    mark_failed(db, &item, &reason).await?;
                }
            }
        }
        Ok(None) => {
            mark_failed(db, &item, "No SoulSeek results found").await?;
        }
        Err(e) => {
            let reason = format!("Download failed: {}", e);
            mark_failed(db, &item, &reason).await?;
        }
    }

    Ok(())
}

async fn set_status(
    db: &Arc<Database>,
    item: &entities::wishlist_item::Model,
    status: WishlistStatus,
    error_reason: Option<String>,
) -> Result<()> {
    let mut active: entities::wishlist_item::ActiveModel = item.clone().into();
    active.status = Set(status);
    active.error_reason = Set(error_reason);
    active.last_attempt_at = Set(Some(Utc::now().timestamp()));
    active
        .update(&db.conn)
        .await
        .wrap_err("Failed to update wishlist item status")?;
    Ok(())
}

async fn mark_failed(
    db: &Arc<Database>,
    item: &entities::wishlist_item::Model,
    reason: &str,
) -> Result<()> {
    let now = Utc::now().timestamp();
    let retry_at = now + 300; // retry in 5 minutes

    let mut active: entities::wishlist_item::ActiveModel = item.clone().into();
    active.status = Set(WishlistStatus::Failed);
    active.error_reason = Set(Some(reason.to_string()));
    active.attempts_count = Set(item.attempts_count + 1);
    active.last_attempt_at = Set(Some(now));
    active.next_retry_at = Set(Some(retry_at));
    active
        .update(&db.conn)
        .await
        .wrap_err("Failed to mark wishlist item as failed")?;

    tracing::warn!(
        "Wishlist item failed (attempt {}): {} - {}",
        item.attempts_count + 1,
        item.spotify_track_id,
        reason
    );

    Ok(())
}

/// Phase 3: After a wishlist track is imported, add it to any local playlists
/// that are linked to spotify playlists containing this track.
pub async fn add_to_linked_playlists(
    db: &Arc<Database>,
    spotify_track_id: &str,
    local_track_id: i64,
) -> Result<()> {
    // Find which spotify playlists contain this track
    let playlist_links = entities::spotify_track_playlist::Entity::find()
        .filter(entities::spotify_track_playlist::Column::SpotifyTrackId.eq(spotify_track_id))
        .all(&db.conn)
        .await
        .wrap_err("Failed to find spotify playlist links")?;

    let playlist_service = PlaylistService::new(db.clone());

    for link in playlist_links {
        // Find local playlists linked to this spotify playlist
        let local_playlists = entities::playlist::Entity::find()
            .filter(entities::playlist::Column::SpotifyPlaylistId.eq(link.spotify_playlist_id))
            .all(&db.conn)
            .await
            .wrap_err("Failed to find linked local playlists")?;

        for local_playlist in local_playlists {
            if let Err(e) = playlist_service
                .add_track(local_playlist.id, local_track_id)
                .await
            {
                tracing::error!(
                    "Failed to add track {} to playlist {}: {}",
                    local_track_id,
                    local_playlist.id,
                    e
                );
            }
        }
    }

    Ok(())
}
