"""Request lines for the NCollection utilities probe family (UTF iterator, UTF string, locale).

Covers every UTF-8 lead byte alone and with continuation, truncation and ASCII followers;
overlong, surrogate and beyond-U+10FFFF encodings; UTF-16 lone, reversed and paired
surrogates; UTF-32 values around every encoding boundary; and seeded random sequences.
"""
from __future__ import annotations

import random

UTF8_SAMPLES = [
    "48 65 6c 6c 6f", "c3 a9", "e2 82 ac", "f0 9f 98 80", "41 c3 a9 e2 82 ac f0 9f 98 80 5a",
    "7f", "c2 80", "df bf", "e0 a0 80", "ef bf bf", "f0 90 80 80", "f4 8f bf bf",
    "f4 90 80 80", "ed a0 80", "ed bf bf", "ed a0 80 ed b0 80", "c0 80", "c0 80 41", "c1 bf",
    "e0 80 80", "f0 80 80 80", "f8 88 80 80 80", "fc 84 80 80 80 80", "fe", "ff", "fe ff 41",
    "80", "80 80 41", "bf", "41 80", "c3", "e2 82", "f0 9f 98", "41 c3", "41 e2 82",
    "fc 80", "f8", "68 65 6c 6c 6f 20 77 6f 72 6c 64",
]
UTF16_SAMPLES = [
    "41", "d800", "dc00", "d800 dc00", "dbff dfff", "d800 d800 dc00", "dc00 d800", "d800 41",
    "41 d800", "41 dc00 42", "ffff", "fffe", "7f 80 7ff 800", "d83d de00 41", "e9 20ac",
    "dbff", "dfff 41", "68 65 6c 6c 6f",
]
UTF32_SAMPLES = [
    "41", "7f", "80", "7ff", "800", "d7ff", "d800", "dbff", "dc00", "dfff", "e000", "ffff",
    "10000", "10ffff", "110000", "7fffffff", "80000000", "ffffffff", "41 d800 42",
    "41 110000 42", "1f600 e9 20ac", "68 65 6c 6c 6f",
]
LOCALE_SAMPLES = ["", "61", "61 62 63", "7f", "80", "c3 a9", "ff", "61 80", "20"]


def utf8_byte_cases() -> list[str]:
    """Every lead byte alone, and followed by continuation, ASCII and full-length tails."""
    lines = []
    for lead in range(1, 256):
        for tail in ["", "80", "bf", "41", "80 80 80 80 80", "bf bf bf bf bf 41"]:
            lines.append(f"ncoll_utilities_utf 8 {lead:x} {tail}".rstrip())
    return lines


def random_cases(rng: random.Random) -> list[str]:
    """Seeded random sequences in each encoding, biased towards interesting units."""
    lines = []
    for _ in range(120):
        count = rng.randrange(1, 9)
        units8 = [rng.choice([rng.randrange(1, 0x80), rng.randrange(0x80, 0x100)]) for _ in range(count)]
        lines.append("ncoll_utilities_utf 8 " + " ".join(f"{unit:x}" for unit in units8))
        units16 = [rng.choice([rng.randrange(1, 0xD800), rng.randrange(0xD800, 0xE000),
                               rng.randrange(0xE000, 0x10000)]) for _ in range(count)]
        lines.append("ncoll_utilities_utf 16 " + " ".join(f"{unit:x}" for unit in units16))
        units32 = [rng.choice([rng.randrange(1, 0x110000), rng.randrange(0x110000, 1 << 32),
                               rng.randrange(0xD800, 0xE000)]) for _ in range(count)]
        lines.append("ncoll_utilities_utf 32 " + " ".join(f"{unit:x}" for unit in units32))
    return lines


def cases() -> list[str]:
    """Deterministic utilities requests: normal, degenerate and boundary inputs."""
    lines = ["ncoll_utilities_utf 8", "ncoll_utilities_utf 16", "ncoll_utilities_utf 32"]
    lines += [f"ncoll_utilities_utf 8 {sample}" for sample in UTF8_SAMPLES]
    lines += [f"ncoll_utilities_utf 16 {sample}" for sample in UTF16_SAMPLES]
    lines += [f"ncoll_utilities_utf 32 {sample}" for sample in UTF32_SAMPLES]
    lines += [f"ncoll_utilities_locale {sample}".rstrip() for sample in LOCALE_SAMPLES]
    lines += utf8_byte_cases()
    return lines + random_cases(random.Random(20130128))
