"""Deterministic requests for the ExtremaPC2d package (tools/oracle/tkgeombase_probe_extrema_pc2d.cxx).

Every curve kind goes through the four ExtremaPC2d_Curve constructors (handle, handle
with a range, adaptor, adaptor with a range) with bounded, unbounded, degenerate,
reversed, whole-period and multi-period domains, and each evaluator answers a sequence
of queries in the three search modes, with and without endpoints: random points, the
degenerate configurations (center, foci, axis, vertex, points on the curve), invalid
tolerances and points at Precision::Infinite(). The grid evaluator, the distance
function, the uniform and curve-aware partitions, AddEndpointExtrema and the
GCPnts_TangentialDeflection sampling get direct requests.

NaN and infinities are not sent: the native reader (`std::istream >> double`) cannot
read them.
"""
from __future__ import annotations
import math
import random
from tkgeombase_corpus_gce import add, point2d, unit2d

INVALID_TOLS = [0.0, -1.0e-9]
FAR = 2.0e100  # Precision::IsInfinite


def ax2d(rng, scale=3.0):
    return [*point2d(rng, scale), *unit2d(rng)]


def conic_curves(rng):
    """Specs, a center-like point and some points on the curve's axes, per conic kind."""
    specs = []
    for _ in range(2):
        a = ax2d(rng)
        center = a[0:2]
        for sense in (1, 0):
            specs.append(([1, *a, sense, rng.uniform(0.5, 4.0)], center))
            specs.append(([2, *a, sense, 3.0, 1.25], center))
            specs.append(([2, *a, sense, 2.0, 2.0], center))
            specs.append(([3, *a, sense, 2.0, 1.0], center))
            specs.append(([4, *a, sense, rng.uniform(0.2, 2.0)], center))
        specs.append(([0, *a[0:2], *a[2:4]], center))
    specs.append(([1, 0.0, 0.0, 1.0, 0.0, 1, 0.0], [0.0, 0.0]))  # zero radius
    specs.append(([4, 0.0, 0.0, 1.0, 0.0, 1, 0.0], [0.0, 0.0]))  # zero focal
    specs.append(([9, 1.0, -2.0, 0.6, 0.8, 0.8, -0.6, 1.5], [1.0, -2.0]))  # indirect frame
    return specs


def spline_curves(rng):
    """Bezier, BSpline, offset, trimmed and sine-wave specs with a reference point."""
    specs = []
    for n in (3, 4, 6):
        poles = [c for _ in range(n) for c in (*point2d(rng, 4.0), rng.uniform(0.5, 2.0))]
        specs.append(([5, 0, n, *poles], point2d(rng, 2.0)))
        specs.append(([5, 1, n, *poles], point2d(rng, 2.0)))
    s = math.sqrt(0.5)
    quarter = [5, 1, 3, 1.0, 0.0, 1.0, 1.0, 1.0, s, 0.0, 1.0, 1.0]
    specs.append((quarter, [0.0, 0.0]))  # rational arc: constant distance from the center
    specs.append(([5, 0, 3, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0], [0.0, 0.0]))  # coincident poles
    specs.append(([5, 0, 2, -1.0, 0.0, 1.0, 3.0, 1.0, 1.0], [0.5, 2.0]))  # two poles
    cubic = [6, 0, 0, 3, 3, 0.0, 4, 1.0, 1, 2.0, 4, 5,
             0.0, 0.0, 1, 1.0, 2.0, 1, 2.0, -1.0, 1, 3.0, 2.0, 1, 4.0, 0.0, 1]
    specs.append((cubic, [2.0, 0.5]))
    periodic = [6, 0, 1, 3, 5, 0.0, 1, 1.0, 1, 2.0, 1, 3.0, 1, 4.0, 1, 4,
                2.0, 0.0, 1, 0.0, 2.0, 1, -2.0, 0.0, 1, 0.0, -2.0, 1]
    specs.append((periodic, [0.0, 0.0]))
    specs.append((periodic, [0.3, 3.0]))
    rational = [6, 1, 0, 2, 2, 0.0, 3, 1.0, 3, 3, 1.0, 0.0, 1.0, 1.0, 1.0, s, 0.0, 1.0, 1.0]
    specs.append((rational, [0.0, 0.0]))
    specs.append(([6, 0, 0, 1, 2, 0.0, 2, 1.0, 2, 2, 0.0, 0.0, 1, 3.0, 1.0, 1], [1.0, 2.0]))
    circle = [1, 0.5, -0.5, 1.0, 0.0, 1, 2.0]
    specs.append(([8, 0.75, 0, *circle], [0.5, -0.5]))
    specs.append(([8, -0.5, 0, *circle], [3.0, 1.0]))
    specs.append(([8, 0.3, 0, *cubic], [2.0, 0.5]))
    specs.append(([8, 0.4, 0, *quarter], [0.0, 0.0]))
    specs.append(([7, 0.5, 2.5, 1, 1, *circle], [0.5, -0.5]))
    specs.append(([7, 0.2, 0.8, 1, 1, *specs[0][0]], [0.0, 0.0]))
    specs.append(([7, -2.0, 3.0, 1, 1, 0, 0.0, 0.0, 0.6, 0.8], [1.0, 1.0]))
    specs.append(([14, 0.0, 0.0, 1.0, 0.0, 1.0, 2.0, 0.0], [0.0, 0.0]))
    specs.append(([14, 0.0, 0.0, 1.0, 0.0, 0.5, 1.0, 0.3], [1.0, 2.0]))
    return specs


def ranges(rng, kind):
    """(ctor, bounds) pairs: every constructor, and several domains for the ranged ones."""
    periodic = kind in (1, 2, 9)
    out = [(0, []), (2, [])]
    bounds = [(0.3, 2.1), (-1.0, 1.0), (0.5, 0.5), (1.0, 0.0), (-0.25, 0.75)]
    if periodic:
        bounds += [(0.0, 2 * math.pi), (1.0, 1.0 + 4 * math.pi), (-7.0, 9.5), (-math.pi, math.pi)]
    else:
        bounds += [(-5.0, 5.0), (-30.0, 40.0)]
    for b in bounds:
        out.append((1, list(b)))
        out.append((3, list(b)))
    return out


def queries(rng, center, tol_modes=True):
    """Special and random points, all modes, with and without endpoints."""
    pts = [center, [center[0] + 1.0, center[1]], [center[0], center[1] + 1.0]]
    pts += [point2d(rng, 6.0) for _ in range(4)]
    out = []
    for p in pts:
        for mode in (0, 1, 2):
            for ends in (0, 1):
                out.append([*p, 1.0e-9, mode, ends])
    out.append([*point2d(rng, 3.0), 1.0e-3, 0, 1])
    out.append([*point2d(rng, 3.0), 1.0e-6, 1, 0])
    if tol_modes:
        for tol in INVALID_TOLS:
            out.append([*center, tol, 0, 0])
            out.append([*center, tol, 0, 1])
        out.append([FAR, 0.0, 1.0e-9, 0, 0])
        out.append([0.0, -FAR, 1.0e-9, 0, 1])
    return out


def curve_requests(rng, requests):
    """pc2d_curve over every curve kind and constructor."""
    for spec, center in conic_curves(rng) + spline_curves(rng):
        for ctor, bounds in ranges(rng, spec[0]):
            q = queries(rng, center)
            add(requests, 'pc2d_curve', *spec, ctor, *bounds, len(q), *[x for item in q for x in item])


def grid_requests(rng, requests):
    """ExtremaPC2d_GridEvaluator with valid, short, unordered and uncovering partitions."""
    circle = [1, 0.0, 0.0, 1.0, 0.0, 1, 2.0]
    bezier = [5, 0, 4, 0.0, 0.0, 1, 1.0, 2.0, 1, 3.0, -2.0, 1, 4.0, 0.0, 1]
    for spec in (circle, bezier):
        dmax = 2 * math.pi if spec[0] == 1 else 1.0
        uniform = [i * dmax / 8 for i in range(9)]
        for params, dom in [(uniform, (0.0, dmax)), (uniform[:1], (0.0, dmax)),
                            (list(reversed(uniform)), (0.0, dmax)), (uniform, (0.0, dmax * 0.5)),
                            ([0.0, dmax], (0.0, dmax)), (uniform, (0.0, 0.0)), (uniform, (1.0, 0.0))]:
            q = queries(rng, [0.0, 0.0])
            add(requests, 'pc2d_grid', *spec, len(params), *params, *dom, len(q), *[x for item in q for x in item])
    for n, a, b in [(5, 0.0, 1.0), (2, -1.0, 3.0), (1, 0.0, 1.0), (0, 0.0, 1.0), (3, 1.0, 0.0),
                    (4, 0.5, 0.5), (64, -3.0, 7.0)]:
        add(requests, 'pc2d_uniform', a, b, n)


def distance_requests(rng, requests):
    """ExtremaPC2d_DistanceFunction, including singular parameters."""
    u = [0.0, 0.25, 0.5, 1.0, 1.7]
    for spec in ([1, 0.0, 0.0, 1.0, 0.0, 1, 2.0], [5, 0, 3, 0.0, 0.0, 1, 0.0, 0.0, 1, 2.0, 1.0, 1],
                 [5, 0, 3, 1.0, 1.0, 1, 1.0, 1.0, 1, 1.0, 1.0, 1], [0, 0.0, 0.0, 0.6, 0.8],
                 [3, 0.0, 0.0, 1.0, 0.0, 1, 2.0, 1.0]):
        for p in ([0.0, 0.0], point2d(rng, 3.0)):
            add(requests, 'pc2d_distance', *spec, *p, len(u), *u)


def endpoint_requests(rng, requests):
    """AddEndpointExtrema on line and circle adaptors."""
    line = [0, 0.0, 0.0, 1.0, 0.0]
    circle = [1, 0.0, 0.0, 1.0, 0.0, 1, 1.0]
    for spec in (line, circle):
        for dom in [(0.0, 10.0), (2.0, 2.0), (-FAR, 3.0), (-3.0, FAR), (-FAR, FAR), (1.0, 0.0), (0.0, 1.0e-12)]:
            for p in ([-2.0, 3.0], [5.0, 3.0], point2d(rng, 4.0)):
                for mode in (0, 1, 2):
                    for closed in (0, 1):
                        add(requests, 'pc2d_endpoints', *spec, *dom, *p, mode, closed)


def deflection_requests(rng, requests):
    """GCPnts_TangentialDeflection and Extrema_Curve2dTool::DeflCurvIntervals."""
    curves = [c for c, _ in spline_curves(rng)] + [[1, 0.0, 0.0, 1.0, 0.0, 1, 2.0], [0, 0.0, 0.0, 0.6, 0.8],
                                                    [2, 0.0, 0.0, 1.0, 0.0, 1, 3.0, 1.0]]
    for spec in curves:
        if spec[0] in (14,):
            add(requests, 'gcpnts_td2d', *spec, 1, 0.0, 6.0, math.pi / 6, 0.01, 2, 1.0e-9, 1.0e-7)
            continue
        for crit in [(math.pi / 6, 0.01, 2, 1.0e-9, 1.0e-7), (0.1, 0.001, 5, 1.0e-5, 1.0e-3),
                     (0.5, 0.5, 20, 1.0e-9, 0.0), (0.0, 0.01, 2, 1.0e-9, 1.0e-7), (0.2, 0.0, 2, 1.0e-9, 1.0e-7)]:
            add(requests, 'gcpnts_td2d', *spec, 0, *crit)
        add(requests, 'gcpnts_td2d', *spec, 1, 0.8, 0.1, 0.2, 0.05, 3, 1.0e-9, 1.0e-7)


def append_extrema_pc2d_requests(requests: list[str]) -> None:
    """Append every ExtremaPC2d request family in a fixed order."""
    rng = random.Random(20250928)
    curve_requests(rng, requests)
    grid_requests(rng, requests)
    distance_requests(rng, requests)
    endpoint_requests(rng, requests)
    deflection_requests(rng, requests)
