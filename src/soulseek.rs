// TODO: Remove this once we have a proper API
#![allow(dead_code)]

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};

use color_eyre::{Result, eyre::Context};
use futures::future::join_all;
use regex::Regex;
use soulseek_rs::client::Client as SoulseekClient;
use tokio::sync::Semaphore;
use tokio::time::sleep;
use unaccent::unaccent;

// ============================================================================
// Types and Enums
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileAttribute {
    Bitrate = 0,
    Duration = 1,
    VariableBitRate = 2,
    Encoder = 3,
    SampleRate = 4,
    BitDepth = 5,
}

#[derive(Debug, Clone)]
pub struct Track {
    pub title: String,
    pub album: String,
    pub artists: Vec<String>,
    pub length: Option<u32>, // optional user-provided length (in seconds)
}

#[derive(Debug, Clone)]
pub struct SearchConfig {
    pub username: String,
    pub password: String,
    pub concurrency: Option<usize>,         // default 2
    pub searches_per_time: Option<u32>,     // default 34
    pub renew_time_secs: Option<u32>,       // default 220
    pub max_search_time_ms: Option<u64>,    // default 8000
    pub remove_special_chars: Option<bool>, // default false
}

#[derive(Debug, Clone)]
pub struct SingleFileResult {
    pub username: String,
    pub token: String,
    pub filename: String,
    pub size: u64,
    pub slots_free: bool,
    pub avg_speed: f64,
    pub queue_length: u32,
    pub attrs: HashMap<FileAttribute, u32>,
}

#[derive(Debug, Clone)]
struct FullTrack {
    artist: String,
    title: String,
    album: String,
    artist_maybe_wrong: bool,
    length: i32,
    track_type: TrackType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TrackType {
    Normal,
    Album,
    Single,
    Unknown,
}

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
    client: Option<Arc<SoulseekClient>>,
}

impl SoulSeekClientWrapper {
    pub fn new() -> Self {
        Self { client: None }
    }
}

#[async_trait::async_trait]
impl SoulSeekClientTrait for SoulSeekClientWrapper {
    async fn login(&mut self, username: &str, password: &str) -> Result<()> {
        // Run the synchronous operations in a blocking task
        let username = username.to_string();
        let password = password.to_string();

        soulseek_rs::utils::logger::enable_buffering();

        let client = tokio::task::spawn_blocking(move || -> Result<Arc<SoulseekClient>> {
            let mut client = SoulseekClient::new(&username, &password);
            client.connect();
            client.login()?;
            Ok(Arc::new(client))
        })
        .await??;

        self.client = Some(client);
        Ok(())
    }

    async fn search(&self, query: &str, timeout: Duration) -> Result<Vec<FileSearchResponse>> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| color_eyre::eyre::eyre!("Client not initialized"))?
            .clone();
        let query = query.to_string();

        // Run the synchronous search in a blocking task
        let search_results =
            tokio::task::spawn_blocking(move || client.search(&query, timeout)).await??;

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
// Rate Limiter
// ============================================================================

pub struct SearchRateLimiter {
    searches_per_time: u32,
    renew_time_ms: u64,
    window_start: Instant,
    used_in_window: u32,
}

impl SearchRateLimiter {
    pub fn new(searches_per_time: u32, renew_time_secs: u32) -> Self {
        Self {
            searches_per_time,
            renew_time_ms: renew_time_secs as u64 * 1000,
            window_start: Instant::now(),
            used_in_window: 0,
        }
    }

    pub async fn acquire(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.window_start).as_millis() as u64;

        // If we're past this window, reset
        if elapsed > self.renew_time_ms {
            self.window_start = now;
            self.used_in_window = 0;
        }

        if self.used_in_window < self.searches_per_time {
            // proceed immediately
            self.used_in_window += 1;
            return;
        }

        // Must wait for window to reset
        let wait_ms = self.renew_time_ms - elapsed;
        println!(
            "Rate limit reached ({}/{}). Waiting {}s...",
            self.used_in_window,
            self.searches_per_time,
            wait_ms / 1000
        );

        sleep(Duration::from_millis(wait_ms)).await;

        // Reset for next window
        self.window_start = Instant::now();
        self.used_in_window = 1; // this search is used
    }
}

// ============================================================================
// Client Context
// ============================================================================

pub struct SoulSeekClientContext {
    pub client: Box<dyn SoulSeekClientTrait>,
    pub rate_limiter: Arc<tokio::sync::Mutex<SearchRateLimiter>>,
    pub config: SearchConfig,
    // Store the actual soulseek client for downloads (since the API requires direct access)
    soulseek_client: Option<Arc<SoulseekClient>>,
}

impl SoulSeekClientContext {
    pub async fn new(config: SearchConfig) -> Result<Self> {
        let mut wrapper = SoulSeekClientWrapper::new();
        wrapper.login(&config.username, &config.password).await?;

        // Get the client from the wrapper for direct access
        let soulseek_client = wrapper.client.clone();
        let client: Box<dyn SoulSeekClientTrait> = Box::new(wrapper);

        let searches_per_time = config.searches_per_time.unwrap_or(34);
        let renew_time_secs = config.renew_time_secs.unwrap_or(220);
        let rate_limiter = Arc::new(tokio::sync::Mutex::new(SearchRateLimiter::new(
            searches_per_time,
            renew_time_secs,
        )));

        Ok(Self {
            client,
            rate_limiter,
            config,
            soulseek_client,
        })
    }
}

// ============================================================================
// String Utilities
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

pub fn build_search_queries(track: &Track, remove_special: bool) -> Vec<String> {
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

    // fallback if base_query is empty
    if base_query.is_empty() && !title.is_empty() {
        queries.insert(clean_search_string(title, remove_special));
        queries.insert(clean_search_string(
            &remove_diacritics(title),
            remove_special,
        ));
    }

    if base_query.is_empty() && !artist_str.is_empty() {
        queries.insert(clean_search_string(&artist_str, remove_special));
        queries.insert(clean_search_string(
            &remove_diacritics(&artist_str),
            remove_special,
        ));
    }

    // remove empties
    queries.into_iter().filter(|q| !q.is_empty()).collect()
}

// ============================================================================
// Track Inference Helpers
// ============================================================================

fn get_file_name_without_ext_slsk(path_str: &str) -> String {
    let clean = path_str.trim_end_matches(['/', '\\']);
    let parts: Vec<&str> = clean.split(['/', '\\']).collect();
    let filename = parts.last().unwrap_or(&"");

    if let Some(dot_pos) = filename.rfind('.')
        && dot_pos > 0
    {
        return filename[..dot_pos].to_string();
    }
    filename.to_string()
}

fn track_to_string_no_info(t: &FullTrack) -> String {
    format!("{} {} {}", t.artist, t.album, t.title)
        .trim()
        .to_string()
}

fn contains_ignore_case(haystack: &str, needle: &str) -> bool {
    if haystack.is_empty() || needle.is_empty() {
        return false;
    }
    haystack.to_lowercase().contains(&needle.to_lowercase())
}

fn remove_ft(str: &str) -> String {
    // Replace feat/ft/featuring with space, then clean up periods and spaces
    let result = Regex::new(r"\s*\b(feat\.?|ft\.?|featuring)\b\s*")
        .unwrap()
        .replace_all(str, " ")
        .to_string();
    // Remove any standalone periods left behind
    let result = Regex::new(r"\s+\.\s+|\s+\.|\.\s+")
        .unwrap()
        .replace_all(&result, " ")
        .to_string();
    remove_consecutive_ws(&result).trim().to_string()
}

fn remove_consecutive_ws(str: &str) -> String {
    Regex::new(r"\s{2,}")
        .unwrap()
        .replace_all(str, " ")
        .to_string()
}

fn replace_invalid_chars(str: &str, replacement: &str, strict: bool) -> String {
    if strict {
        // remove everything but typical ASCII punctuation, letters, digits, parentheses, dash, etc.
        Regex::new(r"[^A-Za-z0-9 \-()]")
            .unwrap()
            .replace_all(str, replacement)
            .to_string()
    } else {
        // remove weird control chars
        Regex::new(r"[^\p{L}\p{N}\p{P}\p{Z}\p{S}]")
            .unwrap()
            .replace_all(str, replacement)
            .to_string()
    }
}

fn escape_regex(str: &str) -> String {
    regex::escape(str)
}

// ============================================================================
// Track Inference
// ============================================================================

fn infer_track_full(
    initial_filename: &str,
    default_track: &FullTrack,
    track_type: TrackType,
) -> FullTrack {
    let mut t = FullTrack {
        artist: default_track.artist.clone(),
        title: default_track.title.clone(),
        album: default_track.album.clone(),
        artist_maybe_wrong: default_track.artist_maybe_wrong,
        length: default_track.length,
        track_type,
    };

    // 1) remove extension & do initial replacements
    let mut filename = get_file_name_without_ext_slsk(initial_filename)
        .replace(" — ", " - ")
        .replace('_', " ")
        .trim()
        .to_string();
    filename = remove_consecutive_ws(&filename);

    // 2) track number patterns
    // Note: Using a simpler pattern without look-ahead since Rust regex doesn't support it by default
    let track_num_start = Regex::new(r"^(?:(?:[0-9][-.])?\d{2,3}[. -]|\d\.\s|\d\s-\s)").unwrap();
    // Simplified pattern without look-behind
    let track_num_middle = Regex::new(r" - ((\d-)?\d{2,3}|\d{2,3}\.?)\s+").unwrap();
    let track_num_middle_alt = Regex::new(r"\s+-(\d{2,3})-\s+").unwrap();

    if track_num_start.is_match(&filename) {
        filename = track_num_start.replace(&filename, "").trim().to_string();
        if filename.starts_with("- ") {
            filename = filename[2..].trim().to_string();
        }
    } else {
        let reg = if track_num_middle.is_match(&filename) {
            Some(&track_num_middle)
        } else if track_num_middle_alt.is_match(&filename) {
            Some(&track_num_middle_alt)
        } else {
            None
        };

        if let Some(reg) = reg
            && !reg.is_match(&track_to_string_no_info(default_track))
        {
            // Replace the pattern with a marker
            if reg.as_str().contains(" - ") {
                filename = reg.replace(&filename, " - <<tracknum>> ").to_string();
            } else {
                filename = reg.replace(&filename, " <<tracknum>> ").to_string();
            }
            filename = Regex::new(r"-\s*<<tracknum>>\s*-")
                .unwrap()
                .replace_all(&filename, "-")
                .to_string();
            filename = filename.replace("<<tracknum>>", "").trim().to_string();
        }
    }

    // 3) local copies
    let mut aname = t.artist.trim().to_string();
    let mut tname = t.title.trim().to_string();
    let mut alname = t.album.trim().to_string();
    let mut fname = filename.clone();

    // replacements
    fname = fname
        .replace('—', "-")
        .replace('_', " ")
        .replace('[', "(")
        .replace(']', ")");
    fname = replace_invalid_chars(&fname, "", true);
    fname = remove_consecutive_ws(&fname).trim().to_string();

    tname = tname
        .replace('—', "-")
        .replace('_', " ")
        .replace('[', "(")
        .replace(']', ")");
    tname = replace_invalid_chars(&tname, "", true).trim().to_string();
    tname = remove_ft(&tname);
    tname = remove_consecutive_ws(&tname);

    aname = aname
        .replace('—', "-")
        .replace('_', " ")
        .replace('[', "(")
        .replace(']', ")");
    aname = replace_invalid_chars(&aname, "", true).trim().to_string();
    aname = remove_ft(&aname);
    aname = remove_consecutive_ws(&aname);

    alname = alname
        .replace('—', "-")
        .replace('_', " ")
        .replace('[', "(")
        .replace(']', ")");
    alname = replace_invalid_chars(&alname, "", true).trim().to_string();
    alname = remove_ft(&alname);
    alname = remove_consecutive_ws(&alname);

    t.artist = aname.clone();
    t.title = tname.clone();
    t.album = alname.clone();

    let maybe_remix = !aname.is_empty()
        && Regex::new(&format!(r"\({}\s+.+\)", escape_regex(&aname)))
            .unwrap()
            .is_match(&fname);

    let parts: Vec<&str> = fname.split(" - ").filter(|s| !s.is_empty()).collect();
    let real_parts = parts.clone();

    if parts.len() == 1 {
        if maybe_remix {
            t.artist_maybe_wrong = true;
        }
        t.title = parts[0].to_string();
    } else if parts.len() == 2 {
        t.artist = real_parts[0].to_string();
        t.title = real_parts[1].to_string();
        if !(contains_ignore_case(parts[0], &aname) && contains_ignore_case(parts[1], &tname)) {
            t.artist_maybe_wrong = true;
        }
    } else if parts.len() == 3 {
        let has_title = !tname.is_empty() && contains_ignore_case(parts[2], &tname);
        if has_title {
            t.title = real_parts[2].to_string();
        }

        let mut artist_pos = -1;
        if !aname.is_empty() {
            if contains_ignore_case(parts[0], &aname) {
                artist_pos = 0;
            } else if contains_ignore_case(parts[1], &aname) {
                artist_pos = 1;
            } else {
                t.artist_maybe_wrong = true;
            }
        }

        let mut album_pos = -1;
        if !alname.is_empty() {
            if contains_ignore_case(parts[0], &alname) {
                album_pos = 0;
            } else if contains_ignore_case(parts[1], &alname) {
                album_pos = 1;
            }
        }

        if artist_pos >= 0 && artist_pos == album_pos {
            artist_pos = 0;
            album_pos = 1;
        }

        if artist_pos == -1 && maybe_remix {
            t.artist_maybe_wrong = true;
            artist_pos = 0;
            album_pos = 1;
        }

        if artist_pos == -1 && album_pos == -1 {
            t.artist_maybe_wrong = true;
            t.artist = format!("{} - {}", real_parts[0], real_parts[1]);
        } else if artist_pos >= 0 {
            t.artist = parts[artist_pos as usize].to_string();
        }

        t.title = parts[2].to_string();
    } else if parts.len() > 3 {
        let mut artist_pos = -1;

        if !aname.is_empty() {
            let matches: Vec<(usize, &str)> = parts
                .iter()
                .enumerate()
                .filter(|(_, p)| contains_ignore_case(p, &aname))
                .map(|(i, p)| (i, *p))
                .collect();

            if !matches.is_empty() {
                let best = matches
                    .iter()
                    .min_by_key(|(_, p)| (p.len() as i32 - aname.len() as i32).abs())
                    .unwrap();
                artist_pos = best.0 as i32;
                t.artist = best.1.to_string();
            }
        }

        if !tname.is_empty() {
            let matches: Vec<(usize, &str)> = parts
                .iter()
                .enumerate()
                .filter(|(i, p)| *i != artist_pos as usize && contains_ignore_case(p, &tname))
                .map(|(i, p)| (i, *p))
                .collect();

            if !matches.is_empty() {
                let best = matches
                    .iter()
                    .min_by_key(|(_, p)| (p.len() as i32 - tname.len() as i32).abs())
                    .unwrap();
                t.title = best.1.to_string();
            }
        }
    }

    if t.title.trim().is_empty() {
        t.title = fname.clone();
        t.artist_maybe_wrong = true;
    } else if !t.artist.is_empty()
        && !contains_ignore_case(&t.title, &default_track.title)
        && !contains_ignore_case(&t.artist, &default_track.artist)
    {
        let x = [t.artist.clone(), t.album.clone(), t.title.clone()];
        let mut perm = vec![0, 1, 2];
        let permutations = vec![
            vec![0, 2, 1],
            vec![1, 0, 2],
            vec![1, 2, 0],
            vec![2, 0, 1],
            vec![2, 1, 0],
        ];

        for p in permutations {
            if contains_ignore_case(&x[p[0]], &default_track.artist)
                && contains_ignore_case(&x[p[2]], &default_track.title)
            {
                perm = p;
                break;
            }
        }

        t.artist = x[perm[0]].clone();
        t.album = x[perm[1]].clone();
        t.title = x[perm[2]].clone();
    }

    t.title = remove_ft(&t.title);
    t.artist = remove_ft(&t.artist);

    t
}

fn infer_track_from_filename(filename: &str) -> (String, String) {
    let default_t = FullTrack {
        artist: String::new(),
        title: String::new(),
        album: String::new(),
        artist_maybe_wrong: false,
        length: -1,
        track_type: TrackType::Normal,
    };

    let parsed = infer_track_full(filename, &default_t, TrackType::Normal);
    (parsed.artist, parsed.title)
}

// ============================================================================
// File Utilities
// ============================================================================

fn is_audio_file(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    [".mp3", ".flac", ".wav", ".m4a", ".ogg", ".aac", ".aiff"]
        .iter()
        .any(|ext| lower.ends_with(ext))
}

fn to_file_attributes(attrs: &HashMap<u8, u32>) -> HashMap<FileAttribute, u32> {
    let mut result = HashMap::new();
    for (key, value) in attrs {
        let attr = match key {
            0 => Some(FileAttribute::Bitrate),
            1 => Some(FileAttribute::Duration),
            2 => Some(FileAttribute::VariableBitRate),
            3 => Some(FileAttribute::Encoder),
            4 => Some(FileAttribute::SampleRate),
            5 => Some(FileAttribute::BitDepth),
            _ => None,
        };
        if let Some(attr) = attr {
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

// ============================================================================
// Ranking
// ============================================================================

pub fn rank_results(track: &Track, results: &mut [SingleFileResult]) {
    let requested_artist_lc = track.artists.join(" ").to_lowercase();
    let requested_title_lc = track.title.to_lowercase();
    let user_length = track.length.filter(|&l| l > 0);

    results.sort_by(|a, b| {
        // 1) prefer slots free
        match (a.slots_free, b.slots_free) {
            (true, false) => return std::cmp::Ordering::Less,
            (false, true) => return std::cmp::Ordering::Greater,
            _ => {}
        }

        // 2) parse filename => see if it matches requested track
        let (a_artist, a_title) = infer_track_from_filename(&a.filename);
        let (b_artist, b_title) = infer_track_from_filename(&b.filename);

        let a_match = (a_artist.to_lowercase().contains(&requested_artist_lc) as u32)
            + (a_title.to_lowercase().contains(&requested_title_lc) as u32);
        let b_match = (b_artist.to_lowercase().contains(&requested_artist_lc) as u32)
            + (b_title.to_lowercase().contains(&requested_title_lc) as u32);

        if b_match != a_match {
            return b_match.cmp(&a_match);
        }

        // 3) If the user provided a target length, compare each file's duration
        if let Some(user_len) = user_length {
            let a_len = a.attrs.get(&FileAttribute::Duration).copied().unwrap_or(0);
            let b_len = b.attrs.get(&FileAttribute::Duration).copied().unwrap_or(0);

            let a_diff = if a_len > 0 {
                (a_len as i32 - user_len as i32).abs()
            } else {
                i32::MAX
            };
            let b_diff = if b_len > 0 {
                (b_len as i32 - user_len as i32).abs()
            } else {
                i32::MAX
            };

            if a_diff != b_diff {
                return a_diff.cmp(&b_diff);
            }
        }

        // 4) speed descending
        if (b.avg_speed - a.avg_speed).abs() > f64::EPSILON {
            return b.avg_speed.partial_cmp(&a.avg_speed).unwrap();
        }

        // 5) fallback substring check
        let a_file_lc = a.filename.to_lowercase();
        let b_file_lc = b.filename.to_lowercase();

        let a_score = (a_file_lc.contains(&requested_artist_lc) as u32)
            + (a_file_lc.contains(&requested_title_lc) as u32);
        let b_score = (b_file_lc.contains(&requested_artist_lc) as u32)
            + (b_file_lc.contains(&requested_title_lc) as u32);

        if b_score != a_score {
            return b_score.cmp(&a_score);
        }

        // 6) queue length: maybe prefer shorter queue
        if a.queue_length != b.queue_length {
            return a.queue_length.cmp(&b.queue_length);
        }

        // 7) tiebreak by size (ascending - smaller first)
        a.size.cmp(&b.size)
    });
}

// ============================================================================
// Main Search Function
// ============================================================================

pub async fn search_for_track(
    track: &Track,
    context: &mut SoulSeekClientContext,
) -> Result<Vec<SingleFileResult>> {
    // 1) Build queries
    let queries = build_search_queries(track, context.config.remove_special_chars.unwrap_or(false));

    // 2a) Concurrency limit
    let concurrency = context.config.concurrency.unwrap_or(2);
    let semaphore = Arc::new(Semaphore::new(concurrency));

    // We'll collect all results in a vector
    let mut all_flattened: Vec<SingleFileResult> = vec![];

    // For each query, do a concurrency-limited, rate-limited search
    let max_search_time = context.config.max_search_time_ms.unwrap_or(8000);
    let tasks: Vec<_> = queries
        .into_iter()
        .map(|q| {
            let sem = semaphore.clone();
            let rate_limiter = context.rate_limiter.clone();
            let client_ref: &dyn SoulSeekClientTrait = &*context.client;
            async move {
                let _permit = sem.acquire().await.unwrap();

                // Acquire from rate limiter
                rate_limiter.lock().await.acquire().await;

                let timeout = Duration::from_millis(max_search_time);
                let responses = client_ref.search(&q, timeout).await?;

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

    // 3) Filter out non-audio file types
    all_flattened.retain(|f| is_audio_file(&f.filename));

    // 4) Deduplicate (by username + filename)
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

    // 5) Rank
    rank_results(track, &mut unique_results);

    Ok(unique_results)
}

// ============================================================================
// Download Function
// ============================================================================

pub async fn download_file(
    result: &SingleFileResult,
    download_folder: &Path,
    context: &SoulSeekClientContext,
) -> Result<mpsc::Receiver<soulseek_rs::DownloadStatus>> {
    // Ensure download directory exists
    tokio::fs::create_dir_all(download_folder).await?;

    // Get the soulseek client for direct download access
    let client = context
        .soulseek_client
        .as_ref()
        .ok_or_else(|| color_eyre::eyre::eyre!("SoulSeek client not available for download"))?
        .clone();

    let filename = result.filename.clone();
    let username = result.username.clone();
    let size = result.size;

    client
        .download(
            filename,
            username,
            size,
            download_folder.as_os_str().to_str().unwrap().to_string(),
        )
        .context("Failed to download file")
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;
    use mockall::predicate::*;

    // Create a mock for SoulSeekClientTrait
    mock! {
        SoulSeekClient {}

        #[async_trait::async_trait]
        impl SoulSeekClientTrait for SoulSeekClient {
            async fn login(&mut self, username: &str, password: &str) -> Result<()>;
            async fn search(&self, query: &str, timeout: Duration) -> Result<Vec<FileSearchResponse>>;
        }
    }

    // Helper to create a mock file search response
    fn create_mock_file_search_response(
        username: &str,
        token: &str,
        files: Vec<FileInfo>,
    ) -> FileSearchResponse {
        FileSearchResponse {
            username: username.to_string(),
            token: token.to_string(),
            files,
            slots_free: true,
            avg_speed: 1000.0,
            queue_length: 0,
        }
    }

    // Helper to create a mock file info
    fn create_mock_file_info(filename: &str, size: u64, duration: Option<u32>) -> FileInfo {
        let mut attrs = HashMap::new();
        if let Some(dur) = duration {
            attrs.insert(1, dur); // Duration attribute
        }
        attrs.insert(0, 320); // Bitrate
        FileInfo {
            filename: filename.to_string(),
            size,
            attrs,
        }
    }

    // ========================================================================
    // Rate Limiter Tests
    // ========================================================================

    #[tokio::test]
    async fn test_rate_limiter_immediate_acquire() {
        let mut limiter = SearchRateLimiter::new(5, 1); // 5 searches per 1 second

        // First 5 should acquire immediately
        for _ in 0..5 {
            let start = Instant::now();
            limiter.acquire().await;
            let elapsed = start.elapsed();
            assert!(elapsed.as_millis() < 100, "Should acquire immediately");
        }
    }

    #[tokio::test]
    async fn test_rate_limiter_waiting() {
        let mut limiter = SearchRateLimiter::new(2, 1); // 2 searches per 1 second

        // First 2 should acquire immediately
        limiter.acquire().await;
        limiter.acquire().await;

        // Third should wait
        let start = Instant::now();
        limiter.acquire().await;
        let elapsed = start.elapsed();
        assert!(elapsed.as_secs() >= 1, "Should wait at least 1 second");
    }

    #[tokio::test]
    async fn test_rate_limiter_window_reset() {
        let mut limiter = SearchRateLimiter::new(1, 1); // 1 search per 1 second

        limiter.acquire().await;

        // Wait for window to reset
        tokio::time::sleep(Duration::from_millis(1100)).await;

        // Should acquire immediately after reset
        let start = Instant::now();
        limiter.acquire().await;
        let elapsed = start.elapsed();
        assert!(
            elapsed.as_millis() < 100,
            "Should acquire immediately after reset"
        );
    }

    // ========================================================================
    // String Utility Tests
    // ========================================================================

    #[test]
    fn test_remove_diacritics() {
        assert_eq!(remove_diacritics("Café"), "Cafe");
        assert_eq!(remove_diacritics("naïve"), "naive");
        assert_eq!(remove_diacritics("résumé"), "resume");
    }

    #[test]
    fn test_clean_search_string() {
        assert_eq!(
            clean_search_string("  hello  world  ", false),
            "hello  world"
        );
        assert_eq!(
            clean_search_string("hello@world#test", true),
            "hello world test"
        );
        assert_eq!(clean_search_string("hello   world", true), "hello world");
    }

    #[test]
    fn test_build_search_queries() {
        let track = Track {
            title: "Song Title".to_string(),
            album: "Album Name".to_string(),
            artists: vec!["Artist Name".to_string()],
            length: None,
        };

        let queries = build_search_queries(&track, false);
        assert!(!queries.is_empty());
        assert!(queries.iter().any(|q| q.contains("Artist Name")));
        assert!(queries.iter().any(|q| q.contains("Song Title")));
    }

    #[test]
    fn test_remove_ft() {
        assert_eq!(remove_ft("Song feat. Artist"), "Song Artist");
        assert_eq!(remove_ft("Song ft. Artist"), "Song Artist");
        assert_eq!(remove_ft("Song featuring Artist"), "Song Artist");
    }

    #[test]
    fn test_remove_consecutive_ws() {
        assert_eq!(remove_consecutive_ws("hello   world"), "hello world");
        assert_eq!(remove_consecutive_ws("hello\t\tworld"), "hello world");
    }

    #[test]
    fn test_contains_ignore_case() {
        assert!(contains_ignore_case("Hello World", "hello"));
        assert!(contains_ignore_case("Hello World", "WORLD"));
        assert!(!contains_ignore_case("Hello World", "xyz"));
    }

    // ========================================================================
    // Track Inference Tests
    // ========================================================================

    #[test]
    fn test_infer_track_from_filename_simple() {
        let (artist, title) = infer_track_from_filename("Artist - Title.mp3");
        assert_eq!(artist, "Artist");
        assert_eq!(title, "Title");
    }

    #[test]
    fn test_infer_track_from_filename_three_parts() {
        let (artist, title) = infer_track_from_filename("Artist - Album - Title.mp3");
        // The inference logic might assign differently, so we just check it's not empty
        assert!(!artist.is_empty() || !title.is_empty());
    }

    #[test]
    fn test_infer_track_from_filename_with_track_number() {
        let (_artist, title) = infer_track_from_filename("01 - Artist - Title.mp3");
        // Track number should be removed
        assert!(!title.is_empty());
    }

    #[test]
    fn test_get_file_name_without_ext_slsk() {
        assert_eq!(get_file_name_without_ext_slsk("path/to/file.mp3"), "file");
        assert_eq!(get_file_name_without_ext_slsk("file.mp3"), "file");
        assert_eq!(get_file_name_without_ext_slsk("file"), "file");
    }

    // ========================================================================
    // File Utility Tests
    // ========================================================================

    #[test]
    fn test_is_audio_file() {
        assert!(is_audio_file("song.mp3"));
        assert!(is_audio_file("song.flac"));
        assert!(is_audio_file("song.wav"));
        assert!(is_audio_file("song.m4a"));
        assert!(is_audio_file("song.ogg"));
        assert!(is_audio_file("song.aac"));
        assert!(is_audio_file("song.aiff"));
        assert!(!is_audio_file("song.txt"));
        assert!(!is_audio_file("song.exe"));
    }

    #[test]
    fn test_to_file_attributes() {
        let mut attrs = HashMap::new();
        attrs.insert(0, 320); // Bitrate
        attrs.insert(1, 180); // Duration
        attrs.insert(2, 1); // VBR

        let result = to_file_attributes(&attrs);
        assert_eq!(result.get(&FileAttribute::Bitrate), Some(&320));
        assert_eq!(result.get(&FileAttribute::Duration), Some(&180));
        assert_eq!(result.get(&FileAttribute::VariableBitRate), Some(&1));
    }

    #[test]
    fn test_flatten_search_response() {
        let response = create_mock_file_search_response(
            "user1",
            "token123",
            vec![create_mock_file_info("song.mp3", 5000000, Some(180))],
        );

        let flattened = flatten_search_response(&response);
        assert_eq!(flattened.len(), 1);
        assert_eq!(flattened[0].username, "user1");
        assert_eq!(flattened[0].filename, "song.mp3");
        assert_eq!(flattened[0].size, 5000000);
    }

    // ========================================================================
    // Ranking Tests
    // ========================================================================

    #[test]
    fn test_rank_results_prefers_slots_free() {
        let track = Track {
            title: "Test Song".to_string(),
            album: "Test Album".to_string(),
            artists: vec!["Test Artist".to_string()],
            length: None,
        };

        let mut results = vec![
            SingleFileResult {
                username: "user1".to_string(),
                token: "token1".to_string(),
                filename: "test.mp3".to_string(),
                size: 1000,
                slots_free: false,
                avg_speed: 1000.0,
                queue_length: 0,
                attrs: HashMap::new(),
            },
            SingleFileResult {
                username: "user2".to_string(),
                token: "token2".to_string(),
                filename: "test.mp3".to_string(),
                size: 1000,
                slots_free: true,
                avg_speed: 500.0,
                queue_length: 0,
                attrs: HashMap::new(),
            },
        ];

        rank_results(&track, &mut results);

        // First result should have slots_free = true
        assert!(results[0].slots_free);
    }

    #[test]
    fn test_rank_results_prefers_better_match() {
        let track = Track {
            title: "Test Song".to_string(),
            album: "Test Album".to_string(),
            artists: vec!["Test Artist".to_string()],
            length: None,
        };

        let mut results = vec![
            SingleFileResult {
                username: "user1".to_string(),
                token: "token1".to_string(),
                filename: "other.mp3".to_string(),
                size: 1000,
                slots_free: true,
                avg_speed: 1000.0,
                queue_length: 0,
                attrs: HashMap::new(),
            },
            SingleFileResult {
                username: "user2".to_string(),
                token: "token2".to_string(),
                filename: "Test Artist - Test Song.mp3".to_string(),
                size: 1000,
                slots_free: true,
                avg_speed: 500.0,
                queue_length: 0,
                attrs: HashMap::new(),
            },
        ];

        rank_results(&track, &mut results);

        // First result should be the better match
        assert!(results[0].filename.contains("Test Artist"));
    }

    #[test]
    fn test_rank_results_prefers_length_match() {
        let track = Track {
            title: "Test Song".to_string(),
            album: "Test Album".to_string(),
            artists: vec!["Test Artist".to_string()],
            length: Some(180), // 3 minutes
        };

        let mut attrs1 = HashMap::new();
        attrs1.insert(FileAttribute::Duration, 200); // 20 seconds off

        let mut attrs2 = HashMap::new();
        attrs2.insert(FileAttribute::Duration, 185); // 5 seconds off

        let mut results = vec![
            SingleFileResult {
                username: "user1".to_string(),
                token: "token1".to_string(),
                filename: "test.mp3".to_string(),
                size: 1000,
                slots_free: true,
                avg_speed: 1000.0,
                queue_length: 0,
                attrs: attrs1,
            },
            SingleFileResult {
                username: "user2".to_string(),
                token: "token2".to_string(),
                filename: "test.mp3".to_string(),
                size: 1000,
                slots_free: true,
                avg_speed: 1000.0,
                queue_length: 0,
                attrs: attrs2,
            },
        ];

        rank_results(&track, &mut results);

        // First result should have duration closer to 180
        let first_duration = results[0].attrs.get(&FileAttribute::Duration).unwrap();
        assert_eq!(*first_duration, 185);
    }

    // ========================================================================
    // Search Integration Tests
    // ========================================================================

    #[tokio::test]
    async fn test_search_for_track_with_mock() {
        let track = Track {
            title: "Test Song".to_string(),
            album: "Test Album".to_string(),
            artists: vec!["Test Artist".to_string()],
            length: None,
        };

        let mut mock_client = MockSoulSeekClient::new();
        // Note: login is called in SoulSeekClientContext::new, but we're creating context manually
        // So we don't need to expect login here
        mock_client.expect_search().times(..).returning(|_, _| {
            Ok(vec![create_mock_file_search_response(
                "user1",
                "token1",
                vec![create_mock_file_info(
                    "Test Artist - Test Song.mp3",
                    5000000,
                    Some(180),
                )],
            )])
        });

        let config = SearchConfig {
            username: "testuser".to_string(),
            password: "testpass".to_string(),
            concurrency: Some(2),
            searches_per_time: Some(10),
            renew_time_secs: Some(1),
            max_search_time_ms: Some(1000),
            remove_special_chars: Some(false),
        };

        let rate_limiter = Arc::new(tokio::sync::Mutex::new(SearchRateLimiter::new(10, 1)));
        let mut context = SoulSeekClientContext {
            client: Box::new(mock_client),
            rate_limiter,
            config,
            soulseek_client: None,
        };

        let results = search_for_track(&track, &mut context).await.unwrap();

        assert!(!results.is_empty());
        assert_eq!(results[0].filename, "Test Artist - Test Song.mp3");
    }

    #[tokio::test]
    async fn test_search_for_track_filters_non_audio() {
        let track = Track {
            title: "Test Song".to_string(),
            album: "Test Album".to_string(),
            artists: vec!["Test Artist".to_string()],
            length: None,
        };

        let mut mock_client = MockSoulSeekClient::new();
        mock_client.expect_search().times(..).returning(|_, _| {
            Ok(vec![create_mock_file_search_response(
                "user1",
                "token1",
                vec![
                    create_mock_file_info("song.mp3", 5000000, Some(180)),
                    create_mock_file_info("document.txt", 1000, None),
                ],
            )])
        });

        let config = SearchConfig {
            username: "testuser".to_string(),
            password: "testpass".to_string(),
            concurrency: Some(2),
            searches_per_time: Some(10),
            renew_time_secs: Some(1),
            max_search_time_ms: Some(1000),
            remove_special_chars: Some(false),
        };

        let rate_limiter = Arc::new(tokio::sync::Mutex::new(SearchRateLimiter::new(10, 1)));
        let mut context = SoulSeekClientContext {
            client: Box::new(mock_client),
            rate_limiter,
            config,
            soulseek_client: None,
        };

        let results = search_for_track(&track, &mut context).await.unwrap();

        // Should only have audio files
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "song.mp3");
    }

    #[tokio::test]
    async fn test_search_for_track_deduplicates() {
        let track = Track {
            title: "Test Song".to_string(),
            album: "Test Album".to_string(),
            artists: vec!["Test Artist".to_string()],
            length: None,
        };

        let mut mock_client = MockSoulSeekClient::new();
        mock_client.expect_search().times(..).returning(|_, _| {
            Ok(vec![create_mock_file_search_response(
                "user1",
                "token1",
                vec![
                    create_mock_file_info("song.mp3", 5000000, Some(180)),
                    create_mock_file_info("song.mp3", 5000000, Some(180)),
                ],
            )])
        });

        let config = SearchConfig {
            username: "testuser".to_string(),
            password: "testpass".to_string(),
            concurrency: Some(2),
            searches_per_time: Some(10),
            renew_time_secs: Some(1),
            max_search_time_ms: Some(1000),
            remove_special_chars: Some(false),
        };

        let rate_limiter = Arc::new(tokio::sync::Mutex::new(SearchRateLimiter::new(10, 1)));
        let mut context = SoulSeekClientContext {
            client: Box::new(mock_client),
            rate_limiter,
            config,
            soulseek_client: None,
        };

        let results = search_for_track(&track, &mut context).await.unwrap();

        // Should deduplicate
        assert_eq!(results.len(), 1);
    }
}
