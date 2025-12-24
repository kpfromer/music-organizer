use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::database::Database;
use crate::soulseek::SoulSeekClientContext;

pub struct AppState {
    pub db: Database,
    pub soulseek_context: Arc<Mutex<SoulSeekClientContext>>,
    pub download_directory: PathBuf,
}
