//! Wall-clock timestamps stored as INTEGER unix seconds.

use chrono::{DateTime, TimeZone, Utc};
use sea_orm::{
    ColIdx, QueryResult, TryGetError, TryGetable,
    sea_query::{ArrayType, ColumnType, Nullable, Value, ValueType, ValueTypeErr},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TimestampError {
    #[error("unix seconds value out of range: {0}")]
    OutOfRange(i64),
}

/// A UTC timestamp persisted as `INTEGER` unix seconds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(DateTime<Utc>);

impl Timestamp {
    /// Current wall-clock time, truncated to seconds.
    pub fn now() -> Self {
        Self::from_unix_seconds(Utc::now().timestamp())
            .unwrap_or_else(|_| Self(DateTime::<Utc>::from_timestamp(0, 0).unwrap_or_default()))
    }

    pub fn from_unix_seconds(secs: i64) -> Result<Self, TimestampError> {
        Utc.timestamp_opt(secs, 0)
            .single()
            .map(Self)
            .ok_or(TimestampError::OutOfRange(secs))
    }

    pub fn from_datetime(dt: DateTime<Utc>) -> Self {
        Self(
            DateTime::<Utc>::from_timestamp(dt.timestamp(), 0)
                .unwrap_or_else(|| DateTime::<Utc>::from_timestamp(0, 0).unwrap_or_default()),
        )
    }

    pub fn unix_seconds(self) -> i64 {
        self.0.timestamp()
    }

    pub fn datetime(self) -> DateTime<Utc> {
        self.0
    }
}

impl From<Timestamp> for DateTime<Utc> {
    fn from(t: Timestamp) -> Self {
        t.0
    }
}

impl From<Timestamp> for Value {
    fn from(t: Timestamp) -> Self {
        Value::BigInt(Some(t.unix_seconds()))
    }
}

impl TryGetable for Timestamp {
    fn try_get_by<I: ColIdx>(res: &QueryResult, idx: I) -> Result<Self, TryGetError> {
        let secs = i64::try_get_by(res, idx)?;
        Self::from_unix_seconds(secs)
            .map_err(|e| TryGetError::DbErr(sea_orm::DbErr::Type(e.to_string())))
    }
}

impl ValueType for Timestamp {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        match v {
            Value::BigInt(Some(s)) => Self::from_unix_seconds(s).map_err(|_| ValueTypeErr),
            Value::Int(Some(s)) => Self::from_unix_seconds(i64::from(s)).map_err(|_| ValueTypeErr),
            _ => Err(ValueTypeErr),
        }
    }

    fn type_name() -> String {
        "Timestamp".to_owned()
    }

    fn array_type() -> ArrayType {
        ArrayType::BigInt
    }

    fn column_type() -> ColumnType {
        ColumnType::BigInteger
    }
}

impl Nullable for Timestamp {
    fn null() -> Value {
        Value::BigInt(None)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_unix_seconds() {
        let ts = Timestamp::from_unix_seconds(1_700_000_000).unwrap();
        let v: Value = ts.into();
        assert_eq!(v, Value::BigInt(Some(1_700_000_000)));
        let back = <Timestamp as ValueType>::try_from(v).unwrap();
        assert_eq!(back, ts);
    }

    #[test]
    fn rejects_out_of_range() {
        assert!(Timestamp::from_unix_seconds(i64::MAX).is_err());
    }

    #[test]
    fn null_value() {
        assert_eq!(<Timestamp as Nullable>::null(), Value::BigInt(None));
    }
}
