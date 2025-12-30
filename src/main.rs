mod acoustid;
mod chromaprint;
mod config;
mod database;
mod entities;
mod file_hash;
mod http_server;
mod import_track;
mod logging;
mod musicbrainz;
mod plex_rs;
mod soulseek;
mod soulseek_tui;

use std::path::PathBuf;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use color_eyre::{
    Result,
    eyre::{Context, OptionExt},
};
use reqwest::Client;
use url::Url;

use crate::{
    config::Config,
    database::Database,
    http_server::app::HttpServerConfig,
    import_track::{import_folder, import_track, watch_directory},
    logging::setup_logging,
    plex_rs::{
        PlexAuthResponse, construct_auth_app_url, create_plex_pin, get_plex_playlists,
        get_plex_resources,
        playlist::{get_playlist_tracks, get_playlists, is_music_playlist},
        poll_for_plex_auth,
    },
    soulseek::{SearchConfig, SoulSeekClientContext},
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The config file to use
    #[arg(short, long, env = "MUSIC_MANAGER_CONFIG")]
    config: Option<PathBuf>,

    /// Console log level (default: off)
    #[arg(long, default_value = "off", global = true)]
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
    },
    #[command(subcommand)]
    Config(ConfigCommands),
    /// Test out plex authentication
    Plex {
        /// The name of the Plex server to use
        #[arg(long, env = "PLEX_SERVER_NAME")]
        plex_server_name: String,

        /// The URL of the Plex server to use
        #[arg(long, env = "PLEX_SERVER_URL")]
        plex_server_url: Url,
    },
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
        } => {
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
            })
            .await?;
        }
        Commands::Plex {
            plex_server_name,
            plex_server_url,
        } => {
            log::debug!("Starting plex command");
            let client = Client::new();
            let pin = create_plex_pin(&client).await?;
            log::info!("Plex pin created: {}", pin.code);
            let url = construct_auth_app_url(&pin.code)?;
            println!("Open this URL in your browser: {}", url);
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            async fn poll_for_plex_auth_loop(client: &Client, pin_id: i32) -> Result<String> {
                for _ in 0..10 {
                    let auth_response = poll_for_plex_auth(client, pin_id).await;
                    log::info!("Plex auth response: {:?}", auth_response);
                    match auth_response {
                        Ok(PlexAuthResponse {
                            auth_token: Some(user_token),
                        }) => {
                            log::info!("User token: {}", user_token);
                            return Ok(user_token);
                        }
                        Ok(PlexAuthResponse { auth_token: None }) => {
                            log::info!("No user token found");
                        }
                        Err(_) => {
                            // TODO: use thiserror
                            log::error!("Error polling for plex auth");
                        }
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
                Err(color_eyre::eyre::eyre!("Failed to poll for plex auth"))
            }
            let user_token = poll_for_plex_auth_loop(&client, pin.id).await?;
            let plex_resources = get_plex_resources(&client, &user_token).await?;
            let plex_server = plex_resources
                .into_iter()
                .find(|resource| resource.name == plex_server_name)
                .ok_or_eyre("Plex server not found")?;
            let access_token = plex_server
                .access_token
                .ok_or_eyre("No access token found")?;
            let plex_playlists =
                get_plex_playlists(&client, &plex_server_url, &access_token).await?;
            println!("Plex playlists");
            println!("{:#?}", plex_playlists);

            // 1. Fetch all playlists
            let playlists = get_playlists(&client, &plex_server_url, &access_token).await?;

            // 2. Pick the first music playlist
            let playlist = playlists
                .iter()
                .find(|p| is_music_playlist(p))
                .ok_or_eyre("No music playlists found")?;

            println!("Playlist: {} (id={})", playlist.title, playlist.rating_key);

            // 3. Fetch tracks for that playlist
            let tracks = get_playlist_tracks(
                &client,
                &plex_server_url,
                &access_token,
                &playlist.rating_key,
            )
            .await?;

            // 4. Print tracks
            for (idx, track) in tracks.iter().enumerate() {
                println!(
                    "{:>3}. {} – {} ({})",
                    idx + 1,
                    track.artist.as_deref().unwrap_or("Unknown Artist"),
                    track.title,
                    track.album.as_deref().unwrap_or("Unknown Album"),
                );
            }
        }
    }

    Ok(())
}
