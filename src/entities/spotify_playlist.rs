use async_trait::async_trait;
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelBehavior, ActiveValue::Set};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "spotify_playlist")]
#[allow(dead_code)]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub account_id: i64,
    #[sea_orm(unique)]
    pub spotify_id: String,
    pub name: String,
    pub description: Option<String>,
    pub snapshot_id: String,
    pub track_count: i32,
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
