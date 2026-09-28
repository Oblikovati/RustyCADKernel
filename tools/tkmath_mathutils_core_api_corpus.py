"""Deterministic MathUtils core API requests (Core, Config, Convergence, Domain, Poly, Deriv, Utils,
Gauss/Kronrod wrappers, Random) for tools/oracle/tkmath_mathutils_core_api_probe.cxx.

Scalars mix ordinary values with signed zeros, the ZERO_TOL and machine-epsilon thresholds,
subnormals, huge values, infinities and NaN; tolerances include negative, zero, NaN and infinite
ones. The native reader cannot parse inf/NaN, so they travel as sentinels decoded by both probes.
"""
import math
import random

INF, NEG_INF, NAN = 7.77e307, -7.77e307, 6.66e307
SPECIAL = [0.0, -0.0, 1.0, -1.0, 0.5, 2.0, -2.5, 3.5, 1e-15, -1e-15, 1e-16, 2.220446049250313e-16,
           5e-324, 1e-300, 1e15, 1e16, 1e100, 2e100, -2e100, 1e300, -1e300, INF, NEG_INF, NAN]
TOLERANCES = [0.0, 1e-15, 1e-10, 1e-3, 0.5, 1000.0, -1.0, 1e-300, INF, NAN]


def line(op: str, values: list) -> str:
    """Format one request with round-trip float text; line('mathutils_core_random', [1, 2])."""
    return op + ' ' + ' '.join(repr(float(value)) for value in values)


def scalar(rng: random.Random) -> float:
    """A special value or a random magnitude; scalar(random.Random(1))."""
    if rng.random() < 0.45:
        return rng.choice(SPECIAL)
    return rng.uniform(-10, 10) * 10 ** rng.randint(-20, 20)


def append_scalar(rng: random.Random, requests: list[str]) -> None:
    """Core/Config/Convergence/Domain/Utils scalar requests."""
    for case in range(900):
        a = scalar(rng)
        b = a + rng.choice([0.0, 1e-12, -1e-12, 1.0, rng.uniform(-50, 50)]) if case % 3 == 0 else scalar(rng)
        values = [scalar(rng), a, b, rng.choice(TOLERANCES), rng.choice(TOLERANCES + [2.0, 6.5]),
                  rng.choice([0, 1, 5, 100])]
        requests.append(line('mathutils_core_scalar', values))
    for exponent in range(-1074, 1024, 7):
        # Sweep every binade residue mod 3 for the reproduced glibc cbrt kernel (CubeRoot).
        x = rng.uniform(-1, 1) * 2.0 ** exponent if exponent > -1022 else rng.choice([1, -3, 7]) * 2.0 ** exponent
        requests.append(line('mathutils_core_scalar', [x, 1.0, 2.0, 1e-10, 0.5, 5]))
    requests.append(line('mathutils_core_defaults', []))


def append_vector(rng: random.Random, requests: list[str]) -> None:
    """Convergence and norm requests on vectors of equal and unequal size."""
    for case in range(300):
        n = rng.randint(1, 6)
        m = n if case % 5 else rng.randint(1, 6)
        xs = [scalar(rng) if case % 4 == 0 else rng.uniform(-3, 3) for _ in range(n)]
        ys = [x + rng.choice([0.0, 1e-12, 1e-6, 0.1]) for x in xs[:m]] + [rng.uniform(-3, 3)] * (m - n)
        requests.append(line('mathutils_core_vector', [n, m] + xs + ys[:m] + [rng.choice(TOLERANCES)]))


def roots_poly(rng: random.Random) -> list[float]:
    """Ascending coefficients of a quartic with chosen real roots, padded to six slots."""
    coefficients = [1.0]
    for root in [round(rng.uniform(-4, 4), 1) for _ in range(4)]:
        shifted = [0.0] + coefficients
        coefficients = [shifted[i] - root * (coefficients[i] if i < len(coefficients) else 0.0)
                        for i in range(len(shifted))]
    return coefficients + [rng.uniform(-1, 1)]


def append_poly(rng: random.Random, requests: list[str]) -> None:
    """Horner evaluation, Newton refinement, sorting, deduplication and depression requests."""
    for case in range(600):
        c = roots_poly(rng) if case % 2 else [scalar(rng) if case % 7 == 0 else rng.uniform(-5, 5) for _ in range(6)]
        roots = [rng.choice([1.0, -1.0, 1.0 + 1e-11, 0.0, -0.0, NAN, INF]) if case % 6 == 0
                 else round(rng.uniform(-3, 3), rng.choice([0, 1, 12])) for _ in range(4)]
        values = [rng.randint(-1, 5)] + c + [scalar(rng) if case % 5 == 0 else rng.uniform(-4, 4),
                  rng.uniform(-4, 4), rng.choice([0, 1, 5, 30]), rng.choice(TOLERANCES), rng.randint(0, 4)]
        requests.append(line('mathutils_core_poly', values + roots))


def append_deriv(rng: random.Random, requests: list[str]) -> None:
    """Finite-difference, EvaluateFinite and iteration-count BracketMinimum requests."""
    steps = [1e-8, 1e-5, 0.25, 1e-30, 0.0, -1e-3, NAN, INF, 1e300]
    for case in range(500):
        center = round(rng.uniform(-2, 2), 2)
        x = rng.choice([center, center + 1e-3, center - 0.5, 1e16, -0.0, NAN, rng.uniform(-3, 3)])
        values = [case % 4, center, x, rng.choice(steps), rng.choice([0.0, 1.5, rng.uniform(-5, 5), NAN]),
                  rng.uniform(-2, 2), rng.choice([0, 1, 3, 50])]
        requests.append(line('mathutils_core_deriv', values))


def append_gauss_random(rng: random.Random, requests: list[str]) -> None:
    """Quadrature wrappers at valid, even, oversized and mismatched orders; generator streams."""
    for order in list(range(-1, 32)) + [60, 61, 62, 63, 64, 123, 124, 125]:
        requests.append(line('mathutils_core_gauss', [order, max(order, 1)]))
    for order, size in [(5, 4), (7, 8), (15, 1)]:
        requests.append(line('mathutils_core_gauss', [order, size]))
    for seed in [0, 1, 2, 42, 2 ** 32, 2 ** 52, rng.randint(0, 2 ** 53)]:
        requests.append(line('mathutils_core_random', [seed, rng.randint(0, 2 ** 40)]))


def append_mathutils_core_api_requests(requests: list[str]) -> None:
    """Append every MathUtils core API request; append_mathutils_core_api_requests(r)."""
    rng = random.Random(20260928)
    append_scalar(rng, requests)
    append_vector(rng, requests)
    append_poly(rng, requests)
    append_deriv(rng, requests)
    append_gauss_random(rng, requests)
    assert all(math.isfinite(float(v)) for r in requests for v in r.split()[1:])
