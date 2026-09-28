"""Deterministic requests for the Geom2dEval curves and the Geom2dHash hashers.

Each request is one `operation numbers...` line read by tools/oracle/tkg2d_probe_eval.cxx
and crates/tkg2d/examples/g2_probe/eval.rs; eval curve specifications (kinds 11..16)
follow the grammar documented in tkg2d_probe_eval.cxx. The corpus has its own seed so
it never perturbs the Geom2d requests of tkg2d_corpus.py.
"""
from __future__ import annotations
import math
import random
from tkg2d_corpus import (bezier, bspline, bspline_periodic, bspline_random, conic, fmt, line, moves, offset,
                          params, point, poles, trimmed, trsf, unit)


def spiral(kind, p, d, *values):
    return [kind, *p, *d, *values]


def tbezier(points, alpha, weights=None, lower=1):
    flat = [c for pole in points for c in pole]
    stored = weights or []
    return [15, int(weights is not None), lower, len(points), *flat, len(stored), *stored, alpha]


def aht(points, degree, alpha, beta, weights=None, lower=1):
    flat = [c for pole in points for c in pole]
    stored = weights or []
    return [16, int(weights is not None), lower, degree, alpha, beta, len(points), *flat, len(stored), *stored]


def planar(rng: random.Random, count):
    return [point(rng, 4) for _ in range(count)]


def positive(rng: random.Random, count):
    return [rng.uniform(0.3, 3.0) for _ in range(count)]


def analytic_specs(rng: random.Random):
    """Valid spirals, involutes and sine waves, then each constructor refusal."""
    specs = []
    for _ in range(4):
        specs.append(spiral(11, point(rng), unit(rng), rng.uniform(0, 3), rng.uniform(0.05, 2)))
        specs.append(spiral(12, point(rng), unit(rng), rng.uniform(0.1, 3)))
        specs.append(spiral(13, point(rng), unit(rng), rng.uniform(0.1, 3), rng.uniform(0.01, 1)))
        specs.append(spiral(14, point(rng), unit(rng), rng.uniform(0.1, 3), rng.uniform(0.1, 4), rng.uniform(-3, 3)))
    specs += [spiral(11, [0, 0], [1, 0], 0.0, 1.0), spiral(11, [0, 0], [1, 0], -1.0, 1.0),
              spiral(11, [0, 0], [1, 0], 0.0, 0.0), spiral(11, [0, 0], [1, 0], 0.0, -1.0),
              spiral(12, [0, 0], [0, 1], 0.0), spiral(12, [0, 0], [1, 0], -1.0), spiral(12, [1, 2], [1, 0], 1e-300),
              spiral(13, [0, 0], [1, 0], 0.0, 0.2), spiral(13, [0, 0], [1, 0], -1.0, 0.2),
              spiral(13, [0, 0], [1, 0], 1.0, 0.0), spiral(13, [0, 0], [1, 0], 1.0, -0.2), spiral(13, [0, 0], [1, 0], 1.0, 5.0),
              spiral(14, [0, 0], [1, 0], 0.0, 1.0, 0.0), spiral(14, [0, 0], [1, 0], 1.0, 0.0, 0.0),
              spiral(14, [0, 0], [1, 0], -2.0, 1.0, 0.0), spiral(14, [0, 0], [0, -1], 2.0, 3.0, 0.0)]
    return specs


def tbezier_specs(rng: random.Random):
    """Orders 1 to 4, rational or not, with shifted lower bounds, closed and invalid curves."""
    specs = []
    for count in (3, 5, 7, 9):
        alpha = rng.uniform(0.2, 3.0)
        specs.append(tbezier(planar(rng, count), alpha, lower=rng.choice([1, 0, -3, 5])))
        specs.append(tbezier(planar(rng, count), alpha, positive(rng, count), lower=rng.choice([1, 0, -3, 5])))
    closed = [[1, 1], [2, 3], [0, 0], [-1, 2], [0.5, 0.5]]
    specs += [tbezier([[0, 0], [0, 1], [1, 0]], 1.0), tbezier(closed, 2.0), tbezier(closed, 2.0, [1, 1, 1, 1, 1]),
              tbezier(planar(rng, 3), 1.0, [1.0, 1.0, 1.0]),
              tbezier(planar(rng, 2), 1.0), tbezier(planar(rng, 1), 1.0), tbezier([], 1.0), tbezier(planar(rng, 4), 1.0),
              tbezier(planar(rng, 3), 0.0), tbezier(planar(rng, 3), -1.0), tbezier(planar(rng, 3), 1.0, [1, 1, 1, 1, 1]),
              tbezier(planar(rng, 3), 1.0, [1, -1, 1]), tbezier(planar(rng, 3), 1.0, [1, 0, 1]),
              tbezier(planar(rng, 3), 0.0, [1, 2]), tbezier(planar(rng, 5), 1e-3, positive(rng, 5))]
    return specs


def aht_specs(rng: random.Random):
    """Every basis shape (algebraic, hyperbolic, trigonometric, mixed), rational or not,
    closed curves and each constructor refusal."""
    specs = []
    for degree in range(0, 5):
        for alpha, beta in ((0.0, 0.0), (rng.uniform(0.2, 2), 0.0), (0.0, rng.uniform(0.2, 4)),
                            (rng.uniform(0.2, 2), rng.uniform(0.2, 4))):
            count = degree + 1 + (2 if alpha > 0 else 0) + (2 if beta > 0 else 0)
            lower = rng.choice([1, 0, -2, 4])
            specs.append(aht(planar(rng, count), degree, alpha, beta, lower=lower))
            specs.append(aht(planar(rng, count), degree, alpha, beta, positive(rng, count), lower=lower))
    closed = [[1, 1], [2, -1], [-2, 1]]
    specs += [aht(closed, 2, 0.0, 0.0), aht(closed, 2, 0.0, 0.0, [1.0, 1.0, 1.0]),
              aht([[0, 0], [1, 0], [1, 1], [0, 1], [0, 1], [1, 0]], 1, 1.0, 1.0),
              aht(planar(rng, 3), -1, 0.0, 0.0), aht(planar(rng, 3), 2, -1.0, 0.0), aht(planar(rng, 3), 2, 0.0, -1.0),
              aht(planar(rng, 4), 1, 1.0, 1.0), aht(planar(rng, 3), 2, 0.0, 0.0, [1.0, 1.0]),
              aht(planar(rng, 3), 2, 0.0, 0.0, [1.0, -1.0, 1.0]), aht(planar(rng, 3), 2, 0.0, 0.0, [1.0, 0.0, 1.0]),
              aht(planar(rng, 7), 2, 30.0, 25.0, positive(rng, 7))]
    return specs


ANALYTIC_PARAMS = [0.0, 1.0, 0.5, -0.25, 7.0, 2 * math.pi, 40.0, -3.0, 1e-300]
BEZIER_PARAMS = [0.0, 1.0, 0.5, 0.25, -0.5, 1.3, math.pi, 1e-300]


def eval_specs(rng: random.Random):
    """(spec, parameters) pairs of every eval curve kind, bare and nested."""
    pairs = [(spec, ANALYTIC_PARAMS) for spec in analytic_specs(rng)]
    pairs += [(spec, BEZIER_PARAMS) for spec in tbezier_specs(rng) + aht_specs(rng)]
    sine = spiral(14, [0, 0], [1, 0], 2.0, 3.0, 0.5)
    tb = tbezier(planar(rng, 5), 0.5, positive(rng, 5))
    pairs += [(trimmed(0.5, 2.0, sine), ANALYTIC_PARAMS), (trimmed(2.0, 0.5, sine, False), ANALYTIC_PARAMS),
              (trimmed(0.2, 1.2, tb), BEZIER_PARAMS), (offset(0.3, sine), ANALYTIC_PARAMS),
              (offset(-0.2, spiral(12, [0, 0], [1, 0], 1.0)), ANALYTIC_PARAMS),
              (offset(0.1, aht(planar(rng, 5), 2, 0.0, 1.5)), BEZIER_PARAMS),
              (offset(0.2, trimmed(0.1, 0.9, tb)), BEZIER_PARAMS)]
    return pairs


def append_eval_curve_requests(rng: random.Random, requests: list[str]) -> None:
    for spec, u in eval_specs(rng):
        requests.append('g2_curve ' + fmt(spec + params(u)))
        requests.append('g2_eval ' + fmt(spec + params(u)))
        requests.append('g2_transform ' + fmt(spec + params(u[:4]) + trsf(rng) + moves(rng)))


def hash_items(rng: random.Random):
    """Hasher inputs: copies, near copies, every curve class, eval curves, nulls, aliases."""
    circle = conic(1, [0, 0], [1, 0], True, 5.0)
    near = conic(1, [0, 0], [1, 0], True, 5.0 + 5e-13)
    bez = [[0, 0, 1], [1, 2, 1], [3, 2, 1], [4, 0, 1]]
    spline = bspline_random(rng, 3, 2, True)
    periodic = bspline_periodic(rng, 3, 6, False)
    curves = [line([1, 2], [1, 0]), line([1, 2], [1, 0]), line([1, 2 + 1e-8], [1, 0]), line([1e10, 1e10], [0, 1]),
              circle, circle, near, conic(1, [0, 0], [0, 1], True, 5.0), conic(1, [0, 0], [1, 0], False, 5.0),
              conic(2, [0, 0], [1, 0], True, 10.0, 5.0), conic(2, [0, 0], [1, 0], True, 5.0, 5.0),
              conic(3, [0, 0], [1, 0], True, 5.0, 3.0), conic(4, [0, 0], [1, 0], True, 2.0),
              conic(4, [0, 0], [1, 0], True, 2.0 + 1e-11), bezier(bez), bezier(bez),
              bezier([[0, 0, 1], [1, 2, 2], [3, 2, 1], [4, 0, 1]], True), bezier([[0, 0, 1], [1, 2, 1], [3, 2, 1]]),
              spline, spline, periodic, periodic, bspline_random(rng, 3, 2, False),
              trimmed(0.0, 10.0, line([0, 0], [1, 0])), trimmed(0.0, 20.0, line([0, 0], [1, 0])),
              trimmed(0.5, 2.0, circle), offset(5.0, line([0, 0], [1, 0])), offset(5.0, line([0, 0], [1, 0])),
              offset(0.5, circle), spiral(14, [0, 0], [1, 0], 2.0, 3.0, 0.5), spiral(14, [0, 0], [1, 0], 2.0, 3.0, 0.5),
              spiral(12, [0, 0], [1, 0], 1.0), tbezier([[0, 0], [0, 1], [1, 0]], 1.0),
              aht([[0, 0], [1, 1], [2, 0]], 2, 0.0, 0.0), offset(0.3, spiral(11, [0, 0], [1, 0], 0.5, 1.0)),
              trimmed(0.0, 1.0, spiral(13, [0, 0], [1, 0], 1.0, 0.2))]
    items = [[0, *spec] for spec in curves]
    items += [[1], [1], [2, 0], [2, 4], [2, 30]]
    rng.shuffle(items)
    return items


TOLERANCES = [(1e-12, 1e-7), (0.1, 0.1), (1e-12, 0.0), (1e-12, 1e-300), (0.0, 1e-7), (1e-6, 1e-3), (1e-7, 2.5)]


def append_hash_requests(rng: random.Random, requests: list[str]) -> None:
    for comp, tol in TOLERANCES:
        items = hash_items(rng)
        # Aliases refer to earlier items only: move each alias after the item it names.
        ordered, pending = [], []
        for item in items:
            (pending if item[0] == 2 else ordered).append(item)
        ordered += [[2, min(item[1], len(ordered) - 1)] for item in pending]
        flat = [c for item in ordered for c in item]
        requests.append('g2_hash ' + fmt([comp, tol, len(ordered), *flat]))
    requests.append('g2_hash ' + fmt([1e-12, 1e-7, 2, 0, *spiral(11, [0, 0], [1, 0], -1.0, 1.0), 1]))


def frame(p, x, y):
    return [*p, *x, *y]


def append_hash_frame_requests(rng: random.Random, requests: list[str]) -> None:
    edges = [[0.0, -0.0], [1.5e-7, -1.5e-7], [0.5e-7, 2.5e-7], [1e300, -1e300], [9.223372036854775808e11, 0.0],
             [-9.223372036854775808e11, 9.2233720368547748e11], [1e-300, 3.0]]
    for comp, tol in TOLERANCES:
        for index in range(6):
            p1 = point(rng) if index < 3 else rng.choice(edges)
            p2 = p1 if index % 3 == 0 else (rng.choice(edges) if index % 3 == 1 else point(rng))
            d1 = unit(rng)
            d2 = d1 if index % 2 == 0 else unit(rng)
            f1 = frame(point(rng), unit(rng), unit(rng))
            f2 = f1 if index % 3 == 0 else frame(point(rng), unit(rng), unit(rng))
            requests.append('g2_hash_frame ' + fmt([comp, tol, *p1, *p2, *d1, *d2, *f1, *f2]))
    requests.append('g2_hash_frame ' + fmt([1e-12, 1e-7, 0, 0, 1, 1, 1, 0, 1, 0, *frame([0, 0], [1, 0], [1, 0]),
                                            *frame([0, 0], [1, 0], [0, 1])]))


def append_tkg2d_eval_requests(requests: list[str]) -> None:
    """Append the Geom2dEval and Geom2dHash requests; append_tkg2d_eval_requests(requests)."""
    rng = random.Random(20260928)
    append_eval_curve_requests(rng, requests)
    append_hash_requests(rng, requests)
    append_hash_frame_requests(rng, requests)
