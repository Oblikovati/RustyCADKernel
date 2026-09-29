"""Deterministic requests for the Geom2dInt curve-curve classes and Geom2dAPI_InterCurveCurve
(tools/oracle/tkgeomalgo_probe_ginter.cxx).

Curves are written as ga_curve2d reads them and domains as in tkgeomalgo_corpus_intcurve.py.
The requests cover the curve polygon (few and many samples, the box refinement against
overlapping, touching and disjoint boxes, closed polygons), the exact intersection point from
polygon seats (inner, first, last and out-of-range chords) and from explicit bounds, the
polygon-polygon intersector (crossing, tangent, overlapping and disjoint curves, self
intersections, the protected iterations), Geom2dInt_GInter on every pair of curve kinds by
every constructor and Perform (conic pairs, conic and curve, curve pairs, composite curves,
self intersections, unbounded and inner domains), and Geom2dAPI_InterCurveCurve with its
segments.
"""
from __future__ import annotations
import math
import random
from tkgeomalgo_corpus_intcurve import INFINITE, bounded, first_only, fmt
from tkgeomalgo_corpus_geom2d import (BEZIER, CURVES, KINKED, PERIODIC, RANGES, RATIONAL, RBEZIER,
                                      WAVE, c_bezier, c_bspline, c_circle, c_ellipse, c_line,
                                      c_offset, c_trimmed)

TWO_PI = 2.0 * math.pi
LOOP = c_bezier([(0.0, 0.0), (3.0, 2.0), (-1.0, 2.0), (2.0, 0.0)])
EIGHT = c_bspline(3, [0.0, 1.0, 2.0, 3.0, 4.0], [1, 1, 1, 1, 1],
                  [(2.0, 1.0), (-2.0, -1.0), (-2.0, 1.0), (2.0, -1.0)], periodic=1)
WAVE_UP = c_bspline(3, [0.0, 1.0, 2.0, 3.0], [4, 1, 1, 4],
                    [(-3.0, 1.0), (-2.0, 3.0), (-1.0, -1.0), (0.0, 3.0), (1.0, -1.0), (3.0, 1.0)])
ARC = c_bezier([(-2.0, 0.0), (0.0, 2.0), (2.0, 0.0)])
ARC_TOUCH = c_bezier([(-2.0, 2.0), (0.0, 0.0), (2.0, 2.0)])
OCC23706_1 = c_bspline(2, [0.0, 0.2, 0.3, 0.4, 0.5], [3, 1, 1, 1, 3],
                       [(2.0, 0.0), (3.0, -1.0), (5.0, 5.0), (5.0, 5.0), (6.0, 8.0), (4.0, 7.0)],
                       weights=[1.0] * 6)
OCC23706_2 = c_bspline(2, [1.0, 2.0, 3.0, 5.0], [3, 1, 1, 3],
                       [(6.0, 3.0), (5.001, 5.01), (5.001, 5.01), (3.0, 9.0), (2.0, 11.0)], weights=[1.0] * 5)
# (curve, first, last) of the parametric curves that the polygon code handles.
PCURVES = [(BEZIER, 0.0, 1.0), (RBEZIER, 0.0, 1.0), (WAVE, 0.0, 3.0), (KINKED, 0.0, 3.0),
           (PERIODIC, 0.0, 4.0), (RATIONAL, 0.0, 1.0), (LOOP, 0.0, 1.0), (EIGHT, 0.0, 4.0),
           (WAVE_UP, 0.0, 3.0), (ARC, 0.0, 1.0), (ARC_TOUCH, 0.0, 1.0), (c_offset(0.3, WAVE), 0.0, 3.0),
           (c_trimmed(0.2, 2.8, WAVE), 0.2, 2.8), (OCC23706_1, 0.0, 0.5), (OCC23706_2, 1.0, 5.0)]


def dom(first, last, tol=1e-9):
    return bounded(first, last, tol)


def append_polygon_requests(requests: list[str]) -> None:
    """Polygons of every curve, refined in the box of another one."""
    for index, (curve, u0, u1) in enumerate(PCURVES):
        other, v0, v1 = PCURVES[(index + 3) % len(PCURVES)]
        for n, tol in ((2, 1e-7), (5, 1e-3), (20, 1e-9), (40, 1e-5)):
            requests.append('g2int_polygon ' + fmt([*curve, *dom(u0, u1), n, tol, *other, *dom(v0, v1), 10, tol,
                                                    1e-4, 0]))
        inner = dom(u0 + 0.3 * (u1 - u0), u0 + 0.6 * (u1 - u0))
        requests.append('g2int_polygon ' + fmt([*curve, *dom(u0, u1), 30, 1e-7, *curve, *inner, 8, 1e-7, 0.0, 1]))
        far = c_line(50.0, 50.0, 1.0, 0.0)
        requests.append('g2int_polygon ' + fmt([*curve, *dom(u0, u1), 12, 1e-7, *far, *dom(0.0, 1.0), 5, 1e-7,
                                                0.5, 1]))
        edge = c_line(-10.0, 0.0, 1.0, 0.0)
        requests.append('g2int_polygon ' + fmt([*curve, *dom(u0, u1), 25, 1e-7, *edge, *dom(0.0, 20.0), 3, 1e-7,
                                                1e-9, 0]))
    requests.append('g2int_polygon ' + fmt([*WAVE, *INFINITE, 10, 1e-7, *WAVE, *dom(0.0, 3.0), 10, 1e-7, 0.0, 0]))


def append_exact_requests(requests: list[str]) -> None:
    """Exact points from polygon seats and from explicit bounds."""
    pairs = [(0, 2), (2, 8), (2, 3), (6, 9), (9, 10), (13, 14), (4, 7), (1, 5), (3, 11), (0, 0)]
    for a, b in pairs:
        (ca, a0, a1), (cb, b0, b1) = PCURVES[a], PCURVES[b]
        for n in (6, 20):
            for seg1, par1, seg2, par2 in ((1, 0.0, 1, 0.0), (3, 0.5, 4, 0.25), (n - 1, 0.0, n - 1, 1.0),
                                           (n, 0.0, 2, 0.3), (0, 0.7, -1, 0.2), (n + 3, 0.5, 2, 0.5)):
                requests.append('g2int_exact ' + fmt([*ca, *dom(a0, a1), n, *cb, *dom(b0, b1), n, 1e-7, seg1, par1,
                                                      seg2, par2, 0.5 * (a0 + a1), 0.5 * (b0 + b1), a0, b0, a1, b1]))
        requests.append('g2int_exact ' + fmt([*ca, *dom(a0, a1), 10, *cb, *dom(b0, b1), 10, 1e-2, 5, 0.5, 5, 0.5,
                                              a1, b0, a1, b0, a1, b0]))


def append_polypoly_requests(requests: list[str]) -> None:
    """Pairs and self intersections through the polygon-polygon intersector."""
    rng = random.Random(4211)
    for a in range(len(PCURVES)):
        for b in range(len(PCURVES)):
            if (a + 2 * b) % 3 and a != b:
                continue
            (ca, a0, a1), (cb, b0, b1) = PCURVES[a], PCURVES[b]
            tc = (1e-9, 1e-7, 1e-3)[(a + b) % 3]
            requests.append('g2int_polypoly ' + fmt([*ca, *dom(a0, a1), *cb, *dom(b0, b1), tc, tc, 20, (a + b) % 2, 0, 0]))
    for curve, u0, u1 in PCURVES:
        for tc in (1e-9, 1e-4):
            requests.append('g2int_polypoly ' + fmt([*curve, *dom(u0, u1), *curve, *dom(u0, u1), tc, tc, 20, 0, 1, 0]))
    for mode, nbiter in ((2, 1), (2, 3), (2, 11), (3, 1), (3, 3), (3, 4)):
        for a, b in ((2, 8), (6, 6), (7, 7), (13, 14)):
            (ca, a0, a1), (cb, b0, b1) = PCURVES[a], PCURVES[b]
            requests.append('g2int_polypoly ' + fmt([*ca, *dom(a0, a1), *cb, *dom(b0, b1), 1e-7, 1e-7, 20, 0, mode,
                                                     nbiter]))
    for _ in range(40):
        a, b = rng.randrange(len(PCURVES)), rng.randrange(len(PCURVES))
        (ca, a0, a1), (cb, b0, b1) = PCURVES[a], PCURVES[b]
        s0, s1 = sorted((rng.uniform(a0, a1), rng.uniform(a0, a1)))
        t0, t1 = sorted((rng.uniform(b0, b1), rng.uniform(b0, b1)))
        requests.append('g2int_polypoly ' + fmt([*ca, *dom(s0, s1, 1e-7), *cb, *dom(t0, t1, 1e-7), 1e-7, 1e-7,
                                                 rng.choice((3, 20, 50)), 0, 0, 0]))
    requests.append('g2int_polypoly ' + fmt([*WAVE, *INFINITE, *WAVE_UP, *dom(0.0, 3.0), 1e-7, 1e-7, 20, 0, 0, 0]))


def append_ginter_requests(requests: list[str]) -> None:
    """Geom2dInt_GInter on every pair of curves and by every entry point."""
    curves = CURVES + [(LOOP), (EIGHT), (ARC_TOUCH), (OCC23706_1)]
    ranges = RANGES + [(0.0, 1.0), (0.0, 4.0), (0.0, 1.0), (0.0, 0.5)]
    for a, (ca, (a0, a1)) in enumerate(zip(curves, ranges)):
        for b, (cb, (b0, b1)) in enumerate(zip(curves, ranges)):
            mode = (a + b) % 4
            requests.append('g2int_ginter ' + fmt([*ca, *dom(a0, a1, 1e-7), *cb, *dom(b0, b1, 1e-7), 1e-7, 1e-7, 20,
                                                   (a * b) % 2, mode, 0]))
        requests.append('g2int_ginter ' + fmt([*ca, *dom(a0, a1), *ca, *dom(a0, a1), 1e-9, 1e-9, 20, 0, 4, 0]))
        requests.append('g2int_ginter ' + fmt([*ca, *dom(a0, a1), *ca, *dom(a0, a1), 1e-9, 1e-9, 20, 0, 5, 1]))
        inner = dom(a0 + 0.2 * (a1 - a0), a0 + 0.7 * (a1 - a0), 1e-7)
        requests.append('g2int_ginter ' + fmt([*ca, *inner, *curves[(a + 5) % len(curves)], *INFINITE, 1e-7, 1e-7, 20,
                                               0, 1, 1]))
    for mode in range(6):
        for a, b in ((8, 9), (1, 6), (3, 17), (0, 8), (9, 10), (13, 14)):
            (a0, a1), (b0, b1) = ranges[a], ranges[b]
            requests.append('g2int_ginter ' + fmt([*curves[a], *dom(a0, a1), *curves[b], *dom(b0, b1), 1e-9, 1e-7, 40,
                                                   1, mode, 1]))
            requests.append('g2int_ginter ' + fmt([*curves[a], *dom(a0, a1), *curves[b], *dom(b0, b1), 1e-9, 1e-7, 5,
                                                   0, mode, 0]))
    requests.append('g2int_ginter ' + fmt([*KINKED, *first_only(0.0), *WAVE, *dom(0.0, 3.0), 1e-7, 1e-7, 20, 0, 3, 0]))
    requests.append('g2int_ginter ' + fmt([*c_line(0.0, 0.0, 1.0, 0.0), *INFINITE, *c_line(0.0, 0.0, 0.0, 1.0),
                                           *INFINITE, 1e-7, 1e-7, 20, 0, 4, 0]))


def append_intercc_requests(requests: list[str]) -> None:
    """Geom2dAPI_InterCurveCurve on crossing, overlapping and self-crossing curves."""
    ellipse1 = c_ellipse(0.0, 0.0, 2.0, 1.0)
    ellipse2 = c_ellipse(0.5, 0.5, 2.0, 1.0, 1, 1.0, 1.0)
    line = c_line(-3.0, 0.1, 1.0, 0.0)
    cases = [(ellipse1, ellipse2), (OCC23706_1, OCC23706_2), (WAVE, WAVE_UP), (WAVE, c_trimmed(0.5, 2.5, WAVE)),
             (c_trimmed(-2.0, 2.0, line), WAVE), (c_circle(0.0, 0.0, 1.5), c_trimmed(0.5, 4.0, c_circle(0.0, 0.0, 1.5))),
             (c_trimmed(-1.0, 3.0, line), c_trimmed(0.0, 5.0, line)), (ARC, ARC_TOUCH), (LOOP, BEZIER),
             (KINKED, WAVE), (PERIODIC, EIGHT), (RATIONAL, c_circle(0.0, 0.0, 1.0)), (RBEZIER, ARC),
             (c_trimmed(0.0, 2.0, KINKED), c_trimmed(1.0, 3.0, KINKED)), (c_offset(0.3, WAVE), WAVE)]
    for c1, c2 in cases:
        for tol in (1e-7, 1e-3):
            requests.append('g2api_intercc ' + fmt([*c1, 0, *c2, tol]))
            requests.append('g2api_intercc ' + fmt([*c2, 0, *c1, tol]))
    for curve in (LOOP, EIGHT, WAVE, KINKED, ellipse1, c_trimmed(0.0, 3.0, EIGHT)):
        requests.append('g2api_intercc ' + fmt([*curve, 1, *curve, 1e-7]))
    requests.append('g2api_intercc ' + fmt([*line, 1, *line, 1e-7]))


def append_ginter_family_requests(requests: list[str]) -> None:
    """Append the curve-curve requests in a fixed order."""
    append_polygon_requests(requests)
    append_exact_requests(requests)
    append_polypoly_requests(requests)
    append_ginter_requests(requests)
    append_intercc_requests(requests)
