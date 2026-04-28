//! Domain newtypes shared across `music-manager` crates.
//!
//! These types replace the bare primitives that would otherwise appear on
//! entity models. See `tdds/implementation/conventions.md` §4a for the
//! "strong types over primitives" rule and the rationale.
//!
//! Every public type implements:
//! - `From<Self> for sea_orm::Value` and `sea_orm::TryGetable` so SeaORM
//!   round-trips it transparently.
//! - `sea_query::ValueType` and `sea_query::Nullable` so it works as
//!   `Option<Self>` in entity columns.
//! - `serde::Serialize` and `serde::Deserialize`.
//! - `Display` and `FromStr` where there's a canonical text form.
//!
//! Canonical types:
//! - [`Timestamp`] — `chrono::DateTime<Utc>` ↔ `INTEGER` unix seconds.
//! - [`DurationMs`] — `std::time::Duration` ↔ `INTEGER` milliseconds.
//! - [`DurationSecs`] — `std::time::Duration` ↔ `INTEGER` seconds.
//! - [`RelativePath`] — `Utf8PathBuf` (relative, no `..`) ↔ `TEXT`.
//! - [`AbsolutePath`] — `Utf8PathBuf` (absolute) ↔ `TEXT`.
//! - [`Mbid`] — `uuid::Uuid` (MusicBrainz identifier) ↔ `TEXT`.
//! - [`Isrc`] — 12-char ISRC code ↔ `TEXT`.
//! - [`ImageMimeType`] — JPEG/PNG/GIF/WebP/Other ↔ `TEXT`.
//! - [`AudioFormat`] — FLAC/MP3/AAC/Ogg/Opus/WAV/M4A/Other ↔ `TEXT`.
//! - [`AudioCodec`] — FLAC/MP3/AAC/Vorbis/Opus/PCM/Other ↔ `TEXT`.

mod duration;
mod format;
mod ids;
mod path;
mod timestamp;

pub use duration::{DurationMs, DurationOutOfRange, DurationSecs};
pub use format::{AudioCodec, AudioFormat, ImageMimeType};
pub use ids::{Isrc, IsrcParseError, Mbid, MbidParseError};
pub use path::{AbsolutePath, PathError, RelativePath};
pub use timestamp::{Timestamp, TimestampError};
