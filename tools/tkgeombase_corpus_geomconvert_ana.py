"""Deterministic requests for the GeomConvert recognizers
(tools/oracle/tkgeombase_probe_geomconvert_ana.cxx).

Curves that are lines, circles and ellipses in B-spline, Bezier, trimmed and offset
form (exact and perturbed just inside and outside the tolerance), short curves below
twice the tolerance, closed curves, and every conversion type and target, including a
target the Target type does not handle; point sets that are linear, confused or
spread; circles through collinear, coincident and huge points; surfaces that are
planes, cylinders, cones, spheres and tori as B-splines, Bezier patches, swept and
offset surfaces (with every conversion type and target, and ranges inside and beyond
the bounds); coinciding and distinct elementary surface pairs; the least-squares
functions on exact and noisy clouds with short variable vectors; and the unit
conversions of pcurves on every elementary surface.
"""
from __future__ import annotations
import math
from tkgeombase_corpus_gce import add
from tkgeombase_corpus_geomspecs import (
    AX3, AX3_INDIRECT, AX3_TILTED, BEZIER, BEZIER_SURFACE, C2, CIRCLE, CONE, CYLINDER, ELLIPSE,
    EXT_CYLINDER, EXT_PLANE, FLAT_BEZIER_SURFACE, FLAT_BSPLINE_SURFACE, LINE, LINEAR_BEZIER,
    LINEAR_BSPLINE, PLANE, QUARTER, QUARTER_CYLINDER, R2, REV_BSPLINE, REV_CONE, REV_CYLINDER,
    REV_SPHERE, REV_TORUS, SPHERE, TORUS, bezier, bspline, bspline2d, bspline_surface, circle,
    circle2d, cone, cylinder, ellipse2d, line, line2d, offset, offset_surface, plane, sphere, torus,
    trimmed, trimmed2d, trimmed_surface,
)

PI = math.pi
# A rational quarter ellipse (radii 3 and 1) and a quarter circle with one pole moved.
QUARTER_ELLIPSE = bspline(2, [(3.0, 0.0, 0.0), (3.0, 1.0, 0.0), (0.0, 1.0, 0.0)], [(0.0, 3), (1.0, 3)], [1.0, R2, 1.0])
BENT_QUARTER = bspline(2, [(2.0, 0.0, 0.0), (2.0, 2.0, 1e-4), (0.0, 2.0, 0.0)], [(0.0, 3), (1.0, 3)], [1.0, R2, 1.0])
NEAR_LINE = bspline(3, [(0.0, 0.0, 0.0), (1.0, 1.0, 1e-6), (2.0, 2.0, -1e-6), (3.0, 3.0, 0.0)], [(0.0, 4), (1.0, 4)])
TINY_ARC = trimmed(0.0, 1e-8, circle(1.0))
CLOSED_BEZIER = bezier([(0.0, 0.0, 0.0), (2.0, 2.0, 0.0), (-2.0, 2.0, 0.0), (0.0, 0.0, 0.0)])

ANA_CURVES = [
    LINEAR_BSPLINE, LINEAR_BEZIER, NEAR_LINE, QUARTER, BENT_QUARTER, QUARTER_ELLIPSE,
    trimmed(0.2, 0.8, QUARTER), trimmed(0.3, 2.0, CIRCLE), CIRCLE, trimmed(0.25, 3.5, ELLIPSE),
    trimmed(-1.0, 2.0, LINE), trimmed(0.0, 3.0, offset(0.5, CIRCLE)), BEZIER, C2, TINY_ARC, CLOSED_BEZIER,
    trimmed(0.0, 1.0, trimmed(-2.0, 2.0, LINE)),
]


def ana_curve_requests(requests):
    for c in ANA_CURVES:
        for ct in (0, 1, 2):
            for target in (0, 1, 2, 6):
                if ct != 0 and target != 0:
                    continue
                for tol in (1e-7, 1e-3):
                    add(requests, 'gcv_ana_curve', 0, ct, target, tol, 0.0, 1.0, *c)
        for mode in (1, 2, 3, 4):
            for f, l in ((0.0, 1.0), (0.1, 0.9), (0.5, 0.5)):
                add(requests, 'gcv_ana_curve', mode, 2, 0, 1e-5, f, l, *c)
        add(requests, 'gcv_ana_curve', 1, 0, 1, 1e-5, 0.0, 1.0, *c)
        add(requests, 'gcv_ana_curve', 1, 0, 2, 1e-5, 0.0, 1.0, *c)
        add(requests, 'gcv_ana_curve', 1, 1, 0, 1e-5, 0.0, 1.0, *c)
        add(requests, 'gcv_ana_curve', 1, 0, 6, 1e-5, 0.0, 1.0, *c)
    for f, l in ((0.0, 2 * PI), (-1.0, 1.0), (0.0, 1e-9), (-1e10, 1e10)):
        add(requests, 'gcv_ana_curve', 1, 2, 0, 1e-6, f, l, *CIRCLE)
        add(requests, 'gcv_ana_curve', 1, 2, 0, 1e-6, f, l, *LINE)
    for tol in (1e-7, 0.5):
        add(requests, 'gcv_ana_empty', tol)


def point_requests(requests):
    sets = [
        [], [(1.0, 2.0, 3.0)], [(1.0, 2.0, 3.0), (1.0, 2.0, 3.0)], [(0.0, 0.0, 0.0), (1.0, 1.0, 1.0), (2.0, 2.0, 2.0)],
        [(0.0, 0.0, 0.0), (1.0, 1e-5, 0.0), (2.0, 0.0, 0.0), (3.0, 0.0, 1e-4)], [(0.0, 0.0, 0.0), (0.0, 1.0, 0.0), (1.0, 0.0, 0.0)],
        [(5.0, 5.0, 5.0), (0.0, 0.0, 0.0), (2.5, 2.5, 2.5), (2.5, 2.5, 2.5 + 1e-9)],
    ]
    for pts in sets:
        for tol in (1e-7, 1e-3):
            add(requests, 'gcv_is_linear', tol, len(pts), *[c for p in pts for c in p])
    for p1, p2 in (((0.0, 0.0, 0.0), (1.0, 2.0, 2.0)), ((1.0, 1.0, 1.0), (1.0, 1.0, 1.0)), ((-3.0, 0.5, 1e10), (2.0, 0.5, -1e10))):
        add(requests, 'gcv_get_line', *p1, *p2)
    triples = [
        ((1.0, 0.0, 0.0), (0.0, 1.0, 0.0), (-1.0, 0.0, 0.0)), ((0.0, 0.0, 0.0), (1.0, 1.0, 1.0), (2.0, 2.0, 2.0)),
        ((0.0, 0.0, 0.0), (0.0, 0.0, 0.0), (1.0, 0.0, 0.0)),
        ((1e200, 0.0, 0.0), (0.0, 1.0, 0.0), (-1.0, 0.0, 0.0)), ((2.0, 0.0, 1.0), (0.0, 2.0, 1.0), (-2.0, 0.0, 1.0)),
        ((1.0, 0.0, 0.0), (1.0, 1e-3, 0.0), (1.0, 2e-3, 1e-9)),
    ]
    # Three points within gp::Resolution() are left out: gce_MakeCirc then leaves its
    # status unassigned natively, so the native result is undefined.
    for t in triples:
        add(requests, 'gcv_get_circle', *[c for p in t for c in p])


# A rational quarter sphere patch (a quarter circle revolved as a B-spline) and a quarter
# torus section, both exact.
QUARTER_SPHERE = bspline_surface(
    (2, 2), [[(2.0, 0.0, 0.0), (2.0, 2.0, 0.0), (0.0, 2.0, 0.0)],
             [(2.0, 0.0, 2.0), (2.0, 2.0, 2.0), (0.0, 2.0, 2.0)],
             [(0.0, 0.0, 2.0), (0.0, 0.0, 2.0), (0.0, 0.0, 2.0)]],
    [(0.0, 3), (1.0, 3)], [(0.0, 3), (1.0, 3)], [[1.0, R2, 1.0], [R2, 0.5, R2], [1.0, R2, 1.0]])
CONE_PATCH = bspline_surface(
    (2, 1), [[(1.0, 0.0, 0.0), (2.0, 0.0, 2.0)], [(1.0, 1.0, 0.0), (2.0, 2.0, 2.0)], [(0.0, 1.0, 0.0), (0.0, 2.0, 2.0)]],
    [(0.0, 3), (1.0, 3)], [(0.0, 2), (1.0, 2)], [[1.0, 1.0], [R2, R2], [1.0, 1.0]])
TILTED_FLAT = bspline_surface((1, 1), [[(0.0, 0.0, 0.0), (0.0, 1.0, 1.0)], [(1.0, 0.0, 1.0), (1.0, 1.0, 2.0)]],
                              [(0.0, 2), (1.0, 2)], [(0.0, 2), (1.0, 2)])
ANA_SURFACES = [
    FLAT_BSPLINE_SURFACE, FLAT_BEZIER_SURFACE, TILTED_FLAT, QUARTER_CYLINDER, QUARTER_SPHERE, CONE_PATCH,
    trimmed_surface(0.0, 6.0, -1.0, 2.0, REV_CYLINDER), trimmed_surface(0.0, 3.0, 0.0, 2.0, REV_CONE), REV_TORUS,
    REV_SPHERE, trimmed_surface(0.5, 2.0, 0.5, 2.5, REV_BSPLINE), trimmed_surface(0.0, 6.0, 0.0, 2.0, EXT_CYLINDER),
    trimmed_surface(-1.0, 1.0, 0.0, 1.0, EXT_PLANE), BEZIER_SURFACE,
    trimmed_surface(0.0, 1.0, 0.0, 1.0, offset_surface(0.5, PLANE)), PLANE, trimmed_surface(0.0, 2.0, -1.0, 1.5, CYLINDER),
    CONE, SPHERE, TORUS, plane(AX3_INDIRECT), sphere(2.0, AX3_TILTED),
]


def ana_surf_requests(requests):
    for s in ANA_SURFACES:
        for ct, target in ((-1, 0), (1, 0), (2, 0), (0, 0), (0, 1), (0, 2), (0, 3), (0, 4), (0, 6)):
            add(requests, 'gcv_ana_surf', 0, ct, target, 1e-4, 0, 0, 0, 0, *s)
        for u1, u2, v1, v2 in ((0.1, 0.9, 0.1, 0.9), (0.0, 1.0, 0.0, 1.0), (-1.0, 0.5, 0.0, 1.0), (0.2, 0.2 + 1e-12, 0.0, 1.0)):
            add(requests, 'gcv_ana_surf', 1, 2, 0, 1e-4, u1, u2, v1, v2, *s)
        add(requests, 'gcv_ana_surf', 0, 1, 0, 1e-9, 0, 0, 0, 0, *s)
    add(requests, 'gcv_ana_surf_empty', 1e-4)
    pairs = [
        (PLANE, PLANE), (PLANE, plane([0.0, 0.0, 1e-9, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0])), (PLANE, plane(AX3_TILTED)),
        (CYLINDER, cylinder(2.0, [0.0, 0.0, 5.0, 0.0, 0.0, -1.0, 0.0, 1.0, 0.0, 1])), (CYLINDER, cylinder(2.1)),
        (CONE, CONE), (CONE, cone(0.5, 1.5)), (SPHERE, sphere(3.0, AX3_INDIRECT)), (SPHERE, sphere(3.0)),
        (TORUS, TORUS), (TORUS, torus(5.0, 1.5)), (PLANE, CYLINDER), (BEZIER_SURFACE, PLANE), (PLANE, BEZIER_SURFACE),
    ]
    for a, b in pairs:
        for tol in (1e-7, 1e-2):
            add(requests, 'gcv_same_surf', tol, *a, *b)


def cloud(kind, n, noise):
    pts = []
    for i in range(n):
        a = 0.7 * i + 0.1
        h = 0.3 * (i % 5) - 0.5
        e = noise * math.sin(3.1 * i + 0.2)
        if kind == 0:
            r = 1.0 + 0.5 * h
            pts.append((r * math.cos(a) + e, r * math.sin(a), h))
        elif kind == 1:
            pts.append((2.0 * math.cos(a) + e, 2.0 * math.sin(a), h))
        else:
            b = 0.4 * i - 1.0
            pts.append((3.0 * math.cos(a) * math.cos(b) + e, 3.0 * math.sin(a) * math.cos(b), 3.0 * math.sin(b)))
    return pts


def ls_requests(requests):
    xs = {0: [[0.0, 0.0, -2.0, 0.4636476090008061, 0.0], [0.1, -0.1, -1.5, 0.5, 0.2], [0.0, 0.0, 0.0, 0.3]],
          1: [[0.0, 0.0, 0.0, 2.0], [0.2, -0.1, 0.3, 1.8], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0, 2.0, 7.0]],
          2: [[0.0, 0.0, 0.0, 3.0], [0.1, 0.2, -0.1, 2.5], [1.0, 1.0]]}
    for kind in (0, 1, 2):
        for noise in (0.0, 1e-3):
            pts = cloud(kind, 12, noise)
            for x in xs[kind]:
                for setters in (0, 1):
                    add(requests, 'gcv_ls_dist', kind, setters, 0.0, 0.0, 1.0, len(pts), *[c for p in pts for c in p],
                        len(x), *x)
    add(requests, 'gcv_ls_dist', 1, 0, 0.0, 0.6, 0.8, 1, 1.0, 2.0, 3.0, 4, 0.5, 0.5, 0.5, 1.0)


def units_requests(requests):
    surfaces = [PLANE, CYLINDER, CONE, SPHERE, TORUS, trimmed_surface(0.0, 2.0, -1.0, 1.0, CYLINDER), BEZIER_SURFACE]
    curves = [line2d((0.5, 0.25), (0.6, 0.8)), line2d((1.0, 0.0), (0.0, 1.0)), circle2d(1.5), ellipse2d(2.0, 1.0),
              trimmed2d(0.0, 2.0, circle2d(1.0)),
              bspline2d(2, [(0.0, 0.0), (1.0, 1.0), (2.0, 0.5)], [(0.0, 3), (1.0, 3)])]
    for s in surfaces:
        for c in curves:
            for mode in (0, 1, 2):
                add(requests, 'gcv_units', mode, 1000.0, 180.0 / PI, *c, *s)
        add(requests, 'gcv_units', 0, 1.0, 1.0, *curves[0], *s)


def append_geomconvert_ana_requests(requests: list[str]) -> None:
    """Append the recognizer, least-squares and units request families."""
    ana_curve_requests(requests)
    point_requests(requests)
    ana_surf_requests(requests)
    ls_requests(requests)
    units_requests(requests)
