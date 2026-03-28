# crimson_rs Python API Reference

See also: [Archive Format](archive-format.md) for binary format details and mod loading explanation.

## Enums

```python
from crimson_rs import Compression, Crypto, Language

Compression.NONE      # 0
Compression.LZ4       # 2
Compression.ZLIB      # 3

Crypto.NONE           # 0
Crypto.ICE            # 1
Crypto.AES            # 2
Crypto.CHACHA20       # 3

Language.ALL          # 0x3FFF (all 14 languages)
Language.KOR          # 0x0001
Language.ENG          # 0x0002
Language.JPN          # 0x0004
Language.CHT          # 0x0008
Language.GER          # 0x0010
Language.FRA          # 0x0020
Language.SPA          # 0x0040
Language.POR          # 0x0080
Language.RUS          # 0x0100
Language.TUR          # 0x0200
Language.THA          # 0x0400
Language.IND          # 0x0800
Language.CHS          # 0x1000
Language.ARA          # 0x2000
```

---

## High-Level: Mod Packing

### `pack_mod(...)`

Packs a mod folder into a new pack group and updates the PAPGT index. This is the main entrypoint for modders.

```python
from crimson_rs import Compression, Crypto, Language
from crimson_rs.pack_mod import pack_mod

pack_mod(
    game_dir="/path/to/Crimson Desert",
    mod_folder="/path/to/modified/files",
    output_dir="/path/to/output",
    group_name="0036",
    compression=Compression.LZ4,    # default
    crypto=Crypto.NONE,             # default
    language=Language.ALL,           # default
)
```

**Parameters:**
- `game_dir` — Path to the Crimson Desert installation (to read original `meta/0.papgt`)
- `mod_folder` — Directory containing mod files in game directory structure
- `output_dir` — Where to write the packed output (paz + pamt + papgt)
- `group_name` — Pack group name (e.g. `"0036"`)
- `compression` — `Compression.LZ4` (default), `.ZLIB`, or `.NONE`
- `crypto` — `Crypto.NONE` (default), `.ICE`, `.AES`, or `.CHACHA20`
- `encrypt_info` — 3 bytes of encryption key material (default: `b"\x00\x00\x00"`)
- `max_chunk_size` — Max bytes per `.paz` file (default: 500MB)
- `is_optional` — Whether the group is optional (default: `False`)
- `language` — Language flags (default: `Language.ALL`)

**Output structure:**
```
output_dir/
├── {group_name}/
│   ├── 0.paz
│   ├── 0.pamt
│   └── ...
└── meta/
    └── 0.papgt    # Updated with mod entry at front
```

---

## Low-Level: PAPGT

### `parse_papgt_file(path: str) -> PapgtData`

Parse a PAPGT file (pack group tree meta — master index).

### `parse_papgt_bytes(data: bytes) -> PapgtData`

Parse PAPGT from raw bytes.

### `write_papgt_file(data: PapgtData, path: str) -> None`

Serialize PAPGT data and write to file.

### `serialize_papgt(data: PapgtData) -> bytes`

Serialize PAPGT data to raw bytes.

### `add_papgt_entry(papgt_data, group_name, pack_meta_checksum, is_optional, language) -> PapgtData`

Upsert a pack group entry. Inserts at front for mod priority (see [Mod Loading](archive-format.md#mod-loading-overlay-approach)). If `group_name` already exists, updates it in place and moves to front.

```python
papgt = crimson_rs.parse_papgt_file("meta/0.papgt")
updated = crimson_rs.add_papgt_entry(papgt, "0036", checksum, 0, 0x3FFF)
crimson_rs.write_papgt_file(updated, "output/meta/0.papgt")
```

---

## Low-Level: PAMT

### `parse_pamt_file(path: str) -> PamtData`

Parse a PAMT file (pack meta — VFS listing for a single group).

### `parse_pamt_bytes(data: bytes) -> PamtData`

Parse PAMT from raw bytes.

### `write_pamt_file(data: PamtData, path: str) -> None`

Serialize PAMT data and write to file.

### `serialize_pamt(data: PamtData) -> bytes`

Serialize PAMT data to raw bytes.

---

## Low-Level: PackGroupBuilder

Streaming builder that creates `.paz` chunks and `0.pamt` index on disk.

```python
builder = crimson_rs.PackGroupBuilder(
    output_dir="/path/to/0036",
    compression=int(Compression.LZ4),
    crypto=int(Crypto.NONE),
    encrypt_info=b"\x00\x00\x00",
    max_chunk_size=500_000_000,
)
builder.add_file("gamedata/binary__/client/bin", "iteminfo.pabgb", raw_bytes)
builder.add_file_from_path("textures", "icon.dds", "/path/to/icon.dds")
pamt_bytes = builder.finish()  # writes .paz chunks + 0.pamt, returns PAMT bytes
```

---

## Compression

### `compress_data(data: bytes, compression: int) -> bytes`

Compress data. `compression`: 0=None, 2=LZ4, 3=Zlib.

### `decompress_data(data: bytes, compression: int, uncompressed_size: int) -> bytes`

Decompress data.

---

## Checksum

### `calculate_checksum(data: bytes) -> int`

Compute Jenkins hashlittle2 checksum (seed `0xDEBA1DCD`).

---

## File Extraction

### `extract_file(game_dir: str, group_name: str, dir_path: str, file_name: str) -> bytes`

Extract a single file from a pack group archive. Reads the PAMT index, locates the file in the `.paz` chunk, decrypts and decompresses it.

```python
data = crimson_rs.extract_file(
    "/path/to/Crimson Desert",
    "0008",
    "gamedata/binary__/client/bin",
    "iteminfo.pabgb",
)
```

---

## ItemInfo (pabgb)

### `parse_iteminfo_from_file(path: str) -> list[dict]`

Parse all items from a binary file.

```python
items = crimson_rs.parse_iteminfo_from_file("iteminfo_decompressed.pabgb")
```

**Parameters:**
- `path` - Path to the decompressed iteminfo binary file.

**Returns:** List of item dicts. Each dict has the fields documented in [ItemInfo](#iteminfo).

**Raises:** `IOError` if the file cannot be read, `ValueError` on parse errors.

---

### `parse_iteminfo_from_bytes(data: bytes) -> list[dict]`

Parse all items from raw bytes.

```python
with open("iteminfo_decompressed.pabgb", "rb") as f:
    items = crimson_rs.parse_iteminfo_from_bytes(f.read())
```

**Parameters:**
- `data` - Raw binary data.

**Returns:** List of item dicts.

**Raises:** `ValueError` on parse errors.

---

### `write_iteminfo_to_file(items: list[dict], path: str) -> None`

Serialize items and write to a file.

```python
crimson_rs.write_iteminfo_to_file(items, "output.pabgb")
```

**Parameters:**
- `items` - List of item dicts (same structure as returned by `parse_iteminfo_from_file`).
- `path` - Output file path.

**Raises:** `IOError` on write failure, `KeyError` if a required field is missing, `ValueError` on invalid data.

---

### `serialize_iteminfo(items: list[dict]) -> bytes`

Serialize items to raw bytes.

```python
data = crimson_rs.serialize_iteminfo(items)
```

**Parameters:**
- `items` - List of item dicts.

**Returns:** Binary data as `bytes`.

---

## Data Types

All data is returned as plain Python dicts, lists, and primitives. No custom classes are used.

### Type Mapping

| Binary Type | Python Type | Notes |
|---|---|---|
| `u8`, `u16`, `u32`, `u64` | `int` | |
| `i8`, `i64` | `int` | |
| `f32` | `float` | |
| `CString` | `str` | |
| `CArray<T>` | `list[T]` | |
| `COptional<T>` | `T \| None` | |
| `LocalizableString` | `dict` | See [LocalizableString](#localizablestring) |
| Key types (`ItemKey`, etc.) | `int` | Raw u32 or u16 value |
| Structs | `dict` | See individual struct docs below |
| `[f32; 3]` | `list[float]` | 3-element list |
| `[u32; 4]` | `list[int]` | 4-element list |

---

## ItemInfo

Each item is a dict with 105 fields. All fields are required for serialization.

### Identity Fields

| Field | Type | Description |
|---|---|---|
| `key` | `int` | Unique item ID (u32) |
| `string_key` | `str` | String identifier (e.g. `"Pyeonjeon_Arrow"`) |
| `is_blocked` | `int` | Blocked flag (u8) |
| `max_stack_count` | `int` | Maximum stack size (u64) |
| `item_name` | [LocalizableString](#localizablestring) | Localized item name |
| `broken_item_prefix_string` | `int` | LocalStringInfoKey (u32) |

### Inventory & Equipment

| Field | Type | Description |
|---|---|---|
| `inventory_info` | `int` | InventoryKey (u16) |
| `equip_type_info` | `int` | EquipTypeKey (u32) |
| `occupied_equip_slot_data_list` | `list[dict]` | See [OccupiedEquipSlotData](#occupiedequipslotdata) |
| `equipable_hash` | `int` | (u32) |
| `equipable_level` | `int` | Required level to equip (u32) |
| `category_info` | `int` | CategoryKey (u16) |
| `quick_slot_index` | `int` | Quick slot position (u8) |

### Tags & Classification

| Field | Type | Description |
|---|---|---|
| `item_tag_list` | `list[int]` | Item tags (u32 list) |
| `consumable_type_list` | `list[int]` | Consumable types (u32 list) |
| `item_type` | `int` | Item type ID (u8) |
| `item_tier` | `int` | Item tier/rarity (u8) |
| `material_key` | `int` | Material ID (u32) |
| `material_match_info` | `int` | MaterialMatchKey (u32) |
| `filter_type` | `str` | Filter type string |
| `item_group_info_list` | `list[int]` | ItemGroupKey list (u16) |

### Usage & Interaction

| Field | Type | Description |
|---|---|---|
| `item_use_info_list` | `list[int]` | ItemUseKey list (u32) |
| `use_immediately` | `int` | Auto-use flag (u8) |
| `apply_max_stack_cap` | `int` | (u8) |
| `cooltime` | `int` | Cooldown in ticks (i64) |
| `item_charge_type` | `int` | Charge type (u8) |
| `max_charged_useable_count` | `int` | Max charges (u32) |
| `is_save_game_data_at_use_item` | `int` | (u8) |
| `is_logout_at_use_item` | `int` | (u8) |
| `shared_cool_time_group_name_hash` | `int` | Shared cooldown group (u32) |

### Visual & Icons

| Field | Type | Description |
|---|---|---|
| `item_icon_list` | `list[dict]` | See [ItemIconData](#itemicondata) |
| `map_icon_path` | `int` | StringInfoKey (u32) |
| `money_icon_path` | `int` | StringInfoKey (u32) |
| `use_map_icon_alert` | `int` | (u8) |
| `emoji_texture_id` | `str` | Emoji texture string |
| `prefab_data_list` | `list[dict]` | See [PrefabData](#prefabdata) |
| `gimmick_visual_prefab_data_list` | `list[dict]` | See [GimmickVisualPrefabData](#gimmickvisualprefabdata) |

### Description & Knowledge

| Field | Type | Description |
|---|---|---|
| `item_desc` | [LocalizableString](#localizablestring) | Item description |
| `item_desc2` | [LocalizableString](#localizablestring) | Secondary description |
| `item_memo` | `str` | Internal memo |
| `knowledge_info` | `int` | KnowledgeKey (u32) |
| `knowledge_obtain_type` | `int` | (u8) |

### Economy & Pricing

| Field | Type | Description |
|---|---|---|
| `price_list` | `list[dict]` | See [ItemPriceInfo](#itempriceinfo) |
| `is_register_trade_market` | `int` | Tradeable flag (u8) |
| `is_blocked_store_sell` | `int` | (u8) |

### Combat & Equipment Stats

| Field | Type | Description |
|---|---|---|
| `equip_passive_skill_list` | `list[dict]` | See [PassiveSkillLevel](#passiveskilllevel) |
| `enchant_data_list` | `list[dict]` | See [EnchantData](#enchantdata) |
| `sharpness_data` | `dict` | See [ItemInfoSharpnessData](#iteminfoSharpnessdata) |
| `max_endurance` | `int` | Maximum durability (u16) |
| `repair_data_list` | `list[dict]` | See [RepairData](#repairdata) |
| `is_shield_item` | `int` | (u8) |
| `is_tower_shield_item` | `int` | (u8) |
| `hackable_character_group_info_list` | `list[int]` | CharacterGroupKey list (u16) |

### Gimmick & Seal

| Field | Type | Description |
|---|---|---|
| `gimmick_info` | `int` | GimmickInfoKey (u32) |
| `gimmick_tag_list` | `list[str]` | Gimmick tag strings |
| `is_all_gimmick_sealable` | `int` | (u8) |
| `sealable_item_info_list` | `list[dict]` | See [SealableItemInfo](#sealableiteminfo) |
| `sealable_character_info_list` | `list[dict]` | See [SealableItemInfo](#sealableiteminfo) |
| `sealable_gimmick_info_list` | `list[dict]` | See [SealableItemInfo](#sealableiteminfo) |
| `sealable_gimmick_tag_list` | `list[dict]` | See [SealableItemInfo](#sealableiteminfo) |
| `sealable_tribe_info_list` | `list[dict]` | See [SealableItemInfo](#sealableiteminfo) |
| `sealable_money_info_list` | `list[int]` | ItemKey list (u32) |
| `delete_by_gimmick_unlock` | `int` | (u8) |
| `gimmick_unlock_message_local_string_info` | `int` | LocalStringInfoKey (u32) |

### Crafting & Transmutation

| Field | Type | Description |
|---|---|---|
| `can_disassemble` | `int` | (u8) |
| `transmutation_material_gimmick_list` | `list[int]` | GimmickInfoKey list (u32) |
| `transmutation_material_item_list` | `list[int]` | ItemKey list (u32) |
| `transmutation_material_item_group_list` | `list[int]` | ItemGroupKey list (u16) |
| `extract_multi_change_info` | `int` | MultiChangeKey (u32) |
| `multi_change_info_list` | `list[int]` | MultiChangeKey list (u32) |

### Drop & Sub-items

| Field | Type | Description |
|---|---|---|
| `max_drop_result_sub_item_count` | `int` | (u32) |
| `use_drop_set_target` | `int` | (u8) |
| `apply_drop_stat_type` | `int` | (u8) |
| `drop_default_data` | `dict` | See [DropDefaultData](#dropdefaultdata) |
| `default_sub_item` | `dict` | See [SubItem](#subitem) |

### Pages & Inspect

| Field | Type | Description |
|---|---|---|
| `fixed_page_data_list` | `list[dict]` | See [PageData](#pagedata) |
| `dynamic_page_data_list` | `list[dict]` | See [PageData](#pagedata) |
| `inspect_data_list` | `list[dict]` | See [InspectData](#inspectdata) |
| `inspect_action` | `dict` | See [InspectAction](#inspectaction) |

### Docking & Inventory Change

| Field | Type | Description |
|---|---|---|
| `docking_child_data` | `dict \| None` | See [DockingChildData](#dockingchilddata) |
| `inventory_change_data` | `dict \| None` | See [InventoryChangeData](#inventorychangedata) |

### Misc Flags

| Field | Type | Description |
|---|---|---|
| `is_editor_usable` | `int` | (u8) |
| `discardable` | `int` | (u8) |
| `is_dyeable` | `int` | (u8) |
| `is_editable_grime` | `int` | (u8) |
| `is_destroy_when_broken` | `int` | (u8) |
| `is_important_item` | `int` | (u8) |
| `is_wild` | `int` | (u8) |
| `is_preorder_item` | `int` | (u8) |
| `enable_equip_in_clone_actor` | `int` | (u8) |
| `hide_from_inventory_on_pop_item` | `int` | (u8) |
| `enable_alert_system_to_ui` | `int` | (u8) |
| `usable_alert` | `int` | (u8) |
| `discard_offset_y` | `float` | (f32) |
| `respawn_time_seconds` | `int` | (i64) |

### Related Items

| Field | Type | Description |
|---|---|---|
| `packed_item_info` | `int` | ItemKey (u32) |
| `unpacked_item_info` | `int` | ItemKey (u32) |
| `convert_item_info_by_drop_npc` | `int` | ItemKey (u32) |
| `look_detail_game_advice_info_wrapper` | `int` | GameAdviceInfoKey (u32) |
| `look_detail_mission_info` | `int` | MissionKey (u32) |
| `item_bundle_data_list` | `list[dict]` | See [ItemBundleData](#itembundledata) |
| `money_type_define` | `dict \| None` | See [MoneyTypeDefine](#moneytypedefine) |
| `reserve_slot_target_data_list` | `list[dict]` | See [ReserveSlotTargetData](#reserveslottargetdata) |
| `destroy_effec_info` | `int` | EffectKey (u32) |

---

## Nested Structs

### LocalizableString

```python
{
    "category": int,  # u8 - localization category
    "index": int,     # u64 - localization table index
    "default": str    # default string value
}
```

### OccupiedEquipSlotData

```python
{
    "equip_slot_name_key": int,          # u32
    "equip_slot_name_index_list": [int]  # list of u8 values (as ints)
}
```

### ItemIconData

```python
{
    "icon_path": int,              # StringInfoKey (u32)
    "check_exist_sealed_data": int,# u8
    "gimmick_state_list": [int]    # list of u32
}
```

### PassiveSkillLevel

```python
{
    "skill": int,  # SkillKey (u32)
    "level": int   # u32
}
```

### ReserveSlotTargetData

```python
{
    "reserve_slot_info": int,  # ReserveSlotKey (u32)
    "condition_info": int      # ConditionKey (u32)
}
```

### SubItem

Variant type with a type tag.

```python
{
    "type_id": int,       # u8 - variant tag
    "value": int | None   # key value or None
}
```

| `type_id` | Meaning | `value` |
|---|---|---|
| 0 | Item | ItemKey (u32) |
| 3 | Character | CharacterKey (u32) |
| 9 | Gimmick | GimmickInfoKey (u32) |
| 14 | None | `None` |

### SealableItemInfo

Variant type with a type tag.

```python
{
    "type_tag": int,       # u8 - variant tag
    "item_key": int,       # ItemKey (u32)
    "unknown0": int,       # u64
    "value": int | str     # depends on type_tag
}
```

| `type_tag` | Meaning | `value` type |
|---|---|---|
| 0 | Item | `int` (ItemKey) |
| 1 | Gimmick | `int` (GimmickInfoKey) |
| 2 | String | `str` |
| 3 | Character | `int` (CharacterKey) |
| 4 | Tribe | `int` (TribeInfoKey) |

### DropDefaultData

```python
{
    "drop_enchant_level": int,                # u16
    "socket_item_list": [int],                # ItemKey list (u32)
    "add_socket_material_item_list": [dict],  # SocketMaterialItem list
    "default_sub_item": dict,                 # SubItem
    "socket_valid_count": int,                # u8
    "use_socket": int                         # u8
}
```

### SocketMaterialItem

```python
{
    "item": int,   # ItemKey (u32)
    "value": int   # u64
}
```

### EnchantData

```python
{
    "level": int,               # u16
    "enchant_stat_data": dict,  # EnchantStatData
    "buy_price_list": [dict],   # ItemPriceInfo list
    "equip_buffs": [dict]       # EquipmentBuff list
}
```

### EnchantStatData

```python
{
    "max_stat_list": [dict],           # EnchantStatChange list
    "regen_stat_list": [dict],         # EnchantStatChange list
    "stat_list_static": [dict],        # EnchantStatChange list
    "stat_list_static_level": [dict]   # EnchantLevelChange list
}
```

### EnchantStatChange

```python
{
    "stat": int,       # StatusKey (u32)
    "change_mb": int   # i64
}
```

### EnchantLevelChange

```python
{
    "stat": int,       # StatusKey (u32)
    "change_mb": int   # i8
}
```

### ItemPriceInfo

```python
{
    "key": int,     # ItemKey (u32)
    "price": dict   # PriceFloor
}
```

### PriceFloor

```python
{
    "price": int,              # u64
    "sym_no": int,             # u32
    "item_info_wrapper": int   # ItemKey (u32)
}
```

### EquipmentBuff

```python
{
    "buff": int,   # BuffKey (u32)
    "level": int   # u32
}
```

### ItemInfoSharpnessData

```python
{
    "max_sharpness": int,    # u16
    "craft_tool_info": int,  # CraftToolKey (u16)
    "stat_data": dict        # EnchantStatData
}
```

### RepairData

```python
{
    "resource_item_info": int,   # ItemKey (u32)
    "repair_value": int,         # u16
    "repair_style": int,         # u8
    "resource_item_count": int   # u64
}
```

### ItemBundleData

```python
{
    "count_mb": int,  # u64
    "key": int        # GimmickInfoKey (u32)
}
```

### GimmickVisualPrefabData

```python
{
    "tag_name_hash": int,          # u32
    "scale": [float, float, float],# [f32; 3]
    "prefab_names": [int],         # StringInfoKey list (u32)
    "animation_path_list": [int],  # StringInfoKey list (u32)
    "use_gimmick_prefab": int      # u8
}
```

### PrefabData

```python
{
    "prefab_names": [int],       # StringInfoKey list (u32)
    "equip_slot_list": [int],    # u16 list
    "tribe_gender_list": [int],  # StringInfoKey list (u32)
    "is_craft_material": int     # u8
}
```

### PageData

```python
{
    "left_page_texture_path": str,               # string
    "right_page_texture_path": str,              # string
    "left_page_related_knowledge_info": int,     # KnowledgeKey (u32)
    "right_page_related_knowledge_info": int     # KnowledgeKey (u32)
}
```

### InspectData

```python
{
    "item_info": int,                              # ItemKey (u32)
    "gimmick_info": int,                           # GimmickInfoKey (u32)
    "character_info": int,                         # CharacterKey (u32)
    "spawn_reason_hash": int,                      # u32
    "socket_name": str,                            # string
    "speak_character_info": int,                   # CharacterKey (u32)
    "inspect_target_tag": int,                     # u32
    "reward_own_knowledge": int,                   # u8
    "reward_knowledge_info": int,                  # KnowledgeKey (u32)
    "item_desc": dict,                             # LocalizableString
    "board_key": int,                              # u32
    "inspect_action_type": int,                    # u8
    "gimmick_state_name_hash": int,                # u32
    "target_page_index": int,                      # u32
    "is_left_page": int,                           # u8
    "target_page_related_knowledge_info": int,     # KnowledgeKey (u32)
    "enable_read_after_reward": int,               # u8
    "refer_to_left_page_inspect_data": int,        # u8
    "inspect_effect_info_key": int,                # EffectKey (u32)
    "inspect_complete_effect_info_key": int         # EffectKey (u32)
}
```

### InspectAction

```python
{
    "action_name_hash": int,          # u32
    "catch_tag_name_hash": int,       # u32
    "catcher_socket_name": str,       # string
    "catch_target_socket_name": str   # string
}
```

### GameEventExecuteData

```python
{
    "game_event_type": int,      # u8
    "player_condition": int,     # ConditionKey (u32)
    "target_condition": int,     # ConditionKey (u32)
    "event_condition": int       # ConditionKey (u32)
}
```

### InventoryChangeData

```python
{
    "game_event_execute_data": dict,  # GameEventExecuteData
    "to_inventory_info": int          # InventoryKey (u16)
}
```

### DockingChildData

```python
{
    "gimmick_info_key": int,                          # GimmickInfoKey (u32)
    "character_key": int,                             # CharacterKey (u32)
    "item_key": int,                                  # ItemKey (u32)
    "attach_parent_socket_name": str,                 # string
    "attach_child_socket_name": str,                  # string
    "docking_tag_name_hash": [int, int, int, int],    # [u32; 4]
    "docking_equip_slot_no": int,                     # u16
    "spawn_distance_level": int,                      # u32
    "is_item_equip_docking_gimmick": int,             # u8
    "send_damage_to_parent": int,                     # u8
    "is_body_part": int,                              # u8
    "docking_type": int,                              # u8
    "is_summoner_team": int,                          # u8
    "is_player_only": int,                            # u8
    "is_npc_only": int,                               # ConditionKey (u32)
    "is_sync_break_parent": int,                      # u8
    "hit_part": int,                                  # u8
    "detected_by_npc": int,                           # u8
    "is_bag_docking": int,                            # u8
    "enable_collision": int,                          # u8
    "disable_collision_with_other_gimmick": int,      # u8
    "docking_slot_key": str                           # string
}
```

### MoneyTypeDefine

```python
{
    "price_floor_value": int,       # u64
    "unit_data_list_map": [dict]    # MoneyUnitEntry list
}
```

### MoneyUnitEntry

```python
{
    "key": int,     # u32
    "value": dict   # UnitData
}
```

### UnitData

```python
{
    "ui_component": str,    # string
    "minimum": int,         # u32
    "icon_path": int,       # StringInfoKey (u32)
    "item_name": dict,      # LocalizableString
    "item_desc": dict       # LocalizableString
}
```
