"""Deterministic requests for the line builder probes
(tools/oracle/tkfillet_probe_linebuilders.cxx).

`fl_surf_rst_walk` marches the constant fillet between a surface and a restriction (an
edge of another face) with the inverses of ChFi3d_FilBuilder: a floor and the top edge of
a step wall along the X axis (the faces and the edge cut at several lengths, so that the
marching stops on the floor domain, on the end of the edge, or past the bound), and a
cylinder on the circular edge of a floor ring. The modes complete the line, search the
arc to reframe on, and start on the restrictions.
"""
from __future__ import annotations
import math
import random

from tkfillet_corpus_blendfunc import X, Y, Z, guide, numbers, surface

FLAGS = (1, 1, 1)


def surf_rst_walk(mode, s1, b1, s2, b2, rst, sref, pcref, g, gbounds, run, x, rec=FLAGS, pextra=0.0) -> str:
    """One `fl_surf_rst_walk` request; `rst` and `pcref` are (kind, a, b, c, d, first,
    last), `run` (radius, choix, choixinv, pdep, pmax, maxstep, tol3d, tol2d, tolguide,
    fleche, appro)."""
    fields = [mode, *s1, *b1, *s2, *b2, *rst, *sref, *pcref, *g, *gbounds, 0, 0, 0, *run, *x, *rec, pextra]
    return 'fl_surf_rst_walk ' + numbers(fields)


def step(l1=10.0, l2=10.0, h=0.5):
    """The floor z = 0 (u in [0, l1], v in [0, 5]), the wall y = 0 of height h (u in
    [0, l2]) whose top edge v = -h is the restriction, the top face z = h behind it."""
    floor, wall = surface(0, n=Z, x=X), surface(0, n=Y, x=X)
    rst = (0, 0.0, -h, 1.0, 0.0, 0.0, l2)
    top = surface(0, p=(0.0, 0.0, h), n=Z, x=X)
    pcref = (0, 0.0, 0.0, 1.0, 0.0, 0.0, l2)
    return floor, (0.0, l1, 0.0, 5.0), wall, (0.0, l2, -h, 0.0), rst, top, pcref, guide(0, n=X, x=Z), (-1.0, 11.0)


def contact(radius: float, h: float) -> float:
    """The floor contact distance from the wall of the ball on the edge of height h."""
    return math.sqrt(max(radius * radius - (radius - h) ** 2, 0.0))


def step_requests(requests: list[str]) -> None:
    """The step: every choice, the stops on the floor, the edge end and the bound."""
    for choix in range(1, 9):
        s = step()
        run = (1.0, choix, choix, 1.0, 12.0, 2.0, 1e-6, 1e-6, 1e-5, 0.01, 1)
        requests.append(surf_rst_walk(0, *s, run, (1.0, contact(1.0, 0.5), 1.0)))
    for l1, l2, h, radius in ((10.0, 8.0, 0.5, 1.0), (6.0, 10.0, 0.5, 1.0), (7.0, 7.0, 0.3, 0.8), (10.0, 10.0, 0.9, 1.0)):
        s = step(l1, l2, h)
        for pdep, pmax, maxstep in ((3.0, 12.0, 2.0), (3.0, 12.0, 0.4), (5.0, -1.0, 1.5)):
            run = (radius, 2, 2, pdep, pmax, maxstep, 1e-6, 1e-6, 1e-5, 0.01, 1)
            requests.append(surf_rst_walk(0, *s, run, (pdep, contact(radius, h), pdep)))
    s = step(10.0, 8.0)
    run = (1.0, 2, 2, 3.0, 12.0, 2.0, 1e-6, 1e-6, 1e-5, 0.01, 1)
    x = (3.0, contact(1.0, 0.5), 3.0)
    for mode, pextra in ((1, -2.0), (1, 1.0), (2, 0.0), (2, 0.4), (2, -7.0)):
        requests.append(surf_rst_walk(mode, *s, run, x, pextra=pextra))
    exact = (1.0, 2, 2, 2.0, 12.0, 2.0, 1e-6, 1e-6, 1e-5, 0.01, 0)
    requests.append(surf_rst_walk(0, *s, exact, (2.0, contact(1.0, 0.5), 2.0)))
    for pdep, pmax, rec, pextra in ((7.9, 12.0, (1, 1, 1), -1.0), (7.9, 12.0, (0, 1, 0), -1.0), (7.9, 12.0, (1, 0, 0), -1.0),
                                    (4.0, 12.0, (1, 1, 1), -1.0), (0.2, -1.0, (1, 1, 1), 9.0)):
        run3 = (1.0, 2, 2, pdep, pmax, 2.0, 1e-6, 1e-6, 1e-5, 0.01, 1)
        requests.append(surf_rst_walk(3, *s, run3, (pdep, contact(1.0, 0.5), pdep), rec=rec, pextra=pextra))
    s6 = step(6.0, 10.0)
    run3 = (1.0, 2, 2, 5.9, 12.0, 2.0, 1e-6, 1e-6, 1e-5, 0.01, 1)
    requests.append(surf_rst_walk(3, *s6, run3, (5.9, contact(1.0, 0.5), 5.9), pextra=-1.0))


def ring_requests(requests: list[str]) -> None:
    """The cylinder of radius 5 (u in [0, u1], v in [0, 4]) on the edge circle of radius
    6 of the floor z = 0, a lower ring z = -0.5 behind it, along the circle of radius 5."""
    cylinder = surface(1, n=Z, x=X, a=5.0)
    floor = surface(0, n=Z, x=X)
    lower = surface(0, p=(0.0, 0.0, -0.5), n=Z, x=X)
    circle = guide(1, n=Z, x=X, a=5.0)
    for u1, last, radius, fleche in ((2.0, 2.0 * math.pi, 1.0, 0.01), (4.0, 3.0, 1.0, 0.001), (2.0, 2.0 * math.pi, 1.5, 0.1)):
        rst = (1, 0.0, 0.0, 6.0, 0.0, 0.0, last)
        z = math.sqrt(2.0 * radius - 1.0)
        for choix in (1, 2, 3, 4):
            run = (radius, choix, choix, 0.3, 3.5, 0.5, 1e-6, 1e-6, 1e-5, fleche, 1)
            requests.append(surf_rst_walk(0, cylinder, (0.0, u1, 0.0, 4.0), floor, (-10.0, 10.0, -10.0, 10.0), rst,
                                          lower, rst, circle, (0.0, 2.0 * math.pi), run, (0.3, z, 0.3)))
    rst = (1, 0.0, 0.0, 6.0, 0.0, 0.0, 2.0 * math.pi)
    run = (1.0, 2, 2, 1.0, 1.5, 0.5, 1e-6, 1e-6, 1e-5, 0.01, 1)
    for mode, pextra in ((1, -0.5), (2, 0.1), (3, 2.5)):
        requests.append(surf_rst_walk(mode, cylinder, (0.0, 2.0, 0.0, 4.0), floor, (-10.0, 10.0, -10.0, 10.0), rst,
                                      lower, rst, circle, (0.0, 2.0 * math.pi), run, (1.0, 1.0, 1.0), pextra=pextra))


def random_requests(requests: list[str]) -> None:
    """Seeded steps: lengths, heights, radii, starts, steps, deflections, perturbed starts."""
    rng = random.Random(20261009)
    for _ in range(20):
        l1, l2, h = rng.uniform(4.0, 10.0), rng.uniform(4.0, 10.0), rng.uniform(0.2, 0.9)
        radius = rng.uniform(max(h, 0.4), 2.0)
        s = step(l1, l2, h)
        pdep = rng.uniform(0.5, min(l1, l2) - 0.5)
        pmax = rng.choice((12.0, -1.0, rng.uniform(-1.0, 12.0)))
        run = (radius, rng.choice((2, 2, 1, 4)), 2, pdep, pmax, rng.uniform(0.1, 3.0), 1e-6, 1e-6, 1e-5,
               rng.choice((0.001, 0.01, 0.1)), 1)
        x = (pdep + rng.uniform(-0.1, 0.1), contact(radius, h) + rng.uniform(-0.1, 0.1), pdep + rng.uniform(-0.1, 0.1))
        requests.append(surf_rst_walk(rng.choice((0, 0, 1, 2)), *s, run, x, pextra=rng.uniform(-2.0, 13.0)))


def append_linebuilder_requests(requests: list[str]) -> None:
    """Append the line builder request families."""
    step_requests(requests)
    ring_requests(requests)
    random_requests(requests)
