"""Deterministic requests for the V3d view camera operations (tools/oracle/tkv3d_probe_v3d.cxx).

`v3_view` opens a view of a window on the null driver and runs a seeded sequence of camera
operations (projections, sizes, zooms, rotations, turns, twists, eye and center moves,
panning, fits, axial scales, perspective focal length, resets); after each operation it
prints the camera queries and the pixel, view and world conversions. Invalid inputs (zero
size, zero depth, null projection, bad scales) keep their native exceptions.
"""
from __future__ import annotations
import math
import random

from tkservice_corpus_camera import numbers

# The number of operation codes of v3_view_op; the last code is SetUp(orientation).
OPERATION_CODES = 41


def operation(rng: random.Random, width: int, height: int) -> list[float]:
    """One `Code A B C D` operation with arguments suited to its code."""
    code = rng.randrange(OPERATION_CODES)
    angle = rng.choice([0.0, math.pi / 6, -math.pi / 4, rng.uniform(-3.5, 3.5)])
    pixel = [rng.randrange(-10, width + 10), rng.randrange(-10, height + 10)]
    pixels = pixel + [rng.randrange(-10, width + 10), rng.randrange(-10, height + 10)]
    real = [rng.choice([0.0, -1.0, rng.uniform(-200, 200)]) for _ in range(4)]
    positive = [rng.choice([0.0, 1e-3, rng.uniform(0.1, 500)]) for _ in range(4)]
    by_code = {
        0: [rng.randrange(26), rng.randrange(2), 0, 0], 1: positive, 2: [rng.uniform(0.2, 3), rng.randrange(2), 0, 0],
        3: [angle, angle / 2, -angle / 3, rng.randrange(2)], 4: [angle, -angle, angle / 5, rng.randrange(2)],
        5: [angle, 0, 0, 0], 11: pixel + [rng.uniform(0.5, 2), rng.randrange(2)], 12: pixels, 13: pixels, 16: pixel + [rng.uniform(0.5, 2), 0],
        17: pixels, 21: [angle, rng.randrange(3), real[0], real[1]], 25: pixels[:2] + [rng.randrange(3), 0], 26: pixel + [0, 0],
        29: [real[0], rng.randrange(3), rng.randrange(2), 0], 30: [real[0], rng.randrange(3), rng.randrange(2), 0],
        31: pixels, 33: positive, 36: [angle, rng.randrange(3), rng.randrange(2), 0], 37: [angle, rng.randrange(3), rng.randrange(2), 0],
        38: [angle, -angle, angle, real[0]], 39: pixels, 40: [rng.randrange(26), 0, 0, 0],
    }
    return [code] + by_code.get(code, real)


def append_v3d_requests(requests: list[str]) -> None:
    """Append seeded view operation sequences."""
    rng = random.Random(20261009)
    for _ in range(120):
        width, height = rng.choice([(400, 300), (300, 400), (640, 480), (1, 1), (17, 900)])
        count = rng.randrange(1, 7)
        ops: list[float] = []
        for _ in range(count):
            ops += operation(rng, width, height)
        requests.append('v3_view ' + numbers([width, height, count] + ops))
