// TODO: Remove this once we have a proper API
#![allow(dead_code)]

use color_eyre::{Result, eyre::Context};
use migration::MigratorTrait;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectOptions, Database as SeaDatabase,
    DatabaseConnection, EntityTrait, QueryFilter,
};
use std::path::Path;
use std::time::Duration;

use crate::entities;

pub struct Database {
    conn: DatabaseConnection,
}

#[derive(Debug, Clone)]
pub struct Artist {
    pub id: i64,
    pub name: String,
    pub musicbrainz_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Album {
    pub id: i64,
    pub title: String,
    pub musicbrainz_id: Option<String>,
    pub year: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct Track {
    pub id: i64,
    pub album_id: i64,
    pub title: String,
    pub track_number: Option<i32>,
    pub duration: Option<i32>,
    pub musicbrainz_id: Option<String>,
    pub file_path: String,
    pub sha256: String,
}

impl Database {
    /// Open or create a database at the given path
    pub async fn open(path: &Path) -> Result<Self> {
        log::debug!("Opening database at: {}", path.display());

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).context(format!(
                "Failed to create database directory: {}",
                parent.display()
            ))?;
        }

        // Create SQLite connection URL
        let url = format!("sqlite://{}?mode=rwc", path.display());

        // Configure connection options
        let mut opt = ConnectOptions::new(url);
        opt.max_connections(100)
            .min_connections(5)
            .connect_timeout(Duration::from_secs(8))
            .acquire_timeout(Duration::from_secs(8))
            .idle_timeout(Duration::from_secs(8))
            .max_lifetime(Duration::from_secs(8))
            .sqlx_logging(false);

        let conn = SeaDatabase::connect(opt)
            .await
            .context(format!("Failed to open database: {}", path.display()))?;

        // Enable foreign keys via raw SQL
        sea_orm::Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            "PRAGMA foreign_keys = ON".to_owned(),
        );

        // Run migrations
        log::debug!("Running database migrations");
        migration::Migrator::up(&conn, None)
            .await
            .context("Failed to run database migrations")?;

        log::info!("Database ready at: {}", path.display());
        Ok(Database { conn })
    }

    // ========== Artist Methods ==========

    /// Create or get an artist by name and MusicBrainz ID
    pub async fn upsert_artist(&self, name: &str, musicbrainz_id: Option<&str>) -> Result<i64> {
        log::debug!(
            "Upserting artist: '{}' (MusicBrainz ID: {:?})",
            name,
            musicbrainz_id
        );

        if let Some(mbid) = musicbrainz_id {
            // Try to find by MusicBrainz ID first
            if let Ok(Some(id)) = self.get_artist_id_by_musicbrainz_id(mbid).await {
                log::debug!("Found existing artist by MusicBrainz ID (ID: {})", id);
                // Update name if it changed
                let artist = entities::artist::Entity::find_by_id(id)
                    .one(&self.conn)
                    .await
                    .context("Failed to find artist")?
                    .ok_or_else(|| color_eyre::eyre::eyre!("Artist not found"))?;

                let mut active_artist: entities::artist::ActiveModel = artist.into();
                active_artist.name = ActiveValue::Set(name.to_string());
                active_artist.updated_at = ActiveValue::Set(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64,
                );
                active_artist
                    .update(&self.conn)
                    .await
                    .context("Failed to update artist name")?;
                log::info!("Artist updated: '{}' (ID: {})", name, id);
                return Ok(id);
            }
        }

        // Try to find by name
        if let Ok(Some(id)) = self.get_artist_id_by_name(name).await {
            log::debug!("Found existing artist by name (ID: {})", id);
            // Update MusicBrainz ID if provided
            if let Some(mbid) = musicbrainz_id {
                let artist = entities::artist::Entity::find_by_id(id)
                    .one(&self.conn)
                    .await
                    .context("Failed to find artist")?
                    .ok_or_else(|| color_eyre::eyre::eyre!("Artist not found"))?;

                let mut active_artist: entities::artist::ActiveModel = artist.into();
                active_artist.musicbrainz_id = ActiveValue::Set(Some(mbid.to_string()));
                active_artist.updated_at = ActiveValue::Set(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64,
                );
                active_artist
                    .update(&self.conn)
                    .await
                    .context("Failed to update artist MusicBrainz ID")?;
            }
            log::info!("Artist updated: '{}' (ID: {})", name, id);
            return Ok(id);
        }

        // Create new artist
        log::debug!("Creating new artist: '{}'", name);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let new_artist = entities::artist::ActiveModel {
            id: ActiveValue::NotSet,
            name: ActiveValue::Set(name.to_string()),
            musicbrainz_id: ActiveValue::Set(musicbrainz_id.map(|s| s.to_string())),
            created_at: ActiveValue::Set(now),
            updated_at: ActiveValue::Set(now),
        };

        let result = new_artist
            .insert(&self.conn)
            .await
            .context("Failed to insert artist")?;

        log::info!("Artist created: '{}' (ID: {})", name, result.id);
        Ok(result.id)
    }

    pub async fn get_artist_id_by_name(&self, name: &str) -> Result<Option<i64>> {
        let artist = entities::artist::Entity::find()
            .filter(entities::artist::Column::Name.eq(name))
            .one(&self.conn)
            .await
            .context("Failed to query artist by name")?;

        Ok(artist.map(|a| a.id))
    }

    pub async fn get_artist_id_by_musicbrainz_id(
        &self,
        musicbrainz_id: &str,
    ) -> Result<Option<i64>> {
        let artist = entities::artist::Entity::find()
            .filter(entities::artist::Column::MusicbrainzId.eq(musicbrainz_id))
            .one(&self.conn)
            .await
            .context("Failed to query artist by MusicBrainz ID")?;

        Ok(artist.map(|a| a.id))
    }

    pub async fn get_artist(&self, id: i64) -> Result<Option<Artist>> {
        let artist = entities::artist::Entity::find_by_id(id)
            .one(&self.conn)
            .await
            .context("Failed to get artist")?;

        Ok(artist.map(|a| Artist {
            id: a.id,
            name: a.name,
            musicbrainz_id: a.musicbrainz_id,
        }))
    }

    // ========== Album Methods ==========

    /// Create or get an album by title and MusicBrainz ID
    pub async fn upsert_album(
        &self,
        title: &str,
        musicbrainz_id: Option<&str>,
        year: Option<i32>,
    ) -> Result<i64> {
        log::debug!(
            "Upserting album: '{}' (MusicBrainz ID: {:?}, year: {:?})",
            title,
            musicbrainz_id,
            year
        );

        if let Some(mbid) = musicbrainz_id {
            // Try to find by MusicBrainz ID first
            if let Ok(Some(id)) = self.get_album_id_by_musicbrainz_id(mbid).await {
                log::debug!("Found existing album by MusicBrainz ID (ID: {})", id);
                // Update title and year if changed
                let album = entities::album::Entity::find_by_id(id)
                    .one(&self.conn)
                    .await
                    .context("Failed to find album")?
                    .ok_or_else(|| color_eyre::eyre::eyre!("Album not found"))?;

                let mut active_album: entities::album::ActiveModel = album.into();
                active_album.title = ActiveValue::Set(title.to_string());
                active_album.year = ActiveValue::Set(year);
                active_album.updated_at = ActiveValue::Set(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64,
                );
                active_album
                    .update(&self.conn)
                    .await
                    .context("Failed to update album")?;
                log::info!("Album updated: '{}' (ID: {})", title, id);
                return Ok(id);
            }
        }

        // Try to find by title
        if let Ok(Some(id)) = self.get_album_id_by_title(title).await {
            log::debug!("Found existing album by title (ID: {})", id);
            // Update MusicBrainz ID and year if provided
            if musicbrainz_id.is_some() || year.is_some() {
                let album = entities::album::Entity::find_by_id(id)
                    .one(&self.conn)
                    .await
                    .context("Failed to find album")?
                    .ok_or_else(|| color_eyre::eyre::eyre!("Album not found"))?;

                let mut active_album: entities::album::ActiveModel = album.into();
                if let Some(mbid) = musicbrainz_id {
                    active_album.musicbrainz_id = ActiveValue::Set(Some(mbid.to_string()));
                }
                if year.is_some() {
                    active_album.year = ActiveValue::Set(year);
                }
                active_album.updated_at = ActiveValue::Set(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64,
                );
                active_album
                    .update(&self.conn)
                    .await
                    .context("Failed to update album")?;
            }
            log::info!("Album updated: '{}' (ID: {})", title, id);
            return Ok(id);
        }

        // Create new album
        log::debug!("Creating new album: '{}'", title);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let new_album = entities::album::ActiveModel {
            id: ActiveValue::NotSet,
            title: ActiveValue::Set(title.to_string()),
            musicbrainz_id: ActiveValue::Set(musicbrainz_id.map(|s| s.to_string())),
            year: ActiveValue::Set(year),
            created_at: ActiveValue::Set(now),
            updated_at: ActiveValue::Set(now),
        };

        let result = new_album
            .insert(&self.conn)
            .await
            .context("Failed to insert album")?;

        log::info!("Album created: '{}' (ID: {})", title, result.id);
        Ok(result.id)
    }

    pub async fn get_album_id_by_title(&self, title: &str) -> Result<Option<i64>> {
        let album = entities::album::Entity::find()
            .filter(entities::album::Column::Title.eq(title))
            .one(&self.conn)
            .await
            .context("Failed to query album by title")?;

        Ok(album.map(|a| a.id))
    }

    pub async fn get_album_id_by_musicbrainz_id(
        &self,
        musicbrainz_id: &str,
    ) -> Result<Option<i64>> {
        let album = entities::album::Entity::find()
            .filter(entities::album::Column::MusicbrainzId.eq(musicbrainz_id))
            .one(&self.conn)
            .await
            .context("Failed to query album by MusicBrainz ID")?;

        Ok(album.map(|a| a.id))
    }

    pub async fn get_album(&self, id: i64) -> Result<Option<Album>> {
        let album = entities::album::Entity::find_by_id(id)
            .one(&self.conn)
            .await
            .context("Failed to get album")?;

        Ok(album.map(|a| Album {
            id: a.id,
            title: a.title,
            musicbrainz_id: a.musicbrainz_id,
            year: a.year,
        }))
    }

    // ========== Track Methods ==========

    /// Create or update a track
    #[allow(clippy::too_many_arguments)]
    pub async fn upsert_track(
        &self,
        album_id: i64,
        title: &str,
        track_number: Option<i32>,
        duration: Option<i32>,
        musicbrainz_id: Option<&str>,
        file_path: &str,
        sha256: &str,
    ) -> Result<i64> {
        log::debug!(
            "Upserting track: '{}' (MusicBrainz ID: {:?})",
            title,
            musicbrainz_id
        );

        // Check if track exists by SHA-256 (duplicate file)
        if let Ok(Some(existing_id)) = self.get_track_id_by_sha256(sha256).await {
            log::debug!("Found existing track by SHA-256 (ID: {})", existing_id);
            // Update track metadata
            let track = entities::track::Entity::find_by_id(existing_id)
                .one(&self.conn)
                .await
                .context("Failed to find track")?
                .ok_or_else(|| color_eyre::eyre::eyre!("Track not found"))?;

            let mut active_track: entities::track::ActiveModel = track.into();
            active_track.album_id = ActiveValue::Set(album_id);
            active_track.title = ActiveValue::Set(title.to_string());
            active_track.track_number = ActiveValue::Set(track_number);
            active_track.duration = ActiveValue::Set(duration);
            if let Some(mbid) = musicbrainz_id {
                active_track.musicbrainz_id = ActiveValue::Set(Some(mbid.to_string()));
            }
            active_track.file_path = ActiveValue::Set(file_path.to_string());
            active_track.updated_at = ActiveValue::Set(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
            );
            active_track
                .update(&self.conn)
                .await
                .context("Failed to update track")?;
            log::info!("Track updated: '{}' (ID: {})", title, existing_id);
            return Ok(existing_id);
        }

        // Check if track exists by MusicBrainz ID
        if let Some(mbid) = musicbrainz_id
            && let Ok(Some(existing_id)) = self.get_track_id_by_musicbrainz_id(mbid).await
        {
            log::debug!(
                "Found existing track by MusicBrainz ID (ID: {})",
                existing_id
            );
            // Update track metadata
            let track = entities::track::Entity::find_by_id(existing_id)
                .one(&self.conn)
                .await
                .context("Failed to find track")?
                .ok_or_else(|| color_eyre::eyre::eyre!("Track not found"))?;

            let mut active_track: entities::track::ActiveModel = track.into();
            active_track.album_id = ActiveValue::Set(album_id);
            active_track.title = ActiveValue::Set(title.to_string());
            active_track.track_number = ActiveValue::Set(track_number);
            active_track.duration = ActiveValue::Set(duration);
            active_track.file_path = ActiveValue::Set(file_path.to_string());
            active_track.sha256 = ActiveValue::Set(sha256.to_string());
            active_track.updated_at = ActiveValue::Set(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
            );
            active_track
                .update(&self.conn)
                .await
                .context("Failed to update track")?;
            log::info!("Track updated: '{}' (ID: {})", title, existing_id);
            return Ok(existing_id);
        }

        // Create new track
        log::debug!("Creating new track: '{}'", title);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let new_track = entities::track::ActiveModel {
            id: ActiveValue::NotSet,
            album_id: ActiveValue::Set(album_id),
            title: ActiveValue::Set(title.to_string()),
            track_number: ActiveValue::Set(track_number),
            duration: ActiveValue::Set(duration),
            musicbrainz_id: ActiveValue::Set(musicbrainz_id.map(|s| s.to_string())),
            file_path: ActiveValue::Set(file_path.to_string()),
            sha256: ActiveValue::Set(sha256.to_string()),
            created_at: ActiveValue::Set(now),
            updated_at: ActiveValue::Set(now),
        };

        let result = new_track
            .insert(&self.conn)
            .await
            .context("Failed to insert track")?;

        log::info!("Track created: '{}' (ID: {})", title, result.id);
        Ok(result.id)
    }

    pub async fn get_track_id_by_sha256(&self, sha256: &str) -> Result<Option<i64>> {
        let track = entities::track::Entity::find()
            .filter(entities::track::Column::Sha256.eq(sha256))
            .one(&self.conn)
            .await
            .context("Failed to query track by SHA-256")?;

        Ok(track.map(|t| t.id))
    }

    pub async fn get_track_id_by_musicbrainz_id(
        &self,
        musicbrainz_id: &str,
    ) -> Result<Option<i64>> {
        let track = entities::track::Entity::find()
            .filter(entities::track::Column::MusicbrainzId.eq(musicbrainz_id))
            .one(&self.conn)
            .await
            .context("Failed to query track by MusicBrainz ID")?;

        Ok(track.map(|t| t.id))
    }

    pub async fn get_track_id_by_file_path(&self, file_path: &str) -> Result<Option<i64>> {
        let track = entities::track::Entity::find()
            .filter(entities::track::Column::FilePath.eq(file_path))
            .one(&self.conn)
            .await
            .context("Failed to query track by file path")?;

        Ok(track.map(|t| t.id))
    }

    pub async fn get_track(&self, id: i64) -> Result<Option<Track>> {
        let track = entities::track::Entity::find_by_id(id)
            .one(&self.conn)
            .await
            .context("Failed to get track")?;

        Ok(track.map(|t| Track {
            id: t.id,
            album_id: t.album_id,
            title: t.title,
            track_number: t.track_number,
            duration: t.duration,
            musicbrainz_id: t.musicbrainz_id,
            file_path: t.file_path,
            sha256: t.sha256,
        }))
    }

    // ========== Junction Table Methods ==========

    /// Add an artist to an album
    pub async fn add_album_artist(
        &self,
        album_id: i64,
        artist_id: i64,
        is_primary: bool,
    ) -> Result<()> {
        log::debug!(
            "Adding album artist relationship: album_id={}, artist_id={}, is_primary={}",
            album_id,
            artist_id,
            is_primary
        );

        let album_artist = entities::album_artist::ActiveModel {
            album_id: ActiveValue::Set(album_id),
            artist_id: ActiveValue::Set(artist_id),
            is_primary: ActiveValue::Set(if is_primary { 1 } else { 0 }),
        };

        entities::album_artist::Entity::insert(album_artist)
            .on_conflict(
                sea_orm::sea_query::OnConflict::columns([
                    entities::album_artist::Column::AlbumId,
                    entities::album_artist::Column::ArtistId,
                ])
                .update_column(entities::album_artist::Column::IsPrimary)
                .to_owned(),
            )
            .exec(&self.conn)
            .await
            .context("Failed to add album artist")?;

        Ok(())
    }

    /// Add an artist to a track
    pub async fn add_track_artist(
        &self,
        track_id: i64,
        artist_id: i64,
        is_primary: bool,
    ) -> Result<()> {
        log::debug!(
            "Adding track artist relationship: track_id={}, artist_id={}, is_primary={}",
            track_id,
            artist_id,
            is_primary
        );

        let track_artist = entities::track_artist::ActiveModel {
            track_id: ActiveValue::Set(track_id),
            artist_id: ActiveValue::Set(artist_id),
            is_primary: ActiveValue::Set(if is_primary { 1 } else { 0 }),
        };

        entities::track_artist::Entity::insert(track_artist)
            .on_conflict(
                sea_orm::sea_query::OnConflict::columns([
                    entities::track_artist::Column::TrackId,
                    entities::track_artist::Column::ArtistId,
                ])
                .update_column(entities::track_artist::Column::IsPrimary)
                .to_owned(),
            )
            .exec(&self.conn)
            .await
            .context("Failed to add track artist")?;

        Ok(())
    }

    /// Get all artists for an album
    pub async fn get_album_artists(&self, album_id: i64) -> Result<Vec<(Artist, bool)>> {
        // Load related artists through the junction table
        let album_artists = entities::album_artist::Entity::find()
            .filter(entities::album_artist::Column::AlbumId.eq(album_id))
            .all(&self.conn)
            .await
            .context("Failed to query album artists")?;

        let mut result = Vec::new();
        for album_artist in album_artists {
            let artist = entities::artist::Entity::find_by_id(album_artist.artist_id)
                .one(&self.conn)
                .await
                .context("Failed to get artist")?;

            if let Some(artist) = artist {
                result.push((
                    Artist {
                        id: artist.id,
                        name: artist.name,
                        musicbrainz_id: artist.musicbrainz_id,
                    },
                    album_artist.is_primary == 1,
                ));
            }
        }

        // Sort by is_primary DESC, then by name
        result.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.name.cmp(&b.0.name)));

        Ok(result)
    }

    /// Get all artists for a track
    pub async fn get_track_artists(&self, track_id: i64) -> Result<Vec<(Artist, bool)>> {
        // Load related artists through the junction table
        let track_artists = entities::track_artist::Entity::find()
            .filter(entities::track_artist::Column::TrackId.eq(track_id))
            .all(&self.conn)
            .await
            .context("Failed to query track artists")?;

        let mut result = Vec::new();
        for track_artist in track_artists {
            let artist = entities::artist::Entity::find_by_id(track_artist.artist_id)
                .one(&self.conn)
                .await
                .context("Failed to get artist")?;

            if let Some(artist) = artist {
                result.push((
                    Artist {
                        id: artist.id,
                        name: artist.name,
                        musicbrainz_id: artist.musicbrainz_id,
                    },
                    track_artist.is_primary == 1,
                ));
            }
        }

        // Sort by is_primary DESC, then by name
        result.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.name.cmp(&b.0.name)));

        Ok(result)
    }

    // ========== Helper Methods ==========

    /// Get the primary artist for an album, or fallback to first artist, or "Unknown Artist"
    pub async fn get_primary_album_artist_name(&self, album_id: i64) -> Result<String> {
        let artists = self.get_album_artists(album_id).await?;

        // First try to find primary artist
        for (artist, is_primary) in &artists {
            if *is_primary {
                return Ok(artist.name.clone());
            }
        }

        // Fallback to first artist
        if let Some((artist, _)) = artists.first() {
            return Ok(artist.name.clone());
        }

        // Final fallback
        Ok("Unknown Artist".to_string())
    }

    /// Get the primary artist for a track, or fallback to first artist, or "Unknown Artist"
    pub async fn get_primary_track_artist_name(&self, track_id: i64) -> Result<String> {
        let artists = self.get_track_artists(track_id).await?;

        // First try to find primary artist
        for (artist, is_primary) in &artists {
            if *is_primary {
                return Ok(artist.name.clone());
            }
        }

        // Fallback to first artist
        if let Some((artist, _)) = artists.first() {
            return Ok(artist.name.clone());
        }

        // Final fallback
        Ok("Unknown Artist".to_string())
    }

    // ========== Duplicate Detection ==========

    /// Check if a file is a duplicate by SHA-256 hash
    pub async fn is_duplicate_by_sha256(&self, sha256: &str) -> Result<bool> {
        log::debug!("Checking for duplicate by SHA-256: {}", sha256);
        let track = entities::track::Entity::find()
            .filter(entities::track::Column::Sha256.eq(sha256))
            .one(&self.conn)
            .await
            .context("Failed to check duplicate by SHA-256")?;

        let is_duplicate = track.is_some();
        log::debug!(
            "Duplicate check by SHA-256: {}",
            if is_duplicate { "found" } else { "not found" }
        );
        Ok(is_duplicate)
    }

    /// Check if a track already exists by MusicBrainz ID
    pub async fn is_duplicate_by_musicbrainz_id(&self, musicbrainz_id: &str) -> Result<bool> {
        log::debug!(
            "Checking for duplicate by MusicBrainz ID: {}",
            musicbrainz_id
        );
        let track = entities::track::Entity::find()
            .filter(entities::track::Column::MusicbrainzId.eq(musicbrainz_id))
            .one(&self.conn)
            .await
            .context("Failed to check duplicate by MusicBrainz ID")?;

        let is_duplicate = track.is_some();
        log::debug!(
            "Duplicate check by MusicBrainz ID: {}",
            if is_duplicate { "found" } else { "not found" }
        );
        Ok(is_duplicate)
    }

    /// Get track by SHA-256 hash (for duplicate detection)
    pub async fn get_track_by_sha256(&self, sha256: &str) -> Result<Option<Track>> {
        if let Some(id) = self.get_track_id_by_sha256(sha256).await? {
            self.get_track(id).await
        } else {
            Ok(None)
        }
    }
}
