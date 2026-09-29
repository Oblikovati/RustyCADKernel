"""Deterministic requests for Geom2dInt and Geom2dAPI
(tools/oracle/tkgeomalgo_probe_geom2dint.cxx and _geom2dapi.cxx).

A 2D curve is written as ga_curve2d reads it (see the probe); conics and domains as in
tkgeomalgo_corpus_intcurve.py. The requests cover the Geom2dInt curve tool and projection
on every curve kind (lines, conics, Bezier, B-spline with several C1 intervals, trimmed and
offset curves), conic-curve intersections through both Geom2dInt classes (composite curves,
conic curves, tangent and missing intersections, the reversed flag), the Geom2dAPI extrema
and projections with every accessor and out-of-range index, interpolations (periodic or not,
with and without parameters and tangents, degenerate inputs) and point approximations in
every Init form.
"""
from __future__ import annotations
import math
import random
from tkgeomalgo_corpus_intcurve import (INFINITE, bounded, circle, ellipse, fmt, hyperbola,
                                        line as gline, parabola)

TWO_PI = 2.0 * math.pi


def c_line(x, y, dx, dy):
    return [0, x, y, dx, dy]


def c_circle(x, y, r, sense=1, dx=1.0, dy=0.0):
    return [1, x, y, dx, dy, sense, r]


def c_ellipse(x, y, a, b, sense=1, dx=1.0, dy=0.0):
    return [2, x, y, dx, dy, sense, a, b]


def c_hyperbola(x, y, a, b, sense=1):
    return [3, x, y, 1.0, 0.0, sense, a, b]


def c_parabola(x, y, f, sense=1):
    return [4, x, y, 1.0, 0.0, sense, f]


def c_bezier(points, weights=None):
    rational = 1 if weights else 0
    weights = weights or [1.0] * len(points)
    return [5, rational, len(points), *[v for (x, y), w in zip(points, weights) for v in (x, y, w)]]


def c_bspline(degree, knots, mults, points, periodic=0, weights=None):
    rational = 1 if weights else 0
    weights = weights or [1.0] * len(points)
    return [6, rational, periodic, degree, len(knots), *[v for k, m in zip(knots, mults) for v in (k, m)],
            len(points), *[v for (x, y), w in zip(points, weights) for v in (x, y, w)]]


def c_trimmed(u1, u2, basis, sense=1, adjust=1):
    return [7, u1, u2, sense, adjust, *basis]


def c_offset(d, basis, nocheck=0):
    return [8, d, nocheck, *basis]


WAVE = c_bspline(3, [0.0, 1.0, 2.0, 3.0], [4, 1, 1, 4],
                 [(-3.0, 0.0), (-2.0, 2.0), (-1.0, -2.0), (0.0, 2.0), (1.0, -2.0), (3.0, 0.0)])
KINKED = c_bspline(2, [0.0, 1.0, 2.0, 3.0], [3, 2, 2, 3],
                   [(-3.0, -1.0), (-2.0, 2.0), (-1.0, 1.5), (0.0, -1.0), (1.0, 1.0), (2.0, 2.0), (3.0, -1.0)])
PERIODIC = c_bspline(3, [0.0, 1.0, 2.0, 3.0, 4.0], [1, 1, 1, 1, 1],
                     [(2.0, 0.0), (0.0, 2.0), (-2.0, 0.0), (0.0, -2.0)], periodic=1)
RATIONAL = c_bspline(2, [0.0, 1.0], [3, 3], [(1.0, 0.0), (1.0, 1.0), (0.0, 1.0)], weights=[1.0, 0.7071, 1.0])
BEZIER = c_bezier([(-2.0, -1.0), (-1.0, 3.0), (1.0, -3.0), (2.0, 1.0)])
RBEZIER = c_bezier([(-2.0, 0.0), (0.0, 3.0), (2.0, 0.0)], [1.0, 2.0, 1.0])
CURVES = [
    c_line(-3.0, 0.5, 1.0, 0.0), c_circle(0.0, 0.0, 1.5), c_circle(1.0, 1.0, 0.5, 0),
    c_ellipse(0.0, 0.0, 2.0, 1.0), c_hyperbola(0.0, 0.0, 1.0, 0.5), c_parabola(0.0, 0.0, 0.5),
    BEZIER, RBEZIER, WAVE, KINKED, PERIODIC, RATIONAL,
    c_trimmed(-2.0, 2.0, c_line(0.0, 0.0, 0.6, 0.8)), c_trimmed(0.5, 4.0, c_circle(0.0, 0.0, 2.0)),
    c_trimmed(0.2, 2.8, WAVE), c_offset(0.3, WAVE), c_offset(0.2, c_circle(0.0, 0.0, 1.0)),
    c_circle(0.0, 0.0, 3.0),
]
# (first, last) of each curve in CURVES, for domains and ranges.
RANGES = [(-10.0, 10.0), (0.0, TWO_PI), (0.0, TWO_PI), (0.0, TWO_PI), (-2.0, 2.0), (-3.0, 3.0),
          (0.0, 1.0), (0.0, 1.0), (0.0, 3.0), (0.0, 3.0), (0.0, 4.0), (0.0, 1.0), (-2.0, 2.0),
          (0.5, 4.0), (0.2, 2.8), (0.0, 3.0), (0.0, TWO_PI), (0.0, TWO_PI)]


def append_tool_requests(requests: list[str]) -> None:
    """The curve tool and the projection on every curve."""
    for curve, (u0, u1) in zip(CURVES, RANGES):
        for u in (u0, 0.5 * (u0 + u1), u1, u0 + 0.3 * (u1 - u0)):
            requests.append('g2int_tool ' + fmt([*curve, u0, u1, u, 1e-7]))
        for x, y in ((0.3, 0.4), (2.5, -1.0), (0.0, 0.0), (-4.0, 3.0)):
            requests.append('g2int_proj ' + fmt([*curve, x, y, u0, u1]))
            requests.append('g2int_proj ' + fmt([*curve, x, y, u0 + 0.25 * (u1 - u0), u0 + 0.5 * (u1 - u0)]))


def curve_domain(first, last, tol=1e-9):
    return bounded(first, last, tol)


CONICS = [(gline(-4.0, 0.2, 1.0, 0.0), INFINITE), (gline(0.1, -4.0, 0.0, 1.0), bounded(-5.0, 5.0)),
          (circle(0.0, 0.0, 1.8), bounded(0.0, TWO_PI, closed=True, z=0.0, zp=TWO_PI)),
          (circle(0.5, 0.0, 1.0), bounded(1.0, 4.0)),
          (ellipse(0.0, 0.0, 2.5, 1.2), bounded(0.0, TWO_PI, closed=True, z=0.0, zp=TWO_PI)),
          (parabola(0.0, -1.0, 0.8), bounded(-4.0, 4.0)), (hyperbola(0.0, 0.0, 1.2, 0.8), bounded(-2.0, 2.0)),
          (circle(0.0, 0.0, 3.0), bounded(0.0, TWO_PI, closed=True, z=0.0, zp=TWO_PI))]


def append_conic_curve_requests(requests: list[str]) -> None:
    """Every conic against every curve through both Geom2dInt conic-curve classes."""
    for index, (conic, cdom) in enumerate(CONICS):
        for curve, (u0, u1) in zip(CURVES, RANGES):
            for which in (0, 1):
                dom = curve_domain(u0, u1)
                requests.append('g2int_conic_curve ' + fmt([*conic, *cdom, *curve, *dom, 1e-9, 1e-9, index % 2, which]))
            inner = curve_domain(u0 + 0.1 * (u1 - u0), u0 + 0.9 * (u1 - u0), 1e-7)
            requests.append('g2int_conic_curve ' + fmt([*conic, *cdom, *curve, *inner, 1e-7, 1e-7, 0, 0]))
    requests.append('g2int_conic_curve ' + fmt([*CONICS[0][0], *CONICS[0][1], *WAVE, *INFINITE, 1e-9, 1e-9, 0, 0]))


def append_function_requests(requests: list[str]) -> None:
    """The distance function between curves and the conic function along a curve."""
    rng = random.Random(311)
    for i in range(30):
        a, b = rng.randrange(len(CURVES)), rng.randrange(len(CURVES))
        (a0, a1), (b0, b1) = RANGES[a], RANGES[b]
        conic = CONICS[rng.randrange(len(CONICS))][0]
        requests.append('g2int_functions ' + fmt([*CURVES[a], *CURVES[b], rng.uniform(a0, a1), rng.uniform(b0, b1),
                                                  *conic, rng.uniform(a0, a1)]))


def append_extrema_requests(requests: list[str]) -> None:
    """Extrema between pairs of curves on their ranges (and a reversed range)."""
    pairs = [(0, 1), (0, 3), (1, 17), (1, 2), (3, 6), (6, 8), (8, 9), (0, 12), (12, 12), (10, 1), (7, 11),
             (13, 16), (14, 15), (4, 5), (0, 0), (5, 9)]
    for a, b in pairs:
        (a0, a1), (b0, b1) = RANGES[a], RANGES[b]
        requests.append('g2api_extrema ' + fmt([*CURVES[a], *CURVES[b], a0, a1, b0, b1]))
        requests.append('g2api_extrema ' + fmt([*CURVES[a], *CURVES[b], a0, 0.5 * (a0 + a1), b0 + 0.25 * (b1 - b0), b1]))
    requests.append('g2api_extrema ' + fmt([*CURVES[0], *CURVES[1], 1.0, 0.0, 0.0, 1.0]))
    trimmed1 = c_trimmed(0.0, 23.0, c_line(0.0, 0.0, 0.0, -1.0))
    trimmed2 = c_trimmed(0.0, 2.0, c_line(1.0, -9.5, -1.0, 0.0))
    requests.append('g2api_extrema ' + fmt([*trimmed1, *trimmed2, 0.0, 23.0, 0.0, 2.0]))


def append_project_requests(requests: list[str]) -> None:
    """Projections on every curve, whole and ranged."""
    for curve, (u0, u1) in zip(CURVES, RANGES):
        for x, y in ((0.3, 0.4), (2.5, -1.0), (0.0, 0.0), (-4.0, 3.0)):
            requests.append('g2api_project ' + fmt([*curve, x, y, 0, 0.0, 0.0]))
            requests.append('g2api_project ' + fmt([*curve, x, y, 1, u0 + 0.2 * (u1 - u0), u0 + 0.7 * (u1 - u0)]))
    requests.append('g2api_project ' + fmt([*CURVES[1], 0.0, 0.0, 1, 2.0, 1.0]))


def interpolate(points, periodic=0, tol=1e-6, params=None, mode=0, extra=()):
    head = [periodic, tol, len(points), *[v for p in points for v in p]]
    head += [1, len(params), *params] if params is not None else [0]
    return 'g2api_interpolate ' + fmt([*head, mode, *extra])


def append_interpolate_requests(requests: list[str]) -> None:
    """Interpolations: 2, 3 and more points, periodic or not, parameters, tangents."""
    square = [(100.0, 0.0), (100.0, 100.0), (0.0, 100.0), (0.0, 0.0), (50.0, -50.0)]
    zigzag = [(-30.4, 8.0), (-16.689912, 17.498217), (-23.803064, 24.748543), (-16.907466, 32.919615),
              (-8.543829, 26.549421), (0.0, 39.2)]
    for pts in ([(0.0, 0.0), (1.0, 1.0)], [(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)], square, zigzag):
        for periodic in (0, 1):
            requests.append(interpolate(pts, periodic))
            requests.append(interpolate(pts, periodic, 1e-6, None, 1, (0.3, 0.4, 1.0, 0.0, 1)))
            requests.append(interpolate(pts, periodic, 1e-6, None, 1, (0.3, 0.4, 1.0, 0.0, 0)))
            flags = [(1.0, 0.5, 1)] + [(0.0, 0.0, 0)] * (len(pts) - 2) + [(0.5, -1.0, 1)]
            requests.append(interpolate(pts, periodic, 1e-6, None, 2, (*[v for t in flags for v in t], 1)))
            mid = [(0.0, 0.0, 0)] * len(pts)
            mid[len(pts) // 2] = (1.0, 1.0, 1)
            requests.append(interpolate(pts, periodic, 1e-6, None, 2, (*[v for t in mid for v in t], 0)))
            params = [float(i) for i in range(len(pts) + periodic)]
            requests.append(interpolate(pts, periodic, 1e-6, params))
    requests.append(interpolate([(0.0, 0.0), (1e-9, 0.0), (1.0, 1.0)]))
    requests.append(interpolate(square, 0, 1e-6, [0.0, 1.0, 1.0, 2.0, 3.0]))
    requests.append(interpolate(square, 1, 1e-6, [0.0, 1.0, 2.0, 3.0]))
    requests.append(interpolate(square, 0, 1e-6, None, 1, (0.0, 0.0, 1.0, 0.0, 1)))
    requests.append(interpolate([(0.0, 0.0)], 0))


def fit(mode, points, args, params=None):
    values = [v for p in points for v in p] if mode != 3 else list(points)
    return 'g2api_points_to_bspline ' + fmt([mode, len(points), *values, *(params or []), *args])


def append_points_to_bspline_requests(requests: list[str]) -> None:
    """Every Init of PointsToBSpline on smooth, noisy and degenerate samples."""
    smooth = [(0.1 * i, math.sin(0.1 * i) + 0.05 * (0.1 * i) ** 2) for i in range(15)]
    rng = random.Random(97)
    noisy = [(0.2 * i, rng.uniform(-0.5, 0.5)) for i in range(12)]
    small = [(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)]
    for pts in (smooth, noisy, small):
        for s in (0, 2, 4, 6):
            requests.append(fit(0, pts, (3, 8, s, 1e-6)))
            requests.append(fit(0, pts, (2, 5, s, 1e-2)))
        for pt in (0, 1, 2):
            requests.append(fit(1, pts, (pt, 3, 8, 4, 1e-3)))
        params = [float(i) ** 1.2 for i in range(len(pts))]
        requests.append(fit(2, pts, (3, 8, 4, 1e-3), params))
        requests.append(fit(2, pts, (3, 8, 4, 1e-3), [5.0] * len(pts)))
        requests.append(fit(4, pts, (1.0, 0.0, 0.0, 8, 4, 1e-3)))
        requests.append(fit(4, pts, (0.5, 0.3, 0.2, 6, 2, 1e-2)))
    for ys, x0, dx in (([0.0, 1.0, 0.0], -2.0, 1.0), ([math.sin(0.3 * i) for i in range(12)], 0.0, 0.3),
                       ([0.0, 2.0, 1.0, 3.0], 1.0, -0.5)):
        requests.append(fit(3, ys, (x0, dx, 3, 5, 4, 1e-7)))
        requests.append(fit(3, ys, (x0, dx, 2, 8, 0, 1e-3)))


def append_geom2d_requests(requests: list[str]) -> None:
    """Append the Geom2dInt and Geom2dAPI requests in a fixed order."""
    append_tool_requests(requests)
    append_conic_curve_requests(requests)
    append_function_requests(requests)
    append_extrema_requests(requests)
    append_project_requests(requests)
    append_interpolate_requests(requests)
    append_points_to_bspline_requests(requests)
