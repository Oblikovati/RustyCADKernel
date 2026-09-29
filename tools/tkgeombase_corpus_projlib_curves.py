"""Deterministic requests for the ProjLib approximations: ComputeApprox,
ComputeApproxOnPolarSurface, ProjectedCurve and CompProjectedCurve
(tools/oracle/tkgeombase_probe_projlib_curves.cxx). See tools/tkgeombase_corpus_projlib.py.
"""
from __future__ import annotations
import math
from tkgeombase_corpus_gce import add
from tkgeombase_corpus_geomspecs import (AX2, AX3, AX3_TILTED, BEZIER, C0, C2, CIRCLE, CIRCLE_TILTED, ELLIPSE,
                                         HYPERBOLA, LINE, PARABOLA, PERIODIC, QUARTER, RAT, RAT_BEZIER, FLAT_GRID,
                                         bezier, bezier_surface, bspline, bspline2d, bspline_surface, circle,
                                         circle2d, cone, cylinder, ellipse, extrusion, line, line2d, offset,
                                         offset_surface, plane, revolution, sphere, torus, trimmed, trimmed_surface)
from tkgeombase_corpus_projlib import BSPLINE_SURFACE, S5, C5, ax2

NO_CONSTRAINT, PASS_POINT, TANGENCY_POINT, CURVATURE_POINT = range(4)
C0_SHAPE, G1, C1, G2, C2_SHAPE, C3, CN = range(7)
R2 = math.sqrt(0.5)


def whole(curve):
    return [0, *curve]


def ranged(u1, u2, curve):
    return [1, u1, u2, *curve]


def helix(radius, pitch, frame=AX2):
    return [10, *frame, radius, pitch]


# The hexagon tube: periodic in U, linear in V.
HEXAGON = [(2.0 * math.cos(k * math.pi / 3), 2.0 * math.sin(k * math.pi / 3)) for k in range(6)]
TUBE = bspline_surface((3, 1), [[(x, y, 0.0), (x, y, 2.0)] for x, y in HEXAGON],
                       [(float(k), 1) for k in range(7)], [(0.0, 2), (1.0, 2)], periodic=(1, 0))
# A B-spline cap whose first U row collapses to the apex (0, 0, 1).
APEX_GRID = [[(0.0, 0.0, 1.0)] * 3,
             [(-1.0, 1.0, 0.5), (0.0, 1.0, 0.6), (1.0, 1.0, 0.5)],
             [(-2.0, 2.0, 0.0), (0.0, 2.0, 0.1), (2.0, 2.0, 0.0)]]
APEX_SURFACE = bspline_surface((2, 2), APEX_GRID, [(0.0, 3), (1.0, 3)], [(0.0, 3), (1.0, 3)])
# The quarter meridian of the sphere of radius 2 from the pole to the equator.
POLE_QUARTER = bspline(2, [(0.0, 0.0, 2.0), (2.0, 0.0, 2.0), (2.0, 0.0, 0.0)], [(0.0, 3), (1.0, 3)], [1.0, R2, 1.0])
MERIDIAN = circle(2.0, ax2([0, 0, 0], [0, 1.0, 0], [1.0, 0, 0]))

PLANE = plane(AX3)
CYL = cylinder(2.0)
CONE = cone(0.5, 1.0)
SPHERE = sphere(2.0)
TORUS = torus(3.0, 1.0)
REVOLUTION = revolution([0.0, 0.0, 0.0, 0.0, 0.0, 1.0], line([2.0, 0.0, 0.0], [0.5, 0.0, 1.0]))
REVOLUTION_APEX = revolution([0.0, 0.0, 0.0, 0.0, 0.0, 1.0], line([0.0, 0.0, 0.0], [R2, 0.0, R2]))
EXTRUSION = extrusion([0.0, 0.0, 1.0], CIRCLE)
FLAT = bezier_surface(FLAT_GRID)

# (surface, curves) pairs: curves on the surface, across its seams and poles, and off it.
ELEMENTARY_CASES = [
    (PLANE, [whole(CIRCLE), ranged(-1.0, 2.0, LINE), whole(ELLIPSE), ranged(-1.5, 2.0, PARABOLA),
             ranged(-1.0, 1.5, HYPERBOLA), whole(BEZIER), whole(RAT_BEZIER), whole(C2), whole(PERIODIC), whole(RAT),
             whole(offset(0.25, C2)), whole(C0)]),
    (plane(AX3_TILTED), [whole(CIRCLE_TILTED), whole(C2), whole(BEZIER)]),
    (CYL, [whole(circle(2.0, ax2([0, 0, 1.0], [0, 0, 1.0], [0, 1.0, 0]))), ranged(0.0, 3.0, line([2.0, 0, 0], [0, 0, 1.0])),
           whole(ellipse(2.0 / C5, 2.0, ax2([0, 0, 0], [0, -S5, C5], [0, C5, S5]))),
           ranged(0.0, 12.0, helix(2.0, 0.5)), whole(CIRCLE_TILTED), whole(C2), ranged(0.5, 5.5, helix(2.0, 1.0))]),
    (cylinder(1.5, AX3_TILTED), [whole(CIRCLE_TILTED), ranged(-1.0, 1.0, LINE)]),
    (CONE, [whole(circle(1.0 + S5, ax2([0, 0, C5], [0, 0, 1.0], [1.0, 0, 0]))),
            ranged(0.0, 2.0, line([1.0, 0, 0], [S5, 0, C5])), ranged(0.0, 2.0, line([0, 0, -1.0 / math.tan(0.5)], [S5, 0, C5])),
            whole(ELLIPSE), whole(C2), ranged(-1.0, 1.5, HYPERBOLA)]),
    (SPHERE, [whole(circle(2.0)), whole(circle(math.sqrt(3.0), ax2([0, 0, 1.0], [0, 0, 1.0], [1.0, 0, 0]))), whole(MERIDIAN),
              whole(circle(2.0, ax2([0, 0, 0], [1.0, 0, 1.0], [0, 1.0, 0]))), whole(POLE_QUARTER),
              ranged(0.3, 5.0, circle(2.0, ax2([0, 0, 0], [0, 1.0, 0], [0, 0, 1.0]))), whole(C2), whole(CIRCLE_TILTED)]),
    (sphere(1.5, AX3_TILTED), [whole(CIRCLE_TILTED), whole(QUARTER)]),
    (TORUS, [whole(circle(3.0 + math.sqrt(0.75), ax2([0, 0, 0.5], [0, 0, 1.0], [1.0, 0, 0]))),
             whole(circle(1.0, ax2([3.0, 0, 0], [0, 1.0, 0], [1.0, 0, 0]))), whole(C2), whole(circle(3.0))]),
]

SWEPT_CASES = [
    (EXTRUSION, [ranged(0.0, 8.0, helix(2.5, 0.5)), ranged(-1.0, 1.0, line([2.5, 0, 0], [0, 0, 1.0])), whole(C2)]),
    (REVOLUTION, [whole(circle(2.25, ax2([0, 0, 0.5], [0, 0, 1.0], [1.0, 0, 0]))), whole(CIRCLE_TILTED)]),
    (REVOLUTION_APEX, [ranged(0.0, 2.0, line([0, 0, 0], [R2, 0, R2])), ranged(-2.0, 0.0, line([0, 0, 0], [-R2, 0, -R2]))]),
    (trimmed_surface(0.0, 3.0, -1.0, 1.0, cylinder(1.0)), [ranged(-0.5, 0.5, line([1.0, 0, 0], [0, 0, 1.0])), whole(circle(1.0))]),
    (offset_surface(0.1, FLAT), [ranged(0.0, 1.5, line([0.2, 0.3, 0.5], [1.0, 0.5, 0.0]))]),
]

FREE_CASES = [
    (FLAT, [ranged(0.0, 1.5, line([0.2, 0.3, 0.5], [1.0, 0.5, 0.0])), whole(circle(0.5, ax2([1.0, 1.0, 0.2], [0, 0, 1.0], [1.0, 0, 0]))),
            whole(C2), whole(BEZIER)]),
    (BSPLINE_SURFACE, [ranged(0.0, 2.5, line([0.2, 0.5, 0.3], [1.0, 0.8, 0.0])), whole(circle(1.0, ax2([1.5, 1.5, 0.5], [0, 0, 1.0], [1.0, 0, 0])))]),
    (TUBE, [whole(circle(1.8, ax2([0, 0, 1.0], [0, 0, 1.0], [1.0, 0, 0]))), ranged(0.2, 1.8, line([1.9, 0.3, 0], [0, 0, 1.0])),
            ranged(0.0, 6.0, helix(1.9, 0.25))]),
    (APEX_SURFACE, [ranged(0.0, 1.5, line([0.0, 0.0, 1.0], [0.0, R2, -R2])), whole(C2)]),
]

OPTIONS = [(1e-7, -1, -1, -1, TANGENCY_POINT, -1.0), (1e-5, 3, 8, 20, PASS_POINT, -1.0), (1e-4, 2, 12, 100, TANGENCY_POINT, 0.5),
           (1e-9, -1, 6, 5, PASS_POINT, 2.0)]


def approx_requests(requests):
    for s_index, (surface, curves) in enumerate(ELEMENTARY_CASES):
        for c_index, curve in enumerate(curves):
            option = OPTIONS[(s_index + c_index) % len(OPTIONS)]
            add(requests, 'pl_approx', 0, *option, *curve, *surface)
            add(requests, 'pl_approx', 1, *OPTIONS[0], *curve, *surface)
    add(requests, 'pl_approx', 0, *OPTIONS[1], *whole(C2), *FLAT)


def projected_requests(requests):
    for s_index, (surface, curves) in enumerate(ELEMENTARY_CASES + SWEPT_CASES + FREE_CASES):
        for c_index, curve in enumerate(curves):
            option = OPTIONS[(s_index + c_index) % len(OPTIONS)]
            add(requests, 'pl_projected', 0, *option, *curve, *surface)
            add(requests, 'pl_projected', 1 + (c_index % 2), *OPTIONS[(c_index + 1) % 4], *curve, *surface)


INIT_LINE = line2d((0.1, 0.2), (0.6, 0.8))
INIT_BSPLINE = bspline2d(1, [(0.1, 0.1), (0.5, 0.4), (0.9, 0.5)], [(0.0, 2), (0.5, 1), (1.0, 2)])


def polar_requests(requests):
    cases = FREE_CASES + [(CYL, ELEMENTARY_CASES[2][1][:4]), (SPHERE, ELEMENTARY_CASES[5][1][:5]),
                          (TORUS, ELEMENTARY_CASES[7][1][:2]), (CONE, ELEMENTARY_CASES[4][1][:3]),
                          (offset_surface(0.1, FLAT), SWEPT_CASES[4][1])]
    for s_index, (surface, curves) in enumerate(cases):
        for c_index, curve in enumerate(curves):
            option = OPTIONS[(s_index + c_index) % len(OPTIONS)]
            for variant in [0, 1, 2]:
                add(requests, 'pl_polar', variant, *option, *curve, *surface)
            add(requests, 'pl_polar', 3, *option, *curve, *surface, *INIT_BSPLINE)
            add(requests, 'pl_polar', 5, *option, *curve, *surface, *INIT_LINE)
            add(requests, 'pl_polar', 4, *option, *curve, *surface, *INIT_LINE, *line2d((1.1, 0.2), (0.6, 0.8)))
    for curve in [whole(C2), whole(BEZIER), whole(RAT_BEZIER)]:
        add(requests, 'pl_polar', 2, *OPTIONS[0], *curve, *PLANE)
        add(requests, 'pl_polar', 5, *OPTIONS[0], *curve, *PLANE, *INIT_LINE)
    # The C0 B-spline split into C1 spans, from a built and a given initial curve.
    add(requests, 'pl_polar', 0, *OPTIONS[0], *whole(C0), *FLAT)
    add(requests, 'pl_polar', 5, *OPTIONS[0], *whole(C0), *FLAT, *bspline2d(2, [(0.0, 0.0), (0.5, 0.5), (1.0, 0.0), (1.5, 0.5), (2.0, 0.0)],
                                                                             [(0.0, 3), (1.0, 2), (3.0, 3)]))


COMP_CASES = [
    (PLANE, [ranged(-1.0, 1.0, line([0.3, 0.4, 0.0], [0.0, 0.0, 1.0])), ranged(-1.0, 2.0, line([0, 0.5, 0.0], [1.0, 0, 0])),
             whole(CIRCLE), whole(C2)]),
    (CYL, [whole(CIRCLE_TILTED), ranged(0.0, 3.0, line([2.0, 0, 0], [0, 0, 1.0])), whole(circle(2.0, ax2([0, 0, 1.0], [0, 0, 1.0], [0, 1.0, 0]))),
           ranged(0.0, 12.0, helix(2.0, 0.5))]),
    (SPHERE, [whole(MERIDIAN), whole(POLE_QUARTER), whole(circle(2.0))]),
    (TORUS, [whole(circle(3.0 + math.sqrt(0.75), ax2([0, 0, 0.5], [0, 0, 1.0], [1.0, 0, 0])))]),
    (FLAT, [whole(C2), ranged(0.0, 1.5, line([0.2, 0.3, 0.5], [1.0, 0.5, 0.0])), ranged(-1.0, 4.0, line([-0.5, 1.0, 0.3], [1.0, 0.0, 0.0]))]),
    (BSPLINE_SURFACE, [whole(circle(1.0, ax2([1.5, 1.5, 0.5], [0, 0, 1.0], [1.0, 0, 0])))]),
    (REVOLUTION, [whole(circle(2.25, ax2([0, 0, 0.5], [0, 0, 1.0], [1.0, 0, 0])))]),
    (EXTRUSION, [ranged(0.0, 8.0, helix(2.5, 0.5))]),
    (TUBE, [whole(circle(1.8, ax2([0, 0, 1.0], [0, 0, 1.0], [1.0, 0, 0])))]),
]
COMP_OPTIONS = [(0, 1e-6, 1e-7, 1e-7, -1.0, 1, 0, C2_SHAPE, 14, 16), (1, 1e-5, 1e-6, 1e-6, 1.0, 1, 1, C1, 8, 10),
                (2, 1e-6, 0.0, 0.0, -1.0, 0, 1, C2_SHAPE, 12, 20), (2, 1e-4, 0.0, 0.0, 0.5, 1, 1, C0_SHAPE, 6, 4),
                (1, 1e-6, 1e-7, 1e-7, 0.001, 1, 0, C2_SHAPE, 14, 16)]


def comp_requests(requests):
    for s_index, (surface, curves) in enumerate(COMP_CASES):
        for c_index, curve in enumerate(curves):
            option = COMP_OPTIONS[(s_index + c_index) % len(COMP_OPTIONS)]
            add(requests, 'pl_comp', *option, *curve, *surface)
    for option in COMP_OPTIONS:
        add(requests, 'pl_comp', *option, *whole(C2), *FLAT)
    # A curve far above the patch: no near solution within MaxDist (no part), or
    # projected from far away without it.
    far = ranged(0.0, 1.5, line([0.2, 0.3, 5.0], [1.0, 0.5, 0.0]))
    add(requests, 'pl_comp', 1, 1e-6, 1e-7, 1e-7, 0.5, 1, 1, C2_SHAPE, 14, 16, *far, *FLAT)
    add(requests, 'pl_comp', 2, 1e-6, 0.0, 0.0, 1.0, 1, 0, C2_SHAPE, 14, 16, *far, *FLAT)
    add(requests, 'pl_comp', 0, 1e-6, 1e-7, 1e-7, -1.0, 1, 1, C1, 14, 16, *far, *FLAT)
    # A point projection and an iso line on the patch.
    add(requests, 'pl_comp', 0, 1e-6, 1e-7, 1e-7, -1.0, 1, 1, C2_SHAPE, 14, 16,
        *ranged(-1.0, 1.0, line([1.0, 0.5, 0.0], [0.0, 0.0, 1.0])), *FLAT)
    add(requests, 'pl_comp', 2, 1e-6, 0.0, 0.0, -1.0, 1, 1, C2_SHAPE, 14, 16,
        *ranged(0.2, 1.8, line([0.0, 0.5, 0.0], [1.0, 0.0, 0.0])), *FLAT)


def append_projlib_curve_requests(requests):
    """Append the approximation request families."""
    approx_requests(requests)
    projected_requests(requests)
    polar_requests(requests)
    comp_requests(requests)
