use crate::{http_server::state::AppState, import_track::watch_directory};
use std::{path::Path, sync::Arc, time::Duration};

pub mod youtube;

pub fn run_background_tasks(app_state: Arc<AppState>, watch_directory_path: &Path) {
    // TODO: import files

    let watch_directory_path = watch_directory_path.to_path_buf();
    let app_state_clone = app_state.clone();
    let _ = tokio::spawn(async move {
        tracing::info!("Watching directory in background");
        // TODO: handle errors hear
        // maybe restart 5 times with a delay between each restart
        // if it fails after 5 restarts, log an error and exit?
        match watch_directory(
            &watch_directory_path,
            &app_state_clone.api_key,
            &app_state_clone.config,
            &app_state_clone.db,
        )
        .await
        {
            Ok(_) => {}
            Err(e) => {
                tracing::error!("Error watching directory: {}", e);
            }
        }
    });

    // Fetch youtube videos for subscribed channels
    let youtube_db = app_state.db.clone();
    let _ = tokio::spawn(async move {
        tracing::info!("Fetching youtube videos in background");
        loop {
            tokio::time::sleep(Duration::from_mins(3)).await;
            match youtube::add_new_videos(&youtube_db).await {
                Ok(_) => {}
                Err(e) => tracing::error!("Failed to add new youtube videos: {}", e),
            }
        }
    });
}
