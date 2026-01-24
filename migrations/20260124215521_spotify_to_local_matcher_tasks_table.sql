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
