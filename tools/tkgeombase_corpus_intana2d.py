"""Deterministic requests for the IntAna2d package (tools/oracle/tkgeombase_probe_intana2d.cxx).

Every intersection pair gets random inputs plus the configurations that reach each
branch: parallel, identical, near-parallel and axis-aligned lines (every pivot);
concentric, exterior, externally and internally tangent, secant, nested and nearly
coincident circles, direct and indirect; tangent, secant and missing line-circle
pairs; each gp element against each implicit conic, including identical elements and
tangencies; the polynomial solver on known, multiple, degenerate and null roots.
"""
from __future__ import annotations
import math
import random
from tkgeombase_corpus_gce import add, frame2d, point2d, unit2d

X2 = [0.0, 0.0, 1.0, 0.0, 0.0, 1.0]           # gp_Ax22d at the origin, direct
X2_INDIRECT = [0.0, 0.0, 1.0, 0.0, 0.0, -1.0]  # the same X axis, indirect


def line(p, d):
    return [0, *p, *d]


def circle(frame, r):
    return [1, *frame, r]


def ellipse(frame, major, minor):
    return [2, *frame, major, minor]


def parabola(frame, focal):
    return [3, *frame, focal]


def hyperbola(frame, major, minor):
    return [4, *frame, major, minor]


def at(x, y, sense=1.0, angle=0.0):
    """A gp_Ax22d at (x, y) whose X direction makes `angle` with OX."""
    c, s = math.cos(angle), math.sin(angle)
    return [x, y, c, s, -sense * s, sense * c]


def random_element(rng, kind):
    f = frame2d(rng)
    return [line(point2d(rng), unit2d(rng)), circle(f, rng.uniform(0.5, 4.0)),
            ellipse(f, 3.0, rng.uniform(0.5, 2.5)), parabola(f, rng.uniform(0.25, 2.0)),
            hyperbola(f, rng.uniform(0.5, 3.0), rng.uniform(0.5, 3.0))][kind]


def line_requests(rng, requests):
    for _ in range(12):
        add(requests, 'ia2_lines', *point2d(rng), *unit2d(rng), *point2d(rng), *unit2d(rng))
    s = math.sqrt(0.5)
    dirs = [[1.0, 0.0], [0.0, 1.0], [-1.0, 0.0], [0.0, -1.0], [s, s], [s, -s], [0.6, 0.8], [0.8, 0.6]]
    for d1 in dirs:
        for d2 in dirs:
            add(requests, 'ia2_lines', 1.0, 2.0, *d1, -3.0, 0.5, *d2)
    for eps in [0.0, 1e-17, 1e-16, 1e-15, 1e-9]:
        a = math.atan2(0.8, 0.6) + eps
        add(requests, 'ia2_lines', 1.0, 2.0, 0.6, 0.8, 1.0, 2.0, math.cos(a), math.sin(a))
        add(requests, 'ia2_lines', 1.0, 2.0, 0.6, 0.8, 4.0, -2.0, math.cos(a), math.sin(a))
        add(requests, 'ia2_lines', 1.0, 2.0, 0.6, 0.8, 1.0 + eps, 2.0, 0.6, 0.8)
    add(requests, 'ia2_lines', 1.0, 2.0, 0.6, 0.8, 1.6, 2.8, -0.6, -0.8)
    add(requests, 'ia2_lines', 0.0, 0.0, 1.0, 0.0, 0.0, 1e-300, 1.0, 0.0)


def circle_requests(rng, requests):
    for _ in range(12):
        add(requests, 'ia2_circles', *frame2d(rng), rng.uniform(0.5, 5.0), *frame2d(rng), rng.uniform(0.5, 5.0))
    cases = [
        (at(0, 0), 1.0, at(0, 0), 1.0), (at(0, 0), 1.0, at(0, 0), 2.0), (at(0, 0), 1.0, at(1e-17, 0), 1.0),
        (at(0, 0), 1.0, at(3, 0), 2.0), (at(0, 0), 1.0, at(3.0000000000000004, 0), 2.0),
        (at(0, 0), 1.0, at(5, 0), 1.0), (at(0, 0), 10.0, at(7, 0), 3.0), (at(7, 0), 3.0, at(0, 0), 10.0),
        (at(0, 0), 10.0, at(1, 0), 3.0), (at(0, 0), 2.0, at(1, 1), 2.0), (at(0, 0), 2.0, at(-1, 1, -1.0), 2.0),
        (at(10, 0), 100.0, at(10.00000000000001, 0), 100.0), (at(0, 0, -1.0, 0.3), 1.0, at(1.2, 0.7, 1.0, 2.0), 1.5),
        (at(0, 0), 1.0, at(0, 1.9999999999999), 1.0), (at(0, 0), 3.0, at(0.5, 0.5, 1.0, 1.0), 3.0),
        (at(0, 0), 1.0, at(0.3, 0.0), 0.7), (at(0, 0), 1.0, at(1.0, 1.0), 0.0),
    ]
    for a1, r1, a2, r2 in cases:
        add(requests, 'ia2_circles', *a1, r1, *a2, r2)


def line_circle_requests(rng, requests):
    for _ in range(12):
        add(requests, 'ia2_line_circle', *point2d(rng, 3.0), *unit2d(rng), *frame2d(rng), rng.uniform(0.5, 4.0))
    for y in [1.0, 1.0 + 1e-16, 1.0 - 1e-16, 0.999999999, 0.5, 0.0, -1.0, 1.5, 1e-300]:
        for frame in [X2, X2_INDIRECT, at(0, 0, 1.0, 1.0)]:
            add(requests, 'ia2_line_circle', -2.0, y, 1.0, 0.0, *frame, 1.0)
            add(requests, 'ia2_line_circle', y, -2.0, 0.0, 1.0, *frame, 1.0)
    add(requests, 'ia2_line_circle', 0.0, 0.0, 1.0, 0.0, *X2, 0.0)
    add(requests, 'ia2_line_circle', 0.0, 1e-300, 1.0, 0.0, *X2, 0.0)


def conic_requests(rng, requests):
    for _ in range(3):
        for first in range(5):
            for second in range(5):
                add(requests, 'ia2_conic', *random_element(rng, first), *random_element(rng, second))
    designed = [
        (line([-2, 1], [1, 0]), circle(X2, 1.0)), (line([-2, 0], [1, 0]), line([0, 0], [1, 0])),
        (line([-2, 0], [1, 0]), line([0, 1], [1, 0])), (line([-2, 0], [1, 0]), line([0, 0], [0, 1])),
        (line([0, -2], [0, 1]), parabola(X2, 1.0)), (line([0, 0], [0, 1]), parabola(X2, 1.0)),
        (line([-5, 1], [1, 0]), parabola(X2, 0.25)), (line([0, 0], [1, 0]), hyperbola(X2, 2.0, 1.0)),
        (line([2, 0], [0, 1]), hyperbola(X2, 2.0, 1.0)), (line([0, 3], [1, 0]), ellipse(X2, 3.0, 2.0)),
        (circle(X2, 1.0), circle(X2, 1.0)), (circle(X2, 1.0), circle(at(1, 0), 1.0)),
        (circle(X2_INDIRECT, 1.0), circle(at(1, 0), 1.0)), (circle(X2, 1.0), circle(at(2, 0), 1.0)),
        (circle(X2, 2.0), ellipse(X2, 3.0, 1.0)), (circle(X2, 1.0), line([0, 1], [1, 0])),
        (circle(X2, 1.0), line([0, 5], [1, 0])), (circle(X2, 1.0), hyperbola(X2, 0.5, 1.0)),
        (ellipse(X2, 3.0, 2.0), ellipse(X2, 3.0, 2.0)), (ellipse(X2, 3.0, 1.0), ellipse(at(0, 0, 1.0, math.pi / 2), 3.0, 1.0)),
        (ellipse(X2_INDIRECT, 3.0, 1.0), circle(X2, 2.0)), (ellipse(X2, 3.0, 1.0), line([3, 0], [0, 1])),
        (parabola(X2, 1.0), parabola(X2, 1.0)), (parabola(X2, 1.0), line([0, 0], [0, 1])),
        (parabola(X2_INDIRECT, 1.0), circle(at(1, 0), 1.0)), (parabola(X2, 1.0), parabola(at(0, 0, 1.0, math.pi), 1.0)),
        (parabola(X2, 1.0), line([-1, 0], [0, 1])), (parabola(X2, 0.5), hyperbola(X2, 1.0, 1.0)),
        (hyperbola(X2, 2.0, 1.0), hyperbola(X2, 2.0, 1.0)), (hyperbola(X2, 2.0, 1.0), circle(X2, 2.0)),
        (hyperbola(X2_INDIRECT, 2.0, 1.0), circle(X2, 3.0)), (hyperbola(X2, 2.0, 1.0), line([-3, 0], [0, 1])),
        (hyperbola(X2, 2.0, 1.0), line([0, 0], [1, 0])), (hyperbola(X2, 1.0, 1.0), ellipse(X2, 3.0, 2.0)),
    ]
    for first, second in designed:
        add(requests, 'ia2_conic', *first, *second)


def eval_requests(rng, requests):
    for kind in range(5):
        for _ in range(3):
            add(requests, 'ia2_conic_eval', *random_element(rng, kind), *point2d(rng), *point2d(rng), *unit2d(rng))


def root_requests(rng, requests):
    quartics = [
        [1, -10, 35, -50, 24], [1, 0, -5, 0, 4], [1, -6, 13, -12, 4], [1, 0, 0, 0, 0], [1, 0, 0, 0, -1],
        [0, 1, -6, 11, -6], [0, 0, 1, -3, 2], [0, 0, 0, 1, -2], [0, 0, 0, 0, 1], [0, 0, 0, 0, 0],
        [1e-13, 0, 0, 0, 1e-13], [1, 0, 1, 0, 1], [1, -4, 6, -4, 1], [1e-8, 1, -3, 3, -1], [1, 1e-9, -2, 0, 1],
        [3.5, -2.25, 7.0, 1.5, -0.125], [1e6, 0, -1e-6, 0, 1e-18], [1, -2, 1, -1e-12, 0],
    ]
    for q in quartics:
        add(requests, 'ia2_roots', 0, *map(float, q))
    for _ in range(12):
        add(requests, 'ia2_roots', 0, *[rng.uniform(-3, 3) for _ in range(5)])
    for q in [[1, -3, 2], [1, -2, 1], [1, 0, 1], [0, 1, -1], [0, 0, 1], [0, 0, 0], [1e-13, 0, 1e-13], [2, 1e-9, -1e-9]]:
        add(requests, 'ia2_roots', 1, *map(float, q))
    for _ in range(6):
        add(requests, 'ia2_roots', 1, *[rng.uniform(-3, 3) for _ in range(3)])


def outils_requests(rng, requests):
    tiny = 1e-16
    for x, y in [(1.0, 2.0), (0.0, 0.0), (1e300, -1e-300), (3.0, 3.0 + 4 * tiny)]:
        pts = [[x, y, 0.1, 0.2], [x + tiny, y, 0.3, 0.4], [5.0, 6.0, 0.5, 0.6], [x, y, 0.7, 0.8]]
        for n in range(5):
            add(requests, 'ia2_outils', x, y, x + tiny, y - tiny, *point2d(rng), *point2d(rng), *unit2d(rng),
                n, *[v for p in pts[:n] for v in p])
    add(requests, 'ia2_outils', 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0,
        4, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 1.0, 1.0, 3.0, 3.0)
    for _ in range(4):
        add(requests, 'ia2_int_point', *point2d(rng), *point2d(rng))


def append_intana2d_requests(requests: list[str]) -> None:
    """Append every IntAna2d request family in a fixed order."""
    rng = random.Random(0x1A2D)
    line_requests(rng, requests)
    circle_requests(rng, requests)
    line_circle_requests(rng, requests)
    conic_requests(rng, requests)
    eval_requests(rng, requests)
    root_requests(rng, requests)
    outils_requests(rng, requests)
