"""Deterministic requests for BRepApprox (tools/oracle/tktopalgo_probe_brepapprox*.cxx,
docs/topalgo-design.md section 12).

Every request carries its face records (the intersecting face pairs of
tktopalgo_corpus_breptopadaptor_domain.py) and never relies on an earlier request. The
walking lines are those of IntPatch_Intersection on the two faces (with the
parametric-parametric walk, `geomint` 0, for the quadric pairs):
- `bra_approx`: BRepApprox_Approx of the first two walking lines with every setting of
  SetParameters (tolerances, degrees, iterations, points per interval, tangency and the
  three parametrizations), every choice of approximated spaces, on the two faces (the
  analytic-parametric and parametric-parametric SvSurfaces) and on the line alone, on all
  the points, on short sub-ranges (fewer than five points: one interpolating B-spline) and
  on the default range;
- `bra_line`: BRepApprox_ApproxLine of points and of the poles of three B-spline curves
  (some null), its points 0..n+1 and its approximation alone;
- `bra_mline`, `bra_sv`: the multi-line (values, tangents, MakeMLBetween,
  MakeMLOneMorePoint, Dump, BRepApprox_Approx::Parameters) and the PrmPrm and ImpPrm
  SvSurfaces over BRepAdaptor_Surface;
- `bra_zerimp`, `bra_int2s`: the ZerImpFunc and Int2S instances;
- `bra_tool`: every BRepApprox_SurfaceTool member on each face kind.
"""
from __future__ import annotations
import math
from tktopalgo_corpus_adaptor_faces import adaptor_faces
from tktopalgo_corpus_breptopadaptor_domain import intersection_pairs, numbers, quadric

# The pairs of intersection_pairs() used here: (index, geomint flags that give walking
# lines, SvSurfaces kinds valid in the first order: 1 PrmPrm, 2 the quadric of the first
# face, 3 the quadric of the second face). The ImpPrm SvSurfaces take the quadric of a
# plane, cylinder, sphere or cone only (another kind leaves the quadric undefined).
PAIRS = [(0, (0,), (1, 2, 3)), (1, (0,), (1, 2, 3)), (4, (0,), (1, 2, 3)), (5, (0,), (1, 2, 3)),
         (7, (0,), (1, 2, 3)), (8, (0,), (1, 2, 3)), (9, (0,), (1, 2, 3)), (10, (0,), (1, 2)),
         (11, (1, 0), (1, 3)), (12, (1,), (1, 3)), (13, (1,), (1, 3)), (14, (1, 0), (1, 3)),
         (16, (0,), (1, 2, 3)), (17, (0,), (1, 2, 3)), (18, (1, 0), (1, 2, 3)), (19, (1,), (1, 2, 3)),
         (20, (1, 0), (1,))]
SETTINGS = [
    [0],
    [1, 1.0e-7, 1.0e-7, 4, 8, 0, 30, 1, 0],
    [1, 1.0e-5, 1.0e-5, 2, 6, 5, 10, 1, 1],
    [1, 1.0e-3, 1.0e-4, 3, 5, 2, 5, 0, 2],
    [1, 1.0e-7, 1.0e-7, 4, 8, 0, 30, 0, 1],
    [1, 1.0e-6, 1.0e-6, 1, 3, 3, 4, 1, 0],
]
SPACES = [(1, 1, 1), (1, 0, 0), (0, 1, 0), (0, 0, 1), (1, 1, 0), (0, 1, 1)]
# Ranges `kind [a b]`: all the points, the defaults, and sub-ranges of 2 to 6 points.
RANGES = [[0], [2], [1, 1, 3], [1, 2, 4], [1, 1, 6], [1, 2, 7]]
SWAP_KIND = {1: 1, 2: 3, 3: 2}


def walks() -> list[tuple[list[float], tuple[int, ...]]]:
    """The walk records (both orders, each geomint flag) with their SvSurfaces kinds."""
    pairs = intersection_pairs()
    records = []
    for index, flags, kinds in PAIRS:
        first, second = pairs[index]
        for (a, b), order_kinds in (((first, second), kinds), ((second, first), tuple(SWAP_KIND[k] for k in kinds))):
            for g in flags:
                records.append(([*a.numbers(), 1, *b.numbers(), 1, 1e-7, 1e-7, g], order_kinds))
    return records


def _approx_requests(requests: list[str]) -> None:
    for walk, _ in walks():
        for settings in SETTINGS:
            for spaces in SPACES[:3]:
                requests.append('bra_approx ' + numbers([*walk, *settings, *spaces, 0, 2, 1, 0, 2, 0]))
        requests.append('bra_approx ' + numbers([*walk, *SETTINGS[1], 1, 1, 1, 1, 2, 1, 0, 2, 0]))
        for spaces in SPACES[3:]:
            requests.append('bra_approx ' + numbers([*walk, *SETTINGS[2], *spaces, 1, 1, 1, 0]))
        for rng in RANGES[1:]:
            for mode in (0, 1):
                requests.append('bra_approx ' + numbers([*walk, *SETTINGS[1], 1, 1, 1, mode, 1, 1, *rng]))
    # Restricted and unrestricted adaptors of a spline pair, and a line index past the last.
    pairs = intersection_pairs()
    for index in (11, 20):
        a, b = pairs[index]
        walk = [*a.numbers(), 0, *b.numbers(), 0, 1e-7, 1e-7, 1]
        requests.append('bra_approx ' + numbers([*walk, *SETTINGS[1], 1, 1, 1, 0, 3, 1, 0, 2, 0, 9, 0]))


def bspline(poles: list[list[float]], degree: int) -> list[float]:
    """A `tp_bra_bspline` (3d poles) or `tp_bra_bspline2d` (2d poles) record: present,
    degree, poles and uniform clamped knots."""
    inner = len(poles) - degree
    knots = [[i / inner, degree + 1 if i in (0, inner) else 1] for i in range(inner + 1)]
    return [1, degree, len(poles), *[c for p in poles for c in p], len(knots), *[x for k in knots for x in k]]


def _line_points(n: int) -> list[float]:
    """n points of a helix with parameters on two surfaces."""
    values: list[float] = []
    for i in range(n):
        t = 0.37 * i
        values += [math.cos(t), math.sin(t), 0.2 * t, t, 0.5 * t, 1.0 - t, 0.1 * t * t]
    return values


def _line_requests(requests: list[str]) -> None:
    helix = [[math.cos(0.4 * i), math.sin(0.4 * i), 0.15 * i] for i in range(9)]
    flat = [[0.3 * i, 0.1 * i * i] for i in range(9)]
    other = [[1.0 - 0.1 * i, 0.2 * math.sin(i)] for i in range(9)]
    curves = [[*bspline(helix, 3), *bspline(flat, 3), *bspline(other, 3)],
              [*bspline(helix, 3), 0, 0],
              [0, *bspline(flat, 2), 0],
              [0, 0, *bspline(other, 3)],
              [*bspline(helix[:4], 3), *bspline(flat[:4], 3), 0]]
    tails = [[*SETTINGS[0], 1, 1, 1, 0], [*SETTINGS[1], 1, 0, 0, 2], [*SETTINGS[3], 0, 1, 1, 1, 1, 3],
             [*SETTINGS[2], 1, 1, 1, 1, 2, 7]]
    for n in (0, 1, 2, 4, 7, 12, 40):
        for tang in (0, 1):
            for tail in tails:
                requests.append('bra_line ' + numbers([0, n, *_line_points(n), tang, *tail]))
    for record in curves:
        for tail in tails:
            requests.append('bra_line ' + numbers([1, *record, *tail]))


def _count_ops(ops: list[float]) -> int:
    i, n = 0, 0
    while i < len(ops):
        i += 2 if ops[i] == 6 else 5
        n += 1
    return n


def _mline_requests(requests: list[str]) -> None:
    offs0 = [0.0] * 7
    offs1 = [2.0, 1.0, -0.5, 0.25, 0.5, -1.0, 0.75]
    # nops, then MakeMLBetween (1 low high nb), MakeMLOneMorePoint (2 low high bad), Dump
    # (3), Parameters (4 kind).
    ops = [8, 1, 1, 6, 6, 1, 2, 8, 10, 2, 3, 6, 5, 2, 1, 4, 2, 3, 4, 0, 4, 1, 4, 2]
    for walk, kinds in walks()[::3]:
        for kind in (0, *kinds):
            for layout in ([1, 2, 1, 1, *offs0, 1], [1, 1, 1, 0, *offs1, 1], [0, 1, 0, 1, *offs0, 0],
                           [1, 0, 0, 0, *offs1, 1], [0, 2, 1, 1, *offs1, 0]):
                requests.append('bra_mline ' + numbers([*walk, kind, 1, *layout, 0, 3, 1, 2, 3, *ops]))
            requests.append('bra_mline ' + numbers([*walk, kind, 1, 1, 2, 1, 1, *offs0, 1, 1, 2, 9, 0, 2, 1, 2, 9, 4, 3]))
            requests.append('bra_mline ' + numbers([*walk, kind, 2, 1, 2, 1, 1, *offs0, 1, 2, 1, 0, 1, 1, 0, 0, 1]))


def _sv_ops(u: tuple[float, float, float, float], impprm: bool) -> list[float]:
    """Compute, Pnt, SeekPoint, the tangencies, SetUseSolver around them, and
    FillInitialVectorOfSolution for the ImpPrm SvSurfaces."""
    a, b, c, d = u
    ops = [0, a, b, c, d, 0, a, b, c, d, 1, a, b, c, d, 3, a + 0.01, b, c, d, 0, a, b, c, d, 4, a, b, c, d,
           5, a + 0.02, b, c, d, 2, a, b, c, d, 0, a, b, c, d, 6, 0, 0, a, b, c, d, 2, a, b, c, d, 6, 1,
           0, a + 0.3, b - 0.2, c, d]
    if impprm:
        ops += [7, *u, 7, a + 7.0, b, c - 7.0, d, 7, -20.0, b, 30.0, d]
    return ops


def _sv_requests(requests: list[str]) -> None:
    starts = [(0.5, 0.5, 0.5, 0.5), (1.0, 2.0, 0.3, 1.5), (0.1, 0.2, 2.0, 1.0)]
    for walk, kinds in walks()[::2]:
        for kind in kinds:
            for u in starts:
                ops = _sv_ops(u, kind != 1)
                requests.append('bra_sv ' + numbers([*walk, kind, _count_ops(ops), *ops]))


def _function_requests(requests: list[str]) -> None:
    faces = adaptor_faces()
    quadrics = [quadric(0, (0.0, 0.0, 0.5), (0.1, 0.2, 1.0), (1.0, 0.0, -0.1)),
                quadric(1, (1.0, 1.0, 0.0), (0.0, 0.0, 1.0), (1.0, 0.0, 0.0), 1.5),
                quadric(2, (1.0, 1.0, 0.0), (0.0, 0.0, 1.0), (1.0, 0.0, 0.0), 2.0)]
    for name in ('square', 'cylinder', 'sphere', 'cone', 'torus_patch', 'bezier_surface', 'bspline'):
        for q in quadrics:
            for tol, uv in ((1e-7, (0.5, 0.5)), (1e-3, (1.0, 2.0)), (1e-7, (0.0, 0.0))):
                requests.append('bra_zerimp ' + numbers([*faces[name].numbers(), 1, *q, tol, *uv]))
    pairs = intersection_pairs()
    for index, start in ((11, (1.0, 1.0, 1.0, 1.0)), (20, (1.5, 1.5, 0.5, 0.5)), (14, (1.5, 1.5, 0.1, 0.1)),
                         (5, (0.5, 2.5, 0.0, 0.0)), (19, (0.3, 0.2, 0.5, 1.0))):
        a, b = pairs[index]
        for mode in (0, 1):
            for iso in range(4):
                requests.append('bra_int2s ' + numbers([*a.numbers(), 1, *b.numbers(), 1, 1e-7, mode, iso, *start]))


def _tool_requests(requests: list[str]) -> None:
    faces = adaptor_faces()
    points = [0.0, 0.0, 0.5, 0.25, 1.0, 1.0, 2.5, -0.5]
    for name in sorted(faces):
        for restr in (1, 0):
            requests.append('bra_tool ' + numbers([*faces[name].numbers(), restr, 0.25, 0.75, 0.1, 0.9, 1e-7, 1e-3,
                                                   len(points) // 2, *points]))
            requests.append('bra_tool ' + numbers([*faces[name].numbers(), restr, 0.75, 0.25, -1.0, 3.0, 1e-3, 0.0,
                                                   1, 0.3, 0.7]))


def append_brepapprox_requests(requests: list[str]) -> None:
    """Append the BRepApprox request families."""
    _approx_requests(requests)
    _line_requests(requests)
    _mline_requests(requests)
    _sv_requests(requests)
    _function_requests(requests)
    _tool_requests(requests)
