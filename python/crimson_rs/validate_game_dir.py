"""Validate and update PAMT checksums in a Crimson Desert game directory.

Scans all numeric pack folders (0000, 0001, ...), reads each 0.pamt,
computes its checksum, and updates meta/0.papgt with the correct values.

Usage:
    python -m crimson_rs.validate_game_dir <game_directory>
"""

from __future__ import annotations

import sys
from pathlib import Path

import crimson_rs


def validate_and_update(game_dir: str) -> None:
    game_path = Path(game_dir)
    if not game_path.is_dir():
        print(f"Error: '{game_dir}' is not a directory")
        sys.exit(1)

    papgt_path = game_path / "meta" / "0.papgt"
    if not papgt_path.exists():
        print(f"Error: '{papgt_path}' not found")
        sys.exit(1)

    # Parse the PAPGT
    papgt = crimson_rs.parse_papgt_file(str(papgt_path))
    entries = papgt["entries"]

    # Build a lookup from group_name -> entry index
    entry_map: dict[str, int] = {}
    for i, entry in enumerate(entries):
        entry_map[entry["group_name"]] = i

    # Find all numeric folders and validate
    updated = 0
    validated = 0
    skipped = 0

    # Collect numeric folders
    pack_folders = sorted(
        p for p in game_path.iterdir()
        if p.is_dir() and p.name.isdigit()
    )

    for folder in pack_folders:
        pamt_file = folder / "0.pamt"
        if not pamt_file.exists():
            print(f"  SKIP {folder.name}/ (no 0.pamt)")
            skipped += 1
            continue

        group_name = folder.name
        if group_name not in entry_map:
            print(f"  SKIP {group_name}/ (not in papgt)")
            skipped += 1
            continue

        # Read PAMT and compute checksum of post-header data
        pamt_data = pamt_file.read_bytes()
        # Header is 12 bytes: checksum(4) + count(2) + unknown0(2) + encrypt_info(1+3)
        post_header = pamt_data[12:]
        computed_crc = crimson_rs.calculate_checksum(post_header)

        idx = entry_map[group_name]
        expected_crc = entries[idx]["pack_meta_checksum"]

        if computed_crc == expected_crc:
            print(f"  OK   {group_name}/ checksum={computed_crc:#010x}")
            validated += 1
        else:
            print(
                f"  UPDATE {group_name}/ "
                f"old={expected_crc:#010x} -> new={computed_crc:#010x}"
            )
            entries[idx]["pack_meta_checksum"] = computed_crc
            updated += 1

    print(f"\nValidated: {validated}, Updated: {updated}, Skipped: {skipped}")

    if updated > 0:
        # Write back the updated PAPGT
        crimson_rs.write_papgt_file(papgt, str(papgt_path))
        print(f"Wrote updated papgt to {papgt_path}")
    else:
        print("All checksums match, no update needed.")


def main() -> None:
    if len(sys.argv) != 2:
        print("Usage: python -m crimson_rs.validate_game_dir <game_directory>")
        sys.exit(1)

    validate_and_update(sys.argv[1])


if __name__ == "__main__":
    main()
