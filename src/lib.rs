mod binary;
mod crypto;
mod item_info;
mod python;
pub(crate) mod python_traits;

use pyo3::prelude::*;

#[pymodule]
pub fn crimson_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    python::register(m)
}

#[cfg(test)]
mod tests {
    use crate::binary::BinaryRead;
    use crate::binary::BinaryWrite;
    use crate::binary::paloc::LocalizationFile;
    use crate::binary::pamt::PackMeta;
    use crate::binary::papgt::PackGroupTreeMeta;
    use crate::item_info::ItemInfo;

    const BINARY_PATH: &str =
        "/mnt/e/OpensourceGame/CrimsonDesert/Godmod/backups/iteminfo_1.0.4.1.pabgb";
    const PAPGT_PATH: &str = "/mnt/e/OpensourceGame/CrimsonDesert/Crimson Browser/Original/0.papgt";
    const PAMT_PATH: &str = "/mnt/e/OpensourceGame/CrimsonDesert/Crimson Browser/Original/0.pamt";
    const GAME_DIR: &str = "/mnt/f/Program/Steam/steamapps/common/Crimson Desert";

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

    #[test]
    fn test_papgt_parse() {
        let data = std::fs::read(PAPGT_PATH).expect("papgt file not found");
        let papgt = PackGroupTreeMeta::parse(&data).unwrap();
        println!("PAPGT: {} entries", papgt.entries.len());
        for entry in &papgt.entries {
            println!(
                "  group={}, optional={}, language={:#06x}, checksum={:#010x}",
                entry.group_name,
                entry.entry.is_optional,
                entry.entry.language.0,
                entry.entry.pack_meta_checksum,
            );
        }
        assert!(!papgt.entries.is_empty(), "should have entries");
    }

    #[test]
    fn test_papgt_roundtrip() {
        let data = std::fs::read(PAPGT_PATH).expect("papgt file not found");
        let papgt = PackGroupTreeMeta::parse(&data).unwrap();
        println!("PAPGT: {} entries", papgt.entries.len());
        let written = papgt.to_bytes().unwrap();
        assert_eq!(written.len(), data.len(), "papgt roundtrip size mismatch");
        assert_eq!(written, data, "papgt roundtrip bytes mismatch");
    }

    #[test]
    fn test_pamt_parse() {
        let data = std::fs::read(PAMT_PATH).expect("pamt file not found");
        let pamt = PackMeta::parse(&data, None).unwrap();
        println!(
            "PAMT: {} chunks, {} directories",
            pamt.chunks.len(),
            pamt.directories.len()
        );
        for dir in &pamt.directories {
            println!("  dir={}, {} files", dir.path, dir.files.len());
            for f in dir.files.iter().take(3) {
                println!(
                    "    file={}, compressed={}, uncompressed={}, chunk_id={}",
                    f.name, f.file.compressed_size, f.file.uncompressed_size, f.file.chunk_id
                );
            }
        }
        assert!(!pamt.directories.is_empty(), "should have directories");
    }

    #[test]
    fn test_pamt_roundtrip() {
        let data = std::fs::read(PAMT_PATH).expect("pamt file not found");
        let pamt = PackMeta::parse(&data, None).unwrap();
        let written = pamt.to_bytes().unwrap();
        assert_eq!(written.len(), data.len(), "pamt roundtrip size mismatch");
        assert_eq!(written, data, "pamt roundtrip bytes mismatch");
    }

    fn extract_paloc_data() -> Vec<u8> {
        extract_paloc_from_archive("0020", "localizationstring_eng.paloc")
    }

    fn extract_paloc_from_archive(group: &str, file_name: &str) -> Vec<u8> {
        use crate::binary::paz;
        use std::path::Path;

        let group_dir = Path::new(GAME_DIR).join(group);
        let pamt_data = std::fs::read(group_dir.join("0.pamt"))
            .unwrap_or_else(|e| panic!("{}/0.pamt: {}", group, e));
        let pamt = PackMeta::parse(&pamt_data, None).unwrap();

        let dir = pamt
            .directories
            .iter()
            .find(|d| d.path == "gamedata/stringtable/binary__")
            .expect("directory not found in pamt");
        let file = dir
            .files
            .iter()
            .find(|f| f.name == file_name)
            .unwrap_or_else(|| panic!("{} not found", file_name));

        paz::extract_file(
            &group_dir,
            file,
            "gamedata/stringtable/binary__",
            &pamt.header.encrypt_info.encrypt_info,
        )
        .unwrap()
    }

    #[test]
    fn test_paloc_parse() {
        let data = extract_paloc_data();
        let paloc = LocalizationFile::parse(&data).unwrap();
        println!("PALOC: {} entries", paloc.entries.len());
        for entry in paloc.entries.iter().take(5) {
            println!(
                "  id={}, key={}, value={}",
                entry.unk_id,
                entry.string_key.data,
                &entry.string_value.data[..entry.string_value.data.len().min(80)],
            );
        }
        assert!(!paloc.entries.is_empty(), "should have entries");
    }

    #[test]
    fn test_paloc_roundtrip() {
        let data = extract_paloc_data();
        let paloc = LocalizationFile::parse(&data).unwrap();
        let written = paloc.to_bytes().unwrap();
        assert_eq!(written.len(), data.len(), "paloc roundtrip size mismatch");
        assert_eq!(written, data, "paloc roundtrip bytes mismatch");
    }

    #[test]
    fn test_paloc_kor_parse() {
        let data = extract_paloc_from_archive("0019", "localizationstring_kor.paloc");
        let paloc = LocalizationFile::parse(&data).unwrap();
        println!("PALOC KOR: {} entries", paloc.entries.len());
        for entry in paloc.entries.iter().take(5) {
            let preview: String = entry.string_value.data.chars().take(40).collect();
            println!(
                "  id={}, key={}, value={}",
                entry.unk_id, entry.string_key.data, preview,
            );
        }
        assert!(!paloc.entries.is_empty(), "should have entries");
    }

    #[test]
    fn test_paloc_kor_roundtrip() {
        let data = extract_paloc_from_archive("0019", "localizationstring_kor.paloc");
        let paloc = LocalizationFile::parse(&data).unwrap();
        let written = paloc.to_bytes().unwrap();
        assert_eq!(
            written.len(),
            data.len(),
            "paloc kor roundtrip size mismatch"
        );
        assert_eq!(written, data, "paloc kor roundtrip bytes mismatch");
    }

    #[test]
    fn test_game_dir_papgt_pamt_checksums() {
        use crate::crypto::checksum;
        use std::path::Path;

        let papgt_path = Path::new(GAME_DIR).join("meta/0.papgt");
        let papgt_data = std::fs::read(&papgt_path)
            .unwrap_or_else(|e| panic!("cannot read {}: {}", papgt_path.display(), e));
        let papgt = PackGroupTreeMeta::parse(&papgt_data).unwrap();

        println!(
            "Validating {} PAPGT entries against game directory...",
            papgt.entries.len()
        );

        let mut validated = 0;
        let mut skipped = 0;
        for entry in &papgt.entries {
            let pamt_path = Path::new(GAME_DIR).join(&entry.group_name).join("0.pamt");

            if !pamt_path.exists() {
                println!("  SKIP group={} (no 0.pamt found)", entry.group_name);
                skipped += 1;
                continue;
            }

            let pamt_data = std::fs::read(&pamt_path)
                .unwrap_or_else(|e| panic!("cannot read {}: {}", pamt_path.display(), e));

            // Compute checksum of entire pamt file data after header (8 bytes header)
            // The PAPGT stores pack_meta_checksum which is validated against post-header data
            let pamt_header_size = 4 + 2 + 2 + 1 + 3; // checksum + count + unknown0 + encrypt_info
            let post_header = &pamt_data[pamt_header_size..];
            let computed = checksum::calculate_checksum(post_header);

            assert_eq!(
                computed, entry.entry.pack_meta_checksum,
                "Checksum mismatch for group={}: computed={:#010x}, papgt expected={:#010x}",
                entry.group_name, computed, entry.entry.pack_meta_checksum,
            );

            // Also verify full parse with the expected CRC succeeds
            PackMeta::parse(&pamt_data, Some(entry.entry.pack_meta_checksum))
                .unwrap_or_else(|e| panic!("parse failed for group={}: {}", entry.group_name, e));

            println!(
                "  OK   group={}, checksum={:#010x}",
                entry.group_name, computed
            );
            validated += 1;
        }

        println!("Validated: {}, Skipped: {}", validated, skipped);
        assert!(
            validated > 0,
            "should have validated at least one pamt file"
        );
    }
}
