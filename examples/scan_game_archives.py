"""Scan a Crimson Desert game directory and list all archive groups.

Usage:
    python examples/scan_game_archives.py [--game-dir PATH]
    python examples/scan_game_archives.py [--game-dir PATH] --group 0008
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

import crimson_rs

DEFAULT_GAME_DIR = r"/mnt/f/Program/Steam/steamapps/common/Crimson Desert"


def scan_game_dir(game_dir: str) -> None:
    game_path = Path(game_dir)
    if not game_path.is_dir():
        print(f"ERROR: game directory not found: {game_path}")
        sys.exit(1)

    papgt_path = game_path / "meta" / "0.papgt"
    if not papgt_path.exists():
        print(f"ERROR: PAPGT not found: {papgt_path}")
        sys.exit(1)

    papgt = crimson_rs.parse_papgt_file(str(papgt_path))
    print(f"PAPGT: {papgt_path}")
    print(f"  Entries: {len(papgt['entries'])}")
    print()

    papgt_lookup: dict[str, dict] = {}
    for e in papgt["entries"]:
        papgt_lookup[e["group_name"]] = e

    group_dirs = sorted(
        d for d in game_path.iterdir()
        if d.is_dir() and re.fullmatch(r"\d{4}", d.name)
    )

    print(f"Pack groups found: {len(group_dirs)}")
    print(f"{'Group':>6}  {'PAZ files':>9}  {'Dirs':>5}  {'Files':>6}  {'Size (MB)':>10}  {'Lang':>6}  {'Opt':>3}")
    print("-" * 60)

    total_paz_size = 0
    total_files = 0

    for group_dir in group_dirs:
        name = group_dir.name
        pamt_path = group_dir / "0.pamt"
        paz_files = sorted(group_dir.glob("*.paz"))
        paz_count = len(paz_files)
        paz_size = sum(f.stat().st_size for f in paz_files)
        total_paz_size += paz_size

        entry = papgt_lookup.get(name)
        lang = f"0x{entry['language']:04X}" if entry else "???"
        opt = "Y" if entry and entry["is_optional"] else "N" if entry else "?"

        if pamt_path.exists():
            try:
                pamt = crimson_rs.parse_pamt_file(str(pamt_path))
                n_dirs = len(pamt["directories"])
                n_files = sum(len(d["files"]) for d in pamt["directories"])
                total_files += n_files
            except Exception:
                n_dirs = "ERR"
                n_files = "ERR"
        else:
            n_dirs = "-"
            n_files = "-"

        print(
            f"{name:>6}  {paz_count:>9}  {n_dirs:>5}  {n_files:>6}  "
            f"{paz_size / 1_000_000:>10.1f}  {lang:>6}  {opt:>3}"
        )

    print("-" * 60)
    print(f"Total: {len(group_dirs)} groups, {total_files} files, {total_paz_size / 1_000_000_000:.2f} GB")


def list_group_files(game_dir: str, group_name: str) -> None:
    """List all files in a specific pack group."""
    game_path = Path(game_dir)
    pamt_path = game_path / group_name / "0.pamt"

    if not pamt_path.exists():
        print(f"ERROR: PAMT not found: {pamt_path}")
        sys.exit(1)

    pamt = crimson_rs.parse_pamt_file(str(pamt_path))
    print(f"Group {group_name}: {len(pamt['chunks'])} chunks, {len(pamt['directories'])} directories")
    print()

    for d in pamt["directories"]:
        print(f"  {d['path']}/")
        for f in d["files"]:
            comp = f["compressed_size"]
            uncomp = f["uncompressed_size"]
            ratio = comp / uncomp * 100 if uncomp > 0 else 0
            crypto_name = ["None", "ICE", "AES", "ChaCha20"][f["crypto"]] if f["crypto"] < 4 else "?"
            comp_name = ["None", "Partial", "LZ4", "Zlib", "QuickLZ"][f["compression"]] if f["compression"] < 5 else "?"
            print(
                f"    {f['name']:<40s}  {uncomp:>10,} B  "
                f"{comp_name:<5s} {ratio:5.1f}%  {crypto_name}"
            )


parser = argparse.ArgumentParser(description="Scan Crimson Desert game archives")
parser.add_argument("--game-dir", default=DEFAULT_GAME_DIR, help="Path to game installation directory")
parser.add_argument("--group", default=None, help="List files in a specific group (e.g. 0008)")
args = parser.parse_args()

if args.group:
    list_group_files(args.game_dir, args.group)
else:
    scan_game_dir(args.game_dir)
