"""Deterministic MathOpt::Utils, PolishCoordinateWise and modern Uzawa requests.

Covers the validation predicates on finite, NaN and infinite configs, vectors, matrices and
boxes; every line-search status pair; the evaluation wrappers against an objective that fails
on its k-th callback or returns NaN/infinite values, gradients and Hessians; stationarity
probes at minima, saddles and ordinary points; Armijo backtracking with ascent, zero and
invalid steps; Hessian eigenvalue shifts; coordinate polishing under small budgets; and
Uzawa on equality-only, mixed and inequality-only systems including singular, rank-deficient,
mis-sized and non-finite inputs. NaN and infinities travel as sentinels (7777.25, +-8888.25)
because the native probe reads numbers with operator>>.
"""
import math
import random

NAN, INF = 7777.25, 8888.25


def encode(value: float) -> float:
    """Map NaN/infinities to the probe sentinels; encode(float('nan')) -> 7777.25."""
    if math.isnan(value):
        return NAN
    if math.isinf(value):
        return INF if value > 0 else -INF
    return value


def line(op: str, values: list) -> str:
    """One request with round-trip float text; line('mathopt_line', [0, 3])."""
    return op + ' ' + ' '.join(str(encode(value) if isinstance(value, float) else value) for value in values)


def spiced(rng: random.Random, value: float, rate: float) -> float:
    """Occasionally replace a value by NaN or an infinity; spiced(rng, 1.0, 0.1)."""
    if rng.random() >= rate:
        return value
    return rng.choice([math.nan, math.inf, -math.inf])


def objective(rng: random.Random) -> list:
    """Objective header: mode, fail index, center, coupling."""
    mode = rng.choice([0, 0, 0, 1, 2, 3, 4])
    return [mode, rng.choice([0, 0, 0, 1, 2, 3, 5, 9]), float(round(rng.uniform(-2, 2), 3)),
            float(rng.choice([0.0, 0.5, -3.0, 2.0]))]


def vector(rng: random.Random, n: int, low: float, high: float, rate: float = 0.0) -> list:
    return [spiced(rng, rng.uniform(low, high), rate) for _ in range(n)]


def append_checks(requests: list[str], rng: random.Random) -> None:
    tolerances = [1e-8, 1e-3, 0.0, -1e-8, math.nan, math.inf]
    for case in range(300):
        config = [rng.choice([0, 1, 100])] + [rng.choice(tolerances) for _ in range(4)]
        config.append(rng.choice([2.220446049250313e-16, 0.0, 1e-300, math.nan]))
        if case % 2 == 0:  # valid except, sometimes, a zero relative tolerance (still valid)
            config = [100, 1e-8, 1e-6, 1e-7, rng.choice([1e-8, 0.0]), 2.220446049250313e-16]
        n = rng.randint(1, 4)
        a = vector(rng, n, -5, 5, 0.08)
        b = [value + rng.choice([1.0, 0.0, -1.0, 1e308]) for value in a] if rng.random() < 0.7 else vector(rng, n, -5, 5, 0.1)
        m = vector(rng, 4, -1e3, 1e3, 0.1)
        requests.append(line('mathopt_checks', config + [n] + a + b + m))


def append_lines(requests: list[str]) -> None:
    for first in range(11):
        for second in range(11):
            requests.append(line('mathopt_line', [first, second]))


def append_eval(requests: list[str], rng: random.Random) -> None:
    for _ in range(400):
        n = rng.randint(1, 4)
        x = vector(rng, n, -3, 3, 0.05)
        lower = vector(rng, n, -2, 0, 0.05)
        upper = [value + rng.choice([0.5, 2.0, -1.0]) for value in lower]
        requests.append(line('mathopt_eval', objective(rng) + [n] + x + lower + upper))


def append_stationary(requests: list[str], rng: random.Random) -> None:
    for case in range(300):
        header = objective(rng)
        n = rng.randint(1, 4)
        x = [header[2]] * n if case % 3 == 0 else vector(rng, n, -3, 3, 0.03)
        fx = rng.choice([0.0, rng.uniform(-1, 5), math.nan, -1e300]) if case % 4 else sum((v - header[2]) ** 2 for v in x)
        tolerances = [rng.choice([1e-8, 1e-4, 0.1, 0.0]), rng.choice([1e-10, 1e-6, 1e-2, math.nan])]
        requests.append(line('mathopt_stationary', header + [n] + x + [float(fx)] + tolerances))


def append_backtrack(requests: list[str], rng: random.Random) -> None:
    for case in range(300):
        header = objective(rng) if case % 2 else [0, 0, float(round(rng.uniform(-2, 2), 3)), 0.0]
        n = rng.randint(1, 3)
        x = vector(rng, n, -3, 3, 0.03)
        g = [2.0 * (v - header[2]) if not math.isnan(v) else 0.0 for v in x]
        d = [-v * rng.choice([1.0, 0.1, 10.0]) for v in g] if case % 5 else vector(rng, n, -2, 2, 0.05)
        fx = sum((v - header[2]) ** 2 for v in x) if case % 7 else rng.choice([math.nan, 1.0, math.inf])
        step = rng.choice([1.0, 1.0, 10.0, 1e-3, 0.0, -1.0, math.nan, math.inf])
        step_min = rng.choice([2.220446049250313e-16, 1e-3, 0.5, 2.0])
        requests.append(line('mathopt_backtrack', header + [n] + x + d + g + [float(fx), step, step_min]))


def append_regularize(requests: list[str], rng: random.Random) -> None:
    for case in range(200):
        n = rng.randint(1, 4)
        cells = [[0.0] * n for _ in range(n)]
        for i in range(n):
            for j in range(i + 1):
                cells[i][j] = cells[j][i] = rng.uniform(-5, 5) * rng.choice([1.0, 1e-9, 1e9])
        if case % 17 == 0:
            cells[rng.randrange(n)][rng.randrange(n)] = rng.choice([math.nan, math.inf])
        minimum = rng.choice([1e-8, 0.0, 1.0, -1.0, 1e300, math.nan])
        requests.append(line('mathopt_regularize', [n] + [value for row in cells for value in row] + [minimum]))


def append_polish(requests: list[str], rng: random.Random) -> None:
    for _ in range(150):
        header = objective(rng)
        header[0] = rng.choice([0, 0, 1, 2])
        n = rng.randint(1, 3)
        lower = vector(rng, n, -4, -1)
        upper = vector(rng, n, 1, 4)
        x = [rng.uniform(lo, hi) for lo, hi in zip(lower, upper)]
        value = rng.choice([rng.uniform(0, 10), 1e300, 0.0])
        tolerance = rng.choice([1e-8, 1e-4, 1e-2])
        budget = rng.choice([1, 2, 5, 17, 40, 60])
        requests.append(line('mathopt_polish', header + [n] + x + lower + upper + [value, tolerance, budget]))


def uzawa_request(rng: random.Random, case: int) -> list:
    rows, cols = rng.randint(1, 4), rng.randint(1, 4)
    nce = rng.randint(0, rows) if case % 3 else rows
    nci = rows - nce if case % 23 else rows - nce + 1
    eps = [rng.choice([1e-6, 1e-10, 1e-3, 0.0, math.nan]) if case % 29 == 0 else rng.choice([1e-6, 1e-10, 1e-3])
           for _ in range(2)]
    limit = rng.choice([500, 500, 50, 5, 1, 0])
    c = vector(rng, rows * cols, -3, 3)
    if case % 7 == 0 and rows > 1:
        c[cols:2 * cols] = c[:cols]  # repeated constraint row: C*C^T is singular
    if case % 31 == 0:
        c = [0.0] * (rows * cols)
    if case % 37 == 0:
        c[0] = rng.choice([math.nan, math.inf, 1e200])
    s = vector(rng, rows, -3, 3)
    x0 = vector(rng, cols, -3, 3)
    equality = 1 if case % 4 == 0 else 0
    return [rows, cols, nce, nci] + eps + [limit, equality] + c + s + x0


def append_mathopt_api_requests(requests: list[str]) -> None:
    """Append every MathOpt API request with its own seed; append_mathopt_api_requests(r)."""
    rng = random.Random(20260928)
    append_checks(requests, rng)
    append_lines(requests)
    append_eval(requests, rng)
    append_stationary(requests, rng)
    append_backtrack(requests, rng)
    append_regularize(requests, rng)
    append_polish(requests, rng)
    for case in range(400):
        requests.append(line('mathopt_uzawa', uzawa_request(rng, case)))
