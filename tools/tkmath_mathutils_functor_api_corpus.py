"""Deterministic MathUtils_FunctorScalar / MathUtils_FunctorVector API-completion requests.

Coefficient vectors take every length from empty to nine with shifted lower bounds, so the
empty, single-coefficient and long Horner paths run; the arithmetic combinators get a
Gaussian that fails (zero sigma, NaN input) and a linear polynomial that reaches zero; the
lambda wrappers fail after writing their outputs; N-dimensional functors see zero to six
coordinates, non-finite coordinates and parameters, and non-square residual matrices.
Non-finite values are injected by mode flags because the native probe reads with `>>`.
"""
import random

SPECIALS = [0.0, -0.0, 1.0, -1.0, 1e-300, 1e300, 0.5]


def line(op: str, values: list) -> str:
    """Format one request with round-trip float text; line('mathutils_functor_lambda', [1, 2])."""
    return op + ' ' + ' '.join(str(value) for value in values)


def number(rng: random.Random) -> float:
    """A coordinate that is usually moderate and sometimes a boundary value; number(rng)."""
    return rng.choice(SPECIALS) if rng.random() < 0.15 else rng.uniform(-6, 6)


def append_poly(requests: list[str], rng: random.Random) -> None:
    """Polynomial/Rational over every length and split; append_poly(requests, rng)."""
    for case in range(400):
        size = case % 10
        split = rng.randint(0, size)
        coefficients = [number(rng) for _ in range(size)]
        x = rng.choice([0.0, 1.0, -1.0, number(rng), 1e200, -1e200])
        requests.append(line('mathutils_functor_poly',
                             [rng.randint(-5, 5), size, split, case % 4, x] + coefficients))


def append_arith(requests: list[str], rng: random.Random) -> None:
    """Combinators over a failing Gaussian and a vanishing line; append_arith(requests, rng)."""
    for case in range(400):
        a, b, c, d, e, k = (number(rng) for _ in range(6))
        x = -d / e if case % 7 == 0 and e != 0.0 else number(rng)
        if case % 11 == 0:
            c = -c
        requests.append(line('mathutils_functor_arith', [case % 5, x, a, b, c, d, e, k]))


def append_lambda(requests: list[str], rng: random.Random) -> None:
    """Scalar lambda wrappers on both sides of their failure tests; append_lambda(requests, rng)."""
    for x in [0.0, -0.0, 1.0, -1.0, 2.0, 1e200]:
        requests.append(line('mathutils_functor_lambda', [x, 3.0]))
    for _ in range(200):
        requests.append(line('mathutils_functor_lambda', [number(rng), number(rng)]))


def append_vlambda(requests: list[str], rng: random.Random) -> None:
    """Vector and system lambda wrappers for 1..6 coordinates; append_vlambda(requests, rng)."""
    for case in range(240):
        size = 1 + case % 6
        coordinates = [number(rng) for _ in range(size)]
        requests.append(line('mathutils_functor_vlambda', [rng.randint(-4, 4), size] + coordinates))


def append_nd(requests: list[str], rng: random.Random) -> None:
    """Sphere/Rastrigin/Ackley/residual/quadratic forms; append_nd(requests, rng)."""
    for case in range(400):
        size, rows = case % 7, rng.randint(0, 6)
        values = [rng.randint(-4, 4), size, rows, case % 5, number(rng), number(rng), number(rng)]
        values += [number(rng) for _ in range(size)]
        if size > 0 and rows > 0:
            values += [number(rng) for _ in range(rows * size + rows)]
        requests.append(line('mathutils_functor_nd', values))


def append_mathutils_functor_api_requests(requests: list[str]) -> None:
    """Append every functor API request with its own seed; append_mathutils_functor_api_requests(r)."""
    rng = random.Random(20260928)
    append_poly(requests, rng)
    append_arith(requests, rng)
    append_lambda(requests, rng)
    append_vlambda(requests, rng)
    append_nd(requests, rng)
