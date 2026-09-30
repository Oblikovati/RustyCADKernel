"""Deterministic requests for the BRepExtrema Ext wrappers
(tools/oracle/tktopalgo_probe_brepextrema_ext.cxx, docs/topalgo-design.md section 13):
ExtPC, ExtPF, ExtCC, ExtCF and ExtFF on segments, arcs, circles, C0 B-splines, Bezier
curves, infinite lines, polygon-only edges, bounded and natural planes, cylinder, sphere
and box faces, built by their constructors, initialized once and performed several times,
performed without Initialize, and re-initialized on other edges or faces.
"""
from __future__ import annotations
from tktopalgo_corpus_brepbndlib import numbers
from tktopalgo_corpus_brepextrema import X, Y, Z, named_shapes, sub
from tktopalgo_corpus_brepextrema_shapes import (arc_shape, infinite_line_shape, natural_plane_shape,
                                                 polygon_edge_shape, rectangle_shape, segment_shape)
from tktopalgo_corpus_solids import box, cylinder, sphere

POINTS = [[1.0, 1.0, 5.0, 0.0], [0.0, 0.0, 0.0, 0.0], [1.0, 2.5, 4.0, 1e-3], [-2.0, 0.5, 4.2, 0.0],
          [3.0, 3.0, -1.0, 0.0], [1.0, 1.0, 4.0, 0.0], [2.5, 1.0, 4.0, 0.0]]


def edges(degenerated: bool = True) -> list[tuple[object, list[int]]]:
    """Edges (record, selector) of the wrapper requests; the degenerated pole edge of a
    sphere is left out of the curve-curve requests (natively Extrema_ExtCC overflows an int
    on its zero length, `RealToInt(ratio) + 1`)."""
    shapes = named_shapes()
    pole = [(shapes['sphere'](), sub(2, 0))] if degenerated else []
    return pole + [(shapes['segment'](), sub(2, 0)), (shapes['arc'](), sub(2, 0)), (shapes['circle'](), sub(2, 0)),
            (shapes['c0spline'](), sub(2, 0)), (shapes['bezier'](), sub(2, 0)), (shapes['halfline'](), sub(2, 0)),
            (infinite_line_shape([0.0, 0.0, 4.0], [1.0, 0.0, 0.0]), sub(2, 0)),
            (polygon_edge_shape([[0.0, 0.0, 4.0], [1.0, 0.0, 4.0], [2.0, 1.0, 4.0]]), sub(2, 0)),
            (shapes['box'](), sub(2, 3)), (shapes['sphere'](), sub(2, 1)), (shapes['cylinder'](), sub(6, 2))]


def faces() -> list[tuple[object, list[int]]]:
    """Faces (record, selector) of the wrapper requests."""
    shapes = named_shapes()
    return [(shapes['rectangle'](), sub(3, 0)), (shapes['plane'](), sub(3, 0)), (shapes['cylface'](), sub(3, 0)),
            (shapes['sphface'](), sub(3, 0)), (shapes['box'](), sub(3, 5)), (shapes['box'](), sub(5, 1)),
            (rectangle_shape([0.0, 0.0, 6.0], Z, X, (2.0, 2.0)), sub(3, 0)),
            (natural_plane_shape([0.0, 0.0, 4.0], Z, X), sub(3, 0))]


def _pc_requests(requests: list[str]) -> None:
    points = [c for p in POINTS for c in p]
    for spec, selector in edges():
        for mode in range(4):
            requests.append('bxx_pc ' + numbers([*spec.numbers(), *selector, mode, len(POINTS), *points]))


def _pf_requests(requests: list[str]) -> None:
    points = [c for p in POINTS for c in p]
    for i, (spec, selector) in enumerate(faces()):
        for mode in range(4):
            flag, algo = (2, 0) if mode != 1 else (i % 3, i % 2)
            requests.append('bxx_pf ' + numbers([*spec.numbers(), *selector, flag, algo, mode, len(POINTS), *points]))
        for flag in range(3):
            for algo in range(2):
                requests.append('bxx_pf ' + numbers([*spec.numbers(), *selector, flag, algo, 0, 2, *POINTS[0],
                                                     *POINTS[3]]))


def _cc_requests(requests: list[str]) -> None:
    items = edges(degenerated=False)
    for i, (a, sa) in enumerate(items):
        for j, (b, sb) in enumerate(items):
            if (i * 3 + j) % 4 != 0:
                continue
            for mode in range(4):
                requests.append('bxx_cc ' + numbers([*a.numbers(), *sa, *b.numbers(), *sb, mode]))
    parallel = (segment_shape([0.0, 0.0, 0.0], [4.0, 0.0, 0.0]), segment_shape([1.0, 1.0, 0.0], [3.0, 1.0, 0.0]))
    concentric = (arc_shape([0.0, 0.0, 0.0], Z, X, 1.0), arc_shape([0.0, 0.0, 0.0], Z, X, 2.0, (0.5, 2.0)))
    for a, b in (parallel, concentric):
        for mode in range(4):
            requests.append('bxx_cc ' + numbers([*a.numbers(), *sub(2, 0), *b.numbers(), *sub(2, 0), mode]))


def _cf_requests(requests: list[str]) -> None:
    for i, (a, sa) in enumerate(edges()):
        for j, (b, sb) in enumerate(faces()):
            if (i + j) % 3 != 0:
                continue
            for mode in range(4):
                requests.append('bxx_cf ' + numbers([*a.numbers(), *sa, *b.numbers(), *sb, mode]))
    parallel = (segment_shape([0.0, 0.0, 1.0], [2.0, 0.0, 1.0]), rectangle_shape([0.0, -1.0, 0.0], Z, X, (3.0, 2.0)))
    box_edges = (box([0.0, 0.0, 3.0], [1.0, 1.0, 4.0]), rectangle_shape([0.0, 0.0, 0.0], Z, X, (2.0, 2.0)))
    for a, b in (parallel, box_edges):
        for mode in range(4):
            requests.append('bxx_cf ' + numbers([*a.numbers(), *sub(2, 0), *b.numbers(), *sub(3, 0), mode]))


def _ff_requests(requests: list[str]) -> None:
    items = faces()
    for i, (a, sa) in enumerate(items):
        for j, (b, sb) in enumerate(items):
            if (i + 2 * j) % 3 != 0 or (i in (2, 3) and j in (2, 3)):
                continue
            for mode in range(4):
                requests.append('bxx_ff ' + numbers([*a.numbers(), *sa, *b.numbers(), *sb, mode]))
    stacked = (box([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]), rectangle_shape([0.0, 0.0, 3.0], Z, X, (1.0, 1.0)))
    cyl_plane = (cylinder([0.0, 0.0, 0.0], 1.0, 2.0), rectangle_shape([-2.0, -2.0, 3.0], Z, X, (4.0, 4.0)))
    for a, b in (stacked, cyl_plane):
        for mode in range(4):
            requests.append('bxx_ff ' + numbers([*a.numbers(), *sub(3, 0), *b.numbers(), *sub(3, 0), mode]))
    ball = sphere([0.0, 0.0, 0.0], 1.0)
    requests.append('bxx_ff ' + numbers([*ball.numbers(), *sub(3, 0), *stacked[1].numbers(), *sub(3, 0), 0]))


def append_brepextrema_ext_requests(requests: list[str]) -> None:
    """Append the Ext wrapper request families."""
    _pc_requests(requests)
    _pf_requests(requests)
    _cc_requests(requests)
    _cf_requests(requests)
    _ff_requests(requests)
