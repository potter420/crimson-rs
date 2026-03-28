"""Example: Pack a mod folder into a new pack group for Crimson Desert.

This script reads all files from a mod folder, packs them into .paz
archive chunks with a 0.pamt index, and updates the game's PAPGT with
the new group entry. Output goes to a separate packs directory —
the game install is never modified directly.

Prerequisites:
    pip install crimson-rs   (or: maturin develop)

Usage:
    python examples/pack_mod_example.py
"""

from crimson_rs import Compression, Crypto, Language
from crimson_rs.pack_mod import pack_mod

# Adjust these paths to match your setup
GAME_DIR = r"F:\Program\Steam\steamapps\common\Crimson Desert"
MOD_FOLDER = r"E:\OpensourceGame\CrimsonDesert\Godmod\modified"
OUTPUT_DIR = r"E:\OpensourceGame\CrimsonDesert\Godmod\packs"
GROUP_NAME = "0070"

pack_mod(
    game_dir=GAME_DIR,
    mod_folder=MOD_FOLDER,
    output_dir=OUTPUT_DIR,
    group_name=GROUP_NAME,
    compression=Compression.LZ4,
    crypto=Crypto.NONE,
    language=Language.ALL,
    # language=Language.ENG | Language.KOR,  # or combine specific flags
)
