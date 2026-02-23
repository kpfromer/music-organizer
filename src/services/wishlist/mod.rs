pub mod background_task;

use std::sync::Arc;

use color_eyre::eyre::{OptionExt, Result, WrapErr};
use sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, Set,
};

use crate::database::Database;
use crate::entities;
use crate::entities::wishlist_item::WishlistStatus;
use crate::services::track::PaginatedResult;

pub struct WishlistStats {
    pub pending: i64,
    pub searching: i64,
    pub downloading: i64,
    pub importing: i64,
    pub completed: i64,
    pub failed: i64,
}

pub struct WishlistService {
    db: Arc<Database>,
}

impl WishlistService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub async fn add_to_wishlist(
        &self,
        spotify_track_id: &str,
    ) -> Result<entities::wishlist_item::Model> {
        // Verify the spotify track exists
        entities::spotify_track::Entity::find()
            .filter(entities::spotify_track::Column::SpotifyTrackId.eq(spotify_track_id))
            .one(&self.db.conn)
            .await
            .wrap_err("Failed to fetch spotify track")?
            .ok_or_eyre("Spotify track not found")?;

        // Check if already wishlisted
        let existing = entities::wishlist_item::Entity::find()
            .filter(entities::wishlist_item::Column::SpotifyTrackId.eq(spotify_track_id))
            .one(&self.db.conn)
            .await
            .wrap_err("Failed to check existing wishlist item")?;

        if let Some(existing) = existing {
            // If it was previously completed or failed, reset to pending
            if existing.status == WishlistStatus::Completed
                || existing.status == WishlistStatus::Failed
            {
                let mut active: entities::wishlist_item::ActiveModel = existing.into();
                active.status = Set(WishlistStatus::Pending);
                active.error_reason = Set(None);
                active.next_retry_at = Set(None);
                let updated = active
                    .update(&self.db.conn)
                    .await
                    .wrap_err("Failed to reset wishlist item")?;
                return Ok(updated);
            }
            return Ok(existing);
        }

        let item = entities::wishlist_item::ActiveModel {
            spotify_track_id: Set(spotify_track_id.to_string()),
            ..entities::wishlist_item::ActiveModel::new()
        };

        let model = item
            .insert(&self.db.conn)
            .await
            .wrap_err("Failed to create wishlist item")?;

        Ok(model)
    }

    pub async fn remove_from_wishlist(&self, id: i64) -> Result<()> {
        entities::wishlist_item::Entity::delete_by_id(id)
            .exec(&self.db.conn)
            .await
            .wrap_err("Failed to delete wishlist item")?;
        Ok(())
    }

    pub async fn retry_wishlist_item(&self, id: i64) -> Result<entities::wishlist_item::Model> {
        let item = entities::wishlist_item::Entity::find_by_id(id)
            .one(&self.db.conn)
            .await
            .wrap_err("Failed to fetch wishlist item")?
            .ok_or_eyre("Wishlist item not found")?;

        let mut active: entities::wishlist_item::ActiveModel = item.into();
        active.status = Set(WishlistStatus::Pending);
        active.error_reason = Set(None);
        active.next_retry_at = Set(None);

        let updated = active
            .update(&self.db.conn)
            .await
            .wrap_err("Failed to reset wishlist item")?;

        Ok(updated)
    }

    pub async fn list_wishlist_items(
        &self,
        status_filter: Option<WishlistStatus>,
        page: usize,
        page_size: usize,
    ) -> Result<
        PaginatedResult<(
            entities::wishlist_item::Model,
            entities::spotify_track::Model,
        )>,
    > {
        let mut query = entities::wishlist_item::Entity::find();

        if let Some(status) = &status_filter {
            query = query.filter(entities::wishlist_item::Column::Status.eq(status.clone()));
        }

        let total_count = query
            .clone()
            .count(&self.db.conn)
            .await
            .wrap_err("Failed to count wishlist items")?;

        let offset = (page.saturating_sub(1)) * page_size;
        let items = query
            .order_by_desc(entities::wishlist_item::Column::CreatedAt)
            .limit(page_size as u64)
            .offset(offset as u64)
            .all(&self.db.conn)
            .await
            .wrap_err("Failed to fetch wishlist items")?;

        let mut result = Vec::new();
        for item in items {
            let spotify_track = entities::spotify_track::Entity::find()
                .filter(entities::spotify_track::Column::SpotifyTrackId.eq(&item.spotify_track_id))
                .one(&self.db.conn)
                .await
                .wrap_err("Failed to fetch spotify track")?
                .ok_or_eyre("Spotify track not found for wishlist item")?;
            result.push((item, spotify_track));
        }

        Ok(PaginatedResult {
            items: result,
            total_count,
            page,
            page_size,
        })
    }

    pub async fn get_stats(&self) -> Result<WishlistStats> {
        let count_status = |status: WishlistStatus| {
            let db = self.db.clone();
            async move {
                entities::wishlist_item::Entity::find()
                    .filter(entities::wishlist_item::Column::Status.eq(status))
                    .count(&db.conn)
                    .await
                    .map(|c| c as i64)
                    .wrap_err("Failed to count wishlist items")
            }
        };

        Ok(WishlistStats {
            pending: count_status(WishlistStatus::Pending).await?,
            searching: count_status(WishlistStatus::Searching).await?,
            downloading: count_status(WishlistStatus::Downloading).await?,
            importing: count_status(WishlistStatus::Importing).await?,
            completed: count_status(WishlistStatus::Completed).await?,
            failed: count_status(WishlistStatus::Failed).await?,
        })
    }
}
