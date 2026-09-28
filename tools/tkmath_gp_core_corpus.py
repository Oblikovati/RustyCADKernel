"""Deterministic gp_Vec, gp_Dir, gp_Pnt, gp_Lin, gp_Pln, gp_Ax2/gp_Ax3 and interpolator requests.

Covers null, underflowing (1e-300), subnormal-square (1e-160), huge and exact-axis vectors,
parallel, antiparallel and perpendicular pairs, out-of-range coordinate indices, negative and
zero tolerances and scales, points one ulp apart (std::equal_to<gp_Pnt>) and bit-identical
ones (std::hash<gp_Pnt>), points on a line (gp_Lin::Normal raises), plane equations with tied
and zero coefficients, frames whose X is parallel to the normal, every gp_Trsf construction
kind from tkmath_gp_transform_corpus, and interpolation parameters at, within
Precision::Confusion of, and beyond both ends.
"""
import math
import random

from tkmath_gp_transform_corpus import line, scalar, transform_spec, unit3

INDICES = [1, 2, 3, 1, 2, 3, 0, 4, -1]


def append_gp_core_requests(requests: list[str]) -> None:
    """Append every gp_core request; append_gp_core_requests(requests)."""
    rng = random.Random(20260928)
    for case in range(400):
        requests.append(line('gp3d_vec', vec_request(rng, case)))
    for case in range(300):
        requests.append(line('gp3d_dir', dir_request(rng, case)))
    for case in range(400):
        requests.append(line('gp3d_pnt', pnt_request(rng, case)))
    for case in range(400):
        requests.append(line('gp3d_lin', lin_request(rng, case)))
    for case in range(400):
        requests.append(line('gp3d_pln', pln_request(rng, case)))
    for case in range(400):
        requests.append(line('gp3d_frame', frame_request(rng, case)))
    for case in range(300):
        requests.append(line('gp3d_lerp', lerp_request(rng, case)))


def vector(rng: random.Random, base: list[float] | None = None) -> list[float]:
    """A random, null, tiny, huge, axis or base-related three-component vector."""
    pattern = rng.randrange(10)
    if pattern == 0:
        return [0.0, 0.0, 0.0]
    if pattern == 1:
        return rng.choice([[1e-300, 0.0, 0.0], [1e-160, -1e-160, 1e-160], [0.0, -0.0, 1e300]])
    if pattern == 2:
        return unit3(rng)
    if pattern in (3, 4) and base is not None:
        return [c * rng.choice([2.5, -1.0, 1.0, -0.5]) for c in base]
    return [rng.uniform(-5, 5) for _ in range(3)]


def direction(rng: random.Random, base: list[float] | None = None) -> list[float]:
    """A mostly valid raw direction: unit, random, parallel/perpendicular to base, or null."""
    pattern = rng.randrange(8)
    if pattern == 0:
        return [0.0, 0.0, 0.0]
    if pattern in (1, 2) and base is not None:
        return [c * rng.choice([3.0, -1.0]) for c in base]
    if pattern == 3 and base is not None:
        return perpendicular(base)
    return unit3(rng) if pattern < 6 else [rng.uniform(-3, 3) for _ in range(3)]


def perpendicular(base: list[float]) -> list[float]:
    """A vector orthogonal to base (or X when base is null)."""
    x, y, z = base
    candidate = [-y, x, 0.0] if abs(z) < max(abs(x), abs(y)) else [0.0, -z, y]
    return candidate if any(candidate) else [1.0, 0.0, 0.0]


def point(rng: random.Random, scale: float = 5.0) -> list[float]:
    """A uniform point, sometimes the origin or a huge one."""
    pattern = rng.randrange(10)
    if pattern == 0:
        return [0.0, 0.0, 0.0]
    if pattern == 1:
        return [1e300, -1e300, 0.5]
    return [rng.uniform(-scale, scale) for _ in range(3)]


def trsf(rng: random.Random, case: int) -> list[float]:
    """A gp_Trsf spec cycling through all fifteen construction kinds."""
    return transform_spec(rng, case % 15)


def vec_request(rng: random.Random, case: int) -> list[float]:
    """Four vectors, two scalars, an index, an axis origin/direction, an angle and a gp_Trsf."""
    a = vector(rng)
    b, c, d = vector(rng, a), vector(rng, a), vector(rng)
    s, t = scalar(rng), scalar(rng, 3)
    return a + b + c + d + [s, t, rng.choice(INDICES)] + point(rng) + direction(rng, a) + [
        rng.uniform(-7, 7)] + trsf(rng, case)


def dir_request(rng: random.Random, case: int) -> list[float]:
    """Three raw directions (null, parallel and perpendicular ones included), an index, a value."""
    v = direction(rng)
    w, u = direction(rng, v), direction(rng, v)
    return v + w + u + [rng.choice(INDICES), rng.choice([0.0, -0.0, 1.0, -2.0, 1e-160, rng.uniform(-3, 3)])]


def nudged(values: list[float], rng: random.Random) -> list[float]:
    """The same point, or one ulp (or two) away from it in one coordinate."""
    moved = list(values)
    index = rng.randrange(3)
    for _ in range(rng.choice([0, 1, 2])):
        moved[index] = math.nextafter(moved[index], math.inf)
    return moved


def pnt_request(rng: random.Random, case: int) -> list[float]:
    """Two points (possibly equal or ulp-close), a vector, weights, a scalar, an index, a gp_Trsf."""
    p = point(rng)
    q = nudged(p, rng) if case % 3 == 0 else point(rng)
    alpha = rng.choice([1.0, 0.5, -1.0, 0.0, rng.uniform(-3, 3)])
    beta = rng.choice([-alpha, 1.0, rng.uniform(-3, 3)])
    return p + q + vector(rng) + [alpha, beta, scalar(rng), rng.choice(INDICES)] + trsf(rng, case)


def lin_request(rng: random.Random, case: int) -> list[float]:
    """Line origin, test point (sometimes on the line), second origin, two directions, a gp_Trsf."""
    o, n = point(rng), direction(rng)
    p = point(rng) if case % 4 else [oc + 2.0 * nc for oc, nc in zip(o, n)]
    return o + p + point(rng) + n + direction(rng, n) + [rng.uniform(-7, 7), scalar(rng)] + trsf(rng, case)


def equation(rng: random.Random) -> list[float]:
    """Plane coefficients with ties, zeros and random values."""
    return rng.choice([
        [1.0, 1.0, 0.5, -4.0], [1.0, 0.5, 1.0, 2.0], [0.5, 1.0, 1.0, 3.0], [2.0, 2.0, 2.0, -1.0],
        [0.0, 0.0, 0.0, 1.0], [0.0, 0.0, 1.0, -5.0], [-3.0, 0.0, 0.0, 6.0],
        [rng.uniform(-3, 3) for _ in range(4)], [rng.uniform(-3, 3) for _ in range(4)]])


def pln_request(rng: random.Random, case: int) -> list[float]:
    """Plane origin/normal, points, a second direction, an equation, tolerances and a gp_Trsf."""
    o, n = point(rng), direction(rng)
    tolerances = [rng.choice([1e-7, 0.0, 10.0, -1.0]), rng.choice([1e-12, 0.1, 0.0])]
    return o + point(rng) + point(rng) + n + direction(rng, n) + equation(rng) + [
        rng.uniform(-7, 7), scalar(rng)] + tolerances + trsf(rng, case)


def frame_request(rng: random.Random, case: int) -> list[float]:
    """Frame origin, a point, normal, X and an edit direction (often parallel), angle, scale, gp_Trsf."""
    n = direction(rng)
    x = direction(rng, n)
    m = direction(rng, x if case % 2 else n)
    return point(rng) + point(rng) + n + x + m + [rng.uniform(-7, 7), scalar(rng)] + trsf(rng, case)


def lerp_request(rng: random.Random, case: int) -> list[float]:
    """Scalar and gp_XYZ end points, a parameter and two gp_Trsf ends."""
    t = rng.choice([0.0, 1.0, 1e-8, 1.0 - 1e-8, 0.5, 0.25, rng.uniform(-0.5, 1.5)])
    ends = [scalar(rng), scalar(rng), t] + vector(rng) + vector(rng)
    return ends + trsf(rng, case) + trsf(rng, case // 15)
