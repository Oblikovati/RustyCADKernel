"""Deterministic requests for the gce package (tools/oracle/tkgeombase_probe_gce.cxx).

Every constructor overload of every gce_Make* class gets random valid inputs plus the
degenerate ones that reach each gce_ErrorType and each raising gp constructor:
coincident points (exact, within gp::Resolution() and denormal), collinear points,
null and negative radii, null, right and negative angles, directions along the
coordinate axes (every branch of the X-direction choices), parallel axes, and
magnitudes near 1e-150 and 1e150 where squares underflow or overflow.
"""
from __future__ import annotations
import math
import random
from tkg2d_corpus import fmt
from tkg3d_corpus import axis, frame, point, unit

RES = 2.2250738585072014e-308  # gp::Resolution() = DBL_MIN
TINY = [0.0, 0.25 * RES, RES, 2.0 * RES, 1e-160, 1e-150, 1e-7]
AXES = [[1, 0, 0], [0, 1, 0], [0, 0, 1], [-1, 0, 0], [0, -1, 0], [0, 0, -1],
        [1, 1, 0], [0, 1, 1], [1, 0, 1], [1, 1, 1], [1, -1, 0], [0, 1, -1], [-1, 0, 1], [3, 2, 1],
        [1, 2, 3], [2, 3, 1], [1e-3, 1, 0], [1, 1e-9, 1e-9]]


def normalized(v):
    n = math.sqrt(sum(c * c for c in v))
    return [c / n for c in v]


def point2d(rng: random.Random, scale=10.0):
    return [rng.uniform(-scale, scale), rng.uniform(-scale, scale)]


def unit2d(rng: random.Random):
    a = rng.uniform(-math.pi, math.pi)
    return [math.cos(a), math.sin(a)]


def frame2d(rng: random.Random):
    """A gp_Ax22d input P X Y with Y away from X."""
    x = unit2d(rng)
    sign = rng.choice([1.0, -1.0])
    return [*point2d(rng), *x, -sign * x[1], sign * x[0]]


def axis_frames(rng: random.Random):
    """Frames whose normals run along the axis-aligned and mixed directions of AXES."""
    frames = []
    for n in AXES:
        n = normalized(n)
        x = normalized([n[1] - n[2], n[2] - n[0], n[0] - n[1]]) if len(set(n)) > 1 else [1.0, -1.0, 0.0]
        frames.append([*point(rng, 3.0), *n, *normalized(x)])
    return frames


def add(requests, op, *values):
    requests.append(op + ' ' + fmt(list(values)))


def near(p, delta):
    return [c + delta for c in p]


def circ_requests(rng, requests):
    for f in [frame(rng) for _ in range(4)] + axis_frames(rng)[:6]:
        for radius in [2.5, 0.0, -1e-300, -1.0, 1e-160]:
            add(requests, 'gce_circ', 0, *f, radius)
            add(requests, 'gce_circ', 1, *f, 3.0, radius - 3.0)
        add(requests, 'gce_circ', 1, *f, 3.0, -3.0)
        add(requests, 'gce_circ', 2, *f, 3.0, *point(rng))
        add(requests, 'gce_circ', 2, *f, 3.0, *f[0:3])
    for n in AXES:
        c = point(rng, 3.0)
        n = normalized(n)
        for radius in [1.5, 0.0, -2.0]:
            add(requests, 'gce_circ', 4, *c, *n, radius)
            add(requests, 'gce_circ', 5, *c, *point(rng), *n, radius)
            add(requests, 'gce_circ', 7, *c, *n, radius)
            add(requests, 'gce_circ', 6, *c, *[a + 2.0 * b for a, b in zip(c, n)], radius)
        add(requests, 'gce_circ', 6, *c, *[a + 1e-300 * b for a, b in zip(c, n)], 1.0)
        add(requests, 'gce_circ', 6, *c, *c, 1.0)
        add(requests, 'gce_circ', 6, *c, *c, -1.0)
    three_point_requests(rng, requests, 'gce_circ', 3)


def three_point_requests(rng, requests, op, kind):
    """Three points in 3D: generic, collinear, confused, coincident and tiny ones."""
    for _ in range(12):
        add(requests, op, kind, *point(rng), *point(rng), *point(rng))
    a, b = point(rng), point(rng)
    mid = [(x + y) / 2 for x, y in zip(a, b)]
    add(requests, op, kind, *a, *b, *mid)
    add(requests, op, kind, *a, *b, *[2 * y - x for x, y in zip(a, b)])
    add(requests, op, kind, *a, *a, *b)
    add(requests, op, kind, *a, *b, *a)
    add(requests, op, kind, *a, *b, *b)
    add(requests, op, kind, *a, *a, *a)
    for delta in TINY:
        add(requests, op, kind, 0.0, 0.0, 0.0, delta, 0.0, 0.0, 0.0, delta, 0.0)
        add(requests, op, kind, 0.0, 0.0, 0.0, delta, 0.0, 0.0, 0.0, 1.0, 0.0)
        add(requests, op, kind, *a, *near(a, delta), *b)
    add(requests, op, kind, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0)
    add(requests, op, kind, 1e150, 0.0, 0.0, 0.0, 1e150, 0.0, 0.0, 0.0, 1e150)
    add(requests, op, kind, 1.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 1.0, 0.0)
    add(requests, op, kind, 0.0, 0.0, 0.0, 1.0, 1e-9, 0.0, 2.0, 0.0, 0.0)


def three_point2d_requests(rng, requests, op, kind):
    """Three points in 2D: generic, collinear, confused, coincident and tiny ones."""
    for _ in range(12):
        add(requests, op, kind, *point2d(rng), *point2d(rng), *point2d(rng))
    a, b = point2d(rng), point2d(rng)
    add(requests, op, kind, *a, *b, *[(x + y) / 2 for x, y in zip(a, b)])
    add(requests, op, kind, *a, *b, *[2 * y - x for x, y in zip(a, b)])
    add(requests, op, kind, *a, *a, *b)
    add(requests, op, kind, *a, *b, *a)
    add(requests, op, kind, *a, *b, *b)
    add(requests, op, kind, *a, *a, *a)
    for delta in TINY:
        add(requests, op, kind, 0.0, 0.0, delta, 0.0, 0.0, delta)
        add(requests, op, kind, 0.0, 0.0, delta, 0.0, 0.0, 1.0)
        add(requests, op, kind, 0.0, 0.0, 0.0, 1.0, delta, 0.0)
        add(requests, op, kind, *a, *near(a, delta), *b)
    for pts in ([0, 0, 1, 0, 0, 1], [0, 0, 0, 1, 1, 0], [1, 0, 0, 1, -1, 0], [0, 0, 1, 1, 2, 0],
                [0, 0, 2e-150, 0, 0, 1], [1e150, 0, 0, 1e150, -1e150, 0], [3, 4, -4, 3, 0, 5]):
        add(requests, op, kind, *[float(v) for v in pts])


def circ2d_requests(rng, requests):
    for _ in range(4):
        ax = [*point2d(rng), *unit2d(rng)]
        f = frame2d(rng)
        for radius in [2.0, 0.0, -0.5]:
            for sense in (0, 1):
                add(requests, 'gce_circ2d', 0, *ax, radius, sense)
                add(requests, 'gce_circ2d', 5, *ax[0:2], radius, sense)
            add(requests, 'gce_circ2d', 1, *f, radius)
            add(requests, 'gce_circ2d', 2, *f, 1.5, radius)
        add(requests, 'gce_circ2d', 2, *f, 1.5, -1.5)
        add(requests, 'gce_circ2d', 2, *f, 1.5, -4.0)
        add(requests, 'gce_circ2d', 3, *f, 1.5, *point2d(rng))
        add(requests, 'gce_circ2d', 3, *f, 1.5, *f[0:2])
        for sense in (0, 1):
            add(requests, 'gce_circ2d', 6, *ax[0:2], *point2d(rng), sense)
            add(requests, 'gce_circ2d', 6, *ax[0:2], *ax[0:2], sense)
    three_point2d_requests(rng, requests, 'gce_circ2d', 4)


def cone_requests(rng, requests):
    frames = [frame(rng) for _ in range(3)] + axis_frames(rng)[:4]
    for f in frames:
        for angle, radius in [(0.5, 2.0), (0.5, 0.0), (0.5, -1.0), (0.0, 1.0), (RES, 1.0), (-0.3, 1.0),
                              (math.pi / 2, 1.0), (math.pi / 2 - 1e-300, 1.0), (1.5, 1.0), (2.0, 1.0)]:
            add(requests, 'gce_cone', 0, *f, angle, radius)
        for angle in [0.5, -0.5, 1.2, 1e-3]:
            for radius in [0.0, 3.0]:
                for _ in range(2):
                    add(requests, 'gce_cone', 1, *f, angle, radius, *point(rng))
                add(requests, 'gce_cone', 1, *f, angle, radius, *f[0:3])
                for dist in [1.0, -1.0, -10.0, 0.0]:
                    add(requests, 'gce_cone', 2, *f, angle, radius, dist)
    four_point_cone_requests(rng, requests)
    for _ in range(6):
        a, b = point(rng), point(rng)
        for r1, r2 in [(1.0, 2.0), (2.0, 1.0), (1.0, 1.0), (0.0, 1.0), (-1.0, 1.0), (1.0, -1.0), (1e20, 0.0)]:
            add(requests, 'gce_cone', 6, *a, *b, r1, r2)
        add(requests, 'gce_cone', 6, *a, *a, 1.0, 2.0)
    for d in AXES:
        add(requests, 'gce_cone', 6, 0.0, 0.0, 0.0, *normalized(d), 1.0, 2.0)


def four_point_cone_requests(rng, requests):
    """gce_MakeCone(P1..P4) and the axis/line overloads that delegate to it."""
    for _ in range(10):
        pts = [point(rng) for _ in range(4)]
        add(requests, 'gce_cone', 3, *[c for p in pts for c in p])
        add(requests, 'gce_cone', 4, *axis(rng), *pts[2], *pts[3])
        add(requests, 'gce_cone', 5, *axis(rng), *pts[2], *pts[3])
    o, z = [0.0, 0.0, 0.0], [0.0, 0.0, 10.0]
    for p3, p4 in [([4, 0, 0], [2, 0, 5]), ([3, 0, 0], [3, 0, 5]), ([3, 0, 2], [5, 0, 2]), ([3, 0, 2], [3, 0, 2]),
                   ([0, 0, 0], [2, 0, 5]), ([4, 0, 0], [0, 0, 0]), ([0, 0, 3], [0, 0, 6]), ([1, 1, 0], [2, 2, 1e-300]),
                   ([4, 0, 0], [4, 0, 0]), ([2, 0, -3], [5, 0, 4]), ([1, 0, 0], [0, 1, 5])]:
        add(requests, 'gce_cone', 3, *o, *z, *[float(v) for v in p3], *[float(v) for v in p4])
    add(requests, 'gce_cone', 3, *o, *o, 1.0, 0.0, 0.0, 2.0, 0.0, 1.0)
    for d in AXES:
        n = normalized(d)
        add(requests, 'gce_cone', 4, 0.0, 0.0, 0.0, *n, 3.0, 1.0, 0.5, 1.0, 2.0, 4.0)
        add(requests, 'gce_cone', 5, 1.0, 2.0, 3.0, *n, 3.0, 1.0, 0.5, 1.0, 2.0, 4.0)


def cylinder_requests(rng, requests):
    frames = [frame(rng) for _ in range(3)] + axis_frames(rng)
    for f in frames:
        for radius in [2.0, 0.0, -1.0]:
            add(requests, 'gce_cylinder', 0, *f, radius)
            add(requests, 'gce_cylinder', 5, *f, abs(radius))
            add(requests, 'gce_cylinder', 4, *f[0:6], radius)
            add(requests, 'gce_cylinder', 2, *f, 3.0, radius - 3.0)
        add(requests, 'gce_cylinder', 1, *f, 3.0, *point(rng))
        add(requests, 'gce_cylinder', 1, *f, 3.0, *f[0:3])
    three_point_requests(rng, requests, 'gce_cylinder', 3)


def dir_requests(rng, requests):
    vectors = [point(rng) for _ in range(4)] + [[0.0, 0.0, 0.0], [RES, 0.0, 0.0], [1e-160, 0.0, 0.0],
                                                   [1e-150, 1e-150, 0.0], [1e-300, 0.0, 0.0], [1e200, 1e200, 0.0],
                                                   [0.0, -2.0, 0.0], [1e-154, 0.0, 0.0]]
    for v in vectors:
        for kind in (0, 1, 2):
            add(requests, 'gce_dir', kind, *v)
        add(requests, 'gce_dir', 3, 1.0, 2.0, 3.0, *[1.0 + v[0], 2.0 + v[1], 3.0 + v[2]])
        add(requests, 'gce_dir', 3, 0.0, 0.0, 0.0, *v)
        v2 = v[0:2]
        for kind in (0, 1, 2):
            add(requests, 'gce_dir2d', kind, *v2)
        add(requests, 'gce_dir2d', 3, 1.0, 2.0, *[1.0 + v2[0], 2.0 + v2[1]])
        add(requests, 'gce_dir2d', 3, 0.0, 0.0, *v2)


def conic_requests(rng, requests):
    radii = [(10.0, 5.0), (5.0, 5.0), (3.0, 7.0), (0.0, 0.0), (5.0, 0.0), (5.0, -1.0), (-1.0, 5.0), (-3.0, -2.0),
             (-1.0, -3.0)]
    for f in [frame(rng) for _ in range(3)] + axis_frames(rng)[:3]:
        for major, minor in radii:
            add(requests, 'gce_elips', 0, *f, major, minor)
            add(requests, 'gce_hypr', 0, *f, major, minor)
    for _ in range(3):
        ax, f2 = [*point2d(rng), *unit2d(rng)], frame2d(rng)
        for major, minor in radii:
            for sense in (0, 1):
                add(requests, 'gce_elips2d', 0, *ax, major, minor, sense)
                add(requests, 'gce_hypr2d', 1, *ax, major, minor, sense)
            add(requests, 'gce_elips2d', 1, *f2, major, minor)
            add(requests, 'gce_hypr2d', 2, *f2, major, minor)
    three_point_requests(rng, requests, 'gce_elips', 1)
    three_point_requests(rng, requests, 'gce_hypr', 1)
    three_point2d_requests(rng, requests, 'gce_elips2d', 2)
    three_point2d_requests(rng, requests, 'gce_hypr2d', 0)
    for s1, s2 in [([10, 0, 0], [0, 5, 0]), ([10, 0, 0], [5, 0, 0]), ([3, 0, 0], [0, 7, 0]), ([10, 0, 0], [3, 4, 0]),
                   ([2 * RES, 0, 0], [0, 1.1 * RES, 0]), ([1, 0, 0], [1, 1e-300, 0]), ([10, 0, 0], [20, 5, 0])]:
        add(requests, 'gce_elips', 1, *[float(v) for v in s1], *[float(v) for v in s2], 0.0, 0.0, 0.0)
        add(requests, 'gce_hypr', 1, *[float(v) for v in s1], *[float(v) for v in s2], 0.0, 0.0, 0.0)
        add(requests, 'gce_elips2d', 2, *[float(v) for v in s1[0:2]], *[float(v) for v in s2[0:2]], 0.0, 0.0)
        add(requests, 'gce_hypr2d', 0, *[float(v) for v in s1[0:2]], *[float(v) for v in s2[0:2]], 0.0, 0.0)


def lin_requests(rng, requests):
    for _ in range(4):
        a = axis(rng)
        add(requests, 'gce_lin', 0, *a)
        add(requests, 'gce_lin', 1, *a)
        add(requests, 'gce_lin', 3, *a, *point(rng))
        a2 = [*point2d(rng), *unit2d(rng)]
        add(requests, 'gce_lin2d', 0, *a2)
        add(requests, 'gce_lin2d', 1, *a2)
        add(requests, 'gce_lin2d', 4, *a2, *point2d(rng))
        for dist in [2.5, 0.0, -1.0]:
            add(requests, 'gce_lin2d', 3, *a2, dist)
    for delta in TINY + [1.0, -3.0]:
        p = point(rng)
        add(requests, 'gce_lin', 2, *p, *near(p, delta))
        add(requests, 'gce_lin2d', 5, *p[0:2], *near(p[0:2], delta))
    for a, b, c in [(1.0, 2.0, 3.0), (0.0, 1.0, -2.0), (3.0, 4.0, 5.0), (0.0, 0.0, 1.0), (1e-160, 0.0, 1.0),
                    (1e-150, 1e-150, 0.0), (RES, 0.0, 0.0), (-2.0, 0.0, 1.0), (1e200, 0.0, 0.0)]:
        add(requests, 'gce_lin2d', 2, a, b, c)


def parab_requests(rng, requests):
    for f in [frame(rng) for _ in range(3)]:
        for focal in [1.5, 0.0, -1.0]:
            add(requests, 'gce_parab', 0, *f, focal)
    for _ in range(4):
        a = axis(rng)
        add(requests, 'gce_parab', 1, *a, *point(rng))
        add(requests, 'gce_parab', 1, *a, *a[0:3])
        add(requests, 'gce_parab', 1, *a, *[p + 3.0 * d for p, d in zip(a[0:3], a[3:6])])
        a2, f2 = [*point2d(rng), *unit2d(rng)], frame2d(rng)
        for focal in [1.5, 0.0, -1.0]:
            for sense in (0, 1):
                add(requests, 'gce_parab2d', 0, *a2, focal, sense)
            add(requests, 'gce_parab2d', 1, *f2, focal)
        for sense in (0, 1):
            add(requests, 'gce_parab2d', 2, *a2, *point2d(rng), sense)
            add(requests, 'gce_parab2d', 2, *a2, *a2[0:2], sense)
            add(requests, 'gce_parab2d', 3, *point2d(rng), *a2[0:2], sense)
            add(requests, 'gce_parab2d', 3, *a2[0:2], *a2[0:2], sense)
            add(requests, 'gce_parab2d', 3, *near(a2[0:2], 1e-300), *a2[0:2], sense)


def pln_requests(rng, requests):
    for f in [frame(rng) for _ in range(3)] + axis_frames(rng)[:3]:
        add(requests, 'gce_pln', 0, *f)
        add(requests, 'gce_pln', 1, *f[0:6])
        add(requests, 'gce_pln', 7, *f[0:6])
        add(requests, 'gce_pln', 3, *f, *point(rng))
        for dist in [2.0, 0.0, -1.5]:
            add(requests, 'gce_pln', 4, *f, dist)
    for coefficients in [(1.0, 2.0, 3.0, 4.0), (0.0, 0.0, 1.0, 0.0), (0.0, 0.0, 0.0, 1.0), (1e-160, 0.0, 0.0, 1.0),
                         (1e-150, 1e-150, 1e-150, 0.0), (-2.0, 0.0, 0.0, 3.0), (1e200, 0.0, 0.0, 0.0)]:
        add(requests, 'gce_pln', 2, *coefficients)
    three_point_requests(rng, requests, 'gce_pln', 5)
    for delta in TINY + [2.0]:
        p = point(rng)
        add(requests, 'gce_pln', 6, *p, *near(p, delta))


def transformation_requests(rng, requests):
    for _ in range(3):
        a, f = axis(rng), frame(rng)
        add(requests, 'gce_mirror', 0, *point(rng))
        add(requests, 'gce_mirror', 1, *a)
        add(requests, 'gce_mirror', 2, *a)
        add(requests, 'gce_mirror', 3, *a)
        add(requests, 'gce_mirror', 4, *f)
        add(requests, 'gce_mirror', 5, *f)
        a2 = [*point2d(rng), *unit2d(rng)]
        for kind in range(4):
            add(requests, 'gce_mirror2d', kind, *(a2[0:2] if kind == 0 else a2))
        for angle in [0.7, 0.0, -2.0 * math.pi, 1e10]:
            for kind in range(3):
                add(requests, 'gce_rotation', kind, *a, angle)
            add(requests, 'gce_rotation2d', *a2[0:2], angle)
        for scale in [2.0, -1.0, 1.0, 0.0, 1e-300, -0.5]:
            add(requests, 'gce_scale', *point(rng), scale)
            add(requests, 'gce_scale2d', *point2d(rng), scale)
        add(requests, 'gce_translation', 0, *point(rng))
        add(requests, 'gce_translation', 1, *point(rng), *point(rng))
        add(requests, 'gce_translation2d', 0, *point2d(rng))
        add(requests, 'gce_translation2d', 1, *point2d(rng), *point2d(rng))


def append_gce_requests(requests: list[str]) -> None:
    """Append the gce requests in a fixed order."""
    rng = random.Random(0x6CE)
    requests.append('gce_status')
    circ_requests(rng, requests)
    circ2d_requests(rng, requests)
    cone_requests(rng, requests)
    cylinder_requests(rng, requests)
    dir_requests(rng, requests)
    conic_requests(rng, requests)
    lin_requests(rng, requests)
    parab_requests(rng, requests)
    pln_requests(rng, requests)
    transformation_requests(rng, requests)
