use async_trait::async_trait;
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelBehavior, ActiveValue::Set};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "spotify_track_download_failure")]
#[allow(dead_code)]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub spotify_playlist_id: i64,
    pub spotify_track_id: String,
    pub track_name: String,
    pub artist_name: String,
    pub album_name: Option<String>,
    pub isrc: Option<String>,
    pub reason: String,
    pub attempts_count: i32,
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
            attempts_count: Set(1),
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
