"""Deterministic requests for the Extrema curve-curve classes (tools/oracle/tkgeombase_probe_extrema_cc.cxx).

Extrema_ExtCC and Extrema_ExtCC2d run every analytic pair (a line with a line or
conic, two circles, and the 2D circle-conic pairs) in skew, secant, tangent,
parallel, antiparallel, collinear, concentric and nested configurations, with ranges
that overlap, touch or miss (the trimmed parallel cases), and the general pairs of
Extrema_ECC(2d) (conics with conics, Bezier, B-spline and offset curves, a closed
curve of one interval, very different lengths); a second Perform on one object shows
the kept Extrema_ECC solutions. Extrema_ECC(2d) also runs directly with the single
solution flag and tolerances, Extrema_LocateExtCC and Extrema_LocECC from start pairs
inside, at and outside the ranges, Extrema_CCLocFOfLocECC at regular and singular
points, and the three Extrema_GlobOptFuncCC functions inside and outside the ranges.
"""
from __future__ import annotations
import math
import random
from tkg3d_corpus import frame, point, unit
from tkgeombase_corpus_gce import add

Z = [0.0, 0.0, 1.0]
X = [1.0, 0.0, 0.0]
Y = [0.0, 1.0, 0.0]
O = [0.0, 0.0, 0.0]


def line(p, d, rng_=None):
    """A 3D line adaptor, ranged when `rng_` is given."""
    return [0, *p, *d, *([1, *rng_] if rng_ else [0])]


def circle(p, n, x, r, rng_=None):
    """A 3D circle adaptor."""
    return [1, *p, *n, *x, r, *([1, *rng_] if rng_ else [0])]


def conic(kind, p, n, x, a, b=None, rng_=None):
    """A 3D ellipse (2), hyperbola (3) or parabola (4) adaptor."""
    dims = [a] if b is None else [a, b]
    return [kind, *p, *n, *x, *dims, *([1, *rng_] if rng_ else [0])]


def bezier3(poles, rng_=None):
    """A non-rational 3D Bezier adaptor."""
    return [5, 0, len(poles), *[c for p in poles for c in (*p, 1.0)], *([1, *rng_] if rng_ else [0])]


def bspline3(poles, knots, mults, degree, rng_=None):
    """A non-rational non-periodic 3D B-spline adaptor."""
    body = [6, 0, 0, degree, len(knots), *[v for k, m in zip(knots, mults) for v in (k, m)], len(poles),
            *[c for p in poles for c in (*p, 1.0)]]
    return [*body, *([1, *rng_] if rng_ else [0])]


def pairs3d(rng):
    """(adaptor1, adaptor2, u1, u2, v1, v2) pairs in 3D."""
    f = frame(rng, 2.0)
    p, n, x = f[0:3], f[3:6], f[6:9]
    skew = (line(O, X, (-5.0, 5.0)), line([0.0, 0.0, 2.0], Y, (-5.0, 5.0)))
    out = [(*skew, -1.0, 1.0, -1.0, 1.0)]
    out.append((line(O, X, (0.0, 4.0)), line([0.0, 1.0, 0.0], X, (1.0, 3.0)), 0.0, 4.0, 1.0, 3.0))
    out.append((line(O, X, (0.0, 1.0)), line([0.0, 1.0, 0.0], X, (1.0, 3.0)), 0.0, 1.0, 1.0, 3.0))
    out.append((line(O, X, (0.0, 1.0)), line([0.0, 1.0, 0.0], X, (2.0, 3.0)), 0.0, 1.0, 2.0, 3.0))
    out.append((line(O, X, (0.0, 2.0)), line([5.0, 1.0, 0.0], [-1.0, 0.0, 0.0], (0.0, 4.0)), 0.0, 2.0, 0.0, 4.0))
    out.append((line(O, X), line([0.0, 1.0, 0.0], X, (0.0, 3.0)), -2.0e100, 2.0e100, 0.0, 3.0))
    out.append((line(O, X), line(O, X), -5.0, 5.0, -5.0, 5.0))
    out.append((line([*point(rng, 3.0)], unit(rng)), circle(p, n, x, 1.5), -3.0, 3.0, 0.5, 4.0))
    out.append((line([p[i] + 0.4 * x[i] for i in range(3)], [(n[1] * x[2] - n[2] * x[1]), (n[2] * x[0] - n[0] * x[2]),
                                                              (n[0] * x[1] - n[1] * x[0])], (-4.0, 4.0)),
                circle(p, n, x, 1.5), -4.0, 4.0, 0.0, 6.0))
    out.append((line(p, n, (-2.0, 2.0)), circle(p, n, x, 1.5, (0.0, 2.0)), -2.0, 2.0, 0.0, 2.0))
    out.append((circle(O, Z, X, 1.0), line([0.0, 0.0, 1.0], Z, (-1.0, 1.0)), 0.0, 1.0, -1.0, 1.0))
    out.append((line([*point(rng, 3.0)], unit(rng)), conic(2, p, n, x, 2.0, 1.0), -3.0, 3.0, 0.0, 6.0))
    out.append((conic(3, p, n, x, 2.0, 1.0, (-1.0, 1.0)), line([*point(rng, 3.0)], unit(rng), (-3.0, 3.0)),
                -1.0, 1.0, -3.0, 3.0))
    out.append((line([*point(rng, 3.0)], unit(rng)), conic(4, p, n, x, 0.7, rng_=(-2.0, 2.0)), -3.0, 3.0, -2.0, 2.0))
    out += circle_pairs()
    out += general_pairs3d(rng, f)
    return out


def circle_pairs():
    """Two circles: coplanar secant, tangent, nested, apart, concentric arcs."""
    out = []
    for dx, r2 in ((1.0, 1.0), (2.0, 1.0), (0.3, 0.5), (4.0, 1.0)):
        out.append((circle(O, Z, X, 1.0), circle([dx, 0.0, 0.0], Z, X, r2), 0.0, 6.0, 0.0, 6.0))
    out.append((circle(O, Z, X, 1.0, (0.0, 1.0)), circle(O, Z, X, 2.0, (0.5, 2.0)), 0.0, 1.0, 0.5, 2.0))
    out.append((circle(O, Z, X, 1.0, (0.0, 1.0)), circle(O, Z, X, 2.0, (3.0, 4.0)), 0.0, 1.0, 3.0, 4.0))
    out.append((circle(O, Z, X, 1.0, (0.0, 1.0)), circle(O, [0.0, 0.0, -1.0], X, 2.0, (0.5, 2.0)), 0.0, 1.0, 0.5, 2.0))
    out.append((circle(O, Z, X, 1.0, (0.0, 1.0)), circle(O, Z, X, 2.0, (1.0, 2.0)), 0.0, 1.0, 1.0, 2.0))
    out.append((circle(O, Z, X, 1.0), circle([0.5, 0.0, 0.3], X, Y, 1.0), 0.0, 6.0, 0.0, 6.0))
    return out


def general_pairs3d(rng, f):
    """The pairs of Extrema_ECC."""
    p, n, x = f[0:3], f[3:6], f[6:9]
    bez = bezier3([[0.0, 0.0, 0.0], [1.0, 2.0, 0.0], [3.0, 2.0, 1.0], [4.0, 0.0, 0.0]])
    spl = bspline3([[0.0, 1.0, 0.0], [1.0, -1.0, 0.5], [2.0, 1.0, 0.0], [3.0, -1.0, 0.0], [4.0, 1.0, 0.2]],
                   [0.0, 0.5, 1.0], [4, 1, 4], 3)
    spl1 = bspline3([[0.0, 0.0, 1.0], [2.0, 0.0, 1.0], [2.0, 2.0, 1.0]], [0.0, 1.0, 2.0], [2, 1, 2], 1)
    off = [8, 0.5, *Z, *circle(O, Z, X, 1.0)[:-1]]
    return [(bez, spl, 0.0, 1.0, 0.0, 1.0), (bez, line([2.0, 5.0, 0.0], X, (-5.0, 5.0)), 0.2, 0.8, -3.0, 3.0),
            (spl, circle(p, n, x, 1.0), 0.1, 0.9, 0.0, 6.0), (conic(2, p, n, x, 2.0, 1.0), circle([2.0, 0.0, 0.0], Z, X, 1.0),
                                                           0.0, 6.0, 0.0, 6.0),
            (spl1, bez, 0.0, 2.0, 0.0, 1.0), ([*off, 0], line([0.0, 3.0, 0.0], X, (-4.0, 4.0)), 0.0, 6.0, -4.0, 4.0),
            (line(O, X, (0.0, 1000.0)), bez, 0.0, 1000.0, 0.0, 1.0),
            (conic(2, O, Z, X, 3.0, 1.0), conic(2, [0.5, 0.2, 0.0], Z, X, 2.0, 1.5), 0.0, 6.0, 0.0, 6.0)]


def line2(p, d, rng_=None):
    """A 2D line adaptor."""
    return [0, *p, *d, *([1, *rng_] if rng_ else [0])]


def conic2(kind, p, d, dims, sense=1, rng_=None):
    """A 2D conic adaptor (1 circle, 2 ellipse, 3 hyperbola, 4 parabola)."""
    return [kind, *p, *d, sense, *dims, *([1, *rng_] if rng_ else [0])]


def pairs2d(rng):
    """(adaptor1, adaptor2, u1, u2, v1, v2) pairs in 2D."""
    d = [math.cos(0.3), math.sin(0.3)]
    c = conic2(1, [0.0, 0.0], [1.0, 0.0], [1.0])
    out = [(line2([0.0, 0.0], [1.0, 0.0], (-5.0, 5.0)), line2([0.0, 1.0], d, (-5.0, 5.0)), -5.0, 5.0, -5.0, 5.0),
           (line2([0.0, 0.0], [1.0, 0.0]), line2([0.0, 1.0], [1.0, 0.0]), -3.0, 3.0, -3.0, 3.0),
           (line2([0.0, 3.0], d, (-5.0, 5.0)), c, -5.0, 5.0, 0.0, 6.0), (c, line2([0.0, 3.0], d, (-5.0, 5.0)), 0.0, 6.0, -5.0, 5.0),
           (c, conic2(1, [3.0, 0.5], [1.0, 0.0], [1.0]), 0.0, 6.0, 0.0, 6.0),
           (c, conic2(1, [0.0, 0.0], [1.0, 0.0], [2.0]), 0.0, 6.0, 0.0, 6.0)]
    for kind, dims in ((2, [2.0, 1.0]), (3, [2.0, 1.0]), (4, [0.7])):
        other = conic2(kind, [0.5, -0.3], d, dims, 1, None if kind == 2 else (-1.5, 1.5))
        out.append((c, other, 0.0, 6.0, -1.5, 1.5))
        out.append((other, c, -1.5, 1.5, 0.0, 6.0))
        out.append((line2([0.0, 2.5], [1.0, 0.2], (-5.0, 5.0)), other, -5.0, 5.0, -1.5, 1.5))
        out.append((other, line2([0.0, 2.5], [1.0, 0.2], (-5.0, 5.0)), -1.5, 1.5, -5.0, 5.0))
        out.append((other, conic2(2, [1.0, 0.0], [1.0, 0.0], [3.0, 1.0]), -1.5, 1.5, 0.0, 6.0))
    bez = [5, 0, 4, 0.0, 0.0, 1, 1.0, 2.0, 1, 3.0, 2.0, 1, 4.0, 0.0, 1, 0]
    out += [(bez, c, 0.0, 1.0, 0.0, 6.0), (c, bez, 0.0, 6.0, 0.2, 0.9), (bez, line2([2.0, 3.0], [1.0, 0.0], (-4.0, 4.0)),
                                                                         0.0, 1.0, -4.0, 4.0)]
    out.append((line2([0.0, 2.5], [1.0, 0.0], (-5.0, 5.0)), bez, -5.0, 5.0, 0.0, 1.0))
    out.append((conic2(3, [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], 1, (-1.0, 1.0)), line2([0.0, 0.0], [0.0, 1.0], (-2.0, 2.0)),
                -1.0, 1.0, -2.0, 2.0))
    return out


def append_extrema_cc_requests(requests: list[str]) -> None:
    """Append the curve-curve requests (seed 20261001)."""
    rng = random.Random(20261001)
    p3, p2 = pairs3d(rng), pairs2d(rng)
    for index, (a, b, u1, u2, v1, v2) in enumerate(p3):
        add(requests, 'ex_cc', *a, *b, u1, u2, v1, v2, index % 2)
    for index, (a, b, u1, u2, v1, v2) in enumerate(p2):
        add(requests, 'ex_cc2d', *a, *b, u1, u2, v1, v2, index % 2)
    for index, (a, b, u1, u2, v1, v2) in enumerate(p3[-10:]):
        add(requests, 'ex_ecc', *a, *b, u1, u2, v1, v2, index % 2, [1.0e-7, 1.0e-3][index % 2])
    for index, (a, b, u1, u2, v1, v2) in enumerate(p2[-8:]):
        add(requests, 'ex_ecc2d', *a, *b, u1, u2, v1, v2, index % 2, [1.0e-7, 1.0e-3][index % 2])
    for a, b, u1, u2, v1, v2 in p3[::3]:
        starts = [[0.5 * (u1 + u2), 0.5 * (v1 + v2), 1.0e-9, 1.0e-9], [u1, v2, 1.0e-6, 1.0e-6],
                  [u2 + 1.0, v1 - 1.0, 1.0e-9, 1.0e-9]]
        add(requests, 'ex_locecc', *a, *b, len(starts), *[v for s in starts for v in s])
    for a, b, u1, u2, v1, v2 in p3[-8:]:
        uvs = [[u1, v1], [0.5 * (u1 + u2), 0.5 * (v1 + v2)], [u2, v2], [u2 + 1.0, v1]]
        add(requests, 'ex_fcc', *a, *b, 1.0e-10, len(uvs), *[v for uv in uvs for v in uv])
    cusp = bezier3([[0.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]])
    add(requests, 'ex_fcc', *cusp, *line([0.0, 2.0, 0.0], X, (-3.0, 3.0)), 1.0e-10, 3, 0.5, 0.0, 0.0, 0.5, 1.0, 1.0)
    for a, b, u1, u2, v1, v2 in p3[::4]:
        uvs = [[u1, v1], [0.5 * (u1 + u2), 0.5 * (v1 + v2)], [u2 + 1.0, v1]]
        add(requests, 'ex_gofcc', *a, *b, len(uvs), *[v for uv in uvs for v in uv])
    for a, b, u1, u2, v1, v2 in p2[::4]:
        uvs = [[u1, v1], [0.5 * (u1 + u2), 0.5 * (v1 + v2)], [u2 + 1.0, v1]]
        add(requests, 'ex_gofcc2d', *a, *b, len(uvs), *[v for uv in uvs for v in uv])
