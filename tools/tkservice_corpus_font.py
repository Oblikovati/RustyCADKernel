"""Deterministic requests for Font_FontMgr, Font_FTFont and Font_TextFormatter
(tools/oracle/tkservice_probe_font.cxx).

The fonts are the system fonts of the machine that runs both probes. `sv_fontmgr` compares the
font database (every font, aspect, path and face id), the aliases and FindFont over names,
strict levels and aspects; `sv_ftfont` compares fonts found by name at several sizes,
resolutions, hinting modes, width scalings and fallback settings: metrics, each character's
bitmap (hash), advances with kerning, glyph identity and outline, and the text bounding boxes;
`sv_textfmt` compares formatted texts (line feeds, tabulations, wrapping, word wrapping and
every alignment).
"""
from __future__ import annotations
import random

TEXTS = [
    [ord(c) for c in 'Hello'],
    [ord(c) for c in 'AVATAR Wolf'],
    [ord(c) for c in 'a\tb\tc'],
    [ord(c) for c in 'line one\nline two\n\nend'],
    [ord(c) for c in 'The quick brown fox jumps over the lazy dog'],
    [0x41, 0x0D, 0x42, 0x07, 0x43],
    [0x4E2D, 0x6587, 0x41],
    [0xAC00, 0x0627, 0x0628],
    [0xE9, 0xF1, 0x20AC, 0x2122],
    [ord(c) for c in 'iiii WWWW'],
]


def numbers(values: list[float]) -> str:
    """The fields of a request, with every double spelled exactly."""
    return ' '.join(repr(float(v)) for v in values)


def ftfont_request(rng: random.Random) -> list[float]:
    """`Name Aspect Strict Size Resolution Hinting Italic Single Width Fallback Text`."""
    text = rng.choice(TEXTS)
    hinting = rng.choice([0, 1, 2, 1 | 16, 32, 2 | 32, 1 | 2])
    return [rng.choice([0, 1, 2, 3, 4, 7, 8, 9, -1, -1]), rng.randrange(-1, 4), rng.choice([2, 2, 1, 0]),
            rng.choice([12, 16, 9, 48, 0]), rng.choice([72, 96, 0]), hinting, rng.randrange(2), rng.randrange(2),
            rng.choice([1.0, 1.0, 0.5, 1.25, 0.0]), rng.randrange(2), len(text), *text]


def textfmt_request(rng: random.Random) -> list[float]:
    """`Name AlignX AlignY Wrapping WordWrap Size Text`."""
    text = rng.choice(TEXTS)
    return [rng.choice([0, 1, 3]), rng.randrange(3), rng.randrange(4), rng.choice([0.0, 0.0, 40.0, 80.0, 200.0]),
            rng.randrange(2), rng.choice([12, 16, 24]), len(text), *text]


def append_font_requests(requests: list[str]) -> None:
    """Append the font database request, then seeded font and formatter requests."""
    requests.append('sv_fontmgr 0')
    rng = random.Random(20261011)
    for _ in range(40):
        requests.append('sv_ftfont ' + numbers(ftfont_request(rng)))
    for _ in range(40):
        requests.append('sv_textfmt ' + numbers(textfmt_request(rng)))
