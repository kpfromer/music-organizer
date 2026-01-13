use async_trait::async_trait;
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelBehavior, ActiveValue::Set};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct StringVec(pub Vec<String>);

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "spotify_track")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub spotify_track_id: i64,
    pub title: String,
    pub duration: Option<i32>,
    pub artists: StringVec,
    pub album: String,
    pub isrc: Option<String>,    // ISRC
    pub barcode: Option<String>, // EAN or UPC barcode
    pub created_at: i64,
    pub updated_at: i64,

    #[sea_orm(has_many, via = "spotify_track_playlist")]
    pub spotify_playlists: HasMany<super::spotify_playlist::Entity>,
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
