"""Deterministic requests for the Aspect grid snapping
(tools/oracle/tkservice_probe_grid.cxx).

`sv_rgrid` compares Aspect_RectangularGrid::Init and Compute (and Hit after Activate) on
axis-aligned, skewed and rotated networks, parallel networks (Standard_NumericError),
negative and null steps, and points on lines, between lines and far away. `sv_cgrid`
compares Aspect_CircularGrid::SetGridValues and Compute: exact axis directions of an
unrotated grid, rotated grids, odd division counts, points below the origin, the origin
itself (0/0 direction) and invalid steps and division counts. One division on an unrotated
grid is not requested: Compute divides by `nmax / 4 == 0` natively (SIGFPE).
"""
from __future__ import annotations
import math
import random


def numbers(values: list[float]) -> str:
    """The fields of a request, with every double spelled exactly."""
    return ' '.join(repr(float(v)) for v in values)


def rectangular_request(grid: list[float], point: list[float]) -> str:
    """One `sv_rgrid XStep YStep XO YO A1 A2 Rot X Y` request line."""
    return 'sv_rgrid ' + numbers(grid + point)


def circular_request(grid: list[float], point: list[float]) -> str:
    """One `sv_cgrid RadiusStep Divisions XO YO Rot X Y` request line."""
    return 'sv_cgrid ' + numbers(grid + point)


RECTANGULAR_GRIDS = [
    [1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0], [2.5, 0.5, 1.0, -3.0, 0.0, 0.0, 0.0],
    [1.0, 1.0, 0.0, 0.0, 0.3, 0.0, 0.0], [1.0, 2.0, 0.5, 0.5, 0.0, 0.4, 0.0],
    [1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.7], [0.1, 0.3, -2.0, 4.0, 0.2, -0.3, 1.1],
    [1.0, 1.0, 0.0, 0.0, math.pi / 2, 0.0, 0.0], [1.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0], [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [-2.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
]
CIRCULAR_GRIDS = [
    [1.0, 8, 0.0, 0.0, 0.0], [0.5, 6, 1.0, 2.0, 0.0], [1.0, 5, 0.0, 0.0, 0.0],
    [2.0, 12, -1.0, 0.5, 0.3], [1.0, 1, 0.0, 0.0, 0.4], [1.0, 2, 0.0, 0.0, 0.0],
    [1.0, 3, 0.0, 0.0, 0.0], [0.0, 8, 0.0, 0.0, 0.0], [-1.0, 8, 0.0, 0.0, 0.0],
    [1.0, 0, 0.0, 0.0, 0.0], [1.0, -4, 0.0, 0.0, 0.0],
]
POINTS = [[0.4, 1.6], [-3.7, 2.2], [0.0, 0.0], [10.25, -7.75], [0.5, 0.5], [1e6, -1e6],
          [-0.0, -1.0], [2.0, -0.01]]


def append_grid_requests(requests: list[str]) -> None:
    """Append the grid requests: fixed grids at fixed points, then seeded random ones."""
    rng = random.Random(20261007)
    for grid in RECTANGULAR_GRIDS:
        requests.extend(rectangular_request(grid, point) for point in POINTS)
    for grid in CIRCULAR_GRIDS:
        requests.extend(circular_request(grid, point) for point in POINTS)
    for _ in range(48):
        grid = [rng.uniform(0.05, 3.0), rng.uniform(0.05, 3.0), rng.uniform(-5, 5), rng.uniform(-5, 5),
                rng.uniform(-1.5, 1.5), rng.uniform(-1.5, 1.5), rng.uniform(-3.2, 3.2)]
        requests.append(rectangular_request(grid, [rng.uniform(-20, 20), rng.uniform(-20, 20)]))
        circle = [rng.uniform(0.05, 3.0), rng.randint(2, 36), rng.uniform(-5, 5), rng.uniform(-5, 5),
                  rng.choice([0.0, rng.uniform(-3.2, 3.2)])]
        requests.append(circular_request(circle, [rng.uniform(-20, 20), rng.uniform(-20, 20)]))
