use std::path::PathBuf;

use color_eyre::{Result, eyre::Context};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The directory to store the music
    directory: String,
    /// The path to store the sqlite database
    database_path: String,
}

impl Config {
    pub fn create_default() -> Result<Self> {
        log::debug!("Creating default config");

        let path = Self::config_path().ok_or(color_eyre::eyre::eyre!("Config file not found"))?;

        if path.exists() {
            log::error!("Config file already exists at: {}", path.display());
            return Err(color_eyre::eyre::eyre!("Config file already exists"));
        }

        log::debug!("Creating config directory: {:?}", path.parent());
        std::fs::create_dir_all(
            path.parent()
                .ok_or(color_eyre::eyre::eyre!("Config file not found"))?,
        )?;

        std::fs::write(
            &path,
            toml::to_string(&Self {
                directory: dirs::audio_dir()
                    .ok_or(color_eyre::eyre::eyre!("Music directory not found"))?
                    .join("music-organizer")
                    .as_os_str()
                    .to_str()
                    .ok_or(color_eyre::eyre::eyre!("Music directory not found"))?
                    .to_string(),
                database_path: dirs::audio_dir()
                    .ok_or(color_eyre::eyre::eyre!("Music directory not found"))?
                    .join("music-organizer/library.db")
                    .as_os_str()
                    .to_str()
                    .ok_or(color_eyre::eyre::eyre!("Music directory not found"))?
                    .to_string(),
            })?,
        )?;

        log::info!("Default config created at: {}", path.display());
        Self::load()
    }

    /// Load config from a TOML file
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        log::debug!("Loading config from: {}", path.display());

        let contents = std::fs::read_to_string(path)
            .context(format!("Failed to read config file: {}", path.display()))?;
        let config: Config = toml::from_str(&contents)
            .context(format!("Failed to parse config file: {}", path.display()))?;

        log::info!("Config loaded successfully from: {}", path.display());
        Ok(config)
    }

    /// Get the config file path (similar to beets)
    pub fn config_path() -> Option<PathBuf> {
        dirs::home_dir().map(|path| path.join(".config/music-organizer").join("config.toml"))
    }

    /// Load config with default fallback
    pub fn load() -> Result<Self> {
        log::debug!("Loading config from default location");

        let config_path =
            Self::config_path().ok_or(color_eyre::eyre::eyre!("Config file not found"))?;

        Self::from_file(&config_path)
    }

    /// Expand ~ to home directory
    fn expand_path(&self, path: &str) -> PathBuf {
        if path.starts_with("~/")
            && let Some(home) = dirs::home_dir()
        {
            return home.join(&path[2..]);
        }
        PathBuf::from(path)
    }

    /// Get expanded directory path
    pub fn directory_path(&self) -> PathBuf {
        self.expand_path(&self.directory)
    }

    /// Get expanded database path
    pub fn database_path(&self) -> PathBuf {
        self.expand_path(&self.database_path)
    }
}
