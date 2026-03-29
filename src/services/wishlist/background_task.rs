use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use color_eyre::eyre::{OptionExt, Result, WrapErr};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder, Set,
};
use song_rs::WantedFileTypes;
use tempfile::TempDir;
use tokio::sync::Notify;
use tracing::instrument;

use crate::config::Config;
use crate::database::Database;
use crate::entities;
use crate::entities::wishlist_item::WishlistStatus;
use crate::import_track::import_track;
use crate::services::playlist::PlaylistService;
use song_rs::{Client as SongDownloader, SongQuery};

/// Spawn the wishlist background task. Returns a Notify handle that can be used
/// to wake the task immediately (e.g. when a new item is added).
pub fn spawn_wishlist_background_task(
    db: Arc<Database>,
    song_downloader: &SongDownloader,
    api_key: String,
    config: Config,
) -> Arc<Notify> {
    let notify = Arc::new(Notify::new());
    let notify_clone = notify.clone();
    let song_downloader_clone = song_downloader.clone();

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

            if let Err(e) =
                process_pending_items(&db, &song_downloader_clone, &api_key, &config).await
            {
                tracing::error!("Wishlist background task error: {}", e);
            }
        }
    });

    notify
}

async fn process_pending_items(
    db: &Arc<Database>,
    song_downloader: &SongDownloader,
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
        if let Err(e) = process_single_item(db, song_downloader, api_key, config, item).await {
            tracing::error!("Failed to process wishlist item: {}", e);
        }
    }

    Ok(())
}

async fn process_single_item(
    db: &Arc<Database>,
    song_downloader: &SongDownloader,
    api_key: &str,
    config: &Config,
    item: entities::wishlist_item::Model,
) -> Result<()> {
    let spotify_track = entities::spotify_track::Entity::find()
        .filter(entities::spotify_track::Column::SpotifyTrackId.eq(&item.spotify_track_id))
        .one(&db.conn)
        .await
        .wrap_err("Failed to fetch spotify track")?;

    let Some(spotify_track) = spotify_track else {
        mark_failed(db, &item, "Spotify track not found").await?;
        return Ok(());
    };

    // Skip if already matched to a local track
    if spotify_track.local_track_id.is_some() {
        set_status(db, &item, WishlistStatus::Completed, None).await?;
        return Ok(());
    }

    let Some(duration_ms) = spotify_track.duration else {
        mark_failed(db, &item, "Spotify track duration not found").await?;
        return Ok(());
    };

    let Some(artist) = spotify_track.artists.0.first().cloned() else {
        mark_failed(db, &item, "No artist found").await?;
        return Ok(());
    };

    set_status(db, &item, WishlistStatus::Downloading, None).await?;
    let (_temp_dir, file_path) = match download_track(
        song_downloader,
        &SongQuery {
            title: spotify_track.title.clone(),
            artist,
            album: Some(spotify_track.album.clone()),
            duration_secs: duration_ms as u32 / 1000,
        },
    )
    .await
    {
        Ok(Some((temp_dir, file_path))) => (temp_dir, file_path),
        Ok(None) => {
            mark_failed(db, &item, "No SoulSeek results found").await?;
            return Ok(());
        }
        Err(e) => {
            mark_failed(db, &item, &e.to_string()).await?;
            return Ok(());
        }
    };

    // Set status to downloading -> importing
    set_status(db, &item, WishlistStatus::Importing, None).await?;

    // Import the track
    match import_track(&file_path, api_key, config, db).await {
        Ok(local_track) => {
            // Link spotify track to local track
            let mut spotify_active: entities::spotify_track::ActiveModel = spotify_track.into();
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

    Ok(())
}

#[instrument(skip(song_downloader))]
async fn download_track(
    song_downloader: &SongDownloader,
    song_query: &SongQuery,
) -> Result<Option<(TempDir, PathBuf)>> {
    let search_results = song_downloader
        .search(song_query, Duration::from_secs(10), &WantedFileTypes::all())
        .await?;
    let search_results_length = search_results.len();
    tracing::debug!(
        ?song_query,
        ?search_results,
        "Search results for song_query"
    );

    for (index, to_be_downloaded) in search_results.into_iter().enumerate() {
        tracing::debug!(
            ?song_query,
            ?to_be_downloaded,
            index,
            "Downloading best match {} of {}",
            index + 1,
            search_results_length
        );

        let temp_dir = tempfile::tempdir()?;
        let download_directory = &temp_dir
            .path()
            .as_os_str()
            .to_str()
            .ok_or_eyre("Failed to get temp dir path")?
            .to_string();
        tracing::debug!(?download_directory, "Download directory");

        let (download, mut download_receiver) = song_downloader
            .download(
                &to_be_downloaded,
                download_directory,
                Some(Duration::from_secs(30)),
            )
            .await?;

        // Compute path before the async block so temp_dir stays in scope
        let file_path_candidate = temp_dir.path().join(download.filename.filename());

        // Limit each download attempt to 2 minutes; Soulseek peers sometimes accept
        // a download request but never actually deliver data, hanging indefinitely.
        let completed = tokio::time::timeout(Duration::from_secs(120), async {
            let mut completed = false;
            while let Some(status) = download_receiver.recv().await {
                match status {
                    song_rs::DownloadStatus::Queued => {
                        tracing::debug!(
                            ?song_query,
                            "Download queued. Waiting for download to start."
                        );
                    }
                    song_rs::DownloadStatus::InProgress { .. } => {
                        tracing::debug!(?song_query, "Download in progress.");
                        continue;
                    }
                    song_rs::DownloadStatus::Completed => {
                        completed = true;
                        break;
                    }
                    song_rs::DownloadStatus::Failed => {
                        tracing::error!(
                            ?song_query,
                            "Download failed. Trying next song match from soulseek."
                        );
                        break;
                    }
                    song_rs::DownloadStatus::TimedOut => {
                        tracing::error!(
                            ?song_query,
                            "Download timed out by song_rs. Trying next song match from soulseek."
                        );
                        break;
                    }
                    song_rs::DownloadStatus::Cancelled => {
                        tracing::error!(
                            ?song_query,
                            "Download was cancelled. Trying next song match from soulseek."
                        );
                        break;
                    }
                }
            }
            completed
        })
        .await;

        match completed {
            Err(_elapsed) => {
                tracing::error!(
                    ?song_query,
                    index,
                    "Download attempt timed out after 2 minutes. Trying next song match from soulseek."
                );
                continue;
            }
            Ok(false) => continue,
            Ok(true) => return Ok(Some((temp_dir, file_path_candidate))),
        }
    }
    tracing::error!(?song_query, "No SoulSeek results found");
    Ok(None)
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
