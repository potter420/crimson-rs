# crimson-rs — Crimson Desert Archive Toolkit

Rust library with Python bindings (via PyO3/maturin) for reading, writing, and modding Crimson Desert game archives.

# Development
- Build and install: `maturin develop`
- Run tests: `cargo test`
- Lint: `cargo clippy` (must pass with zero warnings)

# Documentation
- `docs/archive-format.md` — Binary format specs (PAPGT, PAMT, PAZ, trie), mod loading overlay approach
- `docs/api.md` — Full Python API reference (enums, pack_mod, PAPGT/PAMT/PAZ, ItemInfo)
- Update docs when adding new features or changing binary format understanding

# Rules
- Ask before adding new dependencies to Cargo.toml
- Binary format must roundtrip (read and write back identically)
- Custom types for binary serialization: BString (u32 len + [u8; len] + null byte), optional<T> (u8 flag + T if nonzero), array<T> (u32 count + count × T), Variant (u8 tag + match on tag)

# Binary Exploration
- Use `plcli` + `.hexpat` pattern files for binary format exploration, NOT Python scripts
- Write partial hexpat patterns, test with `plcli run -i <file> -p <pattern.hexpat> -v -d`
- Iterate: write struct, run plcli, check output, refine
- Pattern files go in `references/`

# Reference Files
- `/mnt/e/OpensourceGame/CrimsonDesert/Crimson Browser/iteminfo_decompressed.pabgb` (5993 items)
- `/mnt/e/OpensourceGame/CrimsonDesert/Crimson Browser/Original/0.pamt` — Original PAMT
- `/mnt/e/OpensourceGame/CrimsonDesert/Crimson Browser/Original/0.papgt` — Original PAPGT
- `/mnt/f/Program/Steam/steamapps/common/Crimson Desert` — Game installation folder

# Rust Module Structure
```
src/
├── lib.rs              # Crate root
├── python.rs           # PyO3 bindings
├── binary/
│   ├── mod.rs
│   ├── pamt.rs         # PAMT parse/write
│   ├── papgt.rs        # PAPGT parse/write + add_entry (upsert, front-insert)
│   ├── paz.rs          # PackGroupBuilder, compression, PAZ creation
│   └── trie.rs         # Trie buffer read + build (radix-compressed)
├── crypto/
│   ├── mod.rs
│   ├── checksum.rs     # Jenkins hashlittle2
│   └── chacha20.rs     # ChaCha20 encrypt/decrypt
└── item_info/          # iteminfo.pabgb parser
    ├── mod.rs
    ├── item.rs
    ├── keys.rs
    └── structs.rs
```
