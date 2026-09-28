"""BSplSLib cache, pole motion, resolution and interpolation requests.

Usage: for family in cache_families(): requests += family(rng)
(see tkmath_bsplslib_corpus.py for the shared domain rules).
"""
from __future__ import annotations
from collections.abc import Callable
import random
from tkmath_bsplclib_curves import Curve, array
from tkmath_bsplslib_surfaces import (encode_surface, flat_knots, grid, parameter, point_grid, surface,
                                      weight_grid)

Request = tuple[str, list]


def flat_span(rng: random.Random, direction: Curve) -> int:
    """0 (locate) or the flat index of a valid span; flat_span(rng, c) >= 0."""
    if rng.random() < 0.4:
        return 0
    count = len(flat_knots(direction))
    return rng.randint(direction.degree+1, count-direction.degree-1)


def cache_requests(rng: random.Random) -> list[Request]:
    """Static BuildCache (points and matrix), CacheD0..D2 and BSplSLib_Cache."""
    requests = []
    for case in range(384):
        rational = case % 2 == 0
        definition = surface(rng, rng.randint(1, 6) if case % 11 else 9, rng.randint(1, 6), case % 4 == 0,
                             case % 5 == 0, rational, True, rng.choice(['random', 'uniform']))
        u, v = definition.u, definition.v
        spans = [flat_span(rng, u), flat_span(rng, v)]
        domains = [rng.choice([1.0, 0.5, 2.0, 1e-3]) for _ in range(2)]
        caches = [parameter(rng, u.knots), parameter(rng, v.knots), rng.choice([1.0, 0.5, 2.0]),
                  rng.choice([1.0, 0.25, 3.0])]
        points: list = []
        for _ in range(3):
            points += [parameter(rng, u.knots, u.periodic), parameter(rng, v.knots, v.periodic),
                       rng.uniform(-1, 1), rng.uniform(-1, 1)]
        requests.append(('bsps_cache', encode_surface(definition, True)
                         + [parameter(rng, u.knots), parameter(rng, v.knots)]+domains+spans+caches+[3]+points))
    return requests


def bezier_requests(rng: random.Random) -> list[Request]:
    """PolesCoefficients (both overloads) and CoefsD0..D2 on Bezier patches."""
    requests = []
    for case in range(256):
        rows, cols = rng.randint(2, 8), rng.randint(2, 8)
        lowers = rng.choice([(1, 1), (0, 0), (2, 3)])
        weights = weight_grid(rng, rows, cols, case % 5 == 0)
        points: list = []
        for _ in range(3):
            points += [rng.uniform(-0.5, 1.5), rng.uniform(-0.5, 1.5)]
        requests.append(('bsps_bezier', grid(*lowers, point_grid(rng, rows, cols))+[int(case % 2 == 0)]
                         + grid(*lowers, weights)+[3]+points))
    return requests


def index_range(rng: random.Random, count: int) -> list[int]:
    """Ordered pole indices inside 1..count; index_range(rng, 4)[0] >= 1.

    Half the ranges cover every pole: a range missing the non-zero basis functions
    leaves no pole to move (Standard_OutOfRange on pole 0).
    """
    if rng.random() < 0.5:
        return [1, count]
    first = rng.randint(1, count)
    return [first, rng.randint(first, count)]


def move_requests(rng: random.Random) -> list[Request]:
    """MovePoint over pole ranges, rational or not."""
    requests = []
    for case in range(256):
        rational = case % 2 == 0
        definition = surface(rng, rng.randint(1, 5), rng.randint(1, 5), False, False, rational,
                             True, rng.choice(['random', 'uniform']))
        u, v = definition.u, definition.v
        displ = [rng.uniform(-3, 3) for _ in range(3)]
        requests.append(('bsps_move', [parameter(rng, u.knots, False), parameter(rng, v.knots, False)]+displ
                         + index_range(rng, definition.rows())+index_range(rng, definition.cols())
                         + [u.degree, v.degree, int(rational)]+grid(1, 1, definition.poles)
                         + grid(1, 1, definition.weights)+array(1, flat_knots(u))+array(1, flat_knots(v))))
    return requests


def resolution_requests(rng: random.Random) -> list[Request]:
    """Resolution with every rationality combination, periodic directions."""
    requests = []
    for case in range(256):
        definition = surface(rng, rng.randint(1, 6), rng.randint(1, 6), case % 4 == 0, case % 5 == 0,
                             case % 2 == 0, case % 7 != 3, rng.choice(['random', 'uniform', 'tiny']))
        if definition.has_weights and case % 3 == 0:
            definition.u_rational = False
        if definition.has_weights and case % 3 == 1:
            definition.v_rational = False
        requests.append(('bsps_resolution', encode_surface(definition)+[rng.choice([1e-7, 1e-3, 1.0])]))
    return requests


def schoenberg(direction: Curve) -> list[float]:
    """Knot averages of a clamped direction, one parameter per pole."""
    flat, degree = flat_knots(direction), direction.degree
    return [sum(flat[i+1:i+1+degree])/degree for i in range(direction.count())]


def interpolate_requests(rng: random.Random) -> list[Request]:
    """Interpolate (with and without weights) at the Schoenberg points."""
    requests = []
    for case in range(256):
        rational = case % 2 == 0
        definition = surface(rng, rng.randint(1, 6), rng.randint(1, 6), False, False, rational, True,
                             rng.choice(['random', 'uniform']))
        u, v = definition.u, definition.v
        u_params, v_params = schoenberg(u), schoenberg(v)
        if case % 9 == 0 and len(u_params) > 2:
            u_params[1] = u_params[0]  # singular matrix: InversionProblem
        requests.append(('bsps_interpolate', [u.degree, v.degree]+array(1, flat_knots(u))+array(1, flat_knots(v))
                         + array(1, u_params)+array(1, v_params)+[int(rational)]
                         + grid(1, 1, definition.poles)+grid(1, 1, definition.weights)))
    return requests


def raised(rng: random.Random, direction: Curve, step: int) -> list[float]:
    """Flat knots of the direction's breakpoints at degree + step."""
    degree = direction.degree+step
    mults = [degree+1]+[rng.randint(1, degree) for _ in direction.knots[1:-1]]+[degree+1]
    return [k for k, m in zip(direction.knots, mults) for _ in range(m)]


def function_requests(rng: random.Random) -> list[Request]:
    """FunctionMultiply by a bilinear function, with failing evaluations."""
    requests = []
    for case in range(256):
        definition = surface(rng, rng.randint(1, 4), rng.randint(1, 4), False, False, True, True,
                             rng.choice(['random', 'uniform']))
        definition.u_rational = definition.v_rational = case % 2 == 0
        u, v = definition.u, definition.v
        steps = [rng.randint(1, 3), rng.randint(1, 3)]
        coefficients = [rng.uniform(0.5, 2.0), rng.uniform(-0.2, 0.2), rng.uniform(-0.2, 0.2), rng.uniform(-0.1, 0.1)]
        limit = rng.uniform(u.knots[0], u.knots[-1]) if case % 7 == 0 else u.knots[-1]+10.0
        requests.append(('bsps_function', encode_surface(definition, case % 3 == 0)
                         + array(1, raised(rng, u, steps[0]))+array(1, raised(rng, v, steps[1]))
                         + [u.degree+steps[0], v.degree+steps[1]]+coefficients+[limit]))
    return requests


def cache_families() -> list[Callable[[random.Random], list[Request]]]:
    """The request builders of this module, in corpus order."""
    return [cache_requests, bezier_requests, move_requests, resolution_requests, interpolate_requests,
            function_requests]
