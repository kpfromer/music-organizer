use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    directory: String,
    library: String,
    #[serde(default)]
    soulseek: Option<SoulSeekConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoulSeekConfig {
    pub username: String,
    pub password: String,
    pub output_directory: String,
}

impl Config {
    /// Load config from a TOML file
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .context(format!("Failed to read config file: {}", path.display()))?;
        let config: Config = toml::from_str(&contents)
            .context(format!("Failed to parse config file: {}", path.display()))?;
        Ok(config)
    }

    /// Get the config file path (similar to beets)
    fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|path| path.join("music-organizer").join("config.toml"))
    }

    /// Load config with default fallback
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path().ok_or(anyhow::anyhow!("Config file not found"))?;

        Self::from_file(&config_path)
    }

    /// Expand ~ to home directory
    fn expand_path(&self, path: &str) -> PathBuf {
        if path.starts_with("~/") {
            if let Some(home) = dirs::home_dir() {
                return home.join(&path[2..]);
            }
        }
        PathBuf::from(path)
    }

    /// Get expanded directory path
    pub fn directory_path(&self) -> PathBuf {
        self.expand_path(&self.directory)
    }

    /// Get expanded library path
    pub fn library_path(&self) -> PathBuf {
        self.expand_path(&self.library)
    }

    /// Get SoulSeek config or return defaults
    pub fn soulseek_config(&self) -> SoulSeekConfig {
        if let Some(ref ss_config) = self.soulseek {
            ss_config.clone()
        } else {
            // Try environment variables as fallback
            let username = std::env::var("SOULSEEK_USERNAME").unwrap_or_else(|_| "".to_string());
            let password = std::env::var("SOULSEEK_PASSWORD").unwrap_or_else(|_| "".to_string());
            let output_directory = self.directory_path().to_string_lossy().to_string();

            SoulSeekConfig {
                username,
                password,
                output_directory,
            }
        }
    }

    /// Get output directory for downloads
    pub fn download_output_directory(&self) -> PathBuf {
        if let Some(ref ss_config) = self.soulseek {
            self.expand_path(&ss_config.output_directory)
        } else {
            self.directory_path()
        }
    }
}
