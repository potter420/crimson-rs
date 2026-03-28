mod binary;
mod item_info;

use binary::BinaryRead;
use item_info::ItemInfo;
use std::io::Write;

const BINARY_PATH: &str =
    "/mnt/e/OpensourceGame/CrimsonDesert/Crimson Browser/iteminfo_decompressed.pabgb";

fn main() {
    let data = std::fs::read(BINARY_PATH).expect("failed to read binary file");
    println!("File size: {} bytes", data.len());

    // Parse all items
    let mut offset = 0;
    let mut items = Vec::new();
    while offset < data.len() {
        match ItemInfo::read_from(&data, &mut offset) {
            Ok(item) => items.push(item),
            Err(e) => {
                eprintln!("Error parsing item #{} at offset 0x{:08X}: {}", items.len() + 1, offset, e);
                break;
            }
        }
    }
    println!("Parsed {} items", items.len());

    // Export CSV
    let csv_path = "items.csv";
    let mut f = std::io::BufWriter::new(std::fs::File::create(csv_path).expect("failed to create csv"));
    writeln!(f, "item_key,string_key,item_type,equip_type,category,inventory,item_tier,item_name").unwrap();
    for item in &items {
        writeln!(
            f,
            "{},{},{},{},{},{},{},\"{}\"",
            item.key.0,
            item.string_key.data,
            item.item_type,
            item.equip_type_info.0,
            item.category_info.0,
            item.inventory_info.0,
            item.item_tier,
            item.item_name.default.data.replace('"', "\"\""),
        ).unwrap();
    }
    drop(f);
    println!("Wrote {}", csv_path);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::BinaryWrite;

    #[test]
    fn test_full_roundtrip() {
        let data = std::fs::read(BINARY_PATH).expect("binary file not found");
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(ItemInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");

        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out.len(), data.len(), "size mismatch");
        assert_eq!(out, data, "roundtrip bytes mismatch");
    }
}
