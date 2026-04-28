//! Audio durations stored as INTEGER milliseconds or seconds.

use std::time::Duration;

use sea_orm::{
    ColIdx, QueryResult, TryGetError, TryGetable,
    sea_query::{ArrayType, ColumnType, Nullable, Value, ValueType, ValueTypeErr},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DurationOutOfRange {
    #[error("duration value is negative: {0}")]
    Negative(i64),
    #[error("duration value overflows u64 milliseconds: {0}")]
    Overflow(i64),
}

/// Audio duration persisted as `INTEGER` milliseconds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DurationMs(Duration);

impl DurationMs {
    pub const fn from_duration(d: Duration) -> Self {
        Self(d)
    }

    pub fn from_millis_i64(ms: i64) -> Result<Self, DurationOutOfRange> {
        if ms < 0 {
            return Err(DurationOutOfRange::Negative(ms));
        }
        let ms_u: u64 =
            TryInto::<u64>::try_into(ms).map_err(|_| DurationOutOfRange::Overflow(ms))?;
        Ok(Self(Duration::from_millis(ms_u)))
    }

    pub fn as_millis_i64(self) -> i64 {
        TryInto::<i64>::try_into(self.0.as_millis()).unwrap_or(i64::MAX)
    }

    pub fn duration(self) -> Duration {
        self.0
    }
}

impl From<DurationMs> for Duration {
    fn from(d: DurationMs) -> Self {
        d.0
    }
}

impl From<DurationMs> for Value {
    fn from(d: DurationMs) -> Self {
        Value::BigInt(Some(d.as_millis_i64()))
    }
}

impl TryGetable for DurationMs {
    fn try_get_by<I: ColIdx>(res: &QueryResult, idx: I) -> Result<Self, TryGetError> {
        let ms = i64::try_get_by(res, idx)?;
        Self::from_millis_i64(ms)
            .map_err(|e| TryGetError::DbErr(sea_orm::DbErr::Type(e.to_string())))
    }
}

impl ValueType for DurationMs {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        match v {
            Value::BigInt(Some(ms)) => Self::from_millis_i64(ms).map_err(|_| ValueTypeErr),
            Value::Int(Some(ms)) => Self::from_millis_i64(i64::from(ms)).map_err(|_| ValueTypeErr),
            _ => Err(ValueTypeErr),
        }
    }
    fn type_name() -> String {
        "DurationMs".to_owned()
    }
    fn array_type() -> ArrayType {
        ArrayType::BigInt
    }
    fn column_type() -> ColumnType {
        ColumnType::BigInteger
    }
}

impl Nullable for DurationMs {
    fn null() -> Value {
        Value::BigInt(None)
    }
}

/// Audio duration persisted as `INTEGER` seconds (used by the AcoustID
/// cache, which expresses duration in whole seconds).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DurationSecs(Duration);

impl DurationSecs {
    pub const fn from_duration(d: Duration) -> Self {
        Self(d)
    }

    pub fn from_secs_i32(secs: i32) -> Result<Self, DurationOutOfRange> {
        if secs < 0 {
            return Err(DurationOutOfRange::Negative(i64::from(secs)));
        }
        Ok(Self(Duration::from_secs(u64::from(secs as u32))))
    }

    pub fn as_secs_i32(self) -> i32 {
        TryInto::<i32>::try_into(self.0.as_secs()).unwrap_or(i32::MAX)
    }

    pub fn duration(self) -> Duration {
        self.0
    }
}

impl From<DurationSecs> for Duration {
    fn from(d: DurationSecs) -> Self {
        d.0
    }
}

impl From<DurationSecs> for Value {
    fn from(d: DurationSecs) -> Self {
        Value::Int(Some(d.as_secs_i32()))
    }
}

impl TryGetable for DurationSecs {
    fn try_get_by<I: ColIdx>(res: &QueryResult, idx: I) -> Result<Self, TryGetError> {
        let secs = i32::try_get_by(res, idx)?;
        Self::from_secs_i32(secs)
            .map_err(|e| TryGetError::DbErr(sea_orm::DbErr::Type(e.to_string())))
    }
}

impl ValueType for DurationSecs {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        match v {
            Value::Int(Some(s)) => Self::from_secs_i32(s).map_err(|_| ValueTypeErr),
            Value::BigInt(Some(s)) => {
                let s32: i32 = TryInto::try_into(s).map_err(|_| ValueTypeErr)?;
                Self::from_secs_i32(s32).map_err(|_| ValueTypeErr)
            }
            _ => Err(ValueTypeErr),
        }
    }
    fn type_name() -> String {
        "DurationSecs".to_owned()
    }
    fn array_type() -> ArrayType {
        ArrayType::Int
    }
    fn column_type() -> ColumnType {
        ColumnType::Integer
    }
}

impl Nullable for DurationSecs {
    fn null() -> Value {
        Value::Int(None)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn ms_roundtrip() {
        let d = DurationMs::from_millis_i64(123_456).unwrap();
        let v: Value = d.into();
        assert_eq!(v, Value::BigInt(Some(123_456)));
        assert_eq!(<DurationMs as ValueType>::try_from(v).unwrap(), d);
    }

    #[test]
    fn ms_rejects_negative() {
        assert!(DurationMs::from_millis_i64(-1).is_err());
    }

    #[test]
    fn secs_roundtrip() {
        let d = DurationSecs::from_secs_i32(180).unwrap();
        let v: Value = d.into();
        assert_eq!(v, Value::Int(Some(180)));
        assert_eq!(<DurationSecs as ValueType>::try_from(v).unwrap(), d);
    }
}
