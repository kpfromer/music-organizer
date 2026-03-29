use std::sync::Arc;

use axum::{
    extract::State,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder};

use crate::{entities, entities::wishlist_item::WishlistStatus, http_server::state::AppState};

pub async fn wishlist_export_csv(State(app_state): State<Arc<AppState>>) -> Response {
    let items = match entities::wishlist_item::Entity::find()
        .filter(
            Condition::any()
                .add(entities::wishlist_item::Column::Status.eq(WishlistStatus::Pending))
                .add(entities::wishlist_item::Column::Status.eq(WishlistStatus::Failed)),
        )
        .order_by_asc(entities::wishlist_item::Column::CreatedAt)
        .all(&app_state.db.conn)
        .await
    {
        Ok(items) => items,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to fetch wishlist items: {}", e),
            )
                .into_response();
        }
    };

    let track_ids: Vec<String> = items.iter().map(|i| i.spotify_track_id.clone()).collect();

    let tracks = match entities::spotify_track::Entity::find()
        .filter(entities::spotify_track::Column::SpotifyTrackId.is_in(track_ids))
        .all(&app_state.db.conn)
        .await
    {
        Ok(tracks) => tracks,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to fetch spotify tracks: {}", e),
            )
                .into_response();
        }
    };

    let track_map: std::collections::HashMap<String, entities::spotify_track::Model> = tracks
        .into_iter()
        .map(|t| (t.spotify_track_id.clone(), t))
        .collect();

    let escape = |s: &str| format!("\"{}\"", s.replace('"', "\"\""));

    let mut csv = String::from("title,album,artist,all_artists,duration_seconds\n");
    for item in &items {
        if let Some(track) = track_map.get(&item.spotify_track_id) {
            let artists = &track.artists.0;
            let primary = artists.first().map(String::as_str).unwrap_or("");
            let all = artists.join(", ");
            let duration = track
                .duration
                .map(|d| (d / 1000).to_string())
                .unwrap_or_default();
            csv.push_str(&format!(
                "{},{},{},{},{}\n",
                escape(&track.title),
                escape(&track.album),
                escape(primary),
                escape(&all),
                duration,
            ));
        }
    }

    (
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8".to_string()),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"wishlist.csv\"".to_string(),
            ),
        ],
        csv,
    )
        .into_response()
}
