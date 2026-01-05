use axum::{
    body::{Body, Bytes},
    extract::{self, State},
    http::{HeaderValue, Response, StatusCode, header},
    response::IntoResponse,
};
use futures_util::StreamExt;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc::channel;
use tokio_stream::wrappers::ReceiverStream;

use crate::{http_server::state::AppState, soulseek::SingleFileResult};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct DownloadFileInput {
    username: String,
    token: String,
    filename: String,
    size: u64,
}

#[derive(serde::Serialize, Debug, Clone)]
#[serde(tag = "type")]
enum DownloadEvent {
    Started,
    Progress {
        bytes_downloaded: u64,
        total_bytes: u64,
    },
    Completed,
    #[serde(rename = "Failed")]
    Failed {
        message: String,
    },
}

#[axum::debug_handler]
pub async fn download_file(
    State(app_state): State<Arc<AppState>>,
    extract::Json(input): extract::Json<DownloadFileInput>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let result = SingleFileResult {
        username: input.username.clone(),
        token: input.token.clone(),
        filename: input.filename.clone(),
        size: input.size,
        slots_free: true,
        avg_speed: 0.0,
        queue_length: 0,
        attrs: HashMap::new(),
    };

    let mut download_receiver = app_state
        .soulseek_context
        .download_file(&result, &app_state.download_directory)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to initiate download: {e}"),
            )
                .into_response()
        })?;

    let (tx, rx) = channel::<DownloadEvent>(64);

    // Send initial "Started" event immediately to establish the HTTP stream
    // This prevents the connection from being aborted if the task exits before sending data
    let _ = tx.send(DownloadEvent::Started).await;

    let ctx = app_state.soulseek_context.clone();

    // Spawn async task to consume the download status receiver
    tokio::spawn(async move {
        while let Some(status) = download_receiver.recv().await {
            let event = match status {
                soulseek_rs::DownloadStatus::Queued => {
                    // Skip sending Started since we already sent it initially
                    continue;
                }

                soulseek_rs::DownloadStatus::InProgress {
                    bytes_downloaded,
                    total_bytes,
                    speed_bytes_per_sec: _,
                } => DownloadEvent::Progress {
                    bytes_downloaded,
                    total_bytes,
                },

                soulseek_rs::DownloadStatus::Completed => {
                    let _ = tx.send(DownloadEvent::Completed).await;
                    break;
                }

                soulseek_rs::DownloadStatus::Failed => {
                    ctx.report_session_error("Download failed").await;
                    let _ = tx
                        .send(DownloadEvent::Failed {
                            message: "Failed to download file".to_string(),
                        })
                        .await;
                    break;
                }

                soulseek_rs::DownloadStatus::TimedOut => {
                    ctx.report_session_error("Download timed out").await;
                    let _ = tx
                        .send(DownloadEvent::Failed {
                            message: "Download timed out".to_string(),
                        })
                        .await;
                    break;
                }
            };

            if tx.send(event).await.is_err() {
                // Client went away, stop producing
                break;
            }
        }

        // When this function exits, tx is dropped, and the response stream ends cleanly.
    });

    let json_stream = ReceiverStream::new(rx).map(|item| {
        let mut line = serde_json::to_vec(&item)
            .map_err(|e| std::io::Error::other(format!("json encode: {e}")))?;
        line.push(b'\n');
        Ok::<Bytes, std::io::Error>(Bytes::from(line))
    });

    let body = Body::from_stream(json_stream);

    let mut response = Response::new(body);
    *response.status_mut() = StatusCode::OK;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/x-ndjson; charset=utf-8"),
    );
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    // Explicitly set Transfer-Encoding: chunked for streaming responses
    // Note: Axum should handle this automatically, but being explicit helps with some clients
    response.headers_mut().insert(
        header::TRANSFER_ENCODING,
        HeaderValue::from_static("chunked"),
    );

    Ok::<Response<Body>, axum::response::Response>(response)
}
