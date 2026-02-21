use std::sync::Arc;

use chrono::Utc;
use color_eyre::eyre::{OptionExt, Result};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, Set, TransactionTrait,
};

use crate::database::Database;
use crate::entities;

pub struct PlaylistService {
    db: Arc<Database>,
}

impl PlaylistService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        name: String,
        description: Option<String>,
    ) -> Result<entities::playlist::Model> {
        let playlist = entities::playlist::ActiveModel {
            name: Set(name),
            description: Set(description),
            ..Default::default()
        };

        let model = playlist
            .insert(&self.db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to create playlist: {}", e))?;

        Ok(model)
    }

    pub async fn add_track(&self, playlist_id: i64, track_id: i64) -> Result<()> {
        let playlist = entities::playlist::Entity::find_by_id(playlist_id)
            .one(&self.db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to find playlist: {}", e))?
            .ok_or_eyre("Playlist not found")?;

        entities::track::Entity::find_by_id(track_id)
            .one(&self.db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to find track: {}", e))?
            .ok_or_eyre("Track not found")?;

        // Check if track is already in playlist
        let existing = entities::playlist_track::Entity::find()
            .filter(
                Condition::all()
                    .add(entities::playlist_track::Column::PlaylistId.eq(playlist_id))
                    .add(entities::playlist_track::Column::TrackId.eq(track_id)),
            )
            .one(&self.db.conn)
            .await
            .map_err(|e| {
                color_eyre::eyre::eyre!("Failed to check existing playlist track: {}", e)
            })?;

        if existing.is_some() {
            return Ok(());
        }

        let now = Utc::now();
        let playlist_track = entities::playlist_track::ActiveModel {
            playlist_id: Set(playlist_id),
            track_id: Set(track_id),
            created_at: Set(now),
            updated_at: Set(now),
        };

        self.db
            .conn
            .transaction::<_, (), color_eyre::eyre::Report>(|txn| {
                Box::pin(async move {
                    playlist_track.insert(txn).await.map_err(|e| {
                        color_eyre::eyre::eyre!("Failed to add track to playlist: {}", e)
                    })?;

                    let mut playlist_model: entities::playlist::ActiveModel = playlist.into();
                    playlist_model.updated_at = Set(Utc::now());
                    playlist_model
                        .update(txn)
                        .await
                        .map_err(|e| color_eyre::eyre::eyre!("Failed to update playlist: {}", e))?;

                    Ok(())
                })
            })
            .await
            .map_err(|_| color_eyre::eyre::eyre!("Failed to add track to playlist"))?;

        Ok(())
    }
}
