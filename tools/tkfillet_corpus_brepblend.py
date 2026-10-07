"""Deterministic requests for the BRepBlend probes (tools/oracle/tkfillet_probe_brepblend.cxx).

`fl_rad_inv` evaluates the radius inverses of BRepBlend (the fillets between a surface and
a restriction ending on a point or a curve, and the restriction-restriction point
inverse) on the plane z = 0 along the X axis, with the second curve the line
(t, 0, 1): exact solutions for the radius 1, perturbed points, every choice and radius
law, and curved surfaces. `fl_surf_rst` and `fl_rst_rst` evaluate the restriction fillets.
`fl_blend_tool` projects points on 2d lines and circles and
intersects segments with them.
"""
from __future__ import annotations
import math
import random

from tkfillet_corpus_blendfunc import X, Y, Z, guide, numbers, surface

LAWS = ((0, -3.0, 1.0, 3.0, 1.0, 1.0), (1, -3.0, 0.5, 3.0, 2.0, 0.0),
        (2, -3.0, 0.5, 3.0, 2.0, 0.0), (3, -3.0, 0.5, 3.0, 1.5, 2.0))


def rad_inv(kind, s, c, c2, rst, law, radius, choix, p, x3, tol=1e-7) -> str:
    """One `fl_rad_inv` request; `rst` is (u, v, du, dv), `law` (kind, p1, r1, p2, r2, r3)."""
    return 'fl_rad_inv ' + numbers([kind, *s, *c, *c2, *rst, *law, 0.0, radius, choix, *p, *x3, tol])


def plane_requests(requests: list[str]) -> None:
    """The exact sections of radius 1 through (t, 0, 1), and perturbed ones."""
    plane, c = surface(0, n=Z, x=X), guide(0, n=X, x=Y)
    c2, rst = guide(0, p=(0.0, 0.0, 1.0), n=X, x=Y), (0.0, 1.0, 1.0, 0.0)
    for t in (0.0, 0.8, -1.7):
        exact = {0: [t, t, 1.0], 1: [t, t, 1.0], 2: [t, t, t], 3: [t, t, t], 4: [t, t, 0.0]}
        for kind in range(5):
            for law in LAWS[:1] if kind in (0, 2, 4) else LAWS:
                for choix in (1, 3, 5):
                    requests.append(rad_inv(kind, plane, c, c2, rst, law, 1.0, choix, (t, 0.0, 1.0), exact[kind]))
            x3 = [v + 0.13 for v in exact[kind]]
            requests.append(rad_inv(kind, plane, c, c2, rst, LAWS[1], 1.0, 1, (t, 0.0, 1.0), x3))


def curved_requests(requests: list[str]) -> None:
    """A sphere and a cylinder with a circular guide: every derivative term is alive."""
    sphere = surface(2, n=Z, x=X, a=2.0)
    cylinder = surface(1, n=Z, x=X, a=1.5)
    circle = guide(1, p=(0.0, 0.0, 1.0), n=Z, x=X, a=math.sqrt(3.0))
    c2 = guide(1, p=(0.0, 0.0, 1.2), n=Z, x=Y, a=1.1)
    for s in (sphere, cylinder):
        for kind in range(5):
            for law in (LAWS[0], LAWS[3]):
                x3 = [0.6, 0.4, 0.3]
                requests.append(rad_inv(kind, s, circle, c2, (0.2, 0.3, 0.6, 0.8), law, 0.4, 3, (1.0, 0.5, 1.2), x3))


def random_requests(requests: list[str]) -> None:
    rng = random.Random(20261010)
    plane, c = surface(0, n=Z, x=X), guide(0, n=X, x=Y)
    c2 = guide(0, p=(0.0, 0.0, 1.0), n=X, x=Y)
    for _ in range(20):
        kind, t = rng.randint(0, 4), rng.uniform(-2.0, 2.0)
        x3 = [t + rng.uniform(-0.3, 0.3), t + rng.uniform(-0.3, 0.3), rng.uniform(0.5, 1.5)]
        law = LAWS[rng.randint(0, 3)]
        rst = (0.0, rng.uniform(0.5, 1.5), 1.0, rng.uniform(-0.2, 0.2))
        requests.append(rad_inv(kind, plane, c, c2, rst, law, rng.uniform(0.5, 2.0), rng.randint(1, 8),
                                (t, rng.uniform(-0.2, 0.2), 1.0), x3))


def blend_tool(curve2d, p1, p2, c, u0, u1) -> str:
    """One `fl_blend_tool` request; `curve2d` is (kind, a, b, c, d)."""
    return 'fl_blend_tool ' + numbers([*curve2d, *p1, *p2, *c, u0, u1])


def blend_tool_requests(requests: list[str]) -> None:
    line2d, circle2d = (0, 0.0, 1.0, 1.0, 0.0), (1, 0.0, 0.0, 2.0, 0.0)
    line, circle = guide(0, n=X, x=Y), guide(1, n=Z, x=X, a=1.0)
    for curve2d in (line2d, circle2d):
        for p1, p2 in (((0.3, 0.2), (0.3, 3.0)), ((3.0, 0.5), (-3.0, 0.5)), ((0.5, 0.5), (0.5, 0.5)),
                       ((5.0, 5.0), (6.0, 7.0))):
            for c in (line, circle):
                requests.append(blend_tool(curve2d, p1, p2, c, 0.0, 1.0))


def surf_rst(kind, s, srst, rst, c, law, radius, choix, shape, t, x3, tol=1e-7, trim=None) -> str:
    """One `fl_surf_rst` request; `rst` is (u, v, du, dv) in the parameters of `srst`."""
    trim_fields = [1, *trim] if trim else [0, 0.0, 0.0]
    return 'fl_surf_rst ' + numbers([kind, *s, *srst, *rst, *c, *law, 0.0, radius, choix, shape, t,
                                     *trim_fields, *x3, tol])


def surf_rst_requests(requests: list[str]) -> None:
    """The plane z = 0 and the line (w, 0, 1) of the plane y = 0: the fillet of radius
    0.625 touches the plane at (t, 0.5, 0); perturbed points, every shape and choice, trims,
    and a cylinder against a sphere along a circle."""
    plane, wall = surface(0, n=Z, x=X), surface(0, n=Y, x=X)
    rst, c = (0.0, -1.0, 1.0, 0.0), guide(0, n=X, x=Y)
    exact_law = (0, -3.0, 0.625, 3.0, 0.625, 0.625)
    for kind in (0, 1):
        for t in (0.0, 1.25, -2.0):
            exact = [t, 0.5, t]
            for choix in (1, 2, 3, 4):
                for shape in (0, 3):
                    requests.append(surf_rst(kind, plane, wall, rst, c, exact_law, 0.625, choix, shape, t, exact))
            requests.append(surf_rst(kind, plane, wall, rst, c, LAWS[1], 0.625, 1, 1, t, [t + 0.1, 0.55, t - 0.05]))
        requests.append(surf_rst(kind, plane, wall, rst, c, LAWS[3], 0.625, 1, 2, 0.5, [0.5, 0.5, 0.5], trim=(-1.0, 2.0)))
    cylinder = surface(1, n=Z, x=X, a=1.5)
    sphere = surface(2, n=Z, x=X, a=2.0)
    circle = guide(1, p=(0.0, 0.0, 1.0), n=Z, x=X, a=math.sqrt(3.0))
    for kind in (0, 1):
        for law in (LAWS[0], LAWS[2]):
            requests.append(surf_rst(kind, cylinder, sphere, (0.2, 0.5, 1.0, 0.1), circle, law, 0.3, 3, 0, 0.4,
                                     [0.4, 0.8, 0.3]))


def rst_rst(kind, s1, rst1, s2, rst2, c, law, radius, choix, shape, t, x2, tol=1e-7, trim=None) -> str:
    """One `fl_rst_rst` request; the restrictions are (u, v, du, dv) in their surfaces."""
    trim_fields = [1, *trim] if trim else [0, 0.0, 0.0]
    return 'fl_rst_rst ' + numbers([kind, *s1, *rst1, *s2, *rst2, *c, *law, 0.0, radius, choix, shape, t,
                                    *trim_fields, *x2, tol])


def rst_rst_requests(requests: list[str]) -> None:
    """The lines (w, 1, 0) of z = 0 and (w, 0, 1) of y = 0 along the X axis: the circle of
    radius 1 through both at every t (X = (t, t)); every choice and shape, perturbed
    points, points too far apart for the radius, trims, and a sphere against a cylinder."""
    plane, wall = surface(0, n=Z, x=X), surface(0, n=Y, x=X)
    rst1, rst2, c = (0.0, 1.0, 1.0, 0.0), (0.0, -1.0, 1.0, 0.0), guide(0, n=X, x=Y)
    unit_law = (0, -3.0, 1.0, 3.0, 1.0, 1.0)
    for kind in (0, 1):
        for t in (0.0, 0.9, -1.4):
            for choix in (1, 2, 3, 4):
                for shape in (0, 3):
                    requests.append(rst_rst(kind, plane, rst1, wall, rst2, c, unit_law, 1.0, choix, shape, t, [t, t]))
            requests.append(rst_rst(kind, plane, rst1, wall, rst2, c, LAWS[1], 1.0, 1, 1, t, [t + 0.2, t - 0.1]))
            requests.append(rst_rst(kind, plane, rst1, wall, rst2, c, LAWS[1], 0.5, 3, 0, t, [t, t]))
        requests.append(rst_rst(kind, plane, rst1, wall, rst2, c, LAWS[3], 1.0, 2, 2, 0.5, [0.5, 0.5], trim=(-1.0, 2.0)))
    sphere = surface(2, n=Z, x=X, a=2.0)
    cylinder = surface(1, n=Z, x=X, a=1.5)
    circle = guide(1, p=(0.0, 0.0, 1.0), n=Z, x=X, a=math.sqrt(3.0))
    for kind in (0, 1):
        requests.append(rst_rst(kind, sphere, (0.0, 0.5, 1.0, 0.0), cylinder, (0.0, 0.8, 1.0, 0.0), circle,
                                LAWS[2], 0.9, 1, 0, 0.3, [0.35, 0.25]))


def append_brepblend_requests(requests: list[str]) -> None:
    """Append the BRepBlend request families."""
    plane_requests(requests)
    curved_requests(requests)
    random_requests(requests)
    blend_tool_requests(requests)
    surf_rst_requests(requests)
    rst_rst_requests(requests)
