use color_eyre::eyre::{OptionExt, Result, WrapErr};
use reqwest::Client;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use url::Url;

use crate::database::Database;
use crate::entities;
use crate::plex_rs::all_tracks::{
    find_music_section_id, get_all_tracks_paginated, get_library_sections,
};
use crate::plex_rs::playlist::{
    add_track_to_playlist, create_music_playlist, get_machine_identifier, get_playlist_tracks,
    get_playlists, is_music_playlist, remove_track_from_playlist,
};

/// Represents a track that exists in the database playlist but not in the Plex library
#[derive(Debug, Clone)]
pub struct MissingTrack {
    pub track_id: i64,
    pub file_path: String,
    pub title: String,
}

/// Extracts and normalizes the last 3 path components (artist/album/track) for matching.
///
/// The path structure is: `.../ArtistName/AlbumName/TrackNumber - TrackName.ext`
/// This function extracts Artist/Album/Track and normalizes them by:
/// - Converting to lowercase
/// - Removing track number prefix (e.g., "01 ")
/// - Removing file extension
///
/// Returns None if the path doesn't have at least 3 components.
fn normalize_path_key(file_path: &str) -> Option<String> {
    let path = Path::new(file_path);
    let components: Vec<_> = path.iter().filter_map(|c| c.to_str()).collect();

    if components.len() < 3 {
        return None;
    }

    // Get last 3 components: artist, album, track_filename
    let artist = components[components.len() - 3].to_lowercase();
    let album = components[components.len() - 2].to_lowercase();
    let track_filename = components[components.len() - 1].to_lowercase();

    // Remove file extension
    let track_name = Path::new(&track_filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(&track_filename);

    // Remove track number prefix (e.g., "01 ", "1 ", etc.)
    // Pattern: starts with digits followed by space or dash
    let track_name = track_name
        .trim_start_matches(char::is_numeric)
        .trim_start_matches(' ')
        .trim_start_matches('-')
        .trim_start_matches(' ');

    Some(format!("{}/{}/{}", artist, album, track_name))
}

/// Result of syncing a playlist to Plex
#[derive(Debug, Clone)]
pub struct SyncPlaylistResult {
    pub missing_tracks: Vec<MissingTrack>,
    pub tracks_added: u32,
    pub tracks_removed: u32,
    pub tracks_skipped: u32,
}

/// Syncs a database playlist to Plex by matching tracks via file paths.
///
/// This function:
/// - Finds or creates a Plex playlist with the same name
/// - Matches tracks between database and Plex using file paths
/// - Adds missing tracks incrementally (never clears entire playlist)
/// - Removes extra tracks incrementally
/// - Returns statistics about the sync operation
///
/// # Arguments
/// * `db` - Database connection
/// * `client` - HTTP client for Plex API requests
/// * `playlist_id` - Database playlist ID to sync
///
/// # Errors
/// Returns an error if:
/// - Playlist not found in database
/// - No Plex server configured
/// - Multiple Plex servers configured (only one supported)
/// - Plex server missing access token
/// - Failed to fetch Plex library tracks
pub async fn sync_playlist_to_plex(
    db: &Database,
    client: &Client,
    playlist_id: i64,
) -> Result<SyncPlaylistResult> {
    log::info!("Starting sync of playlist ID {} to Plex", playlist_id);

    // Step 1: Get Database Playlist
    let playlist = entities::playlist::Entity::find_by_id(playlist_id)
        .one(&db.conn)
        .await
        .wrap_err("Failed to find playlist")?
        .ok_or_eyre(format!("Playlist with ID {} not found", playlist_id))?;

    log::info!("Found playlist: '{}' (ID: {})", playlist.name, playlist_id);

    // Step 2: Get Database Playlist Tracks
    let playlist_track_models = entities::playlist_track::Entity::find()
        .filter(entities::playlist_track::Column::PlaylistId.eq(playlist_id))
        .all(&db.conn)
        .await
        .wrap_err("Failed to fetch playlist tracks")?;

    let track_ids: Vec<i64> = playlist_track_models.iter().map(|pt| pt.track_id).collect();
    log::info!("Found {} tracks in database playlist", track_ids.len());

    let track_models = entities::track::Entity::find()
        .filter(entities::track::Column::Id.is_in(track_ids.clone()))
        .all(&db.conn)
        .await
        .wrap_err("Failed to fetch track details")?;

    // Step 3: Get Plex Server Configuration
    let servers = entities::plex_server::Entity::find()
        .all(&db.conn)
        .await
        .wrap_err("Failed to fetch Plex servers")?;

    if servers.is_empty() {
        return Err(color_eyre::eyre::eyre!(
            "No Plex server configured. Please add a Plex server first."
        ));
    }

    if servers.len() > 1 {
        return Err(color_eyre::eyre::eyre!(
            "Multiple Plex servers found ({}). Only one server is supported at a time.",
            servers.len()
        ));
    }

    let server = servers.into_iter().next().unwrap();
    log::info!("Using Plex server: '{}'", server.name);

    let access_token = server.access_token.as_ref().ok_or_eyre(
        "Plex server does not have an access token. Please authenticate the server first.",
    )?;

    let server_url = Url::parse(&server.server_url)
        .wrap_err(format!("Invalid server URL: {}", server.server_url))?;

    // Step 4: Build Plex Track Lookup Map
    log::info!("Fetching all Plex library tracks...");
    let sections = get_library_sections(client, &server_url, access_token).await?;
    let music_section_id = find_music_section_id(&sections)
        .ok_or_eyre("No music library section found on Plex server")?;

    let plex_tracks =
        get_all_tracks_paginated(client, &server_url, access_token, music_section_id, 1000).await?;
    log::info!("Found {} tracks in Plex library", plex_tracks.len());

    // Build normalized path key -> rating_key lookup map
    // Use normalized path (last 3 components: artist/album/track) for matching
    let mut plex_lookup: HashMap<String, String> = HashMap::new();
    for track in &plex_tracks {
        match track.file_path() {
            Ok(file_path) => {
                if let Some(normalized_key) = normalize_path_key(file_path) {
                    plex_lookup.insert(normalized_key, track.rating_key.clone());
                } else {
                    log::warn!(
                        "Failed to normalize path for track '{}': {}",
                        track.title,
                        file_path
                    );
                }
            }
            Err(e) => {
                log::warn!("Failed to get file path for track '{}': {}", track.title, e);
            }
        }
    }
    log::info!(
        "Built lookup map with {} normalized paths",
        plex_lookup.len()
    );

    // Step 5: Find or Create Plex Playlist
    let plex_playlists = get_playlists(client, &server_url, access_token).await?;
    let music_playlists: Vec<_> = plex_playlists
        .into_iter()
        .filter(is_music_playlist)
        .collect();

    let plex_playlist = match music_playlists.iter().find(|p| p.title == playlist.name) {
        Some(p) => {
            log::info!(
                "Found existing Plex playlist: '{}' (ID: {})",
                p.title,
                p.rating_key
            );
            p.clone()
        }
        None => {
            log::info!("Creating new Plex playlist: '{}'", playlist.name);
            create_music_playlist(client, &server_url, access_token, &playlist.name).await?
        }
    };

    // Step 6: Get Current Plex Playlist Tracks
    let current_plex_tracks =
        get_playlist_tracks(client, &server_url, access_token, &plex_playlist.rating_key).await?;

    let current_plex_rating_keys: HashSet<String> = current_plex_tracks
        .iter()
        .map(|t| t.rating_key.clone())
        .collect();

    log::info!(
        "Plex playlist currently has {} tracks",
        current_plex_rating_keys.len()
    );

    // Step 7: Identify Missing Tracks
    // Match tracks using normalized path keys (last 3 components)
    let mut missing_tracks = Vec::new();
    for track in &track_models {
        let normalized_key = normalize_path_key(&track.file_path);
        match normalized_key {
            Some(key) => {
                if !plex_lookup.contains_key(&key) {
                    missing_tracks.push(MissingTrack {
                        track_id: track.id,
                        file_path: track.file_path.clone(),
                        title: track.title.clone(),
                    });
                    log::warn!(
                        "Track '{}' (ID: {}) not found in Plex library: {} (normalized: {})",
                        track.title,
                        track.id,
                        track.file_path,
                        key
                    );
                }
            }
            None => {
                missing_tracks.push(MissingTrack {
                    track_id: track.id,
                    file_path: track.file_path.clone(),
                    title: track.title.clone(),
                });
                log::warn!(
                    "Track '{}' (ID: {}) path cannot be normalized (needs at least 3 components): {}",
                    track.title,
                    track.id,
                    track.file_path
                );
            }
        }
    }

    // Step 8: Calculate Differences
    // Build set of database track rating_keys (only for tracks that exist in Plex)
    // Match using normalized path keys
    let db_rating_keys: HashSet<String> = track_models
        .iter()
        .filter_map(|track| {
            normalize_path_key(&track.file_path).and_then(|key| plex_lookup.get(&key).cloned())
        })
        .collect();

    let tracks_to_add: Vec<String> = db_rating_keys
        .difference(&current_plex_rating_keys)
        .cloned()
        .collect();

    let tracks_to_remove: Vec<&crate::plex_rs::playlist::PlexTrack> = current_plex_tracks
        .iter()
        .filter(|t| !db_rating_keys.contains(&t.rating_key))
        .collect();

    log::info!("Tracks to add: {}", tracks_to_add.len());
    log::info!("Tracks to remove: {}", tracks_to_remove.len());

    // Step 9: Get Machine Identifier
    let machine_identifier = get_machine_identifier(client, &server_url, access_token).await?;
    log::debug!("Using machine identifier: {}", machine_identifier);

    // Step 10: Add Missing Tracks (Incremental)
    let mut tracks_added = 0;
    let mut tracks_skipped = 0;

    for rating_key in &tracks_to_add {
        match add_track_to_playlist(
            client,
            &server_url,
            access_token,
            &plex_playlist.rating_key,
            &machine_identifier,
            rating_key,
        )
        .await
        {
            Ok(()) => {
                tracks_added += 1;
                log::info!("Added track to playlist (rating_key: {})", rating_key);
            }
            Err(e) => {
                tracks_skipped += 1;
                log::error!("Failed to add track (rating_key: {}): {}", rating_key, e);
            }
        }
    }

    // Step 11: Remove Extra Tracks (Incremental)
    let mut tracks_removed = 0;

    for plex_track in &tracks_to_remove {
        let playlist_item_id = plex_track.playlist_item_id.ok_or_eyre(format!(
            "Track '{}' (rating_key: {}) missing playlist_item_id",
            plex_track.title, plex_track.rating_key
        ))?;

        match remove_track_from_playlist(
            client,
            &server_url,
            access_token,
            &plex_playlist.rating_key,
            playlist_item_id,
        )
        .await
        {
            Ok(()) => {
                tracks_removed += 1;
                log::info!(
                    "Removed track from playlist: '{}' (rating_key: {})",
                    plex_track.title,
                    plex_track.rating_key
                );
            }
            Err(e) => {
                tracks_skipped += 1;
                log::error!(
                    "Failed to remove track '{}' (rating_key: {}): {}",
                    plex_track.title,
                    plex_track.rating_key,
                    e
                );
            }
        }
    }

    // Step 12: Return Result
    let result = SyncPlaylistResult {
        missing_tracks,
        tracks_added,
        tracks_removed,
        tracks_skipped,
    };

    log::info!(
        "Sync complete for playlist '{}': {} added, {} removed, {} skipped, {} missing",
        playlist.name,
        result.tracks_added,
        result.tracks_removed,
        result.tracks_skipped,
        result.missing_tracks.len()
    );

    Ok(result)
}
