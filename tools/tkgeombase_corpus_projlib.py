"""Deterministic requests for the ProjLib package (tools/oracle/tkgeombase_probe_projlib.cxx
and tkgeombase_probe_projlib_curves.cxx).

The elementary projectors run every conic kind on planes, cylinders, cones, spheres and
tori in direct, tilted and indirect frames: the exact cases (generatrices, parallels,
meridians, iso circles of a torus, lines through the cone apex, circles through the
sphere poles), the rejected ones, default-constructed projectors, SetInBounds, UFrame
and VFrame. The static ProjLib::Project overloads run points (on axes, apexes and
poles) and curves. PrjFunc and PrjResolve run every fixed variable, invalid ones and
bounded searches. ProjectOnPlane runs every curve kind along normal and oblique
directions with and without the parametrization kept. ComputeApprox,
ComputeApproxOnPolarSurface, ProjectedCurve and CompProjectedCurve run curves on and off
elementary, swept, offset, Bezier and B-spline surfaces, across seams, through poles,
leaving bounded surfaces, and with every option and constructor.
"""
from __future__ import annotations
import math
from tkgeombase_corpus_gce import add
from tkgeombase_corpus_geomspecs import (AX2, AX3, AX3_INDIRECT, AX3_TILTED, BEZIER, C0, C2, CIRCLE, CIRCLE_TILTED,
                                         ELLIPSE, HYPERBOLA, LINE, PARABOLA, PERIODIC, QUARTER, RAT, RAT_BEZIER,
                                         TILTED, bezier, bezier_surface, bspline, bspline2d, bspline_surface, circle,
                                         circle2d, cone, cylinder, ellipse, extrusion, hyperbola, line, line2d, offset,
                                         offset_surface, parabola, plane, revolution, sphere, torus, trimmed,
                                         trimmed_surface, FLAT_GRID)

S5 = math.sin(0.5)
C5 = math.cos(0.5)
FRAMES = [AX3, AX3_TILTED, AX3_INDIRECT]
# Elementary surfaces in each frame.
PLANES = [plane(ax) for ax in FRAMES]
CYLINDERS = [cylinder(2.0, ax) for ax in FRAMES]
CONES = [cone(0.5, 1.0, ax) for ax in FRAMES]
SPHERES = [sphere(2.0, ax) for ax in FRAMES]
TORI = [torus(3.0, 1.0, ax) for ax in FRAMES]
ELEMENTARY = PLANES + CYLINDERS + CONES + SPHERES + TORI
APEX_Z = -1.0 / math.tan(0.5)


def ax2(p, n, x):
    return [*p, *n, *x]


# Conics and lines in the direct frame, most of them on some elementary surface.
CURVES = [
    line([2.0, 0.0, 0.0], [0.0, 0.0, 1.0]), line([2.0, 0.0, 1.0], [0.0, 0.0, -1.0]), LINE,
    line([0.0, 1.0, 0.5], [1.0, 0.0, 0.0]), line([1.0, 0.0, 0.0], [S5, 0.0, C5]),
    line([0.0, 0.0, APEX_Z], [S5, 0.0, C5]), line([0.0, 0.0, APEX_Z], [0.0, -S5, -C5]),
    circle(2.0), circle(2.0, ax2([0, 0, 1.0], [0, 0, 1.0], [0, 1.0, 0])), circle(2.0, ax2([0, 0, 1.0], [0, 0, -1.0], [1.0, 0, 0])),
    circle(1.0 + S5, ax2([0, 0, C5], [0, 0, 1.0], [1.0, 0, 0])), circle(math.sqrt(3.0), ax2([0, 0, 1.0], [0, 0, 1.0], [1.0, 0, 0])),
    circle(2.0, ax2([0, 0, 0], [0, 1.0, 0], [1.0, 0, 0])), circle(2.0, ax2([0, 0, 0], [0, 1.0, 0], [0, 0, 1.0])),
    circle(2.0, ax2([0, 0, 0], [0, -1.0, 0], [math.cos(0.3), 0, math.sin(0.3)])),
    circle(2.0, ax2([0, 0, 0], [1.0, 0, 1.0], [0, 1.0, 0])), circle(1.0, ax2([3.0, 0, 0], [0, 1.0, 0], [1.0, 0, 0])),
    circle(1.0, ax2([0, 3.0, 0], [-1.0, 0, 0], [0, 0, 1.0])), circle(3.0 + math.sqrt(0.75), ax2([0, 0, 0.5], [0, 0, 1.0], [1.0, 0, 0])),
    circle(3.0 - math.sqrt(0.75), ax2([0, 0, -0.5], [0, 0, 1.0], [0, 1.0, 0])), circle(1.0, ax2([0, 0, 2.0], [0, 1.0, 0], [1.0, 0, 0])),
    circle(1.5, ax2([0.5, 0.25, 3.0], [0, 0, 1.0], [1.0, 0, 0])), CIRCLE_TILTED,
    ellipse(2.0 / C5, 2.0, ax2([0, 0, 0], [0, -S5, C5], [0, C5, S5])), ELLIPSE, ellipse(3.0, 1.0),
    PARABOLA, parabola(1.5, TILTED), HYPERBOLA, hyperbola(1.0, 2.0),
]


def elem_requests(requests):
    for surface in ELEMENTARY:
        for curve in CURVES:
            add(requests, 'pl_elem', 1, 1, 0, 0.0, 0.0, 0.0, 0.0, *surface, *curve)
    for surface in [PLANES[0], CYLINDERS[0], CONES[1], SPHERES[0], TORI[2]]:
        for curve in CURVES[::3]:
            add(requests, 'pl_elem', 1, 0, 0, 0.0, 0.0, 0.0, 0.0, *surface, *curve)
            add(requests, 'pl_elem', 0, 1, 0, 0.0, 0.0, 0.0, 0.0, *surface, *curve)
    for curve in CURVES[7:22]:
        for u in [0.0, 1.0, 2.5, 4.0, -3.0]:
            add(requests, 'pl_elem', 1, 1, 1, u, 0.0, 0.0, 0.0, *SPHERES[0], *curve)
        add(requests, 'pl_elem', 1, 1, 1, 0.5, 0.0, 0.0, 0.0, *SPHERES[1], *curve)
    for surface in [CYLINDERS[0], CONES[0], TORI[0], CYLINDERS[1]]:
        for curve in CURVES[:20]:
            add(requests, 'pl_elem', 1, 1, 2, 0.5, 1.5, -math.pi, 2 * math.pi, *surface, *curve)
            add(requests, 'pl_elem', 1, 1, 3, 1.0, 2.0, 1.0, 2 * math.pi, *surface, *curve)
    for curve in CURVES[:3]:
        add(requests, 'pl_elem', 1, 1, 2, 0.0, 1.0, 10.0, 0.5, *PLANES[1], *curve)


POINTS = [[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 0.0, 2.0], [0.0, 0.0, -2.0], [0.0, 0.0, APEX_Z], [1.0, -2.0, 0.5],
          [-3.0, 0.0, 1.0], [4.0, 0.0, 0.0], [0.0, 0.0, 5.0], [1e-9, 0.0, 1.0], [-1.0, -1e-12, 0.0]]


def static_requests(requests):
    for surface in ELEMENTARY + [bezier_surface(FLAT_GRID), extrusion([0.0, 0.0, 1.0], CIRCLE)]:
        for point in POINTS:
            add(requests, 'pl_static', 0, *surface, *point)
    for surface in ELEMENTARY:
        for curve in CURVES:
            add(requests, 'pl_static', 1, *surface, *curve)


GRID4 = [[(0.0, 0.0, 0.0), (0.0, 1.0, 0.2), (0.0, 2.0, 0.0), (0.0, 3.0, 0.3)],
         [(1.0, 0.0, 0.3), (1.0, 1.0, 0.8), (1.0, 2.0, 0.4), (1.0, 3.0, 0.0)],
         [(2.0, 0.0, 0.0), (2.0, 1.0, 0.3), (2.0, 2.0, 0.6), (2.0, 3.0, 0.2)],
         [(3.0, 0.0, 0.2), (3.0, 1.0, 0.0), (3.0, 2.0, 0.3), (3.0, 3.0, 0.1)]]
BSPLINE_SURFACE = bspline_surface((2, 2), GRID4, [(0.0, 3), (0.5, 1), (1.0, 3)], [(0.0, 3), (0.5, 1), (1.0, 3)])


def prj_requests(requests):
    curves = [C2, CIRCLE, line([0.5, 0.5, 1.0], [1.0, 1.0, 0.0])]
    surfaces = [SPHERES[0], BSPLINE_SURFACE, PLANES[1], CYLINDERS[0]]
    for fix in [1, 2, 3, 0, 4]:
        for curve in curves:
            for surface in surfaces:
                add(requests, 'pl_prjfunc', fix, 0.3, 0.4, 0.6, *curve, *surface)
    boxes = [(1e-7, 1e-7, 0.0, 0.0, 1.0, 1.0), (1e-9, 1e-9, -10.0, -10.0, 10.0, 10.0), (1e-3, 1e-3, 0.5, 0.5, 0.6, 0.6),
             (1e-7, 1e-7, 0.0, -1.5, 6.3, 1.5)]
    for fix in [1, 2, 3, 5]:
        for curve in curves:
            for surface in surfaces:
                for index, box in enumerate(boxes):
                    ftol = [1e-10, -1.0, 1e-20, 1e-6][index]
                    add(requests, 'pl_prjresolve', fix, 0.35, 0.3, 0.4, *box, ftol, index % 2, *curve, *surface)


def whole(curve):
    return [0, *curve]


def ranged(u1, u2, curve):
    return [1, u1, u2, *curve]


PLANE_DIRS = [(AX3, [0.0, 0.0, 1.0]), (AX3, [1.0, 1.0, 1.0]), (AX3_TILTED, [0.2, -0.3, 1.0]), (AX3_INDIRECT, [0.0, 1.0, 0.5])]
ON_PLANE_CURVES = [ranged(-1.0, 2.0, LINE), ranged(0.0, 3.0, line([1.0, 2.0, 3.0], [0.0, 0.0, 1.0])),
                   ranged(-2.0, 1.0, line([0.0, 0.0, 0.0], [0.6, 0.0, 0.8])), whole(CIRCLE), whole(CIRCLE_TILTED),
                   whole(ELLIPSE), whole(circle(1.0, ax2([0, 0, 0], [1.0, 0, 0], [0, 1.0, 0]))),
                   ranged(0.5, 2.5, CIRCLE), ranged(-1.5, 2.0, PARABOLA), ranged(-2.0, 1.0, parabola(0.75, TILTED)),
                   ranged(-1.0, 1.5, HYPERBOLA), ranged(-0.5, 1.2, hyperbola(1.5, 1.0)), whole(BEZIER), whole(RAT_BEZIER),
                   whole(C2), whole(RAT), whole(PERIODIC), whole(QUARTER), whole(offset(0.25, C2)),
                   whole(trimmed(0.5, 2.0, CIRCLE)), ranged(0.5, 2.5, C0)]


def on_plane_requests(requests):
    for index, curve in enumerate(ON_PLANE_CURVES):
        for pl, direction in PLANE_DIRS:
            for keep in [1, 0]:
                variant = 1 if direction == [0.0, 0.0, 1.0] and pl is AX3 else 2
                add(requests, 'pl_on_plane', variant, keep, 1e-7, *pl, *direction, *curve)
        add(requests, 'pl_on_plane', 3, 1, 1e-4, *AX3_TILTED, 0.0, 0.0, 1.0, *curve)
        # The default constructor projects along X onto the XOY plane (a direction in
        # the plane, all values NaN); the native reparametrization of the NaN segment
        # of a line along Z never returns, so that line is left out.
        if index != 1:
            add(requests, 'pl_on_plane', 0, index % 2, 1e-7, *AX3, 0.0, 0.0, 1.0, *curve)


def append_projlib_requests(requests):
    """Append every ProjLib request family."""
    elem_requests(requests)
    static_requests(requests)
    prj_requests(requests)
    on_plane_requests(requests)
    __import__('tkgeombase_corpus_projlib_curves').append_projlib_curve_requests(requests)
