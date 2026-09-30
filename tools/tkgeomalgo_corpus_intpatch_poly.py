"""Deterministic requests for the IntPatch polyhedra and polygons
(tools/oracle/tkgeomalgo_probe_intpatch_poly.cxx).

`ip_polyh` builds the polyhedron of every catalog surface (tkgeomalgo_corpus_intcurvesurface.py)
with the automatic and given subdivisions (0 clamped to 1, thin grids, poles that make null
edges) and prints its points, triangles, boxes, plane equations, containment and TriConnex
around the first triangles and the unknown triangle. `ip_bvh` builds the BVH sets and the
traversal pairs of overlapping, far and identical polyhedra; `ip_interf` computes their
interference (crossing, tangent, coplanar and self). `ip_polyline` and `ip_polyarc` build the
2d polygons of lines and arcs (deflection from the parabola, resampling near the other box,
radius doubling, offsets, the construction errors); `ip_t3bits` edits the bit sets.
"""
from __future__ import annotations
import math
from tkgeomalgo_corpus_intcurvesurface import O, SURFACES, TWO_PI, X, Z, line_of, plane, sad, sphere
from tkgeomalgo_corpus_intpatch_tools import ARCS

AUTO = [1]


def grid(nbu, nbv):
    return [0, nbu, nbv]


def poly(surface, sub):
    return [*surface, *sub]


def _polyh_requests(requests: list[str]) -> None:
    for s in SURFACES.values():
        requests.append(line_of('ip_polyh', [*poly(s, AUTO), 6, 0.0]))
        requests.append(line_of('ip_polyh', [*poly(s, grid(3, 4)), 30, 0.5]))
    for sub in (grid(0, 0), grid(1, 5), grid(5, 1), grid(-3, 2), grid(2, 2)):
        requests.append(line_of('ip_polyh', [*poly(SURFACES['sphere'], sub), 12, 1e-6]))
        requests.append(line_of('ip_polyh', [*poly(SURFACES['plane'], sub), 12, 2.0]))


PAIRS = [
    (poly(SURFACES['sphere'], grid(10, 10)), poly(sad(sphere(O, Z, 1.0)), grid(10, 10))),
    (poly(sad(sphere(O, Z, 1.0)), grid(10, 10)),
     poly(sad([1, 0.5, 0.0, 0.0, *Z, 0.8], (0.0, TWO_PI, -1.0, 1.0)), grid(10, 10))),
    (poly(sad(sphere(O, Z, 1.0)), grid(5, 5)), poly(sad(plane([10.0, 10.0, 10.0], X), (-1.0, 1.0, -1.0, 1.0)), grid(5, 5))),
    (poly(SURFACES['torus'], AUTO), poly(SURFACES['cone'], grid(8, 6))),
    (poly(SURFACES['bspline_s'], AUTO), poly(SURFACES['plane_tilt'], grid(4, 4))),
    (poly(SURFACES['plane'], grid(4, 4)), poly(sad(plane([0.0, 0.0, 0.3], Z), (-1.0, 1.5, -1.0, 1.5)), grid(3, 5))),
    (poly(SURFACES['plane'], grid(3, 3)), poly(sad(plane([0.0, 0.0, 0.30001], Z), (-1.2, 0.8, -0.5, 1.5)), grid(4, 2))),
    (poly(SURFACES['cyl'], grid(6, 4)), poly(sad(plane([1.0, 0.0, 0.0], X), (-2.0, 2.0, -2.0, 2.0)), grid(4, 4))),
    (poly(SURFACES['sphere_part'], AUTO), poly(SURFACES['sphere'], grid(7, 7))),
    (poly(SURFACES['bezier_s'], AUTO), poly(SURFACES['rev_bezier'], grid(6, 6))),
    # Nearly coplanar planes crossing within the tolerance: tangent zones.
    (poly(sad(plane([0.0, 0.0, 0.3], Z), (-1.0, 1.0, -1.0, 1.0)), grid(2, 2)),
     poly(sad(plane([0.05, 0.03, 0.3], [0.0, 3e-5, 1.0]), (-0.9, 1.1, -1.2, 0.8)), grid(3, 2))),
    (poly(sad(plane([0.0, 0.0, 0.3], Z), (-1.0, 1.0, -1.0, 1.0)), grid(4, 4)),
     poly(sad(plane([0.0, 0.0, 0.3], [2e-5, -1e-5, 1.0]), (-0.3, 0.3, -0.3, 0.3)), grid(1, 1))),
    (poly(sad(plane([0.0, 0.0, 0.3], Z), (-1.0, 1.0, -1.0, 1.0)), grid(1, 1)),
     poly(sad(plane([0.1, 0.0, 0.3], [4e-5, 0.0, 1.0]), (-2.0, 0.5, -0.4, 0.6)), grid(1, 2))),
]


def _bvh_requests(requests: list[str]) -> None:
    for a, b in PAIRS:
        requests.append(line_of('ip_bvh', [*a, *b]))
        requests.append(line_of('ip_interf', [0, *a, *b]))
    for a, b in PAIRS[1:4]:
        requests.append(line_of('ip_interf', [1, *a, *b]))
    for a, _ in PAIRS[:6]:
        requests.append(line_of('ip_interf', [2, *a]))
        requests.append(line_of('ip_interf', [3, *a]))


def points(pts):
    return [len(pts), *[v for p in pts for v in p]]


def _polyline_requests(requests: list[str]) -> None:
    straight = [(0.1 * i, 0.0, 0.0, 0.1 * i, 0.0, 0.2 * i, 1.0) for i in range(5)]
    circle = [(math.cos(t), math.sin(t), 0.0, math.cos(t), math.sin(t), 2 * math.cos(t), 0.5 * math.sin(t))
              for t in [0.1 * i for i in range(12)]]
    wiggle = [(0.0, 0.0, 0.0, 0.2 * i, 0.3 * math.sin(3 * i), 0.1 * i * i, -0.2 * i) for i in range(9)]
    for pts in (straight, circle, wiggle, straight[:2], straight[:1], []):
        for kind in (0, 1):
            for onfirst in (0, 1):
                for defle in (-1.0, 1e-3):
                    requests.append(line_of('ip_polyline', [kind, onfirst, defle, *points(pts)]))


def _polyarc_requests(requests: list[str]) -> None:
    boxes = [[0, -1.0, -1.0, 1.0, 1.0], [0, 0.2, 0.2, 0.3, 0.3], [0, 50.0, 50.0, 51.0, 51.0], [1],
             [0, -10.0, -10.0, 10.0, 10.0], [0, 0.9, -0.1, 1.1, 0.1]]
    for a, (u0, u1) in zip(ARCS[:12], [(0.0, 4.0), (0.0, TWO_PI), (0.5, 3.0), (0.0, TWO_PI), (-1.0, 1.0), (-2.0, 2.0),
                                       (0.0, 1.0), (0.0, 1.0), (0.0, 3.0), (0.0, 3.0), (0.0, 4.0), (0.0, 1.0)]):
        for b in boxes:
            for nb in (2, 7, 20):
                requests.append(line_of('ip_polyarc', [*a, nb, u0, u1, *b, 0.5, -0.25]))
    a = ARCS[1]
    for nb, u0, u1 in ((1, 0.0, 1.0), (0, 0.0, 1.0), (5, -1.7976931348623157e308, 1.0), (5, 0.0, 1.7976931348623157e308)):
        requests.append(line_of('ip_polyarc', [*a, nb, u0, u1, *boxes[0], 0.0, 0.0]))


def _t3bits_requests(requests: list[str]) -> None:
    scripts = [
        (4, [(1, 0), (1, 37), (2, 37), (4, 37), (4, 36), (5, 0), (4, 37), (5, 0), (6, 0)]),
        (4, [(1, 31), (2, 31), (4, 31), (5, 0), (1, 63), (2, 63), (5, 40), (5, 0)]),
        (8, [(1, 5), (1, 300), (2, 300), (2, 5), (5, 10), (5, 0), (5, 0), (3, 5), (4, 5)]),
        (10, [(1, t) for t in range(0, 999, 97)] + [(2, t) for t in range(0, 999, 194)] + [(5, 0)] * 8),
        (4, [(6, 0), (5, 0)]),
    ]
    for size, cmds in scripts:
        requests.append(line_of('ip_t3bits', [size, len(cmds), *[v for c in cmds for v in c]]))


def append_intpatch_poly_requests(requests: list[str]) -> None:
    """The polyhedron, BVH, interference and polygon requests."""
    _polyh_requests(requests)
    _bvh_requests(requests)
    _polyline_requests(requests)
    _polyarc_requests(requests)
    _t3bits_requests(requests)
