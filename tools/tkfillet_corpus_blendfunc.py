"""Deterministic requests for the BlendFunc probes (tools/oracle/tkfillet_probe_blendfunc.cxx).

`fl_const_rad` evaluates BlendFunc_ConstRad between elementary surfaces (planes,
cylinders, spheres, cones, tori) along a line or a circle: exact fillet solutions (the
right-angled planes, a plane and a cylinder, a sphere cut by a plane), points off the
solution, every section shape and choice of the solution, trimmed guides, and points
where a surface normal degenerates (the apex of a cone, the pole of a sphere).
"""
from __future__ import annotations
import math
import random

X, Y, Z = (1.0, 0.0, 0.0), (0.0, 1.0, 0.0), (0.0, 0.0, 1.0)
O = (0.0, 0.0, 0.0)


def numbers(values: list[float]) -> str:
    """The fields of a request, with every double spelled exactly."""
    return ' '.join(repr(float(v)) for v in values)


def surface(kind: int, p=O, n=Z, x=X, a: float = 0.0, b: float = 0.0) -> list[float]:
    """A surface spec: kind, frame, two reals (12 numbers)."""
    return [kind, *p, *n, *x, a, b]


def guide(kind: int, p=O, n=X, x=Y, a: float = 0.0) -> list[float]:
    """A guide spec: kind, frame, one real (11 numbers)."""
    return [kind, *p, *n, *x, a]


def const_rad(s1, s2, c, radius, choix, shape, t, x4, tol=1e-7, trim=None) -> str:
    """One `fl_const_rad` request."""
    trim_fields = [1, *trim] if trim else [0, 0.0, 0.0]
    fields = [*s1, *s2, *c, radius, choix, shape, t, *trim_fields, *x4, tol]
    return 'fl_const_rad ' + numbers(fields)


def right_angle_requests(requests: list[str]) -> None:
    """Planes z = 0 and y = 0 along the X axis: the fillet of radius R touches them at
    (t, R, 0) and (t, 0, R), i.e. X = (t, R, t, -R)."""
    s1, s2 = surface(0, n=Z, x=X), surface(0, n=Y, x=X)
    c = guide(0, n=X, x=Y)
    for radius in (1.0, 0.25, 3.5):
        for t in (0.0, 1.5, -2.0):
            exact = [t, radius, t, -radius]
            for choix in range(0, 9):
                requests.append(const_rad(s1, s2, c, radius, choix, choix % 4, t, exact))
            requests.append(const_rad(s1, s2, c, radius, 1, 0, t, [t + 0.1, radius * 1.1, t - 0.05, -radius]))
    requests.append(const_rad(s1, s2, c, 1.0, 1, 0, 0.5, [0.5, 1.0, 0.5, -1.0], trim=(-1.0, 3.0)))


def curved_requests(requests: list[str]) -> None:
    """A plane and a cylinder, a sphere and a plane along a circle, a cone, a torus."""
    plane = surface(0, n=Z, x=X)
    cylinder = surface(1, n=X, x=Y, a=2.0)
    for t, v in ((0.0, 0.3), (1.0, 0.1), (2.5, -0.2)):
        requests.append(const_rad(plane, cylinder, guide(0, p=(0.0, 2.0, 0.0), n=X, x=Y), 0.5, 3, 0, t,
                                  [t, 1.6, t, v]))
    sphere = surface(2, n=Z, x=X, a=2.0)
    cut = surface(0, p=(0.0, 0.0, 1.0), n=Z, x=X)
    circle = guide(1, p=(0.0, 0.0, 1.0), n=Z, x=X, a=math.sqrt(3.0))
    for t in (0.0, 0.7, 2.0, 5.5):
        for choix in (1, 2, 5, 6):
            requests.append(const_rad(sphere, cut, circle, 0.3, choix, 0, t, [t, 0.4, 1.4 * math.cos(t), 1.4 * math.sin(t)]))
    # The pole of the sphere and the apex of a cone: degenerate normals.
    requests.append(const_rad(sphere, cut, circle, 0.3, 1, 1, 0.0, [0.0, math.pi / 2, 1.0, 0.0]))
    cone = surface(3, n=Z, x=X, a=0.5, b=1.0)
    apex = -1.0 / math.tan(0.5)
    requests.append(const_rad(cone, cut, circle, 0.2, 1, 2, 0.3, [0.3, apex, 1.0, 0.5]))
    requests.append(const_rad(cone, plane, guide(0, n=X, x=Y), 0.2, 4, 3, 0.3, [0.3, 0.5, 1.0, 0.5]))
    torus = surface(4, n=Z, x=X, a=3.0, b=1.0)
    requests.append(const_rad(torus, plane, guide(1, n=Z, x=X, a=3.0), 0.4, 6, 0, 1.0, [1.0, -1.0, 2.0, 0.5]))


def random_requests(requests: list[str]) -> None:
    """Seeded points around the right-angled fillet and the sphere cut."""
    rng = random.Random(20261008)
    s1, s2 = surface(0, n=Z, x=X), surface(0, n=Y, x=X)
    c = guide(0, n=X, x=Y)
    for _ in range(12):
        t, radius = rng.uniform(-3.0, 3.0), rng.uniform(0.1, 4.0)
        x4 = [t + rng.uniform(-0.2, 0.2), radius + rng.uniform(-0.3, 0.3), t + rng.uniform(-0.2, 0.2), -radius]
        requests.append(const_rad(s1, s2, c, radius, rng.randint(1, 8), rng.randint(0, 3), t, x4))


def const_rad_inv(s1, s2, c, curve2d, radius, choix, first, x4, tol=1e-7) -> str:
    """One `fl_const_rad_inv` request; `curve2d` is (u, v, du, dv)."""
    return 'fl_const_rad_inv ' + numbers([*s1, *s2, *c, *curve2d, radius, choix, int(first), *x4, tol])


def inverse_requests(requests: list[str]) -> None:
    """The right-angled planes with the fillet ending on an iso line of either plane,
    exact and perturbed, and a sphere cut by a plane."""
    s1, s2 = surface(0, n=Z, x=X), surface(0, n=Y, x=X)
    c = guide(0, n=X, x=Y)
    for radius in (1.0, 0.5):
        for first in (True, False):
            line = (0.0, radius, 1.0, 0.0) if first else (0.0, -radius, 1.0, 0.0)
            for w in (0.0, 0.7, -1.2):
                exact = [w, w, w, -radius if first else radius]
                for choix in (1, 2, 3, 6):
                    requests.append(const_rad_inv(s1, s2, c, line, radius, choix, first, exact))
                requests.append(const_rad_inv(s1, s2, c, line, radius, 1, first, [w + 0.1, w - 0.2, w, 0.3]))
    sphere = surface(2, n=Z, x=X, a=2.0)
    cut = surface(0, p=(0.0, 0.0, 1.0), n=Z, x=X)
    circle = guide(1, p=(0.0, 0.0, 1.0), n=Z, x=X, a=math.sqrt(3.0))
    for first in (True, False):
        requests.append(const_rad_inv(sphere, cut, circle, (0.0, 0.4, 1.0, 0.0), 0.3, 2, first, [0.5, 0.5, 1.2, 0.4]))
        requests.append(const_rad_inv(sphere, cut, circle, (0.0, math.pi / 2, 1.0, 0.0), 0.3, 1, first, [0.0, 0.1, 1.0, 0.0]))


def append_blendfunc_requests(requests: list[str]) -> None:
    """Append the BlendFunc request families."""
    inverse_requests(requests)
    right_angle_requests(requests)
    curved_requests(requests)
    random_requests(requests)
