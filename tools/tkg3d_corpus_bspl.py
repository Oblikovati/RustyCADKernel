"""TKG3d corpus: Bezier and B-spline curves, their edition and their evaluation
representations (tools/oracle/tkg3d_probe_bspl.cxx)."""
from __future__ import annotations
import math
import random
from tkg3d_corpus import bezier, bspline, circle, fmt, frame, line, moves, params, point, trsf, unit


def poles(rng: random.Random, count, rational):
    """Random poles [x, y, z, w]; weights stay 1 unless rational."""
    return [[*point(rng, 5), rng.uniform(0.5, 3.0) if rational else 1.0] for _ in range(count)]


def clamped_knots(rng: random.Random, degree, interior, mult=1):
    knots = [0.0] + sorted(rng.uniform(0.05, 0.95) for _ in range(interior)) + [1.0]
    return [[k, degree + 1 if i in (0, len(knots) - 1) else mult] for i, k in enumerate(knots)]


def bspline_random(rng: random.Random, degree, interior, rational, mult=1):
    knots = clamped_knots(rng, degree, interior, mult)
    count = sum(m for _, m in knots) - degree - 1
    return bspline(degree, knots, poles(rng, count, rational), rational=rational)


def bspline_periodic(rng: random.Random, degree, count, rational):
    knots = [[float(i), 1] for i in range(count + 1)]
    return bspline(degree, knots, poles(rng, count, rational), periodic=True, rational=rational)


def spline_specs(rng: random.Random) -> list[list]:
    """Valid, invalid, rational, periodic and degenerate Bezier and B-spline curves."""
    specs = []
    for degree in range(1, 7):
        specs.append(bezier(poles(rng, degree + 1, False)))
        specs.append(bezier(poles(rng, degree + 1, True), True))
    specs += [bezier([[0, 0, 0, 1], [1, 2, 3, 1], [0, 0, 0, 1]]), bezier([[0, 0, 0, 2], [1, 2, 1, 2], [3, 0, 1, 2]], True),
              bezier([[0, 0, 0, 1], [1, 2, 0, 0.0], [3, 0, 0, 1]], True), bezier([[0, 0, 0, 1]]),
              bezier([[float(i), 0.0, 0.0, 1.0] for i in range(27)]),
              bezier([[0, 0, 0, 1], [0, 0, 0, 1], [1, 1, 1, 1], [2, 0, 0, 1]]),
              bezier([[0, 0, 0, 1], [1, 1, 0, 1], [1e-8, 0, 0, 1]])]
    for degree in range(1, 6):
        specs.append(bspline_random(rng, degree, rng.randrange(0, 4), False))
        specs.append(bspline_random(rng, degree, rng.randrange(1, 4), True))
        specs.append(bspline_periodic(rng, degree, degree + 3, rng.random() < 0.5))
    specs += [bspline_random(rng, 3, 2, False, 3), bspline_random(rng, 2, 2, True, 2),
              bspline(1, [[float(i), 1] for i in range(6)], poles(rng, 4, False)),
              bspline(2, [[0.0, 3], [0.5, 1], [0.5000000000000001, 1], [1.0, 3]], poles(rng, 5, False)),
              bspline(2, [[0.0, 3], [1.0, 3]], poles(rng, 4, False)), bspline(0, [[0.0, 1], [1.0, 1]], poles(rng, 2, False)),
              bspline(2, [[0.0, 3], [1.0, 3]], [[0, 0, 0, 1], [1, 1, 1, 0.0], [2, 0, 0, 1]], rational=True),
              bspline(2, [[0.0, 3], [1.0, 3]], [[0, 0, 0, 2], [1, 1, 1, 2], [2, 0, 0, 2]], rational=True),
              bspline(3, [[0.0, 4], [0.5, 3], [1.0, 4]], poles(rng, 7, False))]
    return specs


def spline_params(rng: random.Random):
    return params([0.0, 1.0, 0.5, rng.uniform(-1, 2), rng.uniform(0, 1), 2 * math.pi, -0.25, 7.0])


BEZIER_ACTIONS = [(1, 5), (1, 2), (1, 30), (2, 1), (2, 0), (2, 9), (3, 1), (4, 2), (4, 0), (5, 0), (6, 0), (7, 1), (7, 9),
                  (8, 2), (9, 2), (9, 1), (10, 0), (11, 1), (11, 7), (12, 2), (12, 0)]


def bezier_action(rng: random.Random, action, index):
    weight = rng.choice([1.0, 2.5, 0.0, 1.0 + 1e-17])
    a, z = sorted([rng.uniform(-0.2, 1.2), rng.uniform(-0.2, 1.2)])
    return [action, index, *point(rng), weight, a, z, *trsf(rng)]


def append_bezier_requests(rng: random.Random, requests: list[str]) -> None:
    for index in range(56):
        rational = index % 2 == 1
        spec = bezier(poles(rng, 2 + index % 6, rational), rational)
        chosen = [BEZIER_ACTIONS[(index + k) % len(BEZIER_ACTIONS)] for k in range(3)]
        actions = [c for action, pole in chosen for c in bezier_action(rng, action, pole)]
        requests.append('g3_bezier ' + fmt(spec + params([0.0, 0.3, 1.0, 1.4]) + [len(chosen)] + actions))
    closed = bezier([[0, 0, 0, 1], [2, 2, 1, 3], [-1, 1, 0, 1], [0, 0, 0, 1]], True)
    requests.append('g3_bezier ' + fmt(closed + params([0.5]) + [2] + bezier_action(rng, 7, 4) + bezier_action(rng, 9, 1)))
    near = bezier([[0, 0, 0, 1], [1, 1, 0, 1], [5e-8, 0, 0, 1]])
    requests.append('g3_bezier ' + fmt(near + params([0.5]) + [1] + bezier_action(rng, 6, 0)))


BSPLINE_ACTIONS = [1, 2, 3, 4, 5, 6, 7, 8, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24]


def bspline_action(rng: random.Random, action, curve_knots):
    i, j, k = rng.randrange(1, curve_knots + 1), rng.randrange(1, curve_knots + 1), rng.randrange(0, 4)
    if action in (3, 4) and i > j:
        i, j = j, i
    if action == 23:
        i, j = rng.choice([-1, 0, 1, 2]), rng.choice([-1, 0, 1, 2])
    a, z = sorted([rng.uniform(-0.1, 1.1), rng.uniform(-0.1, 1.1)])
    weight = rng.choice([0.0, 1e-9, 1e-3, 1.0, 2.0])
    if action in (5, 6, 12):
        # A parametric tolerance wider than a knot span leaves knot sequences on which
        # the native BSplCLib reads outside its buffers (undefined behaviour).
        weight = rng.choice([0.0, 1e-9, 1e-3])
    if action == 8:
        a, weight = rng.uniform(-3.0, 9.0), rng.choice([1e-9, 1e-3, 0.3])
    return [action, i, j, k, *point(rng), weight, a, z, rng.randrange(2), *trsf(rng)]


def append_bspline_requests(rng: random.Random, requests: list[str]) -> None:
    for index in range(110):
        degree = 1 + index % 4
        if index % 3 == 2:
            spec, nk, u = bspline_periodic(rng, degree, degree + 3, index % 2 == 0), degree + 4, [0.0, 1.5, 4.0, -2.0, 11.0]
        else:
            spec = bspline_random(rng, degree, 1 + index % 3, index % 2 == 0, 1 + (index % 7 == 0) * (degree - 1))
            nk, u = 3 + index % 3, [0.0, 0.37, 1.0, 1.2]
        actions = []
        for step in range(3):
            actions += bspline_action(rng, BSPLINE_ACTIONS[(index * 3 + step) % len(BSPLINE_ACTIONS)], nk)
        requests.append('g3_bspline ' + fmt(spec + params(u) + [3] + actions))
    uniform = bspline(2, [[float(i), 1] for i in range(8)], poles(rng, 5, False))
    for action in (8, 16, 17, 18, 22, 23, 7):
        requests.append('g3_bspline ' + fmt(uniform + params([2.5, 3.0]) + [1] + bspline_action(rng, action, 8)))
    periodic = bspline_periodic(rng, 3, 6, True)
    for origin in (0.0, 2.0, 2.0000001, 2.5, -4.5, 13.25):
        edit = [8, 1, 1, 0, 0.0, 0.0, 0.0, 1e-6, origin, 0.0, 0, *trsf(rng)]
        requests.append('g3_bspline ' + fmt(periodic + params([0.5, 5.5]) + [1] + edit))


def descriptor(rng: random.Random, kind, order):
    return [kind, order, rng.randrange(2), 0.2, 0.9, rng.choice([1.0, 2.0, -0.5, 0.0]), rng.uniform(-1, 1)]


def representation(rng: random.Random):
    return rng.choice([bezier(poles(rng, 4, True), True), circle(frame(rng), 2.0), line(point(rng), unit(rng))])


def append_evalrep_requests(rng: random.Random, requests: list[str]) -> None:
    owners = [bezier(poles(rng, 4, False)), bspline_random(rng, 3, 2, True), bspline_periodic(rng, 2, 5, False)]
    for owner in owners:
        for kind in range(3):
            for order in (0, 2, 3):
                requests.append('g3_evalrep ' + fmt(owner + params([0.0, 0.5, 1.0, 3.0]) + representation(rng) +
                                                    descriptor(rng, kind, order) + trsf(rng)))
    for index in range(len(BSPLINE_ACTIONS)):
        owner = bspline_periodic(rng, 3, 6, False) if index % 2 else bspline_random(rng, 3, 2, True)
        action = BSPLINE_ACTIONS[index]
        edits = bspline_action(rng, action, 4) + bspline_action(rng, BSPLINE_ACTIONS[-1 - index], 4)
        requests.append('g3_evalrep_edit ' + fmt(owner + params([0.25, 0.75]) + representation(rng) +
                                                 descriptor(rng, index % 3, 3) + [2] + edits))


def append_spline_curve_requests(rng: random.Random, requests: list[str]) -> None:
    """The generic curve summaries and moves of every spline spec."""
    for spec in spline_specs(rng):
        requests.append('g3_curve ' + fmt(spec + spline_params(rng)))
        requests.append('g3_transform ' + fmt(spec + spline_params(rng) + trsf(rng) + moves(rng)))


def append_bspl_requests(requests: list[str]) -> None:
    """Append the TKG3d spline curve families with their own generator."""
    rng = random.Random(0x3B5C)
    append_spline_curve_requests(rng, requests)
    append_bezier_requests(rng, requests)
    append_bspline_requests(rng, requests)
    append_evalrep_requests(rng, requests)
