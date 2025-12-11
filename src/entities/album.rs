use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "albums")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub title: String,
    pub musicbrainz_id: Option<String>,
    pub year: Option<i32>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::track::Entity")]
    Track,
}

impl Related<super::artist::Entity> for Entity {
    fn to() -> RelationDef {
        super::album_artist::Relation::Artist.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::album_artist::Relation::Album.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
