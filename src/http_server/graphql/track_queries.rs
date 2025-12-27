use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, SimpleObject)]
pub struct Track {
    pub id: i64,
    pub title: String,
    pub track_number: Option<i32>,
    pub duration: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub album: Album,
    pub artists: Vec<Artist>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct Album {
    pub id: i64,
    pub title: String,
    pub year: Option<i32>,
    pub artwork_url: Option<String>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct Artist {
    pub id: i64,
    pub name: String,
}
