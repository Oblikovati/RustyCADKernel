"""Deterministic MathSys::Utils requests for the MathSys API-completion probe.

Fixed-size eliminations and the symmetric eigen solve see random, symmetric, diagonal,
rank-deficient, zero-row, tiny/huge-scale and NaN/infinite Jacobians; the merit and Armijo
checks see negative, zero, NaN and positive derivatives and norms. Box helpers see valid,
inverted, degenerate, inactive and non-finite bounds with points inside, outside and NaN.
The linear-algebra helpers see square, tall and wide systems with dependent or zero
columns and non-finite entries. The drivers and finalizers run the modern-system fixture
with every combination of the two optional bound pointers.
"""
import random

NAN = float('nan')
INF = float('inf')


def line(op: str, values: list) -> str:
    """Format one request with round-trip float text; line('mathsys_small_box', [1, 2])."""
    return op + ' ' + ' '.join(str(value) for value in values)


def append_mathsys_api_requests(requests: list[str]) -> None:
    """Append every MathSys API request with its own seed; append_mathsys_api_requests(r)."""
    rng = random.Random(20260928)
    for case in range(480):
        requests.append(line('mathsys_small_linear', small_linear(rng, case)))
    for case in range(360):
        requests.append(line('mathsys_small_box', small_box(rng, case)))
    for case in range(360):
        requests.append(line('mathsys_linear_api', linear_api(rng, case)))
    for case in range(420):
        requests.append(line('mathsys_finish_api', finish_api(rng, case)))


def jacobian(rng: random.Random, case: int) -> list[list[float]]:
    """4x4 Jacobian shaped by the case pattern (symmetric, diagonal, singular, scaled, non-finite)."""
    j = [[rng.uniform(-5, 5) for _ in range(4)] for _ in range(4)]
    pattern = case % 12
    if pattern == 1:
        j = [[j[min(r, c)][max(r, c)] for c in range(4)] for r in range(4)]
    elif pattern == 2:
        j = [[j[r][c] if r == c else 0.0 for c in range(4)] for r in range(4)]
    elif pattern == 3:
        j[1] = list(j[0])
    elif pattern == 4:
        j[rng.randrange(4)] = [0.0] * 4
    elif pattern == 5:
        scale = rng.choice([1e-300, 1e-160, 1e160, 1e300])
        j = [[v * scale for v in row] for row in j]
    elif pattern == 6:
        j[rng.randrange(4)][rng.randrange(4)] = rng.choice([NAN, INF, -INF])
    elif pattern == 7:
        j = [[1.0, 1.0 + 1e-15, 0.0, 0.0], [1.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1e-14, 0.0], [0.0, 0.0, 0.0, 1.0]]
    elif pattern == 8:
        j = [[2.0, 1e-20, 0.0, 0.0], [1e-20, -2.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]]
    elif pattern == 9:
        j = [[1e-16 * rng.choice([1, -1]) for _ in range(4)] for _ in range(4)]
    return j


def small_linear(rng: random.Random, case: int) -> list[float]:
    """16 Jacobian entries, 4 residuals, 4 steps, then tol init current trial step_tol."""
    j = jacobian(rng, case)
    f = [rng.uniform(-10, 10) for _ in range(4)]
    if case % 7 == 0:
        f[rng.randrange(4)] = rng.choice([NAN, INF, 1e300, 0.0])
    if case % 11 == 0:
        f = [0.0] * 4
    step = [rng.uniform(-2, 2) for _ in range(4)]
    if case % 5 == 0:
        step = [-v for v in f]
    if case % 13 == 0:
        step[rng.randrange(4)] = NAN
    tol = rng.choice([1e-15, 1e-15, 1e-8, 1e-3, 0.0, 0.5, -1.0])
    init = rng.choice([91.0, NAN, -0.0])
    current = rng.choice([1.0, 0.0, -1.0, NAN, 1e-300, rng.uniform(0.1, 10)])
    trial = rng.choice([0.0, 0.5, 0.999, 1.0, 2.0, NAN, rng.uniform(0, 3)])
    step_tol = rng.choice([0.0, 1e-3, 1.0, 10.0, NAN])
    return [v for row in j for v in row] + f + step + [tol, init, current, trial, step_tol]


def small_box(rng: random.Random, case: int) -> list[float]:
    """lo[4] hi[4] x[4] has soft ratio step f_tol x_tol limit max_step_ratio soft_ext."""
    lo = [rng.uniform(-10, 10) for _ in range(4)]
    width = [rng.choice([0.0, 0.5, 3.0, 50.0, rng.uniform(0, 20)]) for _ in range(4)]
    hi = [a + w for a, w in zip(lo, width)]
    pattern = case % 9
    if pattern == 1:
        k = rng.randrange(4)
        lo[k], hi[k] = hi[k] + 1.0, lo[k]
    elif pattern == 2:
        (lo if rng.random() < 0.5 else hi)[rng.randrange(4)] = rng.choice([NAN, INF, -INF])
    elif pattern == 3:
        lo, hi = [-1e308] * 4, [1e308] * 4
    x = [rng.uniform(-30, 30) for _ in range(4)]
    if case % 6 == 0:
        x[rng.randrange(4)] = NAN
    has = 0.0 if case % 5 == 0 else 1.0
    soft = float(case % 2)
    ratio = rng.choice([0.0, 1e-4, 0.5, 2.0, NAN])
    step = rng.choice([0.5, 0.1, 0.0, -1.0, NAN, 3.0])
    options = [rng.choice([1e-10, 0.0, -1.0, NAN, INF]), rng.choice([1e-16, 0.0, NAN, 1e-8]),
               rng.choice([100, 0, 1]), rng.choice([0.5, 0.0, INF, 2.0]), rng.choice([1e-4, 0.0, -1e-4, NAN])]
    if case % 3 == 0:
        options = [1e-10, 1e-16, 100, 0.5, 1e-4]
    return lo + hi + x + [has, soft, ratio, step] + options


def linear_api(rng: random.Random, case: int) -> list[float]:
    """m n tol init, row-major m x n matrix, then m right-hand-side values."""
    m, n = rng.randint(1, 5), rng.randint(1, 5)
    if case % 3 == 0:
        n = rng.randint(1, m)
    a = [[rng.uniform(-10, 10) for _ in range(n)] for _ in range(m)]
    pattern = case % 10
    if pattern == 1 and n > 1:
        for row in a:
            row[1] = 2.0 * row[0]
    elif pattern == 2:
        for row in a:
            row[rng.randrange(n)] = 0.0
    elif pattern == 3:
        a[rng.randrange(m)][rng.randrange(n)] = rng.choice([NAN, INF])
    elif pattern == 4:
        k = rng.randrange(n)
        for row in a:
            row[k] *= rng.choice([1e-150, 1e150])
    elif pattern == 5:
        a = [[0.0] * n for _ in range(m)]
    b = [rng.uniform(-10, 10) for _ in range(m)]
    if case % 17 == 0:
        b[rng.randrange(m)] = NAN
    tol = rng.choice([64.0 * 2.220446049250313e-16, 1e-10, 1e-3, 0.5])
    init = rng.choice([91.0, 0.0])
    return [m, n, tol, init] + [v for row in a for v in row] + b


def finish_api(rng: random.Random, case: int) -> list[float]:
    """mode m fail target scale x y f_tol limit lo hi mask tol initial_norm."""
    mode = case % 7
    m = rng.choice([1, 2, 3]) if mode in (0, 2, 5, 6) else rng.choice([2, 3])
    fail = rng.choice([0, 0, 0, 1, 2, 3, 5, 9])
    target = rng.choice([0.0, 1.0, 2.0, rng.uniform(-3, 3)])
    scale = rng.choice([1.0, 1e-8, 1e8, -2.0])
    x, y = rng.uniform(-4, 4), rng.uniform(-4, 4)
    ft = rng.choice([1e-10, 1e-6, 1e-3, 0.0])
    limit = rng.choice([1, 3, 20, 100, 0])
    lo = rng.choice([-5.0, -1.0, 0.0, 0.5])
    hi = rng.choice([5.0, 1.0, 0.0, -2.0])
    mask = case // 7 % 4
    tol = rng.choice([1e-10, 1e-6, 1e-3])
    initial = rng.choice([0.0, 1.0, 100.0])
    return [mode, m, fail, target, scale, x, y, ft, limit, lo, hi, mask, tol, initial]
