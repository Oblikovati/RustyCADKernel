"""Deterministic requests for the Hatch package (tools/oracle/tkgeomalgo_probe_hatch.cxx).

`hatch_hatcher` builds a hatcher (tolerance, oriented flag), adds lines of the four kinds
(AddLine with a form, AddLine by direction and distance, AddXLine, AddYLine), trims them
with whole lines, parameter ranges and oriented segments, then queries every line and
interval, including the out-of-range indices around them. The fixed scenarios cover a
closed square in both modes and orientations, open and infinite intervals, intersections
on segment ends, two intersections closer than the tolerance (OCC109), parallel and
identical trimming lines, a degenerate segment and a segment near RealLast; seeded random
polygons follow.
"""
from __future__ import annotations
import math
import random

LineSpec = tuple[float, ...]


def hatch_request(tol: float, oriented: bool, lines: list[LineSpec], trims: list[LineSpec]) -> str:
    """One `hatch_hatcher` request: each line and trim spec starts with its kind code."""
    values = [tol, float(oriented), len(lines)]
    for spec in lines:
        values.extend(spec)
    values.append(len(trims))
    for spec in trims:
        values.extend(spec)
    return 'hatch_hatcher ' + ' '.join(repr(float(v)) for v in values)


def polygon_trims(points: list[tuple[float, float]], first_index: int = 1) -> list[LineSpec]:
    """Segment trims (kind 2) along the closed polygon `points`."""
    trims = []
    for k, start in enumerate(points):
        end = points[(k + 1) % len(points)]
        trims.append((2.0, *start, *end, float(first_index + k)))
    return trims


def fixed_scenarios() -> list[str]:
    """Hand-built hatchers for the boundary branches."""
    square = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]
    grid = [(2.0, 0.25), (2.0, 0.5), (3.0, 0.5), (0.0, 0.0, 0.2, 0.6, 0.8, 2.0), (1.0, 0.6, 0.8, 0.3)]
    requests = []
    for oriented in (True, False):
        requests.append(hatch_request(1e-7, oriented, grid, polygon_trims(square)))
        requests.append(hatch_request(1e-7, oriented, grid, polygon_trims(square[::-1])))
        requests.append(hatch_request(1e-7, oriented, grid, []))
        requests.append(hatch_request(1e-7, oriented, grid, polygon_trims(square)[:1]))
        # A whole trimming line, and a ranged one ending exactly on a hatch.
        requests.append(hatch_request(1e-7, oriented, grid, [(0.0, 0.0, 0.5, 1.0, 0.0, 4.0),
                                                              (1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.5, 5.0)]))
    # Vertices on the hatching lines: intersections on segment ends in both directions.
    diamond = [(0.5, 0.0), (1.0, 0.5), (0.5, 1.0), (0.0, 0.5)]
    requests.append(hatch_request(1e-7, True, [(2.0, 0.5), (3.0, 0.5), (3.0, 1.0)], polygon_trims(diamond)))
    # Two intersections closer than the tolerance cancel each other (akm OCC109).
    requests.append(hatch_request(1e-3, True, [(3.0, 0.5)], [(2.0, 0.2, 0.0, 0.2, 1.0, 1.0),
                                                            (2.0, 0.2005, 1.0, 0.2005, 0.0, 2.0)]))
    # Parallel and identical trimming lines, a degenerate segment, a segment near RealLast.
    requests.append(hatch_request(1e-7, True, [(3.0, 0.5), (2.0, 0.0)], [
        (0.0, 0.0, 0.5, 1.0, 0.0, 1.0), (0.0, 0.0, 2.0, -1.0, 0.0, 2.0),
        (2.0, 0.3, 0.3, 0.3, 0.3, 3.0), (2.0, -1.7e308, 0.2, 1.7e308, 0.4, 4.0),
        (2.0, 0.4, -1.7e308, 0.6, 1.7e308, 5.0), (2.0, -0.1e308, 0.3, 1.55e308, 0.35, 6.0),
        (2.0, 0.45, 1.55e308, 0.45, -0.1e308, 7.0)]))
    # A negative tolerance: every segment is long enough, even a degenerate one.
    requests.append(hatch_request(-1.0, True, [(3.0, 0.5)], [(2.0, 0.3, 0.3, 0.3, 0.3, 1.0)]))
    return requests


def random_polygon(rng: random.Random) -> list[tuple[float, float]]:
    """A star-shaped polygon around the origin with 3 to 8 vertices."""
    count = rng.randint(3, 8)
    angles = sorted(rng.uniform(0.0, 6.283185307179586) for _ in range(count))
    return [(r * math.cos(a), r * math.sin(a))
            for a, r in ((a, rng.uniform(0.5, 3.0)) for a in angles)]


def append_hatch_requests(requests: list[str]) -> None:
    """Append the Hatch requests: the fixed scenarios, then seeded random polygons."""
    requests.extend(fixed_scenarios())
    rng = random.Random(20260929)
    for _ in range(24):
        points = random_polygon(rng)
        if rng.random() < 0.5:
            points.reverse()
        lines: list[LineSpec] = [(2.0, rng.uniform(-3.0, 3.0)) for _ in range(3)]
        lines += [(3.0, rng.uniform(-3.0, 3.0)) for _ in range(3)]
        angle = rng.uniform(0.0, 3.141592653589793)
        lines.append((1.0, math.cos(angle), math.sin(angle), rng.uniform(-2.0, 2.0)))
        requests.append(hatch_request(1e-7, rng.random() < 0.7, lines, polygon_trims(points)))
