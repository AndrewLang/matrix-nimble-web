#![cfg(feature = "postgres")]

use crate::data::query::Value;
use chrono::{DateTime, Utc};
use uuid::Uuid;

pub struct PostgresValueBuilder;

impl PostgresValueBuilder {
    pub fn optional_string(value: &Option<String>) -> Value {
        match value {
            Some(v) => Value::String(v.clone()),
            None => Value::Null,
        }
    }

    pub fn optional_int(value: Option<i64>) -> Value {
        match value {
            Some(v) => Value::Int(v),
            None => Value::Null,
        }
    }

    pub fn optional_i64(value: Option<i64>) -> Value {
        Self::optional_int(value)
    }

    pub fn optional_i32(value: Option<i32>) -> Value {
        match value {
            Some(v) => Value::Int(v as i64),
            None => Value::Null,
        }
    }

    pub fn optional_i16(value: Option<i16>) -> Value {
        match value {
            Some(v) => Value::Int(v as i64),
            None => Value::Null,
        }
    }

    pub fn optional_i8(value: Option<i8>) -> Value {
        match value {
            Some(v) => Value::Int(v as i64),
            None => Value::Null,
        }
    }

    pub fn optional_bool(value: Option<bool>) -> Value {
        match value {
            Some(v) => Value::Bool(v),
            None => Value::Null,
        }
    }

    pub fn optional_f32(value: Option<f32>) -> Value {
        match value {
            Some(v) => Value::Float(v as f64),
            None => Value::Null,
        }
    }

    pub fn optional_f64(value: Option<f64>) -> Value {
        match value {
            Some(v) => Value::Float(v),
            None => Value::Null,
        }
    }

    pub fn optional_uuid(value: Option<Uuid>) -> Value {
        match value {
            Some(v) => Value::Uuid(v),
            None => Value::Null,
        }
    }

    pub fn optional_u64(value: Option<u64>) -> Value {
        match value {
            Some(v) => Value::UInt(v),
            None => Value::Null,
        }
    }

    pub fn optional_u32(value: Option<u32>) -> Value {
        Self::optional_u64(value.map(|v| v as u64))
    }

    pub fn optional_u16(value: Option<u16>) -> Value {
        Self::optional_u64(value.map(|v| v as u64))
    }

    pub fn optional_u8(value: Option<u8>) -> Value {
        Self::optional_u64(value.map(|v| v as u64))
    }

    pub fn optional_datetime(value: &Option<DateTime<Utc>>) -> Value {
        match value {
            Some(v) => Value::DateTime(v.clone()),
            None => Value::Null,
        }
    }
}
