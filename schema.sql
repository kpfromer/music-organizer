-- Create "seaql_migrations" table
CREATE TABLE `seaql_migrations` (
  `version` varchar NOT NULL,
  `applied_at` integer NOT NULL,
  PRIMARY KEY (`version`)
);
-- Create "artist" table
CREATE TABLE `artist` (
  `id` integer NOT NULL PRIMARY KEY AUTOINCREMENT,
  `name` varchar NOT NULL,
  `musicbrainz_id` varchar NULL,
  `created_at` integer NOT NULL DEFAULT (strftime('%s', 'now')),
  `updated_at` integer NOT NULL DEFAULT (strftime('%s', 'now'))
);
-- Create index "artist_musicbrainz_id" to table: "artist"
CREATE UNIQUE INDEX `artist_musicbrainz_id` ON `artist` (`musicbrainz_id`);
-- Create index "idx_artists_musicbrainz_id" to table: "artist"
CREATE INDEX `idx_artists_musicbrainz_id` ON `artist` (`musicbrainz_id`);
-- Create "album" table
CREATE TABLE `album` (
  `id` integer NOT NULL PRIMARY KEY AUTOINCREMENT,
  `title` varchar NOT NULL,
  `musicbrainz_id` varchar NULL,
  `year` integer NULL,
  `created_at` integer NOT NULL DEFAULT (strftime('%s', 'now')),
  `updated_at` integer NOT NULL DEFAULT (strftime('%s', 'now'))
);
-- Create index "album_musicbrainz_id" to table: "album"
CREATE UNIQUE INDEX `album_musicbrainz_id` ON `album` (`musicbrainz_id`);
-- Create index "idx_albums_musicbrainz_id" to table: "album"
CREATE INDEX `idx_albums_musicbrainz_id` ON `album` (`musicbrainz_id`);
-- Create "album_artist" table
CREATE TABLE `album_artist` (
  `album_id` integer NOT NULL,
  `artist_id` integer NOT NULL,
  `is_primary` integer NOT NULL DEFAULT 0,
  PRIMARY KEY (`album_id`, `artist_id`),
  CONSTRAINT `0` FOREIGN KEY (`artist_id`) REFERENCES `artist` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT `1` FOREIGN KEY (`album_id`) REFERENCES `album` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create index "idx_album_artists_album_id" to table: "album_artist"
CREATE INDEX `idx_album_artists_album_id` ON `album_artist` (`album_id`);
-- Create index "idx_album_artists_artist_id" to table: "album_artist"
CREATE INDEX `idx_album_artists_artist_id` ON `album_artist` (`artist_id`);
-- Create "unimportable_files" table
CREATE TABLE `unimportable_files` (
  `id` integer NOT NULL PRIMARY KEY AUTOINCREMENT,
  `file_path` varchar NOT NULL,
  `sha256` varchar NOT NULL,
  `reason` varchar NOT NULL,
  `created_at` integer NOT NULL
);
-- Create "playlists" table
CREATE TABLE `playlists` (
  `id` integer NOT NULL PRIMARY KEY AUTOINCREMENT,
  `name` varchar NOT NULL,
  `description` varchar NULL,
  `created_at` timestamp_text NOT NULL,
  `updated_at` timestamp_text NOT NULL
);
-- Create "playlist_tracks" table
CREATE TABLE `playlist_tracks` (
  `playlist_id` integer NOT NULL,
  `track_id` integer NOT NULL,
  `created_at` timestamp_text NOT NULL,
  `updated_at` timestamp_text NOT NULL,
  PRIMARY KEY (`playlist_id`, `track_id`),
  CONSTRAINT `0` FOREIGN KEY (`track_id`) REFERENCES `tracks` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT `1` FOREIGN KEY (`playlist_id`) REFERENCES `playlists` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create "plex_servers" table
CREATE TABLE `plex_servers` (
  `id` integer NOT NULL PRIMARY KEY AUTOINCREMENT,
  `name` varchar NOT NULL,
  `server_url` varchar NOT NULL,
  `access_token` varchar NULL,
  `created_at` timestamp_text NOT NULL,
  `updated_at` timestamp_text NOT NULL
);
-- Create index "plex_servers_name" to table: "plex_servers"
CREATE UNIQUE INDEX `plex_servers_name` ON `plex_servers` (`name`);
-- Create index "plex_servers_server_url" to table: "plex_servers"
CREATE UNIQUE INDEX `plex_servers_server_url` ON `plex_servers` (`server_url`);
-- Create "tracks" table
CREATE TABLE `tracks` (
  `id` integer NOT NULL PRIMARY KEY AUTOINCREMENT,
  `album_id` integer NOT NULL,
  `title` varchar NOT NULL,
  `track_number` integer NULL,
  `duration` integer NULL,
  `musicbrainz_id` varchar NULL,
  `file_path` varchar NOT NULL,
  `sha256` varchar NOT NULL,
  `created_at` integer NOT NULL DEFAULT (strftime('%s', 'now')),
  `updated_at` integer NOT NULL DEFAULT (strftime('%s', 'now')),
  `isrcs` varchar NULL,
  `barcode` varchar NULL,
  CONSTRAINT `0` FOREIGN KEY (`album_id`) REFERENCES `album` (`id`) ON UPDATE NO ACTION ON DELETE NO ACTION
);
-- Create index "tracks_musicbrainz_id" to table: "tracks"
CREATE UNIQUE INDEX `tracks_musicbrainz_id` ON `tracks` (`musicbrainz_id`);
-- Create index "tracks_file_path" to table: "tracks"
CREATE UNIQUE INDEX `tracks_file_path` ON `tracks` (`file_path`);
-- Create index "tracks_sha256" to table: "tracks"
CREATE UNIQUE INDEX `tracks_sha256` ON `tracks` (`sha256`);
-- Create index "idx_tracks_isrcs" to table: "tracks"
CREATE INDEX `idx_tracks_isrcs` ON `tracks` (`isrcs`);
-- Create "track_artist" table
CREATE TABLE `track_artist` (
  `track_id` integer NOT NULL,
  `artist_id` integer NOT NULL,
  `is_primary` integer NOT NULL DEFAULT 0,
  PRIMARY KEY (`track_id`, `artist_id`),
  CONSTRAINT `0` FOREIGN KEY (`artist_id`) REFERENCES `artist` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT `1` FOREIGN KEY (`track_id`) REFERENCES `tracks` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create "spotify_account" table
CREATE TABLE `spotify_account` (
  `id` integer NOT NULL PRIMARY KEY AUTOINCREMENT,
  `user_id` varchar NOT NULL,
  `display_name` varchar NULL,
  `access_token` varchar NOT NULL,
  `refresh_token` varchar NOT NULL,
  `token_expiry` integer NOT NULL,
  `created_at` integer NOT NULL,
  `updated_at` integer NOT NULL
);
-- Create index "spotify_account_user_id" to table: "spotify_account"
CREATE UNIQUE INDEX `spotify_account_user_id` ON `spotify_account` (`user_id`);
-- Create "spotify_playlist" table
CREATE TABLE `spotify_playlist` (
  `id` integer NOT NULL PRIMARY KEY AUTOINCREMENT,
  `account_id` integer NOT NULL,
  `spotify_id` varchar NOT NULL,
  `name` varchar NOT NULL,
  `description` varchar NULL,
  `snapshot_id` varchar NOT NULL,
  `track_count` integer NOT NULL,
  `created_at` integer NOT NULL,
  `updated_at` integer NOT NULL,
  CONSTRAINT `0` FOREIGN KEY (`account_id`) REFERENCES `spotify_account` (`id`) ON UPDATE CASCADE ON DELETE CASCADE
);
-- Create index "spotify_playlist_spotify_id" to table: "spotify_playlist"
CREATE UNIQUE INDEX `spotify_playlist_spotify_id` ON `spotify_playlist` (`spotify_id`);
-- Create index "idx_spotify_playlists_account_id" to table: "spotify_playlist"
CREATE INDEX `idx_spotify_playlists_account_id` ON `spotify_playlist` (`account_id`);
-- Create "spotify_playlist_sync_state" table
CREATE TABLE `spotify_playlist_sync_state` (
  `id` integer NOT NULL PRIMARY KEY AUTOINCREMENT,
  `spotify_playlist_id` integer NOT NULL,
  `local_playlist_id` integer NULL,
  `last_sync_at` integer NULL,
  `sync_status` varchar NOT NULL DEFAULT 'pending',
  `tracks_downloaded` integer NOT NULL DEFAULT 0,
  `tracks_failed` integer NOT NULL DEFAULT 0,
  `error_log` text NULL,
  `created_at` integer NOT NULL,
  `updated_at` integer NOT NULL,
  CONSTRAINT `0` FOREIGN KEY (`local_playlist_id`) REFERENCES `playlists` (`id`) ON UPDATE CASCADE ON DELETE SET NULL,
  CONSTRAINT `1` FOREIGN KEY (`spotify_playlist_id`) REFERENCES `spotify_playlist` (`id`) ON UPDATE CASCADE ON DELETE CASCADE
);
-- Create index "idx_spotify_playlist_sync_state_spotify_playlist_id" to table: "spotify_playlist_sync_state"
CREATE INDEX `idx_spotify_playlist_sync_state_spotify_playlist_id` ON `spotify_playlist_sync_state` (`spotify_playlist_id`);
-- Create "spotify_track_download_failure" table
CREATE TABLE `spotify_track_download_failure` (
  `id` integer NOT NULL PRIMARY KEY AUTOINCREMENT,
  `spotify_playlist_id` integer NOT NULL,
  `spotify_track_id` varchar NOT NULL,
  `track_name` varchar NOT NULL,
  `artist_name` varchar NOT NULL,
  `album_name` varchar NULL,
  `isrc` varchar NULL,
  `reason` text NOT NULL,
  `attempts_count` integer NOT NULL DEFAULT 1,
  `created_at` integer NOT NULL,
  `updated_at` integer NOT NULL,
  CONSTRAINT `0` FOREIGN KEY (`spotify_playlist_id`) REFERENCES `spotify_playlist` (`id`) ON UPDATE CASCADE ON DELETE CASCADE
);
-- Create "spotify_track" table
CREATE TABLE `spotify_track` (
  `spotify_track_id` varchar NOT NULL,
  `title` varchar NOT NULL,
  `duration` integer NULL,
  `artists` text NOT NULL,
  `album` varchar NOT NULL,
  `isrc` varchar NULL,
  `barcode` varchar NULL,
  `created_at` integer NOT NULL,
  `updated_at` integer NOT NULL,
  `local_track_id` integer NULL,
  PRIMARY KEY (`spotify_track_id`),
  CONSTRAINT `0` FOREIGN KEY (`local_track_id`) REFERENCES `tracks` (`id`) ON UPDATE CASCADE ON DELETE SET NULL
);
-- Create "spotify_track_playlist" table
CREATE TABLE `spotify_track_playlist` (
  `spotify_track_id` varchar NOT NULL,
  `spotify_playlist_id` integer NOT NULL,
  PRIMARY KEY (`spotify_track_id`, `spotify_playlist_id`),
  CONSTRAINT `0` FOREIGN KEY (`spotify_playlist_id`) REFERENCES `spotify_playlist` (`id`) ON UPDATE CASCADE ON DELETE CASCADE,
  CONSTRAINT `1` FOREIGN KEY (`spotify_track_id`) REFERENCES `spotify_track` (`spotify_track_id`) ON UPDATE CASCADE ON DELETE CASCADE
);
-- Create "spotify_to_local_matcher_tasks" table
CREATE TABLE `spotify_to_local_matcher_tasks` (
  `id` integer NOT NULL PRIMARY KEY AUTOINCREMENT,
  `status` varchar NOT NULL,
  `matched_tracks` integer NOT NULL,
  `failed_tracks` integer NOT NULL,
  `total_tracks` integer NOT NULL,
  `error_message` text NULL,
  `created_at` integer NOT NULL,
  `updated_at` integer NOT NULL
);

-- Create "spotify_match_candidate" table
CREATE TABLE `spotify_match_candidate` (
  `id` integer NOT NULL PRIMARY KEY AUTOINCREMENT,
  `spotify_track_id` varchar NOT NULL,
  `local_track_id` integer NOT NULL,
  `score` real NOT NULL,
  `confidence` varchar NOT NULL,
  `title_similarity` real NOT NULL,
  `artist_similarity` real NOT NULL,
  `album_similarity` real NOT NULL,
  `duration_match` varchar NOT NULL,
  `version_match` varchar NOT NULL,
  `status` varchar NOT NULL DEFAULT 'pending',
  `created_at` integer NOT NULL,
  `updated_at` integer NOT NULL,
  FOREIGN KEY (`spotify_track_id`) REFERENCES `spotify_track` (`spotify_track_id`) ON UPDATE CASCADE ON DELETE CASCADE,
  FOREIGN KEY (`local_track_id`) REFERENCES `tracks` (`id`) ON UPDATE CASCADE ON DELETE CASCADE
);
CREATE INDEX `idx_spotify_match_candidate_spotify_track` ON `spotify_match_candidate` (`spotify_track_id`);
CREATE INDEX `idx_spotify_match_candidate_status` ON `spotify_match_candidate` (`status`);
CREATE UNIQUE INDEX `idx_spotify_match_candidate_unique` ON `spotify_match_candidate` (`spotify_track_id`, `local_track_id`);

-- Create "youtube_video" table
CREATE TABLE `youtube_video` (
  `id` integer NOT NULL PRIMARY KEY AUTOINCREMENT,
  `youtube_id` varchar NOT NULL UNIQUE,
  `title` varchar NOT NULL,
  `channel_name` varchar NOT NULL,
  `published_at` integer NOT NULL,
  `thumbnail_url` varchar NOT NULL,
  `video_url` varchar NOT NULL,
  `created_at` integer NOT NULL,
  `updated_at` integer NOT NULL,
  `watched` integer NOT NULL
);
CREATE TABLE `youtube_subscription` (
  `id` integer NOT NULL PRIMARY KEY AUTOINCREMENT,
  `name` varchar NOT NULL,
  `youtube_id` varchar NOT NULL UNIQUE,
  `created_at` integer NOT NULL,
  `updated_at` integer NOT NULL
);