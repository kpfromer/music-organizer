use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelBehavior, ActiveValue::Set};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "playlist_tracks")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub playlist_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub track_id: i64,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    #[sea_orm(belongs_to, from = "playlist_id", to = "id")]
    pub playlist: Option<super::playlist::Entity>,
    #[sea_orm(belongs_to, from = "track_id", to = "id")]
    pub track: Option<super::track::Entity>,
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        let now = Utc::now();

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
        let now = Utc::now();

        if insert {
            self.created_at = Set(now);
        }

        self.updated_at = Set(now);

        Ok(self)
    }
}
