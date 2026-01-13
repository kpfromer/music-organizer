use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "album_artist")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub album_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub artist_id: i64,
    pub is_primary: i32, // SQLite uses INTEGER for boolean (0/1)

    #[sea_orm(belongs_to, from = "album_id", to = "id")]
    pub album: Option<super::album::Entity>,
    #[sea_orm(belongs_to, from = "artist_id", to = "id")]
    pub artist: Option<super::artist::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
