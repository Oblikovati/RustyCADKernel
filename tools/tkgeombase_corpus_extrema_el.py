"""Deterministic requests for the Extrema closed forms (tools/oracle/tkgeombase_probe_extrema_el.cxx).

ExtPElC and ExtPElC2d run every conic in random and axis-aligned frames against random
points, centres, points on the axis and on the curve, with bounded, reversed,
periodic-offset (whole and multiple turns), single-point and infinite ranges and null
tolerances (the circle directions then raise). ExtElC and ExtElC2d pair lines with
every conic in skew, intersecting, parallel, antiparallel, nearly parallel (1e-13 rad),
coplanar, axial and tangent configurations, and circles that are apart, secant,
tangent, nested, concentric and not coplanar. ExtPElS projects on every elementary
surface from random points, the axis, the apex, the centre and the surface itself.
ExtElCS and ExtElSS run sequences of performs on one object, so the stale result
arrays and the Standard_NotImplemented overloads are compared too.
"""
from __future__ import annotations
import math
import random
from tkg3d_corpus import frame, point, unit
from tkgeombase_corpus_gce import add

INF = 2.0e100
TWO_PI = 2.0 * math.pi
Z_FRAME = [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0]


def frame2d(rng, scale=4.0, direct=True):
    """A 2D frame P X Y (gp_Ax22d input), direct or indirect."""
    a = rng.uniform(-math.pi, math.pi)
    x = [math.cos(a), math.sin(a)]
    y = [-x[1], x[0]] if direct else [x[1], -x[0]]
    return [rng.uniform(-scale, scale), rng.uniform(-scale, scale), *x, *y]


def along(f, dist, height=0.0):
    """The point of frame `f` at `dist` along X and `height` along N."""
    return [f[i] + dist * f[6 + i] + height * f[3 + i] for i in range(3)]


def pelc_ranges(periodic):
    """Parameter ranges: bounded, reversed, single point, infinite, and turns."""
    ranges = [(-1.0, 1.0), (0.5, 0.25), (0.3, 0.3), (-INF, INF), (-3.0, 7.0)]
    if periodic:
        ranges += [(0.0, TWO_PI), (math.pi, 3.0 * math.pi), (-math.pi, math.pi), (1.0, 3.0),
                   (5.0, 5.0 + TWO_PI), (0.0, 4.0 * math.pi)]
    return ranges


def pelc_queries(rng, f, periodic, pts):
    """(point, tol, uinf, usup) queries around frame `f`."""
    out = []
    for p in pts:
        for (u1, u2) in pelc_ranges(periodic):
            out.append([*p, 1.0e-7, u1, u2])
    out.append([*pts[0], 0.0, 0.0, TWO_PI])
    out.append([*pts[1], 1.0e-3, -1.0, 1.0])
    return out


def pelc_requests(requests, rng):
    """ExtPElC on every 3D conic."""
    frames = [frame(rng, 3.0) for _ in range(3)] + [Z_FRAME]
    for f in frames:
        pts = [f[0:3], along(f, 0.0, 2.0), along(f, 1.7, 0.4), point(rng, 5.0), along(f, -2.0, -1.0),
               along(f, 2.5), [INF, 0.0, 0.0]]
        conics = [[1, *f, 2.0], [1, *f, 0.0], [2, *f, 3.0, 1.5], [2, *f, 2.0, 2.0], [3, *f, 2.0, 1.0],
                  [4, *f, 0.75], [0, *f[0:3], *f[3:6]], [0, *f[0:3], *f[6:9]]]
        for c in conics:
            queries = pelc_queries(rng, f, c[0] in (1, 2), pts)
            add(requests, 'ex_pelc', *c, len(queries), *[v for q in queries for v in q])
    circle = [1, *Z_FRAME, 1.0]
    near_pi = [[-1.0, 1.0e-17, 0.0, 1.0e-7, 0.0, TWO_PI], [-1.0, -1.0e-17, 0.0, 1.0e-7, 0.0, TWO_PI],
               [1.0, -1.0e-13, 0.0, 1.0e-7, 0.0, TWO_PI], [1.0, 1.0e-9, 0.0, 1.0e-7, 1.0e-9, TWO_PI],
               [1.0, 0.0, 0.0, 10.0, 0.0, 1.0], [3.0, 0.0, 0.0, 1.0e-7, TWO_PI, 3.0 * math.pi]]
    add(requests, 'ex_pelc', *circle, len(near_pi), *[v for q in near_pi for v in q])


def pelc2d_requests(requests, rng):
    """ExtPElC2d on every 2D conic, direct and indirect frames."""
    for direct in (True, False, True):
        f = frame2d(rng, 3.0, direct)
        c0 = f[0:2]
        pts = [c0, [c0[0] + 1.3 * f[2], c0[1] + 1.3 * f[3]], [rng.uniform(-5, 5), rng.uniform(-5, 5)],
               [c0[0] + 2.0 * f[4], c0[1] + 2.0 * f[5]], [INF, 1.0]]
        conics = [[1, *f, 2.0], [2, *f, 3.0, 1.5], [2, *f, 2.0, 2.0], [3, *f, 2.0, 1.0], [4, *f, 0.75],
                  [0, *c0, *f[2:4]]]
        for c in conics:
            queries = []
            for p in pts:
                for (u1, u2) in pelc_ranges(c[0] in (1, 2)):
                    queries.append([*p, 1.0e-7, u1, u2])
            queries.append([*pts[1], 0.5, 0.0, TWO_PI])
            add(requests, 'ex_pelc2d', *c, len(queries), *[v for q in queries for v in q])


def elc_requests(requests, rng):
    """ExtElC: lines with lines and conics, and circle pairs."""
    for _ in range(3):
        f = frame(rng, 3.0)
        g = frame(rng, 3.0)
        line = [*point(rng, 3.0), *unit(rng)]
        add(requests, 'ex_elc', 0, *line, *g, 1.0e-10)
        add(requests, 'ex_elc', 0, *line, *line[0:3], *line[3:6], *g[6:9], 1.0e-10)
        add(requests, 'ex_elc', 0, *line, 1.0, 2.0, 3.0, *[-c for c in line[3:6]], *g[6:9], 1.0e-10)
        add(requests, 'ex_elc', 1, *line, *f, 2.0, 1.0e-10)
        add(requests, 'ex_elc', 1, *along(f, 0.5), *f[6:9], *f, 2.0, 1.0e-10)
        add(requests, 'ex_elc', 1, *along(f, 0.5, 0.7), *f[6:9], *f, 2.0, 1.0e-10)
        add(requests, 'ex_elc', 1, *f[0:3], *f[3:6], *f, 2.0, 1.0e-10)
        add(requests, 'ex_elc', 1, *along(f, 3.0), *f[3:6], *f, 2.0, 1.0e-10)
        add(requests, 'ex_elc', 1, *along(f, 2.0), *[f[9 - 3 + i] for i in range(3)], *f, 2.0, 1.0e-10)
        add(requests, 'ex_elc', 2, *line, *f, 3.0, 1.5)
        add(requests, 'ex_elc', 2, *f[0:3], *f[3:6], *f, 3.0, 1.5)
        add(requests, 'ex_elc', 2, *along(f, 0.5, 1.0), *f[6:9], *f, 3.0, 1.5)
        add(requests, 'ex_elc', 3, *line, *f, 2.0, 1.0)
        add(requests, 'ex_elc', 3, *along(f, 0.0, 1.0), *f[6:9], *f, 2.0, 1.0)
        add(requests, 'ex_elc', 4, *line, *f, 0.75)
        add(requests, 'ex_elc', 4, *along(f, 0.3, 1.0), *f[6:9], *f, 0.75)
    ax = [0.0, 0.0, 1.0]
    add(requests, 'ex_elc', 0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, math.cos(1e-13), math.sin(1e-13), 0.0,
        0.0, 0.0, 1.0, 1.0e-10)
    add(requests, 'ex_elc', 0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0)
    add(requests, 'ex_elc', 1, 0.0, 0.0, 5.0, 1.0, 1.0e-13, 0.0, *Z_FRAME, 1.0, 0.0)
    add(requests, 'ex_elc', 1, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, *Z_FRAME, 1.0, 0.0)
    add(requests, 'ex_elc', 1, 0.0, 0.0, -2.0, *ax, *Z_FRAME, 1.0, 0.0)
    add(requests, 'ex_elc', 2, 0.0, 0.0, -2.0, *ax, *Z_FRAME, 2.0, 2.0)
    add(requests, 'ex_elc', 2, 0.0, 0.0, -2.0, *ax, *Z_FRAME, 3.0, 1.0)
    add(requests, 'ex_elc', 3, 0.0, 5.0, 0.0, 1.0, 0.0, 0.0, *Z_FRAME, 2.0, 1.0)
    add(requests, 'ex_elc', 4, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, *Z_FRAME, 0.5)
    circles = [(1.0, 2.0, 0.0, 3.0), (1.0, 2.0, 0.0, 2.0), (1.0, 2.0, 0.0, 1.0), (1.0, 3.0, 0.0, 2.0),
               (1.0, 3.0, 0.0, 4.0), (2.0, 1.0, 0.0, 0.5), (1.0, 1.0, 0.0, 0.0), (1.0, 2.0, 0.0, 1.0e-9),
               (1.0, 2.0, 1.0e-3, 1.0), (1.0, 2.0, 1.0e-9, 1.5), (1.0, 1.0, 0.0, 1.0)]
    for r1, r2, dz, dx in circles:
        add(requests, 'ex_elc', 5, *Z_FRAME, r1, dx, 0.3 * dx, dz, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, r2)
    add(requests, 'ex_elc', 5, *Z_FRAME, 1.0, 0.5, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0)
    for _ in range(2):
        f = frame(rng, 2.0)
        g = [*along(f, 1.2), *f[3:6], *unit(rng)]
        if abs(sum(g[3 + i] * g[6 + i] for i in range(3))) < 0.95:
            add(requests, 'ex_elc', 5, *f, 1.5, *g, 1.0)


def elc2d_requests(requests, rng):
    """ExtElC2d: lines with lines and conics, circles with conics."""
    for direct in (True, False):
        f = frame2d(rng, 2.0, direct)
        g = frame2d(rng, 2.0, not direct)
        line = [rng.uniform(-3, 3), rng.uniform(-3, 3), *g[2:4]]
        axial = [*f[0:2], *f[2:4]]
        across = [*f[0:2], *f[4:6]]
        add(requests, 'ex_elc2d', 0, *line, *axial, 1.0e-10)
        add(requests, 'ex_elc2d', 0, *line, line[0] + 1.0, line[1], *line[2:4], 1.0e-10)
        add(requests, 'ex_elc2d', 0, *line, line[0] + 1.0, line[1], -line[2], -line[3], 1.0e-10)
        for l in (line, axial, across):
            add(requests, 'ex_elc2d', 1, *l, *f, 2.0, 1.0e-10)
            add(requests, 'ex_elc2d', 2, *l, *f, 3.0, 1.5)
            add(requests, 'ex_elc2d', 3, *l, *f, 2.0, 1.0)
            add(requests, 'ex_elc2d', 4, *l, *f, 0.75)
        circle = [*g, 1.0]
        add(requests, 'ex_elc2d', 5, *circle, *f, 2.0)
        add(requests, 'ex_elc2d', 5, *circle, *g, 2.0)
        add(requests, 'ex_elc2d', 6, *circle, *f, 3.0, 1.5)
        add(requests, 'ex_elc2d', 6, *circle, *g, 3.0, 1.5)
        add(requests, 'ex_elc2d', 7, *circle, *f, 2.0, 1.0)
        add(requests, 'ex_elc2d', 8, *circle, *f, 0.75)
        add(requests, 'ex_elc2d', 8, *[*f[0:2], *g[2:6]], 1.0, *f, 0.75)
    add(requests, 'ex_elc2d', 5, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0e-9, 0.0, 1.0, 0.0, 0.0, 1.0, 2.0)
    add(requests, 'ex_elc2d', 1, 0.0, 5.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 2.0, 0.0)
    add(requests, 'ex_elc2d', 3, 0.0, 5.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 2.0, 1.0)
    add(requests, 'ex_elc2d', 3, 0.0, 5.0, 2.0 / math.sqrt(5.0), 1.0 / math.sqrt(5.0), 0.0, 0.0, 1.0, 0.0, 0.0, 1.0,
        2.0, 1.0)
    add(requests, 'ex_elc2d', 4, 0.0, 5.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.5)


def surface_specs(rng):
    """(spec, special points) for every elementary surface."""
    specs = []
    for f in [frame(rng, 3.0) for _ in range(2)] + [Z_FRAME]:
        c = f[0:3]
        pts = [c, along(f, 0.0, 2.0), along(f, 1.3, 0.5), point(rng, 5.0), along(f, 2.0, -1.0)]
        specs += [([0, *f], pts), ([1, *f, 1.5], pts + [along(f, 1.5, 0.3)]),
                  ([2, *f, 0.4, 1.0], pts + [along(f, -1.0 / math.tan(0.4) * 0.0, -1.0 / math.tan(0.4))]),
                  ([2, *f, -0.4, 0.0], pts), ([2, *f, 0.3, 0.0], pts + [along(f, 1.0, -3.0)]),
                  ([3, *f, 2.0], pts + [along(f, 2.0)]), ([4, *f, 3.0, 1.0], pts + [along(f, 3.0), along(f, 4.0, 0.2)]),
                  ([4, *f, 1.0, 2.0], pts)]
    return specs


def pels_requests(requests, rng):
    """ExtPElS on every elementary surface."""
    for spec, pts in surface_specs(rng):
        queries = [[*p, tol] for p in pts for tol in (1.0e-7, 0.0)]
        queries.append([*pts[1], 5.0])
        add(requests, 'ex_pels', *spec, len(queries), *[v for q in queries for v in q])


def elcs_steps(rng):
    """ExtElCS steps: every curve-surface pair in general and special positions."""
    f = frame(rng, 2.0)
    lines = [[0, *point(rng, 3.0), *unit(rng)], [0, *along(f, 3.0), *f[3:6]], [0, *along(f, 0.5), *f[6:9]],
             [0, *along(f, 0.0, 1.0), *f[6:9]], [0, *along(f, 5.0), *f[6:9]], [0, *f[0:3], *f[3:6]]]
    circles = [[1, *frame(rng, 2.0), 1.0], [1, *along(f, 0.0, 1.0), *f[3:6], *f[6:9], 0.8],
               [1, *along(f, 0.0, 1.0), *f[3:6], *f[6:9], 2.5], [1, *along(f, 1.0), *f[6:9], *f[3:6], 1.0],
               [1, *along(f, 0.3, 0.2), *f[3:6], *f[6:9], 1.5], [1, *along(f, 0.0, 3.0), *f[6:9], *f[3:6], 0.5]]
    hyperbolas = [[2, *frame(rng, 2.0), 2.0, 1.0], [2, *along(f, 0.0, 1.0), *f[3:6], *f[6:9], 2.0, 1.0],
                  [2, *along(f, 0.0, 1.0), *f[6:9], *f[3:6], 2.0, 1.0]]
    surfaces = [[0, *f], [1, *f, 1.5], [2, *f, 0.4, 1.0], [3, *f, 2.0], [4, *f, 3.0, 1.0]]
    steps = []
    for curve in lines + circles:
        for surface in surfaces:
            steps.append([curve[0] * 5 + surface[0], *curve[1:], *surface[1:]])
    for h in hyperbolas:
        steps.append([10, *h[1:], *surfaces[0][1:]])
    return steps


def elcs_requests(requests, rng):
    """ExtElCS sequences on one object (fresh object per chunk)."""
    steps = elcs_steps(rng)
    for start in range(0, len(steps), 5):
        chunk = steps[start:start + 5]
        add(requests, 'ex_elcs', len(chunk), *[v for s in chunk for v in s])
    add(requests, 'ex_elcs', 3, *steps[1], *steps[0], *steps[5])


def elss_requests(requests, rng):
    """ExtElSS sequences: parallel, secant and NotImplemented pairs."""
    f = frame(rng, 2.0)
    g = frame(rng, 2.0)
    parallel = [*along(f, 0.0, 3.0), *[-c for c in f[3:6]], *f[6:9]]
    steps = [[0, *f, *g], [0, *f, *parallel], [0, *f, *f], [1, *f, *g, 1.0], [2, *f, 1.0, *g, 2.0],
             [0, *f, *parallel], [3, *f, 1.0, *g, 0.5], [4, *f, 1.0, *g, 0.5, 1.0], [5, *f, 1.0, *g, 3.0, 1.0]]
    add(requests, 'ex_elss', len(steps), *[v for s in steps for v in s])


def append_extrema_el_requests(requests: list[str]) -> None:
    """Append the elementary Extrema requests (seed 20260929)."""
    rng = random.Random(20260929)
    add(requests, 'ex_pon', 0.5, -1.25, 1.0, 2.0, 3.0, 4.5, 3, 7, -2)
    add(requests, 'ex_pon', -0.0, INF, 0.0, -0.0, 1.0e-300, 0.0, 1, 0, 0)
    pelc_requests(requests, rng)
    pelc2d_requests(requests, rng)
    elc_requests(requests, rng)
    elc2d_requests(requests, rng)
    pels_requests(requests, rng)
    elcs_requests(requests, rng)
    elss_requests(requests, rng)
