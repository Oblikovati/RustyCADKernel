"""Deterministic requests for the Extrema point-surface classes (tools/oracle/tkgeombase_probe_extrema_ps.cxx).

Extrema_FuncPSNorm and Extrema_FuncPSDist are evaluated inside, on and outside the
bounds of every surface kind, with repeated states within PConfusion.
Extrema_GenLocateExtPS starts from interior, boundary, outside and far starts with
both criteria. Extrema_GenExtPS runs the grid and the tree algorithms with every
flag, few and many samples, sub-ranges, a second point and algorithm switches.
Extrema_ExtPS covers the elementary, swept, spline, Bezier, offset and trimmed
surfaces, with points in general position, on the surface, at the centre, on the
axis and far away, periodic and trimmed ranges, infinite bounds and degenerated
isos (the 300-sample grids). Extrema_ExtPExtS and Extrema_ExtPRevS run their
analytical cases (conics normal and oblique to the direction, conics in a plane of
the axis) and their grid cases, the point on the axis, and the re-initialization on
the same handle.
"""
from __future__ import annotations
import math
import random
from tkg3d_corpus import frame, point, unit
from tkgeombase_corpus_gce import add
from tkgeombase_corpus_lprop import surfaces
from tkgeombase_corpus_bndlib import bezier_surface
from tkgeombase_corpus_extrema_cc import bezier3, bspline3, circle, conic, line

PI = math.pi
X = [1.0, 0.0, 0.0]
Y = [0.0, 1.0, 0.0]
Z = [0.0, 0.0, 1.0]
O = [0.0, 0.0, 0.0]


def surface_specs(rng):
    """(name, spec, (u1, u2, v1, v2) finite patch) with the probe's prefix codes."""
    lp = {name: spec for name, spec, _ in surfaces(rng)}
    f = frame(rng, 2.0)
    circle_curve = [1, 3.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0]
    bez_curve = [5, 4, 1.0, 0.0, 0.0, 1, 2.0, 0.0, 1.0, 1, 1.5, 0.0, 2.0, 1, 2.5, 0.0, 3.0, 1]
    return [
        ('plane', [0, *lp['plane']], (-3.0, 3.0, -2.0, 2.0)),
        ('cylinder', [0, *lp['cylinder']], (0.0, 2 * PI, -2.0, 3.0)),
        ('cone', [0, *lp['cone']], (0.0, 2 * PI, -1.0, 2.0)),
        ('sphere', [0, *lp['sphere']], (0.0, 2 * PI, -PI / 2, PI / 2)),
        ('torus', [0, *lp['torus']], (0.0, 2 * PI, 0.0, 2 * PI)),
        ('bspline_saddle', [0, *lp['bspline_saddle']], (0.0, 1.0, 0.0, 1.0)),
        ('bspline_collapsed', [0, *lp['bspline_collapsed']], (0.0, 1.0, 0.0, 1.0)),
        ('revolution', [0, *lp['revolution']], (0.0, 2 * PI, 0.0, 1.0)),
        ('revolution_circle', [0, 6, 0, 0, 0, 0, 0, 1, *circle_curve], (0.0, 2 * PI, 0.0, 2 * PI)),
        ('revolution_bezier', [0, 6, 0, 0, 0, 0, 0, 1, *bez_curve], (0.0, 2 * PI, 0.0, 1.0)),
        ('extrusion', [0, *lp['extrusion']], (0.0, 1.0, -2.0, 2.0)),
        ('extrusion_circle', [0, 7, 0, 0.6, 0.8, 1, *f, 1.5], (0.0, 2 * PI, -1.0, 1.0)),
        ('offset_sphere', [0, *lp['offset_sphere']], (0.0, 2 * PI, -PI / 2, PI / 2)),
        ('bezier', bezier_surface(), (0.0, 1.0, 0.0, 1.0)),
        ('trimmed_sphere', [2, 0.0, PI, -PI / 4, PI / 4, 0, *lp['sphere']], (0.0, PI, -PI / 4, PI / 4)),
    ]


def infinite(name):
    return name.startswith(('plane', 'cylinder', 'cone', 'extrusion'))


def adaptor(name, spec, patch, ranged):
    """The surface spec with the adaptor bounds (always ranged for an infinite surface)."""
    if ranged or infinite(name):
        return [*spec, 1, *patch]
    return [*spec, 0]


def points(rng, name, spec):
    """Points in general position, near the origin, far away, and the centre of a
    sphere or a torus."""
    out = [point(rng, 4.0), point(rng, 1.0), [0.1, 0.2, 0.3], point(rng, 50.0)]
    if name in ('sphere', 'torus', 'offset_sphere'):
        kind_at = spec.index(3 if name != 'torus' else 4, 1)
        out.append(spec[kind_at + 1:kind_at + 4])
    return out


def uv_samples(patch):
    u1, u2, v1, v2 = patch
    mid = (0.5 * (u1 + u2), 0.5 * (v1 + v2))
    return [mid, (u1, v1), (u2, v2), (mid[0] + 1.0e-12, mid[1]), (u2 + 0.5, v1 - 0.5),
            (0.3 * u1 + 0.7 * u2, 0.8 * v1 + 0.2 * v2)]


def func_requests(rng, specs, requests):
    for name, spec, patch in specs:
        for ranged in (False, True):
            ad = adaptor(name, spec, patch, ranged)
            uvs = uv_samples(patch)
            flat = [c for uv in uvs for c in uv]
            add(requests, 'ex_fpsn', *ad, *point(rng, 3.0), len(uvs), *flat)
            add(requests, 'ex_fpsd', *ad, *point(rng, 3.0), len(uvs), *flat)


def locate_requests(rng, specs, requests):
    for name, spec, patch in specs:
        u1, u2, v1, v2 = patch
        starts = [(0.5 * (u1 + u2), 0.5 * (v1 + v2)), (u1, v2), (0.9 * u2 + 0.1 * u1, 0.1 * v2 + 0.9 * v1),
                  (u2 + 1.0, v1 - 1.0), (1.0e5, 1.0e5)]
        flat = [c for s in starts for c in s]
        ad = adaptor(name, spec, patch, False)
        for tol in ((1.0e-9, 1.0e-9), (1.0e-3, 1.0e-4)):
            add(requests, 'ex_glps', *ad, *point(rng, 3.0), *tol, len(starts), *flat)


def genps_requests(rng, specs, requests):
    configs = [(10, 10, 2, 0), (5, 12, 0, 1), (12, 5, 1, 0), (2, 2, 2, 1), (20, 20, 0, 0)]
    for index, (name, spec, patch) in enumerate(specs):
        u1, u2, v1, v2 = patch
        sub = (u1 + 0.25 * (u2 - u1), u2 - 0.2 * (u2 - u1), v1 + 0.1 * (v2 - v1), v2 - 0.3 * (v2 - v1))
        ad = adaptor(name, spec, patch, index % 2 == 1)
        for k, (nu, nv, fl, al) in enumerate(configs):
            tols = (1.0e-9, 1.0e-9) if k % 2 == 0 else (1.0e-6, 1.0e-5)
            add(requests, 'ex_genps', *ad, *point(rng, 3.0), nu, nv, *tols, fl, al, *sub, *point(rng, 3.0))


def ps_requests(rng, specs, requests):
    combos = [(2, 0), (0, 0), (1, 0), (2, 1), (0, 1)]
    for index, (name, spec, patch) in enumerate(specs):
        u1, u2, v1, v2 = patch
        sub = (u1 + 0.1 * (u2 - u1), u2 - 0.3 * (u2 - u1), v1 + 0.2 * (v2 - v1), v2 - 0.1 * (v2 - v1))
        for k, p in enumerate(points(rng, name, spec)):
            fl, al = combos[(index + k) % len(combos)]
            for ranged in (False, True):
                add(requests, 'ex_ps', *adaptor(name, spec, patch, ranged), *p, 1.0e-9, 1.0e-9, fl, al, *sub,
                    *point(rng, 3.0))
    # Infinite bounds become +-1e10; periodic ranges beyond one period; the far
    # trimmed range of a torus.
    lp = {name: spec for name, spec, _ in surfaces(rng)}
    add(requests, 'ex_ps', 0, *lp['plane'], 0, *point(rng, 3.0), 1.0e-9, 1.0e-9, 2, 0, -1.0, 1.0, -1.0, 1.0,
        *point(rng, 3.0))
    add(requests, 'ex_ps', 0, *lp['cylinder'], 1, 3.0, 9.0, -1.0, 1.0, *point(rng, 3.0), 1.0e-9, 1.0e-9, 2, 0,
        4.0, 8.0, -0.5, 0.5, *point(rng, 3.0))
    add(requests, 'ex_ps', 0, *lp['torus'], 1, -7.0, -6.0, 5.0, 6.0, *point(rng, 5.0), 1.0e-9, 1.0e-9, 2, 0,
        -7.0, -6.5, 5.0, 5.5, *point(rng, 5.0))


def swept_tail(rng, u, v, tol=(1.0e-9, 1.0e-9)):
    return [*point(rng, 3.0), *u, *v, *tol, *point(rng, 3.0)]


def extrusion_requests(rng, requests):
    f = frame(rng, 1.0)
    p, n, x = f[0:3], f[3:6], f[6:9]
    oblique = unit(rng)
    curves = [
        (circle(p, n, x, 1.5), n, (0.0, 2 * PI)),
        (circle(p, n, x, 1.5), oblique, (0.0, 2 * PI)),
        (circle(p, n, x, 1.5, (0.5, 2.0)), oblique, (0.5, 2.0)),
        (conic(2, p, n, x, 2.0, 1.0), n, (0.0, 2 * PI)),
        (conic(2, p, n, x, 2.0, 1.0), oblique, (0.0, 2 * PI)),
        (conic(3, p, n, x, 2.0, 1.0, (-1.0, 1.0)), oblique, (-1.0, 1.0)),
        (conic(4, p, n, x, 0.7, rng_=(-2.0, 2.0)), n, (-2.0, 2.0)),
        (line(O, X, (-2.0, 2.0)), Z, (-2.0, 2.0)),
        (line(O, X, (-2.0, 2.0)), [0.6, 0.0, 0.8], (-2.0, 2.0)),
        (circle(O, Z, X, 1.0), X, (0.0, 2 * PI)),
        (bezier3([[0.0, 0.0, 0.0], [1.0, 2.0, 0.0], [3.0, 2.0, 0.0], [4.0, 0.0, 0.0]]), Z, (0.0, 1.0)),
        (bspline3([[0.0, 0.0, 0.0], [1.0, 1.0, 0.0], [2.0, -1.0, 0.0], [3.0, 0.0, 0.0]], [0.0, 1.0], [4, 4], 3),
         Z, (0.0, 1.0)),
    ]
    for curve, d, u in curves:
        for v in ((-1.0, 2.0), (0.0, 0.5)):
            add(requests, 'ex_pexts', *curve, *d, *swept_tail(rng, u, v))
    # Points on the curve plane and on the extrusion line.
    add(requests, 'ex_pexts', *circle(O, Z, X, 1.0), *Z, 0.0, 0.0, 0.0, 0.0, 2 * PI, -1.0, 1.0, 1.0e-9, 1.0e-9,
        1.0, 0.0, 0.5)
    add(requests, 'ex_pexts', *circle(O, Z, X, 1.0), *[0.0, 0.6, 0.8], 2.0, 0.0, 0.0, 0.0, 2 * PI, -1.0, 1.0,
        1.0e-9, 1.0e-9, 0.0, 0.0, 0.0)


def revolution_requests(rng, requests):
    axis = [*O, *Z]
    xz_circle = [1, 3.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0]
    curves = [
        (xz_circle, (0.0, 2 * PI), (0.0, 2 * PI)),
        (xz_circle, (0.0, PI), (0.5, 2.0)),
        (conic(2, [3.0, 0.0, 0.0], Y, X, 1.0, 0.5), (0.0, 2 * PI), (0.0, 2 * PI)),
        (conic(3, [2.0, 0.0, 0.0], Y, X, 1.0, 0.5, (-1.0, 1.0)), (0.0, 2 * PI), (-1.0, 1.0)),
        (conic(4, [2.0, 0.0, 0.0], Y, X, 0.5, rng_=(-1.0, 1.0)), (0.0, 2 * PI), (-1.0, 1.0)),
        (line([2.0, 0.0, 0.0], Z, (-1.0, 1.0)), (0.0, 2 * PI), (-1.0, 1.0)),
        (line([0.0, 0.0, 0.0], [0.6, 0.0, 0.8], (0.0, 2.0)), (0.0, 2 * PI), (0.0, 2.0)),
        (line([2.0, 0.0, 0.0], [0.6, 0.8, 0.0], (-1.0, 1.0)), (0.0, 2 * PI), (-1.0, 1.0)),
        (circle([3.0, 0.0, 0.0], Z, X, 1.0), (0.0, 2 * PI), (0.0, 2 * PI)),
        (bezier3([[1.0, 0.0, 0.0], [2.0, 0.0, 1.0], [1.5, 0.0, 2.0], [2.5, 0.0, 3.0]]), (0.0, 2 * PI), (0.0, 1.0)),
        (bezier3([[0.0, 0.0, 0.0], [2.0, 0.0, 1.0], [1.5, 0.0, 2.0], [2.5, 0.0, 3.0]]), (0.0, 2 * PI), (0.0, 1.0)),
    ]
    for curve, u, v in curves:
        for uu in (u, (0.5, 2.5)):
            add(requests, 'ex_prevs', *curve, *axis, *swept_tail(rng, uu, v))
    # The point on the axis, and in the plane of the curve.
    add(requests, 'ex_prevs', *xz_circle, *axis, 0.0, 0.0, 1.0, 0.0, 2 * PI, 0.0, 2 * PI, 1.0e-9, 1.0e-9,
        5.0, 0.0, 0.5)
    add(requests, 'ex_prevs', *xz_circle, *axis, 5.0, 0.0, 0.5, 0.0, 2 * PI, 0.0, 2 * PI, 1.0e-9, 1.0e-9,
        0.0, 5.0, -0.5)
    tilted = [*O, *unit(rng)]
    add(requests, 'ex_prevs', *xz_circle, *tilted, *swept_tail(rng, (0.0, 2 * PI), (0.0, 2 * PI)))


def append_extrema_ps_requests(requests: list[str]) -> None:
    """Append the point-surface requests (seed 20261002)."""
    rng = random.Random(20261002)
    specs = surface_specs(rng)
    func_requests(rng, specs, requests)
    locate_requests(rng, specs, requests)
    genps_requests(rng, specs, requests)
    ps_requests(rng, specs, requests)
    extrusion_requests(rng, requests)
    revolution_requests(rng, requests)
