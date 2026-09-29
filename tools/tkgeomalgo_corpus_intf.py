"""Deterministic requests for the Intf package records (tools/oracle/tkgeomalgo_probe_intf.cxx).

`intf_triangle` feeds Intf::PlaneEquation and Intf::Contain one triangle and one point:
regular triangles in both orientations, degenerate ones (coincident and collinear
vertices, a normal that underflows below gp::Resolution), and points inside, on the
edges and vertices, outside and off the plane.

`intf_spoint`, `intf_sline`, `intf_tzone` and `intf_interference` run seeded scripts on
Intf_SectionPoint, Intf_SectionLine, Intf_TangentZone and Intf_Interference. The section
points come from a small pool of logical identities (types and addresses), so equal
points, shared line ends and shared zone edges occur often; indices outside the records
reach the range errors. The Intf_Tool and interference requests are in
tkgeomalgo_corpus_intf_tool.py and tkgeomalgo_corpus_intf_poly.py.
"""
from __future__ import annotations
import random

EXTERNAL, FACE, EDGE, VERTEX = range(4)


def numbers(values: list[float]) -> str:
    """The fields of a request, with every double spelled exactly."""
    return ' '.join(repr(float(v)) for v in values)


def triangle_request(p1: list[float], p2: list[float], p3: list[float], point: list[float]) -> str:
    """One `intf_triangle p1 p2 p3 point` request line."""
    return 'intf_triangle ' + numbers([*p1, *p2, *p3, *point])


def fixed_triangles() -> list[tuple[list[float], ...]]:
    """(p1, p2, p3, point) cases chosen for the boundary and degenerate branches."""
    a, b, c = [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]
    tiny = 1e-160  # the edge cross products underflow to zero, below gp::Resolution
    return [
        (a, b, c, [0.25, 0.25, 0.0]), (a, c, b, [0.25, 0.25, 0.0]),
        (a, b, c, [0.5, 0.0, 0.0]), (a, b, c, [0.5, 0.5, 0.0]), (a, b, c, b),
        (a, b, c, [1.0, 1.0, 0.0]), (a, b, c, [0.2, 0.2, 5.0]), (a, b, c, [-1e-300, 0.5, 0.0]),
        (a, a, c, [0.0, 0.5, 0.0]), (a, a, a, a), (a, b, [2.0, 0.0, 0.0], [0.5, 0.0, 0.0]),
        ([0.0, 0.0, 0.0], [tiny, 0.0, 0.0], [0.0, tiny, 0.0], [0.0, 0.0, 0.0]),
        ([1e300, 0.0, 0.0], [0.0, 1e300, 0.0], [0.0, 0.0, 1e300], [1.0, 1.0, 1.0]),
        ([3.0, -2.0, 7.5], [3.0, 4.0, 7.5], [-6.0, 1.0, 7.5], [0.0, 1.0, 7.5]),
    ]


def append_triangle_requests(requests: list[str]) -> None:
    """The fixed triangles, then seeded random ones."""
    for case in fixed_triangles():
        requests.append(triangle_request(*case))
    rng = random.Random(20260929)
    for _ in range(40):
        points = [[rng.uniform(-10.0, 10.0) for _ in range(3)] for _ in range(4)]
        requests.append(triangle_request(*points))


def identity(rng: random.Random) -> tuple[int, ...]:
    """A logical identity: the types and addresses that IsEqual compares."""
    return (rng.randrange(4), rng.randrange(3), rng.randrange(3), rng.randrange(4), rng.randrange(3), rng.randrange(3))


def spoint(rng: random.Random, logic: tuple[int, ...]) -> list[float]:
    """A section point spec (3d, or 2d with the first addresses 0) of identity `logic`,
    with a random location, parameters and incidence."""
    d1, o1, o2, d2, t1, t2 = logic
    location = [rng.choice([0.0, 1.0, -2.5, rng.uniform(-5, 5)]) for _ in range(3)]
    param = lambda: rng.choice([0.0, 0.25, 0.5, 1.0, rng.uniform(0, 1)])  # noqa: E731
    incidence = rng.choice([0.0, 1e-13, 0.5, 1.0])
    if o1 != 0 or t1 != 0 or rng.random() < 0.5:
        return [0, *location, d1, o1, o2, param(), d2, t1, t2, param(), incidence]
    return [1, *location[:2], d1, o2, param(), d2, t2, param(), incidence]


def point_pool(rng: random.Random, count: int) -> list[float]:
    """`count spec...`: a pool of section points sharing a few identities, so that
    logically equal points occur often."""
    identities = [identity(rng) for _ in range(max(2, count // 2))]
    values: list[float] = [count]
    for _ in range(count):
        values += spoint(rng, rng.choice(identities))
    return values


def append_spoint_requests(requests: list[str], rng: random.Random) -> None:
    """Pairs of points, among them the vertex-vertex and edge-vertex cases."""
    vertex_pair = [[0, 0, 0, 0, VERTEX, 1, 2, 0.0, VERTEX, 3, 0, 0.0, 1.0],
                   [0, 1, 1, 1, VERTEX, 5, 2, 0.0, EDGE, 3, 4, 0.5, 1.0]]
    requests.append('intf_spoint ' + numbers(vertex_pair[0] + vertex_pair[1]))
    requests.append('intf_spoint ' + numbers(vertex_pair[1] + vertex_pair[0]))
    requests.append('intf_spoint ' + numbers([0, 1, 2, 3, EDGE, 2147483647, 7, 0.5, FACE, 9, 0, 0.25, 0.5]
                                             + [1, 1, 2, EDGE, 7, 0.5, EDGE, 0, 0.0, 0.0]))
    for _ in range(300):
        first = identity(rng)
        second = first if rng.random() < 0.3 else identity(rng)
        requests.append('intf_spoint ' + numbers(spoint(rng, first) + spoint(rng, second)))


def indices(rng: random.Random, pool: int, low: int, high: int) -> list[int]:
    """`m k...`: between low and high pool indices."""
    count = rng.randint(low, high)
    return [count] + [rng.randrange(pool) for _ in range(count)]


def sline_script(rng: random.Random, pool: int, count: int) -> list[float]:
    """Random commands of the intf_sline probe."""
    values: list[float] = [count]
    for _ in range(count):
        command = rng.randrange(13)
        values.append(command)
        if command in (0, 1, 6, 7):
            values.append(rng.randrange(pool))
        elif command in (4, 5, 12):
            values += indices(rng, pool, 0, 3)
        elif command == 9:
            values.append(rng.randint(-1, 5))
        elif command == 10:
            values.append(rng.randint(0, 3))
    return values


def tzone_script(rng: random.Random, pool: int, count: int) -> list[float]:
    """Random commands of the intf_tzone probe."""
    values: list[float] = [count]
    for _ in range(count):
        command = rng.randrange(11)
        values.append(command)
        if command in (0, 1, 4, 6, 7):
            values.append(rng.randrange(pool))
        elif command in (2, 3):
            values += [rng.randint(-1, 5), rng.randrange(pool)]
        elif command in (5, 9):
            values += indices(rng, pool, 0, 3)
        elif command == 8:
            values.append(rng.randint(-1, 5))
    return values


def interference_script(rng: random.Random, pool: int, count: int) -> list[float]:
    """Random commands of the intf_interference probe, weighted toward the two Insert."""
    values: list[float] = [count]
    for _ in range(count):
        command = rng.choice([0, 0, 1, 1, 1, 2, 2, 2, 2, 3, 4, 5, 6, 7])
        values.append(command)
        if command in (0, 1):
            values += indices(rng, pool, 1, 4)
        elif command == 2:
            values += [rng.randrange(pool), rng.randrange(pool)]
        elif command in (3, 5):
            values.append(rng.randrange(pool))
        elif command == 6:
            values.append(rng.randrange(2))
        elif command == 7:
            values.append(rng.choice([0.0, 1e-7, 0.5]))
    return values


def distinct_points(count: int) -> list[float]:
    """`count spec...`: points of distinct identities (EDGE i on both arguments)."""
    values: list[float] = [count]
    for i in range(count):
        values += [0, float(i), 0.0, 0.0, EDGE, 0, i, 0.5, EDGE, 0, i, 0.5, 1.0]
    return values


def append_script_requests(requests: list[str], rng: random.Random) -> None:
    """The section line, tangent zone and interference scripts."""
    # Segments A-B, C-D, then D-B: B ends the first line, D the second, which takes the
    # first reversed at its end; then the reverse joins at the beginning.
    requests.append('intf_interference ' + numbers(distinct_points(4) + [3, 2, 0, 1, 2, 2, 3, 2, 3, 1]))
    requests.append('intf_interference ' + numbers(distinct_points(4) + [3, 2, 0, 1, 2, 2, 3, 2, 0, 2]))
    requests.append('intf_interference ' + numbers(distinct_points(4) + [4, 2, 0, 1, 2, 1, 2, 2, 2, 3, 2, 3, 0]))
    for index in range(150):
        pool = 3 + index % 4
        requests.append('intf_sline ' + numbers(point_pool(rng, pool) + sline_script(rng, pool, 1 + index % 12)))
    for index in range(150):
        pool = 3 + index % 4
        requests.append('intf_tzone ' + numbers(point_pool(rng, pool) + tzone_script(rng, pool, 1 + index % 12)))
    for index in range(200):
        pool = 3 + index % 5
        requests.append('intf_interference ' + numbers(point_pool(rng, pool) + interference_script(rng, pool, 1 + index % 14)))


def append_intf_requests(requests: list[str]) -> None:
    """Append every Intf request family: the triangles, the records, the tool, the
    interferences."""
    append_triangle_requests(requests)
    rng = random.Random(1991_06_24)
    append_spoint_requests(requests, rng)
    append_script_requests(requests, rng)
    __import__('tkgeomalgo_corpus_intf_tool').append_intf_tool_requests(requests)
    __import__('tkgeomalgo_corpus_intf_poly').append_intf_poly_requests(requests)
