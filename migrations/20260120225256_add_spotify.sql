-- Add column "isrcs" to table: "tracks"
ALTER TABLE `tracks` ADD COLUMN `isrcs` varchar NULL;
-- Add column "barcode" to table: "tracks"
ALTER TABLE `tracks` ADD COLUMN `barcode` varchar NULL;
-- Create index "idx_tracks_isrcs" to table: "tracks"
CREATE INDEX `idx_tracks_isrcs` ON `tracks` (`isrcs`);
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
  CONSTRAINT `1` FOREIGN KEY (`spotify_playlist_id`) REFERENCES `spotify_playlist` (`id`) ON UPDATE CASCADE ON DELETE CASCADE,
  CONSTRAINT `0` FOREIGN KEY (`local_playlist_id`) REFERENCES `playlists` (`id`) ON UPDATE CASCADE ON DELETE SET NULL
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
  CONSTRAINT `1` FOREIGN KEY (`spotify_track_id`) REFERENCES `spotify_track` (`spotify_track_id`) ON UPDATE CASCADE ON DELETE CASCADE,
  CONSTRAINT `0` FOREIGN KEY (`spotify_playlist_id`) REFERENCES `spotify_playlist` (`id`) ON UPDATE CASCADE ON DELETE CASCADE
);
