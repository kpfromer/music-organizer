use async_trait::async_trait;
use sea_orm::entity::prelude::*;

/// Many-to-many relationship between Spotify tracks and playlists
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "spotify_track_playlist")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub spotify_track_id: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub spotify_playlist_id: i64,
    #[sea_orm(belongs_to, from = "spotify_track_id", to = "spotify_track_id")]
    pub spotify_track: Option<super::spotify_track::Entity>,
    #[sea_orm(belongs_to, from = "spotify_playlist_id", to = "id")]
    pub spotify_playlist: Option<super::spotify_playlist::Entity>,
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {}
