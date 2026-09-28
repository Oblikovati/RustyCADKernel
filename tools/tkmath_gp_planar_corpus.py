"""Deterministic planar gp requests (gp_XY, gp_Mat2d, gp_Trsf2d, gp_GTrsf2d and the 2D primitives).

Covers ordinary, zero, tiny (1e-160), huge (1e300) and signed-zero coordinates; zero
scalars (division by zero), invalid one-based indices, singular and rotation matrices with
powers -7..7 (gp_Mat2d's aliased self-product), every gp_Trsf2d construction path with
SetScaleFactor/SetTranslationPart form transitions, SetValues with null determinants,
products of every form pair, inverses of zero scales, gp_Trsf copies, general gp_GTrsf2d
matrices, affinities, SetValue cells and Trsf2d orthogonality failures, and the planar
direction/vector/point/axis/line members with parallel, opposite and zero inputs.
"""
import math
import random

SPECIAL = [0.0, -0.0, 1.0, -1.0, 1e-160, -1e-160, 1e300, -1e300, 0.5, 2.0]


def line(op: str, values: list) -> str:
    """Format one request with round-trip float text; line('gp2d_xy', [1, 2])."""
    return op+' '+' '.join(str(value) for value in values)


def append_gp_planar_requests(requests: list[str]) -> None:
    """Append every planar gp request with its own seed; append_gp_planar_requests(requests)."""
    rng = random.Random(19910108)
    append_xy_requests(rng, requests)
    append_matrix_requests(rng, requests)
    append_transform_requests(rng, requests)
    append_general_requests(rng, requests)
    append_geometry_requests(rng, requests)
    append_standard_requests(requests)


def scalar(rng: random.Random, scale: float = 10.0) -> float:
    """A uniform value, or one special value in five draws; scalar(rng)."""
    if rng.random() < 0.2:
        return rng.choice(SPECIAL)
    return rng.uniform(-scale, scale)


def unit(rng: random.Random) -> list[float]:
    """A random unit vector, sometimes an exact axis; unit(rng)."""
    if rng.random() < 0.3:
        return rng.choice([[1.0, 0.0], [0.0, 1.0], [-1.0, 0.0], [0.0, -1.0]])
    angle = rng.uniform(-math.pi, math.pi)
    return [math.cos(angle), math.sin(angle)]


def rotation(angle: float) -> list[float]:
    """Row-major rotation coefficients; rotation(0.5)."""
    return [math.cos(angle), -math.sin(angle), math.sin(angle), math.cos(angle)]


def matrix(rng: random.Random, case: int) -> list[float]:
    """General, rotation, singular, scaled rotation or special 2x2 matrices by case."""
    pattern = case % 5
    if pattern == 1:
        return rotation(rng.uniform(-4, 4))
    if pattern == 2:
        a, b = rng.uniform(-3, 3), rng.uniform(-3, 3)
        return [a, b, 2*a, 2*b]
    if pattern == 3:
        s = rng.uniform(-3, 3)
        return [s*v for v in rotation(rng.uniform(-4, 4))]
    if pattern == 4:
        return [rng.choice(SPECIAL) for _ in range(4)]
    return [rng.uniform(-5, 5) for _ in range(4)]


def append_xy_requests(rng: random.Random, requests: list[str]) -> None:
    """gp_XY algebra, normalization, linear forms and index checks."""
    for case in range(400):
        values = [scalar(rng) for _ in range(6)]+matrix(rng, case)+[rng.randint(-1, 3)]
        requests.append(line('gp2d_xy', values))
    requests += ['gp2d_xy 0 0 0 0 0 0 1 0 0 1 1', 'gp2d_xy 1e-160 0 0 1e-160 2 3 0 0 0 0 2',
                 'gp2d_xy -0.0 0.0 0.0 -0.0 -2 0 1 2 3 4 0', 'gp2d_xy 1e300 1e300 -1e300 1e300 1e10 2 1 1 1 1 3']


def append_matrix_requests(rng: random.Random, requests: list[str]) -> None:
    """gp_Mat2d products, inverses, powers and release-mode index handling."""
    for case in range(500):
        values = matrix(rng, case)+matrix(rng, case+2)+[scalar(rng, 4), rng.randint(-7, 7), rng.randint(0, 3)]
        requests.append(line('gp2d_mat', values))
    for n in range(-7, 8):
        requests.append(line('gp2d_mat', rotation(0.3)+rotation(-1.1)+[0.5, n, 1]))
        requests.append(line('gp2d_mat', [0, 0, 0, 0, 1, 2, 3, 4, 0, n, 2]))


def transform_spec(rng: random.Random, kind: int) -> list[float]:
    """kind, six parameters, SetScaleFactor flag/value and SetTranslationPart flag/vector."""
    params = [scalar(rng, 5) for _ in range(6)]
    if kind in (5, 7, 8):
        params[2:4] = unit(rng) if rng.random() < 0.8 else [rng.uniform(-3, 3), rng.uniform(-3, 3)]
    if kind == 6 and rng.random() < 0.3:
        params[2] = rng.choice([0.0, 1.0, -1.0, 1e-300])
    if kind == 9 and rng.random() < 0.5:
        params = [*rotation(rng.uniform(-3, 3))[:2], params[2], *rotation(rng.uniform(-3, 3))[2:], params[5]]
        params = [params[0]*1.5, params[1]*1.5, params[2], params[3]*1.5, params[4]*1.5, params[5]]
    if kind == 9 and rng.random() < 0.2:
        params[3:5] = [2*params[0], 2*params[1]]
    if kind == 10:
        params[2:5] = [rng.uniform(-1, 1) for _ in range(3)]
    scale_flag = int(rng.random() < 0.35)
    scale = rng.choice([1.0, -1.0, 0.0, 2.5, rng.uniform(-3, 3)])
    offset_flag = int(rng.random() < 0.35)
    offset = rng.choice([[0.0, 0.0], [1e-300, 0.0], [rng.uniform(-3, 3), rng.uniform(-3, 3)]])
    return [kind]+params+[scale_flag, scale, offset_flag]+offset


def append_transform_requests(rng: random.Random, requests: list[str]) -> None:
    """Every gp_Trsf2d form pair, inversions and powers, with point/vector/direction images."""
    for case in range(1500):
        first, second = transform_spec(rng, case % 12), transform_spec(rng, (case // 12) % 12)
        tail = [scalar(rng), scalar(rng), rng.uniform(-3, 3), rng.uniform(-3, 3), rng.randint(-6, 6)]
        requests.append(line('gp2d_trsf', first+second+tail))
    identity = [0]+[0.0]*6+[0, 1.0, 0, 0.0, 0.0]
    for kind in range(12):
        for n in (-3, -2, 0, 2, 3, 5):
            spec = [kind, 1.0, 2.0, 0.6, 0.8, -1.0, 0.5, 0, 1.0, 0, 0.0, 0.0]
            requests.append(line('gp2d_trsf', spec+identity+[3.0, -4.0, 1.0, 1.0, n]))
    zero_scale = [6, 1.0, 1.0, 0.0, 0, 0, 0, 0, 1.0, 0, 0.0, 0.0]
    requests.append(line('gp2d_trsf', zero_scale+identity+[1.0, 1.0, 0.0, 0.0, -2]))


def general_spec(rng: random.Random, kind: int) -> list[float]:
    """kind, a transform spec, a matrix, a translation, an affinity axis/ratio and one cell."""
    transform = transform_spec(rng, rng.randrange(12))
    axis = [scalar(rng, 5), scalar(rng, 5)]+(unit(rng) if rng.random() < 0.9 else [0.0, 0.0])
    cell = [rng.randint(0, 3), rng.randint(0, 4), scalar(rng, 5)]
    return [kind]+transform+matrix(rng, rng.randrange(5))+[scalar(rng, 5), scalar(rng, 5)]+axis+[scalar(rng, 3)]+cell


def append_general_requests(rng: random.Random, requests: list[str]) -> None:
    """gp_GTrsf2d construction paths, products, inverses, powers and Trsf2d conversion."""
    for case in range(1000):
        first, second = general_spec(rng, case % 7), general_spec(rng, (case // 7) % 7)
        requests.append(line('gp2d_gtrsf', first+second+[scalar(rng), scalar(rng), rng.randint(-6, 6)]))


def append_geometry_requests(rng: random.Random, requests: list[str]) -> None:
    """Planar primitives with random, parallel, opposite, normal and zero inputs."""
    for case in range(800):
        p, q = [scalar(rng) for _ in range(2)], [scalar(rng) for _ in range(2)]
        # gp_Lin2d::Normal is noexcept: a direction from overflowing components is stored as
        # zero and its quarter turn calls std::terminate, so the directions stay finite-normed.
        d = [rng.uniform(-10, 10) for _ in range(2)] if case % 4 else unit(rng)
        pattern = case % 6
        e = [rng.uniform(-10, 10) for _ in range(2)]
        if pattern == 1: e = d[:]
        if pattern == 2: e = [-d[0], -d[1]]
        if pattern == 3: e = [-d[1], d[0]]
        if pattern == 4: e = [0.0, 0.0]
        knobs = [rng.uniform(-7, 7), scalar(rng, 3), rng.choice([0.0, 1e-7, 1.0, -1.0]), rng.choice([0.0, 1e-12, 0.1, -1.0])]
        coefficients = [scalar(rng, 5) for _ in range(3)] if case % 5 else [0.0, 0.0, 1.0]
        requests.append(line('gp2d_geom', p+q+d+e+knobs+coefficients+[rng.randint(0, 3), case % 4]))
    # Coincident points and zero vectors: gp_VectorWithNullMagnitude, unchanged mirrors and
    # length-only IsEqual; a zero direction raises at construction.
    for lin, ang in ((1e-7, 1e-12), (-1.0, 0.1), (0.0, 0.0)):
        requests.append(line('gp2d_geom', [1, 2, 1, 2, 0, 0, 0, 1, 0.5, 2, lin, ang, 0, 0, 0, 1, 0]))
        requests.append(line('gp2d_geom', [1, 2, 3, 5, 0.6, 0.8, 0, 0, 0.5, 2, lin, ang, 1, 1, 0, 2, 1]))
        requests.append(line('gp2d_geom', [0, 0, 1e-160, 0, 1, 0, -1, 0, 3.0, -1, lin, ang, 3, 4, 5, 1, 2]))


def append_standard_requests(requests: list[str]) -> None:
    """Every gp_Dir::D main/X pair (parallel pairs keep the native fallback Y) and the gp statics."""
    for main in range(6):
        for second in range(6):
            requests.append(line('gp_standard', [main, second, 1.5, -2.0, 0.25]))
