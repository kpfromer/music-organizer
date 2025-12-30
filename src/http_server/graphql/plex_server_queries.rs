use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, SimpleObject)]
pub struct PlexServer {
    pub id: i64,
    pub name: String,
    pub server_url: String,
    pub has_access_token: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct AuthResponse {
    pub auth_url: String,
    pub pin_id: i32,
}

