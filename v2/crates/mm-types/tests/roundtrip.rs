//! End-to-end check that every newtype round-trips through the SeaORM
//! `Value` representation we'll be storing in SQLite.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use mm_types::{
    AbsolutePath, AudioCodec, AudioFormat, DurationMs, DurationSecs, ImageMimeType, Isrc, Mbid,
    RelativePath, Timestamp,
};
use sea_orm::sea_query::{Value, ValueType};

fn roundtrip<T: Clone + PartialEq + std::fmt::Debug + Into<Value> + ValueType>(t: T) {
    let v: Value = t.clone().into();
    let back: T = ValueType::try_from(v).expect("value round-trip");
    assert_eq!(back, t);
}

#[test]
fn all_types_roundtrip() {
    roundtrip(Timestamp::from_unix_seconds(1_700_000_000).unwrap());
    roundtrip(DurationMs::from_millis_i64(180_000).unwrap());
    roundtrip(DurationSecs::from_secs_i32(180).unwrap());
    roundtrip(RelativePath::new("artist/album/01.flac").unwrap());
    roundtrip(AbsolutePath::new("/music/inbox/file.flac").unwrap());
    roundtrip(Mbid::parse("f4c1d6f4-1234-4abc-8def-0123456789ab").unwrap());
    roundtrip(Isrc::parse("USRC11700001").unwrap());
    roundtrip(ImageMimeType::Jpeg);
    roundtrip(ImageMimeType::Other("image/avif".into()));
    roundtrip(AudioFormat::Flac);
    roundtrip(AudioCodec::Vorbis);
}
