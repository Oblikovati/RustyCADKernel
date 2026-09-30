"""Deterministic requests for GeomAPI_PointsToBSplineSurface
(tools/oracle/tkgeomalgo_probe_geomapi_surface.cxx).

The grids hold smooth and wavy height fields, a small 3 x 3 grid, the minimal 2 x 2
grid, thin 2 x N and N x 2 grids, rows around a cylinder (for the periodic Init and
Interpolate) and grids whose bounds do not start at 1 (the native code sets its
multipoints at the grid indices). Every Init, Interpolate and constructor runs with
least squares, smoothing and interpolation tolerances, every parametrization and
several continuities; the height overloads run on regular grids, on a single column
and a single row, and on a null step.
"""
from __future__ import annotations
import math
from tkgeomalgo_corpus_intcurvesurface import line_of


def point_grid(rows, cols, f, rl=1, cl=1):
    values = [c for i in range(rows) for j in range(cols) for c in f(i, j)]
    return [rl, rl + rows - 1, cl, cl + cols - 1, *values]


def smooth(i, j):
    return (0.5 * i, 0.4 * j, math.sin(0.5 * i) * math.cos(0.4 * j))


def wavy(i, j):
    return (i + 0.1 * math.sin(j), j + 0.1 * math.cos(i), 0.3 * math.sin(i + 2 * j))


def around(i, j):
    angle = 2.0 * math.pi * i / 6.0
    return (math.cos(angle), math.sin(angle), 0.5 * j + 0.05 * math.sin(3 * angle))


GRIDS = [point_grid(4, 5, smooth), point_grid(6, 6, wavy), point_grid(3, 3, wavy),
         point_grid(2, 2, smooth), point_grid(2, 5, smooth), point_grid(5, 2, wavy)]
PERIODIC = point_grid(6, 4, around)
SHIFTED = [point_grid(3, 4, smooth, 0, 1), point_grid(3, 4, smooth, 1, 2)]


def heights(rows, cols, f, rl=1, cl=1):
    return [rl, rl + rows - 1, cl, cl + cols - 1, *[f(i, j) for i in range(rows) for j in range(cols)]]


HEIGHTS = [heights(4, 5, lambda i, j: math.sin(0.7 * i) + 0.2 * j * j), heights(2, 2, lambda i, j: float((i + j) % 2)),
           heights(3, 1, lambda i, j: float(i % 2)), heights(1, 3, lambda i, j: float(j % 2)),
           heights(5, 4, lambda i, j: 0.1 * i * j - 0.05 * j), heights(3, 3, lambda i, j: 1.0, 0, 0)]


def p2bss(mode, grid, args) -> str:
    return line_of('gapi_p2bss', [mode, *grid, *args])


def point_requests(grid) -> list[str]:
    """Every point-grid overload on one grid."""
    out = []
    for s in (0, 2, 4, 6):
        out.append(p2bss(0, grid, (3, 8, s, 1e-3)))
    out.append(p2bss(0, grid, (2, 5, 4, 1e-1)))
    out.append(p2bss(7, grid, (3, 8, 4, 1e-3)))
    for pt in (0, 1, 2):
        out.append(p2bss(1, grid, (pt, 3, 8, 4, 1e-3, 0)))
        out.append(p2bss(8, grid, (pt, 3, 6, 2, 1e-2)))
        out.append(p2bss(5, grid, (pt, 0)))
    out.append(p2bss(1, grid, (0, 3, 3, 4, -1.0, 0)))
    out.append(p2bss(2, grid, (1.0, 0.0, 0.0, 8, 4, 1e-3)))
    out.append(p2bss(9, grid, (0.5, 0.3, 0.2, 6, 2, 1e-2)))
    out.append(p2bss(4, grid, (0,)))
    return out


def append_p2bss_requests(requests: list[str]) -> None:
    """GeomAPI_PointsToBSplineSurface: every overload on every grid."""
    for grid in GRIDS:
        requests.extend(point_requests(grid))
    for periodic_args in ((0, 3, 8, 4, 1e-3, 1), (1, 3, 8, 4, 1e-1, 1), (0, 3, 3, 4, -1.0, 1)):
        requests.append(p2bss(1, PERIODIC, periodic_args))
    for pt in (0, 1, 2):
        requests.append(p2bss(5, PERIODIC, (pt, 1)))
    requests.append(p2bss(4, PERIODIC, (1,)))
    requests.append(p2bss(4, GRIDS[3], (1,)))
    for grid in SHIFTED:
        requests.append(p2bss(0, grid, (3, 8, 4, 1e-3)))
        requests.append(p2bss(2, grid, (1.0, 0.0, 0.0, 8, 4, 1e-3)))
    for zgrid in HEIGHTS:
        requests.append(p2bss(3, zgrid, (0.0, 1.0, 0.0, 1.0, 3, 8, 4, 1e-3)))
        requests.append(p2bss(3, zgrid, (-1.0, 0.5, 2.0, 0.25, 2, 5, 2, 1e-7)))
        requests.append(p2bss(10, zgrid, (0.0, 1.0, 0.0, 0.0, 3, 5, 4, 1e-7)))
        requests.append(p2bss(6, zgrid, (1.0, 2.0, -1.0, 0.5)))
