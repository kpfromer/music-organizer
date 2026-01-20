use crate::database::Database;
use crate::entities;
use crate::http_server::graphql::spotify::download_best_match_for_spotify_track::download_best_match_for_spotify_track;
use crate::import_track::import_track;
use crate::soulseek::SoulSeekClientContext;
use color_eyre::eyre::Result;
use sea_orm::{EntityTrait, Set};
use tracing;

/// Result of processing a single Spotify track.
#[derive(Debug)]
pub struct ProcessTrackResult {
    /// ID of the local track if successfully processed, None if failed
    pub local_track_id: Option<i64>,
    /// Whether the track was successfully downloaded and imported
    pub success: bool,
}

/// Processes a single Spotify track by either:
/// 1. Using the existing local track if already matched
/// 2. Downloading and matching the track if not yet in local library
///
/// Returns the local track ID if successful, None if the track couldn't be processed.
///
/// # Note
/// The `spotify_track` parameter should be a `ModelEx` with the `local_track` relation loaded.
/// This is typically obtained from `Entity::load().with(...)`.
pub async fn process_spotify_track(
    db: &Database,
    soulseek_context: &SoulSeekClientContext,
    api_key: &str,
    config: &crate::config::Config,
    spotify_playlist_id: i64,
    spotify_track: entities::spotify_track::ModelEx,
) -> Result<ProcessTrackResult> {
    // Check if track is already matched to a local track
    // The local_track relation is loaded via Entity::load().with()
    let local_track = spotify_track.local_track.clone().into_option();

    if let Some(local_track) = local_track {
        tracing::info!("Track already exists in local library: {:?}", &local_track);
        return Ok(ProcessTrackResult {
            local_track_id: Some(local_track.id),
            success: true,
        });
    }

    // Track doesn't exist in local library - need to download and match it
    tracing::info!(
        "Downloading and matching spotify track: {:?}",
        &spotify_track
    );

    let output =
        download_best_match_for_spotify_track(soulseek_context, spotify_track.clone().into()).await;

    match output {
        Ok(Some((_temp_dir, temp_file))) => {
            tracing::info!("Found best match for spotify track: {:?}", &spotify_track);

            // Import the downloaded track into the local library
            let local_track = import_track(&temp_file, api_key, config, db).await?;

            // Link the Spotify track to the newly imported local track
            // Convert ModelEx to Model, then to ActiveModel
            let spotify_track_model: entities::spotify_track::Model = spotify_track.into();
            let mut spotify_track_active: entities::spotify_track::ActiveModel =
                spotify_track_model.into();
            spotify_track_active.local_track_id = Set(Some(local_track.id));
            entities::spotify_track::Entity::update(spotify_track_active)
                .exec(&db.conn)
                .await?;

            Ok(ProcessTrackResult {
                local_track_id: Some(local_track.id),
                success: true,
            })
        }
        Ok(None) => {
            tracing::info!(
                "No best match found for spotify track: {:?}",
                &spotify_track
            );
            // Record the failure for tracking purposes
            record_download_failure(
                db,
                spotify_playlist_id,
                &spotify_track,
                "No match found".to_string(),
            )
            .await?;

            Ok(ProcessTrackResult {
                local_track_id: None,
                success: false,
            })
        }
        Err(e) => {
            tracing::error!(
                "Failed to download best match for spotify track: {:?}: {}",
                &spotify_track,
                e
            );
            // Record the failure with the error message
            record_download_failure(db, spotify_playlist_id, &spotify_track, e.to_string()).await?;

            Ok(ProcessTrackResult {
                local_track_id: None,
                success: false,
            })
        }
    }
}

/// Records a download failure in the database for tracking and debugging purposes.
async fn record_download_failure(
    db: &Database,
    spotify_playlist_id: i64,
    spotify_track: &entities::spotify_track::ModelEx,
    reason: String,
) -> Result<()> {
    // Convert ModelEx to Model to access fields
    let spotify_track_model: entities::spotify_track::Model = spotify_track.clone().into();
    let spotify_track_download_failure = entities::spotify_track_download_failure::ActiveModel {
        spotify_playlist_id: Set(spotify_playlist_id),
        spotify_track_id: Set(spotify_track_model.spotify_track_id),
        track_name: Set(spotify_track_model.title),
        artist_name: Set(spotify_track_model.artists.0.join(", ")),
        album_name: Set(Some(spotify_track_model.album)),
        isrc: Set(spotify_track_model.isrc),
        reason: Set(reason),
        ..Default::default()
    };

    entities::spotify_track_download_failure::Entity::insert(spotify_track_download_failure)
        .exec(&db.conn)
        .await?;

    Ok(())
}
