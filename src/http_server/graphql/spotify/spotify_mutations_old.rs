use std::sync::Arc;

use async_graphql::{Context, Object};
use chrono::{DateTime, Utc};
use color_eyre::eyre::OptionExt;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};

use crate::entities;
use crate::http_server::graphql::spotify_queries::{SpotifyAccount, SpotifyPlaylist};
use crate::http_server::graphql_error::GraphqlResult;
use crate::http_server::state::AppState;
use crate::spotify_rs::auth::{exchange_code_for_token, initiate_oauth, refresh_access_token};
use crate::spotify_rs::client::SpotifyClient;
use color_eyre::eyre::WrapErr;
use sea_orm::ActiveModelBehavior;

#[derive(Default)]
pub struct SpotifyMutation;

#[derive(async_graphql::SimpleObject)]
pub struct SpotifyAuthResponse {
    pub auth_url: String,
    pub state: String,
}

#[Object]
impl SpotifyMutation {
    /// Initiate Spotify OAuth flow
    async fn initiate_spotify_auth(&self, ctx: &Context<'_>) -> GraphqlResult<SpotifyAuthResponse> {
        let app_state = ctx
            .data::<Arc<AppState>>()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get app state: {:?}", e))?;

        let client_id = &app_state.spotify_client_id;
        let redirect_uri = format!("{}/spotify-auth/callback-frontend", app_state.base_url);

        let (auth_response, session) = initiate_oauth(client_id, &redirect_uri);

        // Store OAuth session in app state
        {
            let mut sessions = app_state.spotify_oauth_sessions.write().await;
            sessions.insert(session.state.clone(), session);
        }

        Ok(SpotifyAuthResponse {
            auth_url: auth_response.auth_url,
            state: auth_response.state,
        })
    }

    /// Complete Spotify OAuth by exchanging code for access token
    async fn complete_spotify_auth(
        &self,
        ctx: &Context<'_>,
        code: String,
        state: String,
    ) -> GraphqlResult<SpotifyAccount> {
        let app_state = ctx
            .data::<Arc<AppState>>()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get app state: {:?}", e))?;
        let db = &app_state.db;

        println!("Complete Spotify Auth: code: {}, state: {}", code, state);
        {
            let sessions = app_state.spotify_oauth_sessions.read().await;
            println!("State Sessions: {:?}", sessions);
        }

        // Retrieve and remove OAuth session
        let session = {
            let mut sessions = app_state.spotify_oauth_sessions.write().await;
            sessions
                .get(&state)
                .ok_or_else(|| color_eyre::eyre::eyre!("Invalid or expired OAuth state"))?
                .clone()
            // TODO: USE THIS BEFORE MERGING
            // sessions
            //     .remove(&state)
            //     .ok_or_else(|| color_eyre::eyre::eyre!("Invalid or expired OAuth state"))?
        };

        // Verify state matches (CSRF protection)
        if session.state != state {
            return Err(color_eyre::eyre::eyre!("OAuth state mismatch").into());
        }

        let client_id = &app_state.spotify_client_id;
        let client_secret = &app_state.spotify_client_secret;
        let redirect_uri = format!("{}/spotify-auth/callback-frontend", app_state.base_url);

        // Exchange code for token
        let token_response =
            exchange_code_for_token(client_id, client_secret, &code, &redirect_uri)
                .await
                .map_err(|e| color_eyre::eyre::eyre!("Failed to exchange code: {}", e))?;

        // Get user info
        let spotify_client = SpotifyClient::new(token_response.access_token.clone());
        let user = spotify_client
            .get_current_user()
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get user info: {}", e))?;

        // Calculate token expiry timestamp
        let token_expiry = chrono::Utc::now().timestamp() + token_response.expires_in as i64;

        // Store account in database
        let account = entities::spotify_account::ActiveModel {
            user_id: Set(user.id),
            display_name: Set(user.display_name),
            access_token: Set(token_response.access_token),
            refresh_token: Set(token_response.refresh_token.unwrap_or_default()),
            token_expiry: Set(token_expiry),
            ..entities::spotify_account::ActiveModel::new()
        };

        let account_model = account
            .insert(&db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to create spotify account: {}", e))?;

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

    /// Refresh access token for a Spotify account
    async fn refresh_spotify_token(
        &self,
        ctx: &Context<'_>,
        account_id: i64,
    ) -> GraphqlResult<bool> {
        let app_state = ctx
            .data::<Arc<AppState>>()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get app state: {:?}", e))?;
        let db = &app_state.db;

        // Get account from database
        let account = entities::spotify_account::Entity::find_by_id(account_id)
            .one(&db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to find spotify account: {}", e))?
            .ok_or_else(|| {
                color_eyre::eyre::eyre!("Spotify account with id {} not found", account_id)
            })?;

        // Refresh token
        let client_id = &app_state.spotify_client_id;
        let client_secret = &app_state.spotify_client_secret;
        let token_response = refresh_access_token(client_id, client_secret, &account.refresh_token)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to refresh token: {}", e))?;

        // Calculate new token expiry
        let token_expiry = chrono::Utc::now().timestamp() + token_response.expires_in as i64;

        // Update account in database
        let mut account_active: entities::spotify_account::ActiveModel = account.into();
        account_active.access_token = Set(token_response.access_token);
        if let Some(new_refresh_token) = token_response.refresh_token {
            account_active.refresh_token = Set(new_refresh_token);
        }
        account_active.token_expiry = Set(token_expiry);

        account_active
            .update(&db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to update spotify account: {}", e))?;

        Ok(true)
    }

    /// Sync playlists from Spotify account
    async fn sync_spotify_playlists(
        &self,
        ctx: &Context<'_>,
        account_id: i64,
    ) -> GraphqlResult<Vec<SpotifyPlaylist>> {
        let app_state = ctx
            .data::<Arc<AppState>>()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get app state: {:?}", e))?;
        let db = &app_state.db;

        // Get account from database
        let account = entities::spotify_account::Entity::find_by_id(account_id)
            .one(&db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to find spotify account: {}", e))?
            .ok_or_else(|| {
                color_eyre::eyre::eyre!("Spotify account with id {} not found", account_id)
            })?;

        // Check if token needs refresh (expires in less than 5 minutes)
        let now = chrono::Utc::now().timestamp();
        let needs_refresh = account.token_expiry < (now + 300);

        let access_token = if needs_refresh {
            // Refresh token
            let client_id = &app_state.spotify_client_id;
            let client_secret = &app_state.spotify_client_secret;
            let token_response =
                refresh_access_token(client_id, client_secret, &account.refresh_token)
                    .await
                    .map_err(|e| color_eyre::eyre::eyre!("Failed to refresh token: {}", e))?;

            let token_expiry = now + token_response.expires_in as i64;

            // Update account
            let mut account_active: entities::spotify_account::ActiveModel = account.clone().into();
            account_active.access_token = Set(token_response.access_token.clone());
            if let Some(new_refresh_token) = &token_response.refresh_token {
                account_active.refresh_token = Set(new_refresh_token.clone());
            }
            account_active.token_expiry = Set(token_expiry);

            account_active
                .update(&db.conn)
                .await
                .map_err(|e| color_eyre::eyre::eyre!("Failed to update spotify account: {}", e))?;

            token_response.access_token
        } else {
            account.access_token.clone()
        };

        // Get playlists from Spotify
        let spotify_client = SpotifyClient::new(access_token);
        let playlists = spotify_client
            .get_user_playlists()
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get playlists: {}", e))?;

        // Store playlists in database
        let mut result = Vec::new();
        for playlist in playlists {
            let playlist_model = entities::spotify_playlist::ActiveModel {
                account_id: Set(account.id),
                spotify_id: Set(playlist.id),
                name: Set(playlist.name),
                description: Set(playlist.description),
                snapshot_id: Set(playlist.snapshot_id),
                track_count: Set(playlist.tracks.total),
                ..entities::spotify_playlist::ActiveModel::new()
            };

            let saved_playlist = playlist_model
                .insert(&db.conn)
                .await
                .map_err(|e| color_eyre::eyre::eyre!("Failed to save playlist: {}", e))?;

            result.push(SpotifyPlaylist {
                id: saved_playlist.id,
                spotify_id: saved_playlist.spotify_id,
                name: saved_playlist.name,
                description: saved_playlist.description,
                track_count: saved_playlist.track_count,
                created_at: DateTime::<Utc>::from_timestamp(saved_playlist.created_at, 0)
                    .ok_or_eyre("Failed to convert created_at to DateTime<Utc>")?,
                updated_at: DateTime::<Utc>::from_timestamp(saved_playlist.updated_at, 0)
                    .ok_or_eyre("Failed to convert updated_at to DateTime<Utc>")?,
            });
        }

        Ok(result)
    }

    async fn sync_playlist_to_local_library(
        &self,
        ctx: &Context<'_>,
        playlist_id: i64,
    ) -> GraphqlResult<bool> {
        let app_state = ctx
            .data::<Arc<AppState>>()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get app state: {:?}", e))?;
        let acoustid_api_key = app_state.api_key.clone();
        let db = &app_state.db;

        // Get playlist from database
        let playlist = entities::spotify_playlist::Entity::find_by_id(playlist_id)
            .one(&db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to find playlist: {}", e))?
            .ok_or_else(|| color_eyre::eyre::eyre!("Playlist with id {} not found", playlist_id))?;

        // Get account with access token
        let account = entities::spotify_account::Entity::find_by_id(playlist.account_id)
            .one(&db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to find account: {}", e))?
            .ok_or_else(|| color_eyre::eyre::eyre!("Account not found for playlist"))?;

        // Check if token needs refresh (same logic as sync_spotify_playlists)
        let access_token = if account.token_expiry < (chrono::Utc::now().timestamp() + 300) {
            // Refresh token...
            let client_id = &app_state.spotify_client_id;
            let client_secret = &app_state.spotify_client_secret;
            let token_response =
                refresh_access_token(client_id, client_secret, &account.refresh_token)
                    .await
                    .map_err(|e| color_eyre::eyre::eyre!("Failed to refresh token: {}", e))?;

            // Update account...
            token_response.access_token
        } else {
            account.access_token.clone()
        };

        log::debug!("Initializing sync state for playlist: {}", playlist_id);
        // Initialize sync state
        let sync_state = db
            .upsert_playlist_sync_state(playlist_id, "in_progress", 0, 0, None)
            .await
            .wrap_err("Failed to update sync state")?;
        log::debug!("Sync state initialized: {:?}", sync_state);

        let db = &app_state.db;
        let result = crate::spotify_rs::sync_playlist::sync_playlist_to_local_library(
            &playlist.spotify_id,
            &access_token,
            &app_state.soulseek_context,
            db,
            &acoustid_api_key,
        )
        .await;

        match result {
            Ok(sync_result) => {
                // Log failures
                for failed_track in &sync_result.failed_tracks {
                    if let Err(e) = db
                        .log_track_download_failure(
                            playlist_id,
                            &failed_track.spotify_track,
                            &failed_track.reason,
                        )
                        .await
                    {
                        log::error!("Failed to log track failure: {}", e);
                    }
                }

                // Update sync state to completed
                if let Err(e) = db
                    .upsert_playlist_sync_state(
                        playlist_id,
                        "completed",
                        sync_result.newly_downloaded,
                        sync_result.failed,
                        None,
                    )
                    .await
                {
                    log::error!("Failed to update sync state: {}", e);
                }

                log::info!(
                    "Playlist sync completed: {} total, {} downloaded, {} already existed, {} failed",
                    sync_result.total_tracks,
                    sync_result.newly_downloaded,
                    sync_result.already_downloaded,
                    sync_result.failed
                );
            }
            Err(e) => {
                log::error!("Playlist sync failed: {}", e);

                // Update sync state to failed
                if let Err(e) = db
                    .upsert_playlist_sync_state(playlist_id, "failed", 0, 0, Some(&e.to_string()))
                    .await
                {
                    log::error!("Failed to update sync state: {}", e);
                }
            }
        }

        Ok(true)
    }
}
