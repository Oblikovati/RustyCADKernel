"""Deterministic requests for the MathInteg/MathLin API-completion probe (tkmath_integ_lin_api_probe.cxx).

Tridiagonal matrices cover random, graded (1e-100..1e100), already-deflated (zero couplings),
repeated-diagonal, zero and single-entry shapes, with NaN/inf/-0.0 injected into one slot for the
public QL helpers; shift scales include zero, sign flips and a huge factor. Norm requests mix
zeros, subnormals, huge values and non-finite entries with invalid and valid tolerances. Spectral
requests use symmetric positive definite, indefinite, singular, zero and non-symmetric matrices on
shifted bounds. Singular-endpoint integration spans every integrand mode, reversed, empty and
infinite intervals, invalid tolerances and callback failures.
"""
import random

SCALES = [1.0, 1e-100, 1e-10, 1e10, 1e100]


def line(op: str, values: list) -> str:
    """Format one request with round-trip float text; line('integ_lin_norm', [1, 0])."""
    return op + ' ' + ' '.join(str(value) for value in values)


def tridiagonal(rng: random.Random, case: int, n: int) -> tuple[list[float], list[float]]:
    """Diagonal and coupling slots for one tridiagonal case; tridiagonal(rng, 0, 3)."""
    scale = rng.choice(SCALES)
    diag = [rng.uniform(-10, 10) * scale for _ in range(n)]
    sub = [rng.uniform(-10, 10) * scale for _ in range(n)]
    pattern = case % 6
    if pattern == 1:
        sub = [0.0 if i % 2 == 0 else value for i, value in enumerate(sub)]
    if pattern == 2:
        diag = [diag[0]] * n
    if pattern == 3:
        diag, sub = [0.0] * n, [0.0] * n
    if pattern == 4:
        sub = [value * 1e-20 for value in sub]
    return diag, sub


def append_tridiag(rng: random.Random, requests: list[str]) -> None:
    """QL helper requests: n mode slot shift_scale d[n] e[n]."""
    for case in range(300):
        n = 1 + case % 7
        mode = 0 if case % 3 else rng.randint(1, 4)
        diag, sub = tridiagonal(rng, case, n)
        shift_scale = rng.choice([1.0, 1.0, 0.0, -1.0, 1e-3, 1e300])
        requests.append(line('integ_lin_tridiag', [n, mode, rng.randrange(2 * n), shift_scale] + diag + sub))


def append_norm(rng: random.Random, requests: list[str]) -> None:
    """Norm/tolerance requests: n mode slot tol tolcode dim values[n]."""
    specials = [0.0, -0.0, 5e-324, 1e-300, 1e300, 1.7976931348623157e308, -3.0]
    for case in range(200):
        n = 1 + case % 6
        values = [rng.choice(specials) if rng.random() < 0.3 else rng.uniform(-100, 100) for _ in range(n)]
        mode = 0 if case % 4 else rng.randint(1, 4)
        tol = rng.choice([0.0, -1e-3, 1e-20, 1e-12, 0.5, -0.0])
        code = 0 if case % 5 else 1 + case % 2  # NaN / +inf tolerance
        requests.append(line('integ_lin_norm', [n, mode, rng.randrange(n), tol, code, rng.choice([0, 1, 3, 100])] + values))


def spectral_matrix(rng: random.Random, case: int, n: int) -> list[float]:
    """Row-major entries: SPD, indefinite, singular, zero or non-symmetric."""
    base = [[rng.uniform(-3, 3) for _ in range(n)] for _ in range(n)]
    pattern = case % 5
    if pattern == 4:
        return [value for row in base for value in row]
    if pattern == 3:
        return [0.0] * (n * n)
    matrix = [[sum(base[k][i] * base[k][j] for k in range(n)) for j in range(n)] for i in range(n)]
    if pattern == 1:
        matrix = [[value - (5.0 if i == j else 0.0) for j, value in enumerate(row)] for i, row in enumerate(matrix)]
    if pattern == 2:
        matrix = [[base[0][i] * base[0][j] for j in range(n)] for i in range(n)]  # rank one: singular
    return [value for row in matrix for value in row]


def append_spectral(rng: random.Random, requests: list[str]) -> None:
    """Spectral wrapper requests: n lower mode entries[n*n]."""
    for case in range(150):
        n = 1 + case % 5
        mode = 1 if case % 13 == 0 else 0
        requests.append(line('integ_lin_spectral', [n, rng.choice([0, 1, -2]), mode] + spectral_matrix(rng, case, n)))


def append_singular(rng: random.Random, requests: list[str]) -> None:
    """TanhSinhSingular requests: mode lower upper uppercode tolerance fail (uppercode 2 = +inf)."""
    for case in range(120):
        lower, upper = sorted([rng.uniform(-2, 2), rng.uniform(-2, 2)])
        if case % 7 == 1:
            lower, upper = upper, lower
        if case % 11 == 2:
            upper = lower
        code = 2 if case % 13 == 3 else 0
        tolerance = rng.choice([1e-10, 1e-6, 1e-3, 0.0, -1.0])
        fail = rng.choice([0, 0, 0, 1, 7, 40])
        requests.append(line('integ_lin_singular', [case % 10, lower, upper, code, tolerance, fail]))


def append_integ_lin_api_requests(requests: list[str]) -> None:
    """Append every integ_lin API request with its own seed; append_integ_lin_api_requests(r)."""
    rng = random.Random(20260928)
    append_tridiag(rng, requests)
    append_norm(rng, requests)
    append_spectral(rng, requests)
    append_singular(rng, requests)
