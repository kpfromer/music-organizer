// TODO: Remove this once we have a proper API
#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use color_eyre::Result;
use futures::future::join_all;
use regex::Regex;
use tokio::sync::Semaphore;
use unaccent::unaccent;

use crate::soulseek::client::{FileSearchResponse, SoulSeekClientContext, SoulSeekClientTrait};
use crate::soulseek::types::{FileAttribute, SingleFileResult, Track};

// ============================================================================
// Private Types
// ============================================================================

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
    context: &Arc<Mutex<SoulSeekClientContext>>,
) -> Result<Vec<SingleFileResult>> {
    log::debug!(
        "Starting search for track: '{}' by '{}'",
        track.title,
        track.artists.join(", ")
    );

    // Lock context to read config
    let ctx = context.lock().unwrap();
    let remove_special = ctx.config.remove_special_chars.unwrap_or(false);
    let concurrency = ctx.config.concurrency.unwrap_or(2);
    let max_search_time = ctx.config.max_search_time_ms.unwrap_or(8000);
    drop(ctx); // Release lock before async operations

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
            let context = context.clone();
            let sem = semaphore.clone();
            async move {
                let _permit = sem.acquire().await.unwrap();

                // Lock context to access rate_limiter and wrapper
                let ctx = context.lock().unwrap();

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
