"""Deterministic requests for solver Dump text, statuses and convergence-hook overrides.

Each case runs a solver with its default convergence test and with a subclass/closure
that reports convergence on the n-th test, so the hook placement, the test count and
the resulting Dump text (including math_Status codes and %g formatting) are compared.
Callback failures on a chosen call exercise the FunctionError/DirectionSearchError
exits; tiny iteration limits exercise TooManyIterations.
"""
import random


def append_solver_dump_requests(requests: list[str]) -> None:
    """Append every request with its own seed; append_solver_dump_requests(requests)."""
    rng = random.Random(20260928)
    append_minimizer_requests(rng, requests)
    append_scalar_requests(rng, requests)
    append_polynomial_requests(rng, requests)
    append_set_requests(rng, requests)
    append_small_type_requests(rng, requests)


def line(op: str, values: list) -> str:
    """Format one request with round-trip float text; line('dump_scalar', [1, 0])."""
    return op + ' ' + ' '.join(str(value) for value in values)


def append_minimizer_requests(rng: random.Random, requests: list[str]) -> None:
    for index in range(48):
        n = rng.randint(1, 4)
        quartic = index % 2
        fail = rng.choice([0, 0, 0, 1, 2, 3, 5])
        limit = rng.choice([1, 2, 5, 40, 200])
        tol = rng.choice([1e-8, 1e-3, 1e-12])
        cutoff = rng.randint(1, 4)
        centers = [rng.uniform(-3, 3) for _ in range(n)]
        weights = [rng.uniform(0.5, 4) for _ in range(n)]
        start = [rng.uniform(-5, 5) for _ in range(n)]
        requests.append(line('dump_minimizers', [n, quartic, fail, limit, tol, cutoff, *centers, *weights, *start]))


def append_scalar_requests(rng: random.Random, requests: list[str]) -> None:
    requests.append(line('dump_scalar', [8.0, 1.0, 0.0, 4.0, 0, 2]))
    requests.append(line('dump_scalar', [8.0, 1.0, 3.0, 4.0, 0, 1]))
    for _ in range(40):
        c = rng.uniform(-10, 10)
        a = rng.uniform(-4, 0)
        b = rng.uniform(0.5, 4)
        requests.append(line('dump_scalar', [c, rng.uniform(-2, 2), a, b, rng.choice([0, 0, 1, 3, 7]), rng.randint(1, 5)]))


def append_polynomial_requests(rng: random.Random, requests: list[str]) -> None:
    requests.append(line('dump_polynomial', [0, 0, 0, 0, 0, 0, 6.2]))
    requests.append(line('dump_polynomial', [1, -10, 35, -50, 24, 0, 6.2]))
    for _ in range(32):
        coefficients = [rng.choice([0.0, rng.uniform(-5, 5)]) for _ in range(5)]
        low = rng.uniform(-3, 3)
        requests.append(line('dump_polynomial', [*coefficients, low, low + rng.uniform(0.5, 7)]))


def append_set_requests(rng: random.Random, requests: list[str]) -> None:
    for index in range(40):
        n = rng.randint(1, 3)
        limit = rng.choice([1, 3, 12, 25, 60])
        tol = rng.choice([1e-10, 1e-6])
        fail = rng.choice([0, 0, 0, 1, 2, 4])
        coupling = 0.0 if index % 3 == 0 else rng.uniform(-0.5, 0.5)
        c = [rng.uniform(0.5, 9) for _ in range(n)]
        start = [rng.uniform(0.5, 3) for _ in range(n)]
        requests.append(line('dump_sets', [n, limit, tol, fail, rng.randint(1, 4), coupling, *c, *start]))


def append_small_type_requests(rng: random.Random, requests: list[str]) -> None:
    requests.append(line('small_types', [2, 1.0, 5.0, 1.0, 7.0]))
    for _ in range(16):
        n = rng.randint(1, 12)
        pairs = []
        for _ in range(n):
            pairs += [rng.choice([rng.uniform(-3, 3), 0.5, -0.5]), rng.uniform(0, 2)]
        requests.append(line('small_types', [n, *pairs]))
