// TODO: Remove this once we have a proper API
#![allow(dead_code)]

use std::collections::HashMap;

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
