"""Roundtrip test: extract from game archives, parse, serialize, compare bytes."""
import crimson_rs

GAME_DIR = "/mnt/f/Program/Steam/steamapps/common/Crimson Desert"


def test_paloc(group: str, filename: str, label: str):
    data = crimson_rs.extract_file(GAME_DIR, group, "gamedata/stringtable/binary__", filename)
    entries = crimson_rs.parse_paloc_bytes(data)
    print(f"{label}: {len(entries)} entries")
    print(f"  first: {entries[0]}")
    roundtrip = crimson_rs.serialize_paloc(entries)
    assert roundtrip == data, f"{label} roundtrip failed: {len(roundtrip)} vs {len(data)}"
    print(f"  roundtrip OK")


def test_iteminfo():
    data = crimson_rs.extract_file(GAME_DIR, "0008", "gamedata/binary__/client/bin", "iteminfo.pabgb")
    items = crimson_rs.parse_iteminfo_from_bytes(data)
    print(f"ItemInfo: {len(items)} items")
    roundtrip = crimson_rs.serialize_iteminfo(items)
    assert roundtrip == data, f"ItemInfo roundtrip failed: {len(roundtrip)} vs {len(data)}"
    print(f"  roundtrip OK")


if __name__ == "__main__":
    test_paloc("0020", "localizationstring_eng.paloc", "ENG")
    test_paloc("0019", "localizationstring_kor.paloc", "KOR")
    test_iteminfo()
    print("\nAll roundtrip tests passed!")
