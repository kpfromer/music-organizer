use std::sync::Arc;

use sea_orm::{
    ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};

use crate::database::{self, Database};
use crate::entities;
use crate::http_server::graphql::query_builder::{
    SortInput, SortableField, TrackSortField, apply_multi_column_text_search, apply_pagination,
    apply_sort,
};

#[derive(Debug)]
pub struct TrackWithRelations {
    pub track: entities::track::Model,
    pub album: entities::album::Model,
    /// Vec of (artist, is_primary)
    pub artists: Vec<(database::Artist, bool)>,
}

pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total_count: u64,
    pub page: usize,
    pub page_size: usize,
}

pub struct TrackService {
    db: Arc<Database>,
}

impl TrackService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub async fn list_unimportable_files(
        &self,
        page: Option<i32>,
        page_size: Option<i32>,
    ) -> color_eyre::Result<PaginatedResult<entities::unimportable_file::Model>> {
        let page = page.unwrap_or(1).max(1) as usize;
        let page_size = page_size.unwrap_or(25).clamp(1, 100) as usize;

        let total_count = entities::unimportable_file::Entity::find()
            .count(&self.db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to count unimportable files: {}", e))?;

        let offset = (page.saturating_sub(1)) * page_size;
        let items = entities::unimportable_file::Entity::find()
            .order_by_desc(entities::unimportable_file::Column::CreatedAt)
            .limit(page_size as u64)
            .offset(offset as u64)
            .all(&self.db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to fetch unimportable files: {}", e))?;

        Ok(PaginatedResult {
            items,
            total_count,
            page,
            page_size,
        })
    }

    pub async fn get_track_by_id(&self, track_id: i64) -> color_eyre::Result<TrackWithRelations> {
        let (track, album) = entities::track::Entity::find_by_id(track_id)
            .find_also_related(entities::album::Entity)
            .one(&self.db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to fetch track: {}", e))?
            .ok_or_else(|| color_eyre::eyre::eyre!("Track not found"))?;

        let album = album
            .ok_or_else(|| color_eyre::eyre::eyre!("Track {} has no associated album", track_id))?;

        let artists = self
            .db
            .get_track_artists(track_id)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to fetch track artists: {}", e))?;

        Ok(TrackWithRelations {
            track,
            album,
            artists,
        })
    }

    pub async fn list_tracks(
        &self,
        search: Option<&str>,
        sort_inputs: &[SortInput<TrackSortField>],
        page: Option<i32>,
        page_size: Option<i32>,
    ) -> color_eyre::Result<PaginatedResult<TrackWithRelations>> {
        let mut query = entities::track::Entity::find();

        if let Some(search_term) = search
            && !search_term.is_empty()
        {
            query = apply_multi_column_text_search(
                query,
                vec![entities::track::Column::Title],
                search_term,
            );
        }

        let total_count = query
            .clone()
            .count(&self.db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to count tracks: {}", e))?;

        query = apply_sort(query, sort_inputs, Some(TrackSortField::default_sort()))
            .map_err(|e| color_eyre::eyre::eyre!("Failed to apply sorting: {}", e))?;

        let (query, page, page_size) = apply_pagination(query, page, page_size);

        let track_album_pairs = query
            .find_also_related(entities::album::Entity)
            .all(&self.db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to fetch tracks: {}", e))?;

        let items = self.hydrate_tracks(track_album_pairs).await?;

        Ok(PaginatedResult {
            items,
            total_count,
            page,
            page_size,
        })
    }

    pub async fn list_playlist_tracks(
        &self,
        playlist_id: i64,
        page: Option<i32>,
        page_size: Option<i32>,
    ) -> color_eyre::Result<PaginatedResult<TrackWithRelations>> {
        // Verify playlist exists
        let playlist = entities::playlist::Entity::find_by_id(playlist_id)
            .one(&self.db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to find playlist: {}", e))?;

        if playlist.is_none() {
            return Err(color_eyre::eyre::eyre!("Playlist not found"));
        }

        let page_val = page.unwrap_or(1).max(1) as usize;
        let page_size_val = page_size.unwrap_or(25).clamp(1, 100) as usize;

        let playlist_track_models = entities::playlist_track::Entity::find()
            .filter(entities::playlist_track::Column::PlaylistId.eq(playlist_id))
            .all(&self.db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to fetch playlist tracks: {}", e))?;

        let track_ids: Vec<i64> = playlist_track_models.iter().map(|pt| pt.track_id).collect();
        let total_count = track_ids.len() as u64;

        let offset = (page_val.saturating_sub(1)) * page_size_val;
        let paginated_track_ids: Vec<i64> = track_ids
            .into_iter()
            .skip(offset)
            .take(page_size_val)
            .collect();

        if paginated_track_ids.is_empty() {
            return Ok(PaginatedResult {
                items: Vec::new(),
                total_count,
                page: page_val,
                page_size: page_size_val,
            });
        }

        let track_album_pairs = entities::track::Entity::find()
            .filter(entities::track::Column::Id.is_in(paginated_track_ids))
            .find_also_related(entities::album::Entity)
            .all(&self.db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to fetch tracks: {}", e))?;

        let items = self.hydrate_tracks(track_album_pairs).await?;

        Ok(PaginatedResult {
            items,
            total_count,
            page: page_val,
            page_size: page_size_val,
        })
    }

    pub async fn list_playlists(
        &self,
        search: Option<&str>,
        sort_by: Option<&str>,
        sort_order: Option<&str>,
        page: Option<i32>,
        page_size: Option<i32>,
    ) -> color_eyre::Result<PaginatedResult<(entities::playlist::Model, u64)>> {
        let page_val = page.unwrap_or(1).max(1) as usize;
        let page_size_val = page_size.unwrap_or(25).clamp(1, 100) as usize;

        let mut query = entities::playlist::Entity::find();
        if let Some(search_term) = search
            && !search_term.is_empty()
        {
            let condition = Condition::any()
                .add(entities::playlist::Column::Name.contains(search_term))
                .add(entities::playlist::Column::Description.contains(search_term));
            query = query.filter(condition);
        }

        let total_count = query
            .clone()
            .count(&self.db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to count playlists: {}", e))?;

        let sort_by = sort_by.unwrap_or("created_at");
        let sort_order = sort_order.unwrap_or("desc");
        let is_desc = sort_order == "desc";

        query = match sort_by {
            "name" => {
                if is_desc {
                    query.order_by_desc(entities::playlist::Column::Name)
                } else {
                    query.order_by_asc(entities::playlist::Column::Name)
                }
            }
            "updated_at" => {
                if is_desc {
                    query.order_by_desc(entities::playlist::Column::UpdatedAt)
                } else {
                    query.order_by_asc(entities::playlist::Column::UpdatedAt)
                }
            }
            _ => {
                if is_desc {
                    query.order_by_desc(entities::playlist::Column::CreatedAt)
                } else {
                    query.order_by_asc(entities::playlist::Column::CreatedAt)
                }
            }
        };

        let offset = (page_val.saturating_sub(1)) * page_size_val;
        let playlist_models = query
            .limit(page_size_val as u64)
            .offset(offset as u64)
            .all(&self.db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to fetch playlists: {}", e))?;

        let mut items = Vec::new();
        for playlist_model in playlist_models {
            let track_count = entities::playlist_track::Entity::find()
                .filter(entities::playlist_track::Column::PlaylistId.eq(playlist_model.id))
                .count(&self.db.conn)
                .await
                .map_err(|e| {
                    color_eyre::eyre::eyre!("Failed to count tracks for playlist: {}", e)
                })?;
            items.push((playlist_model, track_count));
        }

        Ok(PaginatedResult {
            items,
            total_count,
            page: page_val,
            page_size: page_size_val,
        })
    }

    pub async fn get_playlist(
        &self,
        id: i64,
    ) -> color_eyre::Result<Option<(entities::playlist::Model, u64)>> {
        let playlist_model = entities::playlist::Entity::find_by_id(id)
            .one(&self.db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to find playlist: {}", e))?;

        if let Some(playlist_model) = playlist_model {
            let track_count = entities::playlist_track::Entity::find()
                .filter(entities::playlist_track::Column::PlaylistId.eq(playlist_model.id))
                .count(&self.db.conn)
                .await
                .map_err(|e| {
                    color_eyre::eyre::eyre!("Failed to count tracks for playlist: {}", e)
                })?;
            Ok(Some((playlist_model, track_count)))
        } else {
            Ok(None)
        }
    }

    async fn hydrate_tracks(
        &self,
        track_album_pairs: Vec<(entities::track::Model, Option<entities::album::Model>)>,
    ) -> color_eyre::Result<Vec<TrackWithRelations>> {
        let mut result = Vec::new();

        for (track, album) in track_album_pairs {
            let album = album.ok_or_else(|| {
                color_eyre::eyre::eyre!("Track {} has no associated album", track.id)
            })?;

            let artists = self
                .db
                .get_track_artists(track.id)
                .await
                .map_err(|e| color_eyre::eyre::eyre!("Failed to fetch track artists: {}", e))?;

            result.push(TrackWithRelations {
                track,
                album,
                artists,
            });
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::test_db;
    use sea_orm::{ActiveModelTrait, Set};

    async fn insert_album(db: &Database) -> i64 {
        let now = chrono::Utc::now().timestamp();
        let album = entities::album::ActiveModel {
            title: Set("Test Album".into()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };
        let result = album.insert(&db.conn).await.unwrap();
        result.id
    }

    async fn insert_artist(db: &Database, name: &str) -> i64 {
        let now = chrono::Utc::now().timestamp();
        let artist = entities::artist::ActiveModel {
            name: Set(name.into()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };
        let result = artist.insert(&db.conn).await.unwrap();
        result.id
    }

    async fn insert_track(db: &Database, album_id: i64, title: &str, file_path: &str) -> i64 {
        let now = chrono::Utc::now().timestamp();
        let track = entities::track::ActiveModel {
            album_id: Set(album_id),
            title: Set(title.into()),
            file_path: Set(file_path.into()),
            sha256: Set(format!("sha256_{}", file_path)),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };
        let result = track.insert(&db.conn).await.unwrap();
        result.id
    }

    async fn insert_track_artist(db: &Database, track_id: i64, artist_id: i64) {
        let ta = entities::track_artist::ActiveModel {
            track_id: Set(track_id),
            artist_id: Set(artist_id),
            is_primary: Set(1),
        };
        entities::track_artist::Entity::insert(ta)
            .exec(&db.conn)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_list_unimportable_files_empty() {
        let db = test_db().await;
        let service = TrackService::new(db);

        let result = service.list_unimportable_files(None, None).await.unwrap();
        assert_eq!(result.items.len(), 0);
        assert_eq!(result.total_count, 0);
    }

    #[tokio::test]
    async fn test_list_unimportable_files_pagination() {
        let db = test_db().await;

        // Insert 3 unimportable files
        for i in 0..3 {
            let now = chrono::Utc::now().timestamp();
            let file = entities::unimportable_file::ActiveModel {
                file_path: Set(format!("/path/file{}.mp3", i)),
                sha256: Set(format!("hash{}", i)),
                reason: Set(entities::unimportable_file::UnimportableReason::UnsupportedFileType),
                created_at: Set(now + i as i64),
                ..Default::default()
            };
            file.insert(&db.conn).await.unwrap();
        }

        let service = TrackService::new(db);
        let result = service
            .list_unimportable_files(Some(1), Some(2))
            .await
            .unwrap();

        assert_eq!(result.items.len(), 2);
        assert_eq!(result.total_count, 3);
        assert_eq!(result.page, 1);
        assert_eq!(result.page_size, 2);
    }

    #[tokio::test]
    async fn test_get_track_by_id() {
        let db = test_db().await;
        let album_id = insert_album(&db).await;
        let artist_id = insert_artist(&db, "Test Artist").await;
        let track_id = insert_track(&db, album_id, "My Song", "/music/song.flac").await;
        insert_track_artist(&db, track_id, artist_id).await;

        let service = TrackService::new(db);
        let result = service.get_track_by_id(track_id).await.unwrap();

        assert_eq!(result.track.title, "My Song");
        assert_eq!(result.album.title, "Test Album");
        assert_eq!(result.artists.len(), 1);
        assert_eq!(result.artists[0].0.name, "Test Artist");
    }

    #[tokio::test]
    async fn test_get_track_by_id_not_found() {
        let db = test_db().await;
        let service = TrackService::new(db);

        let result = service.get_track_by_id(9999).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Track not found"));
    }

    #[tokio::test]
    async fn test_list_tracks_search() {
        let db = test_db().await;
        let album_id = insert_album(&db).await;
        let artist_id = insert_artist(&db, "Artist").await;

        let t1 = insert_track(&db, album_id, "Bohemian Rhapsody", "/music/bohemian.flac").await;
        let t2 = insert_track(&db, album_id, "Stairway to Heaven", "/music/stairway.flac").await;
        let t3 = insert_track(&db, album_id, "Bohemian Grove", "/music/grove.flac").await;
        for tid in [t1, t2, t3] {
            insert_track_artist(&db, tid, artist_id).await;
        }

        let service = TrackService::new(db);
        let result = service
            .list_tracks(Some("Bohemian"), &[], None, None)
            .await
            .unwrap();

        assert_eq!(result.total_count, 2);
        assert!(
            result
                .items
                .iter()
                .all(|t| t.track.title.contains("Bohemian"))
        );
    }
}
