// TODO: Remove this once we have a proper API
#![allow(dead_code)]

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, mpsc};
use std::time::Duration;
use tokio::sync::{Mutex, MutexGuard};

use color_eyre::{Result, eyre::Context};
use futures::future::join_all;
use governor::{
    Quota, RateLimiter, clock::DefaultClock, state::InMemoryState, state::direct::NotKeyed,
};
use soulseek_rs::client::Client as SoulseekClient;
use std::num::NonZeroU32;
use tokio::sync::Semaphore;

use crate::soulseek::types::{FileAttribute, SearchConfig, SingleFileResult, Track};
use regex::Regex;
use unaccent::unaccent;

// ============================================================================
// Trait for SoulSeek Client (for testability)
// ============================================================================

#[async_trait::async_trait]
pub trait SoulSeekClientTrait: Send + Sync {
    async fn login(&mut self, username: &str, password: &str) -> Result<()>;
    async fn search(&self, query: &str, timeout: Duration) -> Result<Vec<FileSearchResponse>>;
}

// Wrapper for real soulseek-rs-lib client
pub struct SoulSeekClientWrapper {
    client: Option<Arc<Mutex<SoulseekClient>>>,
}

impl SoulSeekClientWrapper {
    pub fn new() -> Self {
        Self { client: None }
    }

    /// Get access to the inner client for direct access (e.g., downloads)
    /// Returns a guard that can be used to call methods on the client
    pub async fn get_client(&self) -> Option<MutexGuard<'_, SoulseekClient>> {
        match self.client.as_ref() {
            Some(m) => Some(m.lock().await),
            None => None,
        }
    }
}

#[async_trait::async_trait]
impl SoulSeekClientTrait for SoulSeekClientWrapper {
    async fn login(&mut self, username: &str, password: &str) -> Result<()> {
        log::debug!("Logging in to SoulSeek as user: {}", username);

        // Run the synchronous operations in a blocking task
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

        self.client = Some(Arc::new(Mutex::new(client)));
        log::info!("Successfully logged in to SoulSeek");
        Ok(())
    }

    async fn search(&self, query: &str, timeout: Duration) -> Result<Vec<FileSearchResponse>> {
        let client_mutex = self
            .client
            .as_ref()
            .ok_or_else(|| color_eyre::eyre::eyre!("Client not initialized"))?
            .clone();
        let query = query.to_string();

        let client = client_mutex.lock().await;
        let search_results = client.search(&query, timeout)?;

        // Convert SearchResult to FileSearchResponse
        let mut responses = Vec::new();
        for result in search_results {
            let files: Vec<FileInfo> = result
                .files
                .iter()
                .map(|f| {
                    // Convert HashMap<u32, u32> to HashMap<u8, u32>
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
                queue_length: 0, // Not available in SearchResult
            });
        }

        Ok(responses)
    }
}

// Placeholder types - these should match soulseek-rs-lib types
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
    pub attrs: HashMap<u8, u32>, // Map from attribute key to value
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

fn flatten_search_response(response: &FileSearchResponse) -> Vec<SingleFileResult> {
    response
        .files
        .iter()
        .map(|file| SingleFileResult {
            username: response.username.clone(),
            token: response.token.clone(),
            filename: file.filename.clone(),
            size: file.size,
            slots_free: response.slots_free,
            avg_speed: response.avg_speed,
            queue_length: response.queue_length,
            attrs: to_file_attributes(&file.attrs),
        })
        .collect()
}

fn is_audio_file(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    [".mp3", ".flac", ".wav", ".m4a", ".ogg", ".aac", ".aiff"]
        .iter()
        .any(|ext| lower.ends_with(ext))
}

fn rank_results(track: &Track, results: &mut [SingleFileResult]) {
    let requested_artist_lc = track.artists.join(" ").to_lowercase();
    let requested_title_lc = track.title.to_lowercase();
    let _user_length = track.length.filter(|&l| l > 0);

    results.sort_by(|a, b| {
        // Primary: filename contains both artist and title (exact match preferred)
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
// Client Context
// ============================================================================

type DirectRateLimiter = RateLimiter<NotKeyed, InMemoryState, DefaultClock>;

struct SoulSeekClientContextInner {
    wrapper: SoulSeekClientWrapper,
    rate_limiter: DirectRateLimiter,
    config: SearchConfig,
}

pub struct SoulSeekClientContext {
    inner: Arc<Mutex<SoulSeekClientContextInner>>,
}

impl SoulSeekClientContext {
    pub async fn new(config: SearchConfig) -> Result<Self> {
        log::debug!("Creating SoulSeek client context");

        let mut wrapper = SoulSeekClientWrapper::new();
        wrapper.login(&config.username, &config.password).await?;

        let searches_per_time = config.searches_per_time.unwrap_or(34);
        let renew_time_secs = config.renew_time_secs.unwrap_or(220);

        log::debug!(
            "Rate limiter configured: {} searches per {}s",
            searches_per_time,
            renew_time_secs
        );

        // Create governor rate limiter
        let quota = Quota::with_period(std::time::Duration::from_secs(renew_time_secs as u64))
            .ok_or_else(|| color_eyre::eyre::eyre!("Invalid rate limit period"))?
            .allow_burst(NonZeroU32::new(searches_per_time).unwrap());

        let rate_limiter = RateLimiter::direct(quota);

        let inner = SoulSeekClientContextInner {
            wrapper,
            rate_limiter,
            config,
        };

        log::info!("SoulSeek client context created successfully");
        Ok(Self {
            inner: Arc::new(Mutex::new(inner)),
        })
    }

    /// Download a file from SoulSeek
    pub async fn download_file(
        &self,
        result: &SingleFileResult,
        download_folder: &Path,
    ) -> Result<mpsc::Receiver<soulseek_rs::DownloadStatus>> {
        log::debug!(
            "Starting download: '{}' from user '{}' ({} bytes)",
            result.filename,
            result.username,
            result.size
        );

        // Ensure download directory exists
        tokio::fs::create_dir_all(download_folder).await?;

        // Lock context to get the soulseek client for direct download access
        let filename = result.filename.clone();
        let username = result.username.clone();
        let size = result.size;
        let download_path = download_folder.as_os_str().to_str().unwrap().to_string();

        let receiver = {
            let inner = self.inner.lock().await;
            let client_guard = inner.wrapper.get_client().await.ok_or_else(|| {
                color_eyre::eyre::eyre!("SoulSeek client not available for download")
            })?;

            client_guard
                .download(filename.clone(), username.clone(), size, download_path)
                .context("Failed to download file")?
            // inner and client_guard are dropped here when the block ends
        };
        log::info!("Download initiated: '{}' from '{}'", filename, username);
        Ok(receiver)
    }

    /// Search for a track on SoulSeek
    pub async fn search_for_track(&self, track: &Track) -> Result<Vec<SingleFileResult>> {
        log::debug!(
            "Starting search for track: '{}' by '{}'",
            track.title,
            track.artists.join(", ")
        );

        // Lock context to read config
        let remove_special = {
            let inner = self.inner.lock().await;
            inner.config.remove_special_chars.unwrap_or(false)
        };
        let concurrency = {
            let inner = self.inner.lock().await;
            inner.config.concurrency.unwrap_or(2)
        };
        let max_search_time = {
            let inner = self.inner.lock().await;
            inner.config.max_search_time_ms.unwrap_or(8000)
        };

        // 1) Build queries
        let queries = build_search_queries(track, remove_special);
        log::debug!("Built {} search queries", queries.len());

        // 2a) Concurrency limit
        let semaphore = Arc::new(Semaphore::new(concurrency));

        // We'll collect all results in a vector
        let mut all_flattened: Vec<SingleFileResult> = vec![];

        // For each query, do a concurrency-limited, rate-limited search
        let tasks: Vec<_> = queries
            .into_iter()
            .map(|q| {
                let q = q.clone(); // Clone the String to move into async block
                let inner = self.inner.clone();
                let sem = semaphore.clone();
                async move {
                    let _permit = sem.acquire().await.unwrap();

                    // Lock context to access rate_limiter and wrapper
                    let ctx = inner.lock().await;

                    // Acquire from rate limiter
                    ctx.rate_limiter.until_ready().await;

                    log::debug!("Executing search query: '{}'", q);
                    let timeout = Duration::from_millis(max_search_time);
                    let responses = SoulSeekClientTrait::search(&ctx.wrapper, &q, timeout).await?;

                    log::debug!(
                        "Search query '{}' returned {} responses",
                        q,
                        responses.len()
                    );

                    let mut flattened = vec![];
                    for r in responses {
                        flattened.extend(flatten_search_response(&r));
                    }

                    // Release lock before returning
                    drop(ctx);
                    Result::<Vec<SingleFileResult>, color_eyre::Report>::Ok(flattened)
                }
            })
            .collect();

        let results = join_all(tasks).await;
        for result in results {
            all_flattened.extend(result?);
        }

        log::debug!("Total results before filtering: {}", all_flattened.len());

        // 3) Filter out non-audio file types
        log::debug!("Filtering audio files");
        all_flattened.retain(|f| is_audio_file(&f.filename));
        log::debug!("Results after audio filter: {}", all_flattened.len());

        // 4) Deduplicate (by username + filename)
        log::debug!("Deduplicating results");
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
        log::debug!("Results after deduplication: {}", unique_results.len());

        // 5) Rank
        log::debug!("Ranking results");
        rank_results(track, &mut unique_results);

        log::info!(
            "Search complete for '{}' by '{}': {} results found",
            track.title,
            track.artists.join(", "),
            unique_results.len()
        );

        Ok(unique_results)
    }
}
