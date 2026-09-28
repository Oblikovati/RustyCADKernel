"""Deterministic BSplSLib differential requests for tools/verify_tkmath_numeric.py.

Usage: append_bsplslib_requests(requests) after the BSplCLib requests.
Requests stay inside the native defined domain: pole and weight grids match the
knot vectors, rational calls always carry weights (the native code dereferences
them), span hints never select the last knot of a non-periodic direction, and
rational DN orders stay within the degrees so no uninitialized stack is read.
"""
from __future__ import annotations
import random
from tkmath_bsplclib_curves import array
from tkmath_bsplslib_cache_corpus import cache_families
from tkmath_bsplslib_surfaces import Surface, encode_surface, flat_knots, grid, parameter, surface

Request = tuple[str, list]


def line(op: str, values: list) -> str:
    """Format one request; line('bsps_eval', [3, 0])."""
    return op+' '+' '.join(map(str, values))


def span_hint(rng: random.Random, definition: Surface, flat: bool, u_direction: bool) -> int:
    """0 (locate) or a span index whose local knots and poles stay inside the arrays."""
    direction = definition.u if u_direction else definition.v
    clamped = direction.periodic or direction.mults[0] == direction.degree+1 == direction.mults[-1]
    if rng.random() < 0.6 or not clamped:
        return 0
    if flat:
        count = len(flat_knots(direction))
        return rng.randint(direction.degree+1, count-direction.degree-1)
    return rng.randint(1, len(direction.knots)-1)


def eval_surface(rng: random.Random, case: int) -> Surface:
    """A surface of any degree up to 9 (sometimes 25 or the refused 26), any flags."""
    u_degree = rng.randint(1, 6) if case % 23 else rng.choice([9, 25, 26])
    v_degree = rng.randint(1, 6) if case % 29 else rng.choice([9, 25])
    rational = case % 2 == 0
    definition = surface(rng, u_degree, v_degree, case % 4 == 0, case % 5 == 0, rational,
                         case % 7 != 3, rng.choice(['random', 'uniform', 'tiny', 'huge']))
    if rational and case % 6 == 0:
        definition.v_rational = False
    if not rational and case % 9 == 0:
        definition.has_weights = True
    definition.row_lower, definition.col_lower = rng.choice([(1, 1), (0, 0), (2, 1), (1, 3)])
    return definition


def eval_requests(rng: random.Random) -> list[Request]:
    """D0..D3, DN, HomogeneousD0/D1 with and without multiplicities and hints."""
    requests = []
    for case in range(768):
        definition = eval_surface(rng, case)
        # Flat knots of unclamped directions let the span search run past the last span.
        flat = case % 3 == 0 and case % 7 != 3
        rational = definition.u_rational or definition.v_rational
        u, v = definition.u, definition.v
        nu = rng.randint(0, u.degree if rational else u.degree+1)
        nv = rng.randint(0, v.degree if rational else v.degree+1)
        points = []
        for _ in range(3):
            points += [parameter(rng, u.knots), parameter(rng, v.knots)]
        hints = [span_hint(rng, definition, flat, True), span_hint(rng, definition, flat, False)]
        requests.append(('bsps_eval', encode_surface(definition, flat)+hints+[nu, nv, 3]+points))
    return requests


def rational_requests(rng: random.Random) -> list[Request]:
    """RationalDerivative on raw homogeneous derivative buffers, all or one order."""
    requests = []
    for case in range(256):
        u_degree, v_degree = rng.randint(0, 4), rng.randint(0, 4)
        n, m = rng.randint(0, 5), rng.randint(0, 5)
        values = [rng.uniform(-5, 5) for _ in range((u_degree+1)*(v_degree+1)*4)]
        values[3] = rng.choice([1.0, rng.uniform(0.3, 3.0), -rng.uniform(0.3, 3.0)])
        requests.append(('bsps_rational', [u_degree, v_degree, n, m, int(case % 3 != 0), len(values)]+values))
    return requests


def iso_requests(rng: random.Random) -> list[Request]:
    """Iso in both directions, IsRational over sub-ranges, Reverse of both grids."""
    requests = []
    for case in range(384):
        definition = surface(rng, rng.randint(1, 6), rng.randint(1, 6), case % 4 == 0, case % 5 == 0,
                             case % 2 == 0, case % 7 != 3)
        definition.row_lower, definition.col_lower = rng.choice([(1, 1), (0, 2), (3, 1)])
        is_u = case % 3 != 0
        direction = definition.u if is_u else definition.v
        count = definition.cols() if is_u else definition.rows()
        pairs: list = []
        rl, cl = definition.row_lower, definition.col_lower
        for _ in range(3):
            i1 = rng.randint(rl, rl+definition.rows()-1)
            j1 = rng.randint(cl, cl+definition.cols()-1)
            pairs += [i1, rng.randint(i1, rl+definition.rows()-1), j1, rng.randint(j1, cl+definition.cols()-1),
                      rng.choice([0.0, 1e-9, 0.5])]
        last = rng.randint(rl if is_u else cl, (rl if is_u else cl)+2*count)
        requests.append(('bsps_iso', encode_surface(definition)+[parameter(rng, direction.knots), int(is_u),
                                                                int(definition.has_weights),
                                                                rng.choice([1, 0, 2]), count, 3]
                         + pairs+[last, int(case % 2 == 1)]))
    return requests


def poles_requests(rng: random.Random) -> list[Request]:
    """SetPoles/GetPoles with and without weights along both directions."""
    requests = []
    for case in range(256):
        rows, cols = rng.randint(1, 7), rng.randint(1, 7)
        lowers = rng.choice([(1, 1), (0, 0), (2, 5)])
        poles = [[[rng.uniform(-10, 10) for _ in range(3)] for _ in range(cols)] for _ in range(rows)]
        weights = [[rng.uniform(0.3, 3.0) for _ in range(cols)] for _ in range(rows)]
        flat = [rng.uniform(-10, 10) for _ in range(4*rows*cols)]
        requests.append(('bsps_poles', grid(*lowers, poles)+grid(*lowers, weights)
                         + [case % 2, rng.choice([1, 0, 3])]+array(rng.choice([1, 0, 3]), flat)))
    return requests


def insert_arguments(rng: random.Random, case: int, definition: Surface, u_direction: bool) -> list:
    """AddKnots, optional AddMults, Epsilon and Add of one direction."""
    direction = definition.u if u_direction else definition.v
    knots, degree = direction.knots, direction.degree
    count = rng.randint(1, 3)
    add = sorted(rng.uniform(knots[0], knots[-1]) for _ in range(count))
    if case % 3 == 0: add[rng.randrange(count)] = rng.choice(knots[1:-1] or knots)
    add_mults = [rng.randint(0, degree+1) for _ in add]
    return (array(1, add)+[int(case % 5 != 0)]+array(1, add_mults)
            + [rng.choice([0.0, 1e-9, 1e-3]), int(case % 6 != 0)])


def remove_arguments(rng: random.Random, case: int, definition: Surface, u_direction: bool) -> list:
    """Index, Mult, Tolerance and the resulting pole and knot counts."""
    direction = definition.u if u_direction else definition.v
    knots, mults = direction.knots, direction.mults
    count = definition.rows() if u_direction else definition.cols()
    index = rng.randint(1, len(knots))
    mult = rng.randint(0, mults[index-1]-1)
    # An empty output grid (a whole end knot removed from a Bezier) is not a probe input.
    while count-(mults[index-1]-mult) < 1:
        mult += 1
    new_knots = len(knots)-1 if mult == 0 else len(knots)
    tol = rng.choice([1e-9, 1e-3, 1.0, 1e3, 1e6])
    return [index, mult, tol, count-(mults[index-1]-mult), new_knots]


def degree_arguments(rng: random.Random, definition: Surface, u_direction: bool) -> list:
    """NewDegree and the elevated pole count of one direction."""
    direction = definition.u if u_direction else definition.v
    step = rng.randint(0, 3)
    new_mults = [m+step for m in direction.mults]
    if not direction.periodic:
        new_mults[0] = new_mults[-1] = direction.degree+step+1
    poles = sum(new_mults[:-1]) if direction.periodic else sum(new_mults)-(direction.degree+step)-1
    return [direction.degree+step, poles]


def ops_requests(rng: random.Random) -> list[Request]:
    """InsertKnots, RemoveKnot, IncreaseDegree and Unperiodize along U or V."""
    requests = []
    for case in range(512):
        periodic = case % 4 == 0
        definition = surface(rng, rng.randint(1, 5), rng.randint(1, 5), periodic, periodic, case % 2 == 0,
                             True, rng.choice(['random', 'uniform', 'tiny']))
        u_direction = case % 3 != 1
        requests.append(('bsps_ops', encode_surface(definition)+[int(u_direction)]
                         + insert_arguments(rng, case, definition, u_direction)
                         + remove_arguments(rng, case, definition, u_direction)
                         + degree_arguments(rng, definition, u_direction)))
    return requests


def append_bsplslib_requests(requests: list[str]) -> None:
    """Extend the corpus with seeded BSplSLib requests; append_bsplslib_requests(requests)."""
    rng = random.Random(19950214)
    for family in [eval_requests, rational_requests, iso_requests, poles_requests, ops_requests]+cache_families():
        requests.extend(line(op, values) for op, values in family(rng))
