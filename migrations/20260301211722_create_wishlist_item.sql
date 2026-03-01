-- Disable the enforcement of foreign-keys constraints
PRAGMA foreign_keys = off;
-- Create "new_playlists" table
CREATE TABLE `new_playlists` (
  `id` integer NOT NULL PRIMARY KEY AUTOINCREMENT,
  `name` varchar NOT NULL,
  `description` varchar NULL,
  `spotify_playlist_id` integer NULL,
  `created_at` timestamp_text NOT NULL,
  `updated_at` timestamp_text NOT NULL,
  CONSTRAINT `fk_playlists_spotify_playlist` FOREIGN KEY (`spotify_playlist_id`) REFERENCES `spotify_playlist` (`id`) ON UPDATE CASCADE ON DELETE SET NULL
);
-- Copy rows from old table "playlists" to new temporary table "new_playlists"
INSERT INTO `new_playlists` (`id`, `name`, `description`, `created_at`, `updated_at`) SELECT `id`, `name`, `description`, `created_at`, `updated_at` FROM `playlists`;
-- Drop "playlists" table after copying rows
DROP TABLE `playlists`;
-- Rename temporary table "new_playlists" to "playlists"
ALTER TABLE `new_playlists` RENAME TO `playlists`;
-- Create "wishlist_item" table
CREATE TABLE `wishlist_item` (
  `id` integer NOT NULL PRIMARY KEY AUTOINCREMENT,
  `spotify_track_id` varchar NOT NULL,
  `status` varchar NOT NULL DEFAULT 'pending',
  `error_reason` varchar NULL,
  `attempts_count` integer NOT NULL DEFAULT 0,
  `last_attempt_at` integer NULL,
  `next_retry_at` integer NULL,
  `created_at` integer NOT NULL DEFAULT (strftime('%s', 'now')),
  `updated_at` integer NOT NULL DEFAULT (strftime('%s', 'now')),
  CONSTRAINT `fk_wishlist_item_spotify_track` FOREIGN KEY (`spotify_track_id`) REFERENCES `spotify_track` (`spotify_track_id`) ON UPDATE CASCADE ON DELETE CASCADE
);
-- Create index "idx_wishlist_item_status" to table: "wishlist_item"
CREATE INDEX `idx_wishlist_item_status` ON `wishlist_item` (`status`);
-- Create index "idx_wishlist_item_next_retry" to table: "wishlist_item"
CREATE INDEX `idx_wishlist_item_next_retry` ON `wishlist_item` (`next_retry_at`);
-- Create index "idx_wishlist_item_spotify_track" to table: "wishlist_item"
CREATE UNIQUE INDEX `idx_wishlist_item_spotify_track` ON `wishlist_item` (`spotify_track_id`);
-- Enable back the enforcement of foreign-keys constraints
PRAGMA foreign_keys = on;
