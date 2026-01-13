use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "artist")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub name: String,
    pub musicbrainz_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Related<super::album::Entity> for Entity {
    fn to() -> RelationDef {
        super::album_artist::Relation::Album.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::album_artist::Relation::Artist.def().rev())
    }
}

impl Related<super::track::Entity> for Entity {
    fn to() -> RelationDef {
        super::track_artist::Relation::Track.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::track_artist::Relation::Artist.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
