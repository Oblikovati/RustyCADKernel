"""Deterministic requests for the ExtremaPC package (tools/oracle/tkgeombase_probe_extrema_pc.cxx).

Every 3D curve kind goes through the six ExtremaPC_Curve constructors (Geom handle,
GeomAdaptor_Curve and GeomAdaptor_TransformedCurve, each with and without a range) with
bounded, unbounded, degenerate, reversed, whole-period, multi-period and unrepresentable
(1e17) domains; the transformations are the identity, translations, rotations and
positive and negative scalings. The conics are placed in random frames and queried at
their centers, foci, axes (above and below the plane) and random points, so both the
planar 2D path and the direct 3D fallback run. Bezier, BSpline and offset curves are
planar (exact pole planes, collinear and coincident poles) and non-planar, rational and
not. The grid evaluator, the distance function, AddEndpointExtrema, the planar
projections and GCPnts_TangentialDeflection get direct requests.
"""
from __future__ import annotations
import math
import random
from tkg3d_corpus import frame, point
from tkgeombase_corpus_gce import add

INVALID_TOLS = [0.0, -1.0e-9]
FAR = 2.0e100
TRSFS = [[0, 0, 0, 0, 0, 0, 0, 0], [1, 1.5, -2.0, 0.5, 0, 0, 0, 0],
         [2, 0.3, 0.1, -0.2, 0.0, 0.6, 0.8, 0.7], [3, 1.0, 1.0, 1.0, 2.5, 0, 0, 0],
         [3, 0.0, 0.0, 0.0, -0.5, 0, 0, 0], [4, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.2]]


def axis_points(f, rng):
    """The center, points on the normal on both sides, and in-plane points."""
    p, n, x = f[0:3], f[3:6], f[6:9]
    out = [p, [p[i] + 2.0 * n[i] for i in range(3)], [p[i] - 0.5 * n[i] for i in range(3)]]
    out.append([p[i] + 1.5 * x[i] for i in range(3)])
    out.append([p[i] + 0.7 * x[i] + 3.0 * n[i] for i in range(3)])
    return out


def conic_curves(rng):
    """(spec, special points) for every conic kind in random frames, with degenerate ones."""
    specs = []
    for _ in range(2):
        f = frame(rng, 3.0)
        pts = axis_points(f, rng)
        specs += [([1, *f, rng.uniform(0.5, 4.0)], pts), ([2, *f, 3.0, 1.25], pts), ([2, *f, 2.0, 2.0], pts),
                  ([3, *f, 2.0, 1.0], pts), ([4, *f, rng.uniform(0.2, 2.0)], pts), ([0, *f[0:3], *f[3:6]], pts)]
    f = [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0]
    pts = axis_points(f, rng)
    specs += [([1, *f, 0.0], pts), ([4, *f, 0.0], pts), ([1, *f, 5.0], pts), ([0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0], pts)]
    return specs


def spline_curves(rng):
    """(spec, reference points) for Bezier, BSpline, offset and trimmed curves."""
    specs = []
    for n in (3, 5):
        planar = [c for _ in range(n) for c in (*point(rng, 4.0)[0:2], 0.0, rng.uniform(0.5, 2.0))]
        spatial = [c for _ in range(n) for c in (*point(rng, 4.0), rng.uniform(0.5, 2.0))]
        refs = [point(rng, 3.0), [0.0, 0.0, 2.0]]
        specs += [([5, 0, n, *planar], refs), ([5, 1, n, *planar], refs), ([5, 0, n, *spatial], refs),
                  ([5, 1, n, *spatial], refs)]
    s = math.sqrt(0.5)
    quarter = [5, 1, 3, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0, s, 0.0, 1.0, 0.0, 1.0]
    specs.append((quarter, [[0.0, 0.0, 0.0], [0.0, 0.0, 3.0]]))
    specs.append(([5, 0, 3, 1.0, 2.0, 3.0, 1, 1.0, 2.0, 3.0, 1, 1.0, 2.0, 3.0, 1], [[0.0, 0.0, 0.0]]))
    specs.append(([5, 0, 3, 0.0, 0.0, 0.0, 1, 1.0, 1.0, 1.0, 1, 2.0, 2.0, 2.0, 1], [[0.0, 1.0, 0.0]]))
    specs.append(([5, 0, 2, 0.0, 0.0, 0.0, 1, 0.0, 0.0, 3.0, 1], [[1.0, 0.0, 1.0]]))
    cubic = [6, 0, 0, 3, 3, 0.0, 4, 1.0, 1, 2.0, 4, 5, 0.0, 0.0, 0.0, 1, 1.0, 2.0, 0.0, 1, 2.0, -1.0, 0.0, 1,
             3.0, 2.0, 0.0, 1, 4.0, 0.0, 0.0, 1]
    twisted = [6, 0, 0, 3, 3, 0.0, 4, 1.0, 1, 2.0, 4, 5, 0.0, 0.0, 0.0, 1, 1.0, 2.0, 1.0, 1, 2.0, -1.0, -1.0, 1,
               3.0, 2.0, 0.5, 1, 4.0, 0.0, 0.0, 1]
    periodic = [6, 0, 1, 3, 5, 0.0, 1, 1.0, 1, 2.0, 1, 3.0, 1, 4.0, 1, 4, 2.0, 0.0, 0.0, 1, 0.0, 2.0, 0.0, 1,
                -2.0, 0.0, 0.0, 1, 0.0, -2.0, 0.0, 1]
    rational = [6, 1, 0, 2, 2, 0.0, 3, 1.0, 3, 3, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0, s, 0.0, 1.0, 0.0, 1.0]
    specs += [(cubic, [[2.0, 0.5, 0.0], [2.0, 0.5, 1.5]]), (twisted, [[2.0, 0.5, 0.3]]),
              (periodic, [[0.0, 0.0, 0.0], [0.3, 3.0, -1.0]]), (rational, [[0.0, 0.0, 0.0], [0.0, 0.0, -2.0]])]
    circle = [1, 0.5, -0.5, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 2.0]
    specs += [([8, 0.75, 0.0, 0.0, 1.0, *circle], [[0.5, -0.5, 0.0], [3.0, 1.0, 1.0]]),
              ([8, -0.5, 0.0, 0.0, 1.0, *circle], [[3.0, 1.0, 0.0]]),
              ([8, 0.5, 1.0, 0.0, 0.0, *circle], [[3.0, 1.0, 0.5]]),
              ([8, 0.3, 0.0, 0.0, 1.0, *cubic], [[2.0, 0.5, 0.0], [2.0, 0.5, 1.0]]),
              ([8, 0.3, 0.0, 0.0, 1.0, *twisted], [[2.0, 0.5, 0.0]]),
              ([8, 0.4, 0.0, 0.0, 1.0, 7, 0.5, 2.5, 1, 1, *circle], [[0.5, -0.5, 0.0]]),
              ([8, 0.4, 0.0, 0.0, 1.0, 0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0], [[1.0, 1.0, 0.0]]),
              ([7, 0.5, 2.5, 1, 1, *circle], [[0.5, -0.5, 0.0]]),
              ([7, -2.0, 3.0, 1, 1, 0, 0.0, 0.0, 0.0, 0.6, 0.8, 0.0], [[1.0, 1.0, 1.0]])]
    return specs


def ranges(kind):
    """(ctor, trsf, bounds) triples over the six constructors."""
    periodic = kind == 1 or kind == 2
    bounds = [(0.3, 2.1), (-1.0, 1.0), (0.5, 0.5), (1.0, 0.0), (1.0e17, 1.0e17 + 100.0)]
    bounds += [(0.0, 2 * math.pi), (1.0, 1.0 + 4 * math.pi), (-7.0, 9.5)] if periodic else [(-5.0, 5.0)]
    out = [(0, [], []), (2, [], [])]
    for b in bounds:
        out += [(1, [], list(b)), (3, [], list(b))]
    for t in TRSFS:
        out.append((4, t, []))
    for t in TRSFS[1:4]:
        out.append((5, t, [0.3, 2.1]))
    return out


def queries(rng, special):
    """The special points and random points in the three modes, with and without ends."""
    pts = special + [point(rng, 6.0) for _ in range(3)]
    out = []
    for p in pts:
        for mode in (0, 1, 2):
            for ends in (0, 1):
                out.append([*p, 1.0e-9, mode, ends])
    out.append([*point(rng, 3.0), 1.0e-3, 0, 1])
    for tol in INVALID_TOLS:
        out.append([*special[0], tol, 0, 0])
        out.append([*special[0], tol, 0, 1])
    out.append([FAR, 0.0, 0.0, 1.0e-9, 0, 0])
    return out


def curve_requests(rng, requests):
    for spec, special in conic_curves(rng) + spline_curves(rng):
        for ctor, t, bounds in ranges(spec[0]):
            q = queries(rng, special)
            add(requests, 'pc_curve', *spec, ctor, *t, *bounds, len(q), *[x for item in q for x in item])


def direct_requests(rng, requests):
    """Grid evaluator, distance function, AddEndpointExtrema, projections, deflection."""
    circle = [1, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 2.0]
    bezier = [5, 0, 4, 0.0, 0.0, 0.0, 1, 1.0, 2.0, 1.0, 1, 3.0, -2.0, -1.0, 1, 4.0, 0.0, 0.0, 1]
    for spec, dmax in ((circle, 2 * math.pi), (bezier, 1.0)):
        uniform = [i * dmax / 8 for i in range(9)]
        for params, dom in [(uniform, (0.0, dmax)), (uniform[:1], (0.0, dmax)), (list(reversed(uniform)), (0.0, dmax)),
                            (uniform, (0.0, dmax * 0.5)), (uniform, (0.0, 0.0))]:
            q = queries(rng, [[0.0, 0.0, 0.0], [0.0, 0.0, 1.0]])
            add(requests, 'pc_grid', *spec, len(params), *params, *dom, len(q), *[x for item in q for x in item])
        u = [0.0, 0.25, 0.5, 1.0, 1.7]
        for p in ([0.0, 0.0, 0.0], point(rng, 3.0)):
            add(requests, 'pc_distance', *spec, *p, len(u), *u)
    line = [0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0]
    for spec in (line, circle):
        for dom in [(0.0, 10.0), (2.0, 2.0), (-FAR, 3.0), (-3.0, FAR), (1.0, 0.0), (0.0, 1.0e-12)]:
            for p in ([-2.0, 3.0, 1.0], point(rng, 4.0)):
                for mode in (0, 1, 2):
                    for closed in (0, 1):
                        add(requests, 'pc_endpoints', *spec, *dom, *p, mode, closed)
    u = [0.0, 0.3, 0.5, 1.0, 2.0]
    for spec, _ in spline_curves(rng) + [([0, *point(rng), 0.0, 0.6, 0.8], None), ([0, *point(rng), 0.1, 0.0, 0.99], None)]:
        add(requests, 'pc_project', *spec, len(u), *u)
        if spec[0] in (5, 6, 8):
            for crit in [(math.pi / 6, 0.01, 2, 1.0e-9, 1.0e-7), (0.1, 0.001, 5, 1.0e-5, 1.0e-3), (0.0, 0.01, 2, 1e-9, 1e-7)]:
                add(requests, 'gcpnts_td3d', *spec, 0, *crit)
            add(requests, 'gcpnts_td3d', *spec, 1, 0.2, 0.8, 0.2, 0.05, 3, 1.0e-9, 1.0e-7)


def append_extrema_pc_requests(requests: list[str]) -> None:
    """Append every ExtremaPC request family in a fixed order."""
    rng = random.Random(20250929)
    curve_requests(rng, requests)
    direct_requests(rng, requests)
