pub mod client;

use std::sync::Arc;

use color_eyre::eyre::OptionExt;
use sea_orm::{ActiveModelBehavior, ActiveModelTrait, EntityTrait, Set};
use url::Url;

use crate::database::Database;
use crate::entities;
use crate::plex_rs::all_tracks::{PlexLibraryTrack, PlexMediaContainer};
use crate::plex_rs::library_refresh::PlexActivity;
use crate::plex_rs::playlist::PlexPlaylist;
use crate::ports::plex::PlexClient;

/// Outcome for the plex_tracks query, decoupled from GraphQL types.
pub enum PlexTracksOutcome {
    Success(PlexMediaContainer<PlexLibraryTrack>),
    NoServer,
    MultipleServers(usize),
    NoToken,
    Error(String),
}

pub struct PlexService<C: PlexClient> {
    db: Arc<Database>,
    client: C,
}

impl<C: PlexClient> PlexService<C> {
    pub fn new(db: Arc<Database>, client: C) -> Self {
        Self { db, client }
    }

    // ---- Shared helpers ----

    /// Look up the single plex server. Returns (model, parsed_url, access_token).
    async fn resolve_server(
        &self,
    ) -> color_eyre::Result<(entities::plex_server::Model, Url, String)> {
        let servers = entities::plex_server::Entity::find()
            .all(&self.db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to fetch plex servers: {}", e))?;

        if servers.is_empty() {
            return Err(color_eyre::eyre::eyre!(
                "No Plex server configured. Please add a Plex server first."
            ));
        }
        if servers.len() > 1 {
            return Err(color_eyre::eyre::eyre!(
                "Multiple Plex servers found ({}). Only one server is supported at a time.",
                servers.len()
            ));
        }

        let server = servers.into_iter().next().unwrap();
        let access_token = server.access_token.clone().ok_or_eyre(
            "Plex server does not have an access token. Please authenticate the server first.",
        )?;
        let server_url = Url::parse(&server.server_url)
            .map_err(|e| color_eyre::eyre::eyre!("Invalid server URL: {}", e))?;

        Ok((server, server_url, access_token))
    }

    /// Look up a server by ID. Returns (model, parsed_url, access_token).
    async fn resolve_server_by_id(
        &self,
        id: i64,
    ) -> color_eyre::Result<(entities::plex_server::Model, Url, String)> {
        let server = entities::plex_server::Entity::find_by_id(id)
            .one(&self.db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to fetch Plex server: {}", e))?
            .ok_or_eyre("Plex server not found")?;

        let access_token = server.access_token.clone().ok_or_eyre(
            "Plex server does not have an access token. Please authenticate the server first.",
        )?;
        let server_url = Url::parse(&server.server_url)
            .map_err(|e| color_eyre::eyre::eyre!("Invalid server URL: {}", e))?;

        Ok((server, server_url, access_token))
    }

    /// Get library sections and find the music section ID.
    async fn resolve_music_section(
        &self,
        server_url: &Url,
        token: &str,
    ) -> color_eyre::Result<String> {
        let sections = self.client.get_library_sections(server_url, token).await?;
        let section_id = self
            .client
            .find_music_section_id(&sections)
            .ok_or_eyre("No music library section found on Plex server")?;
        Ok(section_id)
    }

    // ---- CRUD ----

    pub async fn list_servers(&self) -> color_eyre::Result<Vec<entities::plex_server::Model>> {
        entities::plex_server::Entity::find()
            .all(&self.db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to fetch plex servers: {}", e))
    }

    pub async fn create_server(
        &self,
        name: String,
        server_url: String,
    ) -> color_eyre::Result<entities::plex_server::Model> {
        Url::parse(&server_url)
            .map_err(|e| color_eyre::eyre::eyre!("Invalid server URL: {}", e))?;

        let server = entities::plex_server::ActiveModel {
            name: Set(name),
            server_url: Set(server_url),
            ..entities::plex_server::ActiveModel::new()
        };

        let server_model = server
            .insert(&self.db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to create plex server: {}", e))?;

        Ok(server_model)
    }

    // ---- Auth ----

    /// Returns (auth_url, pin_id)
    pub async fn start_authentication(
        &self,
        server_id: i64,
        base_url: &str,
    ) -> color_eyre::Result<(String, i32)> {
        // Verify server exists
        let _server = entities::plex_server::Entity::find_by_id(server_id)
            .one(&self.db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to find plex server: {}", e))?
            .ok_or_else(|| {
                color_eyre::eyre::eyre!("Plex server with id {} not found", server_id)
            })?;

        let pin = self
            .client
            .create_pin()
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to create plex pin: {}", e))?;

        let forward_url = format!("{}/plex-auth/callback", base_url);
        let auth_url = self
            .client
            .construct_auth_url(&pin.code, &forward_url)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to construct auth URL: {}", e))?;

        Ok((auth_url, pin.id))
    }

    pub async fn complete_authentication(
        &self,
        server_id: i64,
        pin_id: i32,
    ) -> color_eyre::Result<entities::plex_server::Model> {
        let server_model = entities::plex_server::Entity::find_by_id(server_id)
            .one(&self.db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to find plex server: {}", e))?
            .ok_or_else(|| {
                color_eyre::eyre::eyre!("Plex server with id {} not found", server_id)
            })?;

        // Poll for auth token
        let mut user_token: Option<String> = None;
        for _ in 0..30 {
            let auth_response = self
                .client
                .poll_for_auth(pin_id)
                .await
                .map_err(|e| color_eyre::eyre::eyre!("Failed to poll for plex auth: {}", e))?;

            if let Some(token) = auth_response.auth_token {
                user_token = Some(token);
                break;
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }

        let user_token = user_token.ok_or_else(|| {
            color_eyre::eyre::eyre!("Authentication timeout: PIN was not claimed within 30 seconds")
        })?;

        // Get resources to find server access token
        let resources = self
            .client
            .get_resources(&user_token)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get plex resources: {}", e))?;

        let matching_resource = resources
            .into_iter()
            .find(|resource| resource.name == server_model.name);

        let access_token = matching_resource
            .and_then(|r| r.access_token)
            .ok_or_else(|| {
                color_eyre::eyre::eyre!(
                    "No matching Plex server found or server has no access token"
                )
            })?;

        let mut server_active: entities::plex_server::ActiveModel = server_model.into();
        server_active.access_token = Set(Some(access_token));

        let updated_server = server_active
            .update(&self.db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to update plex server: {}", e))?;

        Ok(updated_server)
    }

    // ---- Library ----

    pub async fn get_tracks(&self) -> color_eyre::Result<PlexTracksOutcome> {
        let servers = entities::plex_server::Entity::find()
            .all(&self.db.conn)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to fetch plex servers: {}", e))?;

        if servers.is_empty() {
            return Ok(PlexTracksOutcome::NoServer);
        }
        if servers.len() > 1 {
            return Ok(PlexTracksOutcome::MultipleServers(servers.len()));
        }

        let server = servers.into_iter().next().unwrap();

        let access_token = match &server.access_token {
            Some(token) => token.clone(),
            None => return Ok(PlexTracksOutcome::NoToken),
        };

        let server_url = match Url::parse(&server.server_url) {
            Ok(url) => url,
            Err(e) => {
                return Ok(PlexTracksOutcome::Error(format!(
                    "Invalid server URL: {}",
                    e
                )));
            }
        };

        let sections = match self
            .client
            .get_library_sections(&server_url, &access_token)
            .await
        {
            Ok(s) => s,
            Err(e) => {
                return Ok(PlexTracksOutcome::Error(format!(
                    "Failed to fetch library sections: {}",
                    e
                )));
            }
        };

        let music_section_id = match self.client.find_music_section_id(&sections) {
            Some(id) => id,
            None => {
                return Ok(PlexTracksOutcome::Error(
                    "No music library section found on Plex server.".to_string(),
                ));
            }
        };

        match self
            .client
            .get_tracks_page(&server_url, &access_token, &music_section_id, 0, 50)
            .await
        {
            Ok(container) => Ok(PlexTracksOutcome::Success(container)),
            Err(e) => Ok(PlexTracksOutcome::Error(format!(
                "Failed to fetch tracks: {}",
                e
            ))),
        }
    }

    pub async fn refresh_music_library(&self, server_id: i64) -> color_eyre::Result<String> {
        let (_server, server_url, access_token) = self.resolve_server_by_id(server_id).await?;
        let music_section_id = self
            .resolve_music_section(&server_url, &access_token)
            .await?;

        self.client
            .refresh_library_section(&server_url, &access_token, &music_section_id)
            .await?;

        Ok(music_section_id)
    }

    pub async fn get_scan_status(
        &self,
        server_id: i64,
    ) -> color_eyre::Result<Option<PlexActivity>> {
        let (_server, server_url, access_token) = self.resolve_server_by_id(server_id).await?;
        let music_section_id = self
            .resolve_music_section(&server_url, &access_token)
            .await?;

        self.client
            .get_library_scan_status(&server_url, &access_token, &music_section_id)
            .await
    }

    // ---- Playlists ----

    pub async fn get_playlists(&self) -> color_eyre::Result<Vec<PlexPlaylist>> {
        let (_server, server_url, access_token) = self.resolve_server().await?;
        let playlists = self
            .client
            .get_playlists(&server_url, &access_token)
            .await?;
        Ok(playlists)
    }

    pub async fn sync_playlist(
        &self,
        playlist_id: i64,
    ) -> color_eyre::Result<crate::plex_rs::sync_playlist::SyncPlaylistResult> {
        // Delegate to existing function (it mixes DB + API calls; decompose later)
        let client = reqwest::Client::new();
        crate::plex_rs::sync_playlist::sync_playlist_to_plex(&self.db, &client, playlist_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plex_rs::auth::PlexPinResponse;
    use crate::ports::plex::MockPlexClient;
    use crate::test_utils::test_db;

    #[tokio::test]
    async fn test_list_servers_empty() {
        let db = test_db().await;
        let client = MockPlexClient::new();
        let service = PlexService::new(db, client);

        let servers = service.list_servers().await.unwrap();
        assert!(servers.is_empty());
    }

    #[tokio::test]
    async fn test_list_servers() {
        let db = test_db().await;
        let client = MockPlexClient::new();
        let service = PlexService::new(db, client);

        service
            .create_server("Server1".into(), "http://localhost:32400".into())
            .await
            .unwrap();
        service
            .create_server("Server2".into(), "http://localhost:32401".into())
            .await
            .unwrap();

        let servers = service.list_servers().await.unwrap();
        assert_eq!(servers.len(), 2);
    }

    #[tokio::test]
    async fn test_create_server() {
        let db = test_db().await;
        let client = MockPlexClient::new();
        let service = PlexService::new(db, client);

        let server = service
            .create_server("MyPlex".into(), "http://192.168.1.100:32400".into())
            .await
            .unwrap();

        assert_eq!(server.name, "MyPlex");
        assert_eq!(server.server_url, "http://192.168.1.100:32400");
        assert!(server.access_token.is_none());
    }

    #[tokio::test]
    async fn test_create_server_invalid_url() {
        let db = test_db().await;
        let client = MockPlexClient::new();
        let service = PlexService::new(db, client);

        let result = service
            .create_server("Bad".into(), "not-a-url".into())
            .await;

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid server URL")
        );
    }

    #[tokio::test]
    async fn test_start_authentication() {
        let db = test_db().await;
        let mut client = MockPlexClient::new();

        client.expect_create_pin().returning(|| {
            Ok(PlexPinResponse {
                id: 42,
                code: "ABCD1234".to_string(),
            })
        });
        client
            .expect_construct_auth_url()
            .returning(|_code, _forward_url| Ok("https://plex.tv/auth?code=ABCD1234".to_string()));

        let service = PlexService::new(db, client);

        // First create a server so start_authentication can find it
        let server = service
            .create_server("TestServer".into(), "http://localhost:32400".into())
            .await
            .unwrap();

        let (auth_url, pin_id) = service
            .start_authentication(server.id, "http://localhost:3001")
            .await
            .unwrap();

        assert_eq!(pin_id, 42);
        assert!(auth_url.contains("plex.tv"));
    }

    #[tokio::test]
    async fn test_get_tracks_no_server() {
        let db = test_db().await;
        let client = MockPlexClient::new();
        let service = PlexService::new(db, client);

        let result = service.get_tracks().await.unwrap();
        assert!(matches!(result, PlexTracksOutcome::NoServer));
    }
}
