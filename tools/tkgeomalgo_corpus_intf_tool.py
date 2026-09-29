"""Deterministic requests for Intf_Tool (tools/oracle/tkgeomalgo_probe_intf_tool.cxx).

`intf_tool2d` and `intf_tool3d` run one to three box computations on one tool (the
segment arrays keep their values between calls). The curves are lines (with components
exactly zero, so the parallel branches run inside and outside the domain), hyperbolas
and parabolas of several sizes and orientations; the domains are void, whole, finite
with a gap, and finite with every combination of open sides.
"""
from __future__ import annotations
import math
import random

from tkgeomalgo_corpus_intf import numbers

DIRECTIONS2D = [(1.0, 0.0), (0.0, 1.0), (-1.0, 0.0), (0.0, -1.0), (1.0, 1.0), (-1.0, 2.0),
                (math.cos(0.3), math.sin(0.3)), (1.0, 1e-12)]
DIRECTIONS3D = [(1.0, 0.0, 0.0), (0.0, 1.0, 0.0), (0.0, 0.0, 1.0), (-1.0, 0.0, 0.0), (1.0, 1.0, 0.0),
                (1.0, 1.0, 1.0), (-1.0, 2.0, 0.5), (0.0, -1.0, 1e-12)]


def domain2d(rng: random.Random) -> list[float]:
    """A 2d domain spec: void, whole, or bounds with a gap and an open-side mask."""
    mode = rng.choice([0, 1, 2, 2, 2, 2, 2, 2])
    if mode != 2:
        return [mode]
    size = rng.choice([0.5, 1.0, 5.0, 20.0])
    center = [rng.choice([0.0, 1.0, rng.uniform(-3, 3)]) for _ in range(2)]
    open_mask = rng.choice([0, 0, 0, rng.randrange(16)])
    return [2, center[0] - size, center[1] - size, center[0] + size, center[1] + size,
            rng.choice([0.0, 0.0, 0.1]), open_mask]


def domain3d(rng: random.Random) -> list[float]:
    """A 3d domain spec: void, whole, or bounds with a gap and an open-side mask."""
    mode = rng.choice([0, 1, 2, 2, 2, 2, 2, 2])
    if mode != 2:
        return [mode]
    size = rng.choice([0.5, 1.0, 5.0, 20.0])
    center = [rng.choice([0.0, 1.0, rng.uniform(-3, 3)]) for _ in range(3)]
    open_mask = rng.choice([0, 0, 0, rng.randrange(64)])
    return [2, *(c - size for c in center), *(c + size for c in center), rng.choice([0.0, 0.0, 0.1]), open_mask]


def call2d(rng: random.Random) -> list[float]:
    """One 2d call: a line, hyperbola or parabola, then its domain."""
    kind = rng.randrange(3)
    location = [rng.choice([0.0, 0.5, rng.uniform(-4, 4)]) for _ in range(2)]
    values: list[float] = [kind, *location, *rng.choice(DIRECTIONS2D)]
    if kind == 1:
        values += [rng.randrange(2), rng.choice([0.5, 2.0, 3.0]), rng.choice([0.1, 1.0, 2.5])]
    elif kind == 2:
        values += [rng.randrange(2), rng.choice([0.0, 0.25, 1.0, 4.0])]
    return values + domain2d(rng)


def frame(rng: random.Random) -> list[float]:
    """A normal and an X direction that are never parallel."""
    normal = rng.choice([(0.0, 0.0, 1.0), (1.0, 0.0, 0.0), (0.0, 1.0, 1.0)])
    x_direction = {(0.0, 0.0, 1.0): (1.0, 0.0, 0.0), (1.0, 0.0, 0.0): (0.0, 1.0, 0.0),
                   (0.0, 1.0, 1.0): (1.0, 0.0, 0.0)}[normal]
    return [*normal, *x_direction]


def call3d(rng: random.Random) -> list[float]:
    """One 3d call: a line, hyperbola or parabola, then its domain."""
    kind = rng.randrange(3)
    values: list[float] = [kind, *(rng.choice([0.0, 0.5, rng.uniform(-4, 4)]) for _ in range(3))]
    if kind == 0:
        values += rng.choice(DIRECTIONS3D)
    elif kind == 1:
        values += frame(rng) + [rng.choice([0.5, 2.0, 3.0]), rng.choice([0.1, 1.0, 2.5])]
    else:
        values += frame(rng) + [rng.choice([0.0, 0.25, 1.0, 4.0])]
    return values + domain3d(rng)


def fixed_requests() -> list[str]:
    """The upstream GTest configurations and the curve-misses-domain cases."""
    whole5_2d = [2, -5.0, -5.0, 5.0, 5.0, 0.0, 0]
    whole5_3d = [2, -5.0, -5.0, -5.0, 5.0, 5.0, 5.0, 0.0, 0]
    return [
        'intf_tool2d ' + numbers([1, 1, 0, 0, 1, 0, 1, 2.0, 1.0] + whole5_2d),
        'intf_tool2d ' + numbers([1, 2, 0, 0, 1, 0, 1, 1.0] + whole5_2d),
        'intf_tool2d ' + numbers([1, 1, 100, 100, 1, 0, 1, 0.1, 0.05, 2, -1.0, -1.0, 1.0, 1.0, 0.0, 0]),
        'intf_tool3d ' + numbers([1, 2, 0, 0, 0, 0, 0, 1, 1, 0, 0, 1.0] + whole5_3d),
        'intf_tool3d ' + numbers([1, 1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 2.0, 1.0] + whole5_3d),
        'intf_tool2d ' + numbers([2, 0, 7, 0, 0, 1, 2, -1.0, -1.0, 1.0, 1.0, 0.0, 0, 0, 0, 0, 1, 0] + whole5_2d),
        'intf_tool3d ' + numbers([1, 0, 0, 0, 9, 0, 0, 1] + whole5_3d),
        # Tangent to a face at the apex: a double root, merged by the Inters3d sort, and
        # a crossing whose sine is below Precision::Angular.
        'intf_tool3d ' + numbers([1, 2, 0, 0, 0, 0, 0, 1, 1, 0, 0, 1.0, 2, 0.0, -5.0, -5.0, 5.0, 5.0, 5.0, 0.0, 0]),
        'intf_tool3d ' + numbers([1, 1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 2.0, 1.0, 2, 2.0, -5.0, -5.0, 7.0, 5.0, 5.0, 0.0, 0]),
        'intf_tool2d ' + numbers([1, 2, 0, 0, 1, 0, 1, 1.0, 2, 0.0, -5.0, 5.0, 5.0, 0.0, 0]),
        'intf_tool2d ' + numbers([1, 1, 0, 0, 1, 0, 1, 2.0, 1.0, 2, 2.0, -5.0, 7.0, 5.0, 0.0, 0]),
    ]


def append_intf_tool_requests(requests: list[str]) -> None:
    """Append the fixed then the seeded random Intf_Tool requests."""
    requests.extend(fixed_requests())
    rng = random.Random(1993_06_23)
    for index in range(400):
        calls = 1 + index % 3
        request: list[float] = [calls]
        for _ in range(calls):
            request += call2d(rng)
        requests.append('intf_tool2d ' + numbers(request))
    for index in range(400):
        calls = 1 + index % 3
        request = [calls]
        for _ in range(calls):
            request += call3d(rng)
        requests.append('intf_tool3d ' + numbers(request))
