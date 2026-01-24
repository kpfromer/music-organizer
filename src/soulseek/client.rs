// TODO: Remove this once we have a proper API
#![allow(dead_code)]

use std::collections::HashMap;
use std::num::NonZeroU32;
use std::path::Path;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::{Duration, Instant};
use tracing;

use color_eyre::{Result, eyre::Context};
use futures::future::join_all;
use governor::{
    Quota, RateLimiter, clock::DefaultClock, state::InMemoryState, state::direct::NotKeyed,
};
use regex::Regex;
use soulseek_rs::client::Client as SoulseekClient;
use tokio::sync::{Mutex, Semaphore};
use unaccent::unaccent;

use crate::soulseek::types::{FileAttribute, SearchConfig, SingleFileResult, Track};

// ============================================================================
// Types
// ============================================================================

type DirectRateLimiter = RateLimiter<NotKeyed, InMemoryState, DefaultClock>;

#[derive(Debug, Clone)]
pub struct FileSearchResponse {
    pub username: String,
    pub token: String,
    pub files: Vec<FileInfo>,
    pub slots_free: bool,
    pub avg_speed: f64,
    pub queue_length: u32,
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub filename: String,
    pub size: u64,
    pub attrs: HashMap<u8, u32>,
}

// ============================================================================
// Session State
// ============================================================================

/// Replaces `logged_in: bool` with a real session lifecycle.
#[derive(Debug, Clone)]
enum SessionState {
    Disconnected { last_error: Option<String> },
    Connecting,
    LoggedIn { since: Instant },
    Backoff { until: Instant, last_error: String },
}

// ============================================================================
// Client Wrapper (async/sync boundary)
// ============================================================================

/// Wrapper stores the sync client behind a std::sync::Mutex so we can lock it inside spawn_blocking.
/// The outer tokio::Mutex protects swapping Some/None in async code.
pub struct SoulSeekClientWrapper {
    client: Mutex<Option<Arc<StdMutex<SoulseekClient>>>>,
}

impl SoulSeekClientWrapper {
    pub fn new() -> Self {
        Self {
            client: Mutex::new(None),
        }
    }

    pub async fn clear_client(&self) {
        *self.client.lock().await = None;
    }

    pub async fn get_client_arc(&self) -> Option<Arc<StdMutex<SoulseekClient>>> {
        self.client.lock().await.clone()
    }

    async fn set_client(&self, c: SoulseekClient) {
        *self.client.lock().await = Some(Arc::new(StdMutex::new(c)));
    }

    /// Connect + login (blocking) and store client.
    pub async fn login(&self, username: &str, password: &str) -> Result<()> {
        let username = username.to_string();
        let password = password.to_string();

        soulseek_rs::utils::logger::enable_buffering();

        let client = tokio::task::spawn_blocking(move || -> Result<SoulseekClient> {
            let mut client = SoulseekClient::new(&username, &password);
            client.connect();
            client.login()?;
            Ok(client)
        })
        .await??;

        self.set_client(client).await;
        Ok(())
    }

    /// Run a search in a blocking thread so tokio workers don't get stuck.
    pub async fn search(&self, query: &str, timeout: Duration) -> Result<Vec<FileSearchResponse>> {
        let client_arc = self
            .get_client_arc()
            .await
            .ok_or_else(|| color_eyre::eyre::eyre!("Client not initialized"))?;

        let query = query.to_string();

        let search_results =
            tokio::task::spawn_blocking(move || -> Result<Vec<soulseek_rs::SearchResult>> {
                let client = client_arc
                    .lock()
                    .map_err(|_| color_eyre::eyre::eyre!("Soulseek client mutex poisoned"))?;

                let results = client.search(&query, timeout)?;
                Ok(results)
            })
            .await??;

        Ok(convert_search_results(search_results))
    }
}

fn convert_search_results(
    search_results: Vec<soulseek_rs::SearchResult>,
) -> Vec<FileSearchResponse> {
    let mut responses = Vec::new();

    for result in search_results {
        let files: Vec<FileInfo> = result
            .files
            .iter()
            .map(|f| {
                let mut attrs = HashMap::new();
                for (k, v) in &f.attribs {
                    if *k <= u8::MAX as u32 {
                        attrs.insert(*k as u8, *v);
                    }
                }
                FileInfo {
                    filename: f.name.clone(),
                    size: f.size,
                    attrs,
                }
            })
            .collect();

        responses.push(FileSearchResponse {
            username: result.username.clone(),
            token: result.token.to_string(),
            files,
            slots_free: result.slots > 0,
            avg_speed: result.speed as f64,
            queue_length: 0,
        });
    }

    responses
}

// ============================================================================
// Search Helper Functions
// ============================================================================

fn remove_diacritics(str: &str) -> String {
    unaccent(str)
}

fn clean_search_string(str: &str, remove_special: bool) -> String {
    let mut out = str.trim().to_string();
    if remove_special {
        // Remove special characters, keep letters, numbers, spaces
        out = Regex::new(r"[^\p{L}\p{N}\s]")
            .unwrap()
            .replace_all(&out, " ")
            .to_string();
        // Collapse multiple spaces
        out = Regex::new(r"\s+")
            .unwrap()
            .replace_all(&out, " ")
            .to_string();
    }
    out.trim().to_string()
}

fn build_search_queries(track: &Track, remove_special: bool) -> Vec<String> {
    let title = &track.title;
    let album = &track.album;
    let artist_str = track.artists.join(" ");

    let base_query = if !artist_str.is_empty() && !title.is_empty() {
        format!("{} {}", artist_str, title)
    } else {
        String::new()
    };

    let with_album = if !artist_str.is_empty() && !title.is_empty() && !album.is_empty() {
        format!("{} {} {}", artist_str, title, album)
    } else {
        String::new()
    };

    let mut queries = std::collections::HashSet::new();

    // Normal
    if !base_query.is_empty() {
        queries.insert(clean_search_string(&base_query, remove_special));
    }
    if !with_album.is_empty() {
        queries.insert(clean_search_string(&with_album, remove_special));
    }

    // Diacritics removed
    if !base_query.is_empty() {
        queries.insert(clean_search_string(
            &remove_diacritics(&base_query),
            remove_special,
        ));
    }
    if !with_album.is_empty() {
        queries.insert(clean_search_string(
            &remove_diacritics(&with_album),
            remove_special,
        ));
    }

    queries.into_iter().collect()
}

fn to_file_attributes(attrs: &HashMap<u8, u32>) -> HashMap<FileAttribute, u32> {
    let mut result = HashMap::new();
    for (key, value) in attrs {
        if let Some(attr) = match *key {
            0 => Some(FileAttribute::Bitrate),
            1 => Some(FileAttribute::Duration),
            2 => Some(FileAttribute::VariableBitRate),
            3 => Some(FileAttribute::Encoder),
            4 => Some(FileAttribute::SampleRate),
            5 => Some(FileAttribute::BitDepth),
            _ => None,
        } {
            result.insert(attr, *value);
        }
    }
    result
}

fn flatten_search_response(resp: &FileSearchResponse) -> Vec<SingleFileResult> {
    resp.files
        .iter()
        .map(|f| SingleFileResult {
            username: resp.username.clone(),
            token: resp.token.clone(),
            filename: f.filename.clone(),
            size: f.size,
            slots_free: resp.slots_free,
            avg_speed: resp.avg_speed,
            queue_length: resp.queue_length,
            attrs: to_file_attributes(&f.attrs),
        })
        .collect()
}

fn is_audio_file(filename: &str) -> bool {
    let audio_extensions = [
        ".mp3", ".flac", ".wav", ".aac", ".ogg", ".m4a", ".wma", ".aiff", ".alac", ".opus", ".ape",
    ];
    let lower = filename.to_lowercase();
    audio_extensions.iter().any(|ext| lower.ends_with(ext))
}

fn rank_results(track: &Track, results: &mut [SingleFileResult]) {
    let requested_title_lc = track.title.to_lowercase();
    let requested_artist_lc = track.artists.join(" ").to_lowercase();

    results.sort_by(|a, b| {
        let a_filename_lc = a.filename.to_lowercase();
        let b_filename_lc = b.filename.to_lowercase();

        let a_has_artist = requested_artist_lc
            .split_whitespace()
            .any(|word| a_filename_lc.contains(word));
        let b_has_artist = requested_artist_lc
            .split_whitespace()
            .any(|word| b_filename_lc.contains(word));
        let a_has_title = requested_title_lc
            .split_whitespace()
            .any(|word| a_filename_lc.contains(word));
        let b_has_title = requested_title_lc
            .split_whitespace()
            .any(|word| b_filename_lc.contains(word));

        match ((a_has_artist && a_has_title), (b_has_artist && b_has_title)) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => {
                // Secondary: avg_speed (higher is better)
                b.avg_speed
                    .partial_cmp(&a.avg_speed)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }
        }
    });
}

// ============================================================================
// Client Context (main API)
// ============================================================================

/// Main context with wrapper + limiter + config + state outside a single inner mutex.
/// This prevents "await while holding a big lock" and makes reconnect logic safe.
#[derive(Clone)]
pub struct SoulSeekClientContext {
    wrapper: Arc<SoulSeekClientWrapper>,
    rate_limiter: Arc<DirectRateLimiter>,
    config: Arc<SearchConfig>,

    state: Arc<Mutex<SessionState>>,
    session_gate: Arc<Mutex<()>>,  // serializes connect/login attempts
    backoff_secs: Arc<Mutex<u64>>, // exponential backoff
}

impl SoulSeekClientContext {
    pub async fn new(config: SearchConfig) -> Result<Self> {
        tracing::debug!("Creating SoulSeek client context");

        let searches_per_time = config.searches_per_time.unwrap_or(34);
        let renew_time_secs = config.renew_time_secs.unwrap_or(220);

        tracing::debug!(
            "Rate limiter configured: {} searches per {}s",
            searches_per_time,
            renew_time_secs
        );

        let searches_per_time_nonzero = NonZeroU32::new(searches_per_time)
            .ok_or_else(|| color_eyre::eyre::eyre!("searches_per_time must be > 0"))?;

        let quota = Quota::with_period(Duration::from_secs(renew_time_secs as u64))
            .ok_or_else(|| color_eyre::eyre::eyre!("Invalid rate limit period"))?
            .allow_burst(searches_per_time_nonzero);

        let rate_limiter = RateLimiter::direct(quota);

        tracing::info!("SoulSeek client context created successfully");

        Ok(Self {
            wrapper: Arc::new(SoulSeekClientWrapper::new()),
            rate_limiter: Arc::new(rate_limiter),
            config: Arc::new(config),

            state: Arc::new(Mutex::new(SessionState::Disconnected { last_error: None })),
            session_gate: Arc::new(Mutex::new(())),
            backoff_secs: Arc::new(Mutex::new(1)),
        })
    }

    async fn set_backoff_state(&self, err_msg: String) {
        let mut b = self.backoff_secs.lock().await;
        let wait = Duration::from_secs((*b).min(60));
        *b = (*b * 2).min(60);

        let until = Instant::now() + wait;

        *self.state.lock().await = SessionState::Backoff {
            until,
            last_error: err_msg,
        };
    }

    async fn clear_backoff(&self) {
        *self.backoff_secs.lock().await = 1;
    }

    async fn invalidate(&self, err_msg: &str) {
        tracing::warn!("Invalidating SoulSeek session: {}", err_msg);
        self.wrapper.clear_client().await;
        *self.state.lock().await = SessionState::Disconnected {
            last_error: Some(err_msg.to_string()),
        };
    }

    /// Called by consumers when they observe a terminal failure (e.g., download Failed/TimedOut).
    /// Invalidates the current session so the next operation will reconnect.
    pub async fn report_session_error(&self, reason: &str) {
        // Only invalidate if we're currently LoggedIn (avoid double-invalidate during reconnect)
        let should_invalidate = matches!(*self.state.lock().await, SessionState::LoggedIn { .. });
        if should_invalidate {
            self.invalidate(reason).await;
        }
    }

    /// Ensure we are connected+logged in. Serialized by session_gate.
    async fn ensure_session(&self) -> Result<()> {
        // Fast-path check for backoff (before acquiring gate)
        {
            let st = self.state.lock().await.clone();
            if let SessionState::Backoff { until, .. } = st
                && Instant::now() < until
            {
                return Err(color_eyre::eyre::eyre!(
                    "Backoff in effect until {:?}",
                    until
                ));
            }
        }

        let _gate = self.session_gate.lock().await;

        // Check state again after acquiring gate
        {
            let st = self.state.lock().await.clone();
            match st {
                SessionState::LoggedIn { .. } => return Ok(()),
                SessionState::Backoff { until, .. } if Instant::now() < until => {
                    return Err(color_eyre::eyre::eyre!(
                        "Backoff in effect until {:?}",
                        until
                    ));
                }
                _ => {}
            }
        }

        *self.state.lock().await = SessionState::Connecting;

        let username = self.config.username.clone();
        let password = self.config.password.clone();

        tracing::debug!("Logging in to SoulSeek as user: {}", username);

        match self.wrapper.login(&username, &password).await {
            Ok(()) => {
                tracing::info!("Successfully logged in to SoulSeek");
                *self.state.lock().await = SessionState::LoggedIn {
                    since: Instant::now(),
                };
                self.clear_backoff().await;
                Ok(())
            }
            Err(e) => {
                let msg = format!("{e:?}");
                tracing::warn!("SoulSeek login failed: {}", msg);
                self.wrapper.clear_client().await;
                self.set_backoff_state(msg).await;
                Err(e)
            }
        }
    }

    /// Run an operation with session + 1 retry on session-type failures.
    async fn with_session_retry<T, F, Fut>(&self, op_name: &str, f: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        self.ensure_session().await?;

        match f().await {
            Ok(v) => Ok(v),
            Err(e) => {
                tracing::warn!(
                    "{} failed due to session error; retrying once: {:?}",
                    op_name,
                    e
                );
                // We will always retry once, so we can invalidate the session and try again
                self.invalidate(&format!("{e:?}")).await;
                self.ensure_session().await?;
                f().await
            }
        }
    }

    /// Download a file from SoulSeek.
    /// Returns an async receiver that streams download status updates.
    pub async fn download_file(
        &self,
        result: &SingleFileResult,
        download_folder: &Path,
    ) -> Result<tokio::sync::mpsc::Receiver<soulseek_rs::DownloadStatus>> {
        tracing::debug!(
            "Starting download: '{}' from user '{}' ({} bytes)",
            result.filename,
            result.username,
            result.size
        );

        // Ensure download directory exists
        tokio::fs::create_dir_all(download_folder).await?;

        let filename = result.filename.clone();
        let username = result.username.clone();
        let size = result.size;
        let download_path = download_folder
            .to_str()
            .ok_or_else(|| color_eyre::eyre::eyre!("Download path contains invalid UTF-8"))?
            .to_string();

        let wrapper = self.wrapper.clone();

        // Get the sync receiver with retry logic
        let sync_rx = self
            .with_session_retry("download_file", || {
                let wrapper = wrapper.clone();
                let filename = filename.clone();
                let username = username.clone();
                let download_path = download_path.clone();

                async move {
                    let client_arc = wrapper
                        .get_client_arc()
                        .await
                        .ok_or_else(|| color_eyre::eyre::eyre!("SoulSeek client not available for download"))?;

                    tokio::task::spawn_blocking(move || -> Result<std::sync::mpsc::Receiver<soulseek_rs::DownloadStatus>> {
                        let client = client_arc
                            .lock()
                            .map_err(|_| color_eyre::eyre::eyre!("Soulseek client mutex poisoned"))?;

                        let receiver = client
                            .download(filename, username, size, download_path)
                            .context("Failed to download file")?;
                        Ok(receiver)
                    })
                    .await?
                }
            })
            .await?;

        // Bridge sync receiver → async channel
        let (tx, rx) = tokio::sync::mpsc::channel(64);
        tokio::task::spawn_blocking(move || {
            for status in sync_rx {
                if tx.blocking_send(status).is_err() {
                    break; // consumer dropped
                }
            }
        });

        tracing::info!(
            "Download initiated: '{}' from '{}'",
            result.filename,
            result.username
        );
        Ok(rx)
    }

    /// Search for a track on SoulSeek.
    pub async fn search_for_track(&self, track: &Track) -> Result<Vec<SingleFileResult>> {
        tracing::debug!(
            "Starting search for track: '{}' by '{}'",
            track.title,
            track.artists.join(", ")
        );

        let remove_special = self.config.remove_special_chars.unwrap_or(false);
        let concurrency = self.config.concurrency.unwrap_or(2);
        let max_search_time = self.config.max_search_time_ms.unwrap_or(8000);

        // 1) Build queries
        let queries = build_search_queries(track, remove_special);
        tracing::debug!("Built {} search queries", queries.len());

        // 2a) Concurrency limit
        let semaphore = Arc::new(Semaphore::new(concurrency));

        let tasks: Vec<_> = queries
            .into_iter()
            .map(|q| {
                let sem = semaphore.clone();
                let ctx = self.clone();

                async move {
                    let _permit = sem.acquire().await.unwrap();

                    // Rate limiting without holding any other locks
                    ctx.rate_limiter.until_ready().await;

                    tracing::debug!("Executing search query: '{}'", q);
                    let timeout = Duration::from_millis(max_search_time);

                    let responses = ctx
                        .with_session_retry("search", || {
                            let q = q.clone();
                            let wrapper = ctx.wrapper.clone();
                            async move { wrapper.search(&q, timeout).await }
                        })
                        .await?;

                    tracing::debug!(
                        "Search query '{}' returned {} responses",
                        q,
                        responses.len()
                    );

                    let mut flattened = vec![];
                    for r in responses {
                        flattened.extend(flatten_search_response(&r));
                    }

                    Result::<Vec<SingleFileResult>, color_eyre::Report>::Ok(flattened)
                }
            })
            .collect();

        let results = join_all(tasks).await;

        let mut all_flattened: Vec<SingleFileResult> = vec![];
        for result in results {
            all_flattened.extend(result?);
        }

        tracing::debug!("Total results before filtering: {}", all_flattened.len());

        // 3) Filter out non-audio file types
        tracing::debug!("Filtering audio files");
        all_flattened.retain(|f| is_audio_file(&f.filename));
        tracing::debug!("Results after audio filter: {}", all_flattened.len());

        // 4) Deduplicate (by username + filename)
        tracing::debug!("Deduplicating results");
        let mut unique_map: HashMap<String, SingleFileResult> = HashMap::new();
        for item in all_flattened {
            let key = format!("{}::{}", item.username, item.filename);
            if let Some(existing) = unique_map.get(&key) {
                // e.g. keep whichever has better avgSpeed
                if item.avg_speed > existing.avg_speed {
                    unique_map.insert(key, item);
                }
            } else {
                unique_map.insert(key, item);
            }
        }

        let mut unique_results: Vec<SingleFileResult> = unique_map.into_values().collect();
        tracing::debug!("Results after deduplication: {}", unique_results.len());

        // 5) Rank
        tracing::debug!("Ranking results");
        rank_results(track, &mut unique_results);

        tracing::info!(
            "Search complete for '{}' by '{}': {} results found",
            track.title,
            track.artists.join(", "),
            unique_results.len()
        );

        Ok(unique_results)
    }

    /// Optional: keep the session warm and recover if it drops while idle.
    /// Returns a JoinHandle so the caller can abort it on shutdown.
    pub fn spawn_watchdog(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                if let Err(e) = self.ensure_session().await {
                    tracing::debug!("Watchdog: session ensure failed: {:?}", e);
                }
                tokio::time::sleep(Duration::from_secs(60)).await;
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Make helper functions accessible for testing
    use super::{
        build_search_queries, clean_search_string, flatten_search_response, is_audio_file,
        rank_results, remove_diacritics, to_file_attributes,
    };

    // ============================================================================
    // Test Fixtures
    // ============================================================================

    fn create_test_track() -> Track {
        Track {
            title: "Test Song".to_string(),
            album: "Test Album".to_string(),
            artists: vec!["Test Artist".to_string()],
            length: Some(180),
        }
    }

    fn create_test_config() -> SearchConfig {
        SearchConfig {
            username: "test_user".to_string(),
            password: "test_pass".to_string(),
            concurrency: Some(2),
            searches_per_time: Some(1000), // Very permissive for tests
            renew_time_secs: Some(1),      // Very short period for tests
            max_search_time_ms: Some(100), // Short timeout for tests
            remove_special_chars: Some(false),
        }
    }

    fn create_test_file_search_response() -> FileSearchResponse {
        FileSearchResponse {
            username: "test_user".to_string(),
            token: "token123".to_string(),
            files: vec![
                FileInfo {
                    filename: "song.mp3".to_string(),
                    size: 5000000,
                    attrs: HashMap::from([(0, 320), (1, 180)]), // Bitrate, Duration
                },
                FileInfo {
                    filename: "song.flac".to_string(),
                    size: 10000000,
                    attrs: HashMap::from([(0, 1411), (4, 44100)]), // Bitrate, SampleRate
                },
            ],
            slots_free: true,
            avg_speed: 100.0,
            queue_length: 0,
        }
    }

    fn create_test_single_file_result() -> SingleFileResult {
        SingleFileResult {
            username: "test_user".to_string(),
            token: "token123".to_string(),
            filename: "song.mp3".to_string(),
            size: 5000000,
            slots_free: true,
            avg_speed: 100.0,
            queue_length: 0,
            attrs: HashMap::from([(FileAttribute::Bitrate, 320)]),
        }
    }

    // ============================================================================
    // Unit Tests for Helper Functions
    // ============================================================================

    #[test]
    fn test_remove_diacritics() {
        assert_eq!(remove_diacritics("café"), "cafe");
        assert_eq!(remove_diacritics("naïve"), "naive");
        assert_eq!(remove_diacritics("résumé"), "resume");
        assert_eq!(remove_diacritics("Müller"), "Muller");
        assert_eq!(remove_diacritics("no accents"), "no accents");
        assert_eq!(remove_diacritics(""), "");
    }

    #[test]
    fn test_clean_search_string() {
        // Without special character removal
        assert_eq!(clean_search_string("  hello world  ", false), "hello world");
        assert_eq!(clean_search_string("normal text", false), "normal text");

        // With special character removal
        assert_eq!(clean_search_string("hello-world!", true), "hello world");
        assert_eq!(clean_search_string("test@#$%^&*()", true), "test");
        assert_eq!(
            clean_search_string("multiple   spaces", true),
            "multiple spaces"
        );
        assert_eq!(
            clean_search_string("  trim  and  clean  ", true),
            "trim and clean"
        );
        assert_eq!(clean_search_string("", true), "");
    }

    #[test]
    fn test_build_search_queries() {
        let track = create_test_track();

        // Test with all fields
        let queries = build_search_queries(&track, false);
        assert!(queries.len() >= 2); // Should have at least base query and with album
        assert!(queries.iter().any(|q| q.contains("Test Artist")));
        assert!(queries.iter().any(|q| q.contains("Test Song")));
        assert!(queries.iter().any(|q| q.contains("Test Album")));

        // Test with remove_special = true
        let queries_special = build_search_queries(&track, true);
        assert!(queries_special.len() >= 2);

        // Test with missing album
        let track_no_album = Track {
            title: "Test Song".to_string(),
            album: "".to_string(),
            artists: vec!["Test Artist".to_string()],
            length: None,
        };
        let queries_no_album = build_search_queries(&track_no_album, false);
        assert!(!queries_no_album.is_empty());
    }

    #[test]
    fn test_build_search_queries_empty() {
        let track = Track {
            title: "".to_string(),
            album: "".to_string(),
            artists: vec![],
            length: None,
        };
        let queries = build_search_queries(&track, false);
        assert!(queries.is_empty());
    }

    #[test]
    fn test_to_file_attributes() {
        let mut attrs = HashMap::new();
        attrs.insert(0u8, 320u32); // Bitrate
        attrs.insert(1u8, 180u32); // Duration
        attrs.insert(4u8, 44100u32); // SampleRate

        let result = to_file_attributes(&attrs);
        assert_eq!(result.get(&FileAttribute::Bitrate), Some(&320));
        assert_eq!(result.get(&FileAttribute::Duration), Some(&180));
        assert_eq!(result.get(&FileAttribute::SampleRate), Some(&44100));
    }

    #[test]
    fn test_to_file_attributes_unknown_key() {
        let mut attrs = HashMap::new();
        attrs.insert(99u8, 1000u32); // Unknown key

        let result = to_file_attributes(&attrs);
        assert!(result.is_empty());
    }

    #[test]
    fn test_flatten_search_response() {
        let response = create_test_file_search_response();
        let flattened = flatten_search_response(&response);

        assert_eq!(flattened.len(), 2);

        let mp3_result = flattened.iter().find(|f| f.filename == "song.mp3").unwrap();
        assert_eq!(mp3_result.username, "test_user");
        assert_eq!(mp3_result.token, "token123");
        assert_eq!(mp3_result.size, 5000000);
        assert!(mp3_result.slots_free);
        assert_eq!(mp3_result.avg_speed, 100.0);
        assert_eq!(mp3_result.attrs.get(&FileAttribute::Bitrate), Some(&320));
        assert_eq!(mp3_result.attrs.get(&FileAttribute::Duration), Some(&180));
    }

    #[test]
    fn test_is_audio_file() {
        // Positive cases
        assert!(is_audio_file("song.mp3"));
        assert!(is_audio_file("song.MP3"));
        assert!(is_audio_file("song.flac"));
        assert!(is_audio_file("song.wav"));
        assert!(is_audio_file("song.aac"));
        assert!(is_audio_file("song.ogg"));
        assert!(is_audio_file("song.m4a"));
        assert!(is_audio_file("song.wma"));
        assert!(is_audio_file("song.aiff"));
        assert!(is_audio_file("song.alac"));
        assert!(is_audio_file("song.opus"));
        assert!(is_audio_file("song.ape"));

        // With paths
        assert!(is_audio_file("/path/to/song.mp3"));
        assert!(is_audio_file("C:\\Music\\song.flac"));

        // Negative cases
        assert!(!is_audio_file("document.pdf"));
        assert!(!is_audio_file("video.mp4"));
        assert!(!is_audio_file("image.jpg"));
        assert!(!is_audio_file("archive.zip"));
        assert!(!is_audio_file("song.mp3.txt"));
    }

    #[test]
    fn test_rank_results_prefers_matches() {
        let track = Track {
            title: "Thriller".to_string(),
            album: "".to_string(),
            artists: vec!["Michael Jackson".to_string()],
            length: None,
        };

        let mut results = vec![
            SingleFileResult {
                username: "user1".to_string(),
                token: "t1".to_string(),
                filename: "random_song.mp3".to_string(),
                size: 1000,
                slots_free: true,
                avg_speed: 200.0,
                queue_length: 0,
                attrs: HashMap::new(),
            },
            SingleFileResult {
                username: "user2".to_string(),
                token: "t2".to_string(),
                filename: "Michael Jackson - Thriller.mp3".to_string(),
                size: 1000,
                slots_free: true,
                avg_speed: 100.0,
                queue_length: 0,
                attrs: HashMap::new(),
            },
        ];

        rank_results(&track, &mut results);

        // The matching result should come first despite lower speed
        assert!(results[0].filename.contains("Thriller"));
    }

    #[test]
    fn test_rank_results_speed_tiebreaker() {
        let track = Track {
            title: "Song".to_string(),
            album: "".to_string(),
            artists: vec!["Artist".to_string()],
            length: None,
        };

        let mut results = vec![
            SingleFileResult {
                username: "user1".to_string(),
                token: "t1".to_string(),
                filename: "Artist Song.mp3".to_string(),
                size: 1000,
                slots_free: true,
                avg_speed: 100.0,
                queue_length: 0,
                attrs: HashMap::new(),
            },
            SingleFileResult {
                username: "user2".to_string(),
                token: "t2".to_string(),
                filename: "Artist Song.flac".to_string(),
                size: 1000,
                slots_free: true,
                avg_speed: 200.0,
                queue_length: 0,
                attrs: HashMap::new(),
            },
        ];

        rank_results(&track, &mut results);

        // Both match, so higher speed should come first
        assert_eq!(results[0].avg_speed, 200.0);
    }

    // ============================================================================
    // Integration Tests for Context
    // ============================================================================

    #[tokio::test]
    async fn test_context_creation() {
        let config = create_test_config();
        let context = SoulSeekClientContext::new(config).await;
        assert!(context.is_ok());
    }

    #[tokio::test]
    async fn test_context_creation_invalid_searches_per_time() {
        let config = SearchConfig {
            username: "test".to_string(),
            password: "test".to_string(),
            concurrency: None,
            searches_per_time: Some(0), // Invalid
            renew_time_secs: None,
            max_search_time_ms: None,
            remove_special_chars: None,
        };
        let context = SoulSeekClientContext::new(config).await;
        assert!(context.is_err());
    }
}
