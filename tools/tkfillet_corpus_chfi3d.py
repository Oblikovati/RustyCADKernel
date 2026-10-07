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


def append_chfi3d_requests(requests: list[str]) -> None:
    """Append the ChFi3d request families."""
    for polygon in (SQUARE, NOTCH, SPLIT_EDGE, TRIANGLE, STEPS):
        for line in LINES:
            for tol, keep in ((1.0e-7, False), (1.0e-7, True), (1.0e-3, False)):
                requests.append(hatching(polygon, line, tol, keep))
