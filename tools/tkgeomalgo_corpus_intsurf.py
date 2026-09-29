"""Deterministic requests for the IntSurf package (tools/oracle/tkgeomalgo_probe_intsurf.cxx).

`intsurf_quadric_point` and `intsurf_quadric_uv` evaluate IntSurf_Quadric (and
IntSurf_QuadricTool::Tolerance) for the five quadric kinds and the undefined quadric, on
direct and indirect frames, at regular points and at the singular ones: on the axis, at
the sphere centre, at the cone apex, on the torus core circle and at the torus centre.
`intsurf_transition` drives IntSurf::MakeTransition through its four branches (null first
tangent, null second tangent or parallel tangents, IN/OUT, undecided mixed product) and
`intsurf_same` compares IntSurf_PntOn2S points with and without the 2d tolerance.
"""
from __future__ import annotations
import math
import random

# (location, direction, x direction) frames: direct, indirect and oblique.
FRAMES = [
    ([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]),
    ([1.0, -2.0, 0.5], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0]),
    ([0.25, 0.5, -1.0], [1.0, 2.0, 2.0], [2.0, -1.0, 0.0]),
]
# (kind, r1, r2): plane, cylinder, sphere, cone (reference radius, semi-angle), torus, undefined.
KINDS = [(0, 0.0, 0.0), (1, 2.0, 0.0), (1, 0.0, 0.0), (2, 1.5, 0.0), (3, 1.0, 0.4), (3, 0.0, -0.7),
         (4, 3.0, 1.0), (4, 1.0, 2.0), (5, 0.0, 0.0)]


def _fmt(values: list[float]) -> str:
    return ' '.join(repr(float(v)) for v in values)


def _quadric(kind: int, frame: tuple[list[float], ...], r1: float, r2: float) -> list[float]:
    location, direction, x_direction = frame
    return [kind, *location, *direction, *x_direction, r1, r2]


def _special_points(frame: tuple[list[float], ...], r1: float) -> list[list[float]]:
    """The frame origin, a point on the axis, a point at distance r1 off the axis, a far point."""
    location, direction, x_direction = frame
    norm = math.sqrt(sum(c * c for c in direction))
    xnorm = math.sqrt(sum(c * c for c in x_direction))
    on_axis = [location[i] + 2.0 * direction[i] / norm for i in range(3)]
    on_core = [location[i] + r1 * x_direction[i] / xnorm for i in range(3)]
    return [list(location), on_axis, on_core, [7.5, -3.25, 11.0]]


def _quadric_requests(requests: list[str], rng: random.Random) -> None:
    for frame in FRAMES:
        for kind, r1, r2 in KINDS:
            quadric = _quadric(kind, frame, r1, r2)
            points = _special_points(frame, r1) + [[rng.uniform(-4, 4) for _ in range(3)] for _ in range(3)]
            for point in points:
                requests.append('intsurf_quadric_point ' + _fmt(quadric + point))
            for u, v in [(0.0, 0.0), (1.25, -0.5), (math.pi, 2.0), (rng.uniform(0, 6.2), rng.uniform(-1, 1))]:
                requests.append('intsurf_quadric_uv ' + _fmt(quadric + [u, v]))


def _transition_requests(requests: list[str], rng: random.Random) -> None:
    fixed = [
        ([0, 0, 0], [1, 0, 0], [0, 0, 1]), ([1e-8, 0, 0], [0, 1, 0], [0, 0, 1]),
        ([1, 0, 0], [0, 0, 0], [0, 0, 1]), ([1, 0, 0], [2, 0, 0], [0, 0, 1]),
        ([1, 0, 0], [-3, 1e-13, 0], [0, 0, 1]), ([1, 0, 0], [0, 1, 0], [0, 0, 1]),
        ([0, 1, 0], [1, 0, 0], [0, 0, 1]), ([1, 0, 0], [0, 1, 0], [1, 0, 0]),
        ([1, 0, 0], [1, 1e-5, 0], [0, 0, 1]), ([1, 0, 0], [1, 1e-3, 0], [0, 0, -1]),
    ]
    for t1, t2, n in fixed:
        requests.append('intsurf_transition ' + _fmt([*t1, *t2, *n]))
    for _ in range(20):
        values = [rng.uniform(-2, 2) for _ in range(6)] + [rng.uniform(0.1, 1.0) for _ in range(3)]
        requests.append('intsurf_transition ' + _fmt(values))


def _same_requests(requests: list[str]) -> None:
    base = [0.0, 0.0, 0.0, 0.5, 0.5, 1.5, 1.5]
    cases = [
        (base, 0.0, -1.0), ([1e-8, 0.0, 0.0, 0.5, 0.5, 1.5, 1.5], 1e-7, -1.0),
        ([1e-6, 0.0, 0.0, 0.5, 0.5, 1.5, 1.5], 1e-7, -1.0), ([0.0, 0.0, 0.0, 0.5, 0.6, 1.5, 1.5], 0.0, 0.05),
        ([0.0, 0.0, 0.0, 0.5, 0.6, 1.5, 1.5], 0.0, 0.2), ([0.0, 0.0, 0.0, 0.5, 0.5, 1.5, 1.8], 1.0, 0.2),
        ([0.0, 0.0, 0.0, 0.5, 0.5, 1.5, 1.5], 0.0, 0.0),
    ]
    for other, tol3, tol2 in cases:
        requests.append('intsurf_same ' + _fmt(base + other + [tol3, tol2]))


def append_intsurf_requests(requests: list[str]) -> None:
    """Append the IntSurf requests: quadrics, transitions, then point comparisons."""
    rng = random.Random(20260930)
    _quadric_requests(requests, rng)
    _transition_requests(requests, rng)
    _same_requests(requests)
