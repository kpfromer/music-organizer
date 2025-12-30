use crate::config::Config;
use crate::database::Database;
use crate::soulseek::SoulSeekClientContext;
use std::path::PathBuf;

pub struct AppState {
    pub db: Database,
    pub soulseek_context: SoulSeekClientContext,
    pub download_directory: PathBuf,
    pub api_key: String,
    pub config: Config,
    pub base_url: String,
}
