"""Deterministic requests for the ChFi3d probes (tools/oracle/tkfillet_probe_chfi3d.cxx).

`fl_hatching_domains` trims hatching lines by closed polygons, through their interior,
along their edges (where ChFi3d_ComputeHatchingDomains keeps the segments), through their
vertices and outside, with the segments kept or not and two tolerances.
"""
from __future__ import annotations

from tkfillet_corpus_blendfunc import numbers

SQUARE = [(0, 0), (10, 0), (10, 10), (0, 10)]
NOTCH = [(-10, -10), (0, -10), (0, 0), (10, 0), (10, 10), (-10, 10)]
SPLIT_EDGE = [(0, 0), (4, 0), (10, 0), (10, 10), (0, 10)]
TRIANGLE = [(0, 0), (10, 0), (3, 8)]
STEPS = [(0, 0), (4, 0), (4, 3), (8, 3), (8, 0), (12, 0), (12, 6), (0, 6)]

LINES = [
    ((0, 0), (1, 0)),
    ((0, 5), (1, 0)),
    ((0, 10), (1, 0)),
    ((0, -1), (1, 0)),
    ((0, 0), (0, 1)),
    ((10, 0), (0, 1)),
    ((0, 0), (1, 1)),
    ((0, 3), (1, 0)),
    ((3, 8), (-3, -8)),
    ((5, -10), (-1, 0)),
]


def hatching(polygon, line, tol, keep) -> str:
    fields = [len(polygon)]
    for x, y in polygon:
        fields += [float(x), float(y)]
    (px, py), (dx, dy) = line
    fields += [float(px), float(py), float(dx), float(dy), tol, 1 if keep else 0]
    return 'fl_hatching_domains ' + numbers(fields)


import math

# `fl_solid kind a b c d` (tools/oracle/tkfillet_probe_chfi3d.cxx).
SOLIDS = [
    (0, 10, 20, 30, 0),
    (0, 1, 1, 1e-3, 0),
    (1, 10, 8, 5, 2),
    (1, 10, 10, 3, 4),
    (2, 5, 10, 0, 0),
    (3, 5, 2, 8, 0),
    (3, 5, 0, 8, 0),
    (4, 5, 0, 0, 0),
    (5, 10, 3, 0, 0),
    (6, 5, 3, 4, math.pi / 2),
    (6, 5, 3, 4, 2 * math.pi),
]


def guide(kind, p, n, x, a):
    """`fl_guide`: 0 the line (p, n), 1 the circle of radius a in the frame (p, n, x)."""
    return [kind, *p, *n, *x, a]


GUIDES = [
    (guide(0, (0, 0, 0), (1, 0, 0), (0, 1, 0), 0), guide(0, (0, 2, 0), (1, 0, 0), (0, 1, 0), 0)),
    (guide(0, (0, 0, 0), (1, 0, 0), (0, 1, 0), 0), guide(0, (0, 2, 1), (1, 1, 0), (0, 0, 1), 0)),
    (guide(1, (0, 0, 0), (0, 0, 1), (1, 0, 0), 5), guide(1, (0, 0, 0), (0, 0, 1), (1, 0, 0), 3)),
    (guide(1, (0, 0, 0), (0, 0, 1), (1, 0, 0), 5), guide(1, (1, 2, 0), (0, 0, 1), (1, 0, 0), 4)),
    (guide(1, (0, 0, 0), (0, 0, 1), (1, 0, 0), 5), guide(0, (0, 0, 3), (1, 0, 0), (0, 1, 0), 0)),
    (guide(1, (0, 0, 0), (0, 0, 1), (1, 0, 0), 5), guide(1, (0, 0, 1), (0, 1, 1), (1, 0, 0), 5)),
]


def append_chfi3d_requests(requests: list[str]) -> None:
    """Append the ChFi3d request families."""
    for polygon in (SQUARE, NOTCH, SPLIT_EDGE, TRIANGLE, STEPS):
        for line in LINES:
            for tol, keep in ((1.0e-7, False), (1.0e-7, True), (1.0e-3, False)):
                requests.append(hatching(polygon, line, tol, keep))
    for spec in SOLIDS:
        requests.append('fl_concave_side ' + numbers(list(spec)))
    params = [0.0, 0.5, 1.0, 2.5, -3.0]
    for g1, g2 in GUIDES:
        for x0, a, b in ((0.3, -1.0, 4.0), (2.0, 0.0, 6.0), (-2.5, -3.0, 3.0)):
            fields = [*g1, *g2, len(params), *params, x0, 1.0e-9, a, b]
            requests.append('fl_search_sing ' + numbers(fields))
