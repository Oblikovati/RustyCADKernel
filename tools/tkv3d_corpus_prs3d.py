"""Deterministic requests for the Prs3d numeric parts (tools/oracle/tkv3d_probe_prs3d.cxx).

`v3_quadric` triangulates every quadric tool (cylinder and cone, disk with an angle range,
sphere, torus segment, sector) over seeded slice and stack counts and transformations,
including degenerate tools whose normals raise; `v3_arrow` draws segment and shaded arrows;
`v3_prs3d` compares the deflection of corners and of void, closed and open boxes, segment
matching and the box segments.
"""
from __future__ import annotations
import math
import random

from tkservice_corpus_camera import numbers


def trsf(rng: random.Random) -> list[float]:
    """`Tx Ty Tz Ax Ay Az Angle`: a translation and a rotation about a unit-free axis."""
    axis = rng.choice([[0, 0, 1], [1, 0, 0], [1, 1, 1], [0.3, -0.7, 0.2]])
    shift = [rng.uniform(-50, 50) for _ in range(3)] if rng.random() < 0.7 else [0, 0, 0]
    return shift + axis + [rng.choice([0.0, math.pi / 3, rng.uniform(-6, 6)])]


def quadric_params(rng: random.Random, kind: int) -> list[float]:
    """The five tool parameters of a kind, sometimes degenerate."""
    if kind == 0:
        return [rng.choice([1.0, 0.0, 2.5]), rng.choice([1.0, 0.0, 0.5]), rng.choice([2.0, 0.0, 7.25]), 0, 0]
    if kind == 1:
        return [rng.choice([0.0, 0.5]), rng.choice([1.0, 3.0]), rng.uniform(-1, 1), rng.uniform(1, 7), 0]
    if kind == 3:
        return [rng.uniform(1, 5), rng.uniform(0.1, 0.9), rng.uniform(-1, 1), rng.uniform(1, 7), rng.uniform(0.5, 6.3)]
    return [rng.uniform(0.1, 10.0), 0, 0, 0, 0]


def append_prs3d_requests(requests: list[str]) -> None:
    """Append seeded quadric, arrow and package requests."""
    rng = random.Random(20261008)
    for index in range(80):
        kind = index % 5
        slices, stacks = rng.choice([1, 3, 4, 12, 20]), rng.choice([1, 2, 5])
        requests.append('v3_quadric ' + numbers([kind] + quadric_params(rng, kind) + [slices, stacks] + trsf(rng)))
    for _ in range(40):
        loc = [rng.uniform(-10, 10) for _ in range(3)]
        direction = rng.choice([[0, 0, 1], [1, 0, 0], [0, -1, 0], [1, 2, 3], [-0.5, 0.25, 0.1]])
        values = loc + direction + [rng.uniform(0.05, 1.5), rng.uniform(0.1, 20), rng.choice([3, 8, 15])]
        values += [rng.choice([0.0, 0.1, 1.0]), rng.uniform(0, 30), rng.choice([0.0, 0.3, 2.0]),
                   rng.choice([0.0, 1.0, 5.0, 40.0]), rng.choice([3, 6, 12])]
        requests.append('v3_arrow ' + numbers(values))
    for _ in range(40):
        mn = [rng.uniform(-100, 100) for _ in range(3)]
        mx = [v + rng.choice([-5.0, 0.0, rng.uniform(0, 300)]) for v in mn]
        point = [rng.uniform(-2, 2) for _ in range(3)]
        p1 = [rng.uniform(-2, 2) for _ in range(3)]
        p2 = rng.choice([p1, [rng.uniform(-2, 2) for _ in range(3)]])
        values = mn + mx + [rng.choice([0.001, 0.01, 1e-9]), rng.choice([0.1, 1e-4]), rng.randrange(2)]
        values += point + [rng.choice([0.1, 1.0, 5.0])] + p1 + p2
        requests.append('v3_prs3d ' + numbers(values))
