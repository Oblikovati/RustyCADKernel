"""Deterministic requests for GeomBndLib and BndLib (tools/oracle/tkgeombase_probe_bndlib.cxx).

Every evaluator is reached through the Curve/Curve2d/Surface dispatchers from a Geom
handle, a GeomAdaptor over the natural range and a GeomAdaptor over a sub-range, and
through the BndLib_Add* classes on a box that already holds [-1, 1]^n. The ranges cover
the natural bounds, sub-ranges, reversed and degenerate ranges, ranges beyond the bounds,
infinite ends (+/-1e101, and exactly +/-1e100 on the Precision::Infinite() boundary) and
the invalid doubly infinite ones that raise Standard_Failure. The static gp overloads of
BndLib (on a prefilled box) and GeomBndLib get axis-aligned and tilted frames, arcs across
the period seam, spans beyond 2 Pi, cylinder and cone V ends in every infinite
combination, sphere and torus patches with negative, large and reversed V, and
self-intersecting tori (R < r). The .pxx helpers the tests call are probed directly.
"""
from __future__ import annotations
import math
import random
from tkg3d_corpus import axis, frame, point, unit
from tkgeombase_corpus_gce import add, point2d, unit2d
from tkgeombase_corpus_lprop import curves2d, curves3d, surfaces, to3d

PI = math.pi
INF = 1e101
BOUNDARY = 1e100  # Precision::Infinite(): IsInfinite from here on
TOLS = [0.0, 1e-7, 0.25]


def extra_curves3d(rng):
    """3D specs beyond the LProp set: axis-aligned lines, periodic and trimmed splines,
    and offsets of lines, ellipses and splines (the exact and generic offset paths)."""
    periodic = [6, 2, 1, 4, 0, 0, 0, 1, 2, 0, 1, 1, 2, 2, 1, 1, 0, 2, 1, 1, 5, 0, 1, 1, 1, 2, 1, 3, 1, 4, 1]
    s_curve = to3d(curves2d(rng)[6][1])
    ellipse = [2, *frame(rng, 3.0), 4.0, 1.5]
    return [
        ('line_x', [0, *point(rng, 3.0), 1, 0, 0]),
        ('line_minus_y', [0, *point(rng, 3.0), 0, -1, 0]),
        ('line_z', [0, 0, 0, 0, 0, 0, 1]),
        ('line_diag', [0, 1, 2, 3, -1, -1, -1]),
        ('periodic_bspline', periodic),
        ('trimmed_s', [7, 0.2, 0.7, *s_curve]),
        ('trimmed_ellipse', [7, 0.5, 5.0, *ellipse]),
        ('offset_line', [8, 1.5, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0]),
        ('offset_line_parallel', [8, 1.5, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0]),
        ('offset_ellipse', [8, 0.75, 0, 0, 1, *ellipse]),
        ('offset_circle_inplane', [8, 0.5, 1, 0, 0, 1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 2.0]),
        ('offset_circle_collapse', [8, -2.0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 2.0]),
        ('offset_s', [8, 0.4, 0, 0, 1, *s_curve]),
    ]


def curve_ranges(name):
    """(u1, u2) ranges of a curve; the infinite ones only where the curve allows them."""
    ranges = [(0.0, 1.0), (0.2, 0.7), (0.7, 0.2), (0.5, 0.5), (-0.5, 2.5), (1.0, 7.5), (-3.0, 3.0)]
    if name.startswith(('line', 'hyperbola', 'parabola', 'offset_line')):
        ranges += [(-INF, 2.0), (1.0, INF), (-INF, INF), (INF, -INF), (INF, 3.0), (2.0, -INF),
                   (-BOUNDARY, 1.0), (-INF, -INF), (INF, INF)]
    return ranges


def curve3d_requests(rng, requests):
    specs = [(name, spec) for name, spec, _ in curves3d(rng)] + extra_curves3d(rng)
    for name, spec in specs:
        for u1, u2 in curve_ranges(name):
            for mode in range(16):
                optimal = mode % 4 >= 2
                if optimal and (abs(u1) >= BOUNDARY or abs(u2) >= BOUNDARY):
                    continue
                add(requests, 'bnd_curve3d', mode, u1, u2, TOLS[mode % 3], *spec)
    # A curve that cannot be built: the construction error ends the request.
    add(requests, 'bnd_curve3d', 0, 0.0, 1.0, 0.0, 1, *frame(rng), -1.0)


def extra_curves2d(rng):
    """2D specs beyond the LProp set: offsets of lines and splines, axis-aligned lines."""
    s_curve = curves2d(rng)[6][1]
    return [
        ('line_x', [0, *point2d(rng, 3.0), 1, 0]),
        ('line_minus_y', [0, *point2d(rng, 3.0), 0, -1]),
        ('offset_line', [8, 1.25, 0, 0, 0, *unit2d(rng)]),
        ('offset_s', [8, 0.3, *s_curve]),
        ('offset_trimmed', [8, -0.6, 7, 0.1, 0.8, *s_curve]),
    ]


def curve2d_requests(rng, requests):
    specs = [(name, spec) for name, spec, _ in curves2d(rng)] + extra_curves2d(rng)
    for name, spec in specs:
        for u1, u2 in curve_ranges(name):
            for mode in range(24):
                finite = abs(u1) < BOUNDARY and abs(u2) < BOUNDARY
                optimal = mode % 4 >= 2 and mode // 4 in (0, 1, 2)
                if (optimal or mode // 4 in (3, 4)) and not finite:
                    continue
                # A reversed range sends the legacy math_PSO of OtherCurve2d::BoxOptimal to
                # an uninitialized objective value (native undefined behaviour): skipped.
                legacy = name.startswith('offset') or mode // 4 == 4
                if legacy and mode % 4 == 3 and u1 > u2:
                    continue
                add(requests, 'bnd_curve2d', mode, u1, u2, TOLS[mode % 3], *spec)


def bezier_surface():
    rows = [[0, 0, 0, 1], [0, 1, 1, 1], [0, 2, 0, 1], [1, 0, 1, 1], [1, 1, 3, 2], [1, 2, 1, 1],
            [2, 0, 0, 1], [2, 1, -1, 1], [2, 2, 0.5, 1]]
    return [1, 3, 3, *[c for row in rows for c in row]]


def extra_surfaces(rng):
    """Surface specs (with the probe's prefix code) beyond the LProp set."""
    lprop = {name: spec for name, spec, _ in surfaces(rng)}
    line_z = [0, 2, 0, 0, 0, 0, 1]
    return [
        ('bezier', bezier_surface()),
        ('trimmed_sphere', [2, 0.0, PI, -PI / 4, PI / 4, 0, *lprop['sphere']]),
        ('trimmed_bspline', [2, 0.2, 0.8, 0.1, 0.9, 0, *lprop['bspline_saddle']]),
        ('trimmed_bezier', [2, 0.25, 0.75, 0.0, 0.5, *bezier_surface()]),
        ('plane_z', [0, 0, 1, 2, 3, 0, 0, 1, 1, 0, 0]),
        ('plane_x', [0, 0, 1, 2, 3, 1, 0, 0, 0, 1, 0]),
        ('cylinder_z', [0, 1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 2.0]),
        ('cone_apex', [0, 2, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0.5, 0.0]),
        ('torus_degenerate', [0, 4, *frame(rng, 2.0), 1.0, 2.0]),
        ('offset_torus', [0, 8, 0.5, 4, *frame(rng, 2.0), 3.0, 1.0]),
        ('offset_plane', [0, 8, 2.0, 0, *frame(rng, 2.0)]),
        ('offset_bspline', [0, 8, 0.3, *lprop['bspline_saddle']]),
        ('revolution_line', [0, 6, 0, 0, 0, 0, 0, 1, *line_z]),
        ('extrusion_line', [0, 7, 1, 0, 0, *line_z]),
        ('extrusion_circle', [0, 7, 0, 1, 1, 1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 2.0]),
    ]


def surface_patches(name):
    """(u1, u2, v1, v2) patches: natural, sub, beyond the bounds, degenerate, infinite."""
    patches = [(0.0, 1.0, 0.0, 1.0), (0.2, 0.6, 0.3, 0.9), (-0.5, 1.5, -0.5, 1.5), (0.4, 0.4, 0.1, 0.8),
               (0.0, 2 * PI, -PI / 2, PI / 2), (0.5, 4.0, -1.0, 2.0)]
    if name.startswith(('plane', 'cylinder', 'cone', 'revolution_line', 'extrusion', 'offset_plane')):
        patches += [(-INF, INF, -INF, INF), (0.0, 1.0, -INF, 2.0), (0.0, 1.0, 1.0, INF),
                    (0.0, 1.0, INF, -INF), (0.0, 1.0, -INF, -INF), (-INF, 1.0, 0.0, 2.0)]
    return patches


def surface_requests(rng, requests):
    specs = [(name, [0, *spec]) for name, spec, _ in surfaces(rng)] + extra_surfaces(rng)
    for index, (name, spec) in enumerate(specs):
        trsf = [*axis(rng, 2.0), rng.uniform(-PI, PI), *point(rng, 2.0)]
        for patch in surface_patches(name):
            finite = all(abs(c) < BOUNDARY for c in patch)
            for mode in range(20):
                op, src = mode % 4, mode // 4
                if op >= 2 and not finite:
                    continue
                # The PSO + Powell refinement is costly: one optimal patch per surface.
                if op >= 2 and patch != surface_patches(name)[1 + index % 2]:
                    continue
                add(requests, 'bnd_surface', mode, *patch, TOLS[mode % 3], *trsf, *spec)


def axis_frames():
    """gp_Ax2/gp_Ax3 inputs P N X along the global axes and tilted."""
    return [[0, 0, 0, 0, 0, 1, 1, 0, 0], [1, -2, 3, 0, 1, 0, 0, 0, 1], [0, 0, 0, 1, 0, 0, 0, 1, 0],
            [0.5, 0.5, 0.5, 1, 1, 1, 1, -1, 0], [2, 0, -1, 0, 0, -1, 0, 1, 0]]


def ax2d_frames(rng):
    return [[0, 0, 1, 0, 1], [1, 2, 0, 1, 1], [-1, 3, *unit2d(rng), 0], [0.5, 0.5, -1, 0, 1]]


ARCS = [(0.0, PI / 2), (0.3, 2.9), (5.5, 7.0), (-1.0, 0.5), (-7.0, -5.0), (0.0, 2 * PI), (0.0, 7.0),
        (2.0, 1.0), (1.0, 1.0), (0.0, 1e-9), (3.0, 3.0 + 2 * PI - 1e-9)]
OPEN = [(-4.0, 4.0), (0.5, 2.0), (-2.0, -0.5), (0.0, 0.0), (-INF, 2.0), (1.0, INF), (-INF, INF),
        (INF, -INF), (INF, 1.0), (1.0, -INF), (-INF, -INF), (INF, INF), (-BOUNDARY, 0.0)]
V_ENDS = [(0.0, 10.0), (-5.0, 0.0), (3.0, 7.0), (-INF, 2.0), (1.0, INF), (-INF, INF), (INF, -INF),
          (INF, 2.0), (1.0, -INF), (-INF, -INF), (INF, INF)]


def prefill(npre, flat):
    points = [[-3, 4, 1], [5, -2, -6]][:npre]
    return [c for p in points for c in (p[:2] if flat else p)]


def curve_gp_requests(rng, requests):
    for lib in (0, 1):
        for npre in (0, 2):
            pre3, pre2 = prefill(npre, False), prefill(npre, True)
            for f in axis_frames():
                for u1, u2 in OPEN:
                    add(requests, 'bnd_gp', lib, 0, npre, *pre3, *f[:6], u1, u2, 1e-3)
                    add(requests, 'bnd_gp', lib, 10, npre, *pre3, *f, 1.5, u1, u2, 0.0)
                    add(requests, 'bnd_gp', lib, 12, npre, *pre3, *f, 3.0, 2.0, u1, u2, 0.0)
                add(requests, 'bnd_gp', lib, 2, npre, *pre3, *f, 2.5, 0.0)
                add(requests, 'bnd_gp', lib, 6, npre, *pre3, *f, 4.0, 1.5, 0.1)
                for u1, u2 in ARCS:
                    add(requests, 'bnd_gp', lib, 3, npre, *pre3, *f, 2.5, u1, u2, 0.0)
                    add(requests, 'bnd_gp', lib, 7, npre, *pre3, *f, 4.0, 1.5, u1, u2, 1e-7)
            for a in ax2d_frames(rng):
                for sense in (1, 0):
                    for u1, u2 in OPEN:
                        add(requests, 'bnd_gp', lib, 1, npre, *pre2, *a[:4], u1, u2, 1e-3)
                        add(requests, 'bnd_gp', lib, 11, npre, *pre2, *a[:4], sense, 1.5, u1, u2, 0.0)
                        add(requests, 'bnd_gp', lib, 13, npre, *pre2, *a[:4], sense, 3.0, 2.0, u1, u2, 0.0)
                    add(requests, 'bnd_gp', lib, 4, npre, *pre2, *a[:4], sense, 2.5, 0.0)
                    add(requests, 'bnd_gp', lib, 8, npre, *pre2, *a[:4], sense, 4.0, 1.5, 0.1)
                    for u1, u2 in ARCS:
                        add(requests, 'bnd_gp', lib, 5, npre, *pre2, *a[:4], sense, 2.5, u1, u2, 0.0)
                        add(requests, 'bnd_gp', lib, 9, npre, *pre2, *a[:4], sense, 4.0, 1.5, u1, u2, 1e-7)
    # Invalid radii: the gp constructors raise before BndLib runs.
    add(requests, 'bnd_gp', 0, 2, 0, *axis_frames()[0], -1.0, 0.0)
    add(requests, 'bnd_gp', 1, 12, 0, *axis_frames()[0], 1.0, -2.0, 0.0, 1.0, 0.0)


SPHERE_PATCHES = [(0.0, 2 * PI, -PI / 2, PI / 2), (0.0, 2 * PI, 0.0, PI / 2), (0.0, PI / 2, 0.0, PI / 2),
                  (1.0, 4.0, -1.0, 0.3), (-2.0, 0.5, -0.2, 1.2), (5.0, 6.0, -PI / 2, -1.0), (0.0, 1e-3, 0.0, 1e-3)]
TORUS_V = [(0.0, PI), (-PI, 0.0), (-3 * PI, -2 * PI), (-PI / 8, PI / 8), (2.0, 1.0), (0.0, 2 * PI),
           (1.0, 1.0), (7.0, 13.0), (-0.1, 0.1)]


def surface_gp_requests(requests):
    for npre in (0, 2):
        pre = prefill(npre, False)
        for f in axis_frames():
            for v1, v2 in V_ENDS:
                for u1, u2 in [(0.0, 2 * PI), (0.0, PI / 2), (1.0, 3.0)]:
                    add(requests, 'bnd_gp', 0, 14, npre, *pre, *f, 2.0, u1, u2, v1, v2, 0.0)
                    add(requests, 'bnd_gp', 0, 16, npre, *pre, *f, 0.4, 1.0, u1, u2, v1, v2, 1e-7)
                add(requests, 'bnd_gp', 0, 15, npre, *pre, *f, 2.0, v1, v2, 0.1)
                add(requests, 'bnd_gp', 0, 17, npre, *pre, *f, 0.4, 1.0, v1, v2, 0.0)
            add(requests, 'bnd_gp', 0, 18, npre, *pre, *f, 3.0, 0.5)
            add(requests, 'bnd_gp', 0, 20, npre, *pre, *f, 4.0, 1.0, 0.0)
            add(requests, 'bnd_gp', 0, 20, npre, *pre, *f, 1.0, 3.0, 0.0)
            for patch in SPHERE_PATCHES:
                add(requests, 'bnd_gp', 0, 19, npre, *pre, *f, 3.0, *patch, 1e-7)
            for u1, u2 in [(0.0, 2 * PI), (0.0, PI / 2), (1.0, 3.0), (-1.0, 0.5)]:
                for v1, v2 in TORUS_V:
                    add(requests, 'bnd_gp', 0, 21, npre, *pre, *f, 4.0, 1.0, u1, u2, v1, v2, 0.0)
                    add(requests, 'bnd_gp', 0, 21, npre, *pre, *f, 1.0, 3.0, u1, u2, v1, v2, 0.0)


def helper_requests(rng, requests):
    c3 = {name: spec for name, spec, _ in curves3d(rng)}
    c2 = {name: spec for name, spec, _ in curves2d(rng)}
    s = {name: [0, *spec] for name, spec, _ in surfaces(rng)}
    for name in ('bezier_s', 'bspline_c1', 'helix_like', 'circle', 'offset_circle'):
        for u1, u2 in [(0.0, 1.0), (0.1, 0.4), (0.3, 0.305), (1.0, 2.5)]:
            for index in (1, 2, 3):
                add(requests, 'bnd_helpers', 1, u1, u2, 0, 0, 0.5, 1e-7, index, 3, *c3[name])
            add(requests, 'bnd_helpers', 0, u1, u2, 0, 0, 0, 0, 1, 3, *c3[name])
            add(requests, 'bnd_helpers', 2, u1, u2, 0, 0, 0, 1e-7, 1, 3, *c3[name])
            for n in (1, 3, 7):
                add(requests, 'bnd_helpers', 3, u1, u2, 0, 0, 0, 0, 1, n, *c3[name])
    for name in ('bezier_s', 'bspline_c2', 'ellipse', 'offset_ellipse'):
        for u1, u2 in [(0.0, 1.0), (0.1, 0.4), (0.3, 0.305)]:
            for index in (1, 2):
                add(requests, 'bnd_helpers', 5, u1, u2, 0, 0, -0.5, 1e-7, index, 0, *c2[name])
            add(requests, 'bnd_helpers', 4, u1, u2, 0, 0, 0, 0, 1, 0, *c2[name])
            add(requests, 'bnd_helpers', 6, u1, u2, 0, 0, 0, 1e-7, 1, 0, *c2[name])
    for name in ('bspline_saddle', 'torus', 'sphere'):
        for patch in [(0.0, 1.0, 0.0, 1.0), (0.2, 0.3, 0.4, 0.45)]:
            add(requests, 'bnd_helpers', 7, *patch, 0, 0, 1, 0, *s[name])
            for index in (1, 3):
                add(requests, 'bnd_helpers', 8, *patch, 0.25, 1e-7, index, 0, *s[name])
    add(requests, 'bnd_helpers', 7, 0.0, 1.0, 0.0, 1.0, 0, 0, 1, 0, *bezier_surface())


def append_bndlib_requests(requests: list[str]) -> None:
    """Append every GeomBndLib and BndLib request family in a fixed order."""
    rng = random.Random(20260928)
    curve3d_requests(rng, requests)
    curve2d_requests(rng, requests)
    surface_requests(rng, requests)
    curve_gp_requests(rng, requests)
    surface_gp_requests(requests)
    helper_requests(rng, requests)
