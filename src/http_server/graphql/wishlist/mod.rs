use async_graphql::{Context, Object};
use chrono::{DateTime, Utc};
use color_eyre::eyre::{OptionExt, WrapErr};

use crate::database::Database;
use crate::entities;
use crate::entities::wishlist_item::WishlistStatus;
use crate::http_server::graphql::context::get_app_state;
use crate::http_server::graphql_error::GraphqlResult;
use crate::services::wishlist::WishlistService;

#[derive(Default)]
pub struct WishlistQuery;

#[derive(Default)]
pub struct WishlistMutation;

#[derive(async_graphql::SimpleObject)]
pub struct WishlistItem {
    pub id: i64,
    pub spotify_track_id: String,
    pub status: String,
    pub error_reason: Option<String>,
    pub attempts_count: i32,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub track_title: String,
    pub track_artists: Vec<String>,
    pub track_album: String,
}

#[derive(async_graphql::SimpleObject)]
pub struct WishlistItemsResponse {
    pub items: Vec<WishlistItem>,
    pub total_count: i64,
    pub page: i32,
    pub page_size: i32,
}

#[derive(async_graphql::SimpleObject)]
pub struct WishlistStatsGql {
    pub pending: i64,
    pub searching: i64,
    pub downloading: i64,
    pub importing: i64,
    pub completed: i64,
    pub failed: i64,
}

fn status_to_filter(status: &str) -> Option<WishlistStatus> {
    match status {
        "pending" => Some(WishlistStatus::Pending),
        "searching" => Some(WishlistStatus::Searching),
        "downloading" => Some(WishlistStatus::Downloading),
        "importing" => Some(WishlistStatus::Importing),
        "completed" => Some(WishlistStatus::Completed),
        "failed" => Some(WishlistStatus::Failed),
        _ => None,
    }
}

fn status_to_string(status: &WishlistStatus) -> &'static str {
    match status {
        WishlistStatus::Pending => "pending",
        WishlistStatus::Searching => "searching",
        WishlistStatus::Downloading => "downloading",
        WishlistStatus::Importing => "importing",
        WishlistStatus::Completed => "completed",
        WishlistStatus::Failed => "failed",
    }
}

fn timestamp_to_datetime(ts: i64) -> GraphqlResult<DateTime<Utc>> {
    DateTime::from_timestamp(ts, 0)
        .ok_or_eyre("Failed to convert timestamp")
        .map_err(Into::into)
}

fn optional_timestamp_to_datetime(ts: Option<i64>) -> GraphqlResult<Option<DateTime<Utc>>> {
    ts.map(timestamp_to_datetime).transpose()
}

async fn fetch_spotify_track(
    db: &Database,
    spotify_track_id: &str,
) -> color_eyre::Result<entities::spotify_track::Model> {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    entities::spotify_track::Entity::find()
        .filter(entities::spotify_track::Column::SpotifyTrackId.eq(spotify_track_id))
        .one(&db.conn)
        .await
        .wrap_err("Failed to fetch spotify track")?
        .ok_or_eyre("Spotify track not found")
}

fn to_wishlist_item_gql(
    item: entities::wishlist_item::Model,
    spotify_track: entities::spotify_track::Model,
) -> GraphqlResult<WishlistItem> {
    Ok(WishlistItem {
        id: item.id,
        spotify_track_id: item.spotify_track_id,
        status: status_to_string(&item.status).to_string(),
        error_reason: item.error_reason,
        attempts_count: item.attempts_count,
        last_attempt_at: optional_timestamp_to_datetime(item.last_attempt_at)?,
        next_retry_at: optional_timestamp_to_datetime(item.next_retry_at)?,
        created_at: timestamp_to_datetime(item.created_at)?,
        updated_at: timestamp_to_datetime(item.updated_at)?,
        track_title: spotify_track.title,
        track_artists: spotify_track.artists.0,
        track_album: spotify_track.album,
    })
}

#[Object]
impl WishlistQuery {
    async fn wishlist_items(
        &self,
        ctx: &Context<'_>,
        page: Option<i32>,
        page_size: Option<i32>,
        status: Option<String>,
    ) -> GraphqlResult<WishlistItemsResponse> {
        let app_state = get_app_state(ctx)?;
        let service = WishlistService::new(app_state.db.clone());

        let page = page.unwrap_or(1).max(1) as usize;
        let page_size = page_size.unwrap_or(25).clamp(1, 100) as usize;
        let status_filter = status.as_deref().and_then(status_to_filter);

        let result = service
            .list_wishlist_items(status_filter, page, page_size)
            .await?;

        let items: Vec<WishlistItem> = result
            .items
            .into_iter()
            .map(|(item, spotify_track)| to_wishlist_item_gql(item, spotify_track))
            .collect::<GraphqlResult<Vec<_>>>()?;

        Ok(WishlistItemsResponse {
            items,
            total_count: result.total_count as i64,
            page: result.page as i32,
            page_size: result.page_size as i32,
        })
    }

    async fn wishlist_stats(&self, ctx: &Context<'_>) -> GraphqlResult<WishlistStatsGql> {
        let app_state = get_app_state(ctx)?;
        let service = WishlistService::new(app_state.db.clone());
        let stats = service.get_stats().await?;

        Ok(WishlistStatsGql {
            pending: stats.pending,
            searching: stats.searching,
            downloading: stats.downloading,
            importing: stats.importing,
            completed: stats.completed,
            failed: stats.failed,
        })
    }
}

#[Object]
impl WishlistMutation {
    async fn add_to_wishlist(
        &self,
        ctx: &Context<'_>,
        spotify_track_id: String,
    ) -> GraphqlResult<WishlistItem> {
        let app_state = get_app_state(ctx)?;
        let service = WishlistService::new(app_state.db.clone());
        let item = service.add_to_wishlist(&spotify_track_id).await?;
        let spotify_track = fetch_spotify_track(&app_state.db, &item.spotify_track_id).await?;

        // Wake the wishlist background task
        app_state.wishlist_notify.notify_one();

        to_wishlist_item_gql(item, spotify_track)
    }

    async fn remove_from_wishlist(&self, ctx: &Context<'_>, id: i64) -> GraphqlResult<bool> {
        let app_state = get_app_state(ctx)?;
        let service = WishlistService::new(app_state.db.clone());
        service.remove_from_wishlist(id).await?;
        Ok(true)
    }

    async fn retry_wishlist_item(&self, ctx: &Context<'_>, id: i64) -> GraphqlResult<WishlistItem> {
        let app_state = get_app_state(ctx)?;
        let service = WishlistService::new(app_state.db.clone());
        let item = service.retry_wishlist_item(id).await?;
        let spotify_track = fetch_spotify_track(&app_state.db, &item.spotify_track_id).await?;

        // Wake the wishlist background task
        app_state.wishlist_notify.notify_one();

        to_wishlist_item_gql(item, spotify_track)
    }
}
