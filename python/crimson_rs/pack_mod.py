"""Pack a mod folder into a new pack group for Crimson Desert.

Streams files from a mod folder into .paz chunks on disk, creates
the 0.pamt index, then loads the game's original meta/0.papgt,
adds the new group entry, and writes the updated copy to the
output directory — nothing in the game install is modified.
"""

from __future__ import annotations

from pathlib import Path

import crimson_rs
from crimson_rs.enums import Compression, Crypto, Language


def pack_mod(
    game_dir: str,
    mod_folder: str,
    output_dir: str,
    group_name: str,
    compression: Compression = Compression.LZ4,
    crypto: Crypto = Crypto.NONE,
    encrypt_info: bytes = b"\x00\x00\x00",
    max_chunk_size: int = 500_000_000,
    is_optional: bool = False,
    language: Language = Language.ALL,
) -> None:
    """Pack a mod folder into a new pack group.

    Reads files from mod_folder, streams them into .paz chunks under
    output_dir/group_name/, creates the 0.pamt index, loads the original
    meta/0.papgt from game_dir, adds the new entry, and writes the
    updated PAPGT to output_dir/meta/0.papgt.

    Args:
        game_dir: Path to the game installation directory.
        mod_folder: Path to the folder containing mod files.
        output_dir: Path where the packed output will be written.
        group_name: Name for the new group folder (e.g. "0070").
        compression: Compression algorithm to use.
        crypto: Encryption algorithm to use.
        encrypt_info: 3 bytes of encryption info.
        max_chunk_size: Max bytes per .paz chunk file.
        is_optional: Whether this group is optional.
        language: Language flags for this group.
    """
    game_path = Path(game_dir)
    mod_path = Path(mod_folder)
    out_path = Path(output_dir)

    # ── Validate inputs ───────────────────────────────────────────────────
    if not game_path.is_dir():
        raise FileNotFoundError(f"Game directory not found: {game_path}")

    original_papgt = game_path / "meta" / "0.papgt"
    if not original_papgt.exists():
        raise FileNotFoundError(f"Original PAPGT not found: {original_papgt}")

    if not mod_path.is_dir():
        raise FileNotFoundError(f"Mod folder not found: {mod_path}")

    group_path = out_path / group_name
    if group_path.exists():
        raise FileExistsError(f"Group directory already exists: {group_path}")

    # Create output directories
    group_path.mkdir(parents=True, exist_ok=True)
    meta_dir = out_path / "meta"
    meta_dir.mkdir(parents=True, exist_ok=True)

    # ── Step 1: Pack files into .paz chunks + 0.pamt ──────────────────────
    print(f"Packing files from: {mod_path}")
    print(f"Output group dir:   {group_path}")

    builder = crimson_rs.PackGroupBuilder(
        output_dir=str(group_path),
        compression=int(compression),
        crypto=int(crypto),
        encrypt_info=encrypt_info,
        max_chunk_size=max_chunk_size,
    )

    count = 0
    for file_path in sorted(mod_path.rglob("*")):
        if not file_path.is_file():
            continue

        rel = file_path.relative_to(mod_path)
        dir_path = str(rel.parent).replace("\\", "/")
        if dir_path == ".":
            dir_path = ""
        file_name = rel.name

        builder.add_file_from_path(dir_path, file_name, str(file_path))
        count += 1

        if count % 100 == 0:
            print(f"  Added {count} files...")

    if count == 0:
        raise ValueError(f"No files found in {mod_path}")

    print(f"Packed {count} file(s) into group '{group_name}'")
    pamt_bytes = builder.finish()
    print(f"  .paz chunk(s) + 0.pamt written to {group_path}")

    # ── Step 2: Compute PAMT checksum for PAPGT entry ─────────────────────
    # PAMT header is 12 bytes; the PAPGT entry checksum covers post-header data
    pamt_post_header = pamt_bytes[12:]
    pamt_checksum = crimson_rs.calculate_checksum(pamt_post_header)
    print(f"  PAMT checksum: 0x{pamt_checksum:08X}")

    # ── Step 3: Load original PAPGT, add entry, save to output ────────────
    print(f"Loading original PAPGT: {original_papgt}")
    papgt = crimson_rs.parse_papgt_file(str(original_papgt))
    print(f"  Original has {len(papgt['entries'])} entries")

    updated_papgt = crimson_rs.add_papgt_entry(
        papgt_data=papgt,
        group_name=group_name,
        pack_meta_checksum=pamt_checksum,
        is_optional=int(is_optional),
        language=int(language),
    )
    print(f"  Added entry for '{group_name}', now {len(updated_papgt['entries'])} entries")

    output_papgt = meta_dir / "0.papgt"
    crimson_rs.write_papgt_file(updated_papgt, str(output_papgt))
    print(f"  Written updated PAPGT to: {output_papgt}")

    # ── Done ──────────────────────────────────────────────────────────────
    print()
    print("Done! To install, copy these into the game directory:")
    print(f"  {group_path}  ->  {game_path / group_name}")
    print(f"  {output_papgt}  ->  {game_path / 'meta' / '0.papgt'}")
