# Crimson Desert - Explorer

## Overview

This library is a poor man attempts and assembling various tools to explore and understand Crimson Desert game files.

It is a work in progress and currently supports:
- Extracting and parsing .paz files (game archives)
- Parsing and editing iteminfo binary data via Python

## Python Library

A Python binding is provided to parse, modify, and write back `iteminfo_decompressed.pabgb` files.

### Installation

Download the wheel from the [latest release](../../releases/tag/develop) and install:

```bash
pip install crimson_rs-0.1.0-cp312-abi3-win_amd64.whl
```

### Quick Start

```python
import crimson_rs

# Parse all items from a binary file
items = crimson_rs.parse_file("iteminfo_decompressed.pabgb")
print(f"Loaded {len(items)} items")

# Each item is a plain dict with all fields accessible
item = items[0]
print(item["key"])         # 2200
print(item["string_key"])  # "Pyeonjeon_Arrow"
print(item["item_type"])   # 0
print(item["item_tier"])   # 0

# Nested types are also dicts
print(item["item_name"]["category"])  # localization category
print(item["item_name"]["index"])     # localization index
print(item["item_name"]["default"])   # default string value

# Arrays are plain lists
print(item["item_tag_list"])          # [123, 456, ...]
print(item["price_list"])             # [{"key": ..., "price": {...}}, ...]

# Optional fields are None when absent
print(item["docking_child_data"])     # None or dict

# Modify any field
item["item_tier"] = 5
item["max_stack_count"] = 999
item["item_name"]["default"] = "Custom Name"

# Write modified items back to a file (byte-perfect roundtrip)
crimson_rs.write_file(items, "output.pabgb")

# Or get raw bytes
data = crimson_rs.serialize_items(items)

# Parse from bytes
items = crimson_rs.parse_bytes(data)
```

### Filtering and Searching

```python
import crimson_rs

items = crimson_rs.parse_file("iteminfo_decompressed.pabgb")

# Find items by type
weapons = [i for i in items if i["item_type"] == 1]

# Find items by name substring
arrows = [i for i in items if "Arrow" in i["string_key"]]

# Find high-tier items
rare_items = [i for i in items if i["item_tier"] >= 3]

# Export to CSV
import csv
with open("items.csv", "w", newline="") as f:
    w = csv.writer(f)
    w.writerow(["key", "string_key", "item_type", "item_tier"])
    for item in items:
        w.writerow([item["key"], item["string_key"], item["item_type"], item["item_tier"]])
```

### API Reference

See [docs/api.md](docs/api.md) for the full Python API documentation.

# Credit

Thanks to [LukeFZ](https://github.com/LukeFZ)
Reference:
- [pycrimson](https://github.com/LukeFZ/pycrimson)
