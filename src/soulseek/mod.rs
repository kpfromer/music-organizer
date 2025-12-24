// TODO: Remove this once we have a proper API
#![allow(dead_code)]

pub mod types;
pub mod client;
pub mod search;
pub mod download;

// Re-export public API
pub use types::*;
pub use client::SoulSeekClientContext;
pub use search::search_for_track;
pub use download::download_file;

