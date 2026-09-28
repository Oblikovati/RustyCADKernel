"""Deterministic requests for LProp, GeomLProp, GProp and Hermit
(tools/oracle/tkgeombase_probe_lprop.cxx, _gprop.cxx, _hermit.cxx).

The curve and surface specs follow tkgeombase_probe_lprop_build.cxx: a kind code then
its data. Every kind is probed at regular parameters, at the ends of its range, beyond
it, and at singular points: null first derivatives (coincident Bezier poles, a cusp),
inflections, straight parts (null curvature), multiple B-spline knots (C0/C1 joints),
sphere poles and cone apices (degenerate normals), umbilics (sphere, plane) and saddles.
Each property class runs in every call-order mode of the probe, so the lazy caching and
the native by-value reads are compared, not only the final values.
"""
from __future__ import annotations
import math
import random
from tkg3d_corpus import frame, point, unit
from tkgeombase_corpus_gce import add, point2d, unit2d

PI = math.pi


def ax2d(rng):
    return [*point2d(rng, 3.0), *unit2d(rng)]


def curves2d(rng):
    """(name, spec, parameters) of every 2D curve kind, with singular parameters."""
    a = ax2d(rng)
    s_bezier = [5, 4, 0, 0, 1, 1, 3, 1, 1, -2, 2, 4, 0, 1]
    cusp = [5, 4, 0, 0, 1, 0, 0, 1, 2, 1, 1.5, 3, 0.5, 1]
    rational = [5, 3, 0, 0, 1, 1, 2, 3, 2, 0, 1]
    bspline_c2 = [6, 3, 0, 5, 0, 0, 1, 1, 2, 1, 2, -1, 1, 3, 1, 2, 4, 0, 1, 3, 0, 4, 1, 1, 2, 4]
    bspline_c1 = [6, 3, 0, 7, 0, 0, 1, 1, 2, 1, 2, -1, 1, 3, 0.5, 1, 4, 2, 2, 5, -1, 1, 6, 1, 1,
                  4, 0, 4, 1, 2, 2, 1, 3, 4]
    one_span = [6, 3, 0, 4, 0, 0, 1, 1, 2, 1, 2, -2, 1.5, 3, 0, 1, 2, 0, 4, 1, 4]
    bspline_c0 = [6, 2, 0, 5, 0, 0, 1, 1, 1, 1, 2, 0, 1, 3, 1, 2, 4, 0, 1, 3, 0, 3, 1, 2, 2, 3]
    return [
        ('line', [0, *point2d(rng), *unit2d(rng)], [0.0, 2.5, -7.0]),
        ('circle', [1, *a, 1, 2.5], [0.0, PI / 2, 1.0, 2 * PI, -0.3]),
        ('circle_indirect', [1, *a, 0, 2.5], [0.0, 3.0]),
        ('ellipse', [2, *a, 1, 10.0, 3.0], [0.0, PI / 2, PI, 3 * PI / 2, 0.7]),
        ('hyperbola', [3, *a, 1, 6.0, 3.0], [0.0, 0.5, -1.2]),
        ('parabola', [4, *a, 1, 2.0], [0.0, 1.0, -3.0]),
        ('bezier_s', s_bezier, [0.0, 0.25, 0.5, 0.75, 1.0]),
        ('bezier_cusp', cusp, [0.0, 0.5, 1.0]),
        ('bezier_rational', rational, [0.0, 0.3, 1.0]),
        ('bspline_c2', bspline_c2, [0.0, 0.4, 1.0, 1.5, 2.0]),
        ('bspline_c1', bspline_c1, [0.0, 1.0, 1.5, 2.0, 3.0]),
        ('bspline_c0', bspline_c0, [0.0, 1.0, 2.0]),
        ('trimmed_ellipse', [7, 0.5, 4.0, 2, *a, 1, 10.0, 3.0], [0.5, 1.0, 4.0]),
        ('trimmed_s', [7, 0.1, 0.9, *s_bezier], [0.1, 0.5, 0.9]),
        ('offset_ellipse', [8, 0.5, 2, *a, 1, 10.0, 3.0], [0.0, 1.0, PI / 2]),
        ('offset_circle_collapse', [8, -2.5, 1, *a, 1, 2.5], [0.0, 1.0]),
        ('bspline_one_span', one_span, [0.0, 0.5, 1.0]),
    ]


def to3d(spec2d):
    """The planar 3D twin of a Bezier or B-spline 2D spec (z = 0)."""
    kind = spec2d[0]
    head = [kind] + (spec2d[1:3] if kind == 6 else [])
    body = spec2d[len(head):]
    n = body[0]
    poles = []
    for i in range(n):
        x, y, w = body[1 + 3 * i: 4 + 3 * i]
        poles += [x, y, 0.3 * x, w]
    return head + [n] + poles + body[1 + 3 * n:]


def curves3d(rng):
    f = frame(rng, 3.0)
    base = {name: spec for name, spec, _ in curves2d(rng)}
    return [
        ('line', [0, *point(rng), *unit(rng)], [0.0, 1.5]),
        ('circle', [1, *f, 2.5], [0.0, 1.0, 2 * PI]),
        ('ellipse', [2, *f, 4.0, 1.0], [0.0, PI / 2, 2.0]),
        ('hyperbola', [3, *f, 2.0, 1.0], [0.0, 1.1]),
        ('parabola', [4, *f, 0.5], [0.0, 2.0]),
        ('bezier_s', to3d(base['bezier_s']), [0.0, 0.5, 1.0]),
        ('bezier_cusp', to3d(base['bezier_cusp']), [0.0, 0.5]),
        ('bspline_c1', to3d(base['bspline_c1']), [0.0, 2.0, 2.5]),
        ('helix_like', [6, 3, 0, 5, 1, 0, 0, 1, 0, 1, 0.5, 1, -1, 0, 1, 1, 0, -1, 1.5, 1, 1, 0, 2, 1,
                        3, 0, 4, 1, 1, 2, 4], [0.0, 0.3, 1.0]),
        ('trimmed_circle', [7, 0.5, 2.0, 1, *f, 2.5], [0.5, 2.0]),
        ('offset_circle', [8, 0.75, 0, 0, 1, 1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 2.0], [0.0, 1.0]),
    ]


def clprops_requests(rng, requests):
    for name, spec, params in curves2d(rng):
        for mode in range(4):
            add(requests, 'lprop_cl2d', *spec, mode, 3, 1e-7, len(params), *params)
        for n in (0, 1, 2, -1, 4):
            add(requests, 'lprop_cl2d', *spec, 0, n, 1e-7, len(params), *params)
        add(requests, 'lprop_cl2d', *spec, 0, 2, 0.5, len(params), *params)
    for name, spec, params in curves3d(rng):
        for op in ('lprop_cl3d', 'lprop_cl3d_adaptor'):
            for mode in range(4):
                add(requests, op, *spec, mode, 2, 1e-7, len(params), *params)
            add(requests, op, *spec, 0, 1, 1e-7, len(params), *params)
    # A bad curve: the construction error is printed instead of the properties.
    add(requests, 'lprop_cl2d', 1, *ax2d(rng), 1, -1.0, 0, 2, 1e-7, 1, 0.0)


def surfaces(rng):
    f = frame(rng, 3.0)
    bspline = [5, 2, 2, 3, 3]
    for i in range(3):
        for j in range(3):
            bspline += [i, j, 0.3 * ((i - 1) ** 2 - (j - 1) ** 2), 1.0 + 0.5 * (i == 1 and j == 1)]
    bspline += [2, 0, 3, 1, 3, 2, 0, 3, 1, 3]
    collapsed = [5, 1, 2, 2, 3]
    for i in range(2):
        for j in range(3):
            collapsed += ([0, 0, 0] if i == 0 else [j - 1, 1, 0.2 * j]) + [1.0]
    collapsed += [2, 0, 2, 1, 2, 2, 0, 3, 1, 3]
    s_curve = to3d(curves2d(rng)[6][1])
    return [
        ('plane', [0, *f], [(0.0, 0.0), (1.5, -2.0)]),
        ('cylinder', [1, *f, 2.0], [(0.0, 0.0), (1.0, 3.0)]),
        ('cone', [2, *f, 0.5, 1.0], [(0.3, 1.0), (0.0, -1.0 / math.sin(0.5)), (2.0, -1.0)]),
        ('sphere', [3, *f, 2.0], [(0.3, 0.4), (1.0, PI / 2), (1.0, -PI / 2), (0.0, 0.0)]),
        ('torus', [4, *f, 3.0, 1.0], [(0.0, 0.0), (0.5, PI), (1.0, PI / 2)]),
        ('bspline_saddle', bspline, [(0.5, 0.5), (0.0, 0.0), (1.0, 0.25)]),
        ('bspline_collapsed', collapsed, [(0.0, 0.5), (0.5, 0.5)]),
        ('revolution', [6, 0, 0, 0, 0, 0, 1, *s_curve], [(0.5, 0.5), (0.0, 0.0), (2.0, 1.0)]),
        ('extrusion', [7, 0, 0, 1, *s_curve], [(0.5, 0.0), (0.25, 3.0)]),
        ('offset_sphere', [8, 0.5, 3, *f, 2.0], [(0.3, 0.4), (0.0, PI / 2)]),
    ]


def slprops_requests(rng, requests):
    for name, spec, params in surfaces(rng):
        flat = [c for uv in params for c in uv]
        for mode in range(4):
            add(requests, 'lprop_sl', *spec, mode, 2, 1e-7, len(params), *flat)
        for n in (0, 1, 3):
            add(requests, 'lprop_sl', *spec, 0, n, 1e-7, len(params), *flat)


def curinf_requests(rng, requests):
    for name, spec, _ in curves2d(rng):
        for mode in range(4):
            add(requests, 'lprop_curinf2d', *spec, mode)
    # Trimmed ellipse arcs that keep some quarter points only.
    for u1, u2 in [(0.1, 1.0), (-1.0, 4.0), (3.0, 9.0), (PI, 2 * PI)]:
        add(requests, 'lprop_curinf2d', 7, u1, u2, 2, *ax2d(rng), 1, 5.0, 2.0, 1)
    sequences = [
        [(0, 1.5)], [(1, 2.0), (2, 3.0)], [(0, 5.0), (1, 1.0), (2, 3.0), (0, 7.0)],
        [(0, 1.0), (1, 1.0), (2, 1.0)], [(0, 2.0), (0, 2.0), (1, 0.5)], [(0, 1.0), (3, 0.0), (2, 5.0)],
        [(2, -1.0), (0, -1.0), (1, -2.0)],
    ]
    for seq in sequences:
        add(requests, 'lprop_curandinf', len(seq), *[c for pair in seq for c in pair], 4, 0, 1, 2, 5)


def continuity_requests(rng, requests):
    origin = [0.0, 0.0, 0.0]
    x_line = [0, *origin, 1, 0, 0]
    diag_line = [0, *origin, 0.6, 0.8, 0]
    circle = [1, -1, 0, 0, 0, 0, 1, 1, 0, 0, 1.0]
    bspline = to3d(curves2d(rng)[10][1])
    tangent_line = [0, 0, 0, 0, 0, 1, 0]
    cases = [
        (x_line, x_line, 0.0, 0.0), (x_line, diag_line, 0.0, 0.0), (circle, tangent_line, 0.0, 0.0),
        (x_line, tangent_line, 0.0, 0.0), (bspline, bspline, 2.0, 2.0), (bspline, bspline, 1.0, 1.0),
        (x_line, circle, 0.0, 0.0), (x_line, x_line, 1.0, 0.0), ([7, 0.0, 1.0] + bspline, bspline, 1.0, 1.0),
        (to3d(curves2d(rng)[7][1]), to3d(curves2d(rng)[7][1]), 0.0, 0.0),
    ]
    for c1, c2, u1, u2 in cases:
        for r1, r2 in [(0, 0), (1, 0), (0, 1), (1, 1)]:
            add(requests, 'lprop_continuity', *c1, *c2, u1, u2, r1, r2, 0, 0, 0)
            add(requests, 'lprop_continuity', *c1, *c2, u1, u2, r1, r2, 1, 1e-3, 1e-2)


def gprop_points(rng, n, kind):
    pts = []
    for i in range(n):
        p = point(rng, 3.0)
        if kind == 'plane':
            p[2] = 0.5
        if kind == 'line':
            t = rng.uniform(-2, 2)
            p = [1 + t, 2 - 2 * t, 3 * t]
        if kind == 'point':
            p = [1.0, -2.0, 0.5]
        pts.append(p)
    return pts


def gprop_requests(rng, requests):
    for kind in ('space', 'plane', 'line', 'point'):
        for n in (1, 2, 3, 6):
            pts = gprop_points(rng, n, kind)
            dens = [rng.uniform(0.5, 3.0) for _ in pts]
            flat = [c for p, d in zip(pts, dens) for c in (*p, d)]
            for mode in range(3):
                add(requests, 'gprop_points', mode, n, 1, *flat)
            if n in (2, 6):
                add(requests, 'gprop_points', 3, n, 2, *flat)
            for tol in (1e-6, 1e-12, 10.0):
                add(requests, 'gprop_pequation', n, tol, *[c for p in pts for c in p])
    bad = [0, 0, 0, 1.0, 1, 1, 1, 0.0, 2, 2, 2, -1.0]
    for mode in (1, 2, 3):
        add(requests, 'gprop_points', mode, 3, 1 if mode < 3 else 3, *bad)
    add(requests, 'gprop_pequation', 5, 1e-7, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 1, 1, 1)
    for _ in range(3):
        add(requests, 'gprop_hoperator', *point(rng), *point(rng), rng.uniform(0.1, 5))


def elementary_requests(rng, requests):
    f = frame(rng, 2.0)
    loc = point(rng, 2.0)
    arcs = [(0.3, 2.1), (0.0, 2 * PI), (-1.0, 0.5), (1.0, 1.0), (2.0, 0.5)]
    for perform in (0, 1):
        add(requests, 'gprop_elementary', 0, perform, *loc, *f, 1.5)
        for u1, u2 in arcs:
            add(requests, 'gprop_elementary', 1, perform, *loc, *f, 1.5, u1, u2)
            add(requests, 'gprop_elementary', 2, perform, *loc, *point(rng), *unit(rng), u1, u2)
        for solid in (0, 4):
            ax3 = frame(rng, 2.0)
            patches = [
                (0, [1.5], [(0.2, 2.0, -1.0, 2.0), (0.0, 2 * PI, 0.0, 3.0), (1.0, 1.0, 0.0, 1.0)]),
                (1, [0.4, 1.0], [(0.0, 2 * PI, 0.0, 2.0), (0.5, 1.5, -1.0, 1.0), (0.0, 1.0, -2.0, 0.0)]),
                (2, [2.0], [(0.0, 2 * PI, -PI / 2, PI / 2), (0.3, 1.2, -0.4, 0.9), (0.0, 1.0, 0.5, 0.5)]),
                (3, [3.0, 1.0], [(0.0, 2 * PI, 0.0, 2 * PI), (0.2, 1.4, 0.3, 2.5), (0.0, 1.0, 1.0, 1.0)]),
            ]
            for shape, radii, bounds in patches:
                for b in bounds:
                    add(requests, 'gprop_elementary', 3 + solid + shape, perform, *loc, *ax3, *radii, *b)
        add(requests, 'gprop_elementary', 4, perform, *loc, *f, 2.0, 0.0, 0.0, 1.0, 0.0, 1.0)


def compose_requests(rng, requests):
    for _ in range(4):
        loc = point(rng, 2.0)
        items = []
        for _ in range(3):
            n = rng.randrange(1, 4)
            items += [n] + [c for _ in range(n) for c in point(rng, 3.0)] + [rng.uniform(0.2, 2.0)]
        add(requests, 'gprop_compose', *loc, 3, *items)
    add(requests, 'gprop_compose', 0, 0, 0, 2, 1, 1, 0, 0, 1.0, 0, 2.0)
    add(requests, 'gprop_compose', 1, 1, 1, 2, 2, 1, 1, 1, 2, 2, 2, 0.0, 1, 1, 1, 1, 1e-300)


def hermit_spline(w1, w2, w3, dims=3):
    poles = [(0, 0), (1, 1), (2, 0)]
    body = []
    for (x, y), w in zip(poles, (w1, w2, w3)):
        body += [x, y] + ([0] if dims == 3 else []) + [w]
    return [6, 2, 0, 3] + body + [2, 0, 3, 1, 3]


def hermit_requests(rng, requests):
    weights = [(1, 1, 1), (2, 1.5, 3), (0.5, 1, 5), (5, 1, 0.5), (2, 3, 1.5), (2, 1, 2), (1, 20, 1),
               (0.01, 1, 100), (1, 0.1, 1), (3, 0.2, 0.3), (1e-3, 1, 1e3)]
    for w in weights:
        for tp, tk in [(1e-6, 1e-6), (1e-3, 1e-6), (1e-6, 0.3), (0.0, 1e-6)]:
            add(requests, 'hermit', 0, tp, tk, 0.0, 1.0, *hermit_spline(*w))
            add(requests, 'hermit', 1, tp, tk, 0.0, 1.0, *hermit_spline(*w, dims=2))
            add(requests, 'hermit', 2, tp, tk, -1.0, 2.0, *hermit_spline(*w))
    cubic = [6, 3, 0, 5, 0, 0, 0, 1.0, 1, 1, 0, 2.0, 2, 0, 0, 0.5, 3, 1, 0, 3.0, 4, 0, 0, 1.5,
             3, 0, 4, 0.4, 1, 1, 4]
    for mode in (0, 2):
        add(requests, 'hermit', mode, 1e-6, 1e-6, 0.0, 1.0, *cubic)


def append_lprop_requests(requests: list[str]) -> None:
    """Append the LProp, GeomLProp, GProp and Hermit requests in a fixed order."""
    rng = random.Random(20260928)
    clprops_requests(rng, requests)
    slprops_requests(rng, requests)
    curinf_requests(rng, requests)
    continuity_requests(rng, requests)
    gprop_requests(rng, requests)
    elementary_requests(rng, requests)
    compose_requests(rng, requests)
    hermit_requests(rng, requests)
