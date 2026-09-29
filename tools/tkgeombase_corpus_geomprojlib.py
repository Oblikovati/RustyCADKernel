"""Deterministic requests for the GeomProjLib package (tools/oracle/tkgeombase_probe_geomprojlib.cxx).

Every Curve2d overload, Project and ProjectOnPlane run on every Geom curve kind (lines,
conics, Bezier and B-spline curves, rational and periodic ones, trimmed, offset and
GeomEval helix curves) and every Geom surface kind (the elementary surfaces in several
frames, extrusions, revolutions, Bezier and B-spline surfaces, rectangular trimmed and
offset surfaces and the GeomEval ellipsoid): curves lying on the surface (analytic,
across seams, through poles, on swept and free-form surfaces) and curves off it.
Curve2d runs the given and the defaulted ranges, bounds and tolerances; ProjectOnPlane
normal, oblique and in-plane directions with and without the parametrization kept.
"""
from __future__ import annotations
import math
from tkgeombase_corpus_gce import add
from tkgeombase_corpus_geomspecs import (AX2, AX3, AX3_INDIRECT, AX3_TILTED, BEZIER, C0, C2, CIRCLE, CIRCLE_TILTED,
                                         ELLIPSE, HYPERBOLA, LINE, PARABOLA, PERIODIC, QUARTER, RAT, RAT_BEZIER, TILTED,
                                         FLAT_GRID, bezier_surface, circle, cone, cylinder, extrusion, line, offset,
                                         offset_surface, parabola, hyperbola, plane, revolution, sphere, torus, trimmed,
                                         trimmed_surface)
from tkgeombase_corpus_projlib import ON_PLANE_CURVES, PLANE_DIRS
from tkgeombase_corpus_projlib_curves import ELEMENTARY_CASES, FREE_CASES, SWEPT_CASES, helix

R2 = math.sqrt(0.5)


def ellipsoid(a, b, c, ax=AX3):
    return [12, *ax, a, b, c]


def curve_spec(adapted):
    """The Geom curve of a ProjLib adaptor spec: `whole(c)` is c, `ranged(u1, u2, c)`
    the trimmed curve over [u1, u2]."""
    if adapted[0] == 0:
        return adapted[1:]
    return trimmed(adapted[1], adapted[2], adapted[3:])


# Every Geom curve kind over a finite range, near the unit sphere and the XOY plane.
CURVE_KINDS = [
    trimmed(-1.0, 2.0, LINE), circle(1.5), CIRCLE_TILTED, ELLIPSE, trimmed(-1.5, 2.0, PARABOLA),
    trimmed(-1.0, 1.5, HYPERBOLA), BEZIER, RAT_BEZIER, C2, C0, RAT, PERIODIC, QUARTER, offset(0.25, C2),
    trimmed(0.5, 2.0, CIRCLE), trimmed(0.0, 6.0, helix(1.5, 0.5)), trimmed(2.5, 0.5, circle(1.5), 0),
]
# Every Geom surface kind.
SURFACE_KINDS = [
    plane(AX3), plane(AX3_TILTED), cylinder(2.0), cone(0.5, 1.0), sphere(2.0), sphere(1.5, AX3_INDIRECT),
    torus(3.0, 1.0), extrusion([0.0, 0.0, 1.0], CIRCLE), revolution([0.0, 0.0, 0.0, 0.0, 0.0, 1.0], line([2.0, 0.0, 0.0], [0.5, 0.0, 1.0])),
    bezier_surface(FLAT_GRID), FREE_CASES[1][0], trimmed_surface(0.0, 3.0, -1.0, 1.0, cylinder(1.0)),
    offset_surface(0.1, bezier_surface(FLAT_GRID)), ellipsoid(2.0, 1.5, 1.0),
]
# Parametric bounds given to the bounded overloads: the natural ranges of the surfaces,
# a band inside them and bounds a little wider than a bounded surface.
BOUNDS = [(0.0, 2 * math.pi, -2.0, 2.0), (-1.0, 7.0, -3.0, 3.0), (-0.1, 1.1, -0.1, 1.1), (0.5, 4.0, -1.5, 1.5)]
TOLERANCES = [1e-7, 0.0, 1e-4, -1.0]


def on_surface_pairs():
    """The (surface, curve spec, range) triples of the ProjLib cases: curves on the
    surface, across seams and through poles, whole or over a range."""
    for surface, curves in ELEMENTARY_CASES + SWEPT_CASES + FREE_CASES:
        for adapted in curves:
            rng = (adapted[1], adapted[2]) if adapted[0] == 1 else None
            yield surface, curve_spec(adapted), (adapted[3:] if rng else adapted[1:]), rng


def curve2d_on_surface_requests(requests):
    for index, (surface, spec, basis, rng) in enumerate(on_surface_pairs()):
        bounds = BOUNDS[index % len(BOUNDS)]
        tol = TOLERANCES[index % len(TOLERANCES)]
        # The three whole-curve overloads rotate over the pairs: the free-form
        # projections dominate the debug Rust twin's time.
        add(requests, 'gpl_curve2d', 3 + index % 3, 0.0, 0.0, *bounds, tol, *spec, *surface)
        if rng is None:
            continue
        # The basis curve over the given range, with and without bounds.
        for variant in [0, 1, 2]:
            add(requests, 'gpl_curve2d', variant, *rng, *bounds, tol, *basis, *surface)


def curve2d_cross_requests(requests):
    for s_index, surface in enumerate(SURFACE_KINDS):
        for c_index, spec in enumerate(CURVE_KINDS):
            # Every surface meets every other curve kind (every kind meets seven or
            # eight surfaces), alternating the default and the bounded overload; Project
            # runs the whole cross product.
            k = s_index + c_index
            if k % 2:
                continue
            if k % 4 == 0:
                add(requests, 'gpl_curve2d', 3, 0.0, 0.0, *BOUNDS[1], 1e-7, *spec, *surface)
            else:
                add(requests, 'gpl_curve2d', 5, 0.0, 0.0, *BOUNDS[k // 2 % len(BOUNDS)], TOLERANCES[k // 2 % 4], *spec,
                    *surface)


def curve2d_range_requests(requests):
    """The ranged overloads on whole curves: sub-ranges, reversed and degenerate ones."""
    ranges = [(0.25, 1.75), (1.0, 1.0), (1.5, 0.5), (-0.5, 0.5)]
    for s_index, surface in enumerate([plane(AX3), cylinder(2.0), sphere(2.0), bezier_surface(FLAT_GRID)]):
        for c_index, spec in enumerate([circle(1.5), C2, BEZIER, LINE]):
            rng = ranges[(s_index + c_index) % len(ranges)]
            if rng == (1.0, 1.0) and s_index == 2:
                # ProjLib_ComputeApprox of the zero-length segment of a line on the sphere
                # never returns, natively and in the port.
                continue
            for variant in [0, 1, 2]:
                add(requests, 'gpl_curve2d', variant, *rng, *BOUNDS[1], 1e-6, *spec, *surface)


def project_requests(requests):
    for surface, spec, _, _ in on_surface_pairs():
        add(requests, 'gpl_project', *spec, *surface)
    for surface in SURFACE_KINDS:
        for spec in CURVE_KINDS:
            add(requests, 'gpl_project', *spec, *surface)
    # Infinite curves on a plane (exact) and on a cylinder (no finite part).
    for spec in [LINE, PARABOLA, HYPERBOLA]:
        add(requests, 'gpl_project', *spec, *plane(AX3_TILTED))


# In-plane directions: X on XOY (the direction of the default ProjLib_ProjectOnPlane)
# and a direction of the tilted plane.
IN_PLANE = [(AX3, [1.0, 0.0, 0.0]), (AX3_TILTED, [3.0, 0.0, -1.0])]


def is_trimmed_line(spec):
    return spec[0] == 8 and spec[4] == 1


def on_plane_requests(requests):
    specs = [curve_spec(adapted) for adapted in ON_PLANE_CURVES]
    specs += [LINE, PARABOLA, HYPERBOLA, trimmed(0.0, 6.0, helix(1.5, 0.5)), parabola(1.5, TILTED), hyperbola(1.0, 2.0)]
    for spec in specs:
        for pl, direction in PLANE_DIRS:
            for keep in [1, 0]:
                add(requests, 'gpl_on_plane', keep, *pl, *direction, *spec)
        for pl, direction in IN_PLANE:
            # Along a direction in the plane a line projects to a NaN segment, whose
            # native trim never returns (the port refuses it): a kept line reaches it in
            # ProjLib_ProjectOnPlane, a trimmed line in the trimmed result.
            if is_trimmed_line(spec):
                continue
            add(requests, 'gpl_on_plane', 0, *pl, *direction, *spec)
            if spec[0] != 1:
                add(requests, 'gpl_on_plane', 1, *pl, *direction, *spec)


def append_geomprojlib_requests(requests):
    """Append every GeomProjLib request family."""
    curve2d_on_surface_requests(requests)
    curve2d_cross_requests(requests)
    curve2d_range_requests(requests)
    project_requests(requests)
    on_plane_requests(requests)
