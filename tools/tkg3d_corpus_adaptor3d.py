"""TKG3d corpus: Adaptor3d_CurveOnSurface, Adaptor3d_IsoCurve, Adaptor3d_InterFunc and
Adaptor3d_HSurfaceTool (tools/oracle/tkg3d_probe_adaptor3d.cxx). 2D curve, 2D adaptor and
surface adaptor specifications follow the grammar documented in that file; surfaces and
3D curves follow g3_surface and g3_curve."""
from __future__ import annotations
import math
import random
from tkg3d_corpus import bspline, circle, fmt, frame, line, params, point, unit

F0 = [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0]
Z = [0.0, 0.0, 1.0]


# 2D curves.
def line2(p, d):
    return [0, *p, *d]


def conic2(kind, p, x, sense, *radii):
    return [kind, *p, *x, int(sense), *radii]


def bezier2(poles, rational=False):
    return [5, int(rational), len(poles), *[c for pole in poles for c in pole]]


def bspline2(degree, knots, poles, periodic=False, rational=False):
    return [6, int(rational), int(periodic), degree, len(knots), *[c for k in knots for c in k],
            len(poles), *[c for pole in poles for c in pole]]


def trimmed2(u1, u2, basis):
    return [7, u1, u2, *basis]


# 2D adaptors.
def whole(curve):
    return [0, *curve]


def ranged(u1, u2, curve):
    return [1, u1, u2, *curve]


def line2d(p, d, u1, u2):
    return [2, *p, *d, u1, u2]


# Surfaces (g3_surface) and surface adaptors.
def plane(f):
    return [0, *f]


def cylinder(f, radius):
    return [1, *f, radius]


def cone(f, angle, radius):
    return [2, *f, angle, radius]


def sphere(f, radius):
    return [3, *f, radius]


def torus(f, major, minor):
    return [4, *f, major, minor]


def extrusion(direction, basis):
    return [5, *direction, *basis]


def revolution(ax, basis):
    return [6, *ax, *basis]


def bezier_surface(nu, nv, rng, rational=False):
    poles = []
    for i in range(nu):
        for j in range(nv):
            poles += [i, j, rng.uniform(-0.5, 0.5), rng.uniform(0.5, 2.0) if rational else 1.0]
    return [7, int(rational), nu, nv, *poles]


def bspline_surface(uknots, vknots, degrees, rng, rational=False, periodic=(False, False)):
    nu = sum(m for _, m in uknots) - degrees[0] - 1 if not periodic[0] else sum(m for _, m in uknots[:-1])
    nv = sum(m for _, m in vknots) - degrees[1] - 1 if not periodic[1] else sum(m for _, m in vknots[:-1])
    poles = []
    for i in range(nu):
        for j in range(nv):
            poles += [i, j, rng.uniform(-0.5, 0.5), rng.uniform(0.5, 2.0) if rational else 1.0]
    return [8, int(rational), int(periodic[0]), int(periodic[1]), degrees[0], degrees[1],
            len(uknots), *[c for k in uknots for c in k], len(vknots), *[c for k in vknots for c in k],
            nu, nv, *poles]


def rect_trim(u1, u2, v1, v2, basis):
    return [9, u1, u2, v1, v2, 1, 1, *basis]


def offset_surface(d, basis):
    return [11, d, 0, *basis]


def adapted(surface):
    return [0, *surface]


def bounded(bounds, tolerances, surface):
    return [1, *bounds, *tolerances, *surface]


BASE_SURFACE = [2]


def linear_extrusion(direction, curve):
    return [3, *direction, *curve]


def bspline_curve3(rng):
    """A cubic 3D B-spline with an interior double knot (C1 there)."""
    poles = [[i, rng.uniform(-1, 1), rng.uniform(-1, 1), 1.0] for i in range(7)]
    return bspline(3, [[0.0, 4], [0.4, 1], [0.7, 2], [1.0, 4]], poles)


def surfaces(rng: random.Random) -> list[list]:
    """Surface adaptors of every kind, the spline ones with interior knots."""
    knots_u = [[0.0, 3], [0.5, 1], [1.0, 3]]
    knots_v = [[0.0, 3], [0.25, 2], [1.0, 3]]
    grid = bspline_surface(knots_u, knots_v, (2, 2), rng)
    rational = bspline_surface(knots_u, [[0.0, 2], [0.5, 1], [1.0, 2]], (2, 1), rng, rational=True)
    periodic = bspline_surface([[0.0, 1], [0.25, 1], [0.5, 1], [0.75, 1], [1.0, 1]], knots_v, (2, 2), rng,
                               periodic=(True, False))
    return [
        adapted(plane(F0)), adapted(plane(frame(rng))), adapted(cylinder(F0, 2.0)), adapted(cone(F0, 0.3, 2.0)),
        adapted(sphere(F0, 2.0)), adapted(torus(F0, 5.0, 1.0)), adapted(grid), adapted(rational),
        adapted(periodic), adapted(bezier_surface(3, 4, rng)), adapted(bezier_surface(3, 3, rng, rational=True)),
        adapted(extrusion(Z, bspline_curve3(rng))), adapted(revolution([4.0, 0.0, 0.0, 0.0, 0.0, 1.0],
                                                                       bspline_curve3(rng))),
        adapted(extrusion(unit(rng), circle(frame(rng), 2.0))),
        adapted(revolution([0.0, 0.0, 0.0, 0.0, 0.0, 1.0], line([3.0, 0.0, 0.0], [0.0, 0.0, 1.0]))),
        adapted(offset_surface(0.5, grid)), adapted(offset_surface(0.2, extrusion(Z, bspline_curve3(rng)))),
        adapted(offset_surface(1.0, plane(F0))), adapted(offset_surface(0.5, sphere(F0, 2.0))),
        adapted(rect_trim(0.1, 0.9, 0.2, 0.8, grid)),
        bounded([0.0, 0.5, 0.25, 1.0], [1e-3, 1e-3], grid), bounded([-1.0, 2.0, -1.5, 3.0], [0.0, 0.0], plane(F0)),
        bounded([0.0, 1.5 * math.pi, -1.0, 1.0], [0.0, 0.0], cylinder(F0, 1.5)),
        linear_extrusion(Z, bspline_curve3(rng)), linear_extrusion([1.0, 0.0, 0.0], line([0.0, 0.0, 0.0], [0.0, 1.0, 0.0])),
        BASE_SURFACE,
    ]


def curves2d(rng: random.Random) -> list[list]:
    """2D adaptors: iso lines both ways and reversed, oblique lines, every conic, splines."""
    x, y, mx, my = [1.0, 0.0], [0.0, 1.0], [-1.0, 0.0], [0.0, -1.0]
    return [
        line2d([0.3, 0.2], x, 0.0, 0.4), line2d([0.3, 0.2], mx, -0.1, 0.2), line2d([0.3, 0.2], y, 0.0, 0.5),
        line2d([0.5, 0.25], my, 0.0, 0.2), line2d([0.5, 0.25], x, 0.0, 0.5), line2d([0.5, 0.25], y, 0.0, 0.75),
        line2d([0.0, 0.0], [0.6, 0.8], 0.0, 1.0), line2d([1.0, 0.0], [-0.6, 0.8], 0.0, 1.0),
        ranged(0.0, 0.4, line2([0.5, 0.5], x)), ranged(-0.2, 0.3, line2([0.7, 0.25], y)),
        ranged(0.0, 1.0, line2([0.25, 0.5], [0.8, -0.6])),
        whole(conic2(1, [0.5, 0.5], x, True, 0.3)), whole(conic2(1, [0.5, 0.5], y, False, 0.2)),
        whole(conic2(2, [0.5, 0.5], x, True, 0.4, 0.2)), ranged(-1.0, 1.0, conic2(3, [0.5, 0.5], x, True, 0.2, 0.1)),
        ranged(-1.0, 1.0, conic2(4, [0.5, 0.5], y, False, 0.1)),
        whole(bezier2([[0.1, 0.1, 1.0], [0.5, 0.9, 1.0], [0.9, 0.2, 1.0]])),
        whole(bezier2([[0.1, 0.1, 1.0], [0.5, 0.9, 2.0], [0.9, 0.2, 1.0]], rational=True)),
        whole(bspline2(2, [[0.0, 3], [0.5, 1], [1.0, 3]],
                       [[0.1, 0.1, 1.0], [0.3, 0.8, 1.0], [0.6, 0.2, 1.0], [0.9, 0.9, 1.0]])),
        whole(bspline2(2, [[0.0, 3], [1.0, 3]], [[0.2, 0.2, 1.0], [0.5, 0.9, 1.5], [0.8, 0.3, 1.0]], rational=True)),
        whole(trimmed2(0.1, 0.9, bezier2([[0.0, 0.5, 1.0], [0.5, 0.0, 1.0], [1.0, 0.5, 1.0]]))),
        ranged(0.0, 0.5, line2([rng.uniform(0.1, 0.9), rng.uniform(0.1, 0.9)], unit2(rng))),
    ]


def unit2(rng):
    angle = rng.uniform(-math.pi, math.pi)
    return [math.cos(angle), math.sin(angle)]


def elementary_iso_curves() -> list[list]:
    """Iso lines in the parameter spaces of spheres, cylinders, cones and tori, both ways."""
    x, y, mx, my = [1.0, 0.0], [0.0, 1.0], [-1.0, 0.0], [0.0, -1.0]
    return [
        line2d([0.7, 0.4], x, 0.0, 2.0), line2d([0.7, 0.4], mx, 0.0, 2.0), line2d([0.7, 0.4], y, -0.5, 0.5),
        line2d([0.7, 0.4], my, -0.5, 0.5), line2d([0.7, math.pi / 2], x, 0.0, 1.0),
        line2d([0.7, -math.pi / 2], mx, 0.0, 1.0), line2d([0.2, 0.3], [0.6, 0.8], 0.0, 1.0),
    ]


def append_adaptor3d_requests(requests: list[str]) -> None:
    """Append every Adaptor3d request family in a fixed order."""
    rng = random.Random(0xA3D5)
    all_surfaces = surfaces(rng)
    all_curves = curves2d(rng)
    samples = params([0.0, 0.125, 0.25, 0.5, 0.75, 1.0, -0.2, 1.3])
    for s in all_surfaces:
        for c in all_curves:
            requests.append('a3_cos ' + fmt([*c, *s, *samples, 0.05, 0.3]))
    for s in all_surfaces[2:6]:
        for c in elementary_iso_curves():
            requests.append('a3_cos ' + fmt([*c, *s, *samples, 0.1, 0.4]))
    for i, s in enumerate(all_surfaces):
        c1, c2 = all_curves[i % len(all_curves)], all_curves[(3 * i + 5) % len(all_curves)]
        s2 = all_surfaces[(i + 7) % len(all_surfaces)]
        requests.append('a3_cos_load ' + fmt([*c1, *s, *c2, *s2, *samples]))
    iso_values = [0.0, 0.25, 0.5, 1.0, -0.3, 7.0]
    for s in all_surfaces:
        for iso in (0, 1, 2):
            p = rng.choice(iso_values)
            requests.append('a3_iso ' + fmt([*s, iso, p, 0.1, 0.8, *samples, 0.2, 0.6]))
            requests.append('a3_iso ' + fmt([*s, iso, p, -10.0, 10.0, *samples, -1.0, 2.0]))
    for c in all_curves:
        for fix, value in ((1, 0.5), (2, 0.25), (3, 0.0), (0, 1.0)):
            requests.append('a3_interfunc ' + fmt([*c, value, fix, *params([0.0, 0.1, 0.35, 1.0])]))
    uv = params([0.0, 0.0, 0.25, 0.5, 0.5, 0.25, 1.0, 1.0])
    ranges = params([0.0, 0.5, 0.0, 0.1, 0.2, 1.0, 0.0, 1e-3])
    for s in all_surfaces:
        requests.append('a3_hstool ' + fmt([*s, *uv, *ranges]))
