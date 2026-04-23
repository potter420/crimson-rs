"""Smoke test for the tracked reader.

Verifies that `parse_iteminfo_tracked` produces:
 - items identical to `parse_iteminfo_from_bytes`
 - ranges that are contiguous, non-overlapping, and cover every entry byte
 - path shapes we expect (dot for nested struct, [i] for array index)
 - `inspect_legacy_patches` resolves (entry, rel_offset) to field paths
"""
from __future__ import annotations

import os
import sys
import time

import crimson_rs

PATH = r"C:\Users\Coding\CrimsonDesertModding\CrimsonGameMods\_iteminfo_0058_extracted.pabgb"


def main() -> int:
    if not os.path.isfile(PATH):
        print(f"vanilla iteminfo not found at {PATH}", file=sys.stderr)
        return 1
    with open(PATH, "rb") as f:
        data = f.read()
    print(f"loaded {len(data):,} bytes")

    t0 = time.perf_counter()
    items_plain = crimson_rs.parse_iteminfo_from_bytes(data)
    t_plain = time.perf_counter() - t0
    print(f"plain parse: {t_plain:.2f}s, {len(items_plain)} items")

    t0 = time.perf_counter()
    tracked = crimson_rs.parse_iteminfo_tracked(data)
    t_tracked = time.perf_counter() - t0
    print(f"tracked parse: {t_tracked:.2f}s, overhead {t_tracked/t_plain:.1f}x")

    items_tracked = tracked["items"]
    ranges_per_item = tracked["ranges"]
    spans = tracked["spans"]
    assert len(items_tracked) == len(items_plain) == len(ranges_per_item) == len(spans)

    # 1. items identical
    for i, (a, b) in enumerate(zip(items_plain, items_tracked)):
        if a != b:
            print(f"MISMATCH at item {i}: key={a.get('key')} vs {b.get('key')}",
                  file=sys.stderr)
            return 1
    print("OK tracked items identical to plain parse")

    # 2. ranges contiguous + cover entry bytes
    bad = 0
    for i, (ranges, span) in enumerate(zip(ranges_per_item, spans)):
        cursor = span["start"]
        for r in ranges:
            if r["start"] != cursor:
                print(f"  FAIL entry {i} ({items_tracked[i].get('string_key')}): "
                      f"gap at {r['path']!r} expected={cursor} got={r['start']}",
                      file=sys.stderr)
                bad += 1
                break
            cursor = r["end"]
        if cursor != span["end"]:
            print(f"  FAIL entry {i} ranges end at {cursor} but span ends {span['end']}",
                  file=sys.stderr)
            bad += 1
    if bad:
        print(f"{bad} entries with range issues", file=sys.stderr)
        return 1
    print(f"OK ranges contiguous + cover every byte for all {len(ranges_per_item)} entries")

    # 3. path shape sanity
    any_indexed = False
    any_count = False
    any_nested = False
    for r in ranges_per_item[0]:
        if "[" in r["path"]:
            any_indexed = True
        if r["path"].endswith(".__count__"):
            any_count = True
        if "." in r["path"] and not r["path"].startswith("_"):
            any_nested = True
    assert any_indexed, "expected [i] in some path"
    assert any_count, "expected .__count__ in some path"
    assert any_nested, "expected nested dot paths"
    print("OK path shapes: [i] indexing, __count__, nested dots present")

    # 4. inspect_legacy_patches — build a few synthetic patches and resolve
    first = items_tracked[0]
    entry_name = first["string_key"]
    # Grab the cooltime field range from entry 0's range list to find
    # rel_offset to it.
    ranges0 = ranges_per_item[0]
    span0 = spans[0]
    cooltime_range = next(
        (r for r in ranges0 if r["path"] == "cooltime"), None,
    )
    if cooltime_range is None:
        # Field may be absent in some builds — try max_stack_count instead.
        cooltime_range = next(
            r for r in ranges0 if r["path"] == "max_stack_count")
    rel = cooltime_range["start"] - span0["start"]

    hits = crimson_rs.inspect_legacy_patches(
        data,
        [
            {"entry": entry_name, "rel_offset": rel, "length": 1},
            {"entry": entry_name, "rel_offset": 0, "length": 4},  # should be 'key'
            {"entry": "does_not_exist_xyz", "rel_offset": 0},     # should be None
        ],
    )
    assert hits[0] is not None and hits[0]["path"] == cooltime_range["path"], hits[0]
    assert hits[1] is not None and hits[1]["path"] == "key", hits[1]
    assert hits[2] is None, hits[2]
    print(f"OK inspect_legacy_patches: resolved '{cooltime_range['path']}' "
          f"and 'key' at entry '{entry_name}', rejected missing entry")

    print("\nALL GOOD")
    return 0


if __name__ == "__main__":
    sys.exit(main())
