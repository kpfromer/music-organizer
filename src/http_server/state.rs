use crate::database::Database;
use crate::soulseek::SoulSeekClientContext;
use std::path::PathBuf;

pub struct AppState {
    pub db: Database,
    pub soulseek_context: SoulSeekClientContext,
    pub download_directory: PathBuf,
}
