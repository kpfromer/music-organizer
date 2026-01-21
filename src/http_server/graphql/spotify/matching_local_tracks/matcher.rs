//! Track matching library for comparing local music files with Spotify tracks.
//!
//! This library provides fuzzy matching between local music metadata and Spotify
//! track information, handling edge cases like Japanese characters, version
//! indicators, featuring artists, and varying metadata quality.

use serde::Serialize;
use std::collections::HashSet;
use unicode_normalization::UnicodeNormalization;

/// Represents a track with its metadata
#[derive(Debug, Clone, Serialize)]
pub struct Track {
    pub title: String,
    pub primary_artist: String,
    pub secondary_artists: Vec<String>,
    pub album: String,
    pub duration_ms: u32,
}

/// Normalized track data after preprocessing
#[derive(Debug, Clone)]
pub struct NormalizedTrack {
    /// Cleaned title without version indicators or featuring artists
    pub title: String,
    /// Original title for reference
    #[allow(dead_code)]
    pub original_title: String,
    /// Normalized primary artist
    pub primary_artist: String,
    /// All artists (primary + secondary + extracted from title)
    pub all_artists: HashSet<String>,
    /// Normalized album name
    pub album: String,
    /// Duration in milliseconds
    pub duration_ms: u32,
    /// Extracted version indicator (e.g., "remix", "live", "remastered")
    pub version_indicator: Option<String>,
}

/// Result of comparing two tracks
#[derive(Debug, Clone)]
pub struct MatchResult {
    pub confidence: MatchConfidence,
    #[allow(dead_code)]
    pub title_similarity: f64,
    #[allow(dead_code)]
    pub artist_similarity: f64,
    #[allow(dead_code)]
    pub album_similarity: f64,
    #[allow(dead_code)]
    pub duration_match: DurationMatch,
    #[allow(dead_code)]
    pub version_match: VersionMatch,
    /// Overall score from 0.0 to 1.0
    pub score: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchConfidence {
    /// Auto-match with high confidence
    High,
    /// Likely match, could benefit from review
    Medium,
    /// Possible match, needs review
    Low,
    /// Not a match
    NoMatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurationMatch {
    /// Within tight tolerance - same version
    Exact,
    /// Within loose tolerance - possible different version
    Close,
    /// Outside tolerance - likely different track
    Mismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionMatch {
    /// Both have same version indicator or both have none
    Match,
    /// Version indicators conflict
    Mismatch,
    /// One has indicator, other doesn't
    Ambiguous,
}

// =============================================================================
// Normalization
// =============================================================================

/// Known version indicators to extract from titles
const VERSION_INDICATORS: &[&str] = &[
    "remix",
    "remixed",
    "remastered",
    "remaster",
    "live",
    "acoustic",
    "unplugged",
    "radio edit",
    "radio version",
    "single version",
    "album version",
    "extended mix",
    "extended version",
    "extended",
    "instrumental",
    "karaoke",
    "demo",
    "original mix",
    "club mix",
    "dub mix",
    "edit",
    "clean",
    "explicit",
    "deluxe",
    "bonus track",
];

/// Patterns that indicate featuring artists in titles
const FEATURING_PATTERNS: &[&str] = &[
    "feat.",
    "feat ",
    "ft.",
    "ft ",
    "featuring",
    "with ",
    "duet with",
    "vs.",
    "vs ",
    "versus",
    "&",
];

/// Normalize a string for comparison
///
/// Applies: NFKC normalization, lowercase, punctuation removal, whitespace collapse
pub fn normalize_string(s: &str) -> String {
    let normalized: String = s.nfkc().collect();

    normalized
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c.is_whitespace() {
                c
            } else if c == '\'' {
                // Preserve apostrophes as empty (don't -> dont)
                '\0'
            } else {
                ' '
            }
        })
        .filter(|&c| c != '\0')
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Extract content from parentheses and brackets
fn extract_parenthetical_content(s: &str) -> (String, Vec<String>) {
    let mut result = String::new();
    let mut extracted = Vec::new();
    let mut depth = 0;
    let mut current_paren = String::new();
    let mut paren_char = ' ';

    for c in s.chars() {
        match c {
            '(' | '[' | '【' | '「' if depth == 0 => {
                depth = 1;
                paren_char = c;
                current_paren.clear();
            }
            ')' if depth == 1 && paren_char == '(' => {
                depth = 0;
                if !current_paren.trim().is_empty() {
                    extracted.push(current_paren.trim().to_string());
                }
                current_paren.clear();
            }
            ']' if depth == 1 && paren_char == '[' => {
                depth = 0;
                if !current_paren.trim().is_empty() {
                    extracted.push(current_paren.trim().to_string());
                }
                current_paren.clear();
            }
            '】' if depth == 1 && paren_char == '【' => {
                depth = 0;
                if !current_paren.trim().is_empty() {
                    extracted.push(current_paren.trim().to_string());
                }
                current_paren.clear();
            }
            '」' if depth == 1 && paren_char == '「' => {
                depth = 0;
                if !current_paren.trim().is_empty() {
                    extracted.push(current_paren.trim().to_string());
                }
                current_paren.clear();
            }
            _ if depth > 0 => {
                current_paren.push(c);
            }
            _ => {
                result.push(c);
            }
        }
    }

    (result.trim().to_string(), extracted)
}

/// Extract version indicator from parenthetical content
fn extract_version_indicator(parentheticals: &[String]) -> Option<String> {
    for content in parentheticals {
        let lower = content.to_lowercase();
        for indicator in VERSION_INDICATORS {
            if lower.contains(indicator) {
                // Normalize the version indicator
                // "Remastered 2021" and "Remastered" should match
                return Some(indicator.to_string());
            }
        }
    }
    None
}

/// Extract featuring artists from a string
fn extract_featuring_artists(s: &str) -> Vec<String> {
    let lower = s.to_lowercase();
    let mut artists = Vec::new();

    for pattern in FEATURING_PATTERNS {
        if let Some(idx) = lower.find(pattern) {
            let after = &s[idx + pattern.len()..];
            // Take until end or next delimiter
            let artist = after
                .split([',', '&', ')', ']'])
                .next()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());

            if let Some(a) = artist {
                artists.push(normalize_string(&a));
            }
        }
    }

    artists
}

/// Remove leading "the " from a string
fn strip_leading_the(s: &str) -> String {
    let lower = s.to_lowercase();
    if lower.starts_with("the ") {
        s[4..].to_string()
    } else {
        s.to_string()
    }
}

/// Normalize a track for matching
pub fn normalize_track(track: &Track) -> NormalizedTrack {
    // Extract parenthetical content from title
    let (title_without_parens, parentheticals) = extract_parenthetical_content(&track.title);

    // Extract version indicator
    let version_indicator = extract_version_indicator(&parentheticals);

    // Extract featuring artists from title
    let feat_from_title = extract_featuring_artists(&track.title);

    // Also check parentheticals for featuring artists
    let feat_from_parens: Vec<String> = parentheticals
        .iter()
        .flat_map(|p| extract_featuring_artists(p))
        .collect();

    // Build complete artist set
    let mut all_artists: HashSet<String> = HashSet::new();
    all_artists.insert(normalize_string(&strip_leading_the(&track.primary_artist)));
    for artist in &track.secondary_artists {
        all_artists.insert(normalize_string(&strip_leading_the(artist)));
    }
    for artist in feat_from_title {
        all_artists.insert(artist);
    }
    for artist in feat_from_parens {
        all_artists.insert(artist);
    }

    // Normalize title (also strip featuring from the cleaned title)
    let mut clean_title = title_without_parens.clone();
    for pattern in FEATURING_PATTERNS {
        if let Some(idx) = clean_title.to_lowercase().find(pattern) {
            clean_title = clean_title[..idx].to_string();
        }
    }

    NormalizedTrack {
        title: normalize_string(&strip_leading_the(&clean_title)),
        original_title: track.title.clone(),
        primary_artist: normalize_string(&strip_leading_the(&track.primary_artist)),
        all_artists,
        album: normalize_string(&strip_leading_the(&track.album)),
        duration_ms: track.duration_ms,
        version_indicator,
    }
}

// =============================================================================
// String Similarity
// =============================================================================

/// Calculate Jaro-Winkler similarity between two strings
///
/// Returns a value between 0.0 (completely different) and 1.0 (identical)
pub fn jaro_winkler_similarity(s1: &str, s2: &str) -> f64 {
    if s1 == s2 {
        return 1.0;
    }
    if s1.is_empty() || s2.is_empty() {
        return 0.0;
    }

    let jaro = jaro_similarity(s1, s2);

    // Winkler modification: boost for common prefix
    let prefix_len = s1
        .chars()
        .zip(s2.chars())
        .take(4)
        .take_while(|(a, b)| a == b)
        .count();

    let winkler_boost = 0.1 * prefix_len as f64 * (1.0 - jaro);

    jaro + winkler_boost
}

/// Calculate basic Jaro similarity
fn jaro_similarity(s1: &str, s2: &str) -> f64 {
    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    let len1 = s1_chars.len();
    let len2 = s2_chars.len();

    if len1 == 0 && len2 == 0 {
        return 1.0;
    }

    let match_distance = (len1.max(len2) / 2).saturating_sub(1);

    let mut s1_matches = vec![false; len1];
    let mut s2_matches = vec![false; len2];

    let mut matches = 0;
    let mut transpositions = 0;

    // Find matches
    for i in 0..len1 {
        let start = i.saturating_sub(match_distance);
        let end = (i + match_distance + 1).min(len2);

        for j in start..end {
            if s2_matches[j] || s1_chars[i] != s2_chars[j] {
                continue;
            }
            s1_matches[i] = true;
            s2_matches[j] = true;
            matches += 1;
            break;
        }
    }

    if matches == 0 {
        return 0.0;
    }

    // Count transpositions
    let mut k = 0;
    for i in 0..len1 {
        if !s1_matches[i] {
            continue;
        }
        while !s2_matches[k] {
            k += 1;
        }
        if s1_chars[i] != s2_chars[k] {
            transpositions += 1;
        }
        k += 1;
    }

    let matches_f = matches as f64;
    let len1_f = len1 as f64;
    let len2_f = len2 as f64;
    let transpositions_f = (transpositions / 2) as f64;

    (matches_f / len1_f + matches_f / len2_f + (matches_f - transpositions_f) / matches_f) / 3.0
}

/// Token-based similarity - handles word reordering
///
/// "The Quick Brown Fox" vs "Quick Brown Fox, The" will score highly
pub fn token_similarity(s1: &str, s2: &str) -> f64 {
    let tokens1: HashSet<&str> = s1.split_whitespace().collect();
    let tokens2: HashSet<&str> = s2.split_whitespace().collect();

    if tokens1.is_empty() && tokens2.is_empty() {
        return 1.0;
    }
    if tokens1.is_empty() || tokens2.is_empty() {
        return 0.0;
    }

    let intersection = tokens1.intersection(&tokens2).count();
    let union = tokens1.union(&tokens2).count();

    intersection as f64 / union as f64
}

/// Combined similarity using both Jaro-Winkler and token-based
pub fn combined_string_similarity(s1: &str, s2: &str) -> f64 {
    let jw = jaro_winkler_similarity(s1, s2);
    let token = token_similarity(s1, s2);

    // Take the higher of the two, but give slight preference to Jaro-Winkler
    // for character-level accuracy
    jw.max(token * 0.95)
}

// =============================================================================
// Duration Matching
// =============================================================================

/// Check if durations match within tolerance
///
/// Uses percentage-based tolerance with a minimum floor
pub fn check_duration_match(dur1_ms: u32, dur2_ms: u32) -> DurationMatch {
    let shorter = dur1_ms.min(dur2_ms);
    let diff_ms = (dur1_ms as i64 - dur2_ms as i64).unsigned_abs() as u32;

    // Tight tolerance: max(5 seconds, 3% of shorter)
    let tight_tolerance_ms = (5000_u32).max((shorter as f64 * 0.03) as u32);

    // Loose tolerance: max(15 seconds, 8% of shorter)
    let loose_tolerance_ms = (15000_u32).max((shorter as f64 * 0.08) as u32);

    if diff_ms <= tight_tolerance_ms {
        DurationMatch::Exact
    } else if diff_ms <= loose_tolerance_ms {
        DurationMatch::Close
    } else {
        DurationMatch::Mismatch
    }
}

// =============================================================================
// Artist Matching
// =============================================================================

/// Compare artists between two tracks
pub fn compare_artists(local: &NormalizedTrack, spotify: &NormalizedTrack) -> f64 {
    // Primary artist match is most important
    let primary_sim = jaro_winkler_similarity(&local.primary_artist, &spotify.primary_artist);

    // Check overlap of all artists
    let overlap_score = if local.all_artists.is_empty() || spotify.all_artists.is_empty() {
        0.0
    } else {
        let mut total_best_matches = 0.0;
        for local_artist in &local.all_artists {
            let best = spotify
                .all_artists
                .iter()
                .map(|s| jaro_winkler_similarity(local_artist, s))
                .fold(0.0_f64, |a, b| a.max(b));
            total_best_matches += best;
        }
        total_best_matches / local.all_artists.len() as f64
    };

    // Weight primary artist heavily
    primary_sim * 0.7 + overlap_score * 0.3
}

// =============================================================================
// Version Indicator Matching
// =============================================================================

/// Compare version indicators between tracks
pub fn compare_version_indicators(
    local: &NormalizedTrack,
    spotify: &NormalizedTrack,
) -> VersionMatch {
    match (&local.version_indicator, &spotify.version_indicator) {
        (None, None) => VersionMatch::Match,
        (Some(a), Some(b)) if a == b => VersionMatch::Match,
        (Some(_), Some(_)) => VersionMatch::Mismatch,
        _ => VersionMatch::Ambiguous,
    }
}

// =============================================================================
// Main Matching Algorithm
// =============================================================================

/// Compare two normalized tracks and return a match result
pub fn compare_tracks(local: &NormalizedTrack, spotify: &NormalizedTrack) -> MatchResult {
    let title_similarity = combined_string_similarity(&local.title, &spotify.title);
    let artist_similarity = compare_artists(local, spotify);
    let album_similarity = combined_string_similarity(&local.album, &spotify.album);
    let duration_match = check_duration_match(local.duration_ms, spotify.duration_ms);
    let version_match = compare_version_indicators(local, spotify);

    // Calculate overall score
    let mut score = title_similarity * 0.45 + artist_similarity * 0.40;

    // Album boosts but doesn't penalize
    if album_similarity > 0.8 {
        score += 0.10 * album_similarity;
    }

    // Duration affects confidence
    let duration_factor = match duration_match {
        DurationMatch::Exact => 1.0,
        DurationMatch::Close => 0.85,
        DurationMatch::Mismatch => 0.5,
    };
    score *= duration_factor;

    // Version indicator can penalize
    let version_factor = match version_match {
        VersionMatch::Match => 1.0,
        VersionMatch::Ambiguous => 0.9,
        VersionMatch::Mismatch => 0.6,
    };
    score *= version_factor;

    // Determine confidence level
    let confidence = if score >= 0.85
        && matches!(duration_match, DurationMatch::Exact)
        && matches!(version_match, VersionMatch::Match)
    {
        MatchConfidence::High
    } else if score >= 0.70
        && !matches!(duration_match, DurationMatch::Mismatch)
        && !matches!(version_match, VersionMatch::Mismatch)
    {
        MatchConfidence::Medium
    } else if score >= 0.50 {
        MatchConfidence::Low
    } else {
        MatchConfidence::NoMatch
    };

    MatchResult {
        confidence,
        title_similarity,
        artist_similarity,
        album_similarity,
        duration_match,
        version_match,
        score,
    }
}

/// Find the best matching Spotify track for a local track
///
/// Returns candidates sorted by score, filtered to plausible matches
pub fn find_matches(
    local: &Track,
    spotify_tracks: &[Track],
    min_artist_threshold: f64,
) -> Vec<(usize, NormalizedTrack, MatchResult)> {
    let local_normalized = normalize_track(local);

    let mut results: Vec<(usize, NormalizedTrack, MatchResult)> = spotify_tracks
        .iter()
        .enumerate()
        .map(|(idx, spotify)| {
            let spotify_normalized = normalize_track(spotify);
            (idx, spotify_normalized)
        })
        // First pass: filter by artist similarity
        .filter(|(_, spotify_normalized)| {
            jaro_winkler_similarity(
                &local_normalized.primary_artist,
                &spotify_normalized.primary_artist,
            ) >= min_artist_threshold
        })
        // Second pass: filter by duration (loose tolerance)
        .filter(|(_, spotify_normalized)| {
            !matches!(
                check_duration_match(local_normalized.duration_ms, spotify_normalized.duration_ms),
                DurationMatch::Mismatch
            )
        })
        // Score remaining candidates
        .map(|(idx, spotify_normalized)| {
            let result = compare_tracks(&local_normalized, &spotify_normalized);
            (idx, spotify_normalized, result)
        })
        // Filter out no-matches
        .filter(|(_, _, result)| !matches!(result.confidence, MatchConfidence::NoMatch))
        .collect();

    // Sort by score descending
    results.sort_by(|a, b| b.2.score.partial_cmp(&a.2.score).unwrap());

    results
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_string() {
        assert_eq!(normalize_string("Hello World!"), "hello world");
        assert_eq!(normalize_string("Don't Stop"), "dont stop");
        assert_eq!(normalize_string("  Multiple   Spaces  "), "multiple spaces");
        // Full-width characters
        assert_eq!(normalize_string("Ｆｕｌｌ　Ｗｉｄｔｈ"), "full width");
    }

    #[test]
    fn test_extract_parenthetical() {
        let (title, parens) =
            extract_parenthetical_content("Song Title (feat. Artist) [Remastered]");
        assert_eq!(title, "Song Title");
        assert_eq!(parens, vec!["feat. Artist", "Remastered"]);
    }

    #[test]
    fn test_version_indicator_extraction() {
        let track = Track {
            title: "Song (Remastered 2021)".to_string(),
            primary_artist: "Artist".to_string(),
            secondary_artists: vec![],
            album: "Album".to_string(),
            duration_ms: 180000,
        };
        let normalized = normalize_track(&track);
        assert_eq!(normalized.version_indicator, Some("remastered".to_string()));
    }

    #[test]
    fn test_featuring_extraction() {
        let track = Track {
            title: "Song (feat. Guest Artist)".to_string(),
            primary_artist: "Main Artist".to_string(),
            secondary_artists: vec![],
            album: "Album".to_string(),
            duration_ms: 180000,
        };
        let normalized = normalize_track(&track);
        assert!(normalized.all_artists.contains("guest artist"));
    }

    #[test]
    fn test_jaro_winkler() {
        assert!((jaro_winkler_similarity("hello", "hello") - 1.0).abs() < 0.001);
        assert!(jaro_winkler_similarity("hello", "hallo") > 0.8);
        assert!(jaro_winkler_similarity("hello", "world") < 0.5);
    }

    #[test]
    fn test_token_similarity() {
        assert!((token_similarity("quick brown fox", "brown fox quick") - 1.0).abs() < 0.001);
        assert!(token_similarity("the song", "song the") > 0.9);
    }

    #[test]
    fn test_duration_match() {
        // Same duration
        assert!(matches!(
            check_duration_match(180000, 180000),
            DurationMatch::Exact
        ));

        // Within 5 seconds
        assert!(matches!(
            check_duration_match(180000, 183000),
            DurationMatch::Exact
        ));

        // Within loose tolerance
        assert!(matches!(
            check_duration_match(180000, 190000),
            DurationMatch::Close
        ));

        // Outside tolerance
        assert!(matches!(
            check_duration_match(180000, 220000),
            DurationMatch::Mismatch
        ));
    }

    #[test]
    fn test_full_match() {
        let local = Track {
            title: "Bohemian Rhapsody".to_string(),
            primary_artist: "Queen".to_string(),
            secondary_artists: vec![],
            album: "A Night at the Opera".to_string(),
            duration_ms: 354000,
        };

        let spotify = Track {
            title: "Bohemian Rhapsody (Remastered 2011)".to_string(),
            primary_artist: "Queen".to_string(),
            secondary_artists: vec![],
            album: "A Night At The Opera (Deluxe Edition)".to_string(),
            duration_ms: 355000,
        };

        let local_norm = normalize_track(&local);
        let spotify_norm = normalize_track(&spotify);
        let result = compare_tracks(&local_norm, &spotify_norm);

        // Should be a strong match despite version differences
        assert!(result.score > 0.7);
        assert!(matches!(result.version_match, VersionMatch::Ambiguous));
    }

    #[test]
    fn test_different_versions() {
        let original = Track {
            title: "Song".to_string(),
            primary_artist: "Artist".to_string(),
            secondary_artists: vec![],
            album: "Album".to_string(),
            duration_ms: 180000,
        };

        let live = Track {
            title: "Song (Live)".to_string(),
            primary_artist: "Artist".to_string(),
            secondary_artists: vec![],
            album: "Live Album".to_string(),
            duration_ms: 240000, // Live versions often longer
        };

        let original_norm = normalize_track(&original);
        let live_norm = normalize_track(&live);
        let result = compare_tracks(&original_norm, &live_norm);

        // Should flag as different versions
        assert!(matches!(result.version_match, VersionMatch::Ambiguous));
        assert!(matches!(result.duration_match, DurationMatch::Mismatch));
    }
}
