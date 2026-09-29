"""Deterministic requests for the Intf package (tools/oracle/tkgeomalgo_probe_intf.cxx).

`intf_triangle` feeds Intf::PlaneEquation and Intf::Contain one triangle and one point:
regular triangles in both orientations, degenerate ones (coincident and collinear
vertices, a normal that underflows below gp::Resolution), and points inside, on the
edges and vertices, outside and off the plane.
"""
from __future__ import annotations
import random


def triangle_request(p1: list[float], p2: list[float], p3: list[float], point: list[float]) -> str:
    """One `intf_triangle p1 p2 p3 point` request line."""
    return 'intf_triangle ' + ' '.join(repr(float(v)) for v in (*p1, *p2, *p3, *point))


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


def append_intf_requests(requests: list[str]) -> None:
    """Append the Intf requests: the fixed cases, then seeded random triangles."""
    for case in fixed_triangles():
        requests.append(triangle_request(*case))
    rng = random.Random(20260929)
    for _ in range(40):
        points = [[rng.uniform(-10.0, 10.0) for _ in range(3)] for _ in range(4)]
        requests.append(triangle_request(*points))
