"""Deterministic requests for the AdvApprox-based Approx classes
(tools/oracle/tkgeombase_probe_approx_adaptors.cxx and tkgeombase_probe_approx_sweep.cxx).

Approx_Curve3d and Approx_Curve2d approximate every Geom and Geom2d curve kind (conics
on bounded ranges, Bezier and B-spline curves, rational, periodic, C0, trimmed and
offset curves) at loose, tight and unreachable tolerances, every continuity, one
segment or many, low and high degrees. Approx_CurveOnSurface runs pcurves on every
surface kind (planes, cylinders, cones, spheres, tori, swept, Bezier and B-spline,
trimmed and offset surfaces), the iso-line paths (U and V lines, reversed, Bezier and
B-spline lines, the sphere exclusion), 3D only, 2D only, both, and the invalid
request of both only-flags. Approx_CurvlinFunc runs its members on a curve, a curve
on a surface and two of them (intervals, abscissae, evaluations of every order,
trims inside and outside [0, 1]); Approx_CurvilinearParameter its three cases;
Approx_SameParameter pcurves already at the same parameter, reparametrized ones,
C0 pcurves and pcurves off their surface. Approx_SweepApproximation runs polynomial
and rational sections with and without 2D curves, implemented and missing D1, D2 and
Resolution, one or several C3 spans, and its evaluation cache.
"""
from __future__ import annotations
from tkgeombase_corpus_gce import add
from tkgeombase_corpus_geomspecs import (AX22D_TILTED, AX3, AX3_TILTED, BEZIER, C0, C2, CIRCLE, CIRCLE_TILTED,
                                         ELLIPSE, HYPERBOLA, LINE, LINEAR_BEZIER, PARABOLA, PERIODIC, QUARTER, RAT,
                                         RAT_BEZIER, bezier2d, bezier_surface, bspline2d, bspline_surface, circle2d,
                                         cone, cylinder, ellipse2d, extrusion, line, line2d, offset, offset2d,
                                         offset_surface, plane, revolution, sphere, torus, trimmed, trimmed2d,
                                         trimmed_surface, FLAT_GRID)

C0_SHAPE, G1, C1, G2, C2_SHAPE, C3, CN = range(7)


def whole(curve):
    return [0, *curve]


def ranged(u1, u2, curve):
    return [1, u1, u2, *curve]


CURVES = [whole(CIRCLE), whole(CIRCLE_TILTED), whole(ELLIPSE), ranged(-1.0, 2.0, LINE), ranged(-1.5, 2.0, PARABOLA),
          ranged(-1.0, 1.5, HYPERBOLA), whole(BEZIER), whole(RAT_BEZIER), whole(C2), whole(C0), whole(RAT),
          whole(PERIODIC), whole(QUARTER), whole(trimmed(0.5, 2.0, CIRCLE)), whole(offset(0.25, C2)),
          ranged(0.5, 2.5, C2), whole(LINEAR_BEZIER)]

CURVES_2D = [line2d((0.5, -0.25), (0.6, 0.8)), circle2d(1.5), ellipse2d(2.0, 0.5, AX22D_TILTED),
             bezier2d([(0.0, 0.0), (1.0, 2.0), (2.0, -1.0), (3.0, 0.5)]),
             bezier2d([(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)], [1.0, 2.0, 1.0]),
             bspline2d(3, [(0.0, 0.0), (0.5, 1.0), (1.5, -0.5), (2.0, 0.5), (3.0, 0.0)],
                       [(0.0, 4), (0.5, 1), (1.0, 4)]),
             bspline2d(2, [(0.0, 0.0), (1.0, 1.0), (2.0, 0.0), (3.0, 1.0)], [(0.0, 3), (0.4, 2), (1.0, 3)]),
             trimmed2d(0.25, 2.5, circle2d(1.0)), offset2d(0.2, circle2d(1.0))]


def curve3d_requests(requests):
    configs = [(1e-3, C2_SHAPE, 20, 14), (1e-6, C1, 50, 8), (1e-9, C0_SHAPE, 100, 14), (1e-4, G1, 1, 3),
               (1e-12, C3, 3, 5), (1e-2, G2, 10, 1)]
    for index, curve in enumerate(CURVES):
        for tol, order, seg, deg in configs[index % 3:] + configs[:index % 3][:2]:
            add(requests, 'axa_curve3d', tol, order, seg, deg, *curve)


def curve2d_requests(requests):
    ranges = {0: (-1.0, 2.0), 1: (0.0, 6.0), 2: (0.5, 4.0), 7: (0.25, 2.5), 8: (0.0, 3.0)}
    for index, curve in enumerate(CURVES_2D):
        first, last = ranges.get(index, (0.0, 1.0))
        for tu, tv, cont, deg, seg in [(1e-5, 1e-5, C2_SHAPE, 14, 16), (1e-8, 1e-3, C1, 6, 40),
                                       (1e-11, 1e-11, C0_SHAPE, 3, 2), (1e-4, 1e-6, G1, 9, 1)]:
            add(requests, 'axa_curve2d', first, last, tu, tv, cont, deg, seg, *curve)
        add(requests, 'axa_curve2d', first - 0.5, last + 0.5, 1e-6, 1e-6, C2_SHAPE, 14, 16, *curve)


PLANE = plane(AX3)
SURFACES = [PLANE, plane(AX3_TILTED), cylinder(2.0), cone(0.4, 1.0), sphere(1.5), torus(3.0, 1.0),
            extrusion([0.0, 0.3, 1.0], CIRCLE), revolution([0.0, 0.0, 0.0, 0.0, 0.0, 1.0], line([2.0, 0.0, 0.0], [0.5, 0.0, 1.0])),
            bezier_surface([[(0.0, 0.0, 0.0), (0.0, 1.0, 0.5), (0.0, 2.0, 0.0)],
                            [(1.0, 0.0, 0.5), (1.0, 1.0, 1.0), (1.0, 2.0, 0.5)],
                            [(2.0, 0.0, 0.0), (2.0, 1.0, 0.5), (2.0, 2.0, 0.0)]]),
            bspline_surface((2, 2), [[(0.0, 0.0, 0.0), (0.0, 1.0, 0.2), (0.0, 2.0, 0.0), (0.0, 3.0, 0.3)],
                                     [(1.0, 0.0, 0.3), (1.0, 1.0, 0.8), (1.0, 2.0, 0.4), (1.0, 3.0, 0.0)],
                                     [(2.0, 0.0, 0.0), (2.0, 1.0, 0.3), (2.0, 2.0, 0.6), (2.0, 3.0, 0.2)],
                                     [(3.0, 0.0, 0.2), (3.0, 1.0, 0.0), (3.0, 2.0, 0.3), (3.0, 3.0, 0.1)]],
                            [(0.0, 3), (0.5, 1), (1.0, 3)], [(0.0, 3), (0.5, 1), (1.0, 3)]),
            bspline_surface((2, 2), [[(0.0, 0.0, 0.0), (0.0, 1.0, 0.2), (0.0, 2.0, 0.0), (0.0, 3.0, 0.3)],
                                     [(1.0, 0.0, 0.3), (1.0, 1.0, 0.8), (1.0, 2.0, 0.4), (1.0, 3.0, 0.0)],
                                     [(2.0, 0.0, 0.0), (2.0, 1.0, 0.3), (2.0, 2.0, 0.6), (2.0, 3.0, 0.2)],
                                     [(3.0, 0.0, 0.2), (3.0, 1.0, 0.0), (3.0, 2.0, 0.3), (3.0, 3.0, 0.1)]],
                            [(0.0, 3), (0.5, 2), (1.0, 3)], [(0.0, 3), (0.5, 2), (1.0, 3)]),
            trimmed_surface(0.0, 3.0, -1.0, 1.0, cylinder(1.0)), offset_surface(0.1, bezier_surface(FLAT_GRID))]

# Pcurves inside the unit parameter square (and iso lines) for each surface.
PCURVES = [bspline2d(3, [(0.1, 0.1), (0.3, 0.6), (0.6, 0.2), (0.9, 0.8)], [(0.0, 4), (1.0, 4)]),
           line2d((0.2, 0.0), (0.0, 1.0)), line2d((0.0, 0.3), (1.0, 0.0)), line2d((0.4, 0.9), (0.0, -1.0)),
           bezier2d([(0.3, 0.1), (0.3, 0.9)]), bspline2d(1, [(0.1, 0.5), (0.9, 0.5)], [(0.0, 2), (1.0, 2)]),
           line2d((0.1, 0.1), (0.6, 0.8)), bezier2d([(0.3, 0.3), (0.3, 0.3)]),
           bspline2d(2, [(0.1, 0.2), (0.4, 0.8), (0.6, 0.1), (0.9, 0.7)], [(0.0, 3), (0.5, 2), (1.0, 3)])]
# Bounded pcurves for the curvilinear classes.
BOUNDED = [PCURVES[0], bezier2d([(0.2, 0.1), (0.5, 0.7), (0.8, 0.3)]), PCURVES[8]]


def cos_requests(requests):
    options = [(0, 0.0, 1.0, 1e-4, C2_SHAPE, 14, 16, 0, 0), (0, 0.1, 0.9, 1e-6, C1, 8, 50, 1, 0),
               (1, 0.0, 1.0, 1e-5, G1, 10, 30, 0, 1), (0, 0.0, 0.8, 1e-3, G2, 6, 8, 1, 0),
               (1, 0.2, 1.0, 1e-7, CN, 12, 100, 1, 0), (0, 0.0, 1.0, 1e-9, C0_SHAPE, 5, 4, 0, 0)]
    for s_index, surface in enumerate(SURFACES):
        for p_index, pcurve in enumerate(PCURVES):
            option = options[(s_index + p_index) % len(options)]
            add(requests, 'axa_cos', *option, *pcurve, *surface)
            if p_index in (1, 2, 3, 4, 5):
                add(requests, 'axa_cos', 0, 0.0, 1.0, 1e-7, C2_SHAPE, 14, 16, 1, 0, *pcurve, *surface)
    add(requests, 'axa_cos', 0, 0.0, 1.0, 1e-4, C2_SHAPE, 14, 16, 1, 1, *PCURVES[0], *PLANE)
    add(requests, 'axa_cos', 1, 0.0, 1.0, 1e-4, C2_SHAPE, 14, 16, 1, 1, *PCURVES[0], *PLANE)
    for surface in [SURFACES[10], SURFACES[6]]:
        for cont in [C0_SHAPE, C1, C2_SHAPE]:
            add(requests, 'axa_cos', 0, 0.0, 1.0, 1e-5, cont, 12, 30, 0, 0, *PCURVES[8], *surface)
            add(requests, 'axa_cos', 0, 0.0, 1.0, 1e-5, cont, 12, 30, 0, 1, *PCURVES[0], *surface)
    for pcurve in [line2d((0.0, 0.3), (1.0, 0.0)), line2d((1.0, 0.0), (0.0, 1.0))]:
        add(requests, 'axa_cos', 0, 0.0, 2.0, 1e-7, C2_SHAPE, 14, 16, 1, 0, *pcurve, *cylinder(1.5, AX3_TILTED))
        add(requests, 'axa_cos', 0, -5.0, 5.0, 1e-7, C1, 14, 16, 1, 0, *pcurve, *PLANE)
        add(requests, 'axa_cos', 0, 0.0, 1.0, 1e-7, C1, 14, 16, 1, 0, *pcurve, *extrusion([0.0, 0.0, 1.0], C0))


def curvlin_commands(kind):
    commands = [[1], [2, C0_SHAPE], [2, C2_SHAPE], [2, C3], [3, 0.4], [4, 0.0, 0], [4, 0.25, 1], [4, 0.5, 2],
                [4, 1.0, 1], [4, 0.75, 3], [4, 0.33, 0], [8, 0.6], [8, 1.0], [8, -0.1], [9, 0.1, 0.9]]
    commands += [[5, 0.2, 0.8, 1e-7], [1], [4, 0.5, 1], [4, 0.3, 2], [5, 0.5, 0.500000001, 1e-7], [5, -0.1, 0.5, 1e-7],
                 [6], [7, 1e-9], [6], [3, 0.2], [2, C1]]
    flat = [v for c in commands for v in c]
    return [len(commands), *flat]


def curvlin_requests(requests):
    for curve in [CURVES[0], CURVES[2], CURVES[8], CURVES[9], CURVES[10], CURVES[14], CURVES[16]]:
        add(requests, 'axa_curvlin', 1, 1e-7, *curve, *CURVES[8], *curvlin_commands(1))
    for surface, pcurve in [(PLANE, PCURVES[0]), (SURFACES[2], PCURVES[6]), (SURFACES[9], PCURVES[0]),
                            (SURFACES[10], PCURVES[6])]:
        add(requests, 'axa_curvlin', 2, 1e-7, *pcurve, *surface, *CURVES[2], *curvlin_commands(2))
    add(requests, 'axa_curvlin', 3, 1e-7, *PCURVES[0], *PLANE, *PCURVES[6], *SURFACES[9], *CURVES[0],
        *curvlin_commands(3))
    add(requests, 'axa_curvlin', 3, 1e-6, *PCURVES[6], *SURFACES[2], *PCURVES[0], *SURFACES[10], *CURVES[9],
        *curvlin_commands(3))
    for (c1, s1), (c2, s2) in [((BOUNDED[0], PLANE), (BOUNDED[1], SURFACES[9])),
                               ((BOUNDED[2], SURFACES[10]), (BOUNDED[1], SURFACES[8])),
                               ((BOUNDED[1], SURFACES[5]), (BOUNDED[0], SURFACES[2]))]:
        add(requests, 'axa_curvlin', 3, 1e-7, *c1, *s1, *c2, *s2, *CURVES[8], *curvlin_commands(3))
    add(requests, 'axa_curvlin', 1, 1e-7, *ranged(0.0, 0.0, LINE), *CURVES[8], 1, 1)


def curvilinear_requests(requests):
    for curve in [CURVES[0], CURVES[2], CURVES[6], CURVES[8], CURVES[9], CURVES[10], CURVES[14]]:
        for tol, order, deg, seg in [(1e-5, C2_SHAPE, 14, 16), (1e-8, C1, 8, 30), (1e-3, C0_SHAPE, 4, 2)]:
            add(requests, 'axa_curvilinear', 1, tol, order, deg, seg, *curve)
    for surface, pcurve in [(PLANE, PCURVES[0]), (SURFACES[2], PCURVES[6]), (SURFACES[9], PCURVES[0]),
                            (SURFACES[10], PCURVES[6]), (SURFACES[5], PCURVES[2])]:
        for tol, order in [(1e-5, C2_SHAPE), (1e-7, C1)]:
            add(requests, 'axa_curvilinear', 2, tol, order, 12, 20, *pcurve, *surface)
    for tol, order in [(1e-5, C2_SHAPE), (1e-6, C1), (1e-3, C0_SHAPE)]:
        add(requests, 'axa_curvilinear', 3, tol, order, 12, 20, *PCURVES[0], *PLANE, *PCURVES[6], *SURFACES[9])
        for (c1, s1), (c2, s2) in [((BOUNDED[0], PLANE), (BOUNDED[1], SURFACES[9])),
                                   ((BOUNDED[2], SURFACES[10]), (BOUNDED[1], SURFACES[8]))]:
            add(requests, 'axa_curvilinear', 3, tol, order, 12, 20, *c1, *s1, *c2, *s2)


def same_requests(requests):
    flat_line = line([0.0, 0.0, 0.0], [1.0, 0.0, 0.0])
    slow = [7, 0, 0, 2, 3, 0.0, 0.0, 0.0, 0.25, 0.0, 0.0, 1.0, 0.0, 0.0, 2, 0.0, 1.0, 3, 3]
    wavy = [7, 0, 0, 3, 4, 0.0, 0.0, 0.0, 0.2, 0.0, 0.0, 0.7, 0.0, 0.0, 1.0, 0.0, 0.0, 2, 0.0, 1.0, 4, 4]
    pline = trimmed2d(0.0, 1.0, line2d((0.0, 0.0), (1.0, 0.0)))
    cases = [(trimmed(0.0, 1.0, flat_line), pline, PLANE), (slow, pline, PLANE), (wavy, pline, PLANE),
             (slow, trimmed2d(0.0, 1.0, line2d((0.0, 0.5), (1.0, 0.0))), PLANE),
             (trimmed(0.0, 3.0, CIRCLE), trimmed2d(0.0, 3.0, line2d((0.0, 0.0), (1.0, 0.0))), cylinder(2.5)),
             ([7, 0, 0, 2, 3, 2.5, 0.0, 0.0, 2.5, 2.0, 0.0, 0.0, 3.0, 0.0, 2, 0.0, 1.0, 3, 3],
              trimmed2d(0.0, 1.0, line2d((0.0, 0.0), (1.2, 0.0))), cylinder(2.5)),
             (slow, bspline2d(2, [(0.0, 0.0), (0.2, 0.0), (0.5, 0.0), (1.0, 0.0)], [(0.0, 3), (0.5, 1), (1.0, 3)]), PLANE),
             (slow, bspline2d(1, [(0.0, 0.0), (0.3, 0.0), (1.0, 0.0)], [(0.0, 2), (0.4, 1), (1.0, 2)]), PLANE),
             (slow, pline, plane(AX3_TILTED)), (slow, trimmed2d(0.0, 1.0, line2d((0.0, 0.0), (0.0, 1.0))), PLANE),
             (trimmed(0.0, 1.0, line([0.0, 0.0, 0.0], [0.0, 0.0, 0.0001])), pline, PLANE)]
    wiggly = [7, 0, 0, 3, 6, 0.0, 0.0, 0.0, 0.05, 0.0, 0.0, 0.5, 0.0, 0.0, 0.52, 0.0, 0.0, 0.9, 0.0, 0.0, 1.0, 0.0, 0.0,
              3, 0.0, 0.5, 1.0, 4, 2, 4]
    c0_pcurve = bspline2d(1, [(0.0, 0.0), (0.1, 0.0), (0.35, 0.0), (0.4, 0.0), (0.8, 0.0), (1.0, 0.0)],
                          [(0.0, 2), (0.1, 1), (0.33, 1), (0.41, 1), (0.9, 1), (1.0, 2)])
    helix = [7, 0, 0, 3, 5, 2.5, 0.0, 0.0, 2.5, 1.0, 0.1, 1.5, 2.5, 0.3, -0.5, 2.5, 0.6, -2.5, 0.2, 1.0,
             3, 0.0, 0.5, 1.0, 4, 1, 4]
    cases += [(wiggly, pline, PLANE), (wiggly, c0_pcurve, PLANE), (slow, c0_pcurve, PLANE),
              (helix, trimmed2d(0.0, 1.0, line2d((0.0, 0.0), (1.0, 1.0))), cylinder(2.5)),
              (wiggly, trimmed2d(0.0, 1.0, line2d((0.0, 0.0), (1.0, 0.001))), PLANE)]
    for index, (c3d, c2d, surface) in enumerate(cases):
        for tol in [1e-3, 1e-7, 1e-10]:
            add(requests, 'axa_same', index % 3, tol, *c3d, *c2d, *surface)


def sweep(requests, settings, perform, evals):
    add(requests, 'axs_sweep', *settings, *perform, len(evals), *[v for e in evals for v in e])


def sweep_requests(requests):
    evals = [(0.3, 0, 0.0, 1.0), (0.3, 0, 0.0, 1.0), (0.3, 1, 0.0, 1.0), (0.3, 2, 0.0, 1.0), (0.6, 1, 0.0, 1.0),
             (0.6, 0, 0.5, 1.0), (0.6, 3, 0.5, 1.0), (0.3, 2, 0.0, 0.5)]
    for nbpol in [2, 3, 4]:
        for nb2d in [0, 1, 2]:
            for rational in [0, 1]:
                for d1, d2, cont in [(1, 1, C2_SHAPE), (1, 2, C2_SHAPE), (2, 2, C2_SHAPE), (0, 1, C1), (0, 0, C0_SHAPE),
                                     (1, 0, C2_SHAPE)]:
                    for nbc2, nbc3, path in [(1, 1, (0.0, 1.0)), (2, 3, (0.0, 1.0))]:
                        settings = [nbpol, nb2d, rational, d1, d2, 1, nbc2, nbc3, *path]
                        perform = [0.0, 1.0, 1e-4, 1e-3, 1e-5, 1e-2, cont, 11, 50]
                        sweep(requests, settings, perform, evals)
    sweep(requests, [3, 1, 0, 1, 1, 0, 1, 1, 0.0, 1.0], [0.0, 1.0, 1e-4, 1e-3, 1e-5, 1e-2, C2_SHAPE, 11, 50], evals)
    sweep(requests, [3, 0, 1, 1, 1, 1, 1, 1, 0.5, 2.0], [0.5, 2.0, 1e-9, 1e-8, 1e-9, 1e-3, C2_SHAPE, 4, 2], evals)
    sweep(requests, [4, 2, 1, 1, 1, 1, 2, 4, 0.0, 1.0], [0.0, 1.0, 1e-10, 1e-10, 1e-10, 1e-3, C1, 3, 1], evals)


def append_approx_adaptor_requests(requests: list[str]) -> None:
    """Append the Approx adaptor-class and sweep requests in a fixed order."""
    curve3d_requests(requests)
    curve2d_requests(requests)
    cos_requests(requests)
    curvlin_requests(requests)
    curvilinear_requests(requests)
    same_requests(requests)
    sweep_requests(requests)
