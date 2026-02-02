-- Create "youtube_video" table
CREATE TABLE `youtube_video` (
  `id` integer NOT NULL PRIMARY KEY AUTOINCREMENT,
  `youtube_id` varchar NOT NULL,
  `title` varchar NOT NULL,
  `channel_name` varchar NOT NULL,
  `published_at` integer NOT NULL,
  `thumbnail_url` varchar NOT NULL,
  `video_url` varchar NOT NULL,
  `created_at` integer NOT NULL,
  `updated_at` integer NOT NULL,
  `watched` integer NOT NULL
);
-- Create index "youtube_video_youtube_id" to table: "youtube_video"
CREATE UNIQUE INDEX `youtube_video_youtube_id` ON `youtube_video` (`youtube_id`);
-- Create "youtube_subscription" table
CREATE TABLE `youtube_subscription` (
  `id` integer NOT NULL PRIMARY KEY AUTOINCREMENT,
  `name` varchar NOT NULL,
  `youtube_id` varchar NOT NULL,
  `created_at` integer NOT NULL,
  `updated_at` integer NOT NULL
);
-- Create index "youtube_subscription_youtube_id" to table: "youtube_subscription"
CREATE UNIQUE INDEX `youtube_subscription_youtube_id` ON `youtube_subscription` (`youtube_id`);
