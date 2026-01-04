use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelBehavior, ActiveValue::Set};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "plex_servers")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    /// The server name (as seen in the Plex web interface)
    #[sea_orm(unique)]
    pub name: String,
    /// The local plex server URL.
    #[sea_orm(unique)]
    pub server_url: String,
    /// The server access token to use for API calls.
    pub access_token: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
