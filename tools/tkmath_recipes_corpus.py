"""Deterministic math_Recipes kernel requests and linear-solver dump requests.

Covers LU_Decompose (both overloads) on regular, singular, zero-row, rank-deficient,
pivot-threshold and NaN/Inf inputs, LU_Solve with shifted right-hand sides, LU_Invert;
SVD_Decompose/SVD_Solve on square, tall, rank-deficient and zero matrices; Jacobi on
symmetric, diagonal and non-symmetric inputs; DACTCL on skyline profiles including
singular pivots; and the Dump text of math_Gauss, math_GaussLeastSquare, math_SVD and
math_Jacobi.
"""
import random


def append_recipes_requests(requests: list[str]) -> None:
    """Append every request with its own seed; append_recipes_requests(requests)."""
    rng = random.Random(19971999)
    append_lu_requests(rng, requests)
    append_svd_requests(rng, requests)
    append_jacobi_requests(rng, requests)
    append_dactcl_requests(rng, requests)
    append_dump_requests(rng, requests)


def line(op: str, values: list) -> str:
    """Format one request with round-trip float text; line('recipes_lu', [1, 0])."""
    return op + ' ' + ' '.join(str(value) for value in values)


def square(rng: random.Random, n: int) -> list[float]:
    return [rng.uniform(-5, 5) for _ in range(n * n)]


def append_lu_requests(rng: random.Random, requests: list[str]) -> None:
    fixed = [
        (2, 1e-20, [0, 0, 0, 0]), (2, 1e-20, [1, 2, 2, 4]), (2, 1e-20, [0, 1, 1, 0]),
        (3, 1e-20, [1, 0, 0, 0, 0, 0, 0, 0, 1]), (1, 1e-20, [5]), (1, 10, [5]),
        (2, 1e-20, [1e308, 1e308, 1, 1]), (2, 1e-300, [1e-310, 1, 1, 1]),
        (3, 0.5, [4, 1, 0, 1, 0.2, 0, 0, 0, 3]),
    ]
    for n, tiny, cells in fixed:
        requests.append(line('recipes_lu', [n, tiny, *cells, *[rng.uniform(-3, 3) for _ in range(n)]]))
    for _ in range(96):
        n = rng.randint(1, 9)
        cells = square(rng, n)
        if rng.random() < 0.2:
            row = rng.randrange(n)
            cells[row * n:(row + 1) * n] = [value * 1e-22 for value in cells[row * n:(row + 1) * n]]
        rhs = [rng.choice([0.0, rng.uniform(-3, 3)]) for _ in range(n)]
        requests.append(line('recipes_lu', [n, rng.choice([1e-20, 1e-30, 1e-3]), *cells, *rhs]))


def append_svd_requests(rng: random.Random, requests: list[str]) -> None:
    fixed = [(2, 2, [0, 0, 0, 0]), (3, 2, [1, 2, 2, 4, 3, 6]), (1, 1, [-3]),
             (3, 3, [1, 0, 0, 0, 0, 0, 0, 0, -2]), (4, 1, [0, 3, 0, 4])]
    for m, n, cells in fixed:
        requests.append(line('recipes_svd', [m, n, *cells, *[rng.uniform(-2, 2) for _ in range(m)]]))
    for _ in range(64):
        n = rng.randint(1, 7)
        m = n + rng.randint(0, 4)
        cells = [rng.uniform(-5, 5) for _ in range(m * n)]
        if rng.random() < 0.25 and n > 1:
            for row in range(m):
                cells[row * n + n - 1] = cells[row * n] * 2
        requests.append(line('recipes_svd', [m, n, *cells, *[rng.uniform(-2, 2) for _ in range(m)]]))


def append_jacobi_requests(rng: random.Random, requests: list[str]) -> None:
    requests.append(line('recipes_jacobi', [2, 3, 0, 0, 3]))
    requests.append(line('recipes_jacobi', [3, 1, 1, 1, 1, 1, 1, 1, 1, 1]))
    requests.append(line('recipes_jacobi', [1, -4]))
    for index in range(64):
        n = rng.randint(1, 8)
        cells = square(rng, n)
        if index % 4 != 0:
            for i in range(n):
                for j in range(i):
                    cells[i * n + j] = cells[j * n + i]
        requests.append(line('recipes_jacobi', [n, *cells]))


def append_dactcl_requests(rng: random.Random, requests: list[str]) -> None:
    requests.append(line('recipes_dactcl', [1, 1e-20, 1, 4.0, 8.0]))
    requests.append(line('recipes_dactcl', [2, 1e-20, 1, 2, 0.0, 1.0, 1.0, 1.0, 2.0]))
    for index in range(80):
        neq = rng.randint(1, 10)
        heights = [rng.randint(1, j) for j in range(1, neq + 1)]
        entries = []
        for height in heights:
            entries += [rng.uniform(-1, 1) for _ in range(height - 1)]
            entries.append(rng.uniform(4, 9) if index % 5 else rng.choice([0.0, 1e-25, 3.0]))
        rhs = [rng.uniform(-5, 5) for _ in range(neq)]
        pivot = rng.choice([1e-20, 1e-20, 0.5, 10.0])
        requests.append(line('recipes_dactcl', [neq, pivot, *heights, *entries, *rhs]))


def append_dump_requests(rng: random.Random, requests: list[str]) -> None:
    requests.append(line('linear_dump', [2, 2, 1e-20, 0, 0, 0, 0]))
    requests.append(line('linear_dump', [2, 2, 1e-20, 0, 1, 1, 0]))
    requests.append(line('linear_dump', [2, 3, 1e-20, 1, 2, 3, 4, 5, 6]))
    for _ in range(32):
        rows = rng.randint(1, 5)
        cols = rng.choice([rows, rng.randint(1, 5)])
        cells = [rng.uniform(-5, 5) for _ in range(rows * cols)]
        requests.append(line('linear_dump', [rows, cols, rng.choice([1e-20, 1.0]), *cells]))
