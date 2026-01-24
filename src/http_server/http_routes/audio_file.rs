use axum_extra::{TypedHeader, headers::Range};
use std::sync::Arc;
use tracing;

use tokio::fs::File;

use axum_range::{KnownSize, Ranged};

use crate::http_server::state::AppState;
use axum::{
    extract::{Path, State},
    http::{StatusCode, header},
    response::IntoResponse,
};

pub async fn audio_file(
    State(app_state): State<Arc<AppState>>,
    Path(track_id): Path<i64>,
    range: Option<TypedHeader<Range>>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let track = match app_state.db.get_track(track_id).await {
        Ok(Some(track)) => track,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                format!("Track not found: {}", track_id),
            )
                .into_response());
        }
        Err(e) => {
            tracing::error!("Get track database error: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Get track database error",
            )
                .into_response());
        }
    };

    let file = File::open(&track.file_path).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to open file: {}", e),
        )
            .into_response()
    })?;

    // TODO: use tokio::spawn to get the mime type in the background
    let mime_type = infer::get_from_path(track.file_path)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get file mime type: {}", e),
            )
                .into_response()
        })?
        .ok_or(
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to get file mime type",
            )
                .into_response(),
        )?;

    let body = KnownSize::file(file).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to get file size: {}", e),
        )
            .into_response()
    })?;

    let range = range.map(|TypedHeader(range)| range);

    Ok((
        [(header::CONTENT_TYPE, mime_type.mime_type().to_string())],
        Ranged::new(range, body).into_response(),
    ))
}
