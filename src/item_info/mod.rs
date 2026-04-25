#[allow(dead_code)]
pub mod item;
#[allow(dead_code)]
pub mod keys;
#[allow(dead_code)]
pub mod structs;

pub use item::ItemInfo;

use crate::binary::{BinaryRead, BinaryWrite, BinaryReadTracked, FieldRange};
use crate::json_traits::ToJsonValue;

/// Parse raw iteminfo bytes → Vec<serde_json::Value>.
pub fn parse_iteminfo_to_json(data: &[u8]) -> Result<Vec<serde_json::Value>, String> {
    let mut offset = 0;
    let mut items = Vec::new();
    while offset < data.len() {
        let start = offset;
        let item = ItemInfo::read_from(data, &mut offset)
            .map_err(|e| format!("parse error at offset 0x{:08X}: {}", start, e))?;
        items.push(item.to_json_dict());
    }
    Ok(items)
}

/// Serialize Vec<serde_json::Value> → raw iteminfo bytes.
pub fn serialize_iteminfo_from_json(items: &[serde_json::Value]) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    for (i, val) in items.iter().enumerate() {
        ItemInfo::write_from_json_dict(&mut out, val)
            .map_err(|e| format!("item[{}]: {}", i, e))?;
    }
    Ok(out)
}

/// Tracked parse — items + per-item byte ranges.
pub struct TrackedItemEntry {
    pub string_key: String,
    pub ranges: Vec<FieldRange>,
    pub start: usize,
    pub end: usize,
}

pub fn parse_iteminfo_tracked_rust(data: &[u8]) -> Vec<TrackedItemEntry> {
    let mut offset = 0;
    let mut entries = Vec::new();
    while offset < data.len() {
        let start = offset;
        let mut path = String::new();
        let mut ranges = Vec::new();
        match ItemInfo::read_tracked(data, &mut offset, &mut path, &mut ranges) {
            Ok(item) => {
                entries.push(TrackedItemEntry {
                    string_key: item.string_key.data.to_string(),
                    ranges,
                    start,
                    end: offset,
                });
            }
            Err(_) => break,
        }
    }
    entries
}
