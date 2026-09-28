"""Deterministic math_IntegerVector, vector/matrix dump and math_Matrix-member requests.

Covers the integer instantiation of math_VectorBase (truncating double conversions,
reciprocal-based Divided, zero divisors, mixed matrix products, empty and single
element vectors), the double vector members added with it, and the in-place and
two-operand math_Matrix members: shifted bounds, non-square shapes that raise,
out-of-range block writes and the bound-dependent in-place Transpose.
"""
import random


def append_vector_base_requests(requests: list[str]) -> None:
    """Append every request with its own seed; append_vector_base_requests(requests)."""
    rng = random.Random(20230901)
    append_int_vector_requests(rng, requests)
    append_real_vector_requests(rng, requests)
    append_matrix_requests(rng, requests)


def line(op: str, values: list) -> str:
    """Format one request with round-trip float text; line('int_vector', [1, 0])."""
    return op + ' ' + ' '.join(str(value) for value in values)


def append_int_vector_requests(rng: random.Random, requests: list[str]) -> None:
    """Random and edge integer vectors with scalar, divisor and an n x 2 matrix."""
    fixed = [(1, 1, [5], 3, 0), (2, -2, [3, 4], -1, 1), (3, 5, [0, 0, 0], 2, -1),
             (4, 0, [-7, 7, 1, -1], 2, 2), (5, 10, [9, -9, 4, -4, 0], -3, -2)]
    for n, lower, items, scalar, divisor in fixed:
        matrix = [rng.uniform(-3, 3) for _ in range(2 * n)]
        requests.append(line('int_vector', [n, lower, *items, scalar, divisor, *matrix]))
    for _ in range(96):
        n = rng.randint(1, 40)
        items = [rng.randint(-50, 50) for _ in range(n)]
        matrix = [rng.choice([rng.uniform(-3, 3), rng.randint(-3, 3), 0.5, -0.5]) for _ in range(2 * n)]
        requests.append(line('int_vector', [n, rng.randint(-5, 5), *items,
                                            rng.randint(-6, 6), rng.choice([0, 1, -1, 2, 3, -4]), *matrix]))


def append_real_vector_requests(rng: random.Random, requests: list[str]) -> None:
    """Double vectors whose dump exercises %g formatting ranges."""
    specials = [0.0, -0.0, 1e-5, 123456.0, 1234567.0, 1e300, -2.5e-300, 0.1, 1.0 / 3.0]
    requests.append(line('real_vector_extra', [len(specials), -3, *specials, 2.5]))
    requests.append(line('real_vector_extra', [1, 0, 7.0, 0.0]))
    for _ in range(64):
        n = rng.randint(1, 36)
        items = [rng.uniform(-1e3, 1e3) * 10 ** rng.randint(-8, 8) for _ in range(n)]
        requests.append(line('real_vector_extra', [n, rng.randint(-4, 4), *items, rng.uniform(-5, 5)]))


def append_matrix_requests(rng: random.Random, requests: list[str]) -> None:
    """Square and rectangular matrices with shifted bounds and in/out-of-range blocks."""
    shapes = [(1, 1), (2, 2), (3, 3), (2, 3), (3, 2), (4, 4), (1, 4), (9, 9)]
    for rows, cols in shapes:
        for row_lower, col_lower in [(1, 1), (1, 3), (3, 1), (-2, 0), (0, -2)]:
            cells = [rng.uniform(-10, 10) for _ in range(rows * cols)]
            scalar = rng.choice([rng.uniform(-4, 4), 0.0, 1e-17, -1.0])
            block_row = rng.randint(row_lower - 1, row_lower + rows)
            block_col = rng.randint(col_lower - 1, col_lower + cols)
            requests.append(line('matrix_extra', [rows, cols, row_lower, col_lower, *cells,
                                                  scalar, block_row, block_col]))
