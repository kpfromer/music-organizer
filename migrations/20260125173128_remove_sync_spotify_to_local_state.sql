-- Disable the enforcement of foreign-keys constraints
PRAGMA foreign_keys = off;
-- Drop "spotify_playlist_sync_state" table
DROP TABLE `spotify_playlist_sync_state`;
-- Enable back the enforcement of foreign-keys constraints
PRAGMA foreign_keys = on;
