"""Deterministic Bnd differential requests for tools/verify_tkmath_numeric.py.

Usage: append_bnd_requests(requests) after the Convert requests.
Covers Bnd_OBB ReBuild (one, two and many points; DiTO and the optimal BVH path; with
and without tolerances; ties from box corners and grids; collinear, coplanar, duplicate,
tiny and huge clouds; mismatched tolerance bounds), the OBB queries and Add/Enlarge/
SetVoid/Position/DumpJson; Bnd_BoundSortBox in every initialization mode with void,
open, degenerate and large boxes, grid resolutions 8 to 32 and box/plane Compare order;
Bnd_Tools; the remaining Bnd_Box/Bnd_Box2d members; and the transformed IsOut/IsIn of
Bnd_B2/Bnd_B3 (double and float) for every transform form, plus their array constructors,
SetCenter/SetHSize overloads, Center/HSize, Add and Clear.
Every request stays inside the native defined domain: no empty point list, no null
direction and no zero scale.
"""
from __future__ import annotations
import math
import random


def line(op: str, values: list) -> str:
    """Format one request with round-trip float text; line('obb_build', [0, 0, 1, 1, 1, 0, 0, 0])."""
    return op+' '+' '.join(str(value) for value in values)


def append_bnd_requests(requests: list[str]) -> None:
    """Append every Bnd request with its own seed; append_bnd_requests(requests)."""
    rng = random.Random(20170419)
    append_obb_build_requests(rng, requests)
    append_obb_pair_requests(rng, requests)
    append_sortbox_requests(rng, requests)
    append_box_extra_requests(rng, requests)
    append_compact_requests(rng, requests)
    append_compact_array_requests(random.Random(20260928), requests)


def rotation(rng: random.Random) -> list[list[float]]:
    """Rows of a random rotation matrix from a normalized quaternion."""
    w, x, y, z = (rng.gauss(0, 1) for _ in range(4))
    n = math.sqrt(w*w+x*x+y*y+z*z)
    w, x, y, z = w/n, x/n, y/n, z/n
    return [[1-2*(y*y+z*z), 2*(x*y-z*w), 2*(x*z+y*w)],
            [2*(x*y+z*w), 1-2*(x*x+z*z), 2*(y*z-x*w)],
            [2*(x*z-y*w), 2*(y*z+x*w), 1-2*(x*x+y*y)]]


def apply(matrix: list[list[float]], point: list[float], shift: list[float]) -> list[float]:
    """matrix * point + shift."""
    return [sum(matrix[i][j]*point[j] for j in range(3))+shift[i] for i in range(3)]


def cloud(rng: random.Random, kind: int, count: int) -> list[list[float]]:
    """A point cloud of the given shape: 0 random, 1 rotated box corners, 2 integer grid,
    3 collinear, 4 coplanar, 5 duplicates, 6 elongated gaussian, 7 sphere, 8 tiny, 9 huge."""
    matrix, shift = rotation(rng), [rng.uniform(-50, 50) for _ in range(3)]
    if kind == 1:
        half = [rng.choice([1.0, 2.5, rng.uniform(0.1, 9)]) for _ in range(3)]
        corners = [[(1 if i & 1 else -1)*half[0], (1 if i & 2 else -1)*half[1], (1 if i & 4 else -1)*half[2]]
                   for i in range(8)]
        if rng.random() < 0.3:
            return corners
        return [apply(matrix, corner, shift) for corner in corners]
    if kind == 2:
        side = max(2, round(count ** (1/3)))
        return [[float(i), float(j), float(k)] for i in range(side) for j in range(side) for k in range(side)]
    if kind == 3:
        direction, origin = [rng.gauss(0, 1) for _ in range(3)], shift
        return [[origin[i]+t*direction[i] for i in range(3)] for t in (rng.uniform(-5, 5) for _ in range(count))]
    if kind == 4:
        return [apply(matrix, [rng.uniform(-4, 4), rng.uniform(-2, 2), 0.0], shift) for _ in range(count)]
    if kind == 5:
        base = [rng.uniform(-3, 3) for _ in range(3)]
        return [list(base) for _ in range(count)]
    if kind == 6:
        return [apply(matrix, [rng.gauss(0, 10), rng.gauss(0, 2), rng.gauss(0, 0.5)], shift) for _ in range(count)]
    if kind == 7:
        points = []
        for _ in range(count):
            v = [rng.gauss(0, 1) for _ in range(3)]
            n = math.sqrt(sum(c*c for c in v)) or 1.0
            points.append([3*c/n+s for c, s in zip(v, shift)])
        return points
    scale = 1e-9 if kind == 8 else 1e6
    return [[rng.uniform(-1, 1)*scale for _ in range(3)] for _ in range(count)]


def obb_build(rng: random.Random, points: list[list[float]], optimal: int, tolerances: list[float] | None,
              lower: int = 1, tolerance_lower: int | None = None) -> str:
    """One obb_build request over the points and optional tolerances."""
    tolerance_lower = lower if tolerance_lower is None else tolerance_lower
    values = [optimal, 0 if tolerances is None else 1, lower, tolerance_lower, len(points)]
    for index, point in enumerate(points):
        values += point
        if tolerances is not None:
            values.append(tolerances[index])
    return line('obb_build', values)


def append_obb_build_requests(rng: random.Random, requests: list[str]) -> None:
    """ReBuild over every cloud kind, size, mode and tolerance option."""
    requests.append(obb_build(rng, [[1, 2, 3], [3, 2, 1], [2, 3, 1], [1, 3, 2], [2, 1, 3]], 1, None))
    for count in [1, 2]:
        for case in range(60):
            points = [[rng.uniform(-9, 9) for _ in range(3)] for _ in range(count)]
            if case % 6 == 1:
                points[-1] = list(points[0])
            if case % 6 == 2:
                points[-1] = [c+rng.choice([1e-8, 5e-8, 1e-7, 2e-7]) for c in points[0]]
            if case % 6 == 3:
                points[-1] = [points[0][0], points[0][1], points[0][2]+rng.uniform(-3, 3)]
            tolerances = None if case % 2 else [rng.choice([0.0, 1e-7, 0.01, 0.5]) for _ in points]
            requests.append(obb_build(rng, points, int(case % 3 == 0), tolerances, rng.choice([1, 0, -2])))
    for case in range(700):
        kind = case % 10
        count = rng.choice([3, 4, 5, 8, 9, 16, 27, 40, 64, 150])
        points = cloud(rng, kind, count)
        tolerant = case % 4 == 1
        tolerances = [rng.choice([0.0, 1e-7, 1e-3, rng.uniform(0, 0.2)]) for _ in points] if tolerant else None
        lower = rng.choice([1, 1, 0, -3, 7])
        requests.append(obb_build(rng, points, (case // 10) % 2, tolerances, lower))
    for case in range(12):
        points = cloud(rng, case % 10, 9)
        tolerances = [0.01]*len(points)
        requests.append(obb_build(rng, points, case % 2, tolerances, 1, 0))
        requests.append(obb_build(rng, points[:2], case % 2, tolerances[:2], 1, 5))
    for optimal in [0, 1]:
        requests.append(obb_build(rng, cloud(rng, 0, 400), optimal, None))
        requests.append(obb_build(rng, cloud(rng, 6, 300), optimal, [rng.uniform(0, 0.05) for _ in range(300)]))


def obb_spec(rng: random.Random, near: list[float] | None = None, bigger: bool = False) -> list:
    """Center, three (possibly non-orthogonal) axes, three half sizes and the AA flag."""
    if rng.random() < 0.2:
        axes = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
    elif rng.random() < 0.1:
        axes = [[rng.uniform(-1, 1)+0.1*k for k in range(3)] for _ in range(3)]
    else:
        axes = rotation(rng)
    center = near if near is not None else [rng.uniform(-5, 5) for _ in range(3)]
    if near is not None:
        center = [c+rng.uniform(-2, 2) for c in center]
    half = [rng.choice([rng.uniform(0.1, 3), rng.uniform(0.1, 3), 0.0]) for _ in range(3)]
    if bigger:
        half = [h+8 for h in half]
    if rng.random() < 0.08:
        half[rng.randrange(3)] = -1.0
    aa = 1 if (axes[0] == [1.0, 0.0, 0.0] and rng.random() < 0.7) or rng.random() < 0.05 else 0
    return center+[c for axis in axes for c in axis]+half+[aa]


def box_spec(rng: random.Random, low: float = -5, high: float = 5, size: float = 3) -> list:
    """Min corner, max corner, gap and flags (1..32 open sides, 64 void)."""
    corner = [rng.uniform(low, high) for _ in range(3)]
    extent = [rng.choice([0.0, rng.uniform(0, size), rng.uniform(0, size)]) for _ in range(3)]
    gap = rng.choice([0.0, 0.0, 1e-7, rng.uniform(0, 0.5)])
    roll = rng.random()
    flags = 0 if roll < 0.75 else (64 if roll < 0.85 else rng.choice([1, 2, 4, 8, 16, 32, 3, 63, 65]))
    return corner+[c+e for c, e in zip(corner, extent)]+[gap, flags]


def append_obb_pair_requests(rng: random.Random, requests: list[str]) -> None:
    """Pairwise separating-axis queries, containment, Add, Enlarge and conversions."""
    for case in range(700):
        first = obb_spec(rng)
        mode = case % 4
        second = obb_spec(rng, first[:3] if mode else None, bigger=mode == 3)
        if mode == 2:
            second = first[:12]+[h*0.5 for h in first[12:15]]+[first[15]]
        point = [rng.uniform(-6, 6) for _ in range(3)]
        enlarge = rng.choice([0.0, 0.25, -0.5, 1e-7])
        requests.append(line('obb_pair', first+second+point+[enlarge]+box_spec(rng)))


def sort_boxes(rng: random.Random, count: int) -> list[list]:
    """Small, degenerate, large, open and void boxes spread over [0, 100]^3."""
    boxes = []
    for _ in range(count):
        roll = rng.random()
        if roll < 0.08:
            boxes.append(box_spec(rng, 0, 40, 90))
        elif roll < 0.15:
            corner = [rng.uniform(0, 100) for _ in range(3)]
            boxes.append(corner+corner+[0.0, 0])
        else:
            boxes.append(box_spec(rng, 0, 100, rng.choice([2, 8, 20])))
    return boxes


def append_sortbox_requests(rng: random.Random, requests: list[str]) -> None:
    """Every initialization mode, grid resolution and query kind, results in order."""
    for case in range(260):
        mode = case % 4
        count = rng.choice([1, 2, 4, 7, 12, 40, 101, 130]) if case < 250 else rng.choice([1001, 1100])
        boxes = sort_boxes(rng, count)
        roll = rng.random()
        if roll < 0.15:
            enclosing = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 64]
        elif roll < 0.35:
            enclosing = [20.0, 20.0, 20.0, 60.0, 60.0, 60.0, 0.0, 0]
        else:
            enclosing = [-1.0, -2.0, -3.0, 130.0, 131.0, 132.0, rng.choice([0.0, 0.5]), 0]
        queries = [box_spec(rng, -10, 110, rng.choice([1, 10, 60])) for _ in range(rng.randint(1, 6))]
        planes = [[rng.uniform(0, 100) for _ in range(3)]+rng.choice([[0, 0, 1], [1, 0, 0], [rng.gauss(0, 1)+0.01, rng.gauss(0, 1), 1.0]])
                  for _ in range(rng.randint(0, 3))]
        values = [mode, count]+[c for b in boxes for c in b]+enclosing+[len(queries)]
        values += [c for q in queries for c in q]+[len(planes)]+[c for p in planes for c in p]
        requests.append(line('sortbox', values))


def box2_spec(rng: random.Random) -> list:
    """Min corner, max corner, gap and flags (1..8 open sides, 16 void)."""
    corner = [rng.uniform(-5, 5) for _ in range(2)]
    extent = [rng.choice([0.0, rng.uniform(0, 3)]) for _ in range(2)]
    roll = rng.random()
    flags = 0 if roll < 0.7 else (16 if roll < 0.8 else rng.choice([1, 2, 4, 8, 5, 15, 17]))
    return corner+[c+e for c, e in zip(corner, extent)]+[rng.choice([0.0, 1e-7, 0.3]), flags]


def trsf2_spec(rng: random.Random) -> list:
    """Kind (0 identity .. 5 compound), two coordinates and an angle or nonzero scale."""
    kind = rng.randrange(6)
    third = rng.choice([0.5, 2.0, -1.5, 0.7]) if kind in (3, 5) else rng.uniform(-3, 3)
    return [kind, rng.uniform(-4, 4), rng.uniform(-4, 4), third]


def append_box_extra_requests(rng: random.Random, requests: list[str]) -> None:
    """Limits, getters, Update, Dump and Bnd_Tools for both boxes; Box2d rays and transforms."""
    for _ in range(300):
        spec = box_spec(rng)
        if rng.random() < 0.05:
            spec[-1] = 63
        requests.append(line('box_extra', spec+[rng.uniform(-9, 9) for _ in range(3)]))
    for _ in range(300):
        direction = rng.choice([[1.0, 0.0], [0.0, -1.0], [rng.gauss(0, 1)+0.01, rng.gauss(0, 1)], [1e-20, 1.0]])
        requests.append(line('box2_extra', box2_spec(rng)+box2_spec(rng)+[rng.uniform(-9, 9), rng.uniform(-9, 9)]
                             + direction+trsf2_spec(rng)+trsf2_spec(rng)))


def trsf3_spec(rng: random.Random) -> list:
    """Kind (0 identity .. 6 plane mirror), a point, a nonzero direction and an angle/scale."""
    kind = rng.randrange(7)
    direction = [rng.gauss(0, 1) for _ in range(2)]+[1.0]
    third = rng.choice([0.5, 2.0, -1.5, 0.7]) if kind in (3, 5) else rng.uniform(-3, 3)
    return [kind]+[rng.uniform(-4, 4) for _ in range(3)]+direction+[third]


def append_compact_requests(rng: random.Random, requests: list[str]) -> None:
    """Transformed IsOut/IsIn of Bnd_B3/Bnd_B2 with near, far and nested boxes."""
    for case in range(500):
        first = [rng.uniform(-3, 3) for _ in range(3)]+[rng.uniform(0, 2) for _ in range(3)]
        if case % 3 == 0:
            second = [c+rng.uniform(-0.5, 0.5) for c in first[:3]]+[h+rng.uniform(1, 6) for h in first[3:]]
        else:
            second = [rng.uniform(-6, 6) for _ in range(3)]+[rng.uniform(0, 3) for _ in range(3)]
        requests.append(line('compact_trsf', first+second+trsf3_spec(rng)+trsf2_spec(rng)))


def append_compact_array_requests(rng: random.Random, requests: list[str]) -> None:
    """Bnd_B2/Bnd_B3 array constructors and setters, Center/HSize, Add and Clear: random,
    float-inexact (0.1), zero, negative (void) and huge half sizes."""
    specials = [[0.1, -0.3, 0.7], [0.0, 0.0, 0.0], [-1.0, 0.5, 0.5], [1e20, 1e-20, 3e38], [-1e-6, 2.0, 2.0]]
    for case in range(200):
        center = [rng.uniform(-5, 5) for _ in range(3)]
        half = specials[case % len(specials)] if case < 2*len(specials) else [rng.uniform(0, 3) for _ in range(3)]
        point = [rng.uniform(-8, 8) for _ in range(3)]
        other = [rng.uniform(-5, 5) for _ in range(3)]+[rng.choice([0.0, rng.uniform(0, 4)]) for _ in range(3)]
        requests.append(line('compact_arrays', center+half+point+other))
