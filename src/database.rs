// TODO: Remove this once we have a proper API
#![allow(dead_code)]

use anyhow::{Context, Result};
use rusqlite::{Connection, Error as RusqliteError, params};
use rusqlite_migration::{M, Migrations};
use std::path::Path;

// Define migrations
const MIGRATIONS_SLICE: &[M<'static>] = &[
    M::up(
        r#"
        CREATE TABLE artists (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            musicbrainz_id TEXT UNIQUE,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        )
        "#,
    ),
    M::up(
        r#"
        CREATE TABLE albums (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            musicbrainz_id TEXT UNIQUE,
            year INTEGER,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        )
        "#,
    ),
    M::up(
        r#"
        CREATE TABLE tracks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            album_id INTEGER NOT NULL,
            title TEXT NOT NULL,
            track_number INTEGER,
            duration INTEGER,
            musicbrainz_id TEXT UNIQUE,
            file_path TEXT NOT NULL UNIQUE,
            sha256 TEXT NOT NULL UNIQUE,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            FOREIGN KEY (album_id) REFERENCES albums(id)
        )
        "#,
    ),
    M::up(
        r#"
        CREATE TABLE album_artists (
            album_id INTEGER NOT NULL,
            artist_id INTEGER NOT NULL,
            is_primary INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (album_id, artist_id),
            FOREIGN KEY (album_id) REFERENCES albums(id) ON DELETE CASCADE,
            FOREIGN KEY (artist_id) REFERENCES artists(id) ON DELETE CASCADE
        )
        "#,
    ),
    M::up(
        r#"
        CREATE TABLE track_artists (
            track_id INTEGER NOT NULL,
            artist_id INTEGER NOT NULL,
            is_primary INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (track_id, artist_id),
            FOREIGN KEY (track_id) REFERENCES tracks(id) ON DELETE CASCADE,
            FOREIGN KEY (artist_id) REFERENCES artists(id) ON DELETE CASCADE
        )
        "#,
    ),
    // Create indexes for performance
    M::up("CREATE INDEX IF NOT EXISTS idx_artists_musicbrainz_id ON artists(musicbrainz_id)"),
    M::up("CREATE INDEX IF NOT EXISTS idx_albums_musicbrainz_id ON albums(musicbrainz_id)"),
    M::up("CREATE INDEX IF NOT EXISTS idx_tracks_musicbrainz_id ON tracks(musicbrainz_id)"),
    M::up("CREATE INDEX IF NOT EXISTS idx_tracks_sha256 ON tracks(sha256)"),
    M::up("CREATE INDEX IF NOT EXISTS idx_tracks_file_path ON tracks(file_path)"),
    M::up("CREATE INDEX IF NOT EXISTS idx_tracks_album_id ON tracks(album_id)"),
    M::up("CREATE INDEX IF NOT EXISTS idx_album_artists_album_id ON album_artists(album_id)"),
    M::up("CREATE INDEX IF NOT EXISTS idx_album_artists_artist_id ON album_artists(artist_id)"),
    M::up("CREATE INDEX IF NOT EXISTS idx_track_artists_track_id ON track_artists(track_id)"),
    M::up("CREATE INDEX IF NOT EXISTS idx_track_artists_artist_id ON track_artists(artist_id)"),
];

const MIGRATIONS: Migrations<'static> = Migrations::from_slice(MIGRATIONS_SLICE);

pub struct Database {
    conn: Connection,
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
    pub fn open(path: &Path) -> Result<Self> {
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).context(format!(
                "Failed to create database directory: {}",
                parent.display()
            ))?;
        }

        let mut conn = Connection::open(path)
            .context(format!("Failed to open database: {}", path.display()))?;

        // Enable foreign keys
        conn.execute("PRAGMA foreign_keys = ON", [])
            .context("Failed to enable foreign keys")?;

        // Run migrations
        MIGRATIONS
            .to_latest(&mut conn)
            .context("Failed to run database migrations")?;

        Ok(Database { conn })
    }

    // ========== Artist Methods ==========

    /// Create or get an artist by name and MusicBrainz ID
    pub fn upsert_artist(&self, name: &str, musicbrainz_id: Option<&str>) -> Result<i64> {
        if let Some(mbid) = musicbrainz_id {
            // Try to find by MusicBrainz ID first
            if let Ok(Some(id)) = self.get_artist_id_by_musicbrainz_id(mbid) {
                // Update name if it changed
                self.conn
                    .execute(
                        "UPDATE artists SET name = ?1, updated_at = strftime('%s', 'now') WHERE id = ?2",
                        params![name, id],
                    )
                    .context("Failed to update artist name")?;
                return Ok(id);
            }
        }

        // Try to find by name
        if let Ok(Some(id)) = self.get_artist_id_by_name(name) {
            // Update MusicBrainz ID if provided
            if let Some(mbid) = musicbrainz_id {
                self.conn
                    .execute(
                        "UPDATE artists SET musicbrainz_id = ?1, updated_at = strftime('%s', 'now') WHERE id = ?2",
                        params![mbid, id],
                    )
                    .context("Failed to update artist MusicBrainz ID")?;
            }
            return Ok(id);
        }

        // Create new artist
        self.conn
            .execute(
                "INSERT INTO artists (name, musicbrainz_id) VALUES (?1, ?2)",
                params![name, musicbrainz_id],
            )
            .context("Failed to insert artist")?;

        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_artist_id_by_name(&self, name: &str) -> Result<Option<i64>> {
        match self.conn.query_row(
            "SELECT id FROM artists WHERE name = ?1",
            params![name],
            |row| row.get(0),
        ) {
            Ok(id) => Ok(Some(id)),
            Err(RusqliteError::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e).context("Failed to query artist by name"),
        }
    }

    pub fn get_artist_id_by_musicbrainz_id(&self, musicbrainz_id: &str) -> Result<Option<i64>> {
        match self.conn.query_row(
            "SELECT id FROM artists WHERE musicbrainz_id = ?1",
            params![musicbrainz_id],
            |row| row.get(0),
        ) {
            Ok(id) => Ok(Some(id)),
            Err(RusqliteError::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e).context("Failed to query artist by MusicBrainz ID"),
        }
    }

    pub fn get_artist(&self, id: i64) -> Result<Option<Artist>> {
        match self.conn.query_row(
            "SELECT id, name, musicbrainz_id FROM artists WHERE id = ?1",
            params![id],
            |row| {
                Ok(Artist {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    musicbrainz_id: row.get(2)?,
                })
            },
        ) {
            Ok(artist) => Ok(Some(artist)),
            Err(RusqliteError::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e).context("Failed to get artist"),
        }
    }

    // ========== Album Methods ==========

    /// Create or get an album by title and MusicBrainz ID
    pub fn upsert_album(
        &self,
        title: &str,
        musicbrainz_id: Option<&str>,
        year: Option<i32>,
    ) -> Result<i64> {
        if let Some(mbid) = musicbrainz_id {
            // Try to find by MusicBrainz ID first
            if let Ok(Some(id)) = self.get_album_id_by_musicbrainz_id(mbid) {
                // Update title and year if changed
                self.conn
                    .execute(
                        "UPDATE albums SET title = ?1, year = ?2, updated_at = strftime('%s', 'now') WHERE id = ?3",
                        params![title, year, id],
                    )
                    .context("Failed to update album")?;
                return Ok(id);
            }
        }

        // Try to find by title
        if let Ok(Some(id)) = self.get_album_id_by_title(title) {
            // Update MusicBrainz ID and year if provided
            if musicbrainz_id.is_some() || year.is_some() {
                self.conn
                    .execute(
                        "UPDATE albums SET musicbrainz_id = COALESCE(?1, musicbrainz_id), year = COALESCE(?2, year), updated_at = strftime('%s', 'now') WHERE id = ?3",
                        params![musicbrainz_id, year, id],
                    )
                    .context("Failed to update album")?;
            }
            return Ok(id);
        }

        // Create new album
        self.conn
            .execute(
                "INSERT INTO albums (title, musicbrainz_id, year) VALUES (?1, ?2, ?3)",
                params![title, musicbrainz_id, year],
            )
            .context("Failed to insert album")?;

        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_album_id_by_title(&self, title: &str) -> Result<Option<i64>> {
        match self.conn.query_row(
            "SELECT id FROM albums WHERE title = ?1",
            params![title],
            |row| row.get(0),
        ) {
            Ok(id) => Ok(Some(id)),
            Err(RusqliteError::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e).context("Failed to query album by title"),
        }
    }

    pub fn get_album_id_by_musicbrainz_id(&self, musicbrainz_id: &str) -> Result<Option<i64>> {
        match self.conn.query_row(
            "SELECT id FROM albums WHERE musicbrainz_id = ?1",
            params![musicbrainz_id],
            |row| row.get(0),
        ) {
            Ok(id) => Ok(Some(id)),
            Err(RusqliteError::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e).context("Failed to query album by MusicBrainz ID"),
        }
    }

    pub fn get_album(&self, id: i64) -> Result<Option<Album>> {
        match self.conn.query_row(
            "SELECT id, title, musicbrainz_id, year FROM albums WHERE id = ?1",
            params![id],
            |row| {
                Ok(Album {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    musicbrainz_id: row.get(2)?,
                    year: row.get(3)?,
                })
            },
        ) {
            Ok(album) => Ok(Some(album)),
            Err(RusqliteError::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e).context("Failed to get album"),
        }
    }

    // ========== Track Methods ==========

    /// Create or update a track
    pub fn upsert_track(
        &self,
        album_id: i64,
        title: &str,
        track_number: Option<i32>,
        duration: Option<i32>,
        musicbrainz_id: Option<&str>,
        file_path: &str,
        sha256: &str,
    ) -> Result<i64> {
        // Check if track exists by SHA-256 (duplicate file)
        if let Ok(Some(existing_id)) = self.get_track_id_by_sha256(sha256) {
            // Update track metadata
            self.conn
                .execute(
                    r#"
                    UPDATE tracks 
                    SET album_id = ?1, title = ?2, track_number = ?3, duration = ?4, 
                        musicbrainz_id = COALESCE(?5, musicbrainz_id), file_path = ?6,
                        updated_at = strftime('%s', 'now')
                    WHERE id = ?7
                    "#,
                    params![
                        album_id,
                        title,
                        track_number,
                        duration,
                        musicbrainz_id,
                        file_path,
                        existing_id
                    ],
                )
                .context("Failed to update track")?;
            return Ok(existing_id);
        }

        // Check if track exists by MusicBrainz ID
        if let Some(mbid) = musicbrainz_id {
            if let Ok(Some(existing_id)) = self.get_track_id_by_musicbrainz_id(mbid) {
                // Update track metadata
                self.conn
                    .execute(
                        r#"
                        UPDATE tracks 
                        SET album_id = ?1, title = ?2, track_number = ?3, duration = ?4, 
                            file_path = ?5, sha256 = ?6, updated_at = strftime('%s', 'now')
                        WHERE id = ?7
                        "#,
                        params![
                            album_id,
                            title,
                            track_number,
                            duration,
                            file_path,
                            sha256,
                            existing_id
                        ],
                    )
                    .context("Failed to update track")?;
                return Ok(existing_id);
            }
        }

        // Create new track
        self.conn
            .execute(
                r#"
                INSERT INTO tracks (album_id, title, track_number, duration, musicbrainz_id, file_path, sha256)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                "#,
                params![album_id, title, track_number, duration, musicbrainz_id, file_path, sha256],
            )
            .context("Failed to insert track")?;

        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_track_id_by_sha256(&self, sha256: &str) -> Result<Option<i64>> {
        match self.conn.query_row(
            "SELECT id FROM tracks WHERE sha256 = ?1",
            params![sha256],
            |row| row.get(0),
        ) {
            Ok(id) => Ok(Some(id)),
            Err(RusqliteError::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e).context("Failed to query track by SHA-256"),
        }
    }

    pub fn get_track_id_by_musicbrainz_id(&self, musicbrainz_id: &str) -> Result<Option<i64>> {
        match self.conn.query_row(
            "SELECT id FROM tracks WHERE musicbrainz_id = ?1",
            params![musicbrainz_id],
            |row| row.get(0),
        ) {
            Ok(id) => Ok(Some(id)),
            Err(RusqliteError::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e).context("Failed to query track by MusicBrainz ID"),
        }
    }

    pub fn get_track_id_by_file_path(&self, file_path: &str) -> Result<Option<i64>> {
        match self.conn.query_row(
            "SELECT id FROM tracks WHERE file_path = ?1",
            params![file_path],
            |row| row.get(0),
        ) {
            Ok(id) => Ok(Some(id)),
            Err(RusqliteError::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e).context("Failed to query track by file path"),
        }
    }

    pub fn get_track(&self, id: i64) -> Result<Option<Track>> {
        match self.conn.query_row(
            "SELECT id, album_id, title, track_number, duration, musicbrainz_id, file_path, sha256 FROM tracks WHERE id = ?1",
            params![id],
            |row| {
                Ok(Track {
                    id: row.get(0)?,
                    album_id: row.get(1)?,
                    title: row.get(2)?,
                    track_number: row.get(3)?,
                    duration: row.get(4)?,
                    musicbrainz_id: row.get(5)?,
                    file_path: row.get(6)?,
                    sha256: row.get(7)?,
                })
            },
        ) {
            Ok(track) => Ok(Some(track)),
            Err(RusqliteError::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e).context("Failed to get track"),
        }
    }

    // ========== Junction Table Methods ==========

    /// Add an artist to an album
    pub fn add_album_artist(&self, album_id: i64, artist_id: i64, is_primary: bool) -> Result<()> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO album_artists (album_id, artist_id, is_primary) VALUES (?1, ?2, ?3)",
                params![album_id, artist_id, if is_primary { 1 } else { 0 }],
            )
            .context("Failed to add album artist")?;
        Ok(())
    }

    /// Add an artist to a track
    pub fn add_track_artist(&self, track_id: i64, artist_id: i64, is_primary: bool) -> Result<()> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO track_artists (track_id, artist_id, is_primary) VALUES (?1, ?2, ?3)",
                params![track_id, artist_id, if is_primary { 1 } else { 0 }],
            )
            .context("Failed to add track artist")?;
        Ok(())
    }

    /// Get all artists for an album
    pub fn get_album_artists(&self, album_id: i64) -> Result<Vec<(Artist, bool)>> {
        let mut stmt = self
            .conn
            .prepare(
                r#"
                SELECT a.id, a.name, a.musicbrainz_id, aa.is_primary
                FROM artists a
                JOIN album_artists aa ON a.id = aa.artist_id
                WHERE aa.album_id = ?1
                ORDER BY aa.is_primary DESC, a.name
                "#,
            )
            .context("Failed to prepare album artists query")?;

        let rows = stmt
            .query_map(params![album_id], |row| {
                Ok((
                    Artist {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        musicbrainz_id: row.get(2)?,
                    },
                    row.get::<_, i32>(3)? == 1,
                ))
            })
            .context("Failed to query album artists")?;

        let mut artists = Vec::new();
        for row in rows {
            artists.push(row.context("Failed to parse album artist")?);
        }
        Ok(artists)
    }

    /// Get all artists for a track
    pub fn get_track_artists(&self, track_id: i64) -> Result<Vec<(Artist, bool)>> {
        let mut stmt = self
            .conn
            .prepare(
                r#"
                SELECT a.id, a.name, a.musicbrainz_id, ta.is_primary
                FROM artists a
                JOIN track_artists ta ON a.id = ta.artist_id
                WHERE ta.track_id = ?1
                ORDER BY ta.is_primary DESC, a.name
                "#,
            )
            .context("Failed to prepare track artists query")?;

        let rows = stmt
            .query_map(params![track_id], |row| {
                Ok((
                    Artist {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        musicbrainz_id: row.get(2)?,
                    },
                    row.get::<_, i32>(3)? == 1,
                ))
            })
            .context("Failed to query track artists")?;

        let mut artists = Vec::new();
        for row in rows {
            artists.push(row.context("Failed to parse track artist")?);
        }
        Ok(artists)
    }

    // ========== Helper Methods ==========

    /// Get the primary artist for an album, or fallback to first artist, or "Unknown Artist"
    pub fn get_primary_album_artist_name(&self, album_id: i64) -> Result<String> {
        let artists = self.get_album_artists(album_id)?;

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
    pub fn get_primary_track_artist_name(&self, track_id: i64) -> Result<String> {
        let artists = self.get_track_artists(track_id)?;

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
    pub fn is_duplicate_by_sha256(&self, sha256: &str) -> Result<bool> {
        let count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM tracks WHERE sha256 = ?1",
                params![sha256],
                |row| row.get(0),
            )
            .context("Failed to check duplicate by SHA-256")?;

        Ok(count > 0)
    }

    /// Check if a track already exists by MusicBrainz ID
    pub fn is_duplicate_by_musicbrainz_id(&self, musicbrainz_id: &str) -> Result<bool> {
        let count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM tracks WHERE musicbrainz_id = ?1",
                params![musicbrainz_id],
                |row| row.get(0),
            )
            .context("Failed to check duplicate by MusicBrainz ID")?;

        Ok(count > 0)
    }

    /// Get track by SHA-256 hash (for duplicate detection)
    pub fn get_track_by_sha256(&self, sha256: &str) -> Result<Option<Track>> {
        if let Some(id) = self.get_track_id_by_sha256(sha256)? {
            self.get_track(id)
        } else {
            Ok(None)
        }
    }
}
