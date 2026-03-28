use chacha20_crate::cipher::{KeyIvInit, StreamCipher, StreamCipherSeek};

use super::checksum;

/// Hardcoded base key for pack entry encryption.
const PACK_ENTRY_BASE_KEY: [u8; 32] = [
    0x0E, 0x0F, 0x0C, 0x0D, 0x04, 0x05, 0x06, 0x07,
    0x02, 0x03, 0x00, 0x01, 0x08, 0x09, 0x0A, 0x0B,
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
    0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
];

/// Decrypt a pack entry using ChaCha20.
///
/// - `data`: the encrypted file data
/// - `group_encrypt_info`: 3-byte encryption info from the PAMT header
/// - `entry_path`: full virtual path of the file (e.g., "path/to/file.ext")
///
/// The nonce is derived from the filename (after the last '/') checksum,
/// and the key is derived by XOR-ing the base key with the nonce and
/// the XOR of the 3 encrypt_info bytes.
pub fn decrypt_pack_entry(
    data: &[u8],
    group_encrypt_info: &[u8; 3],
    entry_path: &str,
) -> Vec<u8> {
    let filename = entry_path.rsplit('/').next().unwrap_or(entry_path);
    let counter = checksum::calculate_checksum(filename.as_bytes());

    // Build 16-byte nonce: counter repeated 4 times
    let counter_bytes = counter.to_le_bytes();
    let mut nonce_16 = [0u8; 16];
    for i in 0..4 {
        nonce_16[i * 4..(i + 1) * 4].copy_from_slice(&counter_bytes);
    }

    // Derive key
    let k = group_encrypt_info[0] ^ group_encrypt_info[1] ^ group_encrypt_info[2];
    let mut key = PACK_ENTRY_BASE_KEY;
    for i in 0..32 {
        key[i] ^= nonce_16[i % 16] ^ k;
    }

    // Python's ChaCha20 nonce is 16 bytes: first 4 = counter (LE), last 12 = nonce
    let initial_counter = u32::from_le_bytes(nonce_16[0..4].try_into().unwrap());
    let nonce: [u8; 12] = nonce_16[4..16].try_into().unwrap();

    let mut cipher = chacha20_crate::ChaCha20::new(
        &key.into(),
        &nonce.into(),
    );
    // Seek to the initial counter position (each block is 64 bytes)
    cipher.seek(initial_counter as u64 * 64);

    let mut output = data.to_vec();
    cipher.apply_keystream(&mut output);
    output
}

/// Encrypt a pack entry using ChaCha20 (same operation as decrypt for a stream cipher).
pub fn encrypt_pack_entry(
    data: &[u8],
    group_encrypt_info: &[u8; 3],
    entry_path: &str,
) -> Vec<u8> {
    decrypt_pack_entry(data, group_encrypt_info, entry_path)
}
