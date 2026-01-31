use async_graphql::{Context, Object};
use chrono::{DateTime, Utc};
use tracing::{info, instrument};

use crate::http_server::graphql_error::GraphqlResult;
use crate::services;

#[derive(Default)]
pub struct YoutubeQuery;

#[derive(async_graphql::SimpleObject, Debug)]
pub struct Video {
    pub id: String,
    pub title: String,
    pub channel_id: String,
    pub channel_name: String,
    pub published_at: Option<DateTime<Utc>>,
    pub thumbnail_url: String,
    pub video_url: String,
}

#[Object]
impl YoutubeQuery {
    /// Get all videos from subscribed channels
    /// Cache for 3 minutes
    #[graphql(cache_control(max_age = 180))]
    #[instrument(skip(self, _ctx))]
    async fn youtube_videos(&self, _ctx: &Context<'_>) -> GraphqlResult<Vec<Video>> {
        // let db = &get_app_state(ctx)?.db;
        // TODO: get subscriptions from db
        // TODO: filter for watched/unwatched videos from db
        // TODO: don't use cache, use db for youtube videos

        let subscriptions = vec![
            "Vaush".to_string(),
            "PapaMeat".to_string(),
            "AtriocClips".to_string(),
            "TheVaushPit".to_string(),
            "mikeokay".to_string(),
            "_jared".to_string(),
            "nathanlaundry".to_string(),
            "drewisgooden".to_string(),
            "ManCarryingThing".to_string(),
            "TheChristomer".to_string(),
            "Bthelick".to_string(),
            "briandavidgilbert".to_string(),
            "atrioc".to_string(),
            "johnnyharris".to_string(),
            "JeffGeerling".to_string(),
            "Fireship".to_string(),
        ];

        let channel_ids = {
            let mut channel_ids = Vec::new();
            for subscription in subscriptions {
                let channel_id = services::youtube::feed::get_channel_id(&subscription).await?;
                if let Some(channel_id) = channel_id {
                    channel_ids.push(channel_id);
                }
            }
            channel_ids
        };
        info!(channel_ids = ?channel_ids, "Found channel IDs");
        let mut videos = {
            let mut videos = Vec::new();
            for channel_id in channel_ids {
                let feed = services::youtube::feed::fetch_feed(&channel_id).await?;
                videos.extend(feed.entries.into_iter().map(|entry| Video {
                    id: entry.id,
                    title: entry.title,
                    channel_id: entry.channel_id,
                    channel_name: entry.author.name,
                    published_at: entry.published.parse::<DateTime<Utc>>().ok(),
                    thumbnail_url: entry.media_group.thumbnail.url,
                    video_url: entry.link.href,
                }));
            }
            videos
        };

        videos.sort_by_key(|video| video.published_at.unwrap_or(Utc::now()));
        videos.reverse();

        info!(videos = ?videos, "Found videos");
        Ok(videos)
    }
}
