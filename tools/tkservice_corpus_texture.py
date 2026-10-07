"""Deterministic requests for the Graphic3d textures (tools/oracle/tkservice_probe_texture.cxx).

`sv_cubemap` packs a patterned image of every tile grid (1x6, 6x1, 2x3, 3x2) and of shapes that
fit none, in both row orders and several pixel formats, under the default, permuted and
invalid side orders, and compares every tile of every side; `sv_segment` compares the
generation plane of segment textures, including a null segment.
"""
from __future__ import annotations
import random

SHAPES = [(2, 12), (12, 2), (4, 6), (6, 4), (3, 3), (5, 30), (1, 6), (6, 1)]
ORDERS = [[0, 1, 2, 3, 4, 5], [5, 4, 3, 2, 1, 0], [1, 0, 3, 2, 5, 4], [0, 0, 2, 3, 4, 5], [6, 1, 2, 3, 4, 5]]


def append_texture_requests(requests: list[str]) -> None:
    """Append the cube map and segment requests: fixed cases, then seeded segments."""
    for size_x, size_y in SHAPES:
        for code in range(4):
            for top_down in (0, 1):
                for order in ORDERS:
                    fields = [size_x, size_y, code, top_down, *order]
                    requests.append('sv_cubemap ' + ' '.join(str(field) for field in fields))
    segments = [[0, 0, 1, 0, 0, 3], [1, 2, 3, 4, 6, 8], [0, 0, 0, 0, 0, 0], [-1.5, 0.25, 2, 3, -4, 0.5]]
    rng = random.Random(20261013)
    segments += [[rng.uniform(-10.0, 10.0) for _ in range(6)] for _ in range(24)]
    requests.extend('sv_segment ' + ' '.join(repr(float(v)) for v in segment) for segment in segments)
