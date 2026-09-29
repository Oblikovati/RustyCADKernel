"""Deterministic requests for the Intf interferences (tools/oracle/tkgeomalgo_probe_intf_poly.cxx).

`intf_ipoly2d` runs Intf_InterferencePolygon2d performs and constructors on one object:
crossing zigzags, collinear overlaps and near-parallel segments (tangent zones), shared
and touching vertices, zero-length segments, closed and self-crossing polylines, with a
null deflection (Epsilon(1000.)) and positive ones.

`intf_ipp` runs the generic polygon-polyhedron interference, instantiated in the probe
over a triangulated height field whose outer edges are its boundary edges: polylines
through the triangles, through vertices and along edges, lying in a triangle plane,
straight lines and sets of lines, every Perform, Interference and constructor overload,
with and without the grid.
"""
from __future__ import annotations
import math
import random

from tkgeomalgo_corpus_intf import numbers


def polygon2d(points: list[tuple[float, float]], closed: bool = False, deflection: float = 0.0) -> list[float]:
    """A polygon spec `closed deflection n x y...`."""
    return [int(closed), deflection, len(points)] + [c for point in points for c in point]


def random_polygon2d(rng: random.Random) -> list[float]:
    """A zigzag, an axis-aligned staircase or a star, snapped or not to a grid."""
    count = rng.randint(2, 7)
    snap = rng.random() < 0.5
    coordinate = (lambda: float(rng.randint(-3, 3))) if snap else (lambda: rng.uniform(-3, 3))
    points = [(coordinate(), coordinate()) for _ in range(count)]
    if rng.random() < 0.2:
        points.append(points[0])
    return polygon2d(points, rng.random() < 0.3, rng.choice([0.0, 0.0, 1e-7, 0.01, 0.3]))


def fixed_polygon_pairs() -> list[list[float]]:
    """Overlaps, near-parallel pairs, touching vertices and degenerate segments."""
    x_axis = polygon2d([(0.0, 0.0), (4.0, 0.0)])
    return [
        [1, 0, *polygon2d([(0.0, 0.0), (2.0, 2.0)]), *polygon2d([(0.0, 2.0), (2.0, 0.0)])],
        [1, 0, *x_axis, *polygon2d([(1.0, 0.0), (3.0, 0.0)])],
        [1, 0, *x_axis, *polygon2d([(1.0, 1e-9), (3.0, 1e-9)], deflection=1e-7)],
        [1, 0, *x_axis, *polygon2d([(1.0, 0.0), (3.0, 1e-9)], deflection=1e-7)],
        [1, 0, *x_axis, *polygon2d([(-1.0, 0.01), (5.0, -0.01)], deflection=0.05)],
        [1, 0, *x_axis, *polygon2d([(4.0, 0.0), (4.0, 3.0)])],
        [1, 0, *x_axis, *polygon2d([(2.0, 0.0), (2.0, 0.0), (2.0, 3.0)])],
        [1, 0, *polygon2d([(0.0, 0.0), (1.0, 0.0), (2.0, 0.0), (3.0, 0.0)]),
         *polygon2d([(0.5, 0.0), (1.5, 0.0), (2.5, 0.0)], deflection=0.01)],
        [1, 0, *x_axis, *polygon2d([(1.0, 0.0), (3.0, 0.0)], deflection=0.5)],
        [2, 0, *x_axis, *polygon2d([(1.0, 1.0), (1.0, -1.0)]), 1,
         *polygon2d([(0.0, 0.0), (2.0, 0.0), (2.0, 2.0), (0.0, -1.0), (0.0, 0.0)], closed=True)],
        [1, 1, *polygon2d([(0.0, 0.0), (4.0, 0.0), (4.0, 1.0), (2.0, -1.0), (1.0, 3.0), (0.0, 0.0)], closed=True)],
        [1, 3, *polygon2d([(0.0, 0.0), (2.0, 0.0), (1.0, 0.0), (3.0, 0.0)], deflection=0.01)],
        [1, 2, *polygon2d([(0.0, 0.0), (1.0, 1.0)]), *polygon2d([(5.0, 5.0), (6.0, 6.0)])],
        # Crossing at an angle below Precision::Angular with no end contact: the two ends
        # of the interference radius.
        [1, 0, *polygon2d([(0.0, 0.0), (1000.0, 0.0)]), *polygon2d([(0.0, -1e-10), (1000.0, 1e-10)])],
    ]


def append_ipoly2d_requests(requests: list[str], rng: random.Random) -> None:
    for request in fixed_polygon_pairs():
        requests.append('intf_ipoly2d ' + numbers(request))
    for index in range(300):
        performs = 1 + index % 3
        request: list[float] = [performs]
        for _ in range(performs):
            kind = rng.randrange(4)
            request += [kind] + random_polygon2d(rng)
            if kind in (0, 2):
                request += random_polygon2d(rng)
        requests.append('intf_ipoly2d ' + numbers(request))


def height_field(rng: random.Random, n: int, bumpy: bool) -> list[float]:
    """A polyhedron spec: an n x n grid over [0, n-1]^2, flat or bumpy, split into two
    triangles per cell (alternating diagonals), its outer edges as boundary edges."""
    height = (lambda x, y: 0.3 * math.sin(x + 0.7 * y)) if bumpy else (lambda x, y: 0.0)
    points = [(float(i), float(j), height(i, j)) for j in range(n) for i in range(n)]
    index = lambda i, j: j * n + i + 1  # noqa: E731
    triangles: list[tuple[int, int, int]] = []
    for j in range(n - 1):
        for i in range(n - 1):
            a, b, c, d = index(i, j), index(i + 1, j), index(i + 1, j + 1), index(i, j + 1)
            triangles += [(a, b, c), (a, c, d)] if (i + j) % 2 == 0 else [(a, b, d), (b, c, d)]
    bounds = [(index(i, 0), index(i + 1, 0)) for i in range(n - 1)]
    bounds += [(index(n - 1, j), index(n - 1, j + 1)) for j in range(n - 1)]
    bounds += [(index(i, n - 1), index(i + 1, n - 1)) for i in range(n - 1)]
    bounds += [(index(0, j), index(0, j + 1)) for j in range(n - 1)]
    deflection = rng.choice([0.0, 1e-7, 0.01, 0.1])
    border = rng.choice([0.0, 0.05, 0.5])
    return ([deflection, border, len(points)] + [c for p in points for c in p]
            + [len(triangles)] + [k for t in triangles for k in t]
            + [len(bounds)] + [k for e in bounds for k in e])


def polygon3d(rng: random.Random, n: int) -> list[float]:
    """A polyline over the field: random, through grid vertices, along a grid line, or in
    the field plane."""
    style = rng.randrange(4)
    count = rng.randint(2, 5)
    span = float(n - 1)
    if style == 0:
        points = [(rng.uniform(-0.5, span + 0.5), rng.uniform(-0.5, span + 0.5), rng.uniform(-1, 1)) for _ in range(count)]
    elif style == 1:
        points = [(float(rng.randint(0, n - 1)), float(rng.randint(0, n - 1)), rng.choice([-1.0, 0.0, 1.0])) for _ in range(count)]
    elif style == 2:
        row = float(rng.randint(0, n - 1))
        points = [(float(k), row, rng.choice([0.0, 0.5])) for k in range(count)]
    else:
        points = [(rng.uniform(0, span), rng.uniform(0, span), 0.0) for _ in range(count)]
    return [int(rng.random() < 0.3), rng.choice([0.0, 1e-7, 0.05]), len(points)] + [c for p in points for c in p]


def line3d(rng: random.Random, n: int) -> list[float]:
    """A straight line through the field: vertical, slanted or grazing."""
    point = [rng.uniform(0, n - 1), rng.uniform(0, n - 1), rng.choice([0.0, 2.0])]
    if rng.random() < 0.3:
        point[:2] = [float(rng.randint(0, n - 1)), float(rng.randint(0, n - 1))]
    direction = rng.choice([(0.0, 0.0, 1.0), (0.3, 0.1, -1.0), (1.0, 0.0, 0.0), (1.0, 1.0, 1e-9)])
    return point + list(direction)


def ipp_operation(rng: random.Random, n: int) -> list[float]:
    """One operation of `ga_ipp_operation` with its operand."""
    kind = rng.randrange(14)
    base = kind - 8 if kind >= 8 else kind
    if base in (0, 3) or kind in (6, 7):
        return [kind] + polygon3d(rng, n)
    if base in (1, 4):
        return [kind] + line3d(rng, n)
    count = rng.randint(1, 3)
    return [kind, count] + [c for _ in range(count) for c in line3d(rng, n)]


def degenerate_polyhedron() -> list[float]:
    """Collinear and repeated points: triangles with a null normal and null edges."""
    points = [(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (2.0, 0.0, 0.0), (0.0, 1.0, 0.0), (0.0, 0.0, 0.0)]
    triangles = [(1, 2, 3), (1, 5, 4), (1, 2, 4)]
    return ([0.01, 0.05, len(points)] + [c for p in points for c in p] + [len(triangles)]
            + [k for t in triangles for k in t] + [1, 1, 2])


def append_ipp_requests(requests: list[str], rng: random.Random) -> None:
    crossing = [0, 0.0, 2, 0.005, 0.005, -1.0, 0.005, 0.005, 1.0]
    for kind in (0, 3, 6, 7):
        requests.append('intf_ipp ' + numbers(degenerate_polyhedron() + [1, kind] + crossing))
    requests.append('intf_ipp ' + numbers(degenerate_polyhedron() + [1, 1, 0.5, 0.0, 1.0, 0.0, 0.0, -1.0]))
    for index in range(250):
        n = 2 + index % 3
        operations = 1 + index % 3
        request: list[float] = height_field(rng, n, index % 2 == 1) + [operations]
        for _ in range(operations):
            request += ipp_operation(rng, n)
        requests.append('intf_ipp ' + numbers(request))


def append_intf_poly_requests(requests: list[str]) -> None:
    """Append the polygon and polygon-polyhedron interference requests."""
    rng = random.Random(1992_12_21)
    append_ipoly2d_requests(requests, rng)
    append_ipp_requests(requests, rng)
