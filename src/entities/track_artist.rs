use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "track_artists")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub track_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub artist_id: i64,
    pub is_primary: i32, // SQLite uses INTEGER for boolean (0/1)
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::track::Entity",
        from = "Column::TrackId",
        to = "super::track::Column::Id",
        on_delete = "Cascade"
    )]
    Track,
    #[sea_orm(
        belongs_to = "super::artist::Entity",
        from = "Column::ArtistId",
        to = "super::artist::Column::Id",
        on_delete = "Cascade"
    )]
    Artist,
}

impl ActiveModelBehavior for ActiveModel {}
