"""Deterministic requests for GeomConvert (tools/oracle/tkgeombase_probe_geomconvert*.cxx).

Geometry uses the GeomTools builder specs of tkgeombase_corpus_geomspecs.py. The requests
convert every Geom curve and surface kind to B-splines (conics full and trimmed with
each parameterisation, lines, Bezier and B-spline curves rational or not, periodic,
trimmed and offset curves; every elementary, swept, Bezier, B-spline, trimmed and
offset surface), split B-spline curves and surfaces by knot indices and parameters in
order, reversed, coincident and out of range, cut them into knot-continuity pieces
and Bezier arcs and patches, compose curves that touch, touch reversed, gap or close,
compose Bezier patch grids, concatenate G1 and C1 chains and loops, raise C0 curves to
C1, and approximate curves and surfaces at several orders, degrees and segment counts.
The recognizers are in tkgeombase_corpus_geomconvert_ana.py.
"""
from __future__ import annotations
from tkgeombase_corpus_gce import add
from tkgeombase_corpus_geomspecs import (
    AX2, BEZIER, BEZIER_SURFACE, BSPLINE_SURFACE, C0, C0_BSPLINE_SURFACE, C2, CIRCLE, CIRCLE_TILTED,
    CONE, CYLINDER, ELLIPSE, EXT_BSPLINE, EXT_CYLINDER, FLAT_BEZIER_SURFACE, FLAT_BSPLINE_SURFACE,
    HYPERBOLA, LINE, PARABOLA, PERIODIC, PERIODIC_BSPLINE_SURFACE, PLANE, PLANE_TILTED, QUARTER,
    QUARTER_CYLINDER, RAT, RAT_BEZIER, RAT_BEZIER_SURFACE, RAT_BSPLINE_SURFACE, REV_BSPLINE, REV_CONE,
    REV_CYLINDER, REV_SPHERE, REV_TORUS, SPHERE, TORUS, bezier, bezier_surface, bspline, circle,
    cone, cylinder, line, offset, offset_surface, sphere, torus, trimmed, trimmed_surface,
)
import math

PI = math.pi

CURVES = [
    trimmed(-1.0, 2.0, LINE), CIRCLE, trimmed(0.3, 2.0, CIRCLE), trimmed(-1.0, 4.0, CIRCLE_TILTED),
    trimmed(5.0, 1.0, CIRCLE), ELLIPSE, trimmed(0.25, 3.5, ELLIPSE), trimmed(-2.0, 1.5, PARABOLA),
    trimmed(-1.0, 1.25, HYPERBOLA), BEZIER, RAT_BEZIER, C2, C0, RAT, PERIODIC, QUARTER,
    trimmed(0.5, 2.5, C2), trimmed(1.0, 5.0, PERIODIC), trimmed(-0.5, 7.5, PERIODIC),
    trimmed(0.1, 0.9, BEZIER), offset(0.5, CIRCLE), trimmed(0.0, 3.0, offset(-0.25, CIRCLE)),
    trimmed(0.5, 2.0, offset(0.3, bspline(2, [(0.0, 0.0, 0.0), (1.0, 1.0, 0.0), (2.0, 0.0, 0.0)],
                                           [(0.0, 3), (2.0, 3)]))),
    LINE, trimmed(0.0, 1.0, trimmed(-3.0, 3.0, LINE)),
]

SURFACES = [
    trimmed_surface(-1.0, 2.0, 0.5, 3.0, PLANE), trimmed_surface(0.0, 3.0, -1.0, 2.0, PLANE_TILTED),
    trimmed_surface(0.0, 2.0, -1.0, 1.5, CYLINDER), trimmed_surface(0.5, 5.0, 0.0, 2.0, CONE),
    SPHERE, trimmed_surface(0.0, 3.0, -1.0, 1.0, SPHERE), TORUS, trimmed_surface(1.0, 4.0, 0.0, 6.0, TORUS),
    trimmed_surface(0.0, 3.0, 0.0, 2.0, cylinder(1.5, [0.2, 0.1, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0, -1.0, 1])),
    trimmed_surface(0.0, 6.0, -1.0, 2.0, REV_CYLINDER), trimmed_surface(0.0, 3.0, 0.0, 2.0, REV_CONE),
    REV_TORUS, REV_SPHERE, trimmed_surface(0.5, 2.0, 0.5, 2.5, REV_BSPLINE),
    trimmed_surface(0.0, 6.0, 0.0, 2.0, EXT_CYLINDER), trimmed_surface(0.1, 0.9, -1.0, 1.0, EXT_BSPLINE),
    BEZIER_SURFACE, RAT_BEZIER_SURFACE, BSPLINE_SURFACE, C0_BSPLINE_SURFACE, RAT_BSPLINE_SURFACE,
    PERIODIC_BSPLINE_SURFACE, QUARTER_CYLINDER, trimmed_surface(0.2, 0.8, 0.1, 0.9, BSPLINE_SURFACE),
    trimmed_surface(0.0, 1.0, 0.0, 1.0, offset_surface(0.5, PLANE)),
    trimmed_surface(0.5, 2.0, -0.5, 0.5, offset_surface(-0.25, SPHERE)), offset_surface(0.2, BSPLINE_SURFACE),
    PLANE, CYLINDER,
]


def to_bspline_requests(requests):
    for p in range(8):
        for c in (CIRCLE, trimmed(0.3, 2.0, CIRCLE), trimmed(0.25, 3.5, ELLIPSE), ELLIPSE,
                  trimmed(0.2, 6.2, CIRCLE_TILTED)):
            add(requests, 'gcv_to_bspline', p, *c)
    for c in CURVES:
        add(requests, 'gcv_to_bspline', 0, *c)
        add(requests, 'gcv_to_bspline', 5, *c)


def surf_to_bspline_requests(requests):
    for s in SURFACES:
        add(requests, 'gcv_surf_to_bspline', *s)


SPLIT_CURVES = [C2, C0, RAT, PERIODIC, QUARTER]


def split_requests(requests):
    for c in SPLIT_CURVES:
        for a, b in ((1, 3), (3, 1), (2, 2), (0, 2), (1, 9), (1, 2)):
            for same in (1, 0):
                add(requests, 'gcv_split', 0, a, b, 1e-9, same, *c)
        for a, b in ((0.25, 1.75), (1.75, 0.25), (1.0, 1.0), (1.0, 1.0 + 1e-12), (-1.0, 2.0), (0.5, 7.0), (2.5, 5.5)):
            for same in (1, 0):
                add(requests, 'gcv_split', 1, a, b, 1e-9, same, *c)


def split_surface_requests(requests):
    for s in (BSPLINE_SURFACE, C0_BSPLINE_SURFACE, RAT_BSPLINE_SURFACE, PERIODIC_BSPLINE_SURFACE):
        for a, b, c, d in ((1, 3, 1, 2), (3, 1, 2, 1), (2, 2, 1, 3), (0, 2, 1, 2), (1, 9, 1, 2)):
            for su, sv in ((1, 1), (0, 1), (1, 0)):
                add(requests, 'gcv_split_surface', 0, a, b, c, d, 1e-9, 1, su, sv, *s)
        for a, b in ((1, 3), (3, 1), (1, 1), (0, 5)):
            for usplit in (1, 0):
                for same in (1, 0):
                    add(requests, 'gcv_split_surface', 1, a, b, 0, 0, 1e-9, usplit, same, 1, *s)
        for a, b, c, d in ((0.25, 1.5, 0.1, 0.9), (1.5, 0.25, 0.9, 0.1), (0.5, 0.5, 0.1, 0.9), (-1.0, 1.0, 0.0, 1.0)):
            for su, sv in ((1, 1), (0, 0)):
                add(requests, 'gcv_split_surface', 2, a, b, c, d, 1e-9, 1, su, sv, *s)
        for a, b in ((0.25, 0.75), (0.75, 0.25), (0.5, 0.5), (-0.5, 0.5)):
            for usplit in (1, 0):
                for same in (1, 0):
                    add(requests, 'gcv_split_surface', 3, a, b, 0, 0, 1e-9, usplit, same, 1, *s)


def knot_split_requests(requests):
    for c in SPLIT_CURVES:
        for rng in (-1, 0, 1, 2, 3):
            add(requests, 'gcv_knot_split', rng, *c)
    for s in (BSPLINE_SURFACE, C0_BSPLINE_SURFACE, RAT_BSPLINE_SURFACE, PERIODIC_BSPLINE_SURFACE):
        for ur, vr in ((-1, -1), (0, 0), (1, 1), (2, 1), (1, 2), (0, 3), (3, 0)):
            add(requests, 'gcv_knot_split_surface', ur, vr, *s)


def bezier_requests(requests):
    for c in SPLIT_CURVES:
        add(requests, 'gcv_bezier', 0, 0, 0, 1e-9, *c)
        for u1, u2 in ((0.5, 2.5), (1.0, 2.0), (1.0 + 1e-12, 2.0), (2.5, 0.5), (1.5, 1.5), (-1.0, 9.0)):
            add(requests, 'gcv_bezier', 1, u1, u2, 1e-9, *c)
    for s in (BSPLINE_SURFACE, C0_BSPLINE_SURFACE, RAT_BSPLINE_SURFACE, PERIODIC_BSPLINE_SURFACE):
        add(requests, 'gcv_bezier_surface', 0, 0, 0, 0, 0, 1e-9, *s)
        for u in ((0.5, 1.5), (1.0, 2.0), (1.5, 0.5), (1.0, 1.0)):
            for v in ((0.25, 0.75), (0.5, 1.0)):
                add(requests, 'gcv_bezier_surface', 1, *u, *v, 1e-9, *s)


def comp_piece(tol, after, ratio, min_m, spec):
    return [tol, after, ratio, min_m, *spec]


SEG_A = bspline(2, [(0.0, 0.0, 0.0), (1.0, 1.0, 0.0), (2.0, 0.0, 0.0)], [(0.0, 3), (1.0, 3)])
SEG_B = bspline(3, [(2.0, 0.0, 0.0), (3.0, -1.0, 0.0), (4.0, 1.0, 1.0), (5.0, 0.0, 0.0)], [(0.0, 4), (2.0, 4)])
SEG_C = bezier([(5.0, 0.0, 0.0), (6.0, 2.0, 0.0), (7.0, 0.0, 0.0)])
SEG_GAP = bezier([(5.1, 0.0, 0.0), (6.0, 2.0, 0.0), (7.0, 0.0, 0.0)])
ARC1 = trimmed(0.0, PI / 2, circle(1.0))
ARC2 = trimmed(PI / 2, PI, circle(1.0))
ARC3 = trimmed(PI, 2 * PI, circle(1.0))


def comp_requests(requests):
    chains = [
        [comp_piece(1e-7, 1, 0, 0, SEG_A), comp_piece(1e-7, 1, 0, 0, SEG_B), comp_piece(1e-7, 1, 0, 0, SEG_C)],
        [comp_piece(1e-7, 1, 1, 0, SEG_A), comp_piece(1e-7, 1, 1, 0, SEG_B), comp_piece(1e-7, 0, 1, 0, SEG_C)],
        [comp_piece(1e-7, 1, 1, 2, SEG_B), comp_piece(1e-7, 0, 1, 2, SEG_A), comp_piece(1e-7, 1, 1, 2, SEG_C)],
        [comp_piece(1e-7, 1, 0, 0, SEG_B), comp_piece(1e-7, 1, 0, 0, SEG_GAP), comp_piece(0.2, 1, 0, 0, SEG_GAP)],
        [comp_piece(1e-7, 1, 1, 0, ARC1), comp_piece(1e-7, 1, 1, 0, ARC2), comp_piece(1e-7, 1, 1, 0, ARC3)],
        [comp_piece(1e-7, 1, 0, 1, ARC2), comp_piece(1e-7, 0, 0, 1, ARC1), comp_piece(1e-7, 1, 0, 1, ARC3)],
        [comp_piece(1e-7, 1, 1, 0, trimmed(0.0, 2.0, LINE)), comp_piece(1e-7, 1, 1, 0, trimmed(2.0, 3.0, LINE))],
        [comp_piece(1e-7, 1, 1, 0, trimmed(2.0, 0.0, LINE, 1)), comp_piece(1e-7, 1, 1, 0, trimmed(0.0, 2.0, LINE))],
        [comp_piece(1e-7, 1, 1, 0, RAT), comp_piece(1e-7, 1, 1, 0, trimmed(4.0, 6.0, line((4.0, 0.0, 0.0), (1.0, 0.0, 0.0))))],
        [comp_piece(1e-7, 1, 1, 0, PERIODIC), comp_piece(1e-7, 1, 1, 0, C2)],
    ]
    for p in (0, 5, 7):
        for chain in chains:
            for with_first in (1, 0):
                add(requests, 'gcv_comp', p, with_first, len(chain), *[v for piece in chain for v in piece])


def patch(x0, y0, rational=False, lift=0.0):
    grid = [[(x0 + i, y0 + j, lift * (i * j - 1) if 0 < i < 2 and 0 < j < 2 else 0.0) for j in range(3)] for i in range(3)]
    if not rational:
        return bezier_surface(grid)
    return bezier_surface(grid, [[1.0, 1.0, 1.0], [1.0, 2.0, 1.0], [1.0, 1.0, 1.0]])


def comp_bezier_requests(requests):
    grids = [
        (1, 2, [patch(0, 0, lift=1.0), patch(0, 2, lift=0.5)]),
        (2, 2, [patch(0, 0), patch(0, 2, lift=1.0), patch(2, 0, lift=-1.0), patch(2, 2)]),
        (2, 1, [patch(0, 0, lift=0.25), bezier_surface([[(2.0, y, 0.0) for y in (0.0, 1.0, 2.0)],
                                                         [(3.0, y, 0.5) for y in (0.0, 1.0, 2.0)],
                                                         [(3.5, y, 1.0) for y in (0.0, 1.0, 2.0)],
                                                         [(4.0, y, 0.0) for y in (0.0, 1.0, 2.0)]])]),
        (1, 1, [patch(0, 0, rational=True)]),
    ]
    for nu, nv, patches in grids:
        uk = [float(i) for i in range(nu + 1)]
        vk = [0.5 * j for j in range(nv + 1)]
        for mode, tol, rk, uc, vc in ((0, 1e-4, 0, 0, 0), (1, 1e-4, 1, 0, 0), (1, 1e-4, 0, 0, 0),
                                      (2, 1e-4, 0, 0, 0), (2, 1e-4, 0, 2, 2), (2, 1e-4, 0, 1, 0)):
            add(requests, 'gcv_comp_bezier', mode, tol, rk, uc, vc, nu, nv, *uk, *vk, *[v for p in patches for v in p])


def concat_requests(requests):
    open_chain = [SEG_A, SEG_B, bspline(2, [(5.0, 0.0, 0.0), (6.0, 2.0, 0.0), (7.0, 0.0, 0.0)], [(0.0, 3), (1.0, 3)])]
    loop = [bspline(2, [(1.0, 0.0, 0.0), (1.0, 1.0, 0.0), (0.0, 1.0, 0.0)], [(0.0, 3), (1.0, 3)], [1.0, 0.7071067811865476, 1.0]),
            bspline(2, [(0.0, 1.0, 0.0), (-1.0, 1.0, 0.0), (-1.0, 0.0, 0.0)], [(0.0, 3), (2.0, 3)], [1.0, 0.7071067811865476, 1.0]),
            bspline(2, [(-1.0, 0.0, 0.0), (-1.0, -1.0, 0.0), (0.0, -1.0, 0.0)], [(0.0, 3), (1.0, 3)], [1.0, 0.7071067811865476, 1.0]),
            bspline(2, [(0.0, -1.0, 0.0), (1.0, -1.0, 0.0), (1.0, 0.0, 0.0)], [(0.0, 3), (0.5, 3)], [1.0, 0.7071067811865476, 1.0])]
    corner = [bspline(1, [(0.0, 0.0, 0.0), (1.0, 0.0, 0.0)], [(0.0, 2), (1.0, 2)]),
              bspline(1, [(1.0, 0.0, 0.0), (1.0, 1.0, 0.0)], [(0.0, 2), (1.0, 2)]),
              bspline(1, [(1.0, 1.0, 0.0), (2.0, 1.0, 0.0)], [(0.0, 2), (3.0, 2)])]
    g1 = [bspline(2, [(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (2.0, 1.0, 0.0)], [(0.0, 3), (1.0, 3)]),
          bspline(2, [(2.0, 1.0, 0.0), (4.0, 3.0, 0.0), (5.0, 3.0, 0.0)], [(0.0, 3), (1.0, 3)])]
    for kind in (0, 1, 2):
        for curves, closed in ((open_chain, 0), (loop, 1), (loop, 0), (corner, 0), (g1, 0), ([C2], 0), ([C0], 0)):
            tols = [1e-7] * (len(curves) - 1)
            add(requests, 'gcv_concat', kind, closed, 1e-7, 1e-7, len(tols), *tols, len(curves), *[v for c in curves for v in c])
        add(requests, 'gcv_concat', kind, 0, 1e-7, 1e-2, 1, 1e-7, 3, *[v for c in corner for v in c])


def c0c1_requests(requests):
    kinked = bspline(2, [(0.0, 0.0, 0.0), (1.0, 1.0, 0.0), (2.0, 0.0, 0.0), (3.0, 1.0, 0.0), (4.0, 0.0, 0.0)],
                     [(0.0, 3), (1.0, 2), (3.0, 3)])
    smooth_c0 = bspline(2, [(0.0, 0.0, 0.0), (1.0, 1.0, 0.0), (2.0, 2.0, 0.0), (3.0, 3.0, 0.0), (4.0, 3.5, 0.0)],
                        [(0.0, 3), (1.0, 2), (2.0, 3)])
    for c in (C0, kinked, smooth_c0, C2, RAT):
        for mode in (0, 1, 2):
            for tol, ang in ((1e-7, 1e-7), (1e-2, 1e-2), (1e-7, 0.5)):
                add(requests, 'gcv_c0c1', mode, tol, ang, *c)


def approx_requests(requests):
    for c in (trimmed(0.3, 2.0, CIRCLE), trimmed(0.25, 3.5, ELLIPSE), BEZIER, C2, C0, offset(0.5, CIRCLE),
              trimmed(0.5, 2.0, offset(0.2, C2, [0.0, 0.0, 1.0])), trimmed(-1.0, 1.0, HYPERBOLA)):
        for via, ranged, tol, order, seg, deg in ((0, 0, 1e-4, 2, 10, 8), (1, 0, 1e-6, 4, 20, 14), (1, 1, 1e-3, 0, 3, 5),
                                                    (0, 0, 1e-9, 2, 1, 3), (0, 0, 1e-12, 4, 50, 25), (0, 0, 1e-4, 6, 10, 8)):
            add(requests, 'gcv_approx_curve', via, ranged, 0.5, 1.5, tol, order, seg, deg, *c)
    for s in (trimmed_surface(0.0, 2.0, -1.0, 1.5, CYLINDER), trimmed_surface(0.5, 2.0, 0.0, 1.0, TORUS),
              trimmed_surface(0.0, 1.0, 0.0, 1.0, offset_surface(0.3, BSPLINE_SURFACE)), BEZIER_SURFACE,
              trimmed_surface(0.0, 3.0, 0.0, 1.0, REV_CONE), trimmed_surface(0.1, 0.9, -1.0, 1.0, EXT_BSPLINE)):
        for via, tol, uc, vc, du, dv, seg, precis in ((0, 1e-4, 2, 2, 8, 8, 20, 0), (1, 1e-5, 4, 2, 9, 9, 50, 1),
                                                       (0, 1e-3, 0, 0, 3, 3, 4, 2), (1, 1e-7, 4, 4, 14, 14, 100, 1),
                                                       (0, 1e-4, 6, 2, 8, 8, 20, 1)):
            add(requests, 'gcv_approx_surface', via, tol, uc, vc, du, dv, seg, precis, *s)


def append_geomconvert_requests(requests: list[str]) -> None:
    """Append every GeomConvert request family in a fixed order."""
    to_bspline_requests(requests)
    surf_to_bspline_requests(requests)
    split_requests(requests)
    split_surface_requests(requests)
    knot_split_requests(requests)
    bezier_requests(requests)
    comp_requests(requests)
    comp_bezier_requests(requests)
    concat_requests(requests)
    c0c1_requests(requests)
    approx_requests(requests)
    __import__('tkgeombase_corpus_geomconvert_ana').append_geomconvert_ana_requests(requests)
