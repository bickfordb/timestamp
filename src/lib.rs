use std::time::Duration;

use chrono::{DateTime, Local, TimeZone, Utc};
use prost::Message;
use sea_orm::{QueryResult, TryGetError, sea_query};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Message)]
pub struct Timestamp {
    #[prost(int64, tag = 1)]
    pub seconds: i64,
    #[prost(int32, tag = 2)]
    pub nanoseconds: i32,
}

impl Timestamp {
    pub fn new() -> Self {
        let dt = Utc::now();
        dt.into()
    }
    pub fn is_empty(&self) -> bool {
        self.seconds == 0 && self.nanoseconds == 0
    }
    pub fn datetime(&self) -> DateTime<Utc> {
        (*self).into()
    }

    pub fn local_datetime(&self) -> DateTime<Local> {
        self.datetime().into()
    }
}

impl From<Duration> for Timestamp {
    fn from(v: Duration) -> Self {
        Timestamp {
            seconds: v.as_secs() as i64,
            nanoseconds: v.subsec_nanos() as i32,
        }
    }
}

impl<TZ: TimeZone> From<DateTime<TZ>> for Timestamp {
    fn from(v: DateTime<TZ>) -> Self {
        let seconds = v.timestamp();
        let nanoseconds = v.timestamp_subsec_nanos() as i32;
        Timestamp {
            seconds,
            nanoseconds,
        }
    }
}

impl From<Timestamp> for DateTime<Utc> {
    fn from(v: Timestamp) -> Self {
        DateTime::from_timestamp(v.seconds, v.nanoseconds as u32)
            .unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap())
    }
}

impl From<f64> for Timestamp {
    fn from(v: f64) -> Self {
        let seconds = v.floor() as i64;
        let nanoseconds = ((v - v.floor()) * 1e9) as i32;
        Timestamp {
            seconds,
            nanoseconds,
        }
    }
}

impl From<Timestamp> for f64 {
    fn from(ts: Timestamp) -> Self {
        ts.seconds as f64 + (ts.nanoseconds as f64 / 1e9)
    }
}

impl From<i64> for Timestamp {
    fn from(seconds: i64) -> Self {
        Timestamp {
            seconds,
            nanoseconds: 0,
        }
    }
}

impl From<Timestamp> for i64 {
    fn from(ts: Timestamp) -> Self {
        ts.seconds
    }
}

impl sea_query::ValueType for Timestamp {
    fn try_from(v: sea_query::Value) -> Result<Self, sea_query::ValueTypeErr> {
        match v {
            sea_query::Value::ChronoDateTimeWithTimeZone(Some(v)) => Ok((*v).into()),
            sea_query::Value::ChronoDateTimeUtc(Some(v)) => Ok((*v).into()),
            sea_query::Value::BigInt(Some(v)) => Ok(v.into()),
            // sea_query::Value::ChronoDateTime(Some(v)) => (*v).into(),
            sea_query::Value::Double(Some(v)) => Ok(v.into()),
            sea_query::Value::Int(Some(v)) => Ok((v as i64).into()),
            _ => Ok(Timestamp::default()),
        }
    }
    fn type_name() -> String {
        "Timestamp".to_string()
    }

    fn array_type() -> sea_query::ArrayType {
        if cfg!(feature = "sqlite_double") {
            <f64 as sea_query::ValueType>::array_type()
        } else if cfg!(feature = "sqlite_int") {
            <i64 as sea_query::ValueType>::array_type()
        } else {
            <DateTime<Utc> as sea_query::ValueType>::array_type()
        }
    }

    fn column_type() -> sea_query::ColumnType {
        if cfg!(feature = "sqlite_double") {
            <f64 as sea_query::ValueType>::column_type()
        } else if cfg!(feature = "sqlite_int") {
            <i64 as sea_query::ValueType>::column_type()
        } else {
            <DateTime<Utc> as sea_query::ValueType>::column_type()
        }
    }
}

impl From<Timestamp> for sea_query::Value {
    fn from(v: Timestamp) -> Self {
        if cfg!(feature = "sqlite_double") {
            sea_query::Value::Double(Some(v.into()))
        } else if cfg!(feature = "sqlite_int") {
            sea_query::Value::BigInt(Some(v.into()))
        } else {
            sea_query::Value::ChronoDateTimeUtc(Some(Box::new(v.into())))
        }
    }
}

impl sea_orm::TryGetable for Timestamp {
    fn try_get_by<I: sea_orm::ColIdx>(res: &QueryResult, index: I) -> Result<Self, TryGetError> {
        if cfg!(feature = "sqlite_double") {
            let data: f64 = <f64 as sea_orm::TryGetable>::try_get_by(res, index)?;
            Ok(data.into())
        } else if cfg!(feature = "sqlite_int") {
            let data: i64 = <i64 as sea_orm::TryGetable>::try_get_by(res, index)?;
            Ok(data.into())
        } else {
            let data: DateTime<Utc> =
                <DateTime<Utc> as sea_orm::TryGetable>::try_get_by(res, index)?;
            Ok(data.into())
        }
    }
}
impl sea_orm::sea_query::Nullable for Timestamp {
    fn null() -> sea_orm::Value {
        if cfg!(feature = "sqlite_double") {
            sea_orm::Value::Double(None)
        } else if cfg!(feature = "sqlite_int") {
            sea_orm::Value::BigInt(None)
        } else {
            sea_orm::Value::ChronoDateTimeUtc(None)
        }
    }
}

impl sea_orm::TryFromU64 for Timestamp {
    fn try_from_u64(_n: u64) -> Result<Self, sea_orm::DbErr> {
        Err(sea_orm::DbErr::ConvertFromU64("Timestamps not supported"))
    }
}
