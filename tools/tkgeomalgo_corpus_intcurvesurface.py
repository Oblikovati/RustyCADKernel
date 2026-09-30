"""Deterministic requests for the IntCurveSurface package
(tools/oracle/tkgeomalgo_probe_intcurvesurface*.cxx).

The geometry catalog below encodes curves and surfaces for `ga_ics_curve` and
`ga_ics_surface`: lines, conics, polynomial and rational Bezier and B-spline curves
(C1 knots, periodic, degree 1), offset and trimmed curves; planes, cylinders, cones,
spheres, tori, Bezier and B-spline surfaces (several C2 patches), surfaces of
revolution and extrusion of lines, parabolas, hyperbolas and Bezier curves, offset
surfaces of each, and trimmed surfaces, with natural (infinite) and given domains.

This module holds the records (`ics_records`, `ics_intersection`), the curve tool
(`ics_hcurve`), the polygon (`ics_polygon`), the polyhedron (`ics_polyhedron`) and the
header utilities (`ics_utils`); tkgeomalgo_corpus_intcurvesurface_inter.py adds the
algorithm families.
"""
from __future__ import annotations
import math
import random

PI = math.pi
TWO_PI = 2.0 * PI
HALF_PI = PI / 2.0
O, X, Y, Z = [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]


def line_of(op: str, values: list[float]) -> str:
    return op + ' ' + ' '.join(repr(float(v)) for v in values)


def flat(points: list[list[float]]) -> list[float]:
    return [c for p in points for c in p]


# Curves (ga_ics_geom_curve) and curve adaptors (ga_ics_curve).
def line(p, d): return [0, *p, *d]
def circle(p, d, r): return [1, *p, *d, r]
def ellipse(p, d, a, b): return [2, *p, *d, a, b]
def parabola(p, d, f): return [3, *p, *d, f]
def hyperbola(p, d, a, b): return [4, *p, *d, a, b]


def bezier(poles, weights=None):
    return [5, len(poles), *flat(poles), *([1, *weights] if weights else [0])]


def bspline(deg, poles, knots, mults, weights=None, periodic=False):
    return [6, deg, len(poles), *flat(poles), *([1, *weights] if weights else [0]),
            len(knots), *knots, *mults, 1 if periodic else 0]


def offset_curve(off, d, basis): return [7, off, *d, *basis]
def trimmed(u1, u2, basis): return [8, u1, u2, *basis]
def ad(geom, rng=None): return [*geom, 0] if rng is None else [*geom, 1, *rng]


# Surfaces (ga_ics_geom_surface) and surface adaptors (ga_ics_surface).
def plane(p, d): return [0, *p, *d]
def cyl(p, d, r): return [1, *p, *d, r]
def cone(p, d, ang, r): return [2, *p, *d, ang, r]
def sphere(p, d, r): return [3, *p, *d, r]
def torus(p, d, big, small): return [4, *p, *d, big, small]


def bez_s(grid):
    return [5, len(grid), len(grid[0]), *[c for row in grid for p in row for c in p]]


def bsp_s(ud, vd, grid, uk, um, vk, vm):
    return [6, ud, vd, len(grid), len(grid[0]), *[c for row in grid for p in row for c in p],
            len(uk), *uk, *um, len(vk), *vk, *vm]


def revol(p, d, curve): return [7, *p, *d, *curve]
def extr(d, curve): return [8, *d, *curve]
def offs(off, surf): return [9, off, *surf]
def rtrim(u1, u2, v1, v2, surf): return [10, u1, u2, v1, v2, *surf]
def sad(geom, bounds=None): return [*geom, 0] if bounds is None else [*geom, 1, *bounds]


BEZ_POLES = [[-2.0, -0.5, 0.8], [-0.7, 1.2, -0.9], [0.8, -1.1, 1.1], [2.1, 0.6, -0.4]]
BSP_POLES = [[-3.0, 0.0, -1.0], [-2.0, 1.0, 1.5], [-0.5, -1.0, -1.2], [0.5, 1.2, 1.3], [2.0, -0.8, -0.9],
             [3.0, 0.5, 1.0]]
BSP_KNOTS, BSP_MULTS = [0.0, 1.0, 2.0, 3.0], [4, 1, 1, 4]
CURVES = {
    'line': ad(line([-3.0, 0.2, 0.3], X), (0.0, 6.0)),
    'line_nat': ad(line([-3.0, 0.2, 0.3], [1.0, 0.1, 0.05])),
    'line_z': ad(line([0.3, 0.2, -3.0], Z), (0.0, 6.0)),
    'line_diag': ad(line([-2.0, -2.0, -2.0], [1.0, 1.0, 1.0]), (0.0, 7.0)),
    'line_tangent': ad(line([-3.0, 0.0, 1.5], X), (0.0, 6.0)),
    'line_in_plane': ad(line([-1.0, -1.0, 0.3], [1.0, 1.0, 0.0]), (0.0, 3.0)),
    'circle': ad(circle([0.0, 0.0, 0.5], Z, 1.2)),
    'circle_arc': ad(circle([0.0, 0.0, 0.5], [0.3, 0.1, 1.0], 1.2), (0.3, 2.5)),
    'circle_x': ad(circle([0.0, 0.1, 0.0], X, 1.7)),
    'ellipse': ad(ellipse([0.1, 0.0, -0.2], [0.2, 1.0, 0.4], 2.0, 0.9)),
    'parabola': ad(parabola([0.0, -0.5, 0.0], Y, 0.4), (-2.5, 2.5)),
    'hyperbola': ad(hyperbola([0.0, 0.0, 0.1], [0.0, 1.0, 1.0], 0.8, 0.5), (-1.5, 1.5)),
    'bezier': ad(bezier(BEZ_POLES)),
    'bezier_rat': ad(bezier(BEZ_POLES, [1.0, 2.0, 0.5, 1.0])),
    'bspline': ad(bspline(3, BSP_POLES, BSP_KNOTS, BSP_MULTS)),
    'bspline_c1': ad(bspline(3, BSP_POLES + [[4.0, 1.0, 0.0]], [0.0, 1.0, 2.0], [4, 3, 4])),
    'bspline_rat': ad(bspline(2, BSP_POLES[:5], [0.0, 0.5, 1.0, 2.0], [3, 1, 1, 3], [1.0, 1.5, 0.7, 1.2, 1.0])),
    'bspline_per': ad(bspline(2, [[1.5, 0.0, 0.2], [0.0, 1.5, -0.3], [-1.5, 0.0, 0.4], [0.0, -1.5, -0.1]],
                              [0.0, 1.0, 2.0, 3.0, 4.0], [1, 1, 1, 1, 1], periodic=True)),
    'polyline': ad(bspline(1, BSP_POLES[:4], [0.0, 1.0, 2.0, 3.0], [2, 1, 1, 2])),
    'offset_circle': ad(offset_curve(0.3, Z, circle([0.0, 0.0, 0.2], Z, 1.1))),
    'trimmed_parab': ad(trimmed(-1.0, 2.0, parabola([0.0, 0.0, -0.6], [1.0, 0.0, 1.0], 0.5))),
}

SURFACE_GRID = [[[-2.0 + i, -2.0 + j, 0.4 * math.sin(i + 2 * j)] for j in range(4)] for i in range(4)]
SURFACES = {
    'plane': sad(plane([0.0, 0.0, 0.3], Z), (-2.0, 2.0, -2.0, 2.0)),
    'plane_nat': sad(plane([0.0, 0.0, 0.3], Z)),
    'plane_tilt': sad(plane([0.1, 0.0, 0.0], [1.0, 0.2, 0.4]), (-4.0, 4.0, -4.0, 4.0)),
    'cyl': sad(cyl(O, Z, 1.0), (0.0, TWO_PI, -2.0, 2.0)),
    'cyl_nat': sad(cyl([0.2, 0.0, 0.0], [0.1, 1.0, 0.0], 0.8)),
    'cone': sad(cone([0.0, 0.0, -1.0], Z, 0.4, 0.8), (0.0, TWO_PI, -2.0, 3.0)),
    'cone_nat': sad(cone(O, Z, 0.6, 0.5)),
    'sphere': sad(sphere(O, Z, 1.5)),
    'sphere_part': sad(sphere([0.3, 0.0, 0.0], Z, 1.0), (0.5, 4.0, -0.5, 1.0)),
    'torus': sad(torus(O, Z, 3.0, 1.0)),
    'bezier_s': sad(bez_s(SURFACE_GRID)),
    'bspline_s': sad(bsp_s(2, 2, SURFACE_GRID, [0.0, 1.0, 2.0], [3, 1, 3], [0.0, 0.5, 1.0], [3, 1, 3])),
    'rev_line': sad(revol(O, Z, line([1.0, 0.0, 0.0], [0.3, 0.0, 1.0]))),
    'rev_parab': sad(revol(O, Z, parabola([1.0, 0.0, 0.0], Y, 0.5))),
    'rev_hypr': sad(revol(O, Z, hyperbola([0.0, 0.0, 0.0], Y, 1.0, 0.7))),
    'rev_bezier': sad(revol(O, Z, bezier([[1.0, 0.0, -1.0], [2.0, 0.0, 0.0], [0.8, 0.0, 1.5]]))),
    'ext_line': sad(extr(Z, line([0.0, 0.0, 0.0], [1.0, 0.5, 0.0]))),
    'ext_parab': sad(extr(Z, parabola(O, Z, 0.5))),
    'ext_hypr': sad(extr(Z, hyperbola(O, Z, 1.0, 0.6))),
    'ext_bezier': sad(extr([0.0, 0.2, 1.0], bezier([[-2.0, 0.0, 0.0], [0.0, 2.0, 0.0], [2.0, -1.0, 0.0]]))),
    'off_plane': sad(offs(0.4, plane(O, Z))),
    'off_cyl': sad(offs(0.3, cyl(O, Z, 1.0))),
    'off_cone': sad(offs(0.2, cone(O, Z, 0.5, 1.0))),
    'off_sphere': sad(offs(0.5, sphere(O, Z, 1.0))),
    'off_ext': sad(offs(0.3, extr(Z, parabola(O, Z, 0.5)))),
    'off_rev': sad(offs(0.2, revol(O, Z, hyperbola([0.0, 0.0, 0.0], Y, 1.0, 0.7)))),
    'off_bspline': sad(offs(0.2, bsp_s(2, 2, SURFACE_GRID, [0.0, 1.0, 2.0], [3, 1, 3], [0.0, 0.3, 1.0], [3, 1, 3]))),
    'trim_cyl': sad(rtrim(0.5, 3.0, -1.0, 1.0, cyl(O, Z, 1.0))),
    'trim_plane': sad(rtrim(-1.0, 1.5, -0.5, 2.0, plane([0.0, 0.0, 0.3], Z))),
}


def ipoint(rng: random.Random) -> list[float]:
    return [round(rng.uniform(-2, 2), 3) for _ in range(3)] + [round(rng.uniform(-1, 1), 3) for _ in range(3)] + \
        [rng.randint(0, 2)]


def _records(requests: list[str]) -> None:
    rng = random.Random(9031)
    fixed = [[1.0, 2.0, 3.0, 0.5, 0.6, 0.7, 1, 5.0, 6.0, 7.0, 1.0, 2.0, 3.0, 2],
             [0.0, -0.0, 1e-300, -0.0, 0.0, 1e300, 0, 1e-7, 123456.789, -3.25, 0.1, 0.2, 0.3, 1]]
    for values in fixed:
        requests.append(line_of('ics_records', values))
    for _ in range(6):
        requests.append(line_of('ics_records', ipoint(rng) + ipoint(rng)))


def _pt(t: int, values: list[float], tr: int) -> list[float]:
    """Command 0 on target `t`: append the point `values` (p, u, v, w) with transition `tr`."""
    return [t, 0, *values, tr]


def _seg(t: int, a: list[float], b: list[float]) -> list[float]:
    """Command 1: append the segment of two In points."""
    return [t, 1, *a, 1, *b, 2]


def _intersection_scripts(requests: list[str]) -> None:
    base = [0.1, 0.2, 0.3, 0.5, 0.25, 1.0]
    near = [0.1, 0.2, 0.3, 0.5 + 5e-9, 0.25 - 5e-9, 1.0 + 5e-9]
    far = [0.1, 0.2, 0.3, 0.5 + 2e-8, 0.25, 1.0]
    flags = lambda t, d, p: [t, 2, d, p]  # noqa: E731
    show = lambda t: [t, 6]  # noqa: E731
    scripts = [
        [*show(0), *show(1)],
        [*flags(0, 1, 0), *_pt(0, base, 1), *_pt(0, near, 1), *_pt(0, near, 2), *_pt(0, far, 1), *show(0)],
        [*flags(0, 1, 1), *_pt(0, base, 1), *_seg(0, base, far), *show(0), 0, 3, *show(0), 0, 3, *show(0)],
        [*_pt(0, base, 0), *_pt(0, far, 0), *flags(1, 1, 0), *_pt(1, base, 0), *_pt(1, base, 1), 1, 4, *show(1),
         *flags(1, 1, 1), 0, 4, *show(0)],
        [*flags(1, 1, 0), *_pt(1, base, 0), *_pt(1, near, 0), *_seg(1, base, near), *flags(0, 1, 1), *_pt(0, near, 0),
         *_pt(0, far, 1), 1, 5, 0.0, 1.0, *show(1), 0, 5, 2.0, 3.0, *show(0)],
        [*flags(0, 0, 1), *_pt(0, base, 1), *flags(1, 1, 0), *_pt(1, far, 2), 1, 5, 0.0, 1.0, *show(1), 1, 3, *show(1),
         *flags(1, 0, 0), 0, 4, *show(0)],
    ]
    for script in scripts:
        requests.append(line_of('ics_intersection', [_count_commands(script), *script]))


def _count_commands(script: list[float]) -> int:
    """The number of commands of an intersection script (argument counts by command)."""
    sizes = {0: 7, 1: 14, 2: 2, 3: 0, 4: 0, 5: 2, 6: 0}
    i, count = 0, 0
    while i < len(script):
        assert int(script[i]) in (0, 1), script
        i += 2 + sizes[int(script[i + 1])]
        count += 1
    assert i == len(script), script
    return count


def _hcurve(requests: list[str]) -> None:
    ranges = [(0.0, 1.0), (0.3, 2.7), (1.5, 1.5), (-1.0, 4.0), (0.25, 0.75)]
    for k, (name, c) in enumerate(CURVES.items()):
        for j, (u0, u1) in enumerate(ranges):
            u = [0.2, 1.1, 2.9, -0.5, 0.7][(k + j) % 5]
            defl, nbmin = [(0.1, 10), (1e-3, 5), (1.0, 1), (0.05, 30), (0.2, 0)][(k * 3 + j) % 5]
            requests.append(line_of('ics_hcurve', c + [u, u0, u1, defl, nbmin]))


def _approx_queries(n: int) -> list[float]:
    queries = [(-1, 0.5), (0, 0.5), (1, 0.0), (1, 0.3), (2, 1.0), (n, 0.0), (n, 0.4), (n + 1, 0.0), (n + 1, 0.5),
               (2, -0.25), (3, 1.5), (n + 5, 0.2)]
    return [len(queries), *[v for q in queries for v in q]]


def _polygon(requests: list[str]) -> None:
    modes = [[0, 2], [0, 7], [0, 33], [1, 0.25, 1.75, 6], [1, 1.0, 1.0, 5], [2, 4, 0.0, 0.5, 1.5, 2.0],
             [2, 1, 0.5], [2, 2, 0.0, 1.0], [2, 5, 2.0, 1.5, 1.0, 0.5, 0.0], [2, 6, 0.0, 0.2, 0.2, 0.9, 1.1, 1.3]]
    for k, (name, c) in enumerate(CURVES.items()):
        for j, mode in enumerate(modes):
            if (k + j) % 2 and j not in (0, 6, 9):
                continue
            n = mode[1] if mode[0] == 0 else (mode[3] if mode[0] == 1 else mode[1])
            requests.append(line_of('ics_polygon', c + mode + _approx_queries(max(n, 5)) + [0.05 * (j % 3)]))


def _polyhedron(requests: list[str]) -> None:
    grids = [[0, 1, 2, -1.0, -1.0, 1.0, 1.0], [0, 4, 3, -0.5, -1.5, 2.0, 1.0], [0, 5, 5, 0.0, 0.0, 1.0, 1.0],
             [1, 2, -1.0, 1.0, 2, -1.0, 1.0], [1, 4, 0.0, 0.3, 0.9, 1.0, 3, 0.0, 0.5, 1.0], [1, 1, 0.0, 2, 0.0, 1.0]]
    for k, (name, s) in enumerate(SURFACES.items()):
        for j, grid in enumerate(grids):
            flags = [(k + j) % 2, (k // 2 + j) % 2, j % 2, 1]
            flec = [0.0, 1e-5, 0.5, 2e-4][(k + j) % 4]
            requests.append(line_of('ics_polyhedron', s + _surface_grid(name, grid) + [1, 0.2, -0.1, 0.35] + flags +
                                    [flec]))
        big = [0, 40, 25, 0.0, 0.0, 1.0, 1.0]
        requests.append(line_of('ics_polyhedron', s + _surface_grid(name, big) + [0, 0.0, 0.0, 0.0, 0, 0, 0, 0, 0.0]))


def _surface_grid(name: str, grid: list[float]) -> list[float]:
    """A grid on the parameter box of a periodic or infinite surface, scaled from [0, 1]."""
    scale = {'cyl': (TWO_PI, 4.0, 0.0, -2.0), 'cyl_nat': (TWO_PI, 4.0, 0.0, -2.0), 'cone': (TWO_PI, 3.0, 0.0, -1.0),
             'cone_nat': (TWO_PI, 3.0, 0.0, -1.0), 'sphere': (TWO_PI, PI, 0.0, -HALF_PI),
             'sphere_part': (3.5, 1.5, 0.5, -0.5), 'torus': (TWO_PI, TWO_PI, 0.0, 0.0),
             'rev_line': (TWO_PI, 3.0, 0.0, -1.0), 'rev_parab': (TWO_PI, 3.0, 0.0, -1.5),
             'rev_hypr': (TWO_PI, 2.0, 0.0, -1.0), 'rev_bezier': (TWO_PI, 1.0, 0.0, 0.0),
             'off_cyl': (TWO_PI, 4.0, 0.0, -2.0), 'off_cone': (TWO_PI, 3.0, 0.0, -1.0),
             'off_sphere': (TWO_PI, 3.0, 0.0, -1.5), 'off_rev': (TWO_PI, 2.0, 0.0, -1.0),
             'trim_cyl': (2.5, 2.0, 0.5, -1.0), 'trim_plane': (2.5, 2.5, -1.0, -0.5)}.get(name, (1.0, 1.0, 0.0, 0.0))
    su, sv, ou, ov = scale
    mapped = list(grid)
    if grid[0] == 0:
        mapped[3], mapped[5] = ou + su * grid[3], ou + su * grid[5]
        mapped[4], mapped[6] = ov + sv * grid[4], ov + sv * grid[6]
        return mapped
    nu = int(grid[1])
    us = [ou + su * u for u in grid[2:2 + nu]]
    vs = [ov + sv * v for v in grid[3 + nu:]]
    return [1, nu, *us, len(vs), *vs]


def _utils(requests: list[str]) -> None:
    rng = random.Random(4402)
    triangles = [[O, X, Y], [O, X, [2.0, 0.0, 0.0]], [O, O, Y], [O, [1e-8, 0.0, 0.0], Y], [X, Y, Z],
                 [[0.0, 0.0, 0.0], [1.0, 1e-9, 0.0], [2.0, 0.0, 0.0]], [[1e150, 0.0, 0.0], [0.0, 1e150, 0.0], Z]]
    for tri in triangles:
        for p in ([0.2, 0.2, 0.0], [1.0, 1.0, 1.0], [0.5, 0.0, 0.0], X):
            requests.append(line_of('ics_utils', [0, *flat(tri)]))
            requests.append(line_of('ics_utils', [1, *flat(tri), *p]))
            requests.append(line_of('ics_utils', [2, *flat(tri), *p]))
    for index in (-3, 0, 1, 2, 5, 6, 17, 18, 19, 100):
        for nv in (1, 3, 4):
            requests.append(line_of('ics_utils', [3, index, nv]))
    for _ in range(120):
        nu, nv = rng.randint(1, 5), rng.randint(1, 5)
        t, pv, pe = rng.randint(-1, 2 * nu * nv + 1), rng.randint(0, (nu + 1) * (nv + 1) + 1), \
            rng.choice([0, rng.randint(0, (nu + 1) * (nv + 1) + 1)])
        requests.append(line_of('ics_utils', [4, t, pv, pe, nu, nv]))
    for nu, nv in ((1, 1), (3, 3), (0, 4), (40, 25)):
        requests.append(line_of('ics_utils', [5, nu, nv]))
    for flec in (-1.0, 0.0, 9e-5, 1e-4, 0.3):
        requests.append(line_of('ics_utils', [6, flec]))
    for index, t in ((1, 0.5), (4, 0.0), (5, 0.0), (5, 0.5), (2, -0.1), (2, 1.2), (0, 0.5), (7, 0.3)):
        requests.append(line_of('ics_utils', [7, index, t, 0.0, 2.0, 5, 0]))
        requests.append(line_of('ics_utils', [7, index, t, 0.0, 2.0, 5, 5, 0.0, 0.1, 0.5, 1.5, 2.0]))


def append_intcurvesurface_requests(requests: list[str]) -> None:
    """Append every IntCurveSurface request family in a fixed order."""
    _records(requests)
    _intersection_scripts(requests)
    _hcurve(requests)
    _polygon(requests)
    _polyhedron(requests)
    _utils(requests)
    __import__('tkgeomalgo_corpus_intcurvesurface_inter').append_inter_requests(requests)
