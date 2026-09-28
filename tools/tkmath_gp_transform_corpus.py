"""Deterministic gp_Trsf, gp_GTrsf and gp_Quaternion requests.

Covers every gp_Trsf construction path (translations, point/axis/plane mirrors, rotations,
checked scales, frame changes and displacements, SetValues with null determinants and
non-orthogonal matrices, quaternion rotations, gp_Trsf2d copies including axis mirrors), the
SetScaleFactor/SetTranslationPart/SetRotationPart form transitions, products of every form
pair, inverses of zero scales and powers -6..6; general gp_GTrsf matrices, both affinities,
SetValue cells (including out-of-range ones), SetMat4, SetForm and negative general powers;
and quaternions with zero, opposite, parallel and random vectors and interpolation parameters.
"""
import math
import random

SPECIAL = [0.0, -0.0, 1.0, -1.0, 1e-160, 1e300, -1e300, 0.5, 2.0]


def line(op: str, values: list) -> str:
    """Format one request with round-trip float text; line('gp3d_trsf', [1, 2])."""
    return op+' '+' '.join(str(value) for value in values)


def append_gp_transform_requests(requests: list[str]) -> None:
    """Append every gp_Trsf/gp_GTrsf/gp_Quaternion request; append_gp_transform_requests(r)."""
    rng = random.Random(19950606)
    for case in range(1500):
        first, second = transform_spec(rng, case % 15), transform_spec(rng, (case // 15) % 15)
        tail = [scalar(rng), scalar(rng), scalar(rng)] + unit3(rng) + [rng.randint(-6, 6)]
        requests.append(line('gp3d_trsf', first + second + tail))
    for case in range(1000):
        first, second = general_spec(rng, case % 10), general_spec(rng, (case // 10) % 10)
        requests.append(line('gp3d_gtrsf', first + second + [scalar(rng), scalar(rng), scalar(rng), rng.randint(-6, 6)]))
    for case in range(600):
        requests.append(line('gp3d_quat', quaternion_request(rng, case)))


def scalar(rng: random.Random, scale: float = 10.0) -> float:
    """A uniform value, or a special value in one draw out of five."""
    return rng.choice(SPECIAL) if rng.random() < 0.2 else rng.uniform(-scale, scale)


def unit3(rng: random.Random) -> list[float]:
    """A random unit vector, sometimes an exact axis."""
    if rng.random() < 0.3:
        return rng.choice([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [0.0, 0.0, -1.0]])
    z, angle = rng.uniform(-1, 1), rng.uniform(-math.pi, math.pi)
    r = math.sqrt(1 - z*z)
    return [r*math.cos(angle), r*math.sin(angle), z]


def quaternion(rng: random.Random) -> list[float]:
    """Unit, scaled, identity or zero quaternion components."""
    pattern = rng.randrange(6)
    if pattern == 0:
        return [0.0, 0.0, 0.0, 1.0]
    if pattern == 1:
        return [0.0, 0.0, 0.0, 0.0]
    values = [rng.uniform(-1, 1) for _ in range(4)]
    norm = math.sqrt(sum(v*v for v in values))
    return [v / norm * (1.0 if pattern < 4 else rng.uniform(0.5, 3)) for v in values]


def rotation_rows(rng: random.Random, scale: float) -> list[float]:
    """3x4 row-major rows of a scaled rotation about Z with a translation column."""
    a = rng.uniform(-3, 3)
    c, s = math.cos(a)*scale, math.sin(a)*scale
    return [c, -s, 0.0, rng.uniform(-5, 5), s, c, 0.0, rng.uniform(-5, 5), 0.0, 0.0, scale, rng.uniform(-5, 5)]


def parameters(rng: random.Random, kind: int) -> list[float]:
    """Twelve construction parameters shaped for the construction kind."""
    p = [scalar(rng, 5) for _ in range(12)]
    if kind in (3, 5, 6, 8, 9, 10):
        p[3:6] = unit3(rng) if rng.random() < 0.9 else [rng.uniform(-2, 2) for _ in range(3)]
    if kind == 7 and rng.random() < 0.3:
        p[3] = rng.choice([0.0, 1.0, -1.0, 1e-300])
    if kind == 11:
        p = rotation_rows(rng, rng.choice([1.0, -2.0, 0.5])) if rng.random() < 0.6 else p
        if rng.random() < 0.2:
            p[4:7] = [2*p[0], 2*p[1], 2*p[2]]
    if kind in (12, 13):
        p[0:4] = quaternion(rng)
    if kind == 14:
        p[2:4] = [rng.uniform(-1, 1), rng.uniform(-1, 1)]
    return p


def transform_spec(rng: random.Random, kind: int) -> list[float]:
    """kind, twelve parameters, SetScaleFactor, SetTranslationPart and SetRotationPart."""
    scale = [int(rng.random() < 0.3), rng.choice([1.0, -1.0, 0.0, 2.5, rng.uniform(-3, 3)])]
    offset = [int(rng.random() < 0.3)] + rng.choice([[0.0, 0.0, 0.0], [1e-160, 0.0, 0.0], [rng.uniform(-3, 3) for _ in range(3)]])
    rotation = [int(rng.random() < 0.25)] + quaternion(rng)
    return [kind] + parameters(rng, kind) + scale + offset + rotation


def general_spec(rng: random.Random, kind: int) -> list[float]:
    """kind, a transform spec, a matrix, a translation, an affinity axis/ratio and a cell."""
    transform = transform_spec(rng, rng.randrange(15))
    matrix = [rng.uniform(-3, 3) for _ in range(9)]
    if rng.random() < 0.3:
        matrix = rotation_rows(rng, rng.choice([1.0, 2.0]))
        matrix = matrix[0:3] + matrix[4:7] + matrix[8:11]
    axis = [scalar(rng, 5) for _ in range(3)] + (unit3(rng) if rng.random() < 0.9 else [0.0, 0.0, 0.0])
    cell = [rng.randint(0, 4), rng.randint(0, 5), scalar(rng, 5)]
    return [kind] + transform + matrix + [scalar(rng, 5) for _ in range(3)] + axis + [scalar(rng, 3)] + cell


def quaternion_request(rng: random.Random, case: int) -> list[float]:
    """Two quaternions, three vectors, an angle and an interpolation parameter."""
    u = unit3(rng)
    pattern = case % 6
    v = [rng.uniform(-2, 2) for _ in range(3)]
    if pattern == 1:
        v = [-c for c in u]
    if pattern == 2:
        v = [2.5*c for c in u]
    if pattern == 3:
        u = [0.0, 0.0, 0.0]
    h = [rng.uniform(-2, 2) for _ in range(3)]
    return quaternion(rng) + quaternion(rng) + u + v + h + [rng.uniform(-7, 7), rng.choice([0.0, 1.0, 0.25, rng.uniform(-0.5, 1.5)])]
