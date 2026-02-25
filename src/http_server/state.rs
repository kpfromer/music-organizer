use spotify_rs::{AuthCodeFlow, Unauthenticated, client::Client as SpotifyClient};
use tokio::sync::{Mutex, Notify};

use crate::config::Config;
use crate::database::Database;
use crate::services::spotify::client::SpotifyApiCredentials;
use crate::soulseek::SoulSeekClientContext;
use std::path::PathBuf;
use std::sync::Arc;

pub struct AppState {
    pub db: Arc<Database>,
    pub soulseek_context: Arc<SoulSeekClientContext>,
    pub download_directory: PathBuf,
    pub api_key: String,
    pub config: Config,
    pub base_url: String,
    pub spotify_credentials: Option<SpotifyApiCredentials>,
    pub spotify_oauth_session: Mutex<Option<SpotifyClient<Unauthenticated, AuthCodeFlow>>>,
    pub wishlist_notify: Arc<Notify>,
}
