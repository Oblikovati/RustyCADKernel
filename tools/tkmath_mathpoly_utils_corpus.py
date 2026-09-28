"""Deterministic MathPoly::Utils requests appended after the planar gp corpus.

Polynomials are built from chosen roots (simple, repeated and complex-conjugate), from
random coefficients and from degenerate shapes: zero leading or constant terms, huge and
tiny coefficient spreads, NaN/infinite coefficients and tolerances. Query points sit on,
near and away from roots; exponents cover ToDouble overflow and underflow; multiplicities,
iteration budgets and root lists drive the multiplicity, deflation and consolidation paths.
"""
import random

EXPONENTS = [0, 0, 1, -2, 3, 40, -40, 1100, -1100]


def line(op: str, values: list) -> str:
    """Format one request with round-trip float text; line('mathpoly_utils_long', [1, 2])."""
    return op+' '+' '.join(str(value) for value in values)


def append_mathpoly_utils_requests(requests: list[str]) -> None:
    """Append every MathPoly::Utils request with its own seed; append_mathpoly_utils_requests(r)."""
    rng = random.Random(20260704)
    for case in range(700):
        requests.append(line('mathpoly_utils_long', request(rng, case, rng.randint(-1, 6))))
    for case in range(700):
        requests.append(line('mathpoly_utils_complex', request(rng, case, rng.randint(1, 6))))


def expand(roots: list[complex], leading: float) -> list[float]:
    """Ascending real coefficients of leading * prod(x - r); expand([1, 2], 1.0)."""
    coefficients = [complex(leading)]
    for root in roots:
        shifted = [0j] + coefficients
        coefficients = [shifted[i] - root*(coefficients[i] if i < len(coefficients) else 0)
                        for i in range(len(shifted))]
    return [c.real for c in coefficients]


def chosen_roots(rng: random.Random, degree: int) -> tuple[list[complex], list[float], list[complex]]:
    """Roots with repeats and conjugate pairs; returns (all roots, reals, upper complexes)."""
    roots, reals, uppers = [], [], []
    while len(roots) < degree:
        if degree - len(roots) >= 2 and rng.random() < 0.3:
            z = complex(round(rng.uniform(-3, 3), 2), round(rng.uniform(0.1, 3), 2))
            roots += [z, z.conjugate()]
            uppers.append(z)
            continue
        value = rng.choice(reals) if reals and rng.random() < 0.35 else round(rng.uniform(-4, 4), 1)
        roots.append(complex(value))
        reals.append(value)
    return roots, reals, uppers


def coefficients(rng: random.Random, case: int, degree: int) -> tuple[list[float], list[float], list[complex]]:
    """Seven coefficient slots plus the real and complex roots used to build them."""
    pattern = case % 6
    if degree < 1 or pattern == 5:
        c = [rng.uniform(-10, 10) for _ in range(7)]
        return c, [], []
    roots, reals, uppers = chosen_roots(rng, degree)
    leading = rng.choice([1.0, -2.5, 1e-150, 3e140, rng.uniform(-5, 5)])
    c = expand(roots, leading) + [0.0]*(6-degree)
    if pattern == 1:
        c[0] = 0.0
    if pattern == 2:
        c[degree] = 0.0
    if pattern == 3:
        c = [value*rng.choice([1e-120, 1e120, 1.0]) for value in c]
    return c, reals, uppers


def request(rng: random.Random, case: int, degree: int) -> list:
    """One request: degree, seven coefficients, queries, root lists and a special flag."""
    c, reals, uppers = coefficients(rng, case, degree)
    x = rng.choice(reals) + rng.choice([0.0, 0.0, 1e-9, -1e-6]) if reals and case % 3 else rng.uniform(-6, 6)
    z = uppers[0] + complex(rng.choice([0.0, 1e-8]), 0) if uppers and case % 2 else complex(rng.uniform(-3, 3), rng.uniform(-3, 3))
    tolerance = rng.choice([1e-12, 1e-8, 1e-3, 1.0, -1.0, 0.0])
    multiplicity = rng.randint(0, min(max(degree, 0)+1, 3))
    start = complex(rng.uniform(-2, 2), rng.uniform(-2, 2))
    listed = (reals + [rng.uniform(-4, 4) for _ in range(6)])[:6]
    # Cluster sizes above degree + 1 would index a negative derivative degree (UB natively).
    nb_real = rng.randint(0, min(6, degree)) if degree >= 1 else 0
    complexes = (uppers + [complex(rng.uniform(-3, 3), rng.uniform(-3, 3)) for _ in range(4)])[:4]
    nb_complex = rng.randint(0, min(4, degree // 2)) if degree >= 1 else 0
    special = rng.choice([0]*9 + [1, 2, 3, 4]) if degree >= 0 else 0
    return ([degree] + c + [x, tolerance, z.real, z.imag, multiplicity, rng.choice([0, 1, 5, 100]),
            rng.choice(EXPONENTS), start.real, start.imag, nb_real] + listed + [nb_complex]
            + [part for w in complexes for part in (w.real, w.imag)] + [special])
