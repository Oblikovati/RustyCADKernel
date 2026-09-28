"""Deterministic BVH requests appended after the CSLib corpus.

Covers float and double boxes in 2D, 3D and 4D: invalid, inverted, flat and
point boxes, touching and nested pairs, parallel/denormal/signed-zero rays,
every Voronoi region of point-triangle projection plus degenerate triangles,
3D affine box transforms, and JSON dumps re-read from every kind of prefix.
Also IntFloor at NaN, infinities and the int range edges, quick and radix
sorts with tied centroids (serial and parallel, full and partial ranges,
valid and invalid scene boxes), binned SAH builds with 2..48 bins, main-axis
splits, coincident centroids, SAH and QBVH collapse, and BVH_Transform inverses
and projective box application.
"""
import random

DIMENSIONS = (2, 3, 4)
PRECISIONS = (0, 1)


def line(op: str, values: list) -> str:
    """Format one request with round-trip float text; line('bvh_int_floor', [1, 2])."""
    return op+' '+' '.join(str(value) for value in values)


def append_bvh_requests(requests: list[str]) -> None:
    """Append every BVH request with its own seed; append_bvh_requests(requests)."""
    rng = random.Random(20131220)
    append_box_requests(rng, requests)
    append_int_floor_requests(rng, requests)
    append_sort_requests(rng, requests)
    append_build_requests(rng, requests)
    append_transform_requests(rng, requests)


def point(rng: random.Random, scale: float = 10.0) -> list[float]:
    """Four coordinates in [-scale, scale]; point(rng, 1.0)."""
    return [rng.uniform(-scale, scale) for _ in range(4)]


def corners(rng: random.Random, case: int) -> list[float]:
    """Minimum and maximum corners in one of several shapes chosen by case."""
    low, high = point(rng), point(rng)
    ordered = [min(a, b) for a, b in zip(low, high)] + [max(a, b) for a, b in zip(low, high)]
    shape = case % 6
    if shape == 1: ordered[4:] = ordered[:4]  # single point
    if shape == 2: ordered[5] = ordered[1]; ordered[6] = ordered[2]  # segment
    if shape == 3: ordered[6] = ordered[2]  # flat
    if shape == 4: return high + low  # possibly inverted, often invalid
    if shape == 5: ordered = [round(value) for value in ordered]
    return ordered


def ray_direction(rng: random.Random, case: int) -> list[float]:
    """Direction with zero, signed-zero, denormal or random components."""
    direction = point(rng, 2.0)
    pattern = case % 7
    if pattern == 1: direction[1] = 0.0
    if pattern == 2: direction[0] = -0.0; direction[2] = 0.0
    if pattern == 3: direction[1] = 1e-310
    if pattern == 4: direction[0] = 1e-39
    if pattern == 5: direction = [0.0, 0.0, 0.0, 0.0]
    return direction


def triangle(rng: random.Random, case: int) -> list[float]:
    """Three nodes, regular or degenerate (coincident, collinear, tiny)."""
    nodes = point(rng) + point(rng) + point(rng)
    pattern = case % 5
    if pattern == 1: nodes[4:8] = nodes[0:4]
    if pattern == 2: nodes[8:12] = [2*b - a for a, b in zip(nodes[0:4], nodes[4:8])]
    if pattern == 3: nodes = [value*1e-9 for value in nodes]
    return nodes


def matrix(rng: random.Random, case: int) -> list[float]:
    """Column-major 4x4: identity, translation, zero, or random affine."""
    pattern = case % 4
    identity = [1.0 if row == column else 0.0 for column in range(4) for row in range(4)]
    if pattern == 0: return identity
    if pattern == 1: return identity[:12] + point(rng)[:3] + [1.0]
    if pattern == 2: return [0.0]*16
    affine = [rng.uniform(-3, 3) for _ in range(16)]
    affine[3] = affine[7] = affine[11] = 0.0; affine[15] = 1.0
    return affine


def append_box_requests(rng: random.Random, requests: list[str]) -> None:
    """Box, tool, ray, projection, transform and JSON queries."""
    for case in range(768):
        dimension, precision = DIMENSIONS[case % 3], PRECISIONS[(case // 3) % 2]
        flag = 1 if case % 17 == 0 else 0
        a, b = corners(rng, case), corners(rng, case // 6)
        if case % 5 == 1: b = a[:]  # identical boxes
        if case % 5 == 2: b = a[4:] + [value + 1 for value in a[4:]]  # touching corner
        probe = rng.choice([point(rng), a[:4], a[4:], [0.0, -0.0, 0.0, -0.0]])
        cut = rng.choice([-1, -1, 0, 1, 5, 14, 17, 26, 40, 60, rng.randint(0, 120)])
        values = [dimension, precision, flag] + a + b + probe + ray_direction(rng, case)
        requests.append(line('bvh_box', values + triangle(rng, case) + matrix(rng, case) + [cut]))


def append_int_floor_requests(rng: random.Random, requests: list[str]) -> None:
    """IntFloor of a / b in double and float, including NaN and range edges."""
    edges = [(0, 0), (1, 0), (-1, 0), (-7, 2), (7, 2), (-0.5, 1), (-1e-300, 1), (1e300, 1e-300),
             (2147483647.5, 1), (2147483648, 1), (-2147483648, 1), (-2147483648.5, 1),
             (-2147483649, 1), (16777217, 1), (-16777217, 1), (3e9, 1), (-3e9, 1), (1, 3), (-1, 3)]
    for a, b in edges:
        requests.append(line('bvh_int_floor', [a, b]))
    for _ in range(64):
        requests.append(line('bvh_int_floor', [rng.uniform(-1e6, 1e6), rng.choice([1, 3, 7, 1e-3])]))


def vertices(rng: random.Random, dimension: int, count: int, tied: bool) -> list[float]:
    """3*count vertices; tied inputs snap to a coarse grid so centroids repeat."""
    values = []
    for _ in range(3 * count * dimension):
        values.append(rng.randint(0, 3) * 0.5 if tied else rng.uniform(-50, 50))
    return values


def append_sort_requests(rng: random.Random, requests: list[str]) -> None:
    """Quick and radix sorts over full and partial ranges."""
    for case in range(192):
        dimension, precision = DIMENSIONS[case % 3], PRECISIONS[(case // 3) % 2]
        count = rng.choice([0, 1, 2, 5, 17, 40, 100])
        start, last = 0, -1
        if count > 2 and case % 4 == 3:
            start = rng.randint(0, count - 2); last = rng.randint(start, count - 1)
        axis, mode, parallel = case % 3, 1 if case % 11 == 0 else 0, (case // 6) % 2
        head = [dimension, precision, axis, start, last, mode, parallel, count]
        requests.append(line('bvh_sort', head + vertices(rng, dimension, count, case % 2 == 0)))


def clustered(rng: random.Random, dimension: int, count: int, case: int) -> list[float]:
    """Random, tied, coincident or tiny-extent vertices for builder cases."""
    pattern = case % 4
    if pattern == 0: return vertices(rng, dimension, count, False)
    if pattern == 1: return vertices(rng, dimension, count, True)
    if pattern == 2: return [1.25] * (3 * count * dimension)
    return [value * 1e-7 for value in vertices(rng, dimension, count, False)]


def append_build_requests(rng: random.Random, requests: list[str]) -> None:
    """Binned builds with every bin count, leaf size and depth policy."""
    for case in range(192):
        dimension, precision = DIMENSIONS[case % 3], PRECISIONS[(case // 3) % 2]
        bins = [0, 2, 4, 16, 32, 48][(case // 6) % 6]
        leaf, depth = rng.choice([1, 2, 5, 8]), rng.choice([1, 3, 32])
        count = rng.choice([0, 1, 2, 3, 10, 33, 64])
        head = [dimension, precision, bins, leaf, depth, (case // 36) % 2, count]
        requests.append(line('bvh_build', head + clustered(rng, dimension, count, case // 2)))


def append_transform_requests(rng: random.Random, requests: list[str]) -> None:
    """BVH_Transform inverses and projective application to 4D boxes."""
    for case in range(96):
        values = matrix(rng, case)
        if case % 8 == 7: values = [rng.uniform(-2, 2) for _ in range(16)]  # projective
        if case % 8 == 6: values[0:4] = values[4:8]  # singular, nonzero
        box = corners(rng, case)
        requests.append(line('bvh_transform', [case % 2] + values + box))
