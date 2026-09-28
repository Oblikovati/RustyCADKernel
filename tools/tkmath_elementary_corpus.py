"""Deterministic ElCLib, ElSLib and gp conic/quadric/axis requests appended to the corpus.

Covers generic and axis-aligned frames, parallel and zero frame directions, indirect gp_Ax3
frames, negative/zero/equal/inverted radii, degenerate (|F| <= gp::Resolution) and
barely non-degenerate parabolas, derivative orders across every residue including negative
orders, points at centres, on axes and at apexes, infinite and tiny periods, invalid cone
semi-angles, zero scale factors and the gp_Hypr2d asymptote quirks. Values stay normal
floating-point numbers so both parsers read identical bits.
"""
import math
import random

DBL_MIN = 2.2250738585072014e-308
SPECIAL_U = [0.0, -0.0, math.pi / 2, math.pi, 2 * math.pi, -math.pi / 4, 7.5, -12.25, 1e3]


def line(op: str, values: list) -> str:
    """Format one request with round-trip float text; line('elperiod', [1, 0, 6])."""
    return op + ' ' + ' '.join(str(value) for value in values)


def unit(rng: random.Random) -> list[float]:
    """Random direction-like triple, never exactly zero; unit(rng)."""
    return [rng.uniform(-1, 1) for _ in range(3)] if rng.random() < 0.9 else [0.0, 0.0, 1.0]


def frame(rng: random.Random, case: int) -> list[float]:
    """Origin, normal and X direction with degenerate patterns chosen by case."""
    origin = [rng.uniform(-10, 10) for _ in range(3)]
    normal, x = unit(rng), unit(rng)
    pattern = case % 9
    if pattern == 1: normal, x = [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]
    if pattern == 2: normal, x = [0.0, 1.0, 0.0], [0.0, 0.0, -1.0]
    if pattern == 3: x = [2.0 * c for c in normal]
    if pattern == 4: x = [-c for c in normal]
    if pattern == 5: normal = [0.0, 0.0, 0.0]
    if pattern == 6: origin = [0.0, 0.0, 0.0]
    if pattern == 7: x = [c + 1e-9 for c in normal]
    return origin + normal + x


def planar_frame(rng: random.Random, case: int) -> list[float]:
    """Origin, X direction and sense for a gp_Ax22d."""
    origin = [rng.uniform(-10, 10) for _ in range(2)]
    x = [rng.uniform(-1, 1), rng.uniform(-1, 1)]
    if case % 7 == 1: x = [1.0, 0.0]
    if case % 7 == 2: x = [0.0, -1.0]
    if case % 7 == 3: x = [0.0, 0.0]
    if case % 7 == 4: origin = [0.0, 0.0]
    return origin + x + [case % 2]


def radii(rng: random.Random, case: int) -> list[float]:
    """Radius, major and minor radii with invalid and degenerate patterns."""
    r, big = rng.uniform(0.1, 20), rng.uniform(0.1, 20)
    small = rng.uniform(0.0, big)
    pattern = case % 8
    if pattern == 1: r, small = 0.0, 0.0
    if pattern == 2: r = -1.5
    if pattern == 3: small = big
    if pattern == 4: big, small = small, big + 1.0
    if pattern == 5: small = -0.5
    if pattern == 6: big, small = 0.0, 0.0
    return [r, big, small]


def focal(rng: random.Random, case: int) -> float:
    """Focal length including the gp::Resolution degeneracy boundary."""
    return [rng.uniform(0.01, 10), 0.0, DBL_MIN, 2 * DBL_MIN, -1.0, 1e-300][case % 6]


def parameter(rng: random.Random, case: int) -> float:
    """Curve parameter, mixing random values with period boundaries."""
    return SPECIAL_U[case % len(SPECIAL_U)] if case % 3 == 0 else rng.uniform(-10, 10)


def point(rng: random.Random, origin: list[float], case: int) -> list[float]:
    """Query point; some coincide with the frame origin to reach the null branches."""
    if case % 11 == 0: return list(origin)
    return [rng.uniform(-15, 15) for _ in origin]


def append_elementary_requests(requests: list[str]) -> None:
    """Append every elementary-geometry request with its own seed; append_elementary_requests(r)."""
    rng = random.Random(19930803)
    append_curve_requests(rng, requests)
    append_period_requests(rng, requests)
    append_surface_requests(rng, requests)
    append_member_requests(rng, requests)


def append_curve_requests(rng: random.Random, requests: list[str]) -> None:
    """ElCLib 3D/2D evaluators, parameters, overloads and To3d embeddings."""
    for case in range(720):
        values = frame(rng, case)
        shape = [parameter(rng, case)] + radii(rng, case) + [focal(rng, case), case % 13 - 3]
        requests.append(line('elclib3', values + shape + point(rng, values[:3], case)))
        planar = planar_frame(rng, case)
        requests.append(line('elclib2', planar + shape + point(rng, planar[:2], case)))
        embedded = values + planar + radii(rng, case) + [focal(rng, case)]
        requests.append(line('elto3d', embedded + [rng.uniform(-5, 5) for _ in range(4)]))


def append_period_requests(rng: random.Random, requests: list[str]) -> None:
    """InPeriod and AdjustPeriodic across infinite, tiny, reversed and random periods."""
    bounds = [(0.0, 2 * math.pi), (1.5, 4.5), (-3.0, -1.0), (0.0, 1e-300), (2.0, 1.0),
              (0.0, 2e100), (-1e101, 1.0), (6.1645624650899675, 12.447747772269555)]
    for case in range(400):
        first, last = bounds[case % len(bounds)] if case % 2 else sorted(rng.uniform(-20, 20) for _ in range(2))
        u = rng.uniform(-50, 50) if case % 5 else [first, last, 1e101, -1e100, first - 1e-9][case // 5 % 5]
        u1, u2 = rng.uniform(-30, 30), rng.uniform(-30, 30)
        if case % 7 == 0: u2 = u1 + 1e-9
        requests.append(line('elperiod', [u, first, last, [1e-7, 0.0, 0.5, 1e-12][case % 4], u1, u2]))


def append_surface_requests(rng: random.Random, requests: list[str]) -> None:
    """ElSLib on direct and indirect frames with every derivative order and degeneracy."""
    for case in range(720):
        values = frame(rng, case) + [case % 2]
        u, v = parameter(rng, case), parameter(rng, case + 1)
        r = [rng.uniform(0.1, 10), 0.0, -2.0][case % 3 if case % 4 == 0 else 0]
        angle = [rng.uniform(-1.4, 1.4), 0.0, math.pi / 2, -0.3, 1e-300][case % 5]
        big, small = rng.uniform(0.1, 10), rng.uniform(0.0, 10)
        orders = [case % 6 - 1, case // 6 % 6 - 1]
        requests.append(line('elslib', values + [u, v, r, angle, big, small] + orders + point(rng, values[:3], case)))


def motion(rng: random.Random, case: int, size: int) -> list[float]:
    """Pivot, axis direction, angle, scale factor and translation for the gp member probes."""
    pivot = [rng.uniform(-5, 5) for _ in range(size)]
    axis = [rng.uniform(-1, 1) for _ in range(size)]
    scale = [rng.uniform(-3, 3), 0.0, -1.0, 1.0, 2.5][case % 5]
    return pivot + axis + [rng.uniform(-7, 7), scale] + [rng.uniform(-5, 5) for _ in range(size)]


def append_member_requests(rng: random.Random, requests: list[str]) -> None:
    """gp conic, quadric and axis members, including quirk-triggering radii."""
    for case in range(480):
        values = frame(rng, case)
        shape = radii(rng, case)
        conic = [shape[1], shape[2], focal(rng, case), shape[0]]
        requests.append(line('gpconic3', values + conic + motion(rng, case, 3) + point(rng, values[:3], case)))
        planar = planar_frame(rng, case)
        requests.append(line('gpconic2', planar + conic + motion(rng, case, 2) + point(rng, planar[:2], case)))
        angle = [rng.uniform(-1.4, 1.4), 0.0, math.pi / 2][case % 3]
        quadric = [case % 2, shape[0], angle, shape[1], shape[2]]
        requests.append(line('gpquadric', values + quadric + motion(rng, case, 3) + point(rng, values[:3], case)))
        other = [rng.uniform(-5, 5) for _ in range(3)] + unit(rng)
        if case % 5 == 0: other = values[:6]
        if case % 5 == 1: other = values[:3] + values[6:9]
        tolerances = [[1e-7, 1e-12], [10.0, 3.2], [0.0, 0.0]][case % 3]
        requests.append(line('gpaxis', values + other + [rng.uniform(-7, 7), rng.uniform(-3, 3)] + tolerances))
