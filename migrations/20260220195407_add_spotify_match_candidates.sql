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
  CONSTRAINT `0` FOREIGN KEY (`local_track_id`) REFERENCES `tracks` (`id`) ON UPDATE CASCADE ON DELETE CASCADE,
  CONSTRAINT `1` FOREIGN KEY (`spotify_track_id`) REFERENCES `spotify_track` (`spotify_track_id`) ON UPDATE CASCADE ON DELETE CASCADE
);
-- Create index "idx_spotify_match_candidate_spotify_track" to table: "spotify_match_candidate"
CREATE INDEX `idx_spotify_match_candidate_spotify_track` ON `spotify_match_candidate` (`spotify_track_id`);
-- Create index "idx_spotify_match_candidate_status" to table: "spotify_match_candidate"
CREATE INDEX `idx_spotify_match_candidate_status` ON `spotify_match_candidate` (`status`);
-- Create index "idx_spotify_match_candidate_unique" to table: "spotify_match_candidate"
CREATE UNIQUE INDEX `idx_spotify_match_candidate_unique` ON `spotify_match_candidate` (`spotify_track_id`, `local_track_id`);
