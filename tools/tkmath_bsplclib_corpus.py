"""Deterministic BSplCLib differential requests for tools/verify_tkmath_numeric.py.

Usage: append_bsplclib_requests(requests) after the other TKMath requests.
Requests stay inside the native defined domain: arrays and raw buffers are sized
as the C++ code addresses them, knots and multiplicities form valid curves where
the native code walks them through raw pointers, and the Taylor extrapolation
branch of the flat-knot Eval (which reads uninitialized stack) is never entered.
"""
from __future__ import annotations
import random
from tkmath_bsplclib_curves import Curve, array, curve, encode, flat_knots
from tkmath_bsplclib_eval_corpus import eval_families


def line(op: str, values: list) -> str:
    """Format one request; line('bspl_knots', [3, 0])."""
    return op+' '+' '.join(map(str, values))


def random_kind(rng: random.Random, allow_flat: bool = True) -> tuple[int, int]:
    """(kind, dim): flat reals of dimension 1..5, gp_Pnt2d or gp_Pnt."""
    kind = rng.choice([1, 2, 3] if allow_flat else [2, 3])
    return kind, (rng.randint(1, 5) if kind == 1 else kind)


def sequence_ok(degree: int, periodic: bool, mults: list[int]) -> bool:
    """KnotSequence stays in its arrays (see its periodic prefix/suffix walks)."""
    if any(m < 1 for m in mults):
        return False
    if not periodic:
        return True
    m1 = degree+1-mults[0]
    return 0 <= m1 <= min(sum(mults[:-1]), sum(mults[1:]))


def knots_requests(rng: random.Random) -> list[str]:
    """Knot management on valid and invalid multiplicity patterns and array bounds."""
    requests = []
    for case in range(320):
        degree, periodic = rng.randint(1, 7), case % 3 == 0
        definition = curve(rng, degree, periodic, style=rng.choice(['random', 'uniform', 'tiny', 'huge']),
                           clamped=case % 4 != 1)
        knots, mults = definition.knots, list(definition.mults)
        if case % 9 == 0: mults[rng.randrange(len(mults))] = rng.choice([0, degree+2, -1])
        if case % 13 == 0: knots = knots[:1]; mults = mults[:1]
        lower = rng.choice([1, 1, 0, 3])
        xs = [rng.uniform(knots[0]-1, knots[-1]+1) for _ in range(4)]+[knots[0], knots[-1], knots[len(knots)//2]]
        weights = [rng.choice([1.0, 2.0, rng.uniform(0.5, 2)]) for _ in range(rng.randint(1, 6))]
        wl = rng.choice([1, 0, 2])
        pairs = []
        for _ in range(3):
            i1 = rng.randint(wl, wl+len(weights)-1)
            pairs += [i1, i1+rng.randint(0, 2*len(weights))]
        last = rng.randint(wl, wl+2*len(weights))
        requests.append(line('bspl_knots', [degree, int(periodic)]+array(lower, knots)+array(lower, mults)
                             + [len(xs)]+xs+[rng.uniform(-3, 3), rng.uniform(-3, 3),
                                             int(sequence_ok(degree, periodic, mults) and lower == 1)]
                             + array(wl, weights)+[3]+pairs+[last]))
    return requests


def locate_requests(rng: random.Random) -> list[str]:
    """The three public LocateParameter overloads, periodic wrapping and hints."""
    requests = []
    for case in range(384):
        degree, periodic = rng.randint(1, 5), case % 4 == 0
        definition = curve(rng, degree, periodic, style=rng.choice(['random', 'uniform', 'tiny']))
        mode, flat = case % 3, case % 2 == 0
        knots = flat_knots(definition) if mode == 1 or (mode == 2 and flat) else definition.knots
        lower = rng.choice([1, 1, 0, 2])
        span = knots[-1]-knots[0]
        u = rng.choice([rng.uniform(knots[0]-span, knots[-1]+span), rng.choice(knots),
                        rng.choice(knots)+rng.choice([1e-17, -1e-17, 1e-15]), knots[0]-3*span])
        k1 = rng.randint(lower, lower+len(knots)-1)
        k2 = rng.randint(lower, lower+len(knots)-1)
        hint = rng.choice([0, lower-1, rng.randint(lower, lower+len(knots)-1)])
        use = int(mode == 2 and not flat)
        mults = definition.mults if use or mode == 0 else [1]*len(knots)
        requests.append(line('bspl_locate', [mode, degree, int(periodic)]+array(lower, knots)+[use]
                             + array(lower, mults)+[u, k1, k2, hint]))
    return requests


def scheme_requests(rng: random.Random) -> list[str]:
    """Boor, anti-Boor, derivative and Bohm schemes on raw local buffers."""
    requests = []
    for case in range(640):
        op, degree, dim = case % 5, rng.randint(0, 9), rng.randint(1, 6)
        u = rng.uniform(-1, 2)
        depth, length = rng.randint(0, 4), rng.randint(1, 6)
        # AntiBoorScheme with Length 1 reads Poles[(Depth-1)*Dim..(Depth+2)*Dim).
        if op == 2: depth = 1 if length == 1 else max(depth, 1)
        knots_needed = {0: 2*degree, 1: length+degree, 2: length+degree, 3: length+degree, 4: 2*degree}[op]
        poles_needed = {0: (degree+1)*dim, 1: (2*length+1)*dim, 2: (2*length+1)*dim, 3: length*dim, 4: (degree+1)*dim}[op]
        knots = sorted(rng.uniform(-1, 2) for _ in range(knots_needed))
        if case % 7 == 0 and len(knots) > 3: knots[2] = knots[1]
        poles = [rng.uniform(-10, 10) for _ in range(poles_needed)]
        tol = rng.choice([1e-9, 1e-3, 10.0, 1e6])
        requests.append(line('bspl_scheme', [op, degree, dim, u, depth, length, tol, len(knots)]+knots+[len(poles)]+poles))
    return requests


def build_requests(rng: random.Random) -> list[str]:
    """Local knots (flat and with multiplicities), local poles and Boor arrays."""
    requests = []
    for case in range(320):
        degree, periodic = rng.randint(1, 6), case % 3 == 0
        definition = curve(rng, degree, periodic)
        use = case % 2 == 0
        if use:
            knots = definition.knots
            index = rng.randint(1, len(knots) if periodic else len(knots)-1)
        else:
            knots = flat_knots(definition)
            index = rng.randint(degree, len(knots)-degree)
        count = rng.randint(1, 8)
        weights = [rng.uniform(0.5, 2) for _ in range(count)]
        p1 = [rng.uniform(-5, 5) for _ in range(count)]
        p3 = [rng.uniform(-5, 5) for _ in range(3*count)]
        p2 = [rng.uniform(-5, 5) for _ in range(2*count)]
        blength, bdim = rng.randint(0, 4), rng.randint(1, 3)
        requests.append(line('bspl_build', [degree, index, int(periodic), int(use)]+array(1, knots)
                             + array(1, definition.mults)+[rng.randint(-1, count+2), rng.randint(0, 1)]
                             + array(1, weights)+array(1, p1)+[1, count]+p3+[1, count]+p2
                             + [rng.randint(0, count), blength, bdim, rng.randint(0, blength), rng.randint(1, 2*bdim)]))
    return requests


def curve_for_ops(rng: random.Random, case: int, periodic: bool, clamped: bool = True) -> Curve:
    """A curve of any kind for knot/degree operations."""
    kind, dim = random_kind(rng)
    rational = kind > 1 and case % 2 == 0
    return curve(rng, rng.randint(1, 5), periodic, kind, dim, rational, clamped,
                 rng.choice(['random', 'uniform', 'tiny']))


def insert_requests(rng: random.Random) -> list[str]:
    """PrepareInsertKnots/InsertKnots (flat and points), InsertKnot, RaiseMultiplicity."""
    requests = []
    for case in range(448):
        definition = curve_for_ops(rng, case, case % 4 == 0)
        knots, degree = definition.knots, definition.degree
        count = rng.randint(1, 4)
        add = sorted(rng.uniform(knots[0], knots[-1]) for _ in range(count))
        if case % 3 == 0: add[rng.randrange(count)] = rng.choice(knots[1:-1] or knots)
        if case % 11 == 0: add = add[::-1]
        if case % 17 == 0: add[-1] = knots[-1]+1.0
        add.sort() if case % 11 else None
        add_mults = [rng.randint(0, degree+1) for _ in add]
        use = case % 5 != 0
        tol = rng.choice([0.0, 1e-9, 1e-3])
        u = rng.uniform(knots[0], knots[-1])
        index = rng.randint(1, len(knots))
        requests.append(line('bspl_insert', encode(definition)+array(1, add)+[int(use)]+array(1, add_mults)
                             + [tol, int(case % 6 != 0), u, rng.randint(1, degree), index, rng.randint(0, degree)]))
    return requests


def remove_requests(rng: random.Random) -> list[str]:
    """RemoveKnot to every lower multiplicity, exact and smoothing tolerances."""
    requests = []
    for case in range(384):
        definition = curve_for_ops(rng, case, case % 4 == 0)
        knots, mults = definition.knots, definition.mults
        index = rng.randint(0, len(knots)+1) if case % 4 == 3 else rng.randint(1, len(knots))
        valid = 1 <= index <= len(knots)
        mult = rng.randint(0, mults[index-1]-1) if valid and mults[index-1] > 0 else 0
        new_poles = definition.count()-(mults[index-1]-mult if valid else 0)
        new_knots = len(knots)-1 if mult == 0 else len(knots)
        # Random poles are rarely removable: large tolerances reach the averaging path.
        tol = rng.choice([1e-9, 1e-3, 1.0, 1e3, 1e3, 1e6])
        requests.append(line('bspl_remove', encode(definition)+[index, mult, tol, new_poles, new_knots]))
    return requests


def degree_requests(rng: random.Random) -> list[str]:
    """Degree elevation of clamped, periodic and unclamped curves."""
    requests = []
    for case in range(320):
        periodic, clamped = case % 4 == 0, case % 5 != 1
        definition = curve_for_ops(rng, case, periodic, clamped)
        step = rng.randint(0, 3)
        new_mults = [m+step for m in definition.mults]
        if not periodic:
            new_mults[0] = new_mults[-1] = definition.degree+step+1
        poles = sum(new_mults[:-1]) if periodic else sum(new_mults)-(definition.degree+step)-1
        requests.append(line('bspl_degree', encode(definition)+[definition.degree+step, poles]))
    return requests


def unperiodize_requests(rng: random.Random) -> list[str]:
    """PrepareUnperiodize/Unperiodize on periodic curves."""
    return [line('bspl_unperiodize', encode(curve_for_ops(rng, case, True))) for case in range(192)]


def trim_requests(rng: random.Random) -> list[str]:
    """PrepareTrimming/Trimming inside the range, on knots and across spans."""
    requests = []
    for case in range(320):
        definition = curve_for_ops(rng, case, case % 4 == 0)
        knots = definition.knots
        u1, u2 = sorted(rng.uniform(knots[0], knots[-1]) for _ in range(2))
        if case % 3 == 0: u1 = rng.choice(knots[:-1])
        if case % 5 == 0: u2 = rng.choice(knots[1:])
        if u2 <= u1: u2 = knots[-1]
        requests.append(line('bspl_trim', encode(definition)+[u1, u2]))
    return requests


def append_bsplclib_requests(requests: list[str]) -> None:
    """Extend the corpus with seeded BSplCLib requests; append_bsplclib_requests(requests)."""
    rng = random.Random(19910809)
    for family in [knots_requests, locate_requests, scheme_requests, build_requests, insert_requests,
                   remove_requests, degree_requests, unperiodize_requests, trim_requests]:
        requests.extend(family(rng))
    for family in eval_families():
        requests.extend(line(op, values) for op, values in family(rng))
