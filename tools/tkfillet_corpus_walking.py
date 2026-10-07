"""Deterministic requests for the walking probe (tools/oracle/tkfillet_probe_walking.cxx).

`fl_walking` walks the constant-radius fillet (BlendFunc_ConstRad and its inverse) between
two trimmed surfaces whose domains are Adaptor3d_TopolTool: the right-angled planes of a
box edge along the X axis (exact sections, stops on one restriction, on both, past them),
tilted planes, and a cylinder on a plane along a circle (a real march with the deflection
tests). The modes continue or complete the walking, and start it on the restrictions.
"""
from __future__ import annotations
import math
import random

from tkfillet_corpus_blendfunc import X, Y, Z, guide, numbers, surface

FLAGS = (1, 1, 1, 1)


def walking(mode, s1, b1, s2, b2, g, gbounds, run, x, *, vertices=(), saved=(0, 0), flags=FLAGS,
            pextra=0.0, jalons=()) -> str:
    """One `fl_walking` request; `run` is (radius, choix, pdep, pmax, maxstep, tol3d,
    tolguide, fleche, appro), `vertices` (point, direction) pairs, `jalons` (t, u1, v1,
    u2, v2) tuples."""
    spine = [*g, *gbounds, len(vertices)]
    for p, d in vertices:
        spine += [*p, *d]
    fields = [mode, *s1, *b1, *s2, *b2, *spine, *saved, *run, *x, *flags, pextra, len(jalons)]
    for jalon in jalons:
        fields += list(jalon)
    return 'fl_walking ' + numbers(fields)


def box_edge(l1=10.0, l2=10.0):
    """The planes z = 0 (u in [0, l1], v in [0, 5]) and y = 0 (u in [0, l2], v in [-5, 0])
    along the X axis on [-1, 11]: the fillet of radius R is X = (t, R, t, -R)."""
    s1, s2 = surface(0, n=Z, x=X), surface(0, n=Y, x=X)
    return s1, (0.0, l1, 0.0, 5.0), s2, (0.0, l2, -5.0, 0.0), guide(0, n=X, x=Z), (-1.0, 11.0)


def box_requests(requests: list[str]) -> None:
    """The box edge: every choice, the stops on one or both restrictions, every mode."""
    s1, b1, s2, b2, g, gb = box_edge()
    for choix in range(1, 9):
        requests.append(walking(0, s1, b1, s2, b2, g, gb, (1.0, choix, 1.0, 12.0, 2.0, 1e-6, 1e-5, 0.01, 1), (1.0, 1.0, 1.0, -1.0)))
    for l1, l2 in ((10.0, 8.0), (7.0, 10.0), (6.5, 6.5)):
        s1, b1, s2, b2, g, gb = box_edge(l1, l2)
        for pmax, maxstep in ((12.0, 2.0), (12.0, 0.3), (-1.0, 1.5)):
            pdep = 3.0 if pmax > 0 else 5.0
            run = (1.0, 1, pdep, pmax, maxstep, 1e-6, 1e-5, 0.01, 1)
            requests.append(walking(0, s1, b1, s2, b2, g, gb, run, (pdep, 1.0, pdep, -1.0)))
    s1, b1, s2, b2, g, gb = box_edge(10.0, 8.0)
    run = (1.0, 1, 3.0, 12.0, 2.0, 1e-6, 1e-5, 0.01, 1)
    for mode, pextra in ((1, -2.0), (1, 0.5), (2, 14.0), (2, -3.0), (2, 5.0), (3, 14.0), (4, 14.0)):
        requests.append(walking(mode, s1, b1, s2, b2, g, gb, run, (3.0, 1.0, 3.0, -1.0), pextra=pextra))
    exact = (1.0, 1, 2.0, 12.0, 2.0, 1e-6, 1e-5, 0.01, 0)
    requests.append(walking(0, s1, b1, s2, b2, g, gb, exact, (2.0, 1.0, 2.0, -1.0)))
    requests.append(walking(0, s1, b1, s2, b2, g, gb, exact, (2.0, 1.1, 2.0, -1.0)))
    for flags in ((0, 1, 1, 1), (1, 0, 1, 1), (0, 0, 1, 1), (1, 1, 0, 1), (1, 1, 1, 0)):
        requests.append(walking(0, s1, b1, s2, b2, g, gb, run, (3.0, 1.0, 3.0, -1.0), flags=flags))


def box_guide_requests(requests: list[str]) -> None:
    """Guide vertices and saved bounds (the corrections of the ends), singular points,
    and the first sections."""
    s1, b1, s2, b2, g, gb = box_edge()
    run = (1.0, 1, 3.0, 12.0, 2.0, 1e-6, 1e-5, 0.01, 1)
    x = (3.0, 1.0, 3.0, -1.0)
    ends = (((10.0, 0.0, 0.0), X), ((0.0, 0.0, 0.0), X))
    for vertices, saved in (((), (1, 1)), (ends, (0, 0)), (ends, (1, 1)), (ends[:1], (0, 1))):
        requests.append(walking(0, s1, b1, s2, b2, g, (0.0, 10.0), run, x, vertices=vertices, saved=saved))
    s1b, b1b, s2b, b2b, _, _ = box_edge(10.0, 8.0)
    requests.append(walking(0, s1b, b1b, s2b, b2b, g, gb, run, x, vertices=(((8.0, 0.0, 0.0), X),)))
    requests.append(walking(0, s1b, b1b, s2b, b2b, g, gb, run, x, vertices=(((8.0, 0.0, 0.0), Y),)))
    tilted = (((8.0, 0.0, 0.0), (math.cos(0.0005), math.sin(0.0005), 0.0)),)
    requests.append(walking(0, s1b, b1b, s2b, b2b, g, gb, run, x, vertices=tilted))
    for jalons in (((10.0, 10.0, 1.0, 10.0, -1.0),), ((5.0, 5.0, 1.0, 5.0, -1.0), (10.0, 10.0, 1.0, 10.0, -1.0)),
                   ((10.2, 10.2, 1.0, 10.2, -1.0), (9.9, 9.9, 1.0, 9.9, -1.0), (12.0, 12.0, 1.0, 12.0, -1.0))):
        requests.append(walking(0, s1, b1, s2, b2, g, gb, run, x, jalons=jalons))
    for pdep, xs in ((1.0, (1.0, 1.0, 1.0, -1.0)), (12.0, (12.0, 1.0, 12.0, -1.0)), (4.0, (4.2, 0.8, 3.9, -1.3))):
        requests.append(walking(5, s1, b1, s2, b2, g, gb, (1.0, 1, pdep, 12.0, 2.0, 1e-6, 1e-5, 0.01, 1), xs))
    for pdep, pmax, pextra in ((9.5, 12.0, -1.0), (9.98, 12.0, -1.0), (0.3, -1.0, 11.0), (5.0, 12.0, -1.0)):
        run6 = (1.0, 1, pdep, pmax, 2.0, 1e-6, 1e-5, 0.01, 1)
        requests.append(walking(6, s1b, b1b, s2b, b2b, g, gb, run6, (pdep, 1.0, pdep, -1.0), pextra=pextra))


def tilted_requests(requests: list[str]) -> None:
    """Planes z = 0 and a plane through the X axis at the angle a from y = 0."""
    for a in (0.3, -0.4, 0.9):
        s1 = surface(0, n=Z, x=X)
        s2 = surface(0, n=(0.0, math.cos(a), math.sin(a)), x=X)
        b2 = (0.0, 10.0, -5.0, 5.0)
        g = guide(0, n=X, x=Z)
        for choix in (1, 2, 3, 4, 5, 6, 7, 8):
            run = (0.7, choix, 2.0, 12.0, 1.0, 1e-6, 1e-5, 0.01, 1)
            requests.append(walking(0, s1, (0.0, 10.0, -5.0, 5.0), s2, b2, g, (-1.0, 11.0), run, (2.0, 0.7, 2.0, -0.7)))


def cylinder_requests(requests: list[str]) -> None:
    """The cylinder of radius 5 (u in [0, u1], v in [0, 4]) on the plane z = 0 along the
    circle of radius 5: the fillet of radius R touches them at (t, R) and at the radius
    5 + R."""
    cylinder = surface(1, n=Z, x=X, a=5.0)
    plane = surface(0, n=Z, x=X)
    circle = guide(1, n=Z, x=X, a=5.0)
    gb = (0.0, 2.0 * math.pi)
    for u1, pb in ((2.0, (-10.0, 10.0, -10.0, 10.0)), (4.0, (-10.0, 10.0, -10.0, 3.0)), (1.0, (-10.0, 10.0, -10.0, 10.0))):
        for radius, fleche, maxstep in ((1.0, 0.01, 0.5), (0.5, 0.001, 1.0), (1.5, 0.1, 0.2)):
            t = 0.2
            x = (t, radius, (5.0 + radius) * math.cos(t), (5.0 + radius) * math.sin(t))
            run = (radius, 1, t, 3.5, maxstep, 1e-6, 1e-5, fleche, 1)
            requests.append(walking(0, cylinder, (0.0, u1, 0.0, 4.0), plane, pb, circle, gb, run, x))
    for choix in range(1, 9):
        x = (0.5, 1.0, 6.0 * math.cos(0.5), 6.0 * math.sin(0.5))
        run = (1.0, choix, 0.5, 3.5, 0.5, 1e-6, 1e-5, 0.01, 1)
        requests.append(walking(0, cylinder, (0.0, 2.0, 0.0, 4.0), plane, (-10.0, 10.0, -10.0, 10.0), circle, gb, run, x))
    for mode, pextra in ((1, -0.5), (2, 2.5), (3, 2.8), (4, 2.8)):
        x = (1.0, 1.0, 6.0 * math.cos(1.0), 6.0 * math.sin(1.0))
        run = (1.0, 1, 1.0, 1.5, 0.5, 1e-6, 1e-5, 0.01, 1)
        requests.append(walking(mode, cylinder, (0.0, 2.0, 0.0, 4.0), plane, (-10.0, 10.0, -10.0, 10.0), circle, gb, run, x,
                                pextra=pextra))
    x = (1.5, 1.0, 6.0 * math.cos(1.5), 6.0 * math.sin(1.5))
    run = (1.0, 1, 1.5, -0.5, 0.5, 1e-6, 1e-5, 0.01, 1)
    requests.append(walking(0, cylinder, (0.2, 2.0, 0.0, 4.0), plane, (-10.0, 10.0, -10.0, 10.0), circle, gb, run, x))
    x = (1.9, 1.0, 6.0 * math.cos(1.9), 6.0 * math.sin(1.9))
    run = (1.0, 1, 1.9, 3.0, 0.5, 1e-6, 1e-5, 0.01, 1)
    requests.append(walking(6, cylinder, (0.0, 2.0, 0.0, 4.0), plane, (-10.0, 10.0, -10.0, 10.0), circle, gb, run, x, pextra=0.3))


def random_requests(requests: list[str]) -> None:
    """Seeded box edges: lengths, radii, starts, steps, deflections and perturbed starts."""
    rng = random.Random(20261007)
    for _ in range(24):
        l1, l2 = rng.uniform(4.0, 10.0), rng.uniform(4.0, 10.0)
        s1, b1, s2, b2, g, gb = box_edge(l1, l2)
        radius = rng.uniform(0.3, 2.0)
        pdep = rng.uniform(0.5, min(l1, l2) - 0.5)
        pmax = rng.choice((12.0, -1.0, rng.uniform(-1.0, 12.0)))
        run = (radius, 1, pdep, pmax, rng.uniform(0.1, 3.0), 1e-6, 1e-5, rng.choice((0.001, 0.01, 0.1)), 1)
        x = (pdep + rng.uniform(-0.2, 0.2), radius + rng.uniform(-0.2, 0.2), pdep, -radius + rng.uniform(-0.2, 0.2))
        mode = rng.choice((0, 0, 1, 2))
        requests.append(walking(mode, s1, b1, s2, b2, g, gb, run, x, pextra=rng.uniform(-2.0, 13.0)))


def append_walking_requests(requests: list[str]) -> None:
    """Append the walking request families."""
    box_requests(requests)
    box_guide_requests(requests)
    tilted_requests(requests)
    cylinder_requests(requests)
    random_requests(requests)
