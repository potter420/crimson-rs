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
///
/// Matches the game's native format:
/// 1. Split paths on `/` into directory segments
/// 2. Non-root segments get `/` prepended
/// 3. Siblings at each level are radix-compressed (byte-level prefix sharing)
///
/// For example, `"gamedata/binary__"` and `"gamedata/binarygimmickchart__"`:
/// - Root segment `"gamedata"` is shared
/// - Children `"/binary__"` and `"/binarygimmickchart__"` share `"/binary"`
/// - Split into: `"/binary"` → `"__"` and `"gimmickchart__"`
pub fn build_trie_buffer(strings: &[&str]) -> (Vec<u8>, Vec<i32>) {
    if strings.is_empty() {
        return (Vec::new(), Vec::new());
    }

    // Phase 1: Build segment-level trie (split on "/")
    let mut seg_root = SegNode {
        children: Vec::new(),
        terminal_indices: Vec::new(),
    };

    for (i, s) in strings.iter().enumerate() {
        let segments: Vec<&str> = if s.is_empty() {
            vec![]
        } else {
            s.split('/').collect()
        };
        seg_insert(&mut seg_root, &segments, i);
    }

    // Phase 2: Serialize with radix compression among siblings
    let mut buffer = Vec::new();
    let mut offsets = vec![-1i32; strings.len()];

    // Handle empty-string terminals at root
    for &idx in &seg_root.terminal_indices {
        let my_offset = buffer.len() as i32;
        buffer.extend_from_slice(&(-1i32).to_le_bytes());
        buffer.push(0);
        offsets[idx] = my_offset;
    }

    // Emit root-level children (no "/" prefix)
    emit_seg_children(&seg_root.children, -1, true, &mut buffer, &mut offsets);

    (buffer, offsets)
}

// ── Segment trie (phase 1) ───────────────────────────────────────────────

struct SegNode {
    children: Vec<(String, SegNode)>, // (segment_name, child)
    terminal_indices: Vec<usize>,
}

fn seg_insert(node: &mut SegNode, segments: &[&str], string_index: usize) {
    if segments.is_empty() {
        node.terminal_indices.push(string_index);
        return;
    }

    for (name, child) in &mut node.children {
        if name == segments[0] {
            seg_insert(child, &segments[1..], string_index);
            return;
        }
    }

    let mut new_child = SegNode {
        children: Vec::new(),
        terminal_indices: Vec::new(),
    };
    seg_insert(&mut new_child, &segments[1..], string_index);
    node.children.push((segments[0].to_string(), new_child));
}

// ── Sibling radix compression + serialization (phase 2) ─────────────────

/// A node in the sibling radix trie.
/// Each node either completes a segment (has `seg_idx`) or is an
/// intermediate shared-prefix node.
struct SibRadix {
    fragment: String,
    children: Vec<SibRadix>,
    /// Index into the parent SegNode's children, if this completes a segment.
    seg_idx: Option<usize>,
}

fn common_prefix_len(a: &str, b: &str) -> usize {
    a.bytes().zip(b.bytes()).take_while(|(x, y)| x == y).count()
}

fn sib_radix_insert(node: &mut SibRadix, remaining: &str, seg_idx: usize) {
    if remaining.is_empty() {
        node.seg_idx = Some(seg_idx);
        return;
    }

    for i in 0..node.children.len() {
        let prefix_len = common_prefix_len(&node.children[i].fragment, remaining);
        if prefix_len == 0 {
            continue;
        }

        if prefix_len == node.children[i].fragment.len() {
            sib_radix_insert(&mut node.children[i], &remaining[prefix_len..], seg_idx);
            return;
        }

        // Partial match — split
        let shared = node.children[i].fragment[..prefix_len].to_string();
        let child_tail = node.children[i].fragment[prefix_len..].to_string();
        let new_tail = &remaining[prefix_len..];

        let mut old_child = node.children.swap_remove(i);
        old_child.fragment = child_tail;

        let mut split = SibRadix {
            fragment: shared,
            children: vec![old_child],
            seg_idx: None,
        };
        sib_radix_insert(&mut split, new_tail, seg_idx);
        node.children.push(split);
        return;
    }

    node.children.push(SibRadix {
        fragment: remaining.to_string(),
        children: vec![],
        seg_idx: Some(seg_idx),
    });
}

/// Build a sibling radix trie from segment children and serialize to buffer.
fn emit_seg_children(
    seg_children: &[(String, SegNode)],
    parent_offset: i32,
    is_root_level: bool,
    buffer: &mut Vec<u8>,
    offsets: &mut [i32],
) {
    if seg_children.is_empty() {
        return;
    }

    // Build radix trie from sibling fragments
    let mut radix_root = SibRadix {
        fragment: String::new(),
        children: Vec::new(),
        seg_idx: None,
    };

    for (ci, (seg_name, _)) in seg_children.iter().enumerate() {
        let fragment = if is_root_level {
            seg_name.clone()
        } else {
            format!("/{seg_name}")
        };
        sib_radix_insert(&mut radix_root, &fragment, ci);
    }

    // Serialize the radix trie depth-first
    emit_sib_radix(&radix_root, parent_offset, seg_children, buffer, offsets);
}

fn emit_sib_radix(
    node: &SibRadix,
    parent_offset: i32,
    seg_children: &[(String, SegNode)],
    buffer: &mut Vec<u8>,
    offsets: &mut [i32],
) {
    // Virtual root — just recurse
    if node.fragment.is_empty() {
        for child in &node.children {
            emit_sib_radix(child, parent_offset, seg_children, buffer, offsets);
        }
        return;
    }

    // Write this node's entry
    let my_offset = buffer.len() as i32;
    buffer.extend_from_slice(&parent_offset.to_le_bytes());
    let frag_bytes = node.fragment.as_bytes();
    buffer.push(frag_bytes.len() as u8);
    buffer.extend_from_slice(frag_bytes);

    // If this completes a segment, record terminals and recurse into subtree
    if let Some(ci) = node.seg_idx {
        let seg_node = &seg_children[ci].1;

        for &idx in &seg_node.terminal_indices {
            offsets[idx] = my_offset;
        }

        // Emit this segment's children (next directory level)
        emit_seg_children(&seg_node.children, my_offset, false, buffer, offsets);
    }

    // Emit radix continuations (sibling prefix sharing)
    for child in &node.children {
        emit_sib_radix(child, my_offset, seg_children, buffer, offsets);
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
    fn test_build_radix_trie_game_compatible() {
        // Verify the radix trie matches the game's format: byte-level prefix sharing
        let strings = [
            "gamedata",
            "gamedata/binary__",
            "gamedata/binary__/client",
            "gamedata/binary__/client/bin",
            "gamedata/binary__/misc",
            "gamedata/binarygimmickchart__",
        ];
        let (buf, offsets) = build_trie_buffer(&strings);
        let mut trie = TrieStringBuffer::new(buf.clone());

        // All strings should roundtrip correctly
        for (i, s) in strings.iter().enumerate() {
            assert_eq!(trie.get_string(offsets[i]).unwrap(), *s, "string[{}]", i);
        }

        // Verify structure: "gamedata/binary" should be shared between
        // "binary__" and "binarygimmickchart__"
        // Entry 0 should be "gamedata" with parent=-1
        let parent0 = i32::from_le_bytes(buf[0..4].try_into().unwrap());
        let len0 = buf[4] as usize;
        let data0 = std::str::from_utf8(&buf[5..5 + len0]).unwrap();
        assert_eq!(parent0, -1);
        assert_eq!(data0, "gamedata");

        // Entry 1 should be "/binary" with parent=0
        let off1 = 5 + len0;
        let parent1 = i32::from_le_bytes(buf[off1..off1 + 4].try_into().unwrap());
        let len1 = buf[off1 + 4] as usize;
        let data1 = std::str::from_utf8(&buf[off1 + 5..off1 + 5 + len1]).unwrap();
        assert_eq!(parent1, 0);
        assert_eq!(data1, "/binary");
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
