use async_trait::async_trait;
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelBehavior, ActiveValue::Set};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "spotify_playlist_sync_state")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub spotify_playlist_id: i64,
    pub local_playlist_id: Option<i64>,
    pub last_sync_at: Option<i64>,
    // TODO: enum
    pub sync_status: String,
    pub tracks_downloaded: i32,
    pub tracks_failed: i32,
    pub error_log: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            created_at: Set(now),
            updated_at: Set(now),
            sync_status: Set("pending".to_string()),
            tracks_downloaded: Set(0),
            tracks_failed: Set(0),
            ..ActiveModelTrait::default()
        }
    }

    async fn before_save<C>(mut self, _db: &C, insert: bool) -> Result<Self, sea_orm::DbErr>
    where
        C: ConnectionTrait,
    {
        if !insert {
            self.updated_at = Set(chrono::Utc::now().timestamp());
        }
        Ok(self)
    }
}
