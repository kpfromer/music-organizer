use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "tracks")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub album_id: i64,
    pub title: String,
    pub track_number: Option<i32>,
    /// Duration in seconds
    pub duration: Option<i32>,
    pub musicbrainz_id: Option<String>,
    pub file_path: String,
    pub sha256: String,
    pub isrcs: Option<String>, // JSON array of ISRCs: ["USRC11234567", ...]
    pub barcode: Option<String>, // EAN or UPC barcode
    pub created_at: i64,
    pub updated_at: i64,

    #[sea_orm(has_many, via = "playlist_track")]
    pub playlists: HasMany<super::playlist::Entity>,

    #[sea_orm(belongs_to, from = "album_id", to = "id")]
    pub album: Option<super::album::Entity>,
}

impl Related<super::artist::Entity> for Entity {
    fn to() -> RelationDef {
        super::track_artist::Relation::Artist.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::track_artist::Relation::Track.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
