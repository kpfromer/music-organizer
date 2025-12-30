use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, SimpleObject)]
pub struct Playlist {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub track_count: i64,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct PlaylistsResponse {
    pub playlists: Vec<Playlist>,
    pub total_count: i64,
    pub page: i32,
    pub page_size: i32,
}

