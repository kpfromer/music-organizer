use audiotags::Tag;
use color_eyre::eyre::Context;
use std::sync::Arc;
use tracing;

use crate::http_server::state::AppState;
use axum::{
    extract::{Path, State},
    http::{StatusCode, header},
    response::IntoResponse,
};

pub async fn get_track_album_art_image(
    State(app_state): State<Arc<AppState>>,
    Path(track_id): Path<i64>,
) -> impl IntoResponse {
    let track = match app_state.db.get_track(track_id).await {
        Ok(Some(track)) => track,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                format!("Track not found: {}", track_id),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!("Failed to get track: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Track not found: {}", track_id),
            )
                .into_response();
        }
    };

    let tag = match Tag::new()
        .read_from_path(track.file_path)
        .wrap_err("Failed to read audio tags")
    {
        Ok(tag) => tag,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read audio tags: {}", e),
            )
                .into_response();
        }
    };

    if let Some(album_cover) = tag.album_cover() {
        let mime_type: String = album_cover.mime_type.into();
        return (
            StatusCode::OK,
            [(header::CONTENT_TYPE, mime_type)],
            album_cover.data.to_owned(),
        )
            .into_response();
    }

    (
        StatusCode::NOT_FOUND,
        format!("Track art image not found: {}", track_id),
    )
        .into_response()
}
