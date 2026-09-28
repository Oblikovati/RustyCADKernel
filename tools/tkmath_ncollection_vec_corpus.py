"""Deterministic NCollection_Vec2/Vec3/Vec4 and NCollection_Mat3/Mat4 requests.

Vectors run for float, double and int elements with signed zeros, ties, NaN-free extremes,
zero vectors (Normalize keeps them) and float values that round when narrowed. Integer
requests keep divisors non-zero and products within int range, since overflow and division
by zero are undefined natively. Matrices run for float and double with singular, rotation,
projective and random entries, zero scalars and translations.
"""
import math
import random

SPECIAL = [0.0, -0.0, 1.0, -1.0, 0.1, 1e-30, -1e30, 3.4e38, 1e-45, 2.5]


def line(op: str, values: list) -> str:
    """Format one request with round-trip float text; line('ncollection_vec', [0, 1])."""
    return op+' '+' '.join(str(value) for value in values)


def append_ncollection_vec_requests(requests: list[str]) -> None:
    """Append every NCollection vector/matrix request; append_ncollection_vec_requests(r)."""
    rng = random.Random(20130530)
    for case in range(600):
        kind = case % 3
        requests.append(line('ncollection_vec', [kind] + vector_values(rng, kind, case)))
    for case in range(400):
        requests.append(line('ncollection_mat', [case % 2] + matrix_values(rng, case)))


def vector_values(rng: random.Random, kind: int, case: int) -> list[float]:
    """Two four-component vectors, a scalar and an interpolation parameter."""
    if kind == 2:
        values = [float(rng.randint(-40, 40)) for _ in range(8)]
        values[4:8] = [value or 3.0 for value in values[4:8]]
        return values + [float(rng.choice([-3, -2, -1, 1, 2, 5])), float(rng.randint(-2, 3))]
    pick = (lambda: rng.choice(SPECIAL)) if case % 5 == 0 else (lambda: rng.uniform(-50, 50))
    values = [pick() for _ in range(8)] + [rng.choice([0.0, 3.0, -0.5, rng.uniform(-4, 4)]), rng.uniform(-1, 2)]
    if case % 7 == 1:
        values[0:4] = [0.0, 0.0, 0.0, 0.0]
    if case % 7 == 2:
        values[4:8] = values[0:4]
    return values


def rotation(angle: float) -> list[float]:
    """Column-major 4x4 rotation about Z with a translation column."""
    c, s = math.cos(angle), math.sin(angle)
    return [c, s, 0, 0, -s, c, 0, 0, 0, 0, 1, 0, 1.5, -2.0, 0.25, 1]


def matrix(rng: random.Random, pattern: int) -> list[float]:
    """General, rotation, singular, projective or special column-major entries."""
    if pattern == 1:
        return rotation(rng.uniform(-3, 3))
    if pattern == 2:
        values = [rng.uniform(-5, 5) for _ in range(16)]
        values[4:8] = [2 * value for value in values[0:4]]
        return values
    if pattern == 3:
        values = rotation(rng.uniform(-3, 3))
        values[3], values[7], values[11] = rng.uniform(-1, 1), rng.uniform(-1, 1), rng.uniform(-1, 1)
        return values
    if pattern == 4:
        return [rng.choice(SPECIAL[:6]) for _ in range(16)]
    return [rng.uniform(-10, 10) for _ in range(16)]


def matrix_values(rng: random.Random, case: int) -> list[float]:
    """Two matrices, a vector and a scalar."""
    first, second = matrix(rng, case % 5), matrix(rng, (case // 5) % 5)
    vector = [rng.uniform(-3, 3) for _ in range(4)]
    scalar = rng.choice([0.0, 2.0, -0.5, rng.uniform(-4, 4)])
    return first + second + vector + [scalar]
