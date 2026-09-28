"""Deterministic request corpus for the TKG2d (Geom2d) numeric oracle.

Each request is one `operation numbers...` line read by tools/oracle/tkg2d_probe.cxx
and crates/tkg2d/examples/numeric_probe.rs. Curve specifications follow the grammar
documented in tools/oracle/tkg2d_probe_common.cxx (g2_curve).
"""
from __future__ import annotations
import math
import random


def fmt(values) -> str:
    """Space-separated numbers with repr precision; fmt([1, 0.5])."""
    return ' '.join(repr(float(v)) if isinstance(v, float) else str(v) for v in values)


def line(p, d):
    return [0, *p, *d]


def conic(kind, p, xd, sense, *radii):
    return [kind, *p, *xd, int(sense), *radii]


def bezier(poles, rational=False):
    return [5, int(rational), len(poles), *[c for pole in poles for c in pole]]


def bspline(degree, knots, poles, periodic=False, rational=False):
    flat_knots = [c for knot in knots for c in knot]
    flat_poles = [c for pole in poles for c in pole]
    return [6, int(rational), int(periodic), degree, len(knots), *flat_knots, len(poles), *flat_poles]


def trimmed(u1, u2, basis, sense=True, adjust=True):
    return [7, u1, u2, int(sense), int(adjust), *basis]


def offset(distance, basis, no_check=False):
    return [8, distance, int(no_check), *basis]


def params(values):
    return [len(values), *values]


def unit(rng: random.Random):
    angle = rng.uniform(-math.pi, math.pi)
    return [math.cos(angle), math.sin(angle)]


def point(rng: random.Random, scale=10.0):
    return [rng.uniform(-scale, scale), rng.uniform(-scale, scale)]


def trsf(rng: random.Random, kind=None):
    kind = rng.randrange(7) if kind is None else kind
    values = [rng.uniform(-3, 3) for _ in range(5)]
    if kind == 1:
        values[2:4] = unit(rng)
    if kind == 3:
        values[2] = rng.choice([2.0, -1.5, 0.5, -1.0])
    if kind == 6:
        values[4] = rng.choice([2.0, -0.5, 3.0])
    return [kind, *values]


def moves(rng: random.Random):
    """The inputs of g2_moves: P, axis, angle, factor, V, Q, transformation."""
    return [*point(rng), *point(rng), *unit(rng), rng.uniform(-4, 4), rng.choice([2.0, -0.5, 1.0, 0.0]),
            *point(rng), *point(rng), *trsf(rng)]


def poles(rng: random.Random, count, rational):
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


def curve_specs(rng: random.Random):
    """Valid and invalid curves of every kind."""
    specs = []
    for _ in range(6):
        specs.append(line(point(rng), unit(rng)))
        specs.append(conic(1, point(rng), unit(rng), rng.random() < 0.5, rng.uniform(0.1, 5)))
        a = rng.uniform(1, 5)
        specs.append(conic(2, point(rng), unit(rng), rng.random() < 0.5, a, rng.uniform(0.1, a)))
        specs.append(conic(3, point(rng), unit(rng), rng.random() < 0.5, rng.uniform(0.1, 5), rng.uniform(0.1, 5)))
        specs.append(conic(4, point(rng), unit(rng), rng.random() < 0.5, rng.uniform(0.1, 5)))
    specs += [conic(1, [0, 0], [1, 0], True, 0.0), conic(1, [0, 0], [1, 0], True, -1.0),
              conic(2, [0, 0], [1, 0], True, 1.0, 2.0), conic(2, [0, 0], [1, 0], True, 2.0, 0.0),
              conic(2, [0, 0], [1, 0], False, 0.0, 0.0), conic(3, [0, 0], [1, 0], True, 0.0, 1.0),
              conic(3, [1, 1], [0, 1], True, -1.0, 1.0), conic(4, [0, 0], [1, 0], False, 0.0),
              conic(4, [0, 0], [1, 0], True, -2.0), [9, 1, 2, 1, 0, 0, 1, 3.0], [9, 1, 2, 1, 0, 0, -1, 2.0],
              [10, 0, 0, 1, 0, 2, 3], [10, 0, 0, 0, 1, 0, 0], [10, 1, 1, 1, 1, -2, 5]]
    for degree in range(1, 7):
        specs.append(bezier(poles(rng, degree + 1, False)))
        specs.append(bezier(poles(rng, degree + 1, True), True))
    specs += [bezier([[0, 0, 1], [1, 2, 1], [0, 0, 1]]), bezier([[0, 0, 2], [1, 2, 2], [3, 0, 2]], True),
              bezier([[0, 0, 1], [1, 2, -1], [3, 0, 1]], True), bezier([[0, 0, 1]]),
              bezier([[float(i), 0.0, 1.0] for i in range(27)]), bezier([[0, 0, 1], [0, 0, 1], [1, 1, 1], [2, 0, 1]])]
    for degree in range(1, 6):
        specs.append(bspline_random(rng, degree, rng.randrange(0, 4), False))
        specs.append(bspline_random(rng, degree, rng.randrange(1, 4), True))
        specs.append(bspline_periodic(rng, degree, degree + 3, rng.random() < 0.5))
    specs += [bspline_random(rng, 3, 2, False, 3), bspline_random(rng, 2, 2, True, 2),
              bspline(1, [[float(i), 1] for i in range(6)], poles(rng, 4, False)),
              bspline(2, [[0.0, 3], [0.5, 1], [0.5000000000000001, 1], [1.0, 3]], poles(rng, 5, False)),
              bspline(2, [[0.0, 3], [1.0, 3]], poles(rng, 4, False)), bspline(0, [[0.0, 1], [1.0, 1]], poles(rng, 2, False)),
              bspline(2, [[0.0, 3], [1.0, 3]], [[0, 0, 1], [1, 1, 0.0], [2, 0, 1]], rational=True)]
    circle = conic(1, [0, 0], [1, 0], True, 2.0)
    specs += [trimmed(0.5, 2.5, circle), trimmed(2.5, 0.5, circle), trimmed(-1.0, 7.0, circle, False, True),
              trimmed(5.0, 8.0, circle, True, False), trimmed(0.0, 2 * math.pi, circle),
              trimmed(0.0, 3.0, line([0, 0], [1, 0])), trimmed(3.0, -2.0, line([1, 1], [0, 1]), False),
              trimmed(0.2, 0.8, bezier(poles(rng, 4, True), True)), trimmed(0.8, 0.2, bezier(poles(rng, 3, False)), True),
              trimmed(0.5, 3.5, bspline_periodic(rng, 3, 6, False)), trimmed(1.0, 1.0, circle), trimmed(-0.5, 0.5, bezier(poles(rng, 3, False))),
              trimmed(0.1, 0.9, trimmed(0.0, 1.0, bezier(poles(rng, 4, False))))]
    cusp = bezier([[0, 0, 1], [0, 0, 1], [1, 1, 1], [2, 0, 1]])
    tiny = bezier([[0, 0, 1], [1e-110, 0, 1], [1, 1, 1], [2, 0, 1]])
    specs += [offset(0.5, circle), offset(-2.0, circle), offset(1.5, line([0, 0], [1, 1])),
              offset(0.3, conic(2, [0, 0], [1, 0], True, 3.0, 1.0)), offset(0.25, cusp), offset(-0.25, cusp),
              offset(0.1, tiny), offset(0.2, offset(0.3, circle)), offset(0.2, trimmed(0.5, 2.0, circle)),
              offset(0.1, bspline_random(rng, 3, 2, False, 3)), offset(0.1, bspline_random(rng, 3, 2, False, 3), True),
              offset(0.1, bspline(2, [[0.0, 3], [0.5, 2], [1.0, 3]], [[0, 0, 1], [1, 0, 1], [2, 0, 1], [3, 0, 1], [4, 0, 1]])),
              offset(0.4, bspline_periodic(rng, 3, 7, True)), offset(0.2, conic(3, [0, 0], [1, 0], True, 1.0, 0.5)),
              offset(0.5, conic(4, [0, 0], [1, 0], True, 1.0))]
    return specs


def curve_params(rng: random.Random):
    return params([0.0, 1.0, 0.5, rng.uniform(-1, 2), rng.uniform(0, 1), 2 * math.pi, -0.25, 7.0])


def append_curve_requests(rng: random.Random, requests: list[str], specs) -> None:
    for spec in specs:
        requests.append('g2_curve ' + fmt(spec + curve_params(rng)))
        requests.append('g2_transform ' + fmt(spec + curve_params(rng) + trsf(rng) + moves(rng)))


def append_conic_requests(rng: random.Random, requests: list[str]) -> None:
    for kind in [1, 2, 3, 4]:
        for index in range(8):
            radii = [rng.uniform(1, 4)]
            if kind in (2, 3):
                radii.append(rng.uniform(0.1, radii[0]))
            spec = conic(kind, point(rng), unit(rng), index % 2 == 0, *radii)
            value = [rng.uniform(0.1, 5), 0.0, -1.0, radii[0]][index % 4]
            requests.append('g2_conic ' + fmt(spec + curve_params(rng) + [value, *point(rng), *unit(rng), *point(rng)]))


def append_simple_requests(rng: random.Random, requests: list[str]) -> None:
    for index in range(40):
        requests.append('g2_point ' + fmt([*point(rng), *point(rng), *point(rng), *moves(rng)]))
        direction = [0.0, 0.0] if index == 0 else point(rng)
        setters = [0.0, 0.0] if index == 1 else point(rng)
        requests.append('g2_direction ' + fmt([*direction, *setters, *point(rng), *moves(rng)]))
        first = [0.0, 0.0] if index % 10 == 2 else point(rng)
        scalar = [0.0, rng.uniform(-3, 3), 1e-300, -2.0][index % 4]
        requests.append('g2_vector ' + fmt([*first, *point(rng), scalar, *point(rng), *point(rng), *moves(rng)]))
        requests.append('g2_axis ' + fmt([*point(rng), *unit(rng), *point(rng), *unit(rng), *point(rng), *unit(rng), *moves(rng)]))
        scale = [rng.uniform(-3, 3), 0.0, -1.0, 1e-20][index % 4]
        requests.append('g2_trsf ' + fmt([index % 9, *point(rng), *point(rng), *point(rng), *unit(rng), *point(rng), *unit(rng),
                                          scale, rng.randrange(-4, 5)]))
    requests.append('g2_vector ' + fmt([0.0, 0.0, 0.0, 0.0, 2.0, 1, 1, 1, 1, *moves(rng)]))


def append_trimmed_requests(rng: random.Random, requests: list[str]) -> None:
    circle = conic(1, [1, -1], [0, 1], True, 1.5)
    bases = [circle, line([0, 0], [1, 0]), bezier(poles(rng, 4, True), True), bspline_periodic(rng, 2, 5, False),
             bspline_random(rng, 3, 2, False), conic(3, [0, 0], [1, 0], True, 2.0, 1.0)]
    windows = [(0.2, 0.9, 1, 1), (0.9, 0.2, 1, 1), (0.2, 0.9, 0, 1), (-3.0, 4.0, 1, 0), (0.3, 0.3, 1, 1), (-0.5, 0.5, 1, 1),
               (0.0, 1.0, 0, 0), (6.0, 7.5, 1, 1)]
    for basis in bases:
        for window in windows:
            spec = trimmed(0.1, 0.8, basis)
            requests.append('g2_trimmed ' + fmt(spec + curve_params(rng) + list(window) + trsf(rng)))


def append_offset_requests(rng: random.Random, requests: list[str]) -> None:
    circle = conic(1, [0, 0], [1, 0], True, 2.0)
    bases = [circle, conic(2, [0, 0], [1, 0], True, 3.0, 1.0), bezier([[0, 0, 1], [0, 0, 1], [1, 1, 1], [2, 0, 1]]),
             bezier(poles(rng, 5, True), True), bspline_random(rng, 3, 3, True), trimmed(0.2, 1.4, circle),
             line([1, 2], [0.6, 0.8]), bspline(2, [[0.0, 3], [0.5, 2], [1.0, 3]], [[0, 0, 1], [1, 0, 1], [2, 0, 1], [3, 0, 1], [4, 0, 1]])]
    for basis in bases:
        for distance in [0.5, -0.75, 2.0]:
            replacement = rng.choice([circle, trimmed(0.1, 0.9, bezier(poles(rng, 4, False))), offset(0.2, circle)])
            requests.append('g2_offset ' + fmt(offset(0.25, basis) + curve_params(rng) + [distance, rng.randrange(2)] +
                                               trsf(rng) + replacement))
    for index in range(64):
        vectors = [[rng.uniform(-3, 3), rng.uniform(-3, 3)] for _ in range(4)]
        if index % 4 == 1:
            vectors[0] = [1e-110, -2e-110]
        if index % 4 == 2:
            vectors[0] = [1e-160, 0.0]
        if index % 4 == 3:
            vectors[0] = [3e-60, 1e-60]
        requests.append('g2_offset_utils ' + fmt([*point(rng), *[c for v in vectors for c in v], rng.uniform(-2, 2), index % 2]))


BEZIER_ACTIONS = [(1, 5), (1, 2), (1, 30), (2, 1), (2, 0), (2, 9), (3, 1), (4, 2), (4, 0), (5, 0), (6, 0), (7, 1), (7, 9),
                  (8, 2), (9, 2), (9, 1), (10, 0)]


def bezier_action(rng: random.Random, action, index):
    weight = rng.choice([1.0, 2.5, 0.0, 1.0 + 1e-17])
    a, z = sorted([rng.uniform(-0.2, 1.2), rng.uniform(-0.2, 1.2)])
    return [action, index, *point(rng), weight, a, z, *trsf(rng)]


def append_bezier_requests(rng: random.Random, requests: list[str]) -> None:
    for index in range(48):
        rational = index % 2 == 1
        spec = bezier(poles(rng, 2 + index % 6, rational), rational)
        chosen = [BEZIER_ACTIONS[(index + k) % len(BEZIER_ACTIONS)] for k in range(3)]
        actions = [c for action, pole in chosen for c in bezier_action(rng, action, pole)]
        requests.append('g2_bezier ' + fmt(spec + params([0.0, 0.3, 1.0, 1.4]) + [len(chosen)] + actions))
    closed = bezier([[0, 0, 1], [2, 2, 3], [-1, 1, 1], [0, 0, 1]], True)
    requests.append('g2_bezier ' + fmt(closed + params([0.5]) + [2] + bezier_action(rng, 7, 4) + bezier_action(rng, 9, 1)))


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
    return [action, i, j, k, *point(rng), weight, a, z, rng.randrange(2), *trsf(rng)]


def append_bspline_requests(rng: random.Random, requests: list[str]) -> None:
    for index in range(96):
        degree = 1 + index % 4
        if index % 3 == 2:
            spec, nk, u = bspline_periodic(rng, degree, degree + 3, index % 2 == 0), degree + 4, [0.0, 1.5, 4.0, -2.0]
        else:
            spec = bspline_random(rng, degree, 1 + index % 3, index % 2 == 0, 1 + (index % 7 == 0) * (degree - 1))
            nk, u = 3 + index % 3, [0.0, 0.37, 1.0, 1.2]
        actions = []
        for step in range(3):
            actions += bspline_action(rng, 1 + (index * 3 + step) % 24, nk)
        requests.append('g2_bspline ' + fmt(spec + params(u) + [3] + actions))
    uniform = bspline(2, [[float(i), 1] for i in range(8)], poles(rng, 5, False))
    for action in (8, 9, 10, 16, 17, 18, 22, 23):
        requests.append('g2_bspline ' + fmt(uniform + params([2.5, 3.0]) + [1] + bspline_action(rng, action, 8)))


def append_evalrep_requests(rng: random.Random, requests: list[str]) -> None:
    owners = [bezier(poles(rng, 4, False)), bspline_random(rng, 3, 2, True), offset(0.3, conic(1, [0, 0], [1, 0], True, 2.0))]
    for owner in owners:
        for kind in range(3):
            for order in (0, 2, 3):
                rep = rng.choice([bezier(poles(rng, 4, True), True), conic(2, [0, 0], [1, 0], True, 2.0, 1.0),
                                  line([0, 0], [1, 0])])
                desc = [kind, order, rng.randrange(2), 0.2, 0.9, rng.choice([1.0, 2.0, -0.5, 0.0]), rng.uniform(-1, 1)]
                requests.append('g2_evalrep ' + fmt(owner + params([0.0, 0.5, 1.0, 3.0]) + rep + desc + trsf(rng)))


def append_tkg2d_requests(requests: list[str]) -> None:
    """Append the whole TKG2d corpus; append_tkg2d_requests(requests)."""
    rng = random.Random(20250928)
    append_simple_requests(rng, requests)
    append_curve_requests(rng, requests, curve_specs(rng))
    append_conic_requests(rng, requests)
    append_trimmed_requests(rng, requests)
    append_offset_requests(rng, requests)
    append_bezier_requests(rng, requests)
    append_bspline_requests(rng, requests)
    append_evalrep_requests(rng, requests)
