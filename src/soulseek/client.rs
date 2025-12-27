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
    // TODO: Note: A better long-term solution would be to move the rate_limiter
    // outside of the mutex-protected inner struct, since RateLimiter from governor is already thread-safe.
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

        let searches_per_time_nonzero = NonZeroU32::new(searches_per_time)
            .ok_or_else(|| color_eyre::eyre::eyre!("searches_per_time must be greater than 0"))?;
        let quota = Quota::with_period(std::time::Duration::from_secs(renew_time_secs as u64))
            .ok_or_else(|| color_eyre::eyre::eyre!("Invalid rate limit period"))?
            .allow_burst(searches_per_time_nonzero);

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
        let download_path = download_folder
            .to_str()
            .ok_or_else(|| color_eyre::eyre::eyre!("Download path contains invalid UTF-8"))?
            .to_string();

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

                    // Acquire rate limiter permit while not holding the main lock
                    {
                        let ctx = inner.lock().await;
                        ctx.rate_limiter.until_ready().await;
                    }

                    log::debug!("Executing search query: '{}'", q);
                    let timeout = Duration::from_millis(max_search_time);
                    // Re-acquire lock only for the search operation
                    let responses = {
                        let ctx = inner.lock().await;
                        SoulSeekClientTrait::search(&ctx.wrapper, &q, timeout).await?
                    };

                    log::debug!(
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
            title: "Song".to_string(),
            album: String::new(),
            artists: vec!["Artist".to_string()],
            length: None,
        };
        let queries_no_album = build_search_queries(&track_no_album, false);
        assert!(!queries_no_album.is_empty());
        assert!(!queries_no_album.iter().any(|q| q.contains("Album")));

        // Test with missing title
        let track_no_title = Track {
            title: String::new(),
            album: "Album".to_string(),
            artists: vec!["Artist".to_string()],
            length: None,
        };
        let queries_no_title = build_search_queries(&track_no_title, false);
        assert_eq!(queries_no_title.len(), 0); // Should be empty without title

        // Test with multiple artists
        let track_multi_artist = Track {
            title: "Song".to_string(),
            album: "Album".to_string(),
            artists: vec!["Artist1".to_string(), "Artist2".to_string()],
            length: None,
        };
        let queries_multi = build_search_queries(&track_multi_artist, false);
        assert!(queries_multi.iter().any(|q| q.contains("Artist1")));
        assert!(queries_multi.iter().any(|q| q.contains("Artist2")));
    }

    #[test]
    fn test_is_audio_file() {
        // Positive cases
        assert!(is_audio_file("song.mp3"));
        assert!(is_audio_file("song.MP3")); // Case insensitive
        assert!(is_audio_file("song.flac"));
        assert!(is_audio_file("song.wav"));
        assert!(is_audio_file("song.m4a"));
        assert!(is_audio_file("song.ogg"));
        assert!(is_audio_file("song.aac"));
        assert!(is_audio_file("song.aiff"));
        assert!(is_audio_file("path/to/song.mp3"));

        // Negative cases
        assert!(!is_audio_file("song.txt"));
        assert!(!is_audio_file("song.pdf"));
        assert!(!is_audio_file("song"));
        assert!(!is_audio_file("song.mp3.backup"));
        assert!(!is_audio_file(""));
    }

    #[test]
    fn test_to_file_attributes() {
        let attrs = HashMap::from([
            (0, 320),   // Bitrate
            (1, 180),   // Duration
            (2, 1),     // VariableBitRate
            (3, 5),     // Encoder
            (4, 44100), // SampleRate
            (5, 16),    // BitDepth
            (99, 999),  // Unknown (should be ignored)
        ]);

        let result = to_file_attributes(&attrs);

        assert_eq!(result.get(&FileAttribute::Bitrate), Some(&320));
        assert_eq!(result.get(&FileAttribute::Duration), Some(&180));
        assert_eq!(result.get(&FileAttribute::VariableBitRate), Some(&1));
        assert_eq!(result.get(&FileAttribute::Encoder), Some(&5));
        assert_eq!(result.get(&FileAttribute::SampleRate), Some(&44100));
        assert_eq!(result.get(&FileAttribute::BitDepth), Some(&16));
        assert_eq!(result.len(), 6); // Should not include unknown key
    }

    #[test]
    fn test_flatten_search_response() {
        let response = create_test_file_search_response();
        let results = flatten_search_response(&response);

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].username, "test_user");
        assert_eq!(results[0].token, "token123");
        assert_eq!(results[0].filename, "song.mp3");
        assert_eq!(results[0].size, 5000000);
        assert!(results[0].slots_free);
        assert_eq!(results[0].avg_speed, 100.0);

        assert_eq!(results[1].filename, "song.flac");
        assert_eq!(results[1].size, 10000000);

        // Check attributes were converted
        assert!(results[0].attrs.contains_key(&FileAttribute::Bitrate));
        assert!(results[0].attrs.contains_key(&FileAttribute::Duration));
    }

    #[test]
    fn test_rank_results() {
        let track = Track {
            title: "Test Song".to_string(),
            album: String::new(),
            artists: vec!["Test Artist".to_string()],
            length: None,
        };

        let mut results = vec![
            SingleFileResult {
                filename: "other_song.mp3".to_string(),
                avg_speed: 200.0,
                ..create_test_single_file_result()
            },
            SingleFileResult {
                filename: "Test Artist - Test Song.mp3".to_string(),
                avg_speed: 50.0,
                ..create_test_single_file_result()
            },
            SingleFileResult {
                filename: "another_song.mp3".to_string(),
                avg_speed: 150.0,
                ..create_test_single_file_result()
            },
        ];

        rank_results(&track, &mut results);

        // First result should be the one with both artist and title
        assert!(results[0].filename.contains("Test Artist"));
        assert!(results[0].filename.contains("Test Song"));

        // Results with artist+title should come before those without
        let artist_title_count = results
            .iter()
            .take_while(|r| {
                r.filename.to_lowercase().contains("test artist")
                    && r.filename.to_lowercase().contains("test song")
            })
            .count();
        assert!(artist_title_count >= 1);
    }

    #[test]
    fn test_rank_results_speed_comparison() {
        let track = Track {
            title: "Song".to_string(),
            album: String::new(),
            artists: vec!["Artist".to_string()],
            length: None,
        };

        let mut results = vec![
            SingleFileResult {
                filename: "song1.mp3".to_string(),
                avg_speed: 100.0,
                ..create_test_single_file_result()
            },
            SingleFileResult {
                filename: "song2.mp3".to_string(),
                avg_speed: 300.0,
                ..create_test_single_file_result()
            },
            SingleFileResult {
                filename: "song3.mp3".to_string(),
                avg_speed: 200.0,
                ..create_test_single_file_result()
            },
        ];

        rank_results(&track, &mut results);

        // When no exact matches, should be sorted by speed (descending)
        assert!(results[0].avg_speed >= results[1].avg_speed);
        assert!(results[1].avg_speed >= results[2].avg_speed);
    }

    // ============================================================================
    // Mock Implementation
    // ============================================================================

    // Note: Mocking async traits with mockall requires using automock or manual implementation
    // For now, we'll test the helper functions and integration logic without full mocking
    // In a production scenario, you'd want to refactor to use dependency injection
    // to make the client wrapper injectable for testing

    // ============================================================================
    // Integration Tests for SoulSeekClientContext
    // ============================================================================

    // Note: Testing SoulSeekClientContext::new() requires actual SoulSeek credentials
    // and network access, so we'll skip that for now. In a real scenario, you'd
    // want to refactor to inject the wrapper or use a test double.

    #[tokio::test]
    async fn test_search_for_track_filtering() {
        // This test would require mocking the inner client, which is complex
        // For now, we test the helper functions that are used
        let track = create_test_track();
        let queries = build_search_queries(&track, false);
        assert!(!queries.is_empty());
    }

    #[tokio::test]
    async fn test_search_for_track_deduplication_logic() {
        // Test deduplication logic
        let mut unique_map: HashMap<String, SingleFileResult> = HashMap::new();

        let result1 = SingleFileResult {
            username: "user1".to_string(),
            filename: "song.mp3".to_string(),
            avg_speed: 100.0,
            ..create_test_single_file_result()
        };

        let result2 = SingleFileResult {
            username: "user1".to_string(),
            filename: "song.mp3".to_string(),
            avg_speed: 200.0, // Higher speed
            ..create_test_single_file_result()
        };

        let key1 = format!("{}::{}", result1.username, result1.filename);
        let key2 = format!("{}::{}", result2.username, result2.filename);

        assert_eq!(key1, key2); // Same key

        unique_map.insert(key1.clone(), result1);
        if let Some(existing) = unique_map.get(&key2) {
            if result2.avg_speed > existing.avg_speed {
                unique_map.insert(key2, result2);
            }
        } else {
            unique_map.insert(key2, result2);
        }

        // Should keep the one with higher speed
        assert_eq!(unique_map.len(), 1);
        assert_eq!(unique_map.get(&key1).unwrap().avg_speed, 200.0);
    }

    #[test]
    fn test_search_config_defaults() {
        let config = SearchConfig {
            username: "user".to_string(),
            password: "pass".to_string(),
            concurrency: None,
            searches_per_time: None,
            renew_time_secs: None,
            max_search_time_ms: None,
            remove_special_chars: None,
        };

        // Test default value access
        assert_eq!(config.concurrency.unwrap_or(2), 2);
        assert_eq!(config.searches_per_time.unwrap_or(34), 34);
        assert_eq!(config.renew_time_secs.unwrap_or(220), 220);
        assert_eq!(config.max_search_time_ms.unwrap_or(8000), 8000);
        assert!(!config.remove_special_chars.unwrap_or(false));
    }

    #[test]
    fn test_file_search_response_conversion() {
        let response = FileSearchResponse {
            username: "user".to_string(),
            token: "token".to_string(),
            files: vec![FileInfo {
                filename: "test.mp3".to_string(),
                size: 1000,
                attrs: HashMap::new(),
            }],
            slots_free: true,
            avg_speed: 50.0,
            queue_length: 5,
        };

        let results = flatten_search_response(&response);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].username, "user");
        assert_eq!(results[0].filename, "test.mp3");
        assert!(results[0].slots_free);
        assert_eq!(results[0].avg_speed, 50.0);
        assert_eq!(results[0].queue_length, 5);
    }

    #[test]
    fn test_empty_search_queries() {
        let track = Track {
            title: String::new(),
            album: String::new(),
            artists: vec![],
            length: None,
        };

        let queries = build_search_queries(&track, false);
        assert_eq!(queries.len(), 0);
    }

    #[test]
    fn test_unicode_and_special_characters() {
        let track = Track {
            title: "Café & Bar".to_string(),
            album: "Album (2024)".to_string(),
            artists: vec!["Müller".to_string()],
            length: None,
        };

        let queries = build_search_queries(&track, true);
        assert!(!queries.is_empty());

        // Check that special characters are handled
        let cleaned = clean_search_string("Café & Bar", true);
        assert!(!cleaned.contains("&"));
    }

    // ============================================================================
    // Edge Case Tests
    // ============================================================================

    #[test]
    fn test_is_audio_file_case_insensitive() {
        assert!(is_audio_file("SONG.MP3"));
        assert!(is_audio_file("Song.Flac"));
        assert!(is_audio_file("song.WAV"));
        assert!(is_audio_file("SONG.m4a"));
    }

    #[test]
    fn test_clean_search_string_preserves_unicode_when_not_removing_special() {
        let result = clean_search_string("Café & Bar", false);
        assert_eq!(result, "Café & Bar");
    }

    #[test]
    fn test_build_search_queries_with_only_title() {
        let track = Track {
            title: "Song".to_string(),
            album: String::new(),
            artists: vec![],
            length: None,
        };

        let queries = build_search_queries(&track, false);
        assert_eq!(queries.len(), 0); // Should be empty without artist
    }

    #[test]
    fn test_build_search_queries_with_only_artist() {
        let track = Track {
            title: String::new(),
            album: String::new(),
            artists: vec!["Artist".to_string()],
            length: None,
        };

        let queries = build_search_queries(&track, false);
        assert_eq!(queries.len(), 0); // Should be empty without title
    }

    #[test]
    fn test_to_file_attributes_empty() {
        let attrs = HashMap::new();
        let result = to_file_attributes(&attrs);
        assert!(result.is_empty());
    }

    #[test]
    fn test_to_file_attributes_unknown_keys() {
        let attrs = HashMap::from([(99, 999), (100, 1000)]);
        let result = to_file_attributes(&attrs);
        assert!(result.is_empty()); // Unknown keys should be ignored
    }

    #[test]
    fn test_flatten_search_response_empty_files() {
        let response = FileSearchResponse {
            username: "user".to_string(),
            token: "token".to_string(),
            files: vec![],
            slots_free: true,
            avg_speed: 0.0,
            queue_length: 0,
        };

        let results = flatten_search_response(&response);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_rank_results_empty() {
        let track = create_test_track();
        let mut results: Vec<SingleFileResult> = vec![];
        rank_results(&track, &mut results);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_rank_results_single_result() {
        let track = create_test_track();
        let mut results = vec![SingleFileResult {
            filename: "song.mp3".to_string(),
            avg_speed: 100.0,
            ..create_test_single_file_result()
        }];

        rank_results(&track, &mut results);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_rank_results_all_match_artist_and_title() {
        let track = Track {
            title: "Song".to_string(),
            album: String::new(),
            artists: vec!["Artist".to_string()],
            length: None,
        };

        let mut results = vec![
            SingleFileResult {
                filename: "Artist - Song.mp3".to_string(),
                avg_speed: 50.0,
                ..create_test_single_file_result()
            },
            SingleFileResult {
                filename: "Artist Song.mp3".to_string(),
                avg_speed: 100.0,
                ..create_test_single_file_result()
            },
            SingleFileResult {
                filename: "Artist-Song.mp3".to_string(),
                avg_speed: 75.0,
                ..create_test_single_file_result()
            },
        ];

        rank_results(&track, &mut results);

        // All match, so should be sorted by speed (descending)
        assert!(results[0].avg_speed >= results[1].avg_speed);
        assert!(results[1].avg_speed >= results[2].avg_speed);
    }

    #[test]
    fn test_deduplication_keeps_higher_speed() {
        let mut unique_map: HashMap<String, SingleFileResult> = HashMap::new();

        let result1 = SingleFileResult {
            username: "user1".to_string(),
            filename: "song.mp3".to_string(),
            avg_speed: 50.0,
            ..create_test_single_file_result()
        };

        let result2 = SingleFileResult {
            username: "user1".to_string(),
            filename: "song.mp3".to_string(),
            avg_speed: 100.0,
            ..create_test_single_file_result()
        };

        let key = format!("{}::{}", result1.username, result1.filename);

        // Insert first result
        unique_map.insert(key.clone(), result1);

        // Try to insert second result (should replace if higher speed)
        if let Some(existing) = unique_map.get(&key) {
            if result2.avg_speed > existing.avg_speed {
                unique_map.insert(key.clone(), result2);
            }
        } else {
            unique_map.insert(key.clone(), result2);
        }

        assert_eq!(unique_map.len(), 1);
        assert_eq!(unique_map.get(&key).unwrap().avg_speed, 100.0);
    }

    #[test]
    fn test_deduplication_keeps_lower_speed_when_first_is_higher() {
        let mut unique_map: HashMap<String, SingleFileResult> = HashMap::new();

        let result1 = SingleFileResult {
            username: "user1".to_string(),
            filename: "song.mp3".to_string(),
            avg_speed: 100.0,
            ..create_test_single_file_result()
        };

        let result2 = SingleFileResult {
            username: "user1".to_string(),
            filename: "song.mp3".to_string(),
            avg_speed: 50.0,
            ..create_test_single_file_result()
        };

        let key = format!("{}::{}", result1.username, result1.filename);

        // Insert first result
        unique_map.insert(key.clone(), result1);

        // Try to insert second result (should NOT replace since lower speed)
        if let Some(existing) = unique_map.get(&key) {
            if result2.avg_speed > existing.avg_speed {
                unique_map.insert(key.clone(), result2);
            }
        } else {
            unique_map.insert(key.clone(), result2);
        }

        assert_eq!(unique_map.len(), 1);
        assert_eq!(unique_map.get(&key).unwrap().avg_speed, 100.0);
    }

    #[test]
    fn test_audio_file_filtering_logic() {
        let mut results = vec![
            SingleFileResult {
                filename: "song.mp3".to_string(),
                ..create_test_single_file_result()
            },
            SingleFileResult {
                filename: "song.txt".to_string(),
                ..create_test_single_file_result()
            },
            SingleFileResult {
                filename: "song.flac".to_string(),
                ..create_test_single_file_result()
            },
            SingleFileResult {
                filename: "song.pdf".to_string(),
                ..create_test_single_file_result()
            },
        ];

        results.retain(|f| is_audio_file(&f.filename));

        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|r| r.filename.ends_with(".mp3")));
        assert!(results.iter().any(|r| r.filename.ends_with(".flac")));
        assert!(!results.iter().any(|r| r.filename.ends_with(".txt")));
        assert!(!results.iter().any(|r| r.filename.ends_with(".pdf")));
    }

    #[test]
    fn test_build_search_queries_deduplication() {
        let track = Track {
            title: "Song".to_string(),
            album: "Song".to_string(), // Same as title
            artists: vec!["Artist".to_string()],
            length: None,
        };

        let queries = build_search_queries(&track, false);
        // Should deduplicate queries (using HashSet internally)
        let unique_queries: std::collections::HashSet<_> = queries.iter().collect();
        assert_eq!(queries.len(), unique_queries.len());
    }

    #[test]
    fn test_remove_diacritics_various_languages() {
        assert_eq!(remove_diacritics("café"), "cafe");
        assert_eq!(remove_diacritics("naïve"), "naive");
        assert_eq!(remove_diacritics("résumé"), "resume");
        assert_eq!(remove_diacritics("Müller"), "Muller");
        assert_eq!(remove_diacritics("Zürich"), "Zurich");
        assert_eq!(remove_diacritics("São Paulo"), "Sao Paulo");
        assert_eq!(remove_diacritics("北京"), "北京"); // Chinese characters unchanged
    }

    #[test]
    fn test_clean_search_string_multiple_special_chars() {
        assert_eq!(clean_search_string("test@#$%^&*()", true), "test");
        assert_eq!(clean_search_string("hello---world", true), "hello world");
        assert_eq!(clean_search_string("test!!!", true), "test");
    }

    #[test]
    fn test_clean_search_string_whitespace_handling() {
        assert_eq!(
            clean_search_string("  hello   world  ", true),
            "hello world"
        );
        assert_eq!(
            clean_search_string("\t\nhello\t\nworld\n\t", true),
            "hello world"
        );
    }

    #[test]
    fn test_file_attributes_all_types() {
        let attrs = HashMap::from([
            (0, 320),   // Bitrate
            (1, 180),   // Duration
            (2, 1),     // VariableBitRate
            (3, 5),     // Encoder
            (4, 44100), // SampleRate
            (5, 16),    // BitDepth
        ]);

        let result = to_file_attributes(&attrs);

        assert_eq!(result.len(), 6);
        assert_eq!(result.get(&FileAttribute::Bitrate), Some(&320));
        assert_eq!(result.get(&FileAttribute::Duration), Some(&180));
        assert_eq!(result.get(&FileAttribute::VariableBitRate), Some(&1));
        assert_eq!(result.get(&FileAttribute::Encoder), Some(&5));
        assert_eq!(result.get(&FileAttribute::SampleRate), Some(&44100));
        assert_eq!(result.get(&FileAttribute::BitDepth), Some(&16));
    }

    #[test]
    fn test_flatten_search_response_preserves_all_fields() {
        let response = FileSearchResponse {
            username: "testuser".to_string(),
            token: "testtoken".to_string(),
            files: vec![FileInfo {
                filename: "test.mp3".to_string(),
                size: 12345,
                attrs: HashMap::from([(0, 320)]),
            }],
            slots_free: false,
            avg_speed: 250.5,
            queue_length: 3,
        };

        let results = flatten_search_response(&response);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].username, "testuser");
        assert_eq!(results[0].token, "testtoken");
        assert_eq!(results[0].filename, "test.mp3");
        assert_eq!(results[0].size, 12345);
        assert!(!results[0].slots_free);
        assert_eq!(results[0].avg_speed, 250.5);
        assert_eq!(results[0].queue_length, 3);
    }
}
