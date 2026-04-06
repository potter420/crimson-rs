import std.io;
import std.mem;

#pragma endian little
#pragma pattern_limit 10485760
#pragma loop_limit 1000000

// ============================================================
// storeinfo.pabgb — Crimson Desert Store Data
// ============================================================
// Derived from C++ ClassLoader serialization.
// field_e discriminates the store variant:
//   0 = Standard store (105B items, 17B tail)
//   1 = Trade store (variable-length items, extended tail)
//   2 = Short-tail store (105B items, 15B tail)
// format_tag (0x17 0xFD) = standard family
// format_tag (0x00 0x00) = Camp/Church (different body)
// format_tag (0x40 0x0D) = StreetVendor (different body)
// format_tag (0x06 0x00) = Camp variant
// ============================================================

struct BString {
    u32 length;
    char value[length];
};

// --- Tail variants ---

struct StoreTail {
    u32 tail_field_a;
    u32 version;
    u32 tail_hash;
    u32 tail_field_b;
    u8 tail_flag;
};

struct StoreTailShort {
    u32 tail_field_a;
    u32 version;
    u32 tail_hash;
    u16 tail_field_b;
    u8 tail_flag;
};

// --- Standard item entry (105 bytes) ---

struct StoreItemEntry {
    u16 store_key_ref;
    u64 buy_price;
    u64 sell_price;
    u32 purchase_limit;
    u32 field_1;
    u32 field_2;
    u8 pre_marker_a;
    u8 pre_marker_b;
    u16 marker;           // 0x0101
    u32 item_key;
    u8 item_data[53];     // TODO: decode sub-fields (sentinel_2 bitmask may control layout)
    u16 separator;        // 0xFFFF
    u32 item_key_dup;     // == item_key
    u8 entry_tail[8];
};

// --- Trade item entry (variable length) ---
// Same fixed header as standard, but item_data varies in size
// and entry_tail is 20 bytes instead of 8.
// TODO: decode item_data sub-fields (contain count-prefixed arrays)

// --- Non-standard body layouts ---
// These need further investigation to decode fully.
// For now, after common header, the body size is not determinable
// without understanding the ClassLoader serialization for each variant.

// --- Store record ---

struct StoreRecord {
    u16 store_key;
    BString name;

    // Common header (28 bytes)
    u8 flag;
    u32 field_a;         // always 1
    u32 field_b;         // always 1
    u32 field_c;         // always 1
    u64 field_d;
    u32 field_e;         // variant discriminator: 0=std, 1=trade, 2=short
    u8 field_f;
    u8 format_tag_a;     // 0x17 for standard family
    u8 format_tag_b;     // 0xFD for standard family

    if (format_tag_a == 0x17 && format_tag_b == 0xFD) {
        // Standard family header continuation (23 bytes)
        u32 sentinel;       // 0xFFFFFFFF
        u32 sentinel_2;     // bitmask controlling item_data fields
        u16 sentinel_3;
        u32 item_count;
        u32 field_g;
        u8 field_h;
        u32 item_count_2;   // == item_count

        if (field_e == 0 && sentinel_2 != 0xFFFFFFFF) {
            // Standard: 105-byte items + 17-byte tail
            StoreItemEntry items[item_count];
            StoreTail tail;
        } else if (field_e == 2) {
            // Short-tail: 105-byte items + 15-byte tail
            StoreItemEntry items[item_count];
            StoreTailShort tail;
        } else {
            // Trade (field_e=1) or TradeResource (sentinel_2=0xFFFFFFFF)
            // Variable-length items — not yet decoded
            // TODO: parse trade items with variable item_data + 20-byte entry_tail
            return;
        }
    } else {
        // Non-standard formats (Camp, Church, StreetVendor, etc.)
        // TODO: decode body for each format_tag variant
        return;
    }
};

// Place first record — stops at first non-parseable variant
StoreRecord store @ 0x00;
