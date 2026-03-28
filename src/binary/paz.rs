use std::io::{self, Write};
use std::path::{Path, PathBuf};

use super::pamt::*;
use super::trie::build_trie_buffer;
use crate::crypto::checksum;
use crate::crypto::chacha20;

// ── Compression ───────────────────────────────────────────────────────────

pub fn compress(data: &[u8], compression: Compression) -> io::Result<Vec<u8>> {
    match compression {
        Compression::None => Ok(data.to_vec()),
        Compression::Lz4 => Ok(lz4_flex::block::compress(data)),
        Compression::Zlib => {
            let mut encoder = flate2::write::ZlibEncoder::new(
                Vec::new(),
                flate2::Compression::default(),
            );
            encoder.write_all(data)?;
            encoder.finish()
        }
        _ => Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!("compression {:?} not supported for creation", compression),
        )),
    }
}

pub fn decompress(data: &[u8], compression: Compression, uncompressed_size: usize) -> io::Result<Vec<u8>> {
    match compression {
        Compression::None => Ok(data.to_vec()),
        Compression::Lz4 => {
            lz4_flex::block::decompress(data, uncompressed_size)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
        }
        Compression::Zlib => {
            use std::io::Read;
            let mut decoder = flate2::read::ZlibDecoder::new(data);
            let mut out = Vec::with_capacity(uncompressed_size);
            decoder.read_to_end(&mut out)?;
            Ok(out)
        }
        _ => Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!("decompression {:?} not supported", compression),
        )),
    }
}

// ── File processing ───────────────────────────────────────────────────────

/// Process a single file: compress then optionally encrypt.
/// Returns (processed_data, flags_byte).
pub fn process_file(
    data: &[u8],
    compression: Compression,
    crypto: CryptoType,
    encrypt_info: &[u8; 3],
    file_path: &str,
) -> io::Result<(Vec<u8>, u8)> {
    let compressed = compress(data, compression)?;

    let processed = match crypto {
        CryptoType::ChaCha20 => chacha20::encrypt_pack_entry(&compressed, encrypt_info, file_path),
        CryptoType::None => compressed,
        _ => return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!("crypto {:?} not supported for creation", crypto),
        )),
    };

    let flags = compression as u8 | ((crypto as u8) << 4);
    Ok((processed, flags))
}

// ── Pack Group Builder (streaming to disk) ────────────────────────────────

/// Metadata for a file that has been added to a chunk (no data kept in memory).
struct FileMeta {
    dir_path: String,
    file_name: String,
    chunk_id: u16,
    chunk_offset: u32,
    compressed_size: u32,
    uncompressed_size: u32,
    flags: u8,
}

/// Metadata for a completed chunk written to disk.
struct ChunkMeta {
    id: u32,
    checksum: u32,
    size: u32,
}

/// Builds a pack group by streaming .paz files to disk.
///
/// Only file metadata is kept in memory; compressed data is written
/// to `{output_dir}/{chunk_id}.paz` immediately.
pub struct PackGroupBuilder {
    output_dir: PathBuf,
    compression: Compression,
    crypto: CryptoType,
    encrypt_info: [u8; 3],
    max_chunk_size: u64,
    // Completed chunks
    finished_chunks: Vec<ChunkMeta>,
    // Current chunk being built (in memory, flushed when full)
    current_chunk_id: u32,
    current_chunk_data: Vec<u8>,
    // All file metadata (kept for PAMT generation)
    file_metas: Vec<FileMeta>,
}

impl PackGroupBuilder {
    pub fn new(
        output_dir: &Path,
        compression: Compression,
        crypto: CryptoType,
        encrypt_info: [u8; 3],
        max_chunk_size: u64,
    ) -> Self {
        PackGroupBuilder {
            output_dir: output_dir.to_path_buf(),
            compression,
            crypto,
            encrypt_info,
            max_chunk_size,
            finished_chunks: Vec::new(),
            current_chunk_id: 0,
            current_chunk_data: Vec::new(),
            file_metas: Vec::new(),
        }
    }

    /// Add a file by providing its raw (uncompressed, unencrypted) data.
    /// The data is compressed/encrypted and appended to the current .paz chunk.
    /// If the chunk exceeds max_chunk_size, it is flushed to disk first.
    pub fn add_file(&mut self, dir_path: &str, file_name: &str, data: &[u8]) -> io::Result<()> {
        let full_path = if dir_path.is_empty() {
            file_name.to_string()
        } else {
            format!("{}/{}", dir_path, file_name)
        };

        let (processed, flags) = process_file(
            data,
            self.compression,
            self.crypto,
            &self.encrypt_info,
            &full_path,
        )?;

        let compressed_size = processed.len() as u64;

        // Flush current chunk if adding this file would exceed max_chunk_size
        if !self.current_chunk_data.is_empty()
            && self.current_chunk_data.len() as u64 + compressed_size > self.max_chunk_size
        {
            self.flush_current_chunk()?;
        }

        let chunk_offset = self.current_chunk_data.len() as u32;
        self.current_chunk_data.extend_from_slice(&processed);

        self.file_metas.push(FileMeta {
            dir_path: dir_path.to_string(),
            file_name: file_name.to_string(),
            chunk_id: self.current_chunk_id as u16,
            chunk_offset,
            compressed_size: compressed_size as u32,
            uncompressed_size: data.len() as u32,
            flags,
        });

        Ok(())
    }

    /// Add a file by reading it from a path on disk.
    /// Avoids the caller needing to load the file into memory themselves
    /// (though we still load it here for compression).
    pub fn add_file_from_path(&mut self, dir_path: &str, file_name: &str, file_path: &Path) -> io::Result<()> {
        let data = std::fs::read(file_path)?;
        self.add_file(dir_path, file_name, &data)
    }

    /// Flush the current in-progress chunk to disk.
    fn flush_current_chunk(&mut self) -> io::Result<()> {
        if self.current_chunk_data.is_empty() {
            return Ok(());
        }

        let crc = checksum::calculate_checksum(&self.current_chunk_data);
        let size = self.current_chunk_data.len() as u32;

        // Write to disk
        let paz_path = self.output_dir.join(format!("{}.paz", self.current_chunk_id));
        std::fs::write(&paz_path, &self.current_chunk_data)?;

        self.finished_chunks.push(ChunkMeta {
            id: self.current_chunk_id,
            checksum: crc,
            size,
        });

        self.current_chunk_data.clear();
        self.current_chunk_id += 1;

        Ok(())
    }

    /// Finish building: flush remaining data, write 0.pamt, return PAMT bytes.
    pub fn finish(mut self) -> io::Result<Vec<u8>> {
        // Flush any remaining chunk data
        self.flush_current_chunk()?;

        if self.finished_chunks.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "no files were added",
            ));
        }

        // Build PAMT from metadata
        let pamt_bytes = self.build_pamt()?;

        // Write 0.pamt to disk
        let pamt_path = self.output_dir.join("0.pamt");
        std::fs::write(&pamt_path, &pamt_bytes)?;

        Ok(pamt_bytes)
    }

    fn build_pamt(&self) -> io::Result<Vec<u8>> {
        // Collect directories and their files
        let mut dir_order: Vec<String> = Vec::new();
        let mut dir_files: std::collections::HashMap<String, Vec<usize>> =
            std::collections::HashMap::new();

        for (i, meta) in self.file_metas.iter().enumerate() {
            if !dir_files.contains_key(&meta.dir_path) {
                dir_order.push(meta.dir_path.clone());
                dir_files.insert(meta.dir_path.clone(), Vec::new());
            }
            dir_files.get_mut(&meta.dir_path).unwrap().push(i);
        }

        dir_order.sort();

        // Build trie buffers
        let dir_strs: Vec<&str> = dir_order.iter().map(|s| s.as_str()).collect();
        let (dir_names_buffer, dir_offsets) = build_trie_buffer(&dir_strs);

        // File names in directory-sorted order
        let mut ordered_file_indices: Vec<usize> = Vec::new();
        for dir in &dir_order {
            ordered_file_indices.extend_from_slice(&dir_files[dir]);
        }

        let file_names: Vec<&str> = ordered_file_indices
            .iter()
            .map(|&i| self.file_metas[i].file_name.as_str())
            .collect();
        let (file_names_buffer, file_name_offsets) = build_trie_buffer(&file_names);

        // PAMT chunks
        let pamt_chunks: Vec<PackMetaChunk> = self
            .finished_chunks
            .iter()
            .map(|c| PackMetaChunk {
                id: c.id,
                checksum: c.checksum,
                size: c.size,
            })
            .collect();

        // Directories and files
        let mut raw_directories: Vec<PackMetaDirectory> = Vec::new();
        let mut raw_files: Vec<PackMetaFileRaw> = Vec::new();
        let mut file_index: u32 = 0;

        for (dir_idx, dir) in dir_order.iter().enumerate() {
            let dir_file_indices = &dir_files[dir];
            let file_count = dir_file_indices.len() as u32;

            raw_directories.push(PackMetaDirectory {
                name_checksum: checksum::calculate_checksum(dir.as_bytes()),
                name_offset: dir_offsets[dir_idx],
                file_start_index: file_index,
                file_count,
            });

            for (local_idx, &global_idx) in dir_file_indices.iter().enumerate() {
                let meta = &self.file_metas[global_idx];
                raw_files.push(PackMetaFileRaw {
                    name_offset: file_name_offsets[file_index as usize + local_idx] as u32,
                    chunk_offset: meta.chunk_offset,
                    compressed_size: meta.compressed_size,
                    uncompressed_size: meta.uncompressed_size,
                    chunk_id: meta.chunk_id,
                    flags: meta.flags,
                    unknown0: 0,
                });
            }

            file_index += file_count;
        }

        let pamt = PackMeta {
            header: PackMetaHeader {
                checksum: 0,
                count: pamt_chunks.len() as u16,
                unknown0: 0, // seen in real files, always the same, maybe a version or magic?
                encrypt_info: PackEncryptInfo {
                    unknown0: 50,
                    encrypt_info: [2, 14, 97],
                },
            },
            chunks: pamt_chunks,
            directories: Vec::new(),
            dir_names_buffer,
            file_names_buffer,
            raw_directories,
            raw_files,
        };

        pamt.to_bytes_with_checksum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_tempdir() -> tempfile::TempDir {
        tempdir().expect("failed to create temp dir")
    }

    #[test]
    fn test_compress_decompress_none() {
        let data = b"hello world";
        let compressed = compress(data, Compression::None).unwrap();
        assert_eq!(compressed, data);
        let decompressed = decompress(&compressed, Compression::None, data.len()).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_compress_decompress_lz4() {
        let data = b"hello world hello world hello world";
        let compressed = compress(data, Compression::Lz4).unwrap();
        let decompressed = decompress(&compressed, Compression::Lz4, data.len()).unwrap();
        assert_eq!(decompressed, data.as_ref());
    }

    #[test]
    fn test_compress_decompress_zlib() {
        let data = b"hello world hello world hello world";
        let compressed = compress(data, Compression::Zlib).unwrap();
        let decompressed = decompress(&compressed, Compression::Zlib, data.len()).unwrap();
        assert_eq!(decompressed, data.as_ref());
    }

    #[test]
    fn test_pack_group_builder_basic() {
        let dir = make_tempdir();
        let mut builder = PackGroupBuilder::new(
            dir.path(),
            Compression::None,
            CryptoType::None,
            [0, 0, 0],
            1_000_000,
        );

        builder.add_file("textures", "test.dds", b"fake texture data").unwrap();
        builder.add_file("textures", "test2.dds", b"more texture data").unwrap();
        builder.add_file("models", "mesh.obj", b"fake mesh data").unwrap();

        let pamt_bytes = builder.finish().unwrap();

        // .paz and .pamt should exist on disk
        assert!(dir.path().join("0.paz").exists());
        assert!(dir.path().join("0.pamt").exists());

        // PAMT should be parseable
        let pamt = PackMeta::parse(&pamt_bytes, None).unwrap();
        assert_eq!(pamt.directories.len(), 2);
        assert_eq!(pamt.chunks.len(), 1);

        let dir_names: Vec<&str> = pamt.directories.iter().map(|d| d.path.as_str()).collect();
        assert!(dir_names.contains(&"models"));
        assert!(dir_names.contains(&"textures"));

        let total_files: usize = pamt.directories.iter().map(|d| d.files.len()).sum();
        assert_eq!(total_files, 3);
    }

    #[test]
    fn test_pack_group_builder_chunk_splitting() {
        let dir = make_tempdir();
        let mut builder = PackGroupBuilder::new(
            dir.path(),
            Compression::None,
            CryptoType::None,
            [0, 0, 0],
            50, // very small max chunk size
        );

        builder.add_file("dir", "file1.dat", &[0u8; 30]).unwrap();
        builder.add_file("dir", "file2.dat", &[1u8; 30]).unwrap();
        builder.add_file("dir", "file3.dat", &[2u8; 30]).unwrap();

        let pamt_bytes = builder.finish().unwrap();

        // Should have multiple .paz files
        assert!(dir.path().join("0.paz").exists());
        assert!(dir.path().join("1.paz").exists());

        let pamt = PackMeta::parse(&pamt_bytes, None).unwrap();
        assert_eq!(pamt.directories.len(), 1);
        assert_eq!(pamt.directories[0].files.len(), 3);
        assert!(pamt.chunks.len() >= 2);
    }

    #[test]
    fn test_pack_group_builder_deep_paths() {
        // Matches the game's 0008 pack group structure
        let dir = make_tempdir();
        let mut builder = PackGroupBuilder::new(
            dir.path(),
            Compression::None,
            CryptoType::None,
            [0, 0, 0],
            1_000_000,
        );

        builder.add_file("gamedata", "f1.bin", b"d1").unwrap();
        builder.add_file("gamedata/binary__", "f2.bin", b"d2").unwrap();
        builder.add_file("gamedata/binary__/client", "f3.bin", b"d3").unwrap();
        builder.add_file("gamedata/binary__/client/bin", "f4.bin", b"d4").unwrap();
        builder.add_file("gamedata/binary__/misc", "f5.bin", b"d5").unwrap();
        builder.add_file("gamedata/binary__/misc/bin", "f6.bin", b"d6").unwrap();
        builder.add_file("gamedata/binarygimmickchart__", "f7.bin", b"d7").unwrap();
        builder.add_file("gamedata/binarygimmickchart__/bin", "f8.bin", b"d8").unwrap();

        let pamt_bytes = builder.finish().unwrap();
        let pamt = PackMeta::parse(&pamt_bytes, None).unwrap();

        // All 8 directories should be present and resolved correctly
        assert_eq!(pamt.directories.len(), 8);
        let dir_names: Vec<&str> = pamt.directories.iter().map(|d| d.path.as_str()).collect();
        assert!(dir_names.contains(&"gamedata"));
        assert!(dir_names.contains(&"gamedata/binary__"));
        assert!(dir_names.contains(&"gamedata/binary__/client"));
        assert!(dir_names.contains(&"gamedata/binary__/client/bin"));
        assert!(dir_names.contains(&"gamedata/binary__/misc"));
        assert!(dir_names.contains(&"gamedata/binary__/misc/bin"));
        assert!(dir_names.contains(&"gamedata/binarygimmickchart__"));
        assert!(dir_names.contains(&"gamedata/binarygimmickchart__/bin"));

        // Verify the radix trie structure: "gamedata" at root, "/binary" shared
        let buf = &pamt.dir_names_buffer;
        let parent0 = i32::from_le_bytes(buf[0..4].try_into().unwrap());
        let len0 = buf[4] as usize;
        let data0 = std::str::from_utf8(&buf[5..5 + len0]).unwrap();
        assert_eq!(parent0, -1);
        assert_eq!(data0, "gamedata");

        // Second entry should be "/binary" (shared prefix of "binary__" and "binarygimmickchart__")
        let off1 = 5 + len0;
        let parent1 = i32::from_le_bytes(buf[off1..off1 + 4].try_into().unwrap());
        let len1 = buf[off1 + 4] as usize;
        let data1 = std::str::from_utf8(&buf[off1 + 5..off1 + 5 + len1]).unwrap();
        assert_eq!(parent1, 0);
        assert_eq!(data1, "/binary");
    }

    #[test]
    fn test_pack_group_builder_with_compression() {
        let dir = make_tempdir();
        let mut builder = PackGroupBuilder::new(
            dir.path(),
            Compression::Lz4,
            CryptoType::None,
            [0, 0, 0],
            1_000_000,
        );

        let data = vec![0xABu8; 1000]; // repetitive data compresses well
        builder.add_file("data", "big.bin", &data).unwrap();

        let pamt_bytes = builder.finish().unwrap();
        let pamt = PackMeta::parse(&pamt_bytes, None).unwrap();

        let file = &pamt.directories[0].files[0];
        assert_eq!(file.file.uncompressed_size, 1000);
        assert!(file.file.compressed_size < 1000); // should actually compress
        assert_eq!(file.file.compression, Compression::Lz4);
    }
}
