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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::spotify::{MockSpotifyClient, SpotifyApiPlaylist, SpotifyApiTrack};
    use crate::test_utils::test_db;
    use sea_orm::{ActiveModelBehavior, ActiveModelTrait};

    async fn insert_account(db: &Database) -> entities::spotify_account::Model {
        let account = entities::spotify_account::ActiveModel {
            user_id: Set("test_user".into()),
            display_name: Set(Some("Test User".into())),
            access_token: Set("at".into()),
            refresh_token: Set("rt".into()),
            token_expiry: Set(0),
            ..entities::spotify_account::ActiveModel::new()
        };
        account.insert(&db.conn).await.unwrap()
    }

    async fn insert_playlist(
        db: &Database,
        account_id: i64,
        spotify_id: &str,
        name: &str,
    ) -> entities::spotify_playlist::Model {
        let playlist = entities::spotify_playlist::ActiveModel {
            account_id: Set(account_id),
            spotify_id: Set(spotify_id.into()),
            name: Set(name.into()),
            snapshot_id: Set("snap1".into()),
            track_count: Set(0),
            ..entities::spotify_playlist::ActiveModel::new()
        };
        playlist.insert(&db.conn).await.unwrap()
    }

    // ---- SpotifySyncQueryService tests ----

    #[tokio::test]
    async fn test_list_playlists() {
        let db = test_db().await;
        let account = insert_account(&db).await;
        insert_playlist(&db, account.id, "sp1", "Playlist 1").await;
        insert_playlist(&db, account.id, "sp2", "Playlist 2").await;

        let service = SpotifySyncQueryService::new(db);
        let playlists = service.list_playlists(account.id).await.unwrap();

        assert_eq!(playlists.len(), 2);
    }

    #[tokio::test]
    async fn test_list_playlists_wrong_account() {
        let db = test_db().await;
        let account = insert_account(&db).await;
        insert_playlist(&db, account.id, "sp1", "Playlist 1").await;

        let service = SpotifySyncQueryService::new(db);
        let playlists = service.list_playlists(9999).await.unwrap();

        assert!(playlists.is_empty());
    }

    #[tokio::test]
    async fn test_get_playlist_sync_state() {
        let db = test_db().await;
        let account = insert_account(&db).await;
        let playlist = insert_playlist(&db, account.id, "sp1", "Playlist").await;

        // Insert sync state
        let state = entities::spotify_playlist_sync_state::ActiveModel {
            spotify_playlist_id: Set(playlist.id),
            ..entities::spotify_playlist_sync_state::ActiveModel::new()
        };
        state.insert(&db.conn).await.unwrap();

        let service = SpotifySyncQueryService::new(db);
        let result = service.get_playlist_sync_state(playlist.id).await.unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap().spotify_playlist_id, playlist.id);
    }

    #[tokio::test]
    async fn test_get_playlist_sync_state_not_found() {
        let db = test_db().await;
        let service = SpotifySyncQueryService::new(db);

        let result = service.get_playlist_sync_state(9999).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_list_download_failures() {
        let db = test_db().await;
        let account = insert_account(&db).await;
        let playlist = insert_playlist(&db, account.id, "sp1", "Playlist").await;

        // Insert a download failure
        let failure = entities::spotify_track_download_failure::ActiveModel {
            spotify_playlist_id: Set(playlist.id),
            spotify_track_id: Set("track1".into()),
            track_name: Set("Bad Track".into()),
            artist_name: Set("Artist".into()),
            reason: Set("not found".into()),
            ..entities::spotify_track_download_failure::ActiveModel::new()
        };
        failure.insert(&db.conn).await.unwrap();

        let service = SpotifySyncQueryService::new(db);
        let failures = service.list_download_failures(playlist.id).await.unwrap();

        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].track_name, "Bad Track");
    }

    // ---- SpotifySyncService tests ----

    fn make_mock_client(
        playlists: Vec<SpotifyApiPlaylist>,
        tracks: Vec<SpotifyApiTrack>,
    ) -> MockSpotifyClient {
        let mut client = MockSpotifyClient::new();
        client
            .expect_current_user_playlists()
            .returning(move || Ok(playlists.clone()));
        let tracks_clone = tracks.clone();
        client
            .expect_playlist_tracks()
            .returning(move |_| Ok(tracks_clone.clone()));
        client
    }

    #[tokio::test]
    async fn test_sync_account_playlists() {
        let db = test_db().await;
        let account = insert_account(&db).await;

        let playlists = vec![SpotifyApiPlaylist {
            id: "pl1".into(),
            name: "My Playlist".into(),
            description: Some("desc".into()),
            snapshot_id: "snap1".into(),
            total_tracks: 2,
        }];
        let tracks = vec![
            SpotifyApiTrack {
                id: "t1".into(),
                name: "Track 1".into(),
                duration_ms: 200000,
                artists: vec!["Artist A".into()],
                album_name: "Album X".into(),
                isrc: None,
                upc: None,
            },
            SpotifyApiTrack {
                id: "t2".into(),
                name: "Track 2".into(),
                duration_ms: 300000,
                artists: vec!["Artist B".into()],
                album_name: "Album Y".into(),
                isrc: Some("USRC1234".into()),
                upc: None,
            },
        ];

        let client = make_mock_client(playlists, tracks);
        let service = SpotifySyncService::new(db.clone(), client);

        service.sync_account_playlists(account.id).await.unwrap();

        // Verify playlists were saved
        let query_service = SpotifySyncQueryService::new(db.clone());
        let saved_playlists = query_service.list_playlists(account.id).await.unwrap();
        assert_eq!(saved_playlists.len(), 1);
        assert_eq!(saved_playlists[0].name, "My Playlist");

        // Verify tracks were saved
        let saved_tracks = entities::spotify_track::Entity::find()
            .all(&db.conn)
            .await
            .unwrap();
        assert_eq!(saved_tracks.len(), 2);
    }

    #[tokio::test]
    async fn test_sync_account_playlists_upsert() {
        let db = test_db().await;
        let account = insert_account(&db).await;

        let playlists = vec![SpotifyApiPlaylist {
            id: "pl1".into(),
            name: "My Playlist".into(),
            description: None,
            snapshot_id: "snap1".into(),
            total_tracks: 1,
        }];
        let tracks = vec![SpotifyApiTrack {
            id: "t1".into(),
            name: "Track 1".into(),
            duration_ms: 200000,
            artists: vec!["Artist A".into()],
            album_name: "Album X".into(),
            isrc: None,
            upc: None,
        }];

        // Sync twice
        let client1 = make_mock_client(playlists.clone(), tracks.clone());
        let service1 = SpotifySyncService::new(db.clone(), client1);
        service1.sync_account_playlists(account.id).await.unwrap();

        let client2 = make_mock_client(playlists, tracks);
        let service2 = SpotifySyncService::new(db.clone(), client2);
        service2.sync_account_playlists(account.id).await.unwrap();

        // Verify no duplicates
        let query_service = SpotifySyncQueryService::new(db.clone());
        let saved_playlists = query_service.list_playlists(account.id).await.unwrap();
        assert_eq!(saved_playlists.len(), 1);

        let saved_tracks = entities::spotify_track::Entity::find()
            .all(&db.conn)
            .await
            .unwrap();
        assert_eq!(saved_tracks.len(), 1);
    }
}
