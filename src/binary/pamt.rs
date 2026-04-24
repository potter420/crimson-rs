use std::io::{self, Write};

use super::trie::TrieStringBuffer;
use super::{BinaryRead, BinaryWrite, check_remaining};
use crate::binary_struct;
use crate::crypto::checksum;

// ── Enums ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[allow(dead_code)]
pub enum Compression {
    None = 0,
    Partial = 1,
    Lz4 = 2,
    Zlib = 3,
    QuickLz = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CryptoType {
    None = 0,
    Ice = 1,
    Aes = 2,
    ChaCha20 = 3,
}

// ── Structs ────────────────────────────────────────────────────────────────

binary_struct! {
    pub struct PackEncryptInfo {
        pub unknown0: u8,
        pub encrypt_info: [u8; 3],
    }
}

binary_struct! {
    pub struct PackMetaHeader {
        pub checksum: u32,
        pub count: u16,
        pub unknown0: u16,
        pub encrypt_info: PackEncryptInfo,
    }
}

binary_struct! {
    pub struct PackMetaChunk {
        pub id: u32,
        pub checksum: u32,
        pub size: u32,
    }
}

binary_struct! {
    pub struct PackMetaDirectory {
        pub name_checksum: u32,
        pub name_offset: i32,
        pub file_start_index: u32,
        pub file_count: u32,
    }
}

// Raw file entry as stored in the binary format.
// The `flags` byte encodes both compression (bits 0-3) and crypto (bits 4-7).
binary_struct! {
    pub struct PackMetaFileRaw {
        pub name_offset: u32,
        pub chunk_offset: u32,
        pub compressed_size: u32,
        pub uncompressed_size: u32,
        pub chunk_id: u16,
        pub flags: u8,
        pub unknown0: u8,
    }
}

/// Decoded file entry with separated compression and crypto fields.
#[derive(Debug)]
pub struct PackMetaFile {
    pub name_offset: u32,
    pub chunk_offset: u32,
    pub compressed_size: u32,
    pub uncompressed_size: u32,
    pub chunk_id: u16,
    pub flags: u8,
    pub unknown0: u8,
    // Decoded from flags
    pub compression: Compression,
    pub crypto: CryptoType,
    pub is_partial: bool,
}

impl PackMetaFile {
    fn from_raw(raw: PackMetaFileRaw) -> Self {
        let raw_compression = raw.flags & 0x0F;
        let raw_crypto = raw.flags >> 4;

        let (compression, is_partial) = if raw_compression == 1 {
            (Compression::None, true)
        } else {
            (
                match raw_compression {
                    0 => Compression::None,
                    2 => Compression::Lz4,
                    3 => Compression::Zlib,
                    4 => Compression::QuickLz,
                    _ => Compression::None,
                },
                false,
            )
        };

        let crypto = match raw_crypto {
            0 => CryptoType::None,
            1 => CryptoType::Ice,
            2 => CryptoType::Aes,
            3 => CryptoType::ChaCha20,
            _ => CryptoType::None,
        };

        PackMetaFile {
            name_offset: raw.name_offset,
            chunk_offset: raw.chunk_offset,
            compressed_size: raw.compressed_size,
            uncompressed_size: raw.uncompressed_size,
            chunk_id: raw.chunk_id,
            flags: raw.flags,
            unknown0: raw.unknown0,
            compression,
            crypto,
            is_partial,
        }
    }
}

// ── Resolved directory ─────────────────────────────────────────────────────

#[derive(Debug)]
pub struct ResolvedDirectory {
    pub path: String,
    pub raw: PackMetaDirectory,
    pub files: Vec<ResolvedFile>,
}

#[derive(Debug)]
pub struct ResolvedFile {
    pub name: String,
    pub file: PackMetaFile,
}

// ── Top-level PAMT ─────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct PackMeta {
    pub header: PackMetaHeader,
    pub chunks: Vec<PackMetaChunk>,
    pub directories: Vec<ResolvedDirectory>,
    /// Raw directory names trie buffer (for roundtrip).
    pub dir_names_buffer: Vec<u8>,
    /// Raw file names trie buffer (for roundtrip).
    pub file_names_buffer: Vec<u8>,
    /// Raw directory entries (for roundtrip ordering).
    pub raw_directories: Vec<PackMetaDirectory>,
    /// Raw file entries (for roundtrip ordering).
    pub raw_files: Vec<PackMetaFileRaw>,
}

impl PackMeta {
    /// Parse a PAMT from raw bytes.
    ///
    /// Always validates the post-header data using its own computed checksum.
    /// If `expected_crc` is provided (from the PAPGT entry), it also validates against that.
    pub fn parse(data: &[u8], expected_crc: Option<u32>) -> io::Result<Self> {
        let mut offset = 0;
        let header = PackMetaHeader::read_from(data, &mut offset)?;

        if header.unknown0 != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "PackMetaHeader.unknown0 expected 0, got {}",
                    header.unknown0
                ),
            ));
        }

        // Everything after the header is checksummed
        let post_header_data = &data[offset..];

        // Validate against header's own checksum
        checksum::validate_checksum(post_header_data, header.checksum)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // If a PAPGT-provided CRC is given, validate against that too
        if let Some(expected) = expected_crc {
            checksum::validate_checksum(post_header_data, expected)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        }

        // Read chunks
        let mut chunks = Vec::with_capacity(header.count as usize);
        for _ in 0..header.count {
            chunks.push(PackMetaChunk::read_from(data, &mut offset)?);
        }

        // Read directory names trie buffer
        check_remaining(data, offset, 4)?;
        let dir_buf_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        check_remaining(data, offset, dir_buf_len)?;
        let dir_names_buffer = data[offset..offset + dir_buf_len].to_vec();
        offset += dir_buf_len;

        // Read file names trie buffer
        check_remaining(data, offset, 4)?;
        let file_buf_len =
            u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        check_remaining(data, offset, file_buf_len)?;
        let file_names_buffer = data[offset..offset + file_buf_len].to_vec();
        offset += file_buf_len;

        // Read directories
        check_remaining(data, offset, 4)?;
        let dir_count = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        let mut raw_directories = Vec::with_capacity(dir_count);
        for _ in 0..dir_count {
            raw_directories.push(PackMetaDirectory::read_from(data, &mut offset)?);
        }

        // Read files
        check_remaining(data, offset, 4)?;
        let file_count = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        let mut raw_files = Vec::with_capacity(file_count);
        for _ in 0..file_count {
            raw_files.push(PackMetaFileRaw::read_from(data, &mut offset)?);
        }

        // Resolve names
        let mut dir_trie = TrieStringBuffer::new(dir_names_buffer.clone());
        let mut file_trie = TrieStringBuffer::new(file_names_buffer.clone());

        let chunk_map: std::collections::HashMap<u32, &PackMetaChunk> =
            chunks.iter().map(|c| (c.id, c)).collect();

        let mut directories = Vec::with_capacity(raw_directories.len());
        for dir in &raw_directories {
            let dir_path = dir_trie.get_string(dir.name_offset)?;

            // Validate directory name checksum
            let computed = checksum::calculate_checksum(dir_path.as_bytes());
            if computed != dir.name_checksum {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "Directory name checksum mismatch for '{}': expected {:#010x}, got {:#010x}",
                        dir_path, dir.name_checksum, computed
                    ),
                ));
            }

            let start = dir.file_start_index as usize;
            let end = start + dir.file_count as usize;
            if end > raw_files.len() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "Directory '{}' file range {}..{} exceeds file count {}",
                        dir_path,
                        start,
                        end,
                        raw_files.len()
                    ),
                ));
            }

            let mut resolved_files = Vec::with_capacity(dir.file_count as usize);
            for raw_file in &raw_files[start..end] {
                if raw_file.unknown0 != 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "PackMetaFile.unknown0 expected 0, got {}",
                            raw_file.unknown0
                        ),
                    ));
                }

                if !chunk_map.contains_key(&(raw_file.chunk_id as u32)) {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("File references unknown chunk_id {}", raw_file.chunk_id),
                    ));
                }

                let chunk = chunk_map[&(raw_file.chunk_id as u32)];
                if raw_file.chunk_offset + raw_file.compressed_size > chunk.size {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "File data exceeds chunk bounds: offset {} + size {} > chunk size {}",
                            raw_file.chunk_offset, raw_file.compressed_size, chunk.size
                        ),
                    ));
                }

                let file_name = file_trie.get_string(raw_file.name_offset as i32)?;
                let file = PackMetaFile::from_raw(PackMetaFileRaw {
                    name_offset: raw_file.name_offset,
                    chunk_offset: raw_file.chunk_offset,
                    compressed_size: raw_file.compressed_size,
                    uncompressed_size: raw_file.uncompressed_size,
                    chunk_id: raw_file.chunk_id,
                    flags: raw_file.flags,
                    unknown0: raw_file.unknown0,
                });

                resolved_files.push(ResolvedFile {
                    name: file_name,
                    file,
                });
            }

            directories.push(ResolvedDirectory {
                path: dir_path,
                raw: PackMetaDirectory {
                    name_checksum: dir.name_checksum,
                    name_offset: dir.name_offset,
                    file_start_index: dir.file_start_index,
                    file_count: dir.file_count,
                },
                files: resolved_files,
            });
        }

        Ok(PackMeta {
            header,
            chunks,
            directories,
            dir_names_buffer,
            file_names_buffer,
            raw_directories,
            raw_files,
        })
    }

    /// Serialize back to bytes (roundtrip).
    pub fn to_bytes(&self) -> io::Result<Vec<u8>> {
        // Build post-header data
        let mut post_header = Vec::new();

        // Chunks
        for chunk in &self.chunks {
            chunk.write_to(&mut post_header)?;
        }

        // Directory names buffer
        (self.dir_names_buffer.len() as u32).write_to(&mut post_header)?;
        post_header.write_all(&self.dir_names_buffer)?;

        // File names buffer
        (self.file_names_buffer.len() as u32).write_to(&mut post_header)?;
        post_header.write_all(&self.file_names_buffer)?;

        // Directories
        (self.raw_directories.len() as u32).write_to(&mut post_header)?;
        for dir in &self.raw_directories {
            dir.write_to(&mut post_header)?;
        }

        // Files
        (self.raw_files.len() as u32).write_to(&mut post_header)?;
        for file in &self.raw_files {
            file.write_to(&mut post_header)?;
        }

        // Build full output
        let mut out = Vec::new();

        // Header (checksum field is from the original - the actual post-header
        // checksum is validated by the papgt entry, not stored in the pamt header itself)
        self.header.write_to(&mut out)?;
        out.write_all(&post_header)?;

        Ok(out)
    }

    /// Serialize to bytes with a freshly computed checksum.
    /// Use this when creating a new PAMT from scratch.
    pub fn to_bytes_with_checksum(&self) -> io::Result<Vec<u8>> {
        // Build post-header data
        let mut post_header = Vec::new();

        for chunk in &self.chunks {
            chunk.write_to(&mut post_header)?;
        }

        (self.dir_names_buffer.len() as u32).write_to(&mut post_header)?;
        post_header.write_all(&self.dir_names_buffer)?;

        (self.file_names_buffer.len() as u32).write_to(&mut post_header)?;
        post_header.write_all(&self.file_names_buffer)?;

        (self.raw_directories.len() as u32).write_to(&mut post_header)?;
        for dir in &self.raw_directories {
            dir.write_to(&mut post_header)?;
        }

        (self.raw_files.len() as u32).write_to(&mut post_header)?;
        for file in &self.raw_files {
            file.write_to(&mut post_header)?;
        }

        let computed_checksum = checksum::calculate_checksum(&post_header);

        let mut out = Vec::new();
        let header = PackMetaHeader {
            checksum: computed_checksum,
            count: self.header.count,
            unknown0: self.header.unknown0,
            encrypt_info: PackEncryptInfo {
                unknown0: self.header.encrypt_info.unknown0,
                encrypt_info: self.header.encrypt_info.encrypt_info,
            },
        };
        header.write_to(&mut out)?;
        out.write_all(&post_header)?;

        Ok(out)
    }
}
