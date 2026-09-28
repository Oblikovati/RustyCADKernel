"""Deterministic MathRoot::Utils requests for the public helper API.

`mathroot_utils` drives the scalar predicates, the periodic representative count, the
effective X tolerance, multiple-input validation and trig-root mapping with ordinary,
signed-zero, subnormal, huge, infinite and NaN operands. `mathroot_trig_roots` and
`mathroot_sort` exercise the ordered insertion, duplicate filtering and capacity limits.
`mathroot_search` reuses the modern_multiple request layout and objective modes (double
roots, NaN/infinite callbacks, failing call indices, untouched outputs) to reach every
evaluation helper, functor adapter, RefineBracketedRoot, FindMultipleRoots, FindAllRootsImpl
and FindAllRootsWithDerivativeImpl.
"""
import math
import random

SPECIAL = [0.0, -0.0, 1.0, -1.0, 0.5, 2.5, -3.75, math.pi, 2 * math.pi, 7.0, 1e-12, 5e-324,
           1e-300, 1e16, 1e300, -1e300, 1.7976931348623157e308, math.inf, -math.inf, math.nan]


def line(op: str, values: list) -> str:
    """Format one request with round-trip float text; line('mathroot_sort', [1e-3, 0])."""
    return op + ' ' + ' '.join(str(value) for value in values)


def append_mathroot_api_requests(requests: list[str]) -> None:
    """Append every MathRoot::Utils request; append_mathroot_api_requests(requests)."""
    rng = random.Random(20260928)
    append_utils(rng, requests)
    append_trig_roots(rng, requests)
    append_sort(rng, requests)
    append_search(rng, requests)


def operand(rng: random.Random) -> float:
    """A special value half the time, else an ordinary one; operand(rng)."""
    return rng.choice(SPECIAL) if rng.random() < 0.5 else rng.uniform(-20.0, 20.0)


def append_utils(rng: random.Random, requests: list[str]) -> None:
    """Scalar predicates, periodic counts and trig mapping, random plus hand-picked edges."""
    for _ in range(600):
        values = [operand(rng) for _ in range(4)]
        if rng.random() < 0.5:
            values[2] = abs(values[2])
            values[3] = abs(values[3]) * rng.choice([0.0, 1e-12, 1.0])
        requests.append(line('mathroot_utils', values + [rng.randint(0, 1), rng.choice([0, 1, 2, 5, 100])]))
    for first, upper, period, tolerance in [(0.0, 6.0, 2.0, 0.0), (0.0, 6.0, 2.0, 1e-9),
                                            (1.0, 1.0, 1.0, 0.0), (0.0, 1e300, 1e-300, 0.0),
                                            (1e16, 1e16 + 8.0, 1.0, 0.0), (-5.0, 5.0, 2 * math.pi, 1e-12),
                                            (0.0, 2 * math.pi, 2 * math.pi, 1e-12), (3.0, 1.0, 1.0, 0.0)]:
        for flag in (0, 1):
            requests.append(line('mathroot_utils', [first, upper, period, tolerance, flag, 5]))


def append_trig_roots(rng: random.Random, requests: list[str]) -> None:
    """Alternating periodic and distinct insertions, including duplicates and overflow of 4 slots."""
    for case in range(300):
        lower = rng.choice([0.0, -math.pi, rng.uniform(-20, 20)])
        upper = lower + rng.choice([2 * math.pi, 1.0, 0.0, 10.0, rng.uniform(0, 7)])
        tolerance = rng.choice([1e-12, 1e-6, 0.0, 0.5])
        count = rng.randint(0, 8)
        roots = [rng.choice([lower, upper, lower + 1.0, rng.uniform(-30, 30)]) for _ in range(count)]
        if case % 5 == 0 and roots:
            roots = [roots[0]] * count
        if case % 7 == 0 and roots:
            roots[-1] = rng.choice([math.nan, math.inf, -math.inf])
        requests.append(line('mathroot_trig_roots', [lower, upper, tolerance, count] + roots))


def append_sort(rng: random.Random, requests: list[str]) -> None:
    """AddRoot duplicate filtering then insertion sort, with repeats, signed zeros and NaN."""
    for case in range(300):
        eps = rng.choice([0.0, 1e-10, 0.5, 2.0, math.nan])
        count = rng.randint(0, 9)
        pairs = []
        for _ in range(count):
            root = rng.choice([0.0, -0.0, 1.0, rng.uniform(-5, 5), round(rng.uniform(-3, 3), 1)])
            if case % 11 == 0:
                root = rng.choice([root, math.nan, math.inf])
            pairs += [root, rng.uniform(-1, 1)]
        requests.append(line('mathroot_sort', [eps, count] + pairs))


def append_search(rng: random.Random, requests: list[str]) -> None:
    """modern_multiple layout: mode target lower upper n xtol ftol ntol limit fail offset."""
    for case in range(400):
        mode = [0, 1, 2, 3, 4, 5, 6, 7, 9][case % 9]
        target = rng.choice([0.25, 2.0, 0.0, -1.0, rng.uniform(0.1, 4.0)])
        lower, upper = rng.choice([(-3.0, 3.0), (0.0, 2.0), (0.9, 1.1), (-1.0, 0.5), (-4.0, 4.0),
                                   (0.5, 1.9), (-2.0, -0.1)] * 3 + [(2.0, 1.0), (0.0, 0.0)])
        n = rng.choice([2, 5, 10, 23] * 3 + [1])
        xtol = rng.choice([1e-10, 1e-6, 1e-3] * 3 + [0.0])
        ftol = rng.choice([1e-10, 1e-6, 1e-3, 0.0, 0.1])
        ntol = rng.choice([1e-12, 1e-6])
        limit = rng.choice([100] * 6 + [3, 0])
        fail = rng.choice([0] * 8 + [1, 2, 3, 5, 9, 17, 40, 90])
        offset = rng.choice([0.0, 0.0, 0.5, -0.25, 1e-6])
        requests.append(line('mathroot_search', [mode, target, lower, upper, n, xtol, ftol, ntol,
                                                 limit, fail, offset]))
