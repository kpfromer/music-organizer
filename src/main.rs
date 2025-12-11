mod acoustid;
mod chromaprint;
mod config;
mod database;
mod file_hash;
mod import_track;
mod musicbrainz;
mod soulseek;
mod soulseek_tui;

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use crate::{
    config::Config,
    database::Database,
    import_track::{import_folder, import_track},
    soulseek::{SearchConfig, SoulSeekClientContext},
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The config file to use
    #[arg(short, long)]
    config: Option<PathBuf>,

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
    Import {
        /// The folder/file to import
        #[arg(short, long)]
        input: PathBuf,

        /// AcoustID API key for lookups
        #[arg(short = 'k', long = "api-key", env = "ACOUSTID_API_KEY")]
        api_key: String,
    },
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
    let args = Args::parse();
    let config = {
        if let Some(config) = args.config {
            Config::from_file(&config)
        } else {
            Config::load()
        }
    }
    .with_context(|| "Failed to load music-manager config")?;
    let database = Database::open(&config.library_path())?;

    match args.command {
        Commands::Import { input, api_key } => {
            if input.is_file() {
                import_track(&input, &api_key, &config, &database).await?;
            } else {
                import_folder(&input, &api_key, &config, &database).await?;
            }
        }
        Commands::Download {
            username,
            password,
            output_directory,
        } => {
            let soulseek_context = SoulSeekClientContext::new(SearchConfig {
                username,
                password,
                concurrency: Some(2),
                searches_per_time: Some(34),
                renew_time_secs: Some(220),
                max_search_time_ms: Some(8000),
                remove_special_chars: Some(true),
            })
            .await?;
            crate::soulseek_tui::run(soulseek_context, output_directory).await?;
        }
        Commands::Config(config_commands) => match config_commands {
            ConfigCommands::CreateDefault => {
                Config::create_default()?;
            }
            ConfigCommands::Path => match Config::config_path() {
                Some(path) => println!("{}", path.display()),
                None => println!("No default config path found"),
            },
        },
    }

    Ok(())
}
