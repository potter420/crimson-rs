"""Public enums for Crimson Desert archive formats."""

from __future__ import annotations

from enum import IntEnum, IntFlag


class Compression(IntEnum):
    """Compression algorithm for pack entries."""
    NONE = 0
    LZ4 = 2
    ZLIB = 3


class Crypto(IntEnum):
    """Encryption algorithm for pack entries."""
    NONE = 0
    ICE = 1
    AES = 2
    CHACHA20 = 3


class Language(IntFlag):
    """Language bitmask flags for pack group entries."""
    KOR = 1 << 0
    ENG = 1 << 1
    JPN = 1 << 2
    RUS = 1 << 3
    TUR = 1 << 4
    SPA_ES = 1 << 5
    SPA_MX = 1 << 6
    FRE = 1 << 7
    GER = 1 << 8
    ITA = 1 << 9
    POL = 1 << 10
    POR_BR = 1 << 11
    ZHO_TW = 1 << 12
    ZHO_CN = 1 << 13
    ALL = 0x3FFF
