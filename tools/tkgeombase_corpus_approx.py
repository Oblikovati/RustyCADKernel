"""Deterministic requests for the Approx multi-line drivers
(tools/oracle/tkgeombase_probe_approx_line.cxx and tkgeombase_probe_approx.cxx).

Approx_ComputeLine runs through the probe instantiation over AppDef_MultiLine whose
line tool scripts WhatStatus (never adds points as AppDef does, adds them under a
point cap, refuses to approximate, or cycles through the three), MakeMLBetween
(resampling the source curves, or the same line) and MakeMLOneMorePoint (a midpoint
before the loop point): 3D, 2D and mixed lines, smooth, noisy, collinear, with a
repeated point, sampled on nodal cubics that loop, with and without end tangents,
every constructor, every parametrization, cut and uncut, least squares and
gradients, given and mismatched initial parameters, and a second Perform.
AppDef_BSplineCompute runs every constructor, Interpol (2 to many points,
periodic), user knots with and without multiplicities, continuities and every end
constraint. Approx_FitAndDivide and FitAndDivide2d fit the source curves with and
without cutting, inverse degree order, hang checking, a segment cap, failing end
derivatives and a periodic coordinate. Approx_MCurvesToBSpCurve joins 1 to 4
Bezier MultiCurves of 3D, 2D and mixed curves with mixed degrees (C0 and C1
junctions), and an empty sequence.
"""
from __future__ import annotations
import random
from tkgeombase_corpus_appparcurves import line_spec, uvec
from tkgeombase_corpus_gce import add

NO, PASS, TANGENT, CURVATURE = 0, 1, 2, 3
CHORD, CENTRIPETAL, ISO = 0, 1, 2


def explicit_line(rng, nb3d, nb2d, n, tangents=(), noise=0.01, shape=0):
    spec, ts = line_spec(rng, nb3d, nb2d, n, tangent_points=tangents, noise=noise, shape=shape)
    return [0, *spec], ts


def source_line(nb3d, nb2d, ts, tangents=()):
    spec = [1, nb3d, nb2d, len(ts)]
    for i, t in enumerate(ts, start=1):
        spec += [t, 1 if i in tangents else 0]
    return spec


def uniform(a, b, n):
    return [a + (b - a) * i / (n - 1) for i in range(n)]


def options(degmin=4, degmax=8, tol3d=1e-3, tol2d=1e-6, iterations=5, cutting=1, par=CHORD, squares=0):
    return [degmin, degmax, tol3d, tol2d, iterations, cutting, par, squares]


def compute(requests, line, rules, variant, opts, cons=(TANGENT, TANGENT), params=(1, []), repeat=0):
    add(requests, 'ax_compute', *line, *rules, variant, *opts, *cons, *uvec(*params), repeat)


def lines(rng):
    """(line spec, point count) of every shape the drivers run on."""
    result = []
    for nb3d, nb2d in [(1, 0), (0, 1), (1, 1), (2, 0), (0, 2), (2, 1)]:
        for n in [2, 3, 5, 9, 16, 30]:
            spec, _ = explicit_line(rng, nb3d, nb2d, n, tangents=(1, n) if n % 2 else ())
            result.append((spec, n))
    for shape in [1, 2]:
        spec, _ = explicit_line(rng, 1, 1, 12, tangents=(1, 12), shape=shape)
        result.append((spec, 12))
    spec, _ = explicit_line(rng, 1, 0, 14, noise=0.3)
    result.append((spec, 14))
    for nb3d, nb2d in [(1, 0), (0, 1), (1, 1)]:
        for a, b, n in [(-1.4, 1.4, 25), (-1.2, 0.3, 8), (0.1, 1.3, 6)]:
            result.append((source_line(nb3d, nb2d, uniform(a, b, n), tangents=(1, n)), n))
    return result


def compute_requests(rng, requests):
    configs = [options(), options(2, 5, 1e-2, 1e-3), options(1, 3, 1e-6, 1e-8, 0), options(3, 6, cutting=0),
               options(2, 6, 1e-4, 1e-5, 3, 1, CENTRIPETAL), options(2, 5, 1e-3, 1e-4, 5, 1, ISO, 1),
               options(4, 12, 1e-7, 1e-9, 2, 1, CHORD, 1), options(2, 4, 1e-1, 1e-1, 5, 0, CENTRIPETAL, 1)]
    for index, (line, n) in enumerate(lines(rng)):
        for rules in [(0, 0), (1, 40), (2, 0), (3, 30)]:
            opts = configs[(index + rules[0]) % len(configs)]
            compute(requests, line, rules, 0, opts)
        compute(requests, line, (0, 0), 1, configs[index % 3], params=(1, uniform(0.0, 1.0, n)))
        compute(requests, line, (1, 20), 2, configs[(index + 1) % len(configs)], cons=(PASS, PASS), repeat=1)
        compute(requests, line, (0, 0), 3, configs[(index + 2) % len(configs)], cons=(NO, TANGENT),
                params=(1, uniform(0.0, 1.0, n)))
    line, n = lines(rng)[3]
    compute(requests, line, (0, 0), 1, options(cutting=0), params=(3, uniform(0.0, 1.0, n)))
    compute(requests, line, (0, 0), 3, options(), params=(1, uniform(0.0, 1.0, n + 3)))
    compute(requests, line, (0, 0), 2, options(), cons=(PASS, NO))


def bspl(requests, line, rules, variant, opts, cons=(TANGENT, TANGENT), continuity=-2, periodic=0, knots=None,
         mults=None, params=(1, []), setparams=0):
    kind = 0 if knots is None else 1 if mults is None else 2
    extra = [] if knots is None else [len(knots), *knots]
    extra += [] if mults is None else [len(mults), *mults]
    add(requests, 'ax_bspl', *line, *rules, variant, *opts, *cons, continuity, periodic, kind, *extra,
        *uvec(*params), setparams)


def bspl_requests(rng, requests):
    configs = [options(3, 8), options(2, 5, 1e-2, 1e-3), options(3, 6, cutting=0), options(2, 4, 1e-5, 1e-7, 0),
               options(3, 5, 1e-3, 1e-4, 5, 1, CENTRIPETAL, 1), options(2, 3, 1e-4, 1e-6, 5, 1, ISO)]
    for index, (line, n) in enumerate(lines(rng)):
        opts = configs[index % len(configs)]
        bspl(requests, line, (0, 0), 0, opts)
        bspl(requests, line, (0, 0), 1, configs[(index + 1) % len(configs)], params=(1, uniform(0.0, 1.0, n)))
        bspl(requests, line, (0, 0), 2, configs[(index + 2) % len(configs)], cons=(PASS, TANGENT), continuity=1)
        bspl(requests, line, (0, 0), 4, opts, periodic=index % 2)
        bspl(requests, line, (0, 0), 2, configs[(index + 3) % len(configs)], knots=[0.0, 0.4, 1.0])
        # Given multiplicities fit one degree only (other degrees overrun the flat knots
        # natively), and without cutting (after a failed first fit, the native loop reads
        # uninitialized multiplicities).
        bspl(requests, line, (0, 0), 3, options(3, 3, cutting=0), cons=(NO, NO), knots=[0.0, 0.3, 0.6, 1.0], mults=[4, 2, 1, 4],
             params=(1, uniform(0.0, 1.0, n)))
        bspl(requests, line, (0, 0), 4, configs[1], params=(1, uniform(0.0, 1.0, n)), setparams=1)
    line, n = lines(rng)[4]
    bspl(requests, line, (0, 0), 2, options(3, 6), cons=(CURVATURE, CURVATURE))
    bspl(requests, line, (0, 0), 2, options(3, 3, cutting=0), knots=[0.0, 0.5, 1.0], mults=[4, 1, 4])


def fit(requests, cls, nb3d, nb2d, u, d1fail, pdim, period, variant, degmin, degmax, tol3d, tol2d, cutting, fc, lc,
        maxseg, inv, hang):
    add(requests, 'ax_fit', cls, nb3d, nb2d, *u, d1fail, pdim, period, variant, degmin, degmax, tol3d, tol2d,
        cutting, fc, lc, maxseg, inv, hang)


def fit_requests(requests):
    for cls, nb3d, nb2d in [(0, 1, 0), (0, 2, 1), (1, 0, 1), (1, 0, 2), (0, 1, 1)]:
        for u in [(-1.4, 1.4), (0.0, 0.5), (-0.2, 0.2)]:
            fit(requests, cls, nb3d, nb2d, u, 0, 0, 0.0, 0, 3, 8, 1e-5, 1e-5, 0, TANGENT, TANGENT, 0, 1, 1)
            fit(requests, cls, nb3d, nb2d, u, 0, 0, 0.0, 0, 2, 6, 1e-6, 1e-6, 1, PASS, PASS, 0, 1, 1)
            fit(requests, cls, nb3d, nb2d, u, 1, 0, 0.0, 1, 3, 10, 1e-7, 1e-7, 1, TANGENT, TANGENT, 0, 1, 1)
            fit(requests, cls, nb3d, nb2d, u, 0, 1, 0.5, 1, 2, 5, 1e-4, 1e-4, 1, NO, TANGENT, 3, 0, 1)
            fit(requests, cls, nb3d, nb2d, u, 0, 0, 0.0, 1, 3, 12, 1e-9, 1e-9, 1, TANGENT, NO, 0, 1, 0)
            fit(requests, cls, nb3d, nb2d, u, 0, 0, 0.0, 1, 4, 9, 1e-8, 1e-8, 1, PASS, TANGENT, 5, 0, 0)


def chain(rng, nb3d, nb2d, degrees, c1):
    """Bezier MultiCurves as gb_apc_read_points (`n` then `kind nb3d nb2d coords` per
    pole), each starting at the end of the previous one; with `c1` the junction
    tangents are aligned too."""
    kind = 1 if nb2d == 0 else 2 if nb3d == 0 else 3
    width = 3 * nb3d + 2 * nb2d
    curves, previous = [], None
    for degree in degrees:
        poles = [[rng.uniform(-3, 3) for _ in range(width)] for _ in range(degree + 1)]
        if previous is not None:
            poles[0] = list(previous[-1])
            if c1 and degree > 1:
                poles[1] = [2 * a - b for a, b in zip(previous[-1], previous[-2])]
        previous = poles
        spec = [len(poles)]
        for pole in poles:
            spec += [kind, nb3d, nb2d, *pole]
        curves.append(spec)
    return curves


def mcurves_requests(rng, requests):
    for nb3d, nb2d in [(1, 0), (0, 1), (2, 1), (1, 2), (0, 3)]:
        for degrees in [[3], [3, 3], [2, 4, 3], [5, 5, 5, 2]]:
            for c1 in [False, True]:
                curves = chain(rng, nb3d, nb2d, degrees, c1)
                for variant in [0, 1, 2]:
                    add(requests, 'ax_mcurves', len(curves), *[v for c in curves for v in c], variant)
    add(requests, 'ax_mcurves', 0, 0)
    add(requests, 'ax_mcurves', 0, 1)


def append_approx_requests(requests: list[str]) -> None:
    """Append the Approx driver requests in a fixed order."""
    rng = random.Random(20260929)
    compute_requests(rng, requests)
    bspl_requests(rng, requests)
    fit_requests(requests)
    mcurves_requests(rng, requests)
