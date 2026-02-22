use std::sync::Arc;

use color_eyre::eyre::{OptionExt, Result, WrapErr};
use sea_orm::{ActiveModelBehavior, ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use spotify_rs::client::Client as SpotifyRsClient;
use spotify_rs::{AuthCodeFlow, Unauthenticated};

use crate::database::Database;
use crate::entities;

pub struct SpotifyAccountService {
    db: Arc<Database>,
}

impl SpotifyAccountService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub async fn list_accounts(&self) -> Result<Vec<entities::spotify_account::Model>> {
        entities::spotify_account::Entity::find()
            .all(&self.db.conn)
            .await
            .wrap_err("Failed to fetch spotify accounts")
    }

    pub async fn delete_account(&self, account_id: i64) -> Result<()> {
        entities::spotify_account::Entity::delete_by_id(account_id)
            .exec(&self.db.conn)
            .await
            .wrap_err("Failed to delete spotify account")?;
        Ok(())
    }

    /// Complete the OAuth flow: authenticate, get user profile, upsert account.
    /// The caller (resolver) extracts the OAuth session from AppState and passes it in.
    pub async fn complete_auth(
        &self,
        session: SpotifyRsClient<Unauthenticated, AuthCodeFlow>,
        auth_code: String,
        csrf_state: String,
    ) -> Result<entities::spotify_account::Model> {
        let authenticated_client = session
            .authenticate(auth_code, csrf_state)
            .await
            .wrap_err("Failed to authenticate spotify session")?;

        let user = spotify_rs::get_current_user_profile(&authenticated_client)
            .await
            .wrap_err("Failed to get user info")?;

        let access_token = authenticated_client
            .access_token()
            .wrap_err("Failed to get access token")?;
        let refresh_token = authenticated_client
            .refresh_token()
            .wrap_err("Failed to get refresh token")?
            .ok_or_eyre("No refresh token found")?;

        // Check if account already exists
        let existing_account = entities::spotify_account::Entity::find()
            .filter(entities::spotify_account::Column::UserId.eq(&user.id))
            .one(&self.db.conn)
            .await
            .wrap_err("Failed to check for existing spotify account")?;

        let account_model = if let Some(existing) = existing_account {
            // Update existing account with new tokens
            let mut account: entities::spotify_account::ActiveModel = existing.into();
            account.display_name = Set(user.display_name);
            account.access_token = Set(access_token);
            account.refresh_token = Set(refresh_token);
            account.token_expiry = Set(0);

            account
                .update(&self.db.conn)
                .await
                .wrap_err("Failed to update spotify account")?
        } else {
            // Create new account
            let account = entities::spotify_account::ActiveModel {
                user_id: Set(user.id),
                display_name: Set(user.display_name),
                access_token: Set(access_token),
                refresh_token: Set(refresh_token),
                token_expiry: Set(0),
                ..entities::spotify_account::ActiveModel::new()
            };

            account
                .insert(&self.db.conn)
                .await
                .wrap_err("Failed to create spotify account")?
        };

        Ok(account_model)
    }
}
