//! External identifier newtypes: `Mbid`, `Isrc`.

use std::str::FromStr;

use sea_orm::{
    ColIdx, DbErr, QueryResult, TryFromU64, TryGetError, TryGetable,
    sea_query::{ArrayType, ColumnType, Nullable, StringLen, Value, ValueType, ValueTypeErr},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum MbidParseError {
    #[error("invalid MusicBrainz id: {0}")]
    Invalid(String),
}

/// MusicBrainz identifier (UUID). Stored on disk in canonical hyphenated
/// form; parsing is lenient (accepts hyphenated or simple).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Mbid(Uuid);

impl Mbid {
    pub const fn from_uuid(u: Uuid) -> Self {
        Self(u)
    }

    pub fn parse(s: &str) -> Result<Self, MbidParseError> {
        Uuid::parse_str(s)
            .map(Self)
            .map_err(|_| MbidParseError::Invalid(s.to_owned()))
    }

    pub fn uuid(self) -> Uuid {
        self.0
    }
}

impl FromStr for Mbid {
    type Err = MbidParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl std::fmt::Display for Mbid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Canonical hyphenated form, lowercase.
        write!(f, "{}", self.0.as_hyphenated())
    }
}

impl From<Mbid> for Value {
    fn from(m: Mbid) -> Self {
        Value::String(Some(Box::new(m.to_string())))
    }
}

impl TryGetable for Mbid {
    fn try_get_by<I: ColIdx>(res: &QueryResult, idx: I) -> Result<Self, TryGetError> {
        let s = String::try_get_by(res, idx)?;
        Self::parse(&s).map_err(|e| TryGetError::DbErr(sea_orm::DbErr::Type(e.to_string())))
    }
}

impl ValueType for Mbid {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        let s = <String as ValueType>::try_from(v)?;
        Self::parse(&s).map_err(|_| ValueTypeErr)
    }
    fn type_name() -> String {
        "Mbid".to_owned()
    }
    fn array_type() -> ArrayType {
        ArrayType::String
    }
    fn column_type() -> ColumnType {
        ColumnType::String(StringLen::None)
    }
}

impl Nullable for Mbid {
    fn null() -> Value {
        Value::String(None)
    }
}

impl TryFromU64 for Mbid {
    fn try_from_u64(_: u64) -> Result<Self, DbErr> {
        Err(DbErr::ConvertFromU64("Mbid"))
    }
}

#[derive(Debug, Error)]
pub enum IsrcParseError {
    #[error("ISRC must be 12 alphanumeric characters, got {0:?}")]
    BadLength(String),
    #[error("ISRC contains non-alphanumeric characters: {0:?}")]
    BadCharacters(String),
}

/// International Standard Recording Code. Stored as 12 uppercase
/// alphanumeric characters with no separators (`CCXXXYYNNNNN`).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Isrc(String);

impl Isrc {
    pub fn parse(s: &str) -> Result<Self, IsrcParseError> {
        let normalized: String = s
            .chars()
            .filter(|c| !c.is_whitespace() && *c != '-')
            .map(|c| c.to_ascii_uppercase())
            .collect();
        if normalized.len() != 12 {
            return Err(IsrcParseError::BadLength(s.to_owned()));
        }
        if !normalized.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(IsrcParseError::BadCharacters(s.to_owned()));
        }
        Ok(Self(normalized))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl FromStr for Isrc {
    type Err = IsrcParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl std::fmt::Display for Isrc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<Isrc> for Value {
    fn from(i: Isrc) -> Self {
        Value::String(Some(Box::new(i.0)))
    }
}

impl TryGetable for Isrc {
    fn try_get_by<I: ColIdx>(res: &QueryResult, idx: I) -> Result<Self, TryGetError> {
        let s = String::try_get_by(res, idx)?;
        Self::parse(&s).map_err(|e| TryGetError::DbErr(sea_orm::DbErr::Type(e.to_string())))
    }
}

impl ValueType for Isrc {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        let s = <String as ValueType>::try_from(v)?;
        Self::parse(&s).map_err(|_| ValueTypeErr)
    }
    fn type_name() -> String {
        "Isrc".to_owned()
    }
    fn array_type() -> ArrayType {
        ArrayType::String
    }
    fn column_type() -> ColumnType {
        ColumnType::String(StringLen::None)
    }
}

impl Nullable for Isrc {
    fn null() -> Value {
        Value::String(None)
    }
}

impl TryFromU64 for Isrc {
    fn try_from_u64(_: u64) -> Result<Self, DbErr> {
        Err(DbErr::ConvertFromU64("Isrc"))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    const SAMPLE_MBID: &str = "f4c1d6f4-1234-4abc-8def-0123456789ab";

    #[test]
    fn mbid_roundtrip() {
        let m = Mbid::parse(SAMPLE_MBID).unwrap();
        let v: Value = m.into();
        assert_eq!(<Mbid as ValueType>::try_from(v).unwrap(), m);
    }

    #[test]
    fn mbid_rejects_garbage() {
        assert!(Mbid::parse("not-a-uuid").is_err());
    }

    #[test]
    fn isrc_canonicalizes() {
        let i = Isrc::parse("us-rc1-17-00001").unwrap();
        assert_eq!(i.as_str(), "USRC11700001");
    }

    #[test]
    fn isrc_rejects_short() {
        assert!(Isrc::parse("ABC").is_err());
    }

    #[test]
    fn isrc_rejects_symbols() {
        assert!(Isrc::parse("ABC!@#$%^&*()").is_err());
    }
}
