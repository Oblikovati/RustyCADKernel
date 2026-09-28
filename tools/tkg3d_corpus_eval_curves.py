"""Deterministic requests for the GeomEval curves (TKG3d numeric oracle).

Each request is one `operation numbers...` line read by tools/oracle/tkg3d_probe.cxx and
crates/tkg3d/examples/numeric_probe.rs; eval curve specifications (g3_curve kinds 20..23)
follow the grammar documented in tools/oracle/tkg3d_probe_eval_curves.cxx. The module has
its own seed so it never perturbs the other TKG3d request families.
"""
from __future__ import annotations
import math
import random
from tkg3d_corpus import fmt, frame, moves, offset, params, point, trimmed, trsf, unit


def sine(f, amplitude, omega, phase):
    return [20, *f, amplitude, omega, phase]


def helix(f, radius, pitch):
    return [21, *f, radius, pitch]


def tbezier(points, alpha, weights=None, lower=1):
    flat = [c for pole in points for c in pole]
    stored = weights or []
    return [22, int(weights is not None), lower, len(points), *flat, len(stored), *stored, alpha]


def aht(points, degree, alpha, beta, weights=None, lower=1):
    flat = [c for pole in points for c in pole]
    stored = weights or []
    return [23, int(weights is not None), lower, degree, alpha, beta, len(points), *flat, len(stored), *stored]


def spatial(rng: random.Random, count):
    return [point(rng, 4) for _ in range(count)]


def positive(rng: random.Random, count):
    return [rng.uniform(0.3, 3.0) for _ in range(count)]


XY = [0, 0, 0, 0, 0, 1, 1, 0, 0]


def analytic_specs(rng: random.Random):
    """Valid sine waves and helices in random frames, then each constructor refusal."""
    specs = []
    for _ in range(5):
        specs.append(sine(frame(rng), rng.uniform(0.1, 3), rng.uniform(0.1, 4), rng.uniform(-3, 3)))
        specs.append(helix(frame(rng), rng.uniform(0.1, 5), rng.uniform(-10, 10)))
    specs += [sine(XY, 2.0, 3.0, 0.0), sine(XY, 1e-10, 1.0, 0.0), sine(XY, 0.0, 1.0, 0.0), sine(XY, -1.0, 1.0, 0.0),
              sine(XY, 1.0, 0.0, 0.0), sine(XY, 1.0, -1.0, 0.0), sine(XY, 1.0, 1e3, 0.5),
              helix(XY, 5.0, 10.0), helix(XY, 5.0, 0.0), helix(XY, 5.0, -10.0), helix(XY, 0.0, 1.0),
              helix(XY, -1.0, 1.0), helix(XY, 1e-300, 1.0)]
    return specs


def tbezier_specs(rng: random.Random):
    """Orders 1 to 4, rational or not, with shifted lower bounds, closed and invalid curves."""
    specs = []
    for count in (3, 5, 7, 9):
        alpha = rng.uniform(0.2, 3.0)
        specs.append(tbezier(spatial(rng, count), alpha, lower=rng.choice([1, 0, -3, 5])))
        specs.append(tbezier(spatial(rng, count), alpha, positive(rng, count), lower=rng.choice([1, 0, -3, 5])))
    closed = [[1, 1, 1], [2, 3, -1], [0, 0, 2], [-1, 2, 0], [0.5, 0.5, 0.5]]
    specs += [tbezier([[0, 0, 0], [0, 1, 0], [1, 0, 0]], 1.0), tbezier(closed, 2.0), tbezier(closed, 2.0, [1] * 5),
              tbezier(spatial(rng, 3), 1.0, [1.0, 1.0, 1.0]),
              tbezier(spatial(rng, 2), 1.0), tbezier(spatial(rng, 1), 1.0), tbezier([], 1.0),
              tbezier(spatial(rng, 4), 1.0), tbezier(spatial(rng, 3), 0.0), tbezier(spatial(rng, 3), -1.0),
              tbezier(spatial(rng, 3), 1.0, [1, 1, 1, 1, 1]), tbezier(spatial(rng, 3), 1.0, [1, -1, 1]),
              tbezier(spatial(rng, 3), 1.0, [1, 0, 1]), tbezier(spatial(rng, 3), 0.0, [1, 2]),
              tbezier(spatial(rng, 5), 1e-3, positive(rng, 5))]
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
            specs.append(aht(spatial(rng, count), degree, alpha, beta, lower=lower))
            specs.append(aht(spatial(rng, count), degree, alpha, beta, positive(rng, count), lower=lower))
    closed = [[1, 1, 1], [2, -1, 0], [-2, 1, -1]]
    full = [[0, 0, 0], [1, 0, 0], [1, 1, 0], [0, 1, 0], [0, 1, 1], [1, 0, 1]]
    specs += [aht(closed, 2, 0.0, 0.0), aht(closed, 2, 0.0, 0.0, [1.0, 1.0, 1.0]), aht(full, 1, 1.0, 1.0),
              aht([[0, 0, 0], [0, 1, 0], [1, 0, 0]], 0, 0.0, 1.0),
              aht(spatial(rng, 3), -1, 0.0, 0.0), aht(spatial(rng, 3), 2, -1.0, 0.0), aht(spatial(rng, 3), 2, 0.0, -1.0),
              aht(spatial(rng, 4), 1, 1.0, 1.0), aht(spatial(rng, 3), 2, 0.0, 0.0, [1.0, 1.0]),
              aht(spatial(rng, 3), 2, 0.0, 0.0, [1.0, -1.0, 1.0]), aht(spatial(rng, 3), 2, 0.0, 0.0, [1.0, 0.0, 1.0]),
              aht(spatial(rng, 7), 2, 30.0, 25.0, positive(rng, 7))]
    return specs


ANALYTIC_PARAMS = [0.0, 1.0, 0.5, -0.25, 7.0, 2 * math.pi, 40.0, -3.0, 1e-300]
BEZIER_PARAMS = [0.0, 1.0, 0.5, 0.25, -0.5, 1.3, math.pi, 1e-300]


def eval_specs(rng: random.Random):
    """(spec, parameters) pairs of every eval curve kind, bare and nested in trimmed and
    offset curves."""
    pairs = [(spec, ANALYTIC_PARAMS) for spec in analytic_specs(rng)]
    pairs += [(spec, BEZIER_PARAMS) for spec in tbezier_specs(rng) + aht_specs(rng)]
    wave = sine(XY, 2.0, 3.0, 0.5)
    spring = helix(XY, 1.0, 0.5)
    tb = tbezier(spatial(rng, 5), 0.5, positive(rng, 5))
    pairs += [(trimmed(0.5, 2.0, wave), ANALYTIC_PARAMS), (trimmed(2.0, 0.5, wave, False), ANALYTIC_PARAMS),
              (trimmed(0.2, 1.2, tb), BEZIER_PARAMS), (trimmed(-1.0, 4.0, spring), ANALYTIC_PARAMS),
              (offset(0.3, [0, 0, 1], wave), ANALYTIC_PARAMS), (offset(-0.2, unit(rng), spring), ANALYTIC_PARAMS),
              (offset(0.1, [0, 0, 1], aht(spatial(rng, 5), 2, 0.0, 1.5)), BEZIER_PARAMS),
              (offset(0.2, unit(rng), trimmed(0.1, 0.9, tb)), BEZIER_PARAMS)]
    return pairs


def append_eval_curve_requests(requests: list[str]) -> None:
    """Append g3_curve, g3_eval_curve and g3_transform requests for every eval curve;
    append_eval_curve_requests(requests)."""
    rng = random.Random(20260929)
    for spec, u in eval_specs(rng):
        requests.append('g3_curve ' + fmt(spec + params(u)))
        requests.append('g3_eval_curve ' + fmt(spec + params(u)))
        requests.append('g3_transform ' + fmt(spec + params(u[:3]) + trsf(rng) + moves(rng)))
