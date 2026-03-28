"""Pack a mod folder into a new pack group for Crimson Desert.

Reads all files from a mod folder, packs them into .paz archive chunks
with a 0.pamt index, and updates the game's PAPGT with the new group
entry. Output goes to a separate packs directory -- the game install
is never modified directly.

Usage:
    python examples/pack_mod_example.py --game-dir /path/to/game --mod-folder /path/to/mod --output-dir /path/to/output --group 0070
"""

import argparse

from crimson_rs import Compression, Crypto, Language
from crimson_rs.pack_mod import pack_mod

parser = argparse.ArgumentParser(description="Pack mod files into a Crimson Desert archive group")
parser.add_argument("--game-dir", required=True, help="Path to game installation directory")
parser.add_argument("--mod-folder", required=True, help="Path to folder containing modified game files")
parser.add_argument("--output-dir", required=True, help="Path where packed output will be written")
parser.add_argument("--group", required=True, help="Pack group name (e.g. 0070)")
parser.add_argument("--compression", choices=["none", "lz4", "zlib"], default="lz4", help="Compression algorithm (default: lz4)")
parser.add_argument("--crypto", choices=["none", "chacha20"], default="none", help="Encryption algorithm (default: none)")
parser.add_argument("--language", default="all", help="Language flags: 'all' or hex like 0x0003 (default: all)")
args = parser.parse_args()

compression_map = {"none": Compression.NONE, "lz4": Compression.LZ4, "zlib": Compression.ZLIB}
crypto_map = {"none": Crypto.NONE, "chacha20": Crypto.CHACHA20}
language = Language.ALL if args.language == "all" else Language(int(args.language, 0))

pack_mod(
    game_dir=args.game_dir,
    mod_folder=args.mod_folder,
    output_dir=args.output_dir,
    group_name=args.group,
    compression=compression_map[args.compression],
    crypto=crypto_map[args.crypto],
    language=language,
)
