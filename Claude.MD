# Original Binary file
/mnt/e/OpensourceGame/CrimsonDesert/Crimson Browser/iteminfo_decompressed.pabgb
As of now, the correct number of items is 5993

# Correct Parsing Hexpat file
item_info.hexpat

# Rules
- Ask before adding new dependencies to Cargo.toml
- Binary format must roundtrip (read and write back identically)
- Custom types for binary serialization: BString (u32 len + [u8; len] + null byte), optional<T> (u8 flag + T if nonzero), array<T> (u32 count + count × T), Variant (u8 tag + match on tag)
