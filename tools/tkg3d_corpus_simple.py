"""TKG3d corpus: points, directions, vectors, axis placements and transformations
(tools/oracle/tkg3d_probe_simple.cxx)."""
from __future__ import annotations
import random
from tkg3d_corpus import axis, fmt, frame, moves, point, trsf, unit


def append_simple_requests(rng: random.Random, requests: list[str]) -> None:
    """Points, directions (null ones included), vectors, axes and transformations."""
    for _ in range(12):
        requests.append('g3_point ' + fmt([*point(rng), *point(rng), *point(rng), *moves(rng)]))
    directions = [point(rng) for _ in range(10)] + [[0.0, 0.0, 0.0], [1e-300, 0.0, 0.0], [0.0, 3.0, 0.0]]
    for d in directions:
        target = rng.choice([point(rng), [0.0, 0.0, 0.0], [0.0, 0.0, 1e-200]])
        other = rng.choice([point(rng), d, [0.0, 0.0, 0.0]])
        requests.append('g3_direction ' + fmt([*d, *target, *other, *point(rng), *moves(rng)]))
    for _ in range(12):
        a = rng.choice([point(rng), [0.0, 0.0, 0.0]])
        b = rng.choice([point(rng), a, [0.0, 0.0, 0.0]])
        s = rng.choice([rng.uniform(-3, 3), 0.0, 2.0])
        requests.append('g3_vector ' + fmt([*a, *b, *point(rng), s, *point(rng), *point(rng), *moves(rng)]))
    for _ in range(10):
        requests.append('g3_axis1 ' + fmt([*axis(rng), *axis(rng), *unit(rng), *point(rng), *moves(rng)]))
    for index in range(12):
        n = unit(rng)
        vx = n if index % 4 == 0 else unit(rng)
        d = rng.choice([unit(rng), n])
        requests.append('g3_axis2 ' + fmt([*frame(rng), *point(rng), *n, *vx, *d, *unit(rng), *axis(rng), *moves(rng)]))
    for index in range(16):
        power = rng.choice([0, 1, 2, 3, -1, -2])
        factor = rng.choice([2.0, -1.0, 0.0, 1e-20])
        requests.append('g3_trsf ' + fmt([*trsf(rng, index % 8), *trsf(rng), power, *frame(rng), *frame(rng),
                                          *point(rng), *point(rng), *axis(rng), *frame(rng),
                                          rng.uniform(-4, 4), factor, *point(rng)]))
