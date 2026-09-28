"""Deterministic CSLib requests appended after the solver corpus.

Covers regular, parallel, null and overflowing normal derivatives; singular
points of increasing order at interior, edge and corner parameters; raw/unit
normal derivatives with valid, out-of-range and null reference inputs; the
NormalPolyDef trigonometric polynomial at regular and singular angles; and
Class2d polygons across containers, bounds, tolerances and the lazy grid.
"""
import math
import random

UNIT_SQUARE = (0.0, 0.0, 1.0, 1.0)


def append_cslib_requests(requests: list[str]) -> None:
    """Append every CSLib request with its own seed; append_cslib_requests(requests)."""
    rng = random.Random(19910909)
    append_normal_requests(rng, requests)
    append_singular_requests(rng, requests)
    append_derivative_requests(rng, requests)
    append_polynomial_requests(rng, requests)
    append_classifier_requests(rng, requests)


def line(op: str, values: list) -> str:
    """Format one request with round-trip float text; line('cslib_poly', [1, 0.5])."""
    return op+' '+' '.join(str(value) for value in values)


def vector(rng: random.Random, scale: float = 1.0) -> list[float]:
    """Random 3D vector in [-3,3]^3 times scale; vector(rng, 1e-3)."""
    return [rng.uniform(-3, 3)*scale for _ in range(3)]


def normal_vectors(rng: random.Random, case: int) -> list[list[float]]:
    """D1U, D1V, D2U, D2V, D2UV with a degenerate pattern chosen by case."""
    d1u, d1v, d2u, d2v, d2uv = (vector(rng) for _ in range(5))
    pattern = case % 14
    zero = [0.0, 0.0, 0.0]
    if pattern == 1: d1u = zero
    if pattern == 2: d1v = zero
    if pattern == 3: d1u, d1v = zero, zero
    if pattern == 4: d1v = [2.5*c for c in d1u]
    if pattern == 5: d1v = [c+1e-9 for c in d1u]
    if pattern == 6: d1u, d1v = [1e-160, 0.0, 0.0], [0.0, 1e-160, 0.0]
    if pattern == 7: d2u, d2uv, d2v = zero, zero, zero
    if pattern == 8: d1u, d2uv, d2v = zero, zero, zero
    if pattern == 9: d1v, d2uv, d2u = zero, zero, zero
    if pattern == 10: d2uv, d2v = zero, [1e-9*c for c in d2u]
    if pattern == 11: d1u, d1v = vector(rng, 1e150), vector(rng, 1e150)
    # Relative (not absolute) negligibility of one normal derivative.
    if pattern in (12, 13): d1u, d1v, d2uv = vector(rng, 1e4), vector(rng, 1e4), zero
    if pattern == 12: d2u, d2v = vector(rng, 1e4), vector(rng, 1e-9)
    if pattern == 13: d2u, d2v = vector(rng, 1e-9), vector(rng, 1e4)
    return [d1u, d1v, d2u, d2v, d2uv]


def append_normal_requests(rng: random.Random, requests: list[str]) -> None:
    """First-, second-order and magnitude normals; statuses and failures."""
    for case in range(600):
        tol = rng.choice([1e-10, 1e-6, 0.1, 0.0, -1.0, 1e-3])
        flat = [c for v in normal_vectors(rng, case) for c in v]
        requests.append(line('cslib_normal', flat+[tol]))
    requests.append(line('cslib_normal', [1, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0.0]))
    requests.append(line('cslib_normal', [0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, -1.0]))


def singular_table(rng: random.Random, size: int, k0: int, parallel: bool) -> list[list[float]]:
    """(size+1)^2 derivatives of N: null below order k0, collinear at k0 when parallel."""
    base = vector(rng)
    table = [[vector(rng) for _ in range(size+1)] for _ in range(size+1)]
    for i in range(size+1):
        for j in range(size+1):
            if i+j < k0:
                table[i][j] = [0.0, 0.0, 0.0] if rng.random() < 0.8 else vector(rng, 1e-12)
            elif i+j == k0 and parallel:
                ratio = rng.choice([0.0, 0.0, rng.uniform(-2, 2), rng.uniform(0.1, 2), -rng.uniform(0.1, 2)])
                table[i][j] = [ratio*c for c in base]
    return table


def parametric_position(rng: random.Random) -> list[float]:
    """U, V, Umin, Umax, Vmin, Vmax at an interior, edge or corner parameter."""
    bounds = [0.0, 1.0, -2.0, 3.0]
    u = rng.choice([0.0, 1.0, 0.5, rng.uniform(0, 1), 1e-10])
    v = rng.choice([-2.0, 3.0, 0.5, rng.uniform(-2, 3), 3.0 - 1e-10])
    return [u, v]+bounds


def append_singular_requests(rng: random.Random, requests: list[str]) -> None:
    """Singular-point normals, sign analysis over each angular domain."""
    for case in range(640):
        k0 = rng.choice([0, 1, 1, 2, 2, 3, 4])
        size = k0 + rng.choice([0, 0, 1])
        order = rng.choice([k0, k0, k0+1, max(k0-1, -1)])
        order = min(order, size) if case % 17 else order+size+1
        tol = rng.choice([1e-9, 1e-7, 1e-7, 1e-4, 0.0])
        table = singular_table(rng, size, k0, case % 5 != 0)
        flat = [c for row in table for v in row for c in v]
        requests.append(line('cslib_singular', [order, size, tol]+parametric_position(rng)+flat))
    # F = sin X changes sign at 0, but the index-0 interval quirk hides it (Defined).
    requests.append(line('cslib_singular', [1, 1, 1e-7, 0.5, 0.5, 0.0, 1.0, 0.0, 1.0,
                                            0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 2, 3]))
    # F = cos X: the change between intervals 1 and 2 is still seen (InfinityOfSolutions).
    requests.append(line('cslib_singular', [1, 1, 1e-7, 0.5, 0.5, 0.0, 1.0, 0.0, 1.0,
                                            0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 2, 3]))
    # The same cos X polynomial on the U = Umin edge domain [-pi/2, pi/2].
    requests.append(line('cslib_singular', [1, 1, 1e-7, 0.0, 0.5, 0.0, 1.0, 0.0, 1.0,
                                            0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0]))
    for k0 in [25, 26]:
        table = singular_table(rng, k0, k0, True)
        flat = [c for row in table for v in row for c in v]
        requests.append(line('cslib_singular', [k0, k0, 1e-7, 0.5, 0.5, 0.0, 1.0, 0.0, 1.0]+flat))


def append_derivative_requests(rng: random.Random, requests: list[str]) -> None:
    """DNNUV (both overloads) and DNNormal, including out-of-range and null references."""
    for case in range(400):
        size = rng.randint(1, 5)
        nu, nv = rng.randint(0, size), rng.randint(0, size)
        iu, iv = rng.choice([0, 0, 1]), rng.choice([0, 0, 1])
        if case % 23 == 0: nu, nv = -1, 1
        if case % 29 == 0: nu, nv = 1, -1
        scale = rng.choice([1.0, 1e-3, 1e3])
        first = [[vector(rng, scale) for _ in range(size+1)] for _ in range(size+1)]
        second = [[vector(rng, scale) for _ in range(size+1)] for _ in range(size+1)]
        if case % 13 == 0 and iu <= size and iv <= size: first[iu][iv] = [0.0, 0.0, 0.0]
        flat = [c for table in (first, second) for row in table for v in row for c in v]
        requests.append(line('cslib_dn', [size, nu, nv, iu, iv]+flat))


def append_polynomial_requests(rng: random.Random, requests: list[str]) -> None:
    """NormalPolyDef values and derivatives at regular and singular angles."""
    singular = [0.0, math.pi/2, math.pi, -math.pi/2, 1e-310, 3*math.pi/2]
    for case in range(400):
        k = rng.choice([-1, 0, 1, 2, 3, 4, 5, 6, 12, 25])
        x = rng.choice(singular) if case % 7 == 0 else rng.uniform(-4, 4)
        coefficients = [rng.choice([0.0, rng.uniform(-3, 3)]) for _ in range(max(k, 0)+1)]
        requests.append(line('cslib_poly', [k, x]+coefficients))
    # PLib::Bin fails beyond degree 25; only regular angles reach it natively.
    requests.append(line('cslib_poly', [26, 0.7]+[1.0]*27))


def polygon(rng: random.Random, case: int) -> list[tuple[float, float]]:
    """Unit-domain polygon: regular, star, sampled square, random or degenerate."""
    kind = case % 6
    n = rng.choice([3, 4, 5, 8, 23, 24, 32, 64, 97])
    if kind == 0:
        return [(0.5+0.4*math.cos(2*math.pi*i/n), 0.5+0.4*math.sin(2*math.pi*i/n)) for i in range(n)]
    if kind == 1:
        return [(0.5+r*math.cos(2*math.pi*i/n), 0.5+r*math.sin(2*math.pi*i/n))
                for i in range(n) for r in [0.45 if i % 2 == 0 else rng.uniform(0.1, 0.3)]]
    if kind == 2:
        side = max(n//4, 1); low, high = rng.uniform(0, 0.3), rng.uniform(0.7, 1)
        steps = [low+(high-low)*i/side for i in range(side)]
        return [(s, low) for s in steps]+[(high, s) for s in steps]+[(high+low-s, high) for s in steps]+[(low, high+low-s) for s in steps]
    if kind == 3:
        return [(rng.uniform(0, 1), rng.uniform(0, 1)) for _ in range(n)]
    if kind == 4:
        return [(0.1*i, 0.2*i) for i in range(rng.choice([2, 3, 5]))]
    return [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.5, 0.4), (0.0, 1.0)]


def classifier_bounds(rng: random.Random, case: int) -> tuple[float, float, float, float]:
    """(umin, vmin, umax, vmax): unit, anisotropic, tiny, invalid or extreme."""
    kind = case % 8
    if kind == 1: return (-5.0, 3.0, 20.0, 4.0)
    if kind == 2: return (1.0, 1.0, 1.0+1e-11, 2.0)
    if kind == 3: return (1.0, 0.0, 0.0, 1.0)
    if kind == 4: return (-2e100, -2e100, 2e100, 2e100)
    if kind == 5: return (-1.7976931348623157e308, -1.0, 1.7976931348623157e308, 1.0)
    return UNIT_SQUARE


def mapped(point: tuple[float, float], bounds: tuple[float, float, float, float]) -> list[float]:
    """Unit-domain point mapped without overflowing the span of extreme bounds."""
    s, t = point
    coordinates = [bounds[0]*(1-s)+bounds[2]*s, bounds[1]*(1-t)+bounds[3]*t]
    # istream cannot parse 'inf'; overflowing queries saturate to the largest double.
    return [value if math.isfinite(value) else math.copysign(1.7976931348623157e308, value)
            for value in coordinates]


def query_points(rng: random.Random, vertices: list[tuple[float, float]]) -> list[tuple[float, float]]:
    """Random, vertex, edge-midpoint and near-edge queries in unit coordinates."""
    points = [(rng.uniform(-0.1, 1.1), rng.uniform(-0.1, 1.1)) for _ in range(12)]
    points += [(0.5, 0.5), (0.99, 0.99), (0.25, 0.75)]
    points += rng.sample(vertices, min(3, len(vertices)))
    for a, b in zip(vertices[:4], vertices[1:5]):
        offset = rng.choice([0.0, 1e-9, -1e-5, 3e-3])
        points.append(((a[0]+b[0])/2+offset, (a[1]+b[1])/2-offset))
    return points


def append_classifier_requests(rng: random.Random, requests: list[str]) -> None:
    """Class2d SiDans/SiDans_OnMode before and after the grid, copies and moves."""
    for case in range(480):
        bounds = classifier_bounds(rng, case)
        vertices = polygon(rng, case//8)
        queries = query_points(rng, vertices)
        tu, tv = rng.choice([0.0, 1e-10, 1e-4, 0.01, 0.2, -1.0, 2.0]), rng.choice([0.0, 1e-8, 1e-3, 0.05, 1.5])
        on = rng.choice([0.0, 1e-6, 0.01, 0.5, -1.0])
        special = rng.choice([0]*9+[1, 2, 4, 7])
        head = [special, case % 3, tu, tv, on]+list(bounds)+[len(vertices)]
        flat_vertices = [c for p in vertices for c in mapped(p, bounds)]
        flat_queries = [c for p in queries for c in mapped(p, bounds)]
        requests.append(line('cslib_class2d', head+flat_vertices+[len(queries)]+flat_queries))
