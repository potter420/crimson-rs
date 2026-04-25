//! Native Rust JSON conversion traits — no Python dependency.
//! Mirrors python_traits.rs but uses serde_json::Value instead of PyO3.

use serde_json::{Value as JsonValue, Map, Number};
use crate::binary::{CArray, COptional, CString, LocalizableString, BinaryWrite};

// ── Traits ──────────────────────────────────────────────────────────────────

/// Convert a Rust value to a serde_json::Value.
pub trait ToJsonValue {
    fn to_json_value(&self) -> JsonValue;
}

/// Read a serde_json::Value and write it as binary bytes.
pub trait WriteJsonValue {
    fn write_from_json(w: &mut Vec<u8>, val: &JsonValue) -> Result<(), String>;
}

// ── Dict helper ─────────────────────────────────────────────────────────────

pub fn get_json_field<'a>(obj: &'a JsonValue, key: &str) -> Result<&'a JsonValue, String> {
    obj.get(key).ok_or_else(|| format!("missing field '{}'", key))
}

// ── Primitives ──────────────────────────────────────────────────────────────

macro_rules! impl_json_unsigned {
    ($t:ty) => {
        impl ToJsonValue for $t {
            fn to_json_value(&self) -> JsonValue {
                JsonValue::Number(Number::from(*self as u64))
            }
        }
        impl WriteJsonValue for $t {
            fn write_from_json(w: &mut Vec<u8>, val: &JsonValue) -> Result<(), String> {
                let v = val.as_u64().ok_or_else(|| format!("expected unsigned, got {:?}", val))? as $t;
                w.extend_from_slice(&v.to_le_bytes());
                Ok(())
            }
        }
    };
}

macro_rules! impl_json_signed {
    ($t:ty) => {
        impl ToJsonValue for $t {
            fn to_json_value(&self) -> JsonValue {
                JsonValue::Number(Number::from(*self as i64))
            }
        }
        impl WriteJsonValue for $t {
            fn write_from_json(w: &mut Vec<u8>, val: &JsonValue) -> Result<(), String> {
                let v = val.as_i64().ok_or_else(|| format!("expected signed, got {:?}", val))? as $t;
                w.extend_from_slice(&v.to_le_bytes());
                Ok(())
            }
        }
    };
}

impl_json_unsigned!(u8);
impl_json_unsigned!(u16);
impl_json_unsigned!(u32);
impl_json_unsigned!(u64);
impl_json_signed!(i8);
impl_json_signed!(i16);
impl_json_signed!(i32);
impl_json_signed!(i64);

// f32 / f64
impl ToJsonValue for f32 {
    fn to_json_value(&self) -> JsonValue {
        serde_json::Number::from_f64(*self as f64)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null)
    }
}
impl WriteJsonValue for f32 {
    fn write_from_json(w: &mut Vec<u8>, val: &JsonValue) -> Result<(), String> {
        let v = val.as_f64().ok_or_else(|| format!("expected f32, got {:?}", val))? as f32;
        w.extend_from_slice(&v.to_le_bytes());
        Ok(())
    }
}

// ── CString ─────────────────────────────────────────────────────────────────

impl ToJsonValue for CString<'_> {
    fn to_json_value(&self) -> JsonValue {
        JsonValue::String(self.data.to_string())
    }
}

impl WriteJsonValue for CString<'_> {
    fn write_from_json(w: &mut Vec<u8>, val: &JsonValue) -> Result<(), String> {
        let s = val.as_str().ok_or_else(|| format!("expected string, got {:?}", val))?;
        let bytes = s.as_bytes();
        w.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        w.extend_from_slice(bytes);
        Ok(())
    }
}

// ── CArray ──────────────────────────────────────────────────────────────────

impl<T: ToJsonValue> ToJsonValue for CArray<T> {
    fn to_json_value(&self) -> JsonValue {
        JsonValue::Array(self.items.iter().map(|item| item.to_json_value()).collect())
    }
}

impl<T: WriteJsonValue> WriteJsonValue for CArray<T> {
    fn write_from_json(w: &mut Vec<u8>, val: &JsonValue) -> Result<(), String> {
        let arr = val.as_array().ok_or_else(|| format!("expected array, got {:?}", val))?;
        w.extend_from_slice(&(arr.len() as u32).to_le_bytes());
        for item in arr {
            T::write_from_json(w, item)?;
        }
        Ok(())
    }
}

// ── COptional ───────────────────────────────────────────────────────────────

impl<T: ToJsonValue> ToJsonValue for COptional<T> {
    fn to_json_value(&self) -> JsonValue {
        match &self.value {
            Some(v) => v.to_json_value(),
            None => JsonValue::Null,
        }
    }
}

impl<T: WriteJsonValue> WriteJsonValue for COptional<T> {
    fn write_from_json(w: &mut Vec<u8>, val: &JsonValue) -> Result<(), String> {
        if val.is_null() {
            w.push(1); // flag = 1 means absent
        } else {
            w.push(0); // flag = 0 means present
            T::write_from_json(w, val)?;
        }
        Ok(())
    }
}

// ── Fixed-size arrays ───────────────────────────────────────────────────────

macro_rules! impl_json_array {
    ($n:expr, $t:ty) => {
        impl ToJsonValue for [$t; $n] {
            fn to_json_value(&self) -> JsonValue {
                JsonValue::Array(self.iter().map(|v| v.to_json_value()).collect())
            }
        }
        impl WriteJsonValue for [$t; $n] {
            fn write_from_json(w: &mut Vec<u8>, val: &JsonValue) -> Result<(), String> {
                let arr = val.as_array().ok_or("expected array")?;
                for item in arr {
                    <$t as WriteJsonValue>::write_from_json(w, item)?;
                }
                Ok(())
            }
        }
    };
}

impl_json_array!(2, u32);
impl_json_array!(3, f32);
impl_json_array!(4, u32);

// ── LocalizableString ───────────────────────────────────────────────────────

impl ToJsonValue for LocalizableString<'_> {
    fn to_json_value(&self) -> JsonValue {
        serde_json::json!({
            "category": self.category,
            "index": self.index,
            "default": self.default.data,
        })
    }
}

impl WriteJsonValue for LocalizableString<'_> {
    fn write_from_json(w: &mut Vec<u8>, val: &JsonValue) -> Result<(), String> {
        let cat = val.get("category").and_then(|v| v.as_u64()).ok_or("missing category")? as u8;
        let idx = val.get("index").and_then(|v| v.as_u64()).ok_or("missing index")?;
        let def = val.get("default").and_then(|v| v.as_str()).ok_or("missing default")?;
        w.push(cat);
        w.extend_from_slice(&idx.to_le_bytes());
        let bytes = def.as_bytes();
        w.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        w.extend_from_slice(bytes);
        Ok(())
    }
}
