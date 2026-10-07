"""Deterministic requests for Wasm_Window and Xw_Window (tools/oracle/tkservice_probe_window.cxx).

`sv_window` compares a canvas window without Emscripten (mapping, resize, ratio, size and
position of virtual and non-virtual windows, logical sizes scaled by the pixel ratio, the
mouse button and key translations) and the display-free Xw_Window parts: the X key symbol
translation over digits, letters, function keys, named and media keys, and the constructor
checks of a null display.
"""
from __future__ import annotations
import random

from tkservice_corpus_camera import numbers

KEYSYMS = [0x30, 0x39, 0x41, 0x5A, 0x61, 0x7A, 0x5B, 0xFFBE, 0xFFC9, 0xFFCA, 0xFFD5, 0x20, 0x27, 0x2C, 0x2D, 0x2E,
           0x3B, 0x3D, 0x5C, 0x5D, 0xFF08, 0xFF09, 0xFF0D, 0xFF8D, 0xFF1B, 0xFF50, 0xFF51, 0xFF52, 0xFF53, 0xFF54,
           0xFF55, 0xFF56, 0xFF57, 0xFF67, 0xFF7F, 0xFFAA, 0xFFAB, 0xFFAD, 0xFFAF, 0xFFE1, 0xFFE2, 0xFFE3, 0xFFE4,
           0xFFE9, 0xFFEA, 0xFFFF, 0x1008FF11, 0x1008FF12, 0x1008FF13, 0x1008FF14, 0x1008FF15, 0x1008FF16,
           0x1008FF17, 0x1008FF18, 0x1008FF26, 0x1008FF27, 0x1008FF28, 0x1008FF29, 0x1008FF30, 0x20AC, 0]


def append_window_requests(requests: list[str]) -> None:
    """Append seeded window requests."""
    rng = random.Random(20261016)
    for index in range(24):
        size = rng.choice([[640, 480], [0, 480], [800, 0], [-5, 7], [1, 1]])
        logical = rng.choice([[320.5, 200.25], [0.0, 0.0], [99.9, 1.0]])
        keys = KEYSYMS if index == 0 else rng.sample(KEYSYMS, 8)
        fields = [index % 2, rng.choice([1.0, 2.0, 1.5, 0.75])] + size + logical + [rng.randrange(0, 8), rng.choice([0, 13, 65])]
        requests.append('sv_window ' + numbers(fields + [len(keys)] + keys))
