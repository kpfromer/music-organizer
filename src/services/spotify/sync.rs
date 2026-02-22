use std::sync::Arc;

use color_eyre::eyre::{Result, WrapErr};
use sea_orm::{ActiveModelBehavior, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use tracing;

use crate::database::Database;
use crate::entities;
use crate::ports::spotify::{SpotifyApiPlaylist, SpotifyApiTrack, SpotifyClient};

/// Query-only service that doesn't require a Spotify API client adapter.
pub struct SpotifySyncQueryService {
    db: Arc<Database>,
}

impl SpotifySyncQueryService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub async fn list_playlists(
        &self,
        account_id: i64,
    ) -> Result<Vec<entities::spotify_playlist::Model>> {
        entities::spotify_playlist::Entity::find()
            .filter(entities::spotify_playlist::Column::AccountId.eq(account_id))
            .all(&self.db.conn)
            .await
            .wrap_err("Failed to fetch spotify playlists")
    }

    pub async fn get_playlist_sync_state(
        &self,
        spotify_playlist_id: i64,
    ) -> Result<Option<entities::spotify_playlist_sync_state::Model>> {
        entities::spotify_playlist_sync_state::Entity::find()
            .filter(
                entities::spotify_playlist_sync_state::Column::SpotifyPlaylistId
                    .eq(spotify_playlist_id),
            )
            .one(&self.db.conn)
            .await
            .wrap_err("Failed to fetch spotify playlist sync state")
    }

    pub async fn list_download_failures(
        &self,
        spotify_playlist_id: i64,
    ) -> Result<Vec<entities::spotify_track_download_failure::Model>> {
        entities::spotify_track_download_failure::Entity::find()
            .filter(
                entities::spotify_track_download_failure::Column::SpotifyPlaylistId
                    .eq(spotify_playlist_id),
            )
            .all(&self.db.conn)
            .await
            .wrap_err("Failed to fetch spotify track download failures")
    }
}

pub struct SpotifySyncService<C: SpotifyClient> {
    db: Arc<Database>,
    client: C,
}

impl<C: SpotifyClient> SpotifySyncService<C> {
    pub fn new(db: Arc<Database>, client: C) -> Self {
        Self { db, client }
    }

    pub async fn sync_account_playlists(&self, account_id: i64) -> Result<()> {
        let playlists = self.client.current_user_playlists().await?;

        let txn = self
            .db
            .conn
            .begin()
            .await
            .wrap_err("Failed to begin transaction")?;

        for playlist in playlists {
            let saved_playlist = self.upsert_playlist(&txn, account_id, &playlist).await?;

            tracing::info!("Saved spotify playlist: {:?}", saved_playlist);

            let tracks = self
                .client
                .playlist_tracks(&saved_playlist.spotify_id)
                .await?;

            for track in tracks {
                let track_id = self.upsert_track(&txn, &track).await?;
                self.link_track_to_playlist(&txn, &track_id, saved_playlist.id)
                    .await?;
            }
        }

        txn.commit()
            .await
            .wrap_err("Failed to commit transaction")?;

        Ok(())
    }

    async fn upsert_playlist(
        &self,
        txn: &impl sea_orm::ConnectionTrait,
        account_id: i64,
        playlist: &SpotifyApiPlaylist,
    ) -> Result<entities::spotify_playlist::Model> {
        if let Some(existing_playlist) = entities::spotify_playlist::Entity::find()
            .filter(entities::spotify_playlist::Column::SpotifyId.eq(&playlist.id))
            .one(txn)
            .await
            .wrap_err("Failed to fetch saved spotify playlist")?
        {
            let mut model: entities::spotify_playlist::ActiveModel = existing_playlist.into();
            model.account_id = Set(account_id);
            model.spotify_id = Set(playlist.id.clone());
            model.name = Set(playlist.name.clone());
            model.description = Set(playlist.description.clone());
            model.snapshot_id = Set(playlist.snapshot_id.clone());
            model.track_count = Set(playlist.total_tracks);
            model.updated_at = Set(chrono::Utc::now().timestamp());

            Ok(entities::spotify_playlist::Entity::update(model)
                .exec(txn)
                .await
                .wrap_err("Failed to update spotify playlist")?)
        } else {
            let model = entities::spotify_playlist::ActiveModel {
                account_id: Set(account_id),
                spotify_id: Set(playlist.id.clone()),
                name: Set(playlist.name.clone()),
                description: Set(playlist.description.clone()),
                snapshot_id: Set(playlist.snapshot_id.clone()),
                track_count: Set(playlist.total_tracks),
                ..entities::spotify_playlist::ActiveModel::new()
            };

            Ok(entities::spotify_playlist::Entity::insert(model)
                .exec_with_returning(txn)
                .await
                .wrap_err("Failed to save spotify playlist")?)
        }
    }

    async fn upsert_track(
        &self,
        txn: &impl sea_orm::ConnectionTrait,
        track: &SpotifyApiTrack,
    ) -> Result<String> {
        match entities::spotify_track::Entity::find()
            .filter(entities::spotify_track::Column::SpotifyTrackId.eq(&track.id))
            .one(txn)
            .await
            .wrap_err("Failed to fetch saved spotify track")?
        {
            Some(existing_track) => {
                let mut model: entities::spotify_track::ActiveModel = existing_track.into();
                model.updated_at = Set(chrono::Utc::now().timestamp());
                model.duration = Set(Some(track.duration_ms));
                model.artists = Set(entities::spotify_track::StringVec(track.artists.clone()));
                model.album = Set(track.album_name.clone());
                model.isrc = Set(track.isrc.clone());
                model.barcode = Set(track.upc.clone());
                tracing::info!("Updated spotify track in db: {:?}", model);
                entities::spotify_track::Entity::update(model)
                    .exec(txn)
                    .await
                    .wrap_err("Failed to update spotify track")?;
            }
            None => {
                let model = entities::spotify_track::ActiveModel {
                    spotify_track_id: Set(track.id.clone()),
                    title: Set(track.name.clone()),
                    duration: Set(Some(track.duration_ms)),
                    artists: Set(entities::spotify_track::StringVec(track.artists.clone())),
                    album: Set(track.album_name.clone()),
                    isrc: Set(track.isrc.clone()),
                    barcode: Set(track.upc.clone()),
                    created_at: Set(chrono::Utc::now().timestamp()),
                    updated_at: Set(chrono::Utc::now().timestamp()),
                    local_track_id: Set(None),
                };
                entities::spotify_track::Entity::insert(model)
                    .exec(txn)
                    .await
                    .wrap_err("Failed to save spotify track")?;
                tracing::info!("Saved new spotify track to db");
            }
        }

        Ok(track.id.clone())
    }

    async fn link_track_to_playlist(
        &self,
        txn: &impl sea_orm::ConnectionTrait,
        track_id: &str,
        playlist_id: i64,
    ) -> Result<()> {
        if entities::spotify_track_playlist::Entity::find()
            .filter(entities::spotify_track_playlist::Column::SpotifyTrackId.eq(track_id))
            .filter(entities::spotify_track_playlist::Column::SpotifyPlaylistId.eq(playlist_id))
            .one(txn)
            .await
            .wrap_err("Failed to fetch saved spotify track playlist")?
            .is_some()
        {
            return Ok(());
        }

        let model = entities::spotify_track_playlist::ActiveModel {
            spotify_track_id: Set(track_id.to_string()),
            spotify_playlist_id: Set(playlist_id),
        };
        entities::spotify_track_playlist::Entity::insert(model)
            .exec(txn)
            .await
            .wrap_err("Failed to save spotify track playlist")?;
        tracing::info!("Saved spotify track playlist");

        Ok(())
    }
}
