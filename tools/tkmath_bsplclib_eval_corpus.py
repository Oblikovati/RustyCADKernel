"""BSplCLib evaluation, solving and curve-shaping requests (see tkmath_bsplclib_corpus.py).

Usage: for family in eval_families(): requests += family(rng).
"""
from __future__ import annotations
import random
from tkmath_bsplclib_curves import Curve, array, curve, encode, flat_knots

Request = tuple[str, list]


def parameter(rng: random.Random, knots: list[float], outside: bool = True) -> float:
    """Inside, on a knot, next to a knot or (optionally) outside the knot range."""
    span = knots[-1]-knots[0]
    choices = [rng.uniform(knots[0], knots[-1]), rng.choice(knots), rng.choice(knots)*(1+1e-16)]
    if outside:
        choices.append(rng.uniform(knots[0]-span, knots[-1]+span))
    return rng.choice(choices)


def last_u_knot_index(degree: int, mults: list[int]) -> int:
    """BSplCLib::LastUKnotIndex for 1-based multiplicities; last_u_knot_index(3, [4, 4]) == 2."""
    index, sigma = len(mults), mults[-1]
    while sigma <= degree:
        index -= 1
        sigma += mults[index-1]
    return index


def eval_requests(rng: random.Random) -> list[Request]:
    """D0..DN for 1D/2D/3D, rational or not, flat or multiplicity knots, periodic."""
    requests = []
    for case in range(1024):
        kind = [1, 2, 3][case % 3]
        degree, periodic = rng.randint(1, 9 if case % 5 else 25), case % 4 == 0
        flat = case % 6 < 3
        # Flat knots of unclamped curves let the native span search run past the last
        # valid span through a run of equal knots (undefined reads), so they stay clamped.
        definition = curve(rng, degree, periodic, kind, 1 if kind == 1 else kind, case % 2 == 0,
                           flat or case % 7 != 3, rng.choice(['random', 'uniform', 'tiny', 'huge']))
        knots = flat_knots(definition) if flat else definition.knots
        index = rng.choice([0, rng.randint(1, len(knots))])
        # A hint on the last u-knot is kept natively and walks past the multiplicities.
        if not flat and not periodic and index == last_u_knot_index(degree, definition.mults):
            index = 0
        requests.append(('bspl_eval', encode(definition, knots)+[int(flat), parameter(rng, definition.knots),
                                                                   index, rng.randint(0, degree+2)]))
    return requests


def flat_eval_requests(rng: random.Random) -> list[Request]:
    """Eval from flat knots without entering the extrapolation branch."""
    requests = []
    for case in range(512):
        kind = [1, 2, 3][case % 3]
        degree = rng.randint(1, 8)
        definition = curve(rng, degree, False, kind, rng.randint(1, 4) if kind == 1 else kind,
                           case % 2 == 0, True, rng.choice(['random', 'uniform']))
        flat = flat_knots(definition)
        # The periodic flag wraps by FlatKnots(Upper-1) - FlatKnots(2) on these clamped knots.
        definition.periodic = case % 5 == 0
        request = rng.randint(0, degree+2)
        modes = [degree+rng.randint(0, 2), degree+rng.randint(0, 2)]
        requests.append(('bspl_flat_eval', encode(definition, flat)
                         + [parameter(rng, definition.knots), request]+modes+[int(case % 4 == 0)]))
    return requests


def basis_requests(rng: random.Random) -> list[Request]:
    """EvalBsplineBasis: every derivative order, short matrices, null spans, periodicity."""
    requests = []
    for case in range(512):
        degree = rng.randint(0, 10 if case % 9 else 25)
        definition = curve(rng, max(degree, 1), False, style=rng.choice(['random', 'uniform', 'tiny']))
        flat = flat_knots(definition)
        order = definition.degree+1
        request = rng.randint(0, order+1)
        u = parameter(rng, definition.knots)
        # Null spans: only a parameter above them keeps the native span search in range.
        if case % 17 == 0: flat, u = [flat[0]]*len(flat), flat[0]+1.0
        rows = min(request, order-1)+1+rng.choice([0, 0, 1, -1])
        cols = order+rng.choice([0, 0, 2, -1])
        lower = rng.choice([1, 1, 0])
        requests.append(('bspl_basis', [request, order]+array(lower, flat)
                         + [u, int(case % 5 == 0), max(rows, 1), max(cols, 1)]))
    return requests


def interpolate_requests(rng: random.Random) -> list[Request]:
    """Collocation matrices, banded LU, solves and interpolation at Schoenberg points."""
    requests = []
    for case in range(384):
        kind = [1, 2, 3][case % 3]
        degree = rng.randint(1, 7)
        definition = curve(rng, degree, False, kind, rng.randint(1, 4) if kind == 1 else kind, case % 2 == 0,
                           True, rng.choice(['random', 'uniform']))
        flat = flat_knots(definition)
        count = definition.count()
        params = [sum(flat[i+1:i+1+degree])/degree for i in range(count)]
        contacts = [0]*count
        if case % 4 == 1 and count > 2:
            params[1], contacts[1] = params[0], 1
        if case % 4 == 2 and count > 2:
            params[-2], contacts[-2] = params[-1], 1
        requests.append(('bspl_interpolate', encode(definition, flat)+array(1, params)+array(1, contacts)))
    return requests


def cache_requests(rng: random.Random) -> list[Request]:
    """BuildCache (point arrays and cache matrix), CacheD0..D3 and BSplCLib_Cache."""
    requests = []
    for case in range(384):
        kind = [2, 3][case % 2]
        degree, periodic = rng.randint(1, 9 if case % 7 else 25), case % 4 == 0
        definition = curve(rng, degree, periodic, kind, kind, case % 3 == 0, True, rng.choice(['random', 'uniform']))
        flat = flat_knots(definition)
        u = parameter(rng, definition.knots, outside=periodic)
        params = [u, parameter(rng, definition.knots, outside=periodic), rng.choice(definition.knots)]
        span = rng.randint(degree, len(flat)-degree)
        requests.append(('bspl_cache', encode(definition, flat)+[u, rng.choice([1.0, 0.5, 2.0, 1e-3]), span,
                                                                  len(params)]+params))
    return requests


def bezier_requests(rng: random.Random) -> list[Request]:
    """Bezier D0..D3, degree elevation, polynomial coefficients and flat knots."""
    requests = []
    for case in range(256):
        kind = [2, 3][case % 2]
        # Degree 0 is excluded: native D1 then reads an uninitialized derivative.
        degree = rng.randint(1, 8 if case % 5 else 25)
        rational = case % 3 == 0
        poles = [rng.uniform(-5, 5) for _ in range((degree+1)*kind)]
        weights = [rng.uniform(0.3, 3) for _ in range(degree+1)] if rational else []
        definition = Curve(kind, kind, rational, degree, False, [0.0, 1.0], [degree+1]*2, poles, weights)
        requests.append(('bspl_bezier', encode(definition)+[min(degree+rng.randint(0, 3), 25),
                                                           rng.choice([rng.uniform(0, 1), 0.0, 1.0, rng.uniform(-1, 2)])]))
    return requests


def move_requests(rng: random.Random) -> list[Request]:
    """MovePoint and MovePointAndTangent, including rejected conditions."""
    requests = []
    for case in range(384):
        kind = [2, 3][case % 2]
        degree = rng.randint(1, 6)
        definition = curve(rng, degree, False, kind, kind, case % 3 == 0, True, rng.choice(['random', 'uniform']))
        flat, count = flat_knots(definition), definition.count()
        u = parameter(rng, definition.knots, outside=case % 8 == 0)
        displ = [rng.uniform(-2, 2) for _ in range(9)]
        i1, i2 = sorted(rng.randint(1, count) for _ in range(2)) if case % 3 == 0 else (1, count)
        start, end = rng.randint(-1, degree+1), rng.randint(-1, degree+1)
        requests.append(('bspl_move', encode(definition, flat)+[u]+displ+[i1, i2, rng.choice([1e-9, 1e-3, 0.5]),
                                                                           start, end]))
    return requests


def resolution_requests(rng: random.Random) -> list[Request]:
    """Resolution for dimensions 1..5 and points, rational and periodic pole cycles."""
    requests = []
    for case in range(384):
        kind = [1, 2, 3][case % 3]
        degree, periodic = rng.randint(1, 8), case % 4 == 0
        definition = curve(rng, degree, periodic, kind, rng.randint(1, 5) if kind == 1 else kind, case % 2 == 0,
                           True, rng.choice(['random', 'uniform', 'tiny', 'huge']))
        if case % 11 == 0: definition.poles = [0.0]*len(definition.poles)
        requests.append(('bspl_resolution', encode(definition, flat_knots(definition))
                         + [definition.count(), rng.choice([1e-7, 1e-3, 1.0])]))
    return requests


def function_requests(rng: random.Random) -> list[Request]:
    """FunctionMultiply and FunctionReparameterise with an interior-mapping function."""
    requests = []
    for case in range(256):
        kind = [1, 2, 3][case % 3]
        degree = rng.randint(1, 5)
        definition = curve(rng, degree, False, kind, rng.randint(1, 3) if kind == 1 else kind, False, True, 'random')
        knots, mults = definition.knots, definition.mults
        reparam = case % 2 == 1
        new_degree = degree+(0 if reparam else 2)+rng.randint(0, 1)
        start, end = (knots[0], knots[-1]) if not reparam else (rng.uniform(-2, 0), rng.uniform(1, 3))
        new_knots = list(knots) if not reparam else [start+(k-knots[0])*(end-start)/(knots[-1]-knots[0]) for k in knots]
        new_mults = [new_degree+1]+[min(m+new_degree-degree, new_degree) for m in mults[1:-1]]+[new_degree+1]
        new_flat = [k for k, m in zip(new_knots, new_mults) for _ in range(m)]
        if reparam:
            length = knots[-1]-knots[0]
            c1 = 0.8*length/(end-start)
            c0 = knots[0]+0.1*length-c1*start
            coefficients = [c0, c1, rng.choice([0.0, 1e-3*length])]
        else:
            coefficients = [rng.uniform(0.5, 2), rng.uniform(-1, 1), rng.uniform(-1, 1)]
        limit = rng.choice([1e300, new_flat[len(new_flat)//2]]) if case % 5 == 0 else 1e300
        requests.append(('bspl_function', encode(definition, flat_knots(definition))+[new_degree]+array(1, new_flat)
                         + coefficients+[limit, int(reparam)]))
    return requests


def tang_requests(rng: random.Random) -> list[Request]:
    """TangExtendToConstraint before and after, continuity 0..3."""
    requests = []
    for case in range(256):
        continuity = case % 4
        degree = rng.randint(continuity+1, continuity+4)
        definition = curve(rng, degree, False, 1, rng.randint(1, 3), False, True, rng.choice(['random', 'uniform']))
        point = [rng.uniform(-15, 15) for _ in range(definition.dim)]
        requests.append(('bspl_tang', encode(definition, flat_knots(definition))+array(1, point)
                         + [continuity, int(case % 2 == 0), rng.choice([1.0, 0.5, 2.0, 1e-6])]))
    return requests


def merge_requests(rng: random.Random) -> list[Request]:
    """MergeBSplineKnots of two clamped knot vectors, including a degenerate range."""
    requests = []
    for case in range(192):
        first = curve(rng, rng.randint(1, 4), False, style='random')
        second = curve(rng, rng.randint(1, 4), False, style=rng.choice(['random', 'uniform']))
        start = rng.uniform(-2, 2)
        end = start+rng.choice([1.0, 3.0, 0.0, -1.0]) if case % 7 == 0 else start+rng.uniform(0.5, 4)
        tol = rng.choice([1e-9, 1e-3, 0.2])
        requests.append(('bspl_merge', [tol, start, end, first.degree]+array(1, first.knots)+array(1, first.mults)
                         + [second.degree]+array(1, second.knots)+array(1, second.mults)))
    return requests


def intervals_requests(rng: random.Random) -> list[Request]:
    """Intervals of continuity, periodic ranges spanning several periods."""
    requests = []
    for case in range(320):
        degree, periodic = rng.randint(1, 6), case % 3 == 0
        definition = curve(rng, degree, periodic, style=rng.choice(['random', 'uniform']))
        knots = definition.knots
        span = knots[-1]-knots[0]
        if periodic:
            first = rng.uniform(knots[0]-2*span, knots[-1]+span)
            last = first+rng.choice([rng.uniform(0, 3*span), span])
        else:
            first, last = sorted(rng.uniform(knots[0], knots[-1]) for _ in range(2))
            if case % 4 == 0: first, last = knots[0], knots[-1]
        requests.append(('bspl_intervals', [degree, int(periodic), rng.randint(0, degree), first, last,
                                            rng.choice([1e-9, 1e-3])]+array(1, knots)+array(1, definition.mults)))
    return requests


def eval_families() -> list:
    """The request families of this module, in corpus order."""
    return [eval_requests, flat_eval_requests, basis_requests, interpolate_requests, cache_requests,
            bezier_requests, move_requests, resolution_requests, function_requests, tang_requests,
            merge_requests, intervals_requests]
