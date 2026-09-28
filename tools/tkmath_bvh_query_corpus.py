"""Deterministic BVH phase 2 requests appended after the BVH core corpus.

Covers the linear (serial and parallel), spatial-median, sweep-plane and two-bin
builders on triangulations, box sets and indexed box sets with set, invalid,
point and widened scene boxes, and compares whole tree layouts, element orders,
SAH, the empty rebuild and the set-driven rebuild. Traversal requests log every
RejectNode metric and Accept call of a metric-ordered, pruned, branch-accepting
and stopping selector, a default-rule box selector, pair selectors on two trees
and on one tree with itself, BVH_Distance and BVH_PairDistance through their
sets (and without a set). Distance-field requests cover DirectionToNearestPoint
and DistanceToBox on every Voronoi region and degenerate triangles, and whole
voxel grids of geometries with clean and dirty flags, prebuilt trees, parallel
slices, custom top-level builders, closed boxes (signed) and a non-triangulation
object.
"""
import random

from tkmath_bvh_corpus import DIMENSIONS, PRECISIONS, clustered, corners, line, point, triangle, vertices


def append_bvh_query_requests(requests: list[str]) -> None:
    """Append every phase 2 BVH request with its own seed; append_bvh_query_requests(requests)."""
    rng = random.Random(20140911)
    append_builder_requests(rng, requests)
    append_traverse_requests(rng, requests)
    append_nearest_requests(rng, requests)
    append_field_requests(rng, requests)


def append_builder_requests(rng: random.Random, requests: list[str]) -> None:
    """Every builder on every set kind and scene box mode."""
    for case in range(288):
        dimension, precision = DIMENSIONS[case % 3], PRECISIONS[(case // 3) % 2]
        kind = (case // 6) % 4
        set_kind = 0
        if precision == 0 and case % 5 >= 3:  # BVH_BoxSet::Center returns double upstream
            set_kind = case % 5 - 2
        mode = (case // 24) % 4
        leaf = rng.choice([1, 2, 3, 5, 8] if kind == 0 else [0, 1, 2, 3, 5, 8])
        depth = rng.choice([1, 3, 5, 32])
        count = rng.choice([0, 1, 2, 3, 7, 16, 33, 64])
        head = [dimension, precision, kind, leaf, depth, rng.randint(0, 1), set_kind, mode, count]
        requests.append(line('bvh_builders', head + clustered(rng, dimension, count, case // 2)))


def append_traverse_requests(rng: random.Random, requests: list[str]) -> None:
    """Logged single/pair traversals and distance tools over two built meshes."""
    for case in range(192):
        dimension, precision = DIMENSIONS[case % 3], PRECISIONS[(case // 3) % 2]
        kind = [4, 0, 2, 1][(case // 6) % 4]
        count1, count2 = rng.choice([0, 1, 2, 5, 12, 30]), rng.choice([0, 1, 3, 9, 20])
        head = [dimension, precision, kind, rng.choice([1, 2, 4]), count1, count2,
                rng.choice([0, 0, 0, 1, 3, 10])]
        limits = [rng.choice([0.0, 25.0, 400.0, 3000.0, 1e30]), rng.choice([-1.0, 0.0, 4.0, 100.0]),
                  rng.choice([0.0, 1.0, 50.0, 1e6])]
        tied = case % 2 == 0
        meshes = vertices(rng, dimension, count1, tied) + vertices(rng, dimension, count2, tied)
        requests.append(line('bvh_traverse', head + point(rng, 30.0) + limits + meshes))


def nearest_point(rng: random.Random, nodes: list[float], case: int) -> list[float]:
    """Query point: random, on a vertex, on an edge middle, or at the centroid."""
    a, b, c = nodes[0:4], nodes[4:8], nodes[8:12]
    pattern = case % 5
    if pattern == 1: return a[:]
    if pattern == 2: return [(x + y) / 2 for x, y in zip(a, b)]
    if pattern == 3: return [(x + y + z) / 3 for x, y, z in zip(a, b, c)]
    return point(rng)


def append_nearest_requests(rng: random.Random, requests: list[str]) -> None:
    """DirectionToNearestPoint and DistanceToBox for N = 3 and 4."""
    for case in range(384):
        dimension, precision = (3, 4)[case % 2], PRECISIONS[(case // 2) % 2]
        nodes = triangle(rng, case // 4)
        probe = nearest_point(rng, nodes, case // 3)
        box = corners(rng, case // 5)
        requests.append(line('bvh_nearest', [dimension, precision] + probe + nodes + box))


def cube(rng: random.Random, dimension: int) -> list[float]:
    """Twelve outward-facing triangles of a random axis-aligned box (W = 0)."""
    low = [rng.uniform(-8, 0) for _ in range(3)]
    high = [value + rng.uniform(1, 8) for value in low]
    corner = lambda i: [high[k] if i >> k & 1 else low[k] for k in range(3)]
    faces = [(0, 2, 3, 1), (4, 5, 7, 6), (0, 1, 5, 4), (2, 6, 7, 3), (0, 4, 6, 2), (1, 3, 7, 5)]
    values = []
    for a, b, c, d in faces:
        for triple in ((a, b, c), (a, c, d)):
            for index in triple:
                values += corner(index) + [0.0] * (dimension - 3)
    return values


def field_object(rng: random.Random, dimension: int, case: int) -> tuple[int, list[float]]:
    """One triangulation: a closed box, an empty mesh, or random triangles."""
    pattern = rng.randint(0, 3)
    if pattern == 0: return 12, cube(rng, dimension)
    if pattern == 1 and case % 3 == 0: return 0, []
    count = rng.choice([1, 4, 10])
    return count, [rng.uniform(-10, 10) for _ in range(3 * count * dimension)]


def append_field_requests(rng: random.Random, requests: list[str]) -> None:
    """Distance-field grids; parallel slices only once every tree is prebuilt."""
    for case in range(48):
        dimension, precision = (3, 4)[case % 2], PRECISIONS[(case // 2) % 2]
        prebuild = case % 3 == 0
        objects = [field_object(rng, dimension, case) for _ in range(rng.choice([0, 1, 2, 3]))]
        head = [dimension, precision, rng.choice([2, 4, 8, 16, 17, 20]), rng.randint(0, 1),
                int(prebuild and case % 2 == 0), int(case % 10 != 0), int(case % 7 != 0),
                int(prebuild), int(case % 4 == 1), case % 3, len(objects)]
        counts = [count for count, _ in objects]
        coordinates = [value for _, values in objects for value in values]
        requests.append(line('bvh_field', head + counts + coordinates))
