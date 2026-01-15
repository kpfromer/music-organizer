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
  CONSTRAINT `1` FOREIGN KEY (`album_id`) REFERENCES `album` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT `0` FOREIGN KEY (`artist_id`) REFERENCES `artist` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE
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
  CONSTRAINT `1` FOREIGN KEY (`playlist_id`) REFERENCES `playlists` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT `0` FOREIGN KEY (`track_id`) REFERENCES `tracks` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE
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
  CONSTRAINT `0` FOREIGN KEY (`album_id`) REFERENCES `album` (`id`) ON UPDATE NO ACTION ON DELETE NO ACTION
);
-- Create index "tracks_musicbrainz_id" to table: "tracks"
CREATE UNIQUE INDEX `tracks_musicbrainz_id` ON `tracks` (`musicbrainz_id`);
-- Create index "tracks_file_path" to table: "tracks"
CREATE UNIQUE INDEX `tracks_file_path` ON `tracks` (`file_path`);
-- Create index "tracks_sha256" to table: "tracks"
CREATE UNIQUE INDEX `tracks_sha256` ON `tracks` (`sha256`);
-- Create "track_artist" table
CREATE TABLE `track_artist` (
  `track_id` integer NOT NULL,
  `artist_id` integer NOT NULL,
  `is_primary` integer NOT NULL DEFAULT 0,
  PRIMARY KEY (`track_id`, `artist_id`),
  CONSTRAINT `1` FOREIGN KEY (`track_id`) REFERENCES `tracks` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT `0` FOREIGN KEY (`artist_id`) REFERENCES `artist` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE
);