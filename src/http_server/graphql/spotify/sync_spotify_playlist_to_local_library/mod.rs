//! Module for syncing Spotify playlists to the local music library.
//!
//! This module handles the complete workflow of:
//! 1. Creating sync state to track progress
//! 2. Processing each Spotify track (downloading/matching)
//! 3. Adding successfully processed tracks to the local playlist
//! 4. Updating sync state throughout the process

mod add_tracks_to_playlist;
mod create_sync_state;
mod process_track;
mod sync_task;
mod task;

pub use task::sync_spotify_playlist_to_local_library_task;
