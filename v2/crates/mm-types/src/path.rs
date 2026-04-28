//! UTF-8 file paths persisted as TEXT.
//!
//! All on-disk paths in `mm-entities` are UTF-8 by construction (the watch
//! folder rejects non-UTF-8 names at preflight per TDD 03 §Stage 1), so we
//! use `camino::Utf8PathBuf` instead of `std::path::PathBuf`. That gives us
//! cheap `&str` access without ever calling `.to_string_lossy()`.
//!
//! The two newtypes encode a relative-vs-absolute invariant that the schema
//! cannot express: `file.relative_path` and `cover_art.relative_path` must
//! be relative to `MM_MUSIC_ROOT`; `import_progress.source_path` and
//! `unimportable_file.failed_path` are absolute paths from the watcher.

use std::str::FromStr;

use camino::{Utf8Path, Utf8PathBuf};
use sea_orm::{
    ColIdx, QueryResult, TryGetError, TryGetable,
    sea_query::{ArrayType, ColumnType, Nullable, StringLen, Value, ValueType, ValueTypeErr},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PathError {
    #[error("expected a relative path, got absolute: {0}")]
    NotRelative(String),
    #[error("relative path may not contain `..`: {0}")]
    ParentEscape(String),
    #[error("expected an absolute path, got relative: {0}")]
    NotAbsolute(String),
    #[error("path is empty")]
    Empty,
}

/// A path relative to `MM_MUSIC_ROOT`. Forbids `..` segments so callers
/// can join it against the music root without checking for escapes.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RelativePath(Utf8PathBuf);

impl RelativePath {
    pub fn new(path: impl Into<Utf8PathBuf>) -> Result<Self, PathError> {
        let path = path.into();
        if path.as_str().is_empty() {
            return Err(PathError::Empty);
        }
        if path.is_absolute() {
            return Err(PathError::NotRelative(path.into_string()));
        }
        if path
            .components()
            .any(|c| matches!(c, camino::Utf8Component::ParentDir))
        {
            return Err(PathError::ParentEscape(path.into_string()));
        }
        Ok(Self(path))
    }

    pub fn as_path(&self) -> &Utf8Path {
        &self.0
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn into_inner(self) -> Utf8PathBuf {
        self.0
    }
}

impl FromStr for RelativePath {
    type Err = PathError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(Utf8PathBuf::from(s))
    }
}

impl std::fmt::Display for RelativePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<RelativePath> for Value {
    fn from(p: RelativePath) -> Self {
        Value::String(Some(Box::new(p.0.into_string())))
    }
}

impl TryGetable for RelativePath {
    fn try_get_by<I: ColIdx>(res: &QueryResult, idx: I) -> Result<Self, TryGetError> {
        let s = String::try_get_by(res, idx)?;
        Self::new(Utf8PathBuf::from(s))
            .map_err(|e| TryGetError::DbErr(sea_orm::DbErr::Type(e.to_string())))
    }
}

impl ValueType for RelativePath {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        let s = <String as ValueType>::try_from(v)?;
        Self::new(Utf8PathBuf::from(s)).map_err(|_| ValueTypeErr)
    }
    fn type_name() -> String {
        "RelativePath".to_owned()
    }
    fn array_type() -> ArrayType {
        ArrayType::String
    }
    fn column_type() -> ColumnType {
        ColumnType::String(StringLen::None)
    }
}

impl Nullable for RelativePath {
    fn null() -> Value {
        Value::String(None)
    }
}

/// An absolute UTF-8 path. Used for `import_progress.source_path` and
/// `unimportable_file.failed_path` — both are taken from the watcher's
/// canonical path before any move.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AbsolutePath(Utf8PathBuf);

impl AbsolutePath {
    pub fn new(path: impl Into<Utf8PathBuf>) -> Result<Self, PathError> {
        let path = path.into();
        if path.as_str().is_empty() {
            return Err(PathError::Empty);
        }
        if !path.is_absolute() {
            return Err(PathError::NotAbsolute(path.into_string()));
        }
        Ok(Self(path))
    }

    pub fn as_path(&self) -> &Utf8Path {
        &self.0
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn into_inner(self) -> Utf8PathBuf {
        self.0
    }
}

impl FromStr for AbsolutePath {
    type Err = PathError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(Utf8PathBuf::from(s))
    }
}

impl std::fmt::Display for AbsolutePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<AbsolutePath> for Value {
    fn from(p: AbsolutePath) -> Self {
        Value::String(Some(Box::new(p.0.into_string())))
    }
}

impl TryGetable for AbsolutePath {
    fn try_get_by<I: ColIdx>(res: &QueryResult, idx: I) -> Result<Self, TryGetError> {
        let s = String::try_get_by(res, idx)?;
        Self::new(Utf8PathBuf::from(s))
            .map_err(|e| TryGetError::DbErr(sea_orm::DbErr::Type(e.to_string())))
    }
}

impl ValueType for AbsolutePath {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        let s = <String as ValueType>::try_from(v)?;
        Self::new(Utf8PathBuf::from(s)).map_err(|_| ValueTypeErr)
    }
    fn type_name() -> String {
        "AbsolutePath".to_owned()
    }
    fn array_type() -> ArrayType {
        ArrayType::String
    }
    fn column_type() -> ColumnType {
        ColumnType::String(StringLen::None)
    }
}

impl Nullable for AbsolutePath {
    fn null() -> Value {
        Value::String(None)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn relative_accepts_simple() {
        let p = RelativePath::from_str("artist/album/01.flac").unwrap();
        assert_eq!(p.as_str(), "artist/album/01.flac");
    }

    #[test]
    fn relative_rejects_absolute() {
        assert!(matches!(
            RelativePath::from_str("/abs/path"),
            Err(PathError::NotRelative(_))
        ));
    }

    #[test]
    fn relative_rejects_parent() {
        assert!(matches!(
            RelativePath::from_str("a/../b"),
            Err(PathError::ParentEscape(_))
        ));
    }

    #[test]
    fn absolute_accepts_root() {
        assert!(AbsolutePath::from_str("/tmp/foo").is_ok());
    }

    #[test]
    fn absolute_rejects_relative() {
        assert!(matches!(
            AbsolutePath::from_str("foo"),
            Err(PathError::NotAbsolute(_))
        ));
    }

    #[test]
    fn relative_value_roundtrip() {
        let p = RelativePath::from_str("a/b.flac").unwrap();
        let v: Value = p.clone().into();
        assert_eq!(<RelativePath as ValueType>::try_from(v).unwrap(), p);
    }
}
