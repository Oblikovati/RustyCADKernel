"""Deterministic requests for the Adaptor2d and Geom2dAdaptor part of the TKG2d oracle.

Each request is one `operation numbers...` line read by tools/oracle/tkg2d_probe_adaptor.cxx
and crates/tkg2d/examples/g2_probe/adaptor_cases.rs. Adaptor specifications follow the
grammar documented in tkg2d_probe_adaptor.cxx (a2_adaptor); curve specifications the one of
tkg2d_probe_common.cxx (g2_curve).
"""
from __future__ import annotations
import math
import random
from tkg2d_corpus import (bezier, bspline, bspline_periodic, bspline_random, conic, fmt, line, offset, params, point,
                          poles, trimmed, unit)

CONFUSION = 1e-7


def a_line(p, d, u1, u2):
    return [0, *p, *d, u1, u2]


def a_geom(curve, bounds=None):
    return [3, *curve] if bounds is None else [4, *bounds, *curve]


def a_offset(adaptor, distance=None, bounds=None):
    if distance is None:
        return [5, *adaptor]
    return [6, distance, *adaptor] if bounds is None else [7, distance, *bounds, *adaptor]


BASE = [8]


def knotted_bspline(rng: random.Random, rational=False):
    """Degree 3 with interior multiplicities 1, 2 and 3: C2, C1 and C0 knots."""
    knots = [[0.0, 4], [0.25, 1], [0.5, 2], [0.75, 3], [1.0, 4]]
    return bspline(3, knots, poles(rng, 10, rational), rational=rational)


def cusp_bezier():
    return bezier([[0, 0, 1], [0, 0, 1], [1, 1, 1], [2, 0, 1]])


def geom_curves(rng: random.Random):
    """(curve, bounds, parameters) triples covering every Geom2dAdaptor_Curve kind."""
    circle = conic(1, [1, -1], [0, 1], True, 2.0)
    indirect = conic(1, [0, 0], [1, 0], False, 1.5)
    knotted = knotted_bspline(rng)
    periodic = bspline_periodic(rng, 3, 6, False)
    cases = [(line([1, 2], [0.6, 0.8]), [-3.0, 4.0], [-3.0, 0.0, 1.5, 4.0]),
             (circle, None, [0.0, 1.0, math.pi, 2 * math.pi]), (circle, [0.5, 2.0], [0.5, 1.0, 2.0, 7.0]),
             (indirect, [1.0, 1.0], [1.0, 0.25]), (conic(2, [0, 0], [1, 0], True, 3.0, 1.0), None, [0.0, 0.7, 4.0]),
             (conic(3, [1, 0], [0, 1], False, 2.0, 0.5), [-1.0, 1.5], [-1.0, 0.3, 1.5]),
             (conic(4, [0, 0], [1, 0], True, 0.75), [-2.0, 2.0], [-2.0, 0.0, 2.0]),
             (bezier(poles(rng, 5, False)), None, [0.0, 0.3, 1.0, 1.4]),
             (bezier(poles(rng, 4, True), True), [0.2, 0.9], [0.2, 0.5, 0.9, -0.2]),
             (knotted, None, [0.0, 0.25, 0.5, 0.6, 0.75, 1.0]), (knotted, [0.25, 0.75], [0.25, 0.4, 0.5, 0.75]),
             (knotted, [0.3, 0.45], [0.3, 0.45, 0.4]), (knotted, [0.5, 0.5], [0.5]), (knotted, [-0.5, 1.5], [-0.5, 0.2, 1.5]),
             (knotted, [0.2500000000000001, 0.7499999999999999], [0.2500000000000001, 0.7499999999999999]),
             (knotted_bspline(rng, True), [0.1, 0.9], [0.1, 0.5, 0.9]),
             (periodic, None, [0.0, 2.5, 6.0]), (periodic, [-1.0, 7.0], [-1.0, 3.0, 7.0]), (periodic, [1.5, 3.5], [1.5, 2.0, 3.5]),
             (bspline_random(rng, 2, 3, True), [0.0, 1.0], [0.0, 0.33, 1.0]),
             (bspline(1, [[0.0, 2], [0.5, 1], [1.0, 2]], poles(rng, 3, False)), None, [0.0, 0.5, 1.0]),
             (trimmed(0.5, 2.5, circle), [0.7, 2.0], [0.7, 2.0]), (trimmed(0.1, 0.8, knotted), None, [0.0, 0.5, 1.0]),
             (offset(0.5, circle), None, [0.0, 1.0, 3.0]), (offset(-3.0, circle), [0.0, 3.0], [0.0, 2.0, 3.0]),
             (offset(0.25, cusp_bezier()), [0.0, 1.0], [0.0, 1e-9, 0.5, 1.0]),
             (offset(-0.25, cusp_bezier()), None, [0.0, 0.5]),
             (offset(0.1, bezier([[0, 0, 1], [1e-110, 0, 1], [1, 1, 1], [2, 0, 1]])), None, [0.0, 0.5]),
             (offset(0.1, knotted, True), [0.1, 0.9], [0.1, 0.5, 0.9]), (offset(0.2, offset(0.3, circle)), None, [0.5, 4.0]),
             (offset(0.2, trimmed(0.5, 2.0, circle)), None, [0.5, 1.0, 2.0]), (offset(0.4, periodic), [0.5, 5.0], [0.5, 2.0, 5.0]),
             (offset(0.3, bspline(2, [[0.0, 3], [0.5, 2], [1.0, 3]], [[0, 0, 1], [1, 0, 1], [2, 0, 1], [3, 0, 1], [4, 0, 1]])),
              None, [0.0, 0.5, 1.0]),
             (offset(0.3, conic(4, [0, 0], [1, 0], True, 1.0)), [-1.0, 1.0], [-1.0, 0.0, 1.0])]
    return cases


def line_adaptors(rng: random.Random):
    """(adaptor, parameters) pairs of Adaptor2d_Line2d in its three constructions."""
    return [(a_line([0, 0], [1, 0], 0.0, 10.0), [0.0, 5.0, 10.0]),
            (a_line(point(rng), unit(rng), -5.0, 5.0), [-5.0, 0.0, 5.0, 7.5]),
            ([1, *point(rng), *unit(rng)], [0.0, 1e300]), ([2, 1.0, 2.0, 0.6, 0.8, 2.0, 8.0], [2.0, 8.0]),
            (a_line([0, 0], [1, 0], 3.0, 3.0), [3.0])]


def offset_adaptors(rng: random.Random):
    """(adaptor, parameters) pairs of Adaptor2d_OffsetCurve over line, circle, B-spline and cusp bases."""
    circle = a_geom(conic(1, [1, -1], [0, 1], True, 2.0))
    indirect = a_geom(conic(1, [0, 0], [1, 0], False, 1.5))
    knotted = a_geom(knotted_bspline(rng))
    cusp = a_geom(cusp_bezier(), [0.0, 1.0])
    horizontal = a_line([0, 0], [1, 0], 0.0, 10.0)
    cases = [(a_offset(horizontal, 3.0), [0.0, 5.0]), (a_offset(horizontal, -2.0, [1.0, 8.0]), [1.0, 5.0]),
             (a_offset(horizontal), [5.0]), (a_offset(circle, 0.5), [0.0, 1.0, 2 * math.pi]),
             (a_offset(circle, -2.0), [0.0, 1.0]), (a_offset(circle, -3.0), [0.5]), (a_offset(indirect, 0.5), [0.5]),
             (a_offset(indirect, -1.5), [0.5]), (a_offset(indirect, 2.0), [0.5]), (a_offset(circle, 0.0), [0.5]),
             (a_offset(knotted, 0.1), [0.0, 0.2, 0.5, 1.0]), (a_offset(knotted, 0.1, [0.3, 0.8]), [0.3, 0.8]),
             (a_offset(knotted, 0.1, [0.1, 0.2]), [0.15]), (a_offset(knotted, 0.1, [-1.0, 0.2]), [0.1]),
             (a_offset(knotted, 0.1, [0.9, 2.0]), [0.95]), (a_offset(knotted), [0.5]), (a_offset(knotted, 0.0), [0.5]),
             (a_offset(a_geom(knotted_bspline(rng), [0.0, 0.7]), 0.2), [0.0, 0.7]),
             (a_offset(cusp, 0.25), [0.0, 0.5, 1.0]), (a_offset(cusp, -0.25, [0.0, 0.5]), [0.0, 0.25]),
             (a_offset(a_geom(bezier(poles(rng, 4, True), True)), 0.0), [0.0, 0.5]),
             (a_offset(a_geom(bezier(poles(rng, 30, False))), 0.0), [0.5]),
             (a_offset(a_geom(conic(2, [0, 0], [1, 0], True, 3.0, 1.0)), 0.0), [0.3]),
             (a_offset(a_geom(conic(3, [0, 0], [1, 0], True, 1.0, 0.5)), 0.0), [0.3]),
             (a_offset(a_geom(conic(4, [0, 0], [1, 0], True, 1.0)), 0.0), [0.3]),
             (a_offset(a_geom(conic(2, [0, 0], [1, 0], True, 3.0, 1.0)), 0.4), [0.3]),
             (a_offset(a_offset(circle, 0.5), 0.25), [0.0, 1.0]), (a_offset(a_offset(horizontal, 1.0), 1.0), [2.0]),
             (a_offset(BASE), [0.0]), (a_offset(BASE, 1.0), [0.0]),
             (a_offset(a_geom(offset(0.2, conic(1, [0, 0], [1, 0], True, 1.0))), 0.1), [0.5]),
             (a_offset(a_geom(bspline(1, [[0.0, 2], [0.5, 1], [1.0, 2]], poles(rng, 3, False))), 0.1), [0.25])]
    return cases


def append_adaptor_requests(rng: random.Random, requests: list[str]) -> None:
    for curve, bounds, values in geom_curves(rng):
        window = [values[0], values[-1]] if values[0] <= values[-1] else [values[-1], values[0]]
        requests.append('a2_adaptor ' + fmt(a_geom(curve, bounds) + params(values) + window))
    for adaptor, values in line_adaptors(rng) + offset_adaptors(rng):
        requests.append('a2_adaptor ' + fmt(adaptor + params(values) + [values[0], values[-1] + 1.0]))
    requests.append('a2_adaptor ' + fmt(BASE + params([0.5]) + [0.0, 1.0]))
    knotted = knotted_bspline(rng)
    for bounds in ([5.0, 1.0], [0.5, 0.5 - 2 * CONFUSION], [0.5 + CONFUSION / 2, 0.5]):
        requests.append('a2_adaptor ' + fmt(a_geom(knotted, bounds) + params([0.5]) + [0.0, 1.0]))
    requests.append('a2_adaptor ' + fmt(a_geom(knotted) + params([0.5]) + [0.8, 0.2]))


def append_random_adaptor_requests(rng: random.Random, requests: list[str]) -> None:
    """Random B-spline and Bezier adaptors on random sub-ranges: cache spans, boundary spans."""
    for index in range(24):
        degree = 1 + index % 5
        if index % 4 == 3:
            curve, low, high = bspline_periodic(rng, degree, degree + 4, index % 2 == 0), -2.0, 9.0
        elif index % 4 == 2:
            curve, low, high = bezier(poles(rng, degree + 1, index % 3 == 0), index % 3 == 0), -0.3, 1.3
        else:
            curve, low, high = bspline_random(rng, degree, 1 + index % 4, index % 2 == 1, 1 + index % degree), -0.2, 1.2
        u1, u2 = sorted([rng.uniform(low, high), rng.uniform(low, high)])
        values = [u1, u2, rng.uniform(u1, u2), rng.uniform(u1, u2), rng.uniform(low, high), u1]
        adaptor = a_geom(curve, [u1, u2])
        if index % 3 == 1:
            adaptor = a_offset(adaptor, rng.uniform(-1, 1))
        if index % 6 == 5:
            adaptor = a_geom(offset(rng.uniform(-0.5, 0.5), curve, True), [u1, u2])
        requests.append('a2_adaptor ' + fmt(adaptor + params(values) + sorted([rng.uniform(u1, u2), rng.uniform(u1, u2)])))
    many_knots = [[0.0, 4]] + [[i / 110.0, 1] for i in range(1, 110)] + [[1.0, 4]]
    big = bspline(3, many_knots, poles(rng, 113, False))
    for adaptor in (a_geom(big), a_offset(a_geom(big), 0.0), a_geom(offset(0.1, trimmed(0.1, 0.9, big)))):
        requests.append('a2_adaptor ' + fmt(adaptor + params([0.0, 0.5]) + [0.25, 0.75]))


def append_load_requests(rng: random.Random, requests: list[str]) -> None:
    circle = conic(1, [0, 0], [1, 0], True, 2.0)
    knotted = knotted_bspline(rng)
    pairs = [(knotted, trimmed(0.5, 2.0, circle), [0.2, 0.8]), (circle, knotted, [0.1, 0.6]),
             (bezier(poles(rng, 4, True), True), offset(0.2, circle), [0.25, 0.75]),
             (offset(0.3, knotted, True), line([0, 0], [1, 0]), [0.2, 0.4]),
             (bspline_periodic(rng, 2, 5, True), conic(3, [0, 0], [1, 0], True, 2.0, 1.0), [-0.5, 1.0])]
    for c1, c2, (u1, u2) in pairs:
        requests.append('a2_load ' + fmt(c1 + c2 + params([u1, 0.5 * (u1 + u2), u2]) + [u1, u2]))


def append_offset_load_requests(rng: random.Random, requests: list[str]) -> None:
    circle = a_geom(conic(1, [0, 0], [1, 0], True, 2.0))
    knotted = a_geom(knotted_bspline(rng), [0.1, 0.9])
    horizontal = a_line([0, 0], [1, 0], 0.0, 10.0)
    cases = [(horizontal, circle, 0.5, 0.2, 1.5), (circle, knotted, -0.3, 0.2, 0.8), (knotted, BASE, 0.25, 0.3, 0.7),
             (BASE, horizontal, 1.0, 0.0, 2.0), (knotted, horizontal, 0.0, 1.0, 4.0)]
    for b1, b2, distance, w1, w2 in cases:
        requests.append('a2_offset_load ' + fmt(b1 + b2 + params([w1, 0.5 * (w1 + w2), w2]) + [distance, w1, w2]))


def append_evalrep_requests(rng: random.Random, requests: list[str]) -> None:
    owners = [(bezier(poles(rng, 4, False)), [0.1, 0.9]), (knotted_bspline(rng, True), [0.25, 0.75]),
              (offset(0.3, conic(1, [0, 0], [1, 0], True, 2.0)), [0.5, 2.0])]
    for owner, (u1, u2) in owners:
        for kind in range(3):
            rep = rng.choice([bezier(poles(rng, 4, True), True), conic(2, [0, 0], [1, 0], True, 2.0, 1.0)])
            desc = [kind, 2, rng.randrange(2), 0.2, 0.9, rng.choice([1.0, 2.0, -0.5]), rng.uniform(-1, 1)]
            requests.append('a2_evalrep ' + fmt(owner + rep + desc + params([u1, 0.5 * (u1 + u2), u2]) + [u1, u2]))


def append_tkg2d_adaptor_requests(requests: list[str]) -> None:
    """Append the Adaptor2d/Geom2dAdaptor corpus; append_tkg2d_adaptor_requests(requests)."""
    rng = random.Random(20260928)
    append_adaptor_requests(rng, requests)
    append_random_adaptor_requests(rng, requests)
    append_load_requests(rng, requests)
    append_offset_load_requests(rng, requests)
    append_evalrep_requests(rng, requests)
