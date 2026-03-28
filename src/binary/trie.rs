use std::collections::HashMap;
use std::io;

/// Trie-encoded string buffer used by PAMT files.
///
/// Each entry at a given offset consists of:
///   - next_offset: i32 (LE) — offset of parent entry, or -1 for root
///   - string_length: u8
///   - string_data: [u8; string_length]
///
/// To reconstruct a full string, recursively resolve parent entries
/// and concatenate: parent_string + current_string.
#[derive(Debug)]
pub struct TrieStringBuffer {
    data: Vec<u8>,
    cache: HashMap<i32, String>,
}

impl TrieStringBuffer {
    pub fn new(data: Vec<u8>) -> Self {
        TrieStringBuffer {
            data,
            cache: HashMap::new(),
        }
    }

    pub fn get_string(&mut self, offset: i32) -> io::Result<String> {
        if offset == -1 {
            return Ok(String::new());
        }

        if let Some(cached) = self.cache.get(&offset) {
            return Ok(cached.clone());
        }

        let o = offset as usize;
        if o + 5 > self.data.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                format!("trie offset {} out of bounds (buf len {})", offset, self.data.len()),
            ));
        }

        let next_offset = i32::from_le_bytes(self.data[o..o + 4].try_into().unwrap());
        let string_length = self.data[o + 4] as usize;

        if o + 5 + string_length > self.data.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "trie string data out of bounds",
            ));
        }

        // Copy the string data to avoid borrow conflict with recursive call
        let string_part = std::str::from_utf8(&self.data[o + 5..o + 5 + string_length])
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
            .to_string();

        let parent = self.get_string(next_offset)?;
        let value = parent + &string_part;

        self.cache.insert(offset, value.clone());
        Ok(value)
    }

    /// Returns the raw buffer data (for roundtrip writing).
    #[allow(dead_code)]
    pub fn raw_data(&self) -> &[u8] {
        &self.data
    }
}

// ── Trie Buffer Builder ──────────────────────────────────────────────────

/// Build a trie-encoded string buffer from a list of strings.
///
/// Returns `(buffer, offsets)` where `offsets[i]` is the trie offset for `strings[i]`.
/// Strings are split on `/` to share common path prefixes.
pub fn build_trie_buffer(strings: &[&str]) -> (Vec<u8>, Vec<i32>) {
    if strings.is_empty() {
        return (Vec::new(), Vec::new());
    }

    // Build trie tree in memory
    let mut root = BuildTrieNode {
        fragment: String::new(),
        children: Vec::new(),
        terminal_indices: Vec::new(),
    };

    for (i, s) in strings.iter().enumerate() {
        let segments = split_path_segments(s);
        insert_into_trie(&mut root, &segments, i);
    }

    // Serialize trie depth-first
    let mut buffer = Vec::new();
    let mut offsets = vec![-1i32; strings.len()];
    serialize_trie_node(&root, -1, &mut buffer, &mut offsets);

    (buffer, offsets)
}

struct BuildTrieNode {
    fragment: String,
    children: Vec<BuildTrieNode>,
    terminal_indices: Vec<usize>,
}

/// Split a path into segments, keeping `/` attached to the preceding segment.
/// e.g., "a/b/c" -> ["a/", "b/", "c"]
/// Empty string -> [""]
fn split_path_segments(s: &str) -> Vec<String> {
    if s.is_empty() {
        return vec![String::new()];
    }

    let mut segments = Vec::new();
    let mut start = 0;
    let bytes = s.as_bytes();

    for i in 0..bytes.len() {
        if bytes[i] == b'/' {
            segments.push(s[start..=i].to_string());
            start = i + 1;
        }
    }
    if start < bytes.len() {
        segments.push(s[start..].to_string());
    } else if start == bytes.len() {
        // Trailing slash: the last segment is empty, but we already captured the "/"
        // with the preceding segment. Nothing more to add.
    }

    segments
}

fn insert_into_trie(node: &mut BuildTrieNode, segments: &[String], string_index: usize) {
    if segments.is_empty() {
        node.terminal_indices.push(string_index);
        return;
    }

    // Look for existing child with matching fragment
    for child in &mut node.children {
        if child.fragment == segments[0] {
            insert_into_trie(child, &segments[1..], string_index);
            return;
        }
    }

    // Create new child
    let mut new_child = BuildTrieNode {
        fragment: segments[0].clone(),
        children: Vec::new(),
        terminal_indices: Vec::new(),
    };
    insert_into_trie(&mut new_child, &segments[1..], string_index);
    node.children.push(new_child);
}

fn serialize_trie_node(
    node: &BuildTrieNode,
    parent_offset: i32,
    buffer: &mut Vec<u8>,
    offsets: &mut [i32],
) {
    // For the root node (empty fragment), don't write an entry — just recurse children
    if node.fragment.is_empty() && parent_offset == -1 {
        // Root terminals (empty strings)
        for &idx in &node.terminal_indices {
            let my_offset = buffer.len() as i32;
            buffer.extend_from_slice(&(-1i32).to_le_bytes());
            buffer.push(0); // zero-length string
            offsets[idx] = my_offset;
        }
        // Recurse children with parent_offset = -1
        for child in &node.children {
            serialize_trie_node(child, -1, buffer, offsets);
        }
        return;
    }

    // Write this node's entry
    let my_offset = buffer.len() as i32;
    buffer.extend_from_slice(&parent_offset.to_le_bytes());
    let frag_bytes = node.fragment.as_bytes();
    buffer.push(frag_bytes.len() as u8);
    buffer.extend_from_slice(frag_bytes);

    // Record offset for any strings that terminate here
    for &idx in &node.terminal_indices {
        offsets[idx] = my_offset;
    }

    // Recurse children
    for child in &node.children {
        serialize_trie_node(child, my_offset, buffer, offsets);
    }
}

/// Read a null-terminated C string from a buffer at the given offset.
/// Used by PAPGT for group name resolution.
pub fn read_cstring(buffer: &[u8], offset: usize) -> io::Result<String> {
    let start = offset;
    let mut end = start;
    while end < buffer.len() && buffer[end] != 0 {
        end += 1;
    }
    if end >= buffer.len() {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "unterminated C string",
        ));
    }
    std::str::from_utf8(&buffer[start..end])
        .map(|s| s.to_string())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cstring_read() {
        let buf = b"hello\0world\0";
        assert_eq!(read_cstring(buf, 0).unwrap(), "hello");
        assert_eq!(read_cstring(buf, 6).unwrap(), "world");
    }

    #[test]
    fn test_trie_single_entry() {
        // Entry at offset 0: next_offset=-1, length=5, data="hello"
        let mut buf = Vec::new();
        buf.extend_from_slice(&(-1i32).to_le_bytes()); // next_offset
        buf.push(5); // length
        buf.extend_from_slice(b"hello"); // data
        let mut trie = TrieStringBuffer::new(buf);
        assert_eq!(trie.get_string(0).unwrap(), "hello");
    }

    #[test]
    fn test_build_trie_empty() {
        let (buf, offsets) = build_trie_buffer(&[]);
        assert!(buf.is_empty());
        assert!(offsets.is_empty());
    }

    #[test]
    fn test_build_trie_single() {
        let strings = ["hello"];
        let (buf, offsets) = build_trie_buffer(&strings);
        assert_eq!(offsets.len(), 1);
        let mut trie = TrieStringBuffer::new(buf);
        assert_eq!(trie.get_string(offsets[0]).unwrap(), "hello");
    }

    #[test]
    fn test_build_trie_shared_prefix() {
        let strings = ["path/dir1", "path/dir2", "path/dir1/sub"];
        let (buf, offsets) = build_trie_buffer(&strings);
        assert_eq!(offsets.len(), 3);
        let mut trie = TrieStringBuffer::new(buf);
        assert_eq!(trie.get_string(offsets[0]).unwrap(), "path/dir1");
        assert_eq!(trie.get_string(offsets[1]).unwrap(), "path/dir2");
        assert_eq!(trie.get_string(offsets[2]).unwrap(), "path/dir1/sub");
    }

    #[test]
    fn test_build_trie_no_shared_prefix() {
        let strings = ["alpha", "beta", "gamma"];
        let (buf, offsets) = build_trie_buffer(&strings);
        let mut trie = TrieStringBuffer::new(buf);
        for (i, s) in strings.iter().enumerate() {
            assert_eq!(trie.get_string(offsets[i]).unwrap(), *s);
        }
    }

    #[test]
    fn test_build_trie_deep_paths() {
        let strings = ["a/b/c/d", "a/b/c/e", "a/b/f"];
        let (buf, offsets) = build_trie_buffer(&strings);
        let mut trie = TrieStringBuffer::new(buf);
        for (i, s) in strings.iter().enumerate() {
            assert_eq!(trie.get_string(offsets[i]).unwrap(), *s);
        }
    }

    #[test]
    fn test_trie_chained() {
        // Entry at offset 0: next_offset=-1, length=4, data="path"
        // Entry at offset 9: next_offset=0, length=5, data="/file"
        let mut buf = Vec::new();
        // Entry 0
        buf.extend_from_slice(&(-1i32).to_le_bytes());
        buf.push(4);
        buf.extend_from_slice(b"path");
        // Entry 1 at offset 9
        buf.extend_from_slice(&(0i32).to_le_bytes());
        buf.push(5);
        buf.extend_from_slice(b"/file");
        let mut trie = TrieStringBuffer::new(buf);
        assert_eq!(trie.get_string(0).unwrap(), "path");
        assert_eq!(trie.get_string(9).unwrap(), "path/file");
    }
}
