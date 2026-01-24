use std::sync::Arc;
use tracing;

use super::spotify_queries::SpotifyAccount;
use crate::entities;
use crate::http_server::graphql::context::get_app_state;
use crate::http_server::graphql::spotify::context::get_spotify_client;
use crate::http_server::graphql::spotify::matching_local_tracks::match_existing_spotify_tracks_with_local_task;
use crate::http_server::graphql::spotify::sync_spotify_playlist_to_local_library::sync_spotify_playlist_to_local_library_task;
use crate::http_server::graphql_error::GraphqlResult;
use crate::http_server::state::AppState;
use crate::services::spotify::client::start_spotify_auth_flow;
use async_graphql::{Context, Object};
use chrono::{DateTime, Utc};
use color_eyre::eyre::OptionExt;
use color_eyre::eyre::WrapErr;
use sea_orm::ActiveModelBehavior;
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;
use sea_orm::TransactionTrait;
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

        // Store account in database
        let account = entities::spotify_account::ActiveModel {
            user_id: Set(user.id),
            display_name: Set(user.display_name),
            access_token: Set(access_token),
            refresh_token: Set(refresh_token),
            // TODO: remove this?
            token_expiry: Set(0),
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
        let db = &get_app_state(ctx)?.db;
        let spotify_account = entities::spotify_account::Entity::find_by_id(account_id)
            .one(&db.conn)
            .await
            .wrap_err("Failed to fetch spotify account")?
            .ok_or_eyre("Spotify account not found")?;
        let spotify_client = get_spotify_client(ctx, spotify_account).await?;

        let playlists = spotify_rs::current_user_playlists()
            .get(&spotify_client)
            .await
            .wrap_err("Failed to fetch user spotify playlists")?
            .get_all(&spotify_client)
            .await
            .wrap_err("Unable to get all user spotify playlists")?;

        let txn = db
            .conn
            .begin()
            .await
            .wrap_err("Failed to begin transaction")?;
        for playlist in playlists.into_iter().flatten() {
            // Upsert spotify playlist details
            let saved_playlist = if let Some(existing_playlist) =
                entities::spotify_playlist::Entity::find()
                    .filter(entities::spotify_playlist::Column::SpotifyId.eq(&playlist.id))
                    .one(&txn)
                    .await
                    .wrap_err("Failed to fetch saved spotify playlist")?
            {
                let mut existing_playlist_model: entities::spotify_playlist::ActiveModel =
                    existing_playlist.into();

                existing_playlist_model.account_id = Set(account_id);
                existing_playlist_model.spotify_id = Set(playlist.id.clone());
                existing_playlist_model.name = Set(playlist.name);
                existing_playlist_model.description = Set(playlist.description);
                existing_playlist_model.snapshot_id = Set(playlist.snapshot_id);
                existing_playlist_model.track_count =
                    Set(playlist.tracks.map(|t| t.total).unwrap_or(0) as i32);
                existing_playlist_model.updated_at = Set(chrono::Utc::now().timestamp());

                entities::spotify_playlist::Entity::update(existing_playlist_model)
                    .exec(&txn)
                    .await
                    .wrap_err("Failed to update spotify playlist")?
            } else {
                let playlist_model = entities::spotify_playlist::ActiveModel {
                    account_id: Set(account_id),
                    spotify_id: Set(playlist.id.clone()),
                    name: Set(playlist.name),
                    description: Set(playlist.description),
                    snapshot_id: Set(playlist.snapshot_id),
                    track_count: Set(playlist.tracks.map(|t| t.total).unwrap_or(0) as i32),
                    ..entities::spotify_playlist::ActiveModel::new()
                };
                entities::spotify_playlist::Entity::insert(playlist_model)
                    .exec_with_returning(&txn)
                    .await
                    .wrap_err("Failed to save spotify playlist")?
            };

            tracing::info!("Saved spotify playlist: {:?}", saved_playlist);

            let spotify_tracks_from_api = spotify_rs::playlist_items(&saved_playlist.spotify_id)
                .get(&spotify_client)
                .await
                .wrap_err("Failed to fetch spotify tracks from api")?
                .get_all(&spotify_client)
                .await
                .wrap_err("Unable to get all spotify playlist items")?;

            for track in spotify_tracks_from_api.into_iter().flatten() {
                let track_id = if let spotify_rs::model::PlayableItem::Track(track) = track.track {
                    let track_model = entities::spotify_track::ActiveModel {
                        spotify_track_id: Set(track.id.clone()),
                        title: Set(track.name),
                        duration: Set(Some(track.duration_ms as i32)),
                        artists: Set(entities::spotify_track::StringVec(
                            track.artists.iter().map(|a| a.name.clone()).collect(),
                        )),
                        album: Set(track.album.name.clone()),
                        isrc: Set(track.external_ids.isrc.clone()),
                        barcode: Set(track.external_ids.upc.clone()),
                        created_at: Set(chrono::Utc::now().timestamp()),
                        updated_at: Set(chrono::Utc::now().timestamp()),
                        local_track_id: Set(None),
                    };
                    // Upsert spotify track details
                    match entities::spotify_track::Entity::find()
                        .filter(entities::spotify_track::Column::SpotifyTrackId.eq(&track.id))
                        .one(&txn)
                        .await
                        .wrap_err("Failed to fetch saved spotify track")?
                    {
                        Some(existing_track) => {
                            let mut existing_track_model: entities::spotify_track::ActiveModel =
                                existing_track.into();
                            existing_track_model.updated_at = Set(chrono::Utc::now().timestamp());
                            existing_track_model.duration = Set(Some(track.duration_ms as i32));
                            existing_track_model.artists = Set(entities::spotify_track::StringVec(
                                track.artists.iter().map(|a| a.name.clone()).collect(),
                            ));
                            existing_track_model.album = Set(track.album.name.clone());
                            existing_track_model.isrc = Set(track.external_ids.isrc.clone());
                            existing_track_model.barcode = Set(track.external_ids.upc.clone());
                            tracing::info!(
                                "Updated spotify track in db: {:?}",
                                existing_track_model
                            );
                            entities::spotify_track::Entity::update(existing_track_model)
                                .exec(&txn)
                                .await
                                .wrap_err("Failed to update spotify track")?;
                        }
                        None => {
                            entities::spotify_track::Entity::insert(track_model)
                                .exec(&txn)
                                .await
                                .wrap_err("Failed to save spotify track")?;
                            tracing::info!("Saved new spotify track to db",);
                        }
                    }
                    track.id.clone()
                } else {
                    continue;
                };

                // Create link between spotify track and playlist
                if entities::spotify_track_playlist::Entity::find()
                    .filter(entities::spotify_track_playlist::Column::SpotifyTrackId.eq(&track_id))
                    .filter(
                        entities::spotify_track_playlist::Column::SpotifyPlaylistId
                            .eq(saved_playlist.id),
                    )
                    .one(&txn)
                    .await
                    .wrap_err("Failed to fetch saved spotify track playlist")?
                    .is_some()
                {
                    continue;
                }

                let spotify_track_playlist_model = entities::spotify_track_playlist::ActiveModel {
                    spotify_track_id: Set(track_id),
                    spotify_playlist_id: Set(saved_playlist.id),
                };
                entities::spotify_track_playlist::Entity::insert(spotify_track_playlist_model)
                    .exec(&txn)
                    .await
                    .wrap_err("Failed to save spotify track playlist")?;
                tracing::info!("Saved spotify track playlist",);
            }
        }
        txn.commit()
            .await
            .wrap_err("Failed to commit transaction")?;

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
}
