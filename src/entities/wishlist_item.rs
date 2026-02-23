use async_trait::async_trait;
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelBehavior, ActiveValue::Set};

#[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum WishlistStatus {
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "searching")]
    Searching,
    #[sea_orm(string_value = "downloading")]
    Downloading,
    #[sea_orm(string_value = "importing")]
    Importing,
    #[sea_orm(string_value = "completed")]
    Completed,
    #[sea_orm(string_value = "failed")]
    Failed,
}

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "wishlist_item")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub spotify_track_id: String,
    pub status: WishlistStatus,
    pub error_reason: Option<String>,
    pub attempts_count: i32,
    pub last_attempt_at: Option<i64>,
    pub next_retry_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,

    #[sea_orm(belongs_to, from = "spotify_track_id", to = "spotify_track_id")]
    pub spotify_track: HasOne<super::spotify_track::Entity>,
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            created_at: Set(now),
            updated_at: Set(now),
            status: Set(WishlistStatus::Pending),
            attempts_count: Set(0),
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
