use async_trait::async_trait;
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelBehavior, ActiveValue::Set};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum CandidateConfidence {
    #[sea_orm(string_value = "high")]
    High,
    #[sea_orm(string_value = "medium")]
    Medium,
    #[sea_orm(string_value = "low")]
    Low,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum CandidateDurationMatch {
    #[sea_orm(string_value = "exact")]
    Exact,
    #[sea_orm(string_value = "close")]
    Close,
    #[sea_orm(string_value = "mismatch")]
    Mismatch,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum CandidateVersionMatch {
    #[sea_orm(string_value = "match")]
    Match,
    #[sea_orm(string_value = "mismatch")]
    Mismatch,
    #[sea_orm(string_value = "ambiguous")]
    Ambiguous,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum CandidateStatus {
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "accepted")]
    Accepted,
    #[sea_orm(string_value = "dismissed")]
    Dismissed,
}

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "spotify_match_candidate")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub spotify_track_id: String,
    pub local_track_id: i64,
    pub score: f64,
    pub confidence: CandidateConfidence,
    pub title_similarity: f64,
    pub artist_similarity: f64,
    pub album_similarity: f64,
    pub duration_match: CandidateDurationMatch,
    pub version_match: CandidateVersionMatch,
    pub status: CandidateStatus,
    pub created_at: i64,
    pub updated_at: i64,

    #[sea_orm(belongs_to, from = "spotify_track_id", to = "spotify_track_id")]
    pub spotify_track: Option<super::spotify_track::Entity>,
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            created_at: Set(now),
            updated_at: Set(now),
            status: Set(CandidateStatus::Pending),
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
