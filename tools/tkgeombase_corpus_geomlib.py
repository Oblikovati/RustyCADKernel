"""Deterministic requests for GeomLib (tools/oracle/tkgeombase_probe_geomlib*.cxx).

Geometry uses the GeomTools builder specs of tkgeombase_corpus_geomspecs.py. The requests
place every Geom2d curve kind in 3D and transform them by general affinities, reparametrize
curves on equal, shifted, reversed and periodic ranges, build 3D curves of pcurves on
planes, cylinders, spheres, tori and B-spline surfaces, adjust and extend curves (every
continuity, both ends) and surfaces (both directions and ends), compute inertia axes of
planar, collinear, coincident and spread clouds, thin, densify and fuse parameter arrays
(unordered, constant, empty, arbitrary lower bounds), measure curve distances, cancel
denominator derivatives, estimate normals at regular and singular points, test closedness
of elementary, periodic and closed B-spline and Bezier surfaces, detect and build
isolines, fix B-spline tangents, check curves on surfaces (coincident, deviating, periodic,
parallel), evaluate denominator multipliers, interpolate (including every error), test
planarity, sample logarithmically, evaluate polynomials, project points and measure chord
deviations, and rebuild curves from AdvApprox approximations of every space kind.
"""
from __future__ import annotations
import math
from tkgeombase_corpus_gce import add
from tkgeombase_corpus_geomspecs import (
    AX2, AX22D_INDIRECT, AX22D_TILTED, BEZIER, BEZIER_SURFACE, BSPLINE_SURFACE, C0, C2, CIRCLE, CONE,
    CYLINDER, FLAT_BEZIER_SURFACE, FLAT_BSPLINE_SURFACE, LINE, PERIODIC, PERIODIC_BSPLINE_SURFACE, PLANE,
    QUARTER, QUARTER_CYLINDER, RAT, RAT_BEZIER, RAT_BEZIER_SURFACE, RAT_BSPLINE_SURFACE, REV_SPHERE, SPHERE,
    TILTED, TORUS, bezier, bezier2d, bezier_surface, bspline, bspline2d, circle, circle2d, ellipse2d,
    hyperbola2d, line, line2d, offset2d, parabola2d, trimmed, trimmed2d, trimmed_surface,
)

PI = math.pi
BSPLINE2D = bspline2d(3, [(0.0, 0.0), (1.0, 2.0), (2.0, -1.0), (3.0, 1.0), (4.0, 0.0)], [(0.0, 4), (1.0, 1), (2.0, 4)])
RAT2D = bspline2d(2, [(1.0, 0.0), (1.0, 1.0), (0.0, 1.0)], [(0.0, 3), (1.0, 3)], [1.0, math.sqrt(0.5), 1.0])
PER2D = bspline2d(3, [(2.0 * math.cos(k * PI / 3), 2.0 * math.sin(k * PI / 3)) for k in range(6)],
                  [(float(k), 1) for k in range(7)], periodic=1)
BEZ2D = bezier2d([(0.0, 0.0), (1.0, 3.0), (2.0, -2.0), (4.0, 0.0)])
CURVES2D = [line2d((1.0, 2.0), (0.6, 0.8)), circle2d(1.5, AX22D_TILTED), circle2d(0.5, AX22D_INDIRECT),
            ellipse2d(3.0, 1.0, AX22D_TILTED), parabola2d(0.25), hyperbola2d(2.0, 1.0), BEZ2D,
            bezier2d([(0.0, 0.0), (1.0, 2.0), (3.0, 0.0)], [1.0, 2.0, 0.5]), BSPLINE2D, RAT2D, PER2D,
            trimmed2d(0.5, 2.0, circle2d(1.0)), trimmed2d(-1.0, 1.0, line2d()), offset2d(0.25, circle2d(1.0)),
            trimmed2d(0.2, 0.8, offset2d(-0.1, BSPLINE2D))]


def to3d_requests(requests):
    for ax in (AX2, TILTED):
        for c in CURVES2D:
            add(requests, 'gl_to3d', *ax, *c)
    for a in ((1.0, 0.0, 0.0, 1.0), (2.0, 0.0, 0.0, 0.5), (1.0, 0.5, 0.0, 1.0), (0.0, 1.0, -1.0, 0.0), (1.0, 2.0, 2.0, 4.0),
              (2.0, 0.0, 0.0, 2.0), (-1.0, 0.0, 0.0, 1.0)):
        for c in CURVES2D:
            add(requests, 'gl_gtransform', *a, 0.5, -1.0, *c)


def same_range_requests(requests):
    for c in (line2d(), circle2d(1.0), BSPLINE2D, PER2D, trimmed2d(0.5, 2.0, circle2d(1.0)), BEZ2D, RAT2D,
              offset2d(0.25, circle2d(1.0))):
        for f, l, rf, rl in ((0.0, 1.0, 0.0, 1.0), (0.0, 1.0, 2.0, 5.0), (0.5, 1.5, -1.0, 1.0), (0.0, 2.0, 2.0, 0.0),
                             (0.0, 1.0, 0.0, 1.0 + 1e-12), (1.0, 1.0, 0.0, 3.0), (0.0, 6.0, 0.0, 2 * PI)):
            for tol in (1e-9, 1e-3):
                add(requests, 'gl_same_range', tol, f, l, rf, rl, *c)


def build_curve3d_requests(requests):
    cases = [
        (line2d((0.0, 1.0), (1.0, 0.0)), CYLINDER, 0.0, 2.0), (line2d((0.5, 0.0), (0.0, 1.0)), CYLINDER, -1.0, 2.0),
        (line2d((0.0, 0.0), (0.6, 0.8)), CYLINDER, 0.0, 3.0), (circle2d(1.0), PLANE, 0.0, 2 * PI),
        (line2d((0.0, 0.2), (1.0, 0.0)), SPHERE, 0.0, 2.0), (line2d((0.3, 0.0), (0.0, 1.0)), TORUS, 0.0, 2 * PI),
        (line2d((0.2, 0.2), (0.6, 0.8)), BSPLINE_SURFACE, 0.0, 0.8), (BSPLINE2D, PLANE, 0.0, 2.0),
        (bspline2d(2, [(0.1, 0.1), (0.5, 0.9), (0.9, 0.2)], [(0.0, 3), (1.0, 3)]), BSPLINE_SURFACE, 0.0, 1.0),
        (line2d((1.0, 0.0), (0.0, 1.0)), CONE, 0.0, 2.0), (line2d((0.0, 0.5), (1.0, 0.0)), REV_SPHERE, 0.0, 3.0),
    ]
    for c, s, f, l in cases:
        for tol, cont, deg, seg in ((1e-5, 2, 14, 30), (1e-7, 4, 10, 10), (1e-3, 0, 3, 2)):
            add(requests, 'gl_build_curve3d', tol, f, l, cont, deg, seg, *c, *s)


def adjust_extend_requests(requests):
    curves = [C2, C0, RAT, BEZIER, RAT_BEZIER, QUARTER, trimmed(0.5, 2.0, CIRCLE), trimmed(0.0, 2.0, LINE), PERIODIC]
    for c in curves:
        add(requests, 'gl_adjust', 0.0, 0.0, 0.5, 5.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, -1.0, 0.0, *c)
        add(requests, 'gl_adjust', -1.0, 0.0, 0.0, 6.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, *c)
        for cont in (0, 1, 2, 3):
            for after in (1, 0):
                add(requests, 'gl_extend_curve', cont, after, 7.0, 1.0, 0.5, *c)
        add(requests, 'gl_extend_curve', 2, 1, 5.0, 1.0, -1.0, *c)
    surfaces = [BSPLINE_SURFACE, RAT_BSPLINE_SURFACE, BEZIER_SURFACE, RAT_BEZIER_SURFACE, QUARTER_CYLINDER,
                PERIODIC_BSPLINE_SURFACE, trimmed_surface(0.0, 1.0, 0.0, 2.0, PLANE),
                trimmed_surface(0.0, 2.0, -1.0, 1.0, CYLINDER)]
    for s in surfaces:
        for cont in (0, 1, 2, 3):
            for in_u in (1, 0):
                for after in (1, 0):
                    add(requests, 'gl_extend_surf', 0.5, cont, in_u, after, *s)
        add(requests, 'gl_extend_surf', 0.0, 1, 1, 1, *s)
        add(requests, 'gl_extend_surf', 25.0, 2, 0, 1, *s)


def inertia_requests(requests):
    clouds = [
        [(1.0, 2.0, 3.0)], [(0.0, 0.0, 0.0), (1.0, 1.0, 1.0)], [(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (2.0, 0.0, 0.0)],
        [(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (0.0, 1.0, 0.0), (1.0, 1.0, 0.0)],
        [(math.cos(0.4 * i) * 3.0, math.sin(0.4 * i), 0.1 * i) for i in range(12)],
        [(math.cos(0.9 * i) * (1.0 + 0.1 * i), math.sin(1.3 * i), math.cos(2.1 * i)) for i in range(20)],
        [(2.0, 2.0, 2.0)] * 4, [(0.0, 0.0, 0.0), (1.0, 1.0, 0.0), (1.0, -1.0, 0.0), (-1.0, 1.0, 0.0), (-1.0, -1.0, 0.0)],
    ]
    for pts in clouds:
        for tol in (1e-7, 1.0):
            add(requests, 'gl_inertia', tol, len(pts), *[c for p in pts for c in p])


def array(lower, values):
    return [lower, len(values), *values]


def arrays_requests(requests):
    arrays = [array(1, [0.0, 0.1, 0.2, 0.4, 0.6, 0.8, 1.0]), array(-2, [0.0, 0.5, 1.0]), array(3, [1.0, 1.0, 1.0]),
              array(1, [0.0, 2.0, 1.0, 3.0]), array(0, [5.0]), array(1, [0.0, 1e-12, 1.0])]
    # An empty input is left out: the native members read its first value unchecked
    # (a crash), where the port raises Standard_OutOfRange.
    for a in arrays:
        for count in (0, 2, 3, 4, 9):
            add(requests, 'gl_arrays', 0, count, *a)
            add(requests, 'gl_arrays', 1, count, *a)
    for a, b in ((array(-1, [0.0, 0.5, 1.0]), array(3, [0.25, 0.5000000001, 0.75])), (array(1, [0.0, 1.0]), array(1, [])),
                 (array(1, [0.0, 0.3, 0.3, 1.0]), array(1, [0.3, 1.0, 2.0])), (array(1, []), array(1, [1.0, 2.0]))):
        for conf, adjust in ((1e-8, 0), (1e-8, 1), (0.2, 0), (0.2, 1)):
            add(requests, 'gl_arrays', 2, 0, *a, *b, conf, adjust)
    params = array(1, [0.0, 0.25, 0.5, 0.75, 1.0])
    pairs = [(C2, C2), (C2, trimmed(0.0, 3.0, C2)), (BEZIER, bezier([(0.0, 0.0, 0.0), (1.0, 2.1, 0.0), (3.0, 2.0, 1.0),
                                                                      (4.0, 0.0, -0.5)])), (CIRCLE, circle(2.6))]
    for a, b in pairs:
        for mode in (0, 1):
            for tol in (1e-7, 0.05):
                add(requests, 'gl_eval_max', mode, tol, *params, *a, *b)
    add(requests, 'gl_eval_max', 0, 1e-7, *array(1, [0.5, 9.0]), *C2, *C2)


def surface_query_requests(requests):
    for ud, vd in ((1, 1), (1, 0), (0, 1), (0, 0)):
        for s in (RAT_BSPLINE_SURFACE, BSPLINE_SURFACE):
            add(requests, 'gl_cancel_denom', ud, vd, *s)
    for s, uvs in ((PLANE, [(0.5, 0.5)]), (SPHERE, [(0.0, 0.0), (1.0, PI / 2), (0.3, -PI / 2)]),
                   (CONE, [(0.0, -2.0), (1.0, 1.0)]), (BSPLINE_SURFACE, [(0.5, 0.5), (0.0, 0.0), (2.0, 1.0)]),
                   (bezier_surface([[(0.0, 0.0, 0.0)] * 3, [(1.0, -1.0, 0.0), (1.0, 0.0, 1.0), (1.0, 1.0, 0.0)],
                                    [(2.0, -1.0, 0.0), (2.0, 0.0, 0.5), (2.0, 1.0, 0.0)]]), [(0.0, 0.5), (0.5, 0.5)]),
                   (TORUS, [(0.1, 0.2)])):
        for u, v in uvs:
            for tol in (1e-7, 1e-2):
                add(requests, 'gl_norm_estim', u, v, tol, *s)
    closed_bs = [PERIODIC_BSPLINE_SURFACE, BSPLINE_SURFACE, FLAT_BSPLINE_SURFACE, QUARTER_CYLINDER]
    for s in (PLANE, CYLINDER, SPHERE, TORUS, trimmed_surface(0.0, 2 * PI, 0.0, 1.0, CYLINDER),
              trimmed_surface(0.0, PI, 0.0, 1.0, CYLINDER), *closed_bs, BEZIER_SURFACE, RAT_BEZIER_SURFACE):
        for tol in (1e-7, 1.0):
            add(requests, 'gl_is_closed', 0, 0.0, 0.0, tol, *s)
    for s in closed_bs:
        for mode in (1, 2):
            for p1, p2 in ((0.0, 1.0), (0.0, 6.0), (0.25, 0.75)):
                add(requests, 'gl_is_closed', mode, p1, p2, 1e-7, *s)
    ring = bezier_surface([[(0.0, 0.0, 0.0), (0.0, 1.0, 0.0), (0.0, 0.0, 0.0)], [(1.0, 0.0, 0.0), (1.0, 1.0, 1.0), (1.0, 0.0, 0.0)],
                           [(2.0, 0.0, 0.0), (2.0, 1.0, 0.0), (2.0, 0.0, 0.0)]])
    for s in (BEZIER_SURFACE, RAT_BEZIER_SURFACE, ring, FLAT_BEZIER_SURFACE):
        for mode in (3, 4):
            for p1, p2 in ((0.0, 1.0), (0.2, 0.7)):
                add(requests, 'gl_is_closed', mode, p1, p2, 1e-7, *s)


def iso_line_requests(requests):
    pcurves = [line2d((0.5, 0.0), (0.0, 1.0)), line2d((0.0, 0.25), (1.0, 0.0)), line2d((0.5, 0.0), (0.0, -1.0)),
               line2d((0.0, 0.25), (-1.0, 0.0)), line2d((0.0, 0.0), (0.6, 0.8)), line2d((0.5, 0.0), (1e-13, 1.0)),
               bspline2d(1, [(0.3, 0.0), (0.3, 1.0)], [(0.0, 2), (1.0, 2)]), bezier2d([(0.0, 0.7), (1.0, 0.7)]),
               bspline2d(1, [(0.3, 0.0), (0.3, 0.5), (0.3, 1.0)], [(0.0, 2), (0.5, 1), (1.0, 2)]),
               bezier2d([(0.0, 0.7), (0.5, 0.7), (1.0, 0.7)]), circle2d(1.0)]
    for c in pcurves:
        for s in (PLANE, CYLINDER, SPHERE, BSPLINE_SURFACE, trimmed_surface(0.0, 1.0, 0.0, 1.0, PLANE)):
            add(requests, 'gl_iso_line', 0.1, 0.9, 1e-7, 0, 0, 0.0, 0, *c, *s)
    for explicit in ((1, 0.25, 1), (0, 0.5, 0), (1, -5.0, 1), (0, 5.0, 1)):
        for s in (PLANE, CYLINDER, BSPLINE_SURFACE):
            add(requests, 'gl_iso_line', 0.2, 0.6, 1e-7, 1, *explicit, *pcurves[1], *s)
    add(requests, 'gl_iso_line', 0.2, 0.2, 1e-7, 1, 1, 0.3, 1, *pcurves[0], *PLANE)


def check_bspline_requests(requests):
    kink_start = bspline(3, [(0.0, 0.0, 0.0), (-1e-4, 0.0, 0.0), (1.0, 1.0, 0.0), (2.0, 0.0, 0.0), (3.0, 1.0, 0.0),
                             (3.0 + 1e-4, 1.0, 0.0)], [(0.0, 4), (1.0, 1), (2.0, 1), (3.0, 4)])
    kink2d = bspline2d(3, [(0.0, 0.0), (-1e-4, 0.0), (1.0, 1.0), (2.0, 0.0), (3.0, 1.0), (3.0 + 1e-4, 1.0)],
                       [(0.0, 4), (1.0, 1), (2.0, 1), (3.0, 4)])
    for c in (kink_start, C2, C0, RAT):
        for tol, ang, ff, fl in ((1e-3, 1e-2, 1, 1), (1e-3, 1e-2, 1, 0), (1e-7, 1e-7, 0, 1), (1e-3, 1e-2, 0, 0)):
            add(requests, 'gl_check_bspline', 3, tol, ang, ff, fl, *c)
    for c in (kink2d, BSPLINE2D, RAT2D):
        for tol, ang, ff, fl in ((1e-3, 1e-2, 1, 1), (1e-7, 1e-7, 1, 0)):
            add(requests, 'gl_check_bspline', 2, tol, ang, ff, fl, *c)


def cos_requests(requests):
    on_cylinder = (circle(2.0, [0.0, 0.0, 5.0, *AX2[3:]]), line2d((0.0, 5.0), (1.0, 0.0)), CYLINDER, 0.0, 2 * PI)
    off_cylinder = (circle(1.95, [0.0, 0.0, 5.0, *AX2[3:]]), line2d((0.0, 5.0), (1.0, 0.0)), CYLINDER, 0.0, 2 * PI)
    lines = (line((0.0, 0.0, 0.0), (1.0, 0.0, 0.0)), line2d((0.0, 0.0), (0.0, 1.0)), PLANE, -2.0, 3.0)
    ellipse_case = (circle(2.0), ellipse2d(4.0, 1.0), PLANE, 0.0, 2 * PI)
    neg = (circle(2.0), ellipse2d(2.0, 1.0), PLANE, -2.0, -1.0)
    torus_case = (circle(4.0), line2d((0.0, 0.0), (0.6, 0.8)), TORUS, 0.0, 80 * PI)
    bs = (C2, BSPLINE2D, PLANE, 0.0, 2.0)
    for c, p, s, f, l in (on_cylinder, off_cylinder, lines, ellipse_case, neg, torus_case, bs):
        for mode in (0, 1, 2):
            for par in (0, 1):
                add(requests, 'gl_cos', mode, f, l, f, l, 1e-9, par, *c, *p, *s)
        add(requests, 'gl_cos', 0, f, l, f, l + 0.5, 1e-9, 0, *c, *p, *s)
        add(requests, 'gl_cos', 0, f, l, f, l, 0.1, 0, *c, *p, *s)


def small_class_requests(requests):
    # The flat cubic knots of 8, 9 and 10 values the law supports (shorter vectors are
    # read out of bounds natively).
    for knots in ([0.0] * 4 + [1.0] * 4, [0.0] * 4 + [0.5] + [1.0] * 4, [0.0] * 4 + [0.3, 0.6] + [1.0] * 4,
                  [0.0] * 4 + [0.5, 0.5] + [1.0] * 4):
        for s in (RAT_BSPLINE_SURFACE, BSPLINE_SURFACE):
            uv = [0.0, 0.0, 0.5, 0.5, 0.25, 0.25, 1.0, 1.0, 0.75, -0.5, 0.4, 2.0]
            add(requests, 'gl_denom', len(knots), *knots, len(uv) // 2, *uv, *s)
    pts = [(0.0, 0.0, 0.0), (1.0, 2.0, 0.0), (2.0, 1.0, 1.0), (3.0, 3.0, 0.5), (4.0, 0.0, 0.0)]
    params = [0.0, 1.0, 2.0, 3.0, 4.0]
    for deg in (1, 2, 3, 4, 5, 6, 0):
        add(requests, 'gl_interpolate', deg, len(pts), *[c for p in pts for c in p], *params)
    add(requests, 'gl_interpolate', 3, 5, *[c for p in pts for c in p], 0.0, 1.0, 1.0, 3.0, 4.0)
    add(requests, 'gl_interpolate', 3, 5, *[c for p in pts for c in p], 0.0, 2.0, 1.0, 3.0, 4.0)
    add(requests, 'gl_interpolate', 2, 2, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0)
    for s in (PLANE, FLAT_BSPLINE_SURFACE, FLAT_BEZIER_SURFACE, BSPLINE_SURFACE, CYLINDER,
              trimmed_surface(0.0, 1.0, 0.0, 1.0, PLANE), trimmed_surface(0.0, 1e-9, 0.0, 1.0, CYLINDER),
              bezier_surface([[(0.0, 0.0, 0.0), (0.0, 1.0, 1e-5)], [(1.0, 0.0, 0.0), (1.0, 1.0, 0.0)]]), QUARTER_CYLINDER):
        for tol in (1e-7, 1e-3):
            add(requests, 'gl_is_planar', tol, *s)
    for a, b, n in ((0.0, 1.0, 5), (1.0, 10.0, 4), (-1.0, 1.0, 1), (2.0, 2.0, 3), (1.0, 0.0, 6), (0.5, 1.5, 0)):
        add(requests, 'gl_log_sample', a, b, n)
    for lower, coeffs in ((1, [1.0, -2.0, 0.5]), (0, [3.0]), (5, [0.0, 0.0, 0.0, 1.0]), (1, [1e308, 1e308])):
        for x in (0.0, 1.5, -2.0):
            add(requests, 'gl_poly_func', lower, len(coeffs), *coeffs, x)


def tool_requests(requests):
    for p in ((1.0, 2.5, 0.0), (0.0, 0.0, 7.0), (2.5, 0.0, 0.0), (100.0, 0.0, 0.0)):
        for md in (1e-7, 0.1, 1e3):
            for c in (CIRCLE, C2, trimmed(0.0, 2.0, LINE), PERIODIC):
                add(requests, 'gl_tool', 0, md, *p, *c)
            for s in (PLANE, SPHERE, BSPLINE_SURFACE, CYLINDER):
                add(requests, 'gl_tool', 1, md, *p, *s)
            for c in (circle2d(2.5), BSPLINE2D, line2d()):
                add(requests, 'gl_tool', 2, md, *p, *c)
    for c in (BSPLINE2D, circle2d(1.0), BEZ2D, line2d(), PER2D, RAT2D):
        for f, l, st, it in ((0.0, 1.0, 0.5, 100), (0.0, 2.0, 0.3, 5), (0.5, 1.5, 2.0, 100), (1.0, 1.0, 1.0, 10),
                             (0.0, PI, PI / 2, 100)):
            add(requests, 'gl_deviation', 0, f, l, st, it, 10, *c)
            add(requests, 'gl_deviation', 1, f, l, st, it, 10, *c)
        add(requests, 'gl_deviation', 1, 0.0, 1.0, 0.0, 10, 1, *c)


def make_curve_requests(requests):
    for kind in (0, 1, 2):
        for n1, n2, n3 in ((1, 0, 0), (0, 1, 0), (0, 0, 1), (2, 1, 1), (3, 2, 1), (1, 0, 2)):
            add(requests, 'gl_make_curve', kind, n1, n2, n3, 1e-5, 0.0, 2.0, 2, 10, 20)
    add(requests, 'gl_make_curve', 1, 1, 1, 1, 1e-9, -1.0, 1.0, 4, 14, 50)
    add(requests, 'gl_make_curve', 2, 0, 0, 1, 1e-3, 0.0, 6.0, 0, 3, 2)


def append_geomlib_requests(requests: list[str]) -> None:
    """Append every GeomLib request family in a fixed order."""
    to3d_requests(requests)
    same_range_requests(requests)
    build_curve3d_requests(requests)
    adjust_extend_requests(requests)
    inertia_requests(requests)
    arrays_requests(requests)
    surface_query_requests(requests)
    iso_line_requests(requests)
    check_bspline_requests(requests)
    cos_requests(requests)
    small_class_requests(requests)
    tool_requests(requests)
    make_curve_requests(requests)
