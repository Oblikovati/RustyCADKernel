"""Deterministic requests for AppParCurves (tools/oracle/tkgeombase_probe_appparcurves.cxx
and tkgeombase_probe_appparcurves_fit.cxx).

The package functions run on parameters inside, at and outside [0, 1] (and the knot
range), every basis size down to 2 poles, and B-spline knots with interior
multiplicities up to C0. The containers run every constructor, valid and invalid
indexes (including the Dimension(0) and Pole(0, n) quirks), the transformations,
the Dump text and the pole sharing of copies; the B-spline curves every overload
around and outside their knots. The generics run through the AppDef instantiations
on 3D, 2D and mixed MultiLines (smooth, noisy, with collinear and repeated points),
every pair of extremity constraints each Perform overload accepts, tangents along
and against the chord, missing tangents and curvatures (the downgrades), too few
poles (not ready), inner pass/tangency/curvature constraints, the B-spline fits,
the parameter functions and both optimizers at loose and tight tolerances.
Paths that read uninitialized data natively (curvature constraints in
Perform(Parameters), an unperformed fit, the constrained B-spline function) are
left out.
"""
from __future__ import annotations
import math
import random
from tkgeombase_corpus_gce import add

NO, PASS, TANGENT, CURVATURE = 0, 1, 2, 3


def uvec(lower, values):
    return [lower, len(values), *values]


def bernstein_requests(rng, requests):
    for nbpoles in [2, 3, 4, 6, 10, 25]:
        for lower, u in [(1, [0.0, 0.25, 0.5, 1.0]), (3, [-0.5, 1.5, 0.999, 1e-9]),
                         (1, [rng.random() for _ in range(5)])]:
            for dda in sorted({2, 3, 4, nbpoles}):
                add(requests, 'apc_bernstein', nbpoles, *uvec(lower, u), dda)


def flat(knots, mults):
    return [k for k, m in zip(knots, mults) for _ in range(m)]


def spline_requests(rng, requests):
    cases = [(1, [0.0, 1.0], [2, 2]), (2, [0.0, 0.5, 1.0], [3, 1, 3]), (3, [0.0, 0.3, 0.7, 1.0], [4, 1, 2, 4]),
             (3, [0.0, 0.5, 1.0], [4, 3, 4]), (4, [0.0, 1.0, 2.0, 3.0], [5, 1, 1, 5]), (5, [0.0, 1.0], [6, 6]),
             (2, [-1.0, 0.0, 2.0, 5.0], [3, 2, 1, 3])]
    for deg, knots, mults in cases:
        nbpoles = sum(mults) - deg - 1
        a, b = knots[0], knots[-1]
        inside = sorted(rng.uniform(a, b) for _ in range(6))
        for lower, params in [(1, [a, *knots[1:-1], b]), (2, inside), (1, list(reversed(inside))),
                              (1, [a - 0.1 * (b - a), b + 0.2 * (b - a)])]:
            add(requests, 'apc_spline', nbpoles, deg, *uvec(lower, params), *uvec(1, flat(knots, mults)))


def point_spec(rng, kind, nb3d, nb2d):
    values = [kind, nb3d, nb2d]
    for _ in range(nb3d):
        values += [rng.uniform(-5, 5) for _ in range(3)]
    for _ in range(nb2d):
        values += [rng.uniform(-5, 5) for _ in range(2)]
    return values


def multipoint_requests(rng, requests):
    for kind in [0, 1, 2, 3]:
        for nb3d, nb2d in [(1, 0), (0, 1), (2, 2), (1, 3), (0, 0)]:
            if (kind == 1 and nb2d) or (kind == 2 and nb3d):
                continue
            total = nb3d + nb2d
            c = []
            for i in [-1, 0, 1, total, total + 1]:
                c += [5, i, 2, i, 4, i]
            c += [1, 1, 1.0, 2.0, 3.0, 3, nb3d + 1, 4.0, 5.0, 1, 0, 0.0, 0.0, 0.0, 3, nb3d, 1.0, 1.0]
            c += [6, 1, 1.0, 2.0, -1.0, 0.5, 0.0, 3.0, 6, total, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 6, 0, 0, 1, 0, 1, 0, 1]
            c += [7, total, 1.0, -1.0, 2.0, 0.5, 7, 1, 0.0, 1.0, 0.0, 1.0, 8, 9, 1, 7.0, 8.0, 9.0, 2, 1, 8]
            count = len([x for x in commands(c, {1: 4, 2: 1, 3: 3, 4: 1, 5: 1, 6: 7, 7: 5, 8: 0, 9: 4})])
            add(requests, 'apc_multipoint', *point_spec(rng, kind, nb3d, nb2d), count, *c)


def commands(flat_list, sizes):
    """The command codes of a flat list, given the argument count of each code."""
    i = 0
    while i < len(flat_list):
        code = flat_list[i]
        yield code
        i += 1 + sizes[code]


CURVE_SIZES = {1: 2, 2: 2, 3: 2, 4: 1, 5: 2, 6: 3, 7: 7, 8: 5, 9: 0, 10: 0, 12: 1}


def curve_count(c):
    count, i = 0, 0
    while i < len(c):
        code = c[i]
        if code == 11:
            nb3d, nb2d = c[i + 3], c[i + 4]
            i += 2 + 3 + 3 * nb3d + 2 * nb2d
        else:
            i += 1 + CURVE_SIZES[code]
        count += 1
    return count


def curve_commands(nb3d, nb2d, params, nbpoles):
    c = []
    for k in range(1, nb3d + nb2d + 1):
        for u in params:
            c += [1, k, u, 2, k, u, 3, k, u]
        c += [4, k, 5, k, 3 if k <= nb3d else 2, 6, k, 2, 3 if k <= nb3d else 2]
    total = nb3d + nb2d
    c += [4, 0, 4, total + 1, 5, 0, 3, 6, 0, 2, 3, 6, 0, 0, 3, 6, 1, nbpoles + 1, 3]
    if nb3d:
        c += [7, 1, 1.0, 2.0, 0.0, 1.0, -1.0, 0.5, 1, 1, params[1]]
    if nb2d:
        c += [8, nb3d + 1, 0.5, 1.0, 1.0, -1.0, 1, nb3d + 1, params[1]]
    c += [8, 1, 0.0, 1.0, 0.0, 1.0, 9, 10]
    return c


def multicurve_requests(rng, requests):
    # A curve of empty poles, then poles set one by one.
    c = [10, 4, 1, 9, 11, 1, 0, 1, 0, 1.0, 2.0, 3.0, 4, 1, 1, 1, 0.5, 12, 4, 10, 9]
    add(requests, 'apc_multicurve', 0, 3, curve_count(c), *c)
    for nb3d, nb2d in [(1, 0), (0, 1), (2, 1)]:
        for nbpoles in [2, 4, 7]:
            poles = [nbpoles]
            for _ in range(nbpoles):
                poles += point_spec(rng, 3 if nb3d and nb2d else (1 if nb3d else 2), nb3d, nb2d)
            c = curve_commands(nb3d, nb2d, [0.0, 0.3, 1.0, 1.4, -0.2], nbpoles)
            add(requests, 'apc_multicurve', 1, *poles, curve_count(c), *c)
            for deg, knots, mults in [(nbpoles - 1, [0.0, 1.0], [nbpoles, nbpoles]),
                                      (1, [0.0] + [i / (nbpoles - 1) for i in range(1, nbpoles)], [2] + [1] * (nbpoles - 2) + [2]),
                                      (2, [0.0, 0.5, 2.0], [3, nbpoles - 3, 3])]:
                if deg < 1 or sum(mults) - nbpoles - 1 != deg or min(mults) < 1:
                    continue
                c = curve_commands(nb3d, nb2d, [knots[0], knots[-1], (knots[0] + knots[-1]) / 3, knots[-1] + 0.5, knots[0] - 0.25], nbpoles)
                c += [12, nbpoles + 1, 10]
                for kind in [2, 3]:
                    add(requests, 'apc_multicurve', kind, *poles, *uvec(1, knots), len(mults), *mults, curve_count(c), *c)


# --- MultiLines ---

def line_points(rng, n, noise, shape):
    """Parameters t in [0, 1] and the points, first and second derivatives of a
    3D and a 2D curve (shape 0 smooth, 1 collinear, 2 with a repeated point)."""
    result = []
    for i in range(n):
        t = i / (n - 1)
        if shape == 1:
            p3, d3, s3 = [1 + 2 * t, -t, 0.5 * t], [2.0, -1.0, 0.5], [0.0, 0.0, 0.0]
            p2, d2, s2 = [3 * t, 1 - t], [3.0, -1.0], [0.0, 0.0]
        else:
            a = 2.5 * t
            p3 = [math.cos(a), math.sin(a), 0.4 * t * t]
            d3 = [-2.5 * math.sin(a), 2.5 * math.cos(a), 0.8 * t]
            s3 = [-6.25 * math.cos(a), -6.25 * math.sin(a), 0.8]
            p2 = [t, t * t * t - t]
            d2 = [1.0, 3 * t * t - 1]
            s2 = [0.0, 6 * t]
        p3 = [c + rng.uniform(-noise, noise) for c in p3]
        p2 = [c + rng.uniform(-noise, noise) for c in p2]
        result.append((t, p3, d3, s3, p2, d2, s2))
    if shape == 2:
        result[n // 2] = result[n // 2 - 1]
    return result


def line_spec(rng, nb3d, nb2d, n, tangent_points=(), curvature_points=(), noise=0.01, shape=0, reversed_tangents=False):
    pts = line_points(rng, n, noise, shape)
    spec = [nb3d, nb2d, n]
    for i, (t, p3, d3, s3, p2, d2, s2) in enumerate(pts, start=1):
        tan, curv = i in tangent_points, i in curvature_points
        spec += [1 if tan else 0, 1 if curv else 0]
        for k in range(nb3d):
            spec += [p3[0] + k, p3[1], p3[2] - k]
        for k in range(nb2d):
            spec += [p2[0], p2[1] + k]
        sign = -1.0 if reversed_tangents and i == 1 else 1.0
        if tan:
            for _ in range(nb3d):
                spec += [sign * c for c in d3]
            for _ in range(nb2d):
                spec += [sign * c for c in d2]
        if curv:
            for _ in range(nb3d):
                spec += s3
            for _ in range(nb2d):
                spec += s2
    return spec, [t for t, *_ in pts]


def chord_params(ts, noise_rng=None):
    if noise_rng is None:
        return ts
    return [ts[0]] + [t + noise_rng.uniform(-0.01, 0.01) for t in ts[1:-1]] + [ts[-1]]


LINES = [(1, 0), (0, 1), (1, 1), (2, 1)]


def ls_request(requests, line, fp, lp, fc, lc, nbpol, mode, params, variant=0, vectors=(), lambdas_in=(0.0, 0.0),
               lambdas=0, knots=None, mults=None):
    values = [*line, fp, lp, fc, lc, nbpol, mode]
    if mode >= 2:
        values += [len(knots), *knots, len(mults), *mults]
    values += uvec(fp, params[fp - 1:lp])
    values.append(variant)
    for v in vectors:
        values += uvec(1, v)
    if variant > 0:
        values += list(lambdas_in)
    add(requests, 'apc_ls', *values, lambdas)


def ls_requests(rng, requests):
    for nb3d, nb2d in LINES:
        cols = 3 * nb3d + 2 * nb2d
        for n, shape in [(9, 0), (12, 2), (6, 1)]:
            line, ts = line_spec(rng, nb3d, nb2d, n, tangent_points=(1, n, 3), curvature_points=(1, n), shape=shape)
            bare, _ = line_spec(rng, nb3d, nb2d, n, shape=shape)
            reversed_line, _ = line_spec(rng, nb3d, nb2d, n, tangent_points=(1, n), reversed_tangents=True)
            params = chord_params(ts, rng)
            for fc in [NO, PASS, TANGENT]:
                for lc in [NO, PASS, TANGENT]:
                    lam = (1 if fc == TANGENT else 0) | (2 if lc == TANGENT else 0)
                    for nbpol in [2, 3, 4, 6, 8]:
                        if nbpol > n:
                            continue
                        ls_request(requests, line, 1, n, fc, lc, nbpol, 0, params, lambdas=lam)
                    # The unconstrained Householder fit needs no more poles than
                    # points (math_Householder reads past Q otherwise).
                    if fc != NO or lc != NO or n - 2 >= 5:
                        ls_request(requests, line, 2, n - 1, fc, lc, 5, 0, params)
                    ls_request(requests, bare, 1, n, fc, lc, 5, 0, params)
                    ls_request(requests, reversed_line, 1, n, fc, lc, 5, 1, params, lambdas=lam)
            ls_bspline_requests(rng, requests, line, n, params)
            for variant, fc, lc in [(1, TANGENT, TANGENT), (1, PASS, TANGENT), (2, PASS, PASS), (2, TANGENT, TANGENT),
                                    (3, PASS, PASS), (3, CURVATURE, CURVATURE), (1, CURVATURE, CURVATURE)]:
                vectors = [[rng.uniform(-1, 1) for _ in range(cols)] for _ in range({2: 2, 3: 4}.get(variant, 0))]
                for nbpol in [4, 5, 7, 8]:
                    if nbpol > n:
                        continue
                    ls_request(requests, line, 1, n, fc, lc, nbpol, 1, params, variant, vectors, (0.3, 0.25), lambdas=3)


def ls_bspline_requests(rng, requests, line, n, params):
    for deg, knots, mults in [(3, [0.0, 0.5, 1.0], [4, 1, 4]), (2, [0.0, 0.3, 0.6, 1.0], [3, 1, 1, 3]),
                              (3, [0.0, 0.4, 1.0], [4, 2, 4]), (1, [0.0, 0.5, 1.0], [2, 1, 2])]:
        nbpol = sum(mults) - deg - 1
        if nbpol > n:
            continue
        for fc, lc in [(NO, NO), (PASS, PASS), (PASS, NO), (TANGENT, TANGENT), (NO, TANGENT)]:
            lam = (1 if fc == TANGENT else 0) | (2 if lc == TANGENT else 0)
            ls_request(requests, line, 1, n, fc, lc, nbpol, 2, params, lambdas=lam, knots=knots, mults=mults)
            ls_request(requests, line, 1, n, fc, lc, nbpol, 3, params, 1, (), (0.2, 0.4), lambdas=3 if lam == 3 else 0,
                       knots=knots, mults=mults)


def constraint_list(pairs):
    return [len(pairs), *[v for pair in pairs for v in pair]]


def resol_requests(rng, requests):
    for nb3d, nb2d in LINES + [(2, 0), (0, 2)]:
        n = 10
        line, ts = line_spec(rng, nb3d, nb2d, n, tangent_points=(1, 4, 7, n), curvature_points=(5,))
        params = chord_params(ts, rng)
        for pairs, fc, lc in [([(1, PASS), (4, PASS), (n, PASS)], PASS, PASS), ([(1, NO), (4, TANGENT), (n, NO)], NO, NO),
                              ([(1, PASS), (4, PASS), (7, TANGENT), (n, TANGENT)], PASS, TANGENT),
                              ([(4, PASS)], NO, NO), ([(1, TANGENT), (5, CURVATURE), (n, PASS)], TANGENT, PASS),
                              ([(2, PASS), (3, PASS), (8, PASS)], NO, NO)]:
            ccol = 3 * nb3d + 2 * nb2d
            passes = sum(1 for _, c in pairs if c >= PASS)
            rows = ccol * passes + sum(1 for _, c in pairs if c >= TANGENT) * (ccol - 1) + 3 * sum(1 for _, c in pairs if c == CURVATURE)
            for nbpol in [5, 7]:
                # More constraint rows than pole coordinates make C tC singular; the
                # native Uzawa then reads past its factorization (a crash).
                if rows >= ccol * nbpol:
                    continue
                add(requests, 'apc_resol', *line, 1, n, fc, lc, *constraint_list(pairs), nbpol, *uvec(1, params), 1e-10)


def function_calls(params, rng, bspline=False):
    moved = [params[0]] + [u + rng.uniform(-0.02, 0.02) for u in params[1:-1]] + [params[-1]]
    calls = [1, *uvec(1, params), 6, 3, *uvec(1, moved), 6, 2, *uvec(1, params), 4, 5, 7, 1, *uvec(1, moved), 5]
    if bspline:
        calls += [8, 9, 10]
    return calls


def call_count(calls):
    count, i = 0, 0
    while i < len(calls):
        code = calls[i]
        i += 1 + (2 + calls[i + 2] if code <= 3 else 0)
        count += 1
    return count


def function_requests(rng, requests):
    for nb3d, nb2d in LINES + [(0, 2)]:
        n = 9
        line, ts = line_spec(rng, nb3d, nb2d, n, tangent_points=(1, 3, 6, n))
        params = chord_params(ts, rng)
        for pairs in [[(1, PASS), (n, PASS)], [(1, PASS), (3, PASS), (n, PASS)], [(1, NO), (6, TANGENT), (n, PASS)],
                      [(3, PASS)], [(1, TANGENT), (n, TANGENT)]]:
            # A tangency at both ends needs 5 poles (Degree 4) to leave an unknown.
            for deg in ([4, 5] if pairs[0][1] == TANGENT else [3, 5]):
                calls = function_calls(params, rng)
                add(requests, 'apc_function', *line, 1, n, *constraint_list(pairs), *uvec(1, params), deg,
                    call_count(calls), *calls)


def bspfunction_requests(rng, requests):
    for nb3d, nb2d in LINES:
        n = 11
        line, ts = line_spec(rng, nb3d, nb2d, n, tangent_points=(1, n))
        params = chord_params(ts, rng)
        for pairs, l1, l2 in [([(1, PASS), (n, PASS)], 0.0, 0.0), ([(1, TANGENT), (n, TANGENT)], 0.2, 0.3),
                              ([(1, NO), (n, NO)], 0.0, 0.0), ([(1, TANGENT), (n, PASS)], 0.25, 0.0)]:
            for deg, knots, mults in [(3, [0.0, 0.5, 1.0], [4, 1, 4]), (2, [0.0, 0.3, 0.6, 1.0], [3, 1, 1, 3])]:
                nbpol = sum(mults) - deg - 1
                calls = function_calls(params, rng, True)
                add(requests, 'apc_bspfunction', *line, 1, n, *constraint_list(pairs), *uvec(1, params),
                    len(knots), *knots, len(mults), *mults, nbpol, l1, l2, call_count(calls), *calls)


def gradient_requests(rng, requests):
    for nb3d, nb2d in LINES + [(0, 2)]:
        n = 10
        line, ts = line_spec(rng, nb3d, nb2d, n, tangent_points=(1, 4, n), noise=0.02)
        params = chord_params(ts, rng)
        for pairs in [[(1, PASS), (4, PASS), (n, PASS)], [(1, PASS), (n, PASS)], [(1, PASS), (4, TANGENT), (n, PASS)]]:
            for deg, t3, t2, iterations in [(4, 1.0, 1.0, 5), (4, 1e-6, 1e-6, 0), (5, 1e-6, 1e-6, 5), (3, 1e-3, 1e-3, 200)]:
                add(requests, 'apc_gradient', *line, 1, n, *constraint_list(pairs), *uvec(1, params), deg, t3, t2, iterations)


def bspgradient_requests(rng, requests):
    for nb3d, nb2d in LINES:
        n = 12
        line, ts = line_spec(rng, nb3d, nb2d, n, tangent_points=(1, n), noise=0.02)
        params = chord_params(ts, rng)
        for pairs in [[(1, PASS), (n, PASS)], [(1, TANGENT), (n, TANGENT)], [(1, NO), (n, TANGENT)]]:
            for deg, knots, mults in [(3, [0.0, 0.5, 1.0], [4, 1, 4]), (3, [0.0, 0.3, 0.6, 1.0], [4, 1, 1, 4])]:
                for t3, t2, iterations, with_lambdas in [(1.0, 1.0, 1, 0), (1e-6, 1e-6, 1, 0), (1e-6, 1e-6, 5, 1), (1e-6, 1e-6, 0, 1)]:
                    add(requests, 'apc_bspgradient', *line, 1, n, *constraint_list(pairs), *uvec(1, params),
                        len(knots), *knots, len(mults), *mults, deg, t3, t2, iterations, with_lambdas, 0.3, 0.2)


def append_appparcurves_requests(requests: list[str]) -> None:
    """Append the AppParCurves requests in a fixed order."""
    rng = random.Random(19951109)
    bernstein_requests(rng, requests)
    spline_requests(rng, requests)
    multipoint_requests(rng, requests)
    multicurve_requests(rng, requests)
    ls_requests(rng, requests)
    resol_requests(rng, requests)
    function_requests(rng, requests)
    bspfunction_requests(rng, requests)
    gradient_requests(rng, requests)
    bspgradient_requests(rng, requests)
