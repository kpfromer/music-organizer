use std::sync::Arc;

use super::spotify_queries::SpotifyAccount;
use crate::entities;
use crate::http_server::graphql::context::get_app_state;
use crate::http_server::graphql::spotify::context::get_spotify_adapter;
use crate::http_server::graphql_error::GraphqlResult;
use crate::http_server::state::AppState;
use crate::services::spotify::client::start_spotify_auth_flow;
use crate::services::spotify::matching_local_tracks::match_existing_spotify_tracks_with_local_task;
use crate::services::spotify::sync_spotify_playlist_to_local_library::sync_spotify_playlist_to_local_library_task;
use async_graphql::{Context, Object};
use chrono::{DateTime, Utc};
use color_eyre::eyre::OptionExt;
use color_eyre::eyre::WrapErr;
use sea_orm::ActiveModelBehavior;
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};

#[derive(Default)]
pub struct SpotifyMutation;

#[derive(async_graphql::SimpleObject)]
pub struct SpotifyAuthResponse {
    pub redirect_url: String,
}

#[Object]
impl SpotifyMutation {
    /// Initiate Spotify OAuth flow
    async fn initiate_spotify_auth(&self, ctx: &Context<'_>) -> GraphqlResult<SpotifyAuthResponse> {
        let credentials = get_app_state(ctx)?
            .spotify_credentials
            .clone()
            .ok_or_eyre("Spotify credentials not found")?;
        let app_state = get_app_state(ctx)?;

        let (spotify_client, redirect_url) = start_spotify_auth_flow(credentials.clone());

        // Store OAuth session in app state
        {
            let mut session = app_state.spotify_oauth_session.lock().await;
            *session = Some(spotify_client);
        }

        Ok(SpotifyAuthResponse {
            redirect_url: redirect_url.to_string(),
        })
    }

    /// Complete Spotify OAuth by exchanging code for access token
    async fn complete_spotify_auth(
        &self,
        ctx: &Context<'_>,
        auth_code: String,
        csrf_state: String,
    ) -> GraphqlResult<SpotifyAccount> {
        let app_state = ctx
            .data::<Arc<AppState>>()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get app state: {:?}", e))?;
        let db = &app_state.db;

        // Retrieve and remove OAuth session
        let session = {
            let mut session = app_state.spotify_oauth_session.lock().await;
            session.take().ok_or_eyre("No spotify session found")?
        };

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
            .one(&db.conn)
            .await
            .wrap_err("Failed to check for existing spotify account")?;

        let account_model = if let Some(existing) = existing_account {
            // Update existing account with new tokens
            let mut account: entities::spotify_account::ActiveModel = existing.into();
            account.display_name = Set(user.display_name);
            account.access_token = Set(access_token);
            account.refresh_token = Set(refresh_token);
            account.token_expiry = Set(0);
            // updated_at will be set automatically by before_save hook

            account
                .update(&db.conn)
                .await
                .wrap_err("Failed to update spotify account")?
        } else {
            // Create new account
            let account = entities::spotify_account::ActiveModel {
                user_id: Set(user.id),
                display_name: Set(user.display_name),
                access_token: Set(access_token),
                refresh_token: Set(refresh_token),
                // TODO: remove this?
                token_expiry: Set(0),
                ..entities::spotify_account::ActiveModel::new()
            };

            account
                .insert(&db.conn)
                .await
                .map_err(|e| color_eyre::eyre::eyre!("Failed to create spotify account: {}", e))?
        };

        Ok(SpotifyAccount {
            id: account_model.id,
            user_id: account_model.user_id,
            display_name: account_model.display_name,
            created_at: DateTime::<Utc>::from_timestamp(account_model.created_at, 0)
                .ok_or_eyre("Failed to convert created_at to DateTime<Utc>")?,
            updated_at: DateTime::<Utc>::from_timestamp(account_model.updated_at, 0)
                .ok_or_eyre("Failed to convert updated_at to DateTime<Utc>")?,
        })
    }

    async fn delete_spotify_account(
        &self,
        ctx: &Context<'_>,
        account_id: i64,
    ) -> GraphqlResult<bool> {
        let app_state = get_app_state(ctx)?;
        let db = &app_state.db;

        entities::spotify_account::Entity::delete_by_id(account_id)
            .exec(&db.conn)
            .await
            .wrap_err("Failed to delete spotify account")?;
        Ok(true)
    }

    async fn sync_spotify_account_playlists_to_db(
        &self,
        ctx: &Context<'_>,
        account_id: i64,
    ) -> GraphqlResult<bool> {
        let app_state = get_app_state(ctx)?;
        let db = &app_state.db;
        let spotify_account = entities::spotify_account::Entity::find_by_id(account_id)
            .one(&db.conn)
            .await
            .wrap_err("Failed to fetch spotify account")?
            .ok_or_eyre("Spotify account not found")?;
        let adapter = get_spotify_adapter(app_state, spotify_account).await?;

        let service = crate::services::spotify::sync::SpotifySyncService::new(db.clone(), adapter);
        service.sync_account_playlists(account_id).await?;

        Ok(true)
    }

    async fn sync_spotify_playlist_to_local_library(
        &self,
        ctx: &Context<'_>,
        spotify_account_id: i64,
        spotify_playlist_id: i64,
        local_playlist_name: String,
    ) -> GraphqlResult<bool> {
        let app_state = get_app_state(ctx)?;
        let db = app_state.db.clone();
        let soulseek_context = app_state.soulseek_context.clone();
        let api_key = &app_state.api_key;
        let config = &app_state.config;

        sync_spotify_playlist_to_local_library_task(
            db,
            soulseek_context,
            api_key,
            config,
            spotify_account_id,
            spotify_playlist_id,
            local_playlist_name,
        )
        .await?;

        Ok(true)
    }

    async fn match_existing_spotify_tracks_with_local_tracks(
        &self,
        ctx: &Context<'_>,
    ) -> GraphqlResult<bool> {
        let db = get_app_state(ctx)?.db.clone();

        let spotify_tracks = entities::spotify_track::Entity::find()
            .filter(entities::spotify_track::Column::LocalTrackId.is_null())
            .all(&db.conn)
            .await
            .wrap_err("Failed to fetch spotify tracks from db")?;

        match_existing_spotify_tracks_with_local_task(db, spotify_tracks).await?;
        Ok(true)
    }

    /// Accept a match candidate — links the Spotify track to the local track and dismisses other candidates
    async fn accept_spotify_match_candidate(
        &self,
        ctx: &Context<'_>,
        candidate_id: i64,
    ) -> GraphqlResult<bool> {
        let db = &get_app_state(ctx)?.db;
        let service = crate::services::spotify::matching::SpotifyMatchingService::new(db.clone());
        service.accept_candidate(candidate_id).await?;
        Ok(true)
    }

    /// Dismiss all pending candidates for a Spotify track (removes from review queue without matching)
    async fn dismiss_spotify_unmatched_track(
        &self,
        ctx: &Context<'_>,
        spotify_track_id: String,
    ) -> GraphqlResult<bool> {
        let db = &get_app_state(ctx)?.db;
        let service = crate::services::spotify::matching::SpotifyMatchingService::new(db.clone());
        service.dismiss_track(&spotify_track_id).await?;
        Ok(true)
    }

    /// Manually match a Spotify track to a local track (from library search)
    async fn manually_match_spotify_track(
        &self,
        ctx: &Context<'_>,
        spotify_track_id: String,
        local_track_id: i64,
    ) -> GraphqlResult<bool> {
        let db = &get_app_state(ctx)?.db;
        let service = crate::services::spotify::matching::SpotifyMatchingService::new(db.clone());
        service
            .manually_match(&spotify_track_id, local_track_id)
            .await?;
        Ok(true)
    }
}
