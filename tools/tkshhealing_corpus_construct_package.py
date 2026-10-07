"""Requests for the ShapeConstruct package functions (tools/oracle/tkshhealing_probe_construct_package.cxx).

`sh_construct_pkg 0/1` converts every 3D/2D curve kind (B-splines returned as is, conics
with the degree cap at 6, offsets and trimmed curves, empty and reversed ranges that raise
in the trimmed-curve constructor) at several tolerances, continuities, segment counts and
degrees. `2` converts elementary surfaces (exact path), extrusions of lines, conics and
splines, revolutions of plain and offset curves (the offset basis converted first), Bezier
and B-spline surfaces and trimmed wrappers, at tolerances that pass and fail the
approximation. `4`/`5` join two 3D/2D curves (lines, circles, Bezier and B-spline curves,
trimmed circles) over adjacent, swapped and gapped halves of their ranges in all four
orientation pairs, the 2D ones with and without the crossing guard. `3` joins p-curve chains: on a plane (the early return), on a cylinder with
collinear, reversed and kinked segments, with a seam pair (both seam p-curves joined), a
chain mixing a seam with a non-seam edge and a chain with an edge without p-curve.
`sc_surface` (tools/oracle/tkshhealing_probe_custom_surface.cxx) runs ShapeCustom_Surface
ConvertToAnalytical and ConvertToPeriodic sequences on exact B-spline forms of analytic
surfaces, flat and twisted bilinear patches, clamped rings closed in U or V and a plane.
"""
from __future__ import annotations
import math
from itertools import product

from tktopalgo_corpus_breplib_records import (
    bezier_surface,
    bspline_surface,
    cone,
    cylinder,
    geom2d_bezier,
    geom2d_bspline,
    geom2d_gp,
    geom2d_trimmed,
    geom_bezier,
    geom_bspline,
    geom_gp,
    geom_offset,
    geom_trimmed,
    numbers,
    plane,
    sphere,
    torus,
)
from tktopalgo_corpus_faces import CYLINDER, FORWARD, LINE, NO_PCURVE, REVERSED, SEAM, SEGMENT, FaceSpec

POLES3 = [(0.0, 0.0, 0.0), (1.0, 2.0, 0.5), (3.0, 0.0, 1.0), (4.0, 1.0, 0.0)]
POLES2 = [(0.0, 0.0), (1.0, 2.0), (3.0, 0.0), (4.0, 1.0)]
C0, G1, C1, G2, C2, C3, CN = range(7)


def curves3d() -> list[list[float]]:
    spline = geom_bspline(3, False, POLES3, [(0.0, 4), (1.0, 4)])
    return [
        geom_gp(0), geom_gp(1, radius=2.0), geom_gp(2), geom_gp(3), geom_gp(4),
        geom_bezier(POLES3), spline,
        geom_trimmed(0.2, 2.5, geom_gp(1)),
        geom_offset(0.3, (0.0, 0.0, 1.0), geom_gp(1)),
        geom_offset(0.1, (0.0, 0.0, 1.0), spline),
    ]


def curves2d() -> list[list[float]]:
    spline = geom2d_bspline(3, False, POLES2, [(0.0, 4), (1.0, 4)])
    return [
        geom2d_gp(0), geom2d_gp(1, radius=2.0), geom2d_gp(2), geom2d_gp(3), geom2d_gp(4),
        geom2d_bezier(POLES2), spline, geom2d_trimmed(0.2, 2.5, geom2d_gp(1)),
    ]


def curve_requests() -> list[str]:
    requests: list[str] = []
    ranges = [(0.0, 1.0), (0.25, 2.5), (-1.0, 1.0), (1.0, 1.0), (1.0, 0.5)]
    settings = [(1e-4, C1, 9, 9), (1e-7, C2, 20, 14), (1e-2, C0, 1, 3), (1e-6, CN, 30, 8)]
    for op, curves in [(0, curves3d()), (1, curves2d())]:
        for curve, (first, last), (tol, cont, seg, deg) in product(curves, ranges, settings):
            requests.append('sh_construct_pkg ' + numbers([op, *curve, first, last, tol, cont, seg, deg]))
    return requests


def surfaces() -> list[tuple[list[float], tuple[float, float, float, float]]]:
    grid = [[(float(i), float(j), 0.2 * i * j) for j in range(3)] for i in range(3)]
    spline = bspline_surface(2, 2, grid, [(0.0, 3), (1.0, 3)], [(0.0, 3), (1.0, 3)])
    spline3d = geom_bspline(3, False, POLES3, [(0.0, 4), (1.0, 4)])
    line = geom_gp(0, p=(1.0, 0.0, 0.0), d=(0.0, 0.0, 1.0))
    return [
        ([0, *plane()], (0.0, 2.0, -1.0, 3.0)),
        ([0, *cylinder(radius=2.0)], (0.0, math.pi, 0.0, 4.0)),
        ([0, *sphere()], (0.0, 2.0, -1.0, 1.0)),
        ([0, *cone()], (0.0, 3.0, 0.0, 2.0)),
        ([0, *torus()], (0.0, 1.0, 0.0, 6.0)),
        ([0, *bezier_surface(grid)], (0.0, 1.0, 0.0, 1.0)),
        ([0, *spline], (0.0, 1.0, 0.0, 1.0)),
        ([1, *geom_gp(1, radius=2.0), 0.0, 0.0, 1.0], (0.0, 2.0, -1.0, 2.0)),
        ([1, *spline3d, 0.0, 1.0, 1.0], (0.0, 1.0, 0.0, 3.0)),
        ([1, *geom_gp(0), 0.0, 1.0, 0.0], (-1.0, 2.0, 0.0, 1.0)),
        ([2, *line, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0], (0.0, 2.0, 0.0, 3.0)),
        ([2, *geom_bezier([(1.0, 0.0, 0.0), (2.0, 0.0, 1.0), (1.5, 0.0, 2.0)]), 0.0, 0.0, 0.0, 0.0, 0.0, 1.0],
         (0.0, 6.0, 0.0, 1.0)),
        ([2, *geom_offset(0.2, (0.0, 1.0, 0.0), geom_trimmed(0.0, 2.0, line)), 0.0, 0.0, 0.0, 0.0, 0.0, 1.0],
         (0.0, 3.0, 0.0, 2.0)),
        ([3, 0, *cylinder(), 0.0, 3.0, 0.0, 2.0], (0.5, 2.5, 0.5, 1.5)),
        ([3, 1, *geom_gp(1), 0.0, 0.0, 1.0, 0.0, 6.0, 0.0, 2.0], (1.0, 5.0, 0.5, 1.5)),
    ]


def surface_requests() -> list[str]:
    requests: list[str] = []
    settings = [(1e-4, C2, 9, 9), (1e-7, C3, 30, 14), (1e-1, C1, 1, 3), (1e-3, CN, 16, 6)]
    for (surface, box), (tol, cont, seg, deg) in product(surfaces(), settings):
        requests.append('sh_construct_pkg ' + numbers([2, *surface, *box, tol, cont, seg, deg]))
    return requests


def cylinder_chain_face(kink: float, seam_chain: bool) -> FaceSpec:
    """A cylinder face with a chain of three segments along v = 1 (or a seam pair)."""
    face = FaceSpec([CYLINDER, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 2.0])
    points = [(0.0, 1.0), (1.0, 1.0), (2.0, 1.0 + kink), (3.0, 1.0)]
    vertices = [face.vertex(u, v) for u, v in points]
    for i in range(3):
        face.edge(SEGMENT, [*points[i], *points[i + 1]], vertices[i], vertices[i + 1])
    two_pi = 2.0 * math.pi
    top = face.vertex(0.0, 3.0)
    face.edge(SEAM, [two_pi, 1.0, two_pi, 2.0, -two_pi, 0.0], vertices[0], top)
    face.edge(SEAM, [two_pi, 2.0, two_pi, 3.0, -two_pi, 0.0], top, top)
    face.edge(NO_PCURVE, [], vertices[3], vertices[0])
    face.edge(LINE, [0.0, 4.0, 1.0, 0.0, 0.0, 2.0], -1, -1)
    if seam_chain:
        face.wire([(3, FORWARD), (4, FORWARD)])
    else:
        face.wire([(0, FORWARD), (1, FORWARD), (2, FORWARD)])
    return face


def join_requests() -> list[str]:
    requests: list[str] = []
    plane_face = FaceSpec([0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0])
    a, b = plane_face.vertex(0.0, 0.0), plane_face.vertex(1.0, 0.0)
    plane_face.edge(SEGMENT, [0.0, 0.0, 1.0, 0.0], a, b)
    requests.append('sh_construct_pkg ' + numbers([3, *plane_face.numbers(), 1, 0, FORWARD]))
    chains = [
        [(0, FORWARD)], [(0, FORWARD), (1, FORWARD)], [(0, FORWARD), (1, FORWARD), (2, FORWARD)],
        [(2, REVERSED), (1, REVERSED), (0, REVERSED)], [(1, FORWARD), (0, FORWARD)],
        [(3, FORWARD), (4, FORWARD)], [(3, REVERSED), (4, REVERSED)], [(3, FORWARD), (0, FORWARD)],
        [(0, FORWARD), (3, FORWARD)], [(0, FORWARD), (5, FORWARD)], [(0, FORWARD), (6, FORWARD)],
        [(1, FORWARD), (0, REVERSED)],
    ]
    for kink, seam_chain, chain in product([0.0, 0.25], [False, True], chains):
        face = cylinder_chain_face(kink, seam_chain)
        fields = [3, *face.numbers(), len(chain)] + [c for pair in chain for c in pair]
        requests.append('sh_construct_pkg ' + numbers(fields))
    return requests


def join_curve_requests() -> list[str]:
    """ShapeConstruct::JoinCurves on two pieces of one curve."""
    spline3 = geom_bspline(3, False, POLES3, [(0.0, 4), (1.0, 4)])
    spline2 = geom2d_bspline(3, False, POLES2, [(0.0, 4), (1.0, 4)])
    curves = [
        (4, [geom_gp(0), geom_gp(1, radius=2.0), geom_bezier(POLES3), spline3,
             geom_trimmed(0.2, 2.5, geom_gp(1))], [[]]),
        (5, [geom2d_gp(0), geom2d_gp(1, radius=2.0), geom2d_bezier(POLES2), spline2,
             geom2d_trimmed(0.2, 2.5, geom2d_gp(1))], [[0], [1]]),
    ]
    ranges = [((0.0, 0.5), (0.5, 1.0)), ((0.5, 1.0), (0.0, 0.5)), ((0.0, 0.4), (0.6, 1.0)),
              ((0.25, 0.5), (0.5, 0.75))]
    requests = []
    for op, kinds, tails in curves:
        for curve, (r1, r2), o1, o2, tail in product(kinds, ranges, (FORWARD, REVERSED), (FORWARD, REVERSED), tails):
            fields = [op, *curve, *curve, o1, o2, *r1, *r2, *tail]
            requests.append('sh_construct_pkg ' + numbers(fields))
    return requests


def ring_bspline(closed_v: bool) -> list[float]:
    """A clamped quadratic B-spline whose first and last U rows coincide (V when asked)."""
    ring = [(2.0, 0.0), (2.0, 2.0), (-2.0, 2.0), (-2.0, -2.0), (2.0, -2.0), (2.0, 0.0)]
    rows = [[(x, y, z) for z in (0.0, 1.0, 2.0)] for x, y in ring]
    uknots = [(0.0, 3), (1.0, 1), (2.0, 1), (3.0, 1), (4.0, 3)]
    vknots = [(0.0, 3), (1.0, 3)]
    if closed_v:
        rows = [list(column) for column in zip(*rows)]
        return [0, *bspline_surface(2, 2, rows, vknots, uknots)]
    return [0, *bspline_surface(2, 2, rows, uknots, vknots)]


def custom_surface_requests() -> list[str]:
    """ShapeCustom_Surface on analytic B-splines, bilinear patches and closed rings."""
    two_pi = 2.0 * math.pi
    flat = [[(0.0, 0.0, 0.0), (0.0, 1.0, 0.0)], [(1.0, 0.0, 0.0), (1.0, 1.0, 0.0)]]
    twisted = [[(0.0, 0.0, 0.0), (0.0, 1.0, 0.0)], [(1.0, 0.0, 0.0), (1.0, 1.0, 0.3)]]
    surfaces = [
        [4, 0, *cylinder(radius=2.0), 0.0, two_pi, 0.0, 3.0],
        [4, 0, *cylinder(radius=2.0), 0.0, math.pi, 0.0, 3.0],
        [4, 0, *sphere(radius=1.5), 0.0, two_pi, -1.5, 1.5],
        [4, 0, *cone(), 0.0, two_pi, 0.0, 2.0],
        [4, 0, *torus(), 0.0, two_pi, 0.0, two_pi],
        [0, *bezier_surface(flat)], [0, *bezier_surface(twisted)],
        ring_bspline(False), ring_bspline(True), [0, *plane()],
    ]
    op_lists = [
        [0, 1e-6, 0], [0, 1e-2, 1, 0, 1e-6, 0], [0, 1e-9, 1, 1, 1, -1.0],
        [1, 0, -1.0], [1, 1, 1e-3, 0, 1e-6, 0], [1, 1, -1.0, 1, 0, -1.0],
    ]
    requests = []
    for surface, ops in product(surfaces, op_lists):
        requests.append('sc_surface ' + numbers([*surface, count_ops(ops), *ops]))
    return requests


def count_ops(ops: list[float]) -> int:
    """The number of ops in an op list (each op has two fields)."""
    return len(ops) // 3


def append_construct_package_requests(requests: list[str]) -> None:
    """Append the ShapeConstruct package requests."""
    requests.extend(curve_requests())
    requests.extend(surface_requests())
    requests.extend(join_requests())
    requests.extend(join_curve_requests())
    requests.extend(custom_surface_requests())
