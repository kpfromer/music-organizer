//! Env-var loader for music-manager.
//!
//! [`Config::from_env`] is the single place in the workspace that calls
//! `std::env::var`. CI enforces this via `scripts/check-env-source.sh`.
//!
//! Validation happens here and is loud: a missing required variable or a
//! malformed value aborts startup with a clear error rather than letting the
//! server reach an inconsistent state at request time.

use std::env;
use std::fmt;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;

mod error;

pub use error::ConfigError;

/// All runtime configuration, loaded once at startup.
#[derive(Clone)]
pub struct Config {
    pub db_path: PathBuf,
    pub music_root: PathBuf,
    pub watch_folder: PathBuf,
    pub listen_addr: SocketAddr,
    pub acoustid_api_key: String,
    pub musicbrainz_user_agent: String,
    pub soulseek_username: String,
    pub soulseek_password: Secret,
    pub transcode_target: String,
    pub import_workers: usize,
    pub log: String,
    pub log_format: LogFormat,
    pub graphql_playground: bool,
    pub frontend_dir: Option<PathBuf>,
}

/// Wraps a string so its `Debug` impl never leaks the value.
#[derive(Clone, PartialEq, Eq)]
pub struct Secret(String);

impl Secret {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(<redacted>)")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogFormat {
    Text,
    Json,
}

impl FromStr for LogFormat {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "text" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            other => Err(ConfigError::Invalid {
                var: "MM_LOG_FORMAT",
                reason: format!("expected 'text' or 'json', got {other:?}"),
            }),
        }
    }
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("db_path", &self.db_path)
            .field("music_root", &self.music_root)
            .field("watch_folder", &self.watch_folder)
            .field("listen_addr", &self.listen_addr)
            .field("acoustid_api_key", &"<redacted>")
            .field("musicbrainz_user_agent", &self.musicbrainz_user_agent)
            .field("soulseek_username", &self.soulseek_username)
            .field("soulseek_password", &self.soulseek_password)
            .field("transcode_target", &self.transcode_target)
            .field("import_workers", &self.import_workers)
            .field("log", &self.log)
            .field("log_format", &self.log_format)
            .field("graphql_playground", &self.graphql_playground)
            .field("frontend_dir", &self.frontend_dir)
            .finish()
    }
}

impl Config {
    /// Load the full configuration from process env. Validates required vars
    /// and ranges; returns a `ConfigError` describing the first problem found.
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_loader(&EnvLoader)
    }

    fn from_loader(loader: &dyn VarLoader) -> Result<Self, ConfigError> {
        let db_path = path_or_default(loader, "MM_DB_PATH", "./music-manager.db");
        let music_root = required_path(loader, "MM_MUSIC_ROOT")?;
        let watch_folder = match loader.get("MM_WATCH_FOLDER") {
            Some(v) if !v.is_empty() => PathBuf::from(v),
            _ => music_root.join("_inbox"),
        };
        let listen_addr = parse_or_default(loader, "MM_LISTEN_ADDR", "127.0.0.1:8080")?;
        let acoustid_api_key = required_string(loader, "MM_ACOUSTID_API_KEY")?;
        let musicbrainz_user_agent = required_string(loader, "MM_MUSICBRAINZ_USER_AGENT")?;
        validate_user_agent(&musicbrainz_user_agent)?;
        let soulseek_username = required_string(loader, "MM_SOULSEEK_USERNAME")?;
        let soulseek_password = Secret::new(required_string(loader, "MM_SOULSEEK_PASSWORD")?);
        let transcode_target = string_or_default(loader, "MM_TRANSCODE_TARGET", "opus@128k");
        let import_workers = parse_or_default::<usize>(loader, "MM_IMPORT_WORKERS", "2")?;
        if import_workers == 0 {
            return Err(ConfigError::Invalid {
                var: "MM_IMPORT_WORKERS",
                reason: "must be at least 1".to_owned(),
            });
        }
        let log = string_or_default(loader, "MM_LOG", "info");
        let log_format = string_or_default(loader, "MM_LOG_FORMAT", "text").parse::<LogFormat>()?;
        let graphql_playground = parse_bool(loader, "MM_GRAPHQL_PLAYGROUND")?;
        let frontend_dir = loader
            .get("MM_FRONTEND_DIR")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from);

        Ok(Self {
            db_path,
            music_root,
            watch_folder,
            listen_addr,
            acoustid_api_key,
            musicbrainz_user_agent,
            soulseek_username,
            soulseek_password,
            transcode_target,
            import_workers,
            log,
            log_format,
            graphql_playground,
            frontend_dir,
        })
    }
}

fn validate_user_agent(ua: &str) -> Result<(), ConfigError> {
    if ua.contains("you@example.com") || ua.contains("https://example.com") {
        return Err(ConfigError::Invalid {
            var: "MM_MUSICBRAINZ_USER_AGENT",
            reason: "must be customised — MusicBrainz requires a real contact in the UA".to_owned(),
        });
    }
    if !ua.contains('/') {
        return Err(ConfigError::Invalid {
            var: "MM_MUSICBRAINZ_USER_AGENT",
            reason: "expected `name/version (contact)` shape".to_owned(),
        });
    }
    Ok(())
}

trait VarLoader {
    fn get(&self, key: &str) -> Option<String>;
}

struct EnvLoader;

impl VarLoader for EnvLoader {
    fn get(&self, key: &str) -> Option<String> {
        env::var(key).ok()
    }
}

fn required_string(loader: &dyn VarLoader, var: &'static str) -> Result<String, ConfigError> {
    match loader.get(var) {
        Some(v) if !v.is_empty() => Ok(v),
        _ => Err(ConfigError::Missing { var }),
    }
}

fn required_path(loader: &dyn VarLoader, var: &'static str) -> Result<PathBuf, ConfigError> {
    required_string(loader, var).map(PathBuf::from)
}

fn string_or_default(loader: &dyn VarLoader, var: &str, default: &str) -> String {
    loader
        .get(var)
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| default.to_owned())
}

fn path_or_default(loader: &dyn VarLoader, var: &str, default: &str) -> PathBuf {
    PathBuf::from(string_or_default(loader, var, default))
}

fn parse_or_default<T>(
    loader: &dyn VarLoader,
    var: &'static str,
    default: &str,
) -> Result<T, ConfigError>
where
    T: FromStr,
    T::Err: fmt::Display,
{
    let raw = string_or_default(loader, var, default);
    raw.parse().map_err(|e: T::Err| ConfigError::Invalid {
        var,
        reason: format!("could not parse {raw:?}: {e}"),
    })
}

fn parse_bool(loader: &dyn VarLoader, var: &'static str) -> Result<bool, ConfigError> {
    match loader.get(var).as_deref() {
        None | Some("") | Some("0") | Some("false") | Some("no") => Ok(false),
        Some("1") | Some("true") | Some("yes") => Ok(true),
        Some(other) => Err(ConfigError::Invalid {
            var,
            reason: format!("expected boolean (0/1/true/false), got {other:?}"),
        }),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    struct Map(HashMap<&'static str, &'static str>);

    impl VarLoader for Map {
        fn get(&self, key: &str) -> Option<String> {
            self.0.get(key).map(|s| (*s).to_owned())
        }
    }

    fn full() -> Map {
        let mut m = HashMap::new();
        m.insert("MM_MUSIC_ROOT", "/music");
        m.insert("MM_ACOUSTID_API_KEY", "abc123def456ghi789");
        m.insert("MM_MUSICBRAINZ_USER_AGENT", "music-manager/0.1 (kyle@home)");
        m.insert("MM_SOULSEEK_USERNAME", "u");
        m.insert("MM_SOULSEEK_PASSWORD", "p");
        Map(m)
    }

    #[test]
    fn test_from_env_minimum_required_succeeds() {
        let c = Config::from_loader(&full()).expect("config");
        assert_eq!(c.music_root, PathBuf::from("/music"));
        assert_eq!(c.watch_folder, PathBuf::from("/music/_inbox"));
        assert_eq!(c.import_workers, 2);
        assert!(!c.graphql_playground);
        assert_eq!(c.log_format, LogFormat::Text);
    }

    #[test]
    fn test_from_env_missing_music_root_errors() {
        let mut m = full().0;
        m.remove("MM_MUSIC_ROOT");
        let err = Config::from_loader(&Map(m)).unwrap_err();
        assert!(matches!(
            err,
            ConfigError::Missing {
                var: "MM_MUSIC_ROOT"
            }
        ));
    }

    #[test]
    fn test_from_env_default_user_agent_rejected() {
        let mut m = full().0;
        m.insert(
            "MM_MUSICBRAINZ_USER_AGENT",
            "music-manager/0.1 ( https://example.com/contact )",
        );
        let err = Config::from_loader(&Map(m)).unwrap_err();
        assert!(matches!(
            err,
            ConfigError::Invalid {
                var: "MM_MUSICBRAINZ_USER_AGENT",
                ..
            }
        ));
    }

    #[test]
    fn test_from_env_workers_zero_rejected() {
        let mut m = full().0;
        m.insert("MM_IMPORT_WORKERS", "0");
        let err = Config::from_loader(&Map(m)).unwrap_err();
        assert!(matches!(
            err,
            ConfigError::Invalid {
                var: "MM_IMPORT_WORKERS",
                ..
            }
        ));
    }

    #[test]
    fn test_from_env_listen_addr_invalid() {
        let mut m = full().0;
        m.insert("MM_LISTEN_ADDR", "not-an-addr");
        let err = Config::from_loader(&Map(m)).unwrap_err();
        assert!(matches!(
            err,
            ConfigError::Invalid {
                var: "MM_LISTEN_ADDR",
                ..
            }
        ));
    }

    #[test]
    fn test_from_env_log_format_json() {
        let mut m = full().0;
        m.insert("MM_LOG_FORMAT", "json");
        let c = Config::from_loader(&Map(m)).unwrap();
        assert_eq!(c.log_format, LogFormat::Json);
    }

    #[test]
    fn test_from_env_watch_folder_override() {
        let mut m = full().0;
        m.insert("MM_WATCH_FOLDER", "/elsewhere");
        let c = Config::from_loader(&Map(m)).unwrap();
        assert_eq!(c.watch_folder, PathBuf::from("/elsewhere"));
    }

    #[test]
    fn test_from_env_graphql_playground_truthy() {
        let mut m = full().0;
        m.insert("MM_GRAPHQL_PLAYGROUND", "1");
        let c = Config::from_loader(&Map(m)).unwrap();
        assert!(c.graphql_playground);
    }

    #[test]
    fn test_secret_debug_does_not_leak() {
        let s = Secret::new("hunter2");
        assert_eq!(format!("{s:?}"), "Secret(<redacted>)");
        assert_eq!(s.expose(), "hunter2");
    }

    #[test]
    fn test_config_debug_redacts_secrets() {
        let c = Config::from_loader(&full()).unwrap();
        let s = format!("{c:?}");
        assert!(s.contains("<redacted>"));
        assert!(!s.contains("abc123def456ghi789"));
        assert!(!s.contains("p\""));
    }
}
