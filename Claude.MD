# Original Binary file
- `/mnt/e/OpensourceGame/CrimsonDesert/Crimson Browser/iteminfo_decompressed.pabgb` (5993 items, correct parsing verified with hexpat)
- `/mnt/e/OpensourceGame/CrimsonDesert/Crimson Browser/Original/0.pamt` => Binary metafile, contain list of files and folder within .paz archives with their offsets, sizes and hashes.
- `/mnt/e/OpensourceGame/CrimsonDesert/Crimson Browser/Original/0.papgt` => Binary directory tree file, pointing to all .pamt files
- `/mnt/f/Program/Steam/steamapps/common/Crimson Desert` => Game installation folder, contains many subfolders and meta folders.

# Correct Parsing Hexpat file
item_info.hexpat

# Development
- Build and install: `maturin develop`
- Run tests: `cargo test`
- Lint: `cargo clippy` (must pass with zero warnings)

# Rules
- Ask before adding new dependencies to Cargo.toml
- Binary format must roundtrip (read and write back identically)
- Custom types for binary serialization: BString (u32 len + [u8; len] + null byte), optional<T> (u8 flag + T if nonzero), array<T> (u32 count + count × T), Variant (u8 tag + match on tag)
