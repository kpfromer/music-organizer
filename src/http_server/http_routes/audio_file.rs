use axum_extra::{TypedHeader, headers::Range};
use color_eyre::eyre::Context;
use std::sync::Arc;

use tokio::fs::File;

use axum_range::{KnownSize, Ranged};

use crate::http_server::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
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
            log::error!("Failed to get track: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Track not found: {}", track_id),
            )
                .into_response());
        }
    };

    let file = File::open(track.file_path).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to open file: {}", e),
        )
            .into_response()
    })?;

    let body = KnownSize::file(file).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to get file size: {}", e),
        )
            .into_response()
    })?;

    let range = range.map(|TypedHeader(range)| range);
    Ok(Ranged::new(range, body).into_response())
}
