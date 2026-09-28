"""Deterministic requests for the Extrema curve-surface classes (tools/oracle/tkgeombase_probe_extrema_cs.cxx).

Extrema_ExtCS runs every closed form (a line with a plane, cylinder or sphere; a
circle with a cylinder, plane or sphere; a hyperbola with a plane) in secant,
tangent, missing, parallel and trimmed-out configurations (the curve-end fallback),
a line with the other surfaces (the part facing the surface box, down to a single
point), and the generic search with sharp points (B-spline, Bezier, offset, ellipse
and hyperbola curves on quadrics, splines and swept surfaces). Extrema_GenExtCS runs
its three searches (over (t, u, v), a conic over (u, v), a quadric over t) with few
and many samples, sub-ranges and closed curves; the functions are evaluated inside
and outside the bounds.
"""
from __future__ import annotations
import math
import random
from tkg3d_corpus import frame, point, unit
from tkgeombase_corpus_gce import add
from tkgeombase_corpus_extrema_cc import bezier3, bspline3, circle, conic, line
from tkgeombase_corpus_extrema_ps import adaptor, surface_specs

PI = math.pi
X = [1.0, 0.0, 0.0]
Y = [0.0, 1.0, 0.0]
Z = [0.0, 0.0, 1.0]
O = [0.0, 0.0, 0.0]


def curve_specs(rng):
    """(name, curve adaptor spec, (t1, t2) finite range)."""
    f = frame(rng, 1.0)
    p, n, x = f[0:3], f[3:6], f[6:9]
    spline = bspline3([[0.0, 0.0, 0.0], [1.0, 1.0, 0.5], [2.0, -1.0, 0.0], [3.0, 0.0, 1.0], [4.0, 1.0, 0.0]],
                      [0.0, 0.5, 1.0], [4, 1, 4], 3)
    return [
        ('line', line([*point(rng, 1.0)], unit(rng), (-5.0, 5.0)), (-5.0, 5.0)),
        ('circle', circle(p, n, x, 1.5), (0.0, 2 * PI)),
        ('ellipse', conic(2, p, n, x, 2.0, 1.0), (0.0, 2 * PI)),
        ('hyperbola', conic(3, p, n, x, 2.0, 1.0, (-1.0, 1.0)), (-1.0, 1.0)),
        ('parabola', conic(4, p, n, x, 0.7, rng_=(-2.0, 2.0)), (-2.0, 2.0)),
        ('bezier', bezier3([[0.0, 0.0, 0.0], [1.0, 2.0, 1.0], [3.0, 2.0, -1.0], [4.0, 0.0, 0.5]]), (0.0, 1.0)),
        ('bspline', spline, (0.0, 1.0)),
    ]


def pick(specs, *names):
    return [s for s in specs if s[0] in names]


def triples(trange, patch):
    t1, t2 = trange
    u1, u2, v1, v2 = patch
    return [(0.5 * (t1 + t2), 0.5 * (u1 + u2), 0.5 * (v1 + v2)), (t1, u1, v1), (t2, u2, v2),
            (0.5 * (t1 + t2) + 1.0e-12, 0.5 * (u1 + u2), 0.5 * (v1 + v2)), (t2 + 1.0, u1, v2 + 1.0)]


def function_requests(rng, curves, surfaces, requests):
    pairs = [(c, s) for i, c in enumerate(curves) for j, s in enumerate(surfaces) if (i + j) % 4 == 0]
    for (cname, cspec, trange), (sname, sspec, patch) in pairs:
        tr = triples(trange, patch)
        flat = [v for t in tr for v in t]
        ad = adaptor(sname, sspec, patch, False)
        add(requests, 'ex_fecs', *cspec, *ad, len(tr), *flat)
        add(requests, 'ex_gofcs', *cspec, *ad, len(tr), *flat)


def conic_function_requests(curves, surfaces, requests):
    conics = pick(curves, 'line', 'circle', 'ellipse', 'hyperbola', 'parabola')
    for i, (cname, cspec, trange) in enumerate(conics):
        for sname, sspec, patch in surfaces[i::3]:
            u1, u2, v1, v2 = patch
            uvs = [(0.5 * (u1 + u2), 0.5 * (v1 + v2)), (u1, v1), (u2 + 1.0, v1), (0.3 * u1 + 0.7 * u2, v2)]
            flat = [v for uv in uvs for v in uv]
            ranges = [*trange, *patch]
            if cname in ('circle', 'ellipse'):
                ranges[0:2] = [7.0, 7.0 + 2 * PI]
            add(requests, 'ex_gofconics', *cspec, *adaptor(sname, sspec, patch, False), *ranges, len(uvs), *flat)


def quadric_function_requests(curves, surfaces, requests):
    quads = [s for s in surfaces if s[0] in ('plane', 'cylinder', 'cone', 'sphere', 'torus', 'bezier')]
    for i, (sname, sspec, patch) in enumerate(quads):
        for cname, cspec, trange in curves[i::2]:
            t1, t2 = trange
            ts = [0.5 * (t1 + t2), t1, t2, t2 + 1.0]
            u1, u2, v1, v2 = patch
            ranges = [t1, t2, u1, u2, v1, v2]
            if sname in ('cylinder', 'cone', 'sphere', 'torus'):
                ranges[2:4] = [-7.0, -7.0 + 2 * PI]
            add(requests, 'ex_gofcquad', *cspec, *adaptor(sname, sspec, patch, False), *ranges, len(ts), *ts)


def gen_requests(curves, surfaces, requests):
    samples = [(12, 10, 10), (5, 3, 4)]
    for i, (cname, cspec, trange) in enumerate(curves):
        for j, (sname, sspec, patch) in enumerate(surfaces[i % 3::4]):
            nt, nu, nv = samples[(i + j) % 2]
            t1, t2 = trange
            u1, u2, v1, v2 = patch
            sub = [t1 + 0.1 * (t2 - t1), t2 - 0.2 * (t2 - t1), u1 + 0.1 * (u2 - u1), u2, v1, v2 - 0.25 * (v2 - v1)]
            add(requests, 'ex_genecs', *cspec, *adaptor(sname, sspec, patch, j % 2 == 1), nt, nu, nv, 1.0e-9, 1.0e-9,
                *sub)


def closed_form_requests(rng, requests):
    """The Extrema_ExtElCS configurations and their trimmed-out fallbacks."""
    plane = [0, 0, *O, *Z, *X]
    sphere = [0, 3, *O, *Z, *X, 2.0]
    cylinder = [0, 1, *O, *Z, *X, 2.0]
    cases = [
        (line([-5.0, 0.5, 0.3], X, (-10.0, 10.0)), sphere, (-10.0, 10.0)),
        (line([-5.0, 2.0, 0.0], X, (-10.0, 10.0)), sphere, (-10.0, 10.0)),
        (line([-5.0, 3.0, 1.0], X, (-10.0, 10.0)), sphere, (-10.0, 10.0)),
        (line([-5.0, 3.0, 1.0], X, (-10.0, -8.0)), sphere, (-10.0, -8.0)),
        (line([0.0, 0.0, -3.0], Z, (-5.0, 5.0)), cylinder, (-5.0, 5.0)),
        (line([3.0, 0.0, -3.0], [0.0, 0.6, 0.8], (-5.0, 5.0)), cylinder, (-5.0, 5.0)),
        (line([-5.0, 0.5, 0.0], X, (-10.0, 10.0)), cylinder, (-10.0, 10.0)),
        (line([0.0, 0.0, 1.0], X, (-5.0, 5.0)), plane, (-5.0, 5.0)),
        (line([0.0, 0.0, 1.0], [0.6, 0.0, 0.8], (-5.0, 5.0)), plane, (-5.0, 5.0)),
        (circle([0.0, 0.0, 1.0], Z, X, 1.0), cylinder, (0.0, 2 * PI)),
        (circle([0.0, 0.0, 1.0], Z, X, 3.0), cylinder, (0.0, 2 * PI)),
        (circle([0.0, 0.0, 1.0], Z, X, 1.0), plane, (0.0, 2 * PI)),
        (circle([0.0, 0.0, 1.0], [0.0, 0.6, 0.8], X, 1.0), plane, (0.0, 2 * PI)),
        (circle([0.0, 0.0, 1.0], [0.0, 0.6, 0.8], X, 1.0, (0.2, 0.9)), plane, (0.2, 0.9)),
        (circle([0.5, 0.0, 0.0], Z, X, 1.0), sphere, (0.0, 2 * PI)),
        (circle(O, Z, X, 2.0), sphere, (0.0, 2 * PI)),
        (conic(3, [0.0, 0.0, 1.0], Y, X, 2.0, 1.0, (-1.0, 1.0)), plane, (-1.0, 1.0)),
        (conic(3, [0.0, 0.0, 1.0], Z, X, 2.0, 1.0, (-1.0, 1.0)), plane, (-1.0, 1.0)),
    ]
    for curve, surf, trange in cases:
        for patch in ((-3.0, 3.0, -3.0, 3.0), (0.0, 2 * PI, -1.0, 1.0)):
            surf_ad = [*surf, 1, *patch]
            add(requests, 'ex_cs', *curve, *surf_ad, 1.0e-9, 1.0e-9, *trange, *patch)
    # The unranged elementary surfaces, and a parameter range beyond the curve.
    add(requests, 'ex_cs', *line([-5.0, 0.5, 0.3], X), *sphere, 0, 1.0e-9, 1.0e-9, -10.0, 10.0, 0.0, 2 * PI,
        -PI / 2, PI / 2)
    add(requests, 'ex_cs', *circle(O, Z, X, 1.0), *cylinder, 0, 1.0e-9, 1.0e-9, 1.0, 7.0, 0.0, 2 * PI, -1.0, 1.0)


def line_general_requests(rng, surfaces, requests):
    """A line with the surfaces without a closed form: facing range, single point."""
    for sname, sspec, patch in surfaces:
        if sname in ('plane', 'sphere', 'cylinder'):
            continue
        for ln in (line([*point(rng, 1.0)], unit(rng), (-5.0, 5.0)), line([20.0, 20.0, 20.0], X, (-1.0, 1.0))):
            add(requests, 'ex_cs', *ln, *adaptor(sname, sspec, patch, False), 1.0e-9, 1.0e-9, -5.0, 5.0, *patch)


def generic_requests(curves, surfaces, requests):
    for i, (cname, cspec, trange) in enumerate(curves):
        if cname == 'line':
            continue
        for j, (sname, sspec, patch) in enumerate(surfaces[i % 2::3]):
            t1, t2 = trange
            add(requests, 'ex_cs', *cspec, *adaptor(sname, sspec, patch, j % 2 == 0), 1.0e-9, 1.0e-9, t1, t2, *patch)


def append_extrema_cs_requests(requests: list[str]) -> None:
    """Append the curve-surface requests (seed 20261003)."""
    rng = random.Random(20261003)
    curves = curve_specs(rng)
    surfaces = surface_specs(rng)
    function_requests(rng, curves, surfaces, requests)
    conic_function_requests(curves, surfaces, requests)
    quadric_function_requests(curves, surfaces, requests)
    gen_requests(curves, surfaces, requests)
    closed_form_requests(rng, requests)
    line_general_requests(rng, surfaces, requests)
    generic_requests(curves, surfaces, requests)
