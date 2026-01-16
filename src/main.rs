mod acoustid;
mod chromaprint;
mod config;
mod database;
mod entities;
mod file_hash;
mod http_server;
mod import_track;
mod logging;
mod migrator;
mod musicbrainz;
mod plex_rs;
mod soulseek;
mod soulseek_tui;

use std::path::PathBuf;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use color_eyre::{Result, eyre::Context};

use crate::{
    config::Config,
    database::Database,
    http_server::app::HttpServerConfig,
    import_track::{import_folder, import_track, watch_directory},
    logging::setup_logging,
    soulseek::{SearchConfig, SoulSeekClientContext},
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The config file to use
    #[arg(short, long, env = "MUSIC_MANAGER_CONFIG")]
    config: Option<PathBuf>,

    /// Console log level (default: off)
    #[arg(long, default_value = "off", global = true, env = "LOG_LEVEL")]
    log_level: log::LevelFilter,

    /// File log level (default: debug)
    #[arg(long, default_value = "debug", global = true)]
    log_file_level: log::LevelFilter,

    /// Path to log file
    #[arg(long, env = "MUSIC_MANAGER_LOG_FILE", global = true)]
    log_file: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

fn is_directory(s: &str) -> Result<PathBuf, String> {
    let p: PathBuf = s.into();
    if p.is_dir() {
        Ok(p)
    } else {
        Err(format!("`{}` is not an existing directory", s))
    }
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Import folder/file into the music database
    Import {
        /// The folder/file to import
        #[arg(short, long)]
        input: PathBuf,

        /// AcoustID API key for lookups
        #[arg(short = 'k', long = "api-key", env = "ACOUSTID_API_KEY")]
        api_key: String,
    },
    /// Download music from SoulSeek
    Download {
        /// The soulseek username to use for downloads
        #[arg(short, long)]
        username: String,

        /// The soulseek password to use for downloads
        #[arg(short, long)]
        password: String,

        /// The directory to download to
        #[arg(short, long, value_parser = is_directory)]
        output_directory: PathBuf,
    },
    /// Watch a directory for new music files
    Watch {
        /// The directory to watch for new music files
        #[arg(short, long, value_parser = is_directory, env = "MUSIC_MANAGER_WATCH_DIRECTORY")]
        directory: PathBuf,

        /// AcoustID API key for lookups
        #[arg(short = 'k', long = "api-key", env = "ACOUSTID_API_KEY")]
        api_key: String,
    },
    /// Serve the HTTP server
    Serve {
        /// The port to run the server on
        #[arg(short, long, default_value = "3000", env = "MUSIC_MANAGER_HTTP_PORT")]
        port: u16,

        /// The directory to watch for new music files
        #[arg(short, long, value_parser = is_directory, env = "MUSIC_MANAGER_WATCH_DIRECTORY")]
        directory: PathBuf,

        /// AcoustID API key for lookups
        #[arg(short = 'k', long = "api-key", env = "ACOUSTID_API_KEY")]
        api_key: String,

        /// SoulSeek username for searching and downloading
        #[arg(long, env = "SOULSEEK_USERNAME")]
        soulseek_username: String,

        /// SoulSeek password for searching and downloading
        #[arg(long, env = "SOULSEEK_PASSWORD")]
        soulseek_password: String,

        /// Directory to download SoulSeek files to
        #[arg(long, value_parser = is_directory, env = "SOULSEEK_DOWNLOAD_DIRECTORY")]
        download_directory: PathBuf,

        /// Base URL for the service (used for auth redirects)
        #[arg(long, env = "BASE_URL")]
        base_url: Option<String>,
    },
    #[command(subcommand)]
    Config(ConfigCommands),
}

#[derive(Subcommand, Debug)]
enum ConfigCommands {
    /// Create a default config file, if it doesn't exist
    CreateDefault,
    /// Print the path to the config file
    Path,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();
    setup_logging(args.log_level, args.log_file.clone(), args.log_file_level)?;

    log::debug!("Music manager starting");
    log::debug!("Loading configuration");

    let config = {
        if let Some(config) = args.config {
            Config::from_file(&config)
        } else {
            Config::load()
        }
    }
    .with_context(|| "Failed to load music-manager config")?;

    log::debug!("Opening database at: {}", config.database_path().display());
    let database = Database::open(&config.database_path()).await?;

    match args.command {
        Commands::Import { input, api_key } => {
            log::debug!("Starting import command for: {}", input.display());
            if input.is_file() {
                import_track(&input, &api_key, &config, &database).await?;
            } else {
                import_folder(&input, &api_key, &config, &database).await?;
            }
            log::info!("Import command completed successfully");
        }
        Commands::Download {
            username,
            password,
            output_directory,
        } => {
            log::debug!("Starting download command with username: {}", username);
            let soulseek_context = Arc::new(
                SoulSeekClientContext::new(SearchConfig {
                    username,
                    password,
                    concurrency: Some(2),
                    searches_per_time: Some(34),
                    renew_time_secs: Some(220),
                    max_search_time_ms: Some(8000),
                    remove_special_chars: Some(true),
                })
                .await?,
            );
            crate::soulseek_tui::run(soulseek_context, output_directory).await?;
            log::info!("Download command completed successfully");
        }
        Commands::Watch { directory, api_key } => {
            log::debug!(
                "Starting watch command for directory: {}",
                directory.display()
            );
            watch_directory(&directory, &api_key, &config, &database).await?;
        }
        Commands::Config(config_commands) => match config_commands {
            ConfigCommands::CreateDefault => {
                log::debug!("Creating default config");
                Config::create_default()?;
                log::info!("Default config created successfully");
            }
            ConfigCommands::Path => match Config::config_path() {
                Some(path) => println!("{}", path.display()),
                None => println!("No default config path found"),
            },
        },
        Commands::Serve {
            port,
            directory,
            api_key,
            soulseek_username,
            soulseek_password,
            download_directory,
            base_url,
        } => {
            // Set default base_url in debug mode, require it in release mode
            let base_url = if let Some(url) = base_url {
                url
            } else if cfg!(debug_assertions) {
                "http://localhost:3001".to_string()
            } else {
                return Err(color_eyre::eyre::eyre!(
                    "BASE_URL is required in release mode. Set it via --base-url or BASE_URL environment variable"
                ));
            };
            log::info!("Starting HTTP server on port: {}", port);
            http_server::app::start(HttpServerConfig {
                port,
                database,
                config,
                acoustid_api_key: api_key,
                watch_directory_path: directory,
                soulseek_username,
                soulseek_password,
                download_directory,
                base_url,
            })
            .await?;
        }
    }

    Ok(())
}
