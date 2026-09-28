"""Deterministic requests for the Geom2dGridEval grid evaluators.

Each request is one `operation numbers...` line read by tools/oracle/tkg2d_probe_grid.cxx
and crates/tkg2d/examples/g2_probe/grid.rs. A parameter grid is written `lower n u*n`
(the bounds of the NCollection_Array1 the evaluators receive). Curve and adaptor
specifications follow tkg2d_probe_common.cxx (g2_curve) and tkg2d_probe_adaptor.cxx
(a2_adaptor). The corpus has its own seed so it never perturbs the other TKG2d requests.
"""
from __future__ import annotations
import math
import random
from tkg2d_adaptor_corpus import BASE, a_geom, a_line, a_offset, cusp_bezier, geom_curves, knotted_bspline
from tkg2d_corpus import (bezier, bspline, bspline_periodic, bspline_random, conic, fmt, line, offset, point, poles,
                          trimmed, unit)
from tkg2d_eval_corpus import aht, spiral, tbezier


def grid(values, lower=1):
    """A parameter grid with its lower bound; grid([0.0, 1.0], -3)."""
    return [lower, len(values), *values]


def linspace(first, last, count):
    step = (last - first) / (count - 1)
    return [first + i * step for i in range(count)]


def spline_under(spec):
    """The B-spline spec under trimmed (7) and offset (8) wrappers, or None."""
    while spec[0] in (7, 8):
        spec = spec[5:] if spec[0] == 7 else spec[3:]
    return spec if spec[0] == 6 else None


def flat_span(flat, degree, count, u):
    """The 1-based flat-knot span that BSplCLib::LocateParameter gives `u`."""
    spans = [s for s in range(degree + 1, count + 1) if flat[s - 1] < flat[s]]
    below = [s for s in spans if flat[s - 1] <= u]
    return below[-1] if below else spans[0]


def dn_safe(spec, values):
    """Drops the parameters whose flat span index equals the last knot index of a
    non-periodic B-spline: Geom2dGridEval_BSplineCurve::EvaluateGridDN passes that index to
    BSplCLib::DN with the multiplicities, and BuildKnots then reads past both arrays
    natively (undefined), where the port raises Standard_OutOfRange."""
    spline = spline_under(spec)
    if spline is None or spline[2]:
        return values
    degree, nk = spline[3], spline[4]
    knots = spline[5:5 + 2 * nk]
    flat = [k for k, m in zip(knots[::2], knots[1::2]) for _ in range(int(m))]
    count = len(flat) - degree - 1
    return [u for u in values if flat_span(flat, degree, count, u) != nk]


def grids(rng: random.Random, first, last, spec):
    """Sorted dense grids (span-cache runs), sparse and shuffled ones (direct calls), a
    single value, values beyond the range and the empty grid, with varied lower bounds."""
    dense = linspace(first, last, 25)
    shuffled = linspace(first, last, 9)
    rng.shuffle(shuffled)
    outside = [first - 0.5 * (last - first), first, 0.5 * (first + last), last, last + 0.75 * (last - first), 1e-300]
    sets = [(dense, 1), (linspace(first, last, 3), 0), (shuffled, -3), ([rng.uniform(first, last)], 7), (outside, 2)]
    return [grid(dn_safe(spec, values), lower) for values, lower in sets] + [grid([])]


def conic_specs(rng: random.Random):
    """(spec, first, last): lines and every conic on direct and indirect frames, the
    degenerate zero-focal parabola, and conics built from gp_Ax22d or a directrix."""
    specs = []
    for sense in (True, False):
        specs += [(line(point(rng), unit(rng)), -5.0, 5.0),
                  (conic(1, point(rng), unit(rng), sense, rng.uniform(0.1, 5)), 0.0, 2 * math.pi),
                  (conic(2, point(rng), unit(rng), sense, 4.0, rng.uniform(0.1, 4)), -1.0, 7.0),
                  (conic(3, point(rng), unit(rng), sense, rng.uniform(0.1, 5), rng.uniform(0.1, 5)), -3.0, 3.0),
                  (conic(4, point(rng), unit(rng), sense, rng.uniform(0.1, 5)), -4.0, 4.0)]
    specs += [(conic(4, [0, 0], [1, 0], True, 0.0), -1.0, 1.0), (conic(1, [0, 0], [1, 0], True, 0.0), 0.0, 1.0),
              ([9, 1, 2, 1, 0, 0, -1, 3.0], 0.0, 2 * math.pi), ([10, 0, 0, 1, 0, 2, 3], -2.0, 2.0),
              (conic(3, [0, 0], [1, 0], True, 2.0, 1.0), -40.0, 800.0), (line([1e10, -1e10], [0.6, -0.8]), -1e12, 1e12)]
    return specs


def spline_specs(rng: random.Random):
    """(spec, first, last): Bezier curves of degree 1 to 6 and 25, B-splines with simple,
    multiple and C0 knots, periodic and rational ones, and a degree 1 spline with many
    knots, whose flat span indices fall inside the knot index range of the DN call."""
    specs = []
    for degree in (1, 2, 3, 6):
        specs.append((bezier(poles(rng, degree + 1, False)), 0.0, 1.0))
        specs.append((bezier(poles(rng, degree + 1, True), True), 0.0, 1.0))
    specs += [(bezier([[float(i), float(i % 3), 1.0] for i in range(26)]), 0.0, 1.0), (cusp_bezier(), 0.0, 1.0)]
    for degree in (1, 2, 3, 5):
        specs.append((bspline_random(rng, degree, 3, False), 0.0, 1.0))
        specs.append((bspline_random(rng, degree, 2, True), 0.0, 1.0))
        specs.append((bspline_periodic(rng, degree, degree + 4, degree % 2 == 1), 0.0, float(degree + 4)))
    uniform = bspline(1, [[0.0, 2]] + [[float(i), 1] for i in range(1, 9)] + [[9.0, 2]], poles(rng, 10, False))
    specs += [(knotted_bspline(rng), 0.0, 1.0), (knotted_bspline(rng, True), 0.0, 1.0), (uniform, 0.0, 9.0),
              (bspline(2, [[float(i), 1] for i in range(12)], poles(rng, 9, True), rational=True), 2.0, 9.0),
              (bspline_random(rng, 3, 2, False, 3), 0.0, 1.0)]
    return specs


def wrapped_specs(rng: random.Random):
    """(spec, first, last): trimmed curves (nested too), offsets of every basis kind
    (cusps, tiny tangents, offsets of offsets, trimmed and periodic bases) and eval
    curves, bare or wrapped, which the dispatcher leaves uninitialized."""
    circle = conic(1, [0, 0], [1, 0], True, 2.0)
    knotted = knotted_bspline(rng)
    tiny = bezier([[0, 0, 1], [1e-110, 0, 1], [1, 1, 1], [2, 0, 1]])
    sine = spiral(14, [0, 0], [1, 0], 2.0, 3.0, 0.5)
    return [(trimmed(0.5, 2.5, circle), 0.0, 3.0), (trimmed(0.1, 0.9, trimmed(0.0, 1.0, knotted)), 0.0, 1.0),
            (trimmed(0.5, 3.5, bspline_periodic(rng, 3, 6, False)), -1.0, 8.0),
            (offset(0.5, circle), 0.0, 2 * math.pi), (offset(-3.0, circle), 0.0, 2 * math.pi),
            (offset(1.5, line([0, 0], [0.6, 0.8])), -2.0, 2.0), (offset(0.3, conic(2, [0, 0], [1, 0], False, 3.0, 1.0)), 0.0, 6.0),
            (offset(0.2, conic(3, [0, 0], [1, 0], True, 1.0, 0.5)), -1.0, 1.0),
            (offset(0.5, conic(4, [0, 0], [1, 0], True, 1.0)), -1.0, 1.0),
            (offset(0.25, cusp_bezier()), 0.0, 1.0), (offset(-0.25, cusp_bezier()), 0.0, 1.0), (offset(0.1, tiny), 0.0, 1.0),
            (offset(0.2, offset(0.3, circle)), 0.0, 6.0), (offset(0.2, trimmed(0.5, 2.0, circle)), 0.0, 3.0),
            (offset(0.1, knotted, True), 0.0, 1.0), (offset(0.4, bspline_periodic(rng, 3, 7, True)), 0.0, 7.0),
            (offset(0.3, bezier(poles(rng, 5, True), True)), 0.0, 1.0),
            (offset(0.3, bspline(2, [[0.0, 3], [0.5, 2], [1.0, 3]], [[0, 0, 1], [1, 0, 1], [2, 0, 1], [3, 0, 1], [4, 0, 1]])),
             0.0, 1.0),
            (sine, -1.0, 4.0), (trimmed(0.5, 2.0, sine), 0.0, 3.0), (offset(0.3, sine), 0.0, 2.0),
            (spiral(12, [0, 0], [1, 0], 1.0), 0.0, 5.0), (tbezier([[0, 0], [1, 2], [3, 1]], 1.0, [1.0, 2.0, 1.0]), 0.0, 1.0),
            (aht([[0, 0], [1, 1], [2, 0], [3, 2], [4, 1]], 2, 0.0, 1.5), 0.0, 1.0)]


def append_grid_curve_requests(rng: random.Random, requests: list[str]) -> None:
    for spec, first, last in conic_specs(rng) + spline_specs(rng) + wrapped_specs(rng):
        for parameters in grids(rng, first, last, spec):
            requests.append('g2_grid ' + fmt(spec + parameters))


def adaptor_specs(rng: random.Random):
    """(adaptor, first, last): every adaptor kind; a Geom2dAdaptor_Curve with a range still
    goes through its curve, the other adaptors through the fallback, and the base class
    raises from GetType."""
    circle = conic(1, [1, -1], [0, 1], True, 2.0)
    knotted = knotted_bspline(rng)
    specs = [(a_line([0, 0], [1, 0], 0.0, 10.0), 0.0, 10.0), (a_line(point(rng), unit(rng), -5.0, 5.0), -5.0, 5.0),
             ([1, *point(rng), *unit(rng)], -3.0, 3.0), ([2, *point(rng), *unit(rng), -1.0, 2.0], -1.0, 2.0),
             (a_geom(knotted, [0.25, 0.75]), 0.0, 1.0), (a_geom(circle, [0.5, 2.0]), 0.0, 3.0),
             (a_geom(spiral(14, [0, 0], [1, 0], 2.0, 3.0, 0.5)), 0.0, 2.0),
             (a_offset(a_geom(circle), 0.5), 0.0, 2 * math.pi), (a_offset(a_geom(knotted), 0.1, [0.1, 0.9]), 0.1, 0.9),
             (a_offset(a_geom(cusp_bezier()), 0.25), 0.0, 1.0), (a_offset(a_geom(circle)), 0.0, 3.0),
             (a_offset(a_line([0, 0], [1, 0], 0.0, 4.0), 1.0), 0.0, 4.0), (BASE, 0.0, 1.0)]
    specs += [(a_geom(curve, bounds), min(u), max(u)) for curve, bounds, u in geom_curves(rng)[::4]]
    return specs


def adapted_curve(spec):
    """The curve of a Geom2dAdaptor_Curve spec (kinds 3 and 4), or a line spec otherwise."""
    if spec[0] == 3:
        return spec[1:]
    return spec[3:] if spec[0] == 4 else [0]


def append_grid_adaptor_requests(rng: random.Random, requests: list[str]) -> None:
    for spec, first, last in adaptor_specs(rng):
        for parameters in grids(rng, first, last, adapted_curve(spec))[:4]:
            requests.append('g2_grid_adaptor ' + fmt(spec + parameters))


def append_grid_evalrep_requests(rng: random.Random, requests: list[str]) -> None:
    """Bezier, B-spline and offset curves carrying each descriptor kind: the grids ignore it."""
    owners = [bezier(poles(rng, 4, False)), bspline_random(rng, 3, 2, True), offset(0.3, conic(1, [0, 0], [1, 0], True, 2.0))]
    for owner in owners:
        for kind in range(3):
            rep = rng.choice([bezier(poles(rng, 4, True), True), conic(2, [0, 0], [1, 0], True, 2.0, 1.0), line([0, 0], [1, 0])])
            desc = [kind, 3, rng.randrange(2), 0.2, 0.9, rng.choice([1.0, 2.0, -0.5]), rng.uniform(-1, 1)]
            requests.append('g2_grid_evalrep ' + fmt(owner + rep + desc + grid(dn_safe(owner, linspace(0.0, 1.0, 9)))))


def append_tkg2d_grid_requests(requests: list[str]) -> None:
    """Append the Geom2dGridEval requests; append_tkg2d_grid_requests(requests)."""
    rng = random.Random(20260929)
    append_grid_curve_requests(rng, requests)
    append_grid_adaptor_requests(rng, requests)
    append_grid_evalrep_requests(rng, requests)
    for parameters in (grid([0.0, 0.5, 1.0]), grid([]), grid([2.0], -4)):
        requests.append('g2_grid_null ' + fmt(parameters))
