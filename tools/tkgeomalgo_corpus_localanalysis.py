"""Deterministic requests for the LocalAnalysis package
(tools/oracle/tkgeomalgo_probe_localanalysis.cxx).

`la_curve` compares every query of LocalAnalysis_CurveContinuity and the Dump text for
the seven GeomAbs_Shape orders on joined and broken curve pairs: a line tangent to a
circle, two arcs with the same or different curvature, Bezier curves with null
derivatives (NullFirst/SecondDerivative, TangentNotDefined), straight pieces (the null
curvature of G2) and reversed tangents, with default and tight tolerances.
`la_surface` does the same for LocalAnalysis_SurfaceContinuity at points, on two 2d
lines and through ComputeAnalysis on local properties: planes, cylinders, spheres and
tori meeting with and without tangency, at poles (undefined normals) and umbilics.
"""
from __future__ import annotations
import math

DEFAULT_CURVE_EPS = [0.001, 0.001, 0.001, 0.001, 0.001, 0.001, 0.01, 10000.0]
TIGHT_CURVE_EPS = [1e-9, 1e-9, 1e-9, 1e-9, 1e-9, 1e-9, 1e-6, 100.0]
DEFAULT_SURF_EPS = [0.001, 0.001, 0.001, 0.001, 0.001, 0.01, 10000.0]
TIGHT_SURF_EPS = [1e-9, 1e-9, 1e-9, 1e-9, 1e-9, 1e-6, 100.0]


def _line(op: str, values: list[float]) -> str:
    return op + ' ' + ' '.join(repr(float(v)) for v in values)


def _circle(c: list[float], n: list[float], radius: float) -> list[float]:
    return [0, *c, *n, radius]


def _straight(p: list[float], d: list[float]) -> list[float]:
    return [1, *p, *d]


def _bezier(*poles: list[float]) -> list[float]:
    return [2] + [x for p in poles for x in p]


def _curve_pairs() -> list[tuple[list[float], float, list[float], float]]:
    unit = _circle([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1.0)
    return [
        (_straight([1.0, -1.0, 0.0], [0.0, 1.0, 0.0]), 1.0, unit, 0.0),
        (unit, 0.0, _straight([1.0, 0.0, 0.0], [0.0, -1.0, 0.0]), 0.0),
        (unit, math.pi / 2, _circle([0.0, -1.0, 0.0], [0.0, 0.0, 1.0], 2.0), math.pi / 2),
        (unit, 1.0, unit, 1.0),
        (unit, 1.0, _circle([0.0, 0.0, 0.0], [0.0, 0.0, -1.0], 1.0), -1.0),
        (_straight([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]), 1.0, _straight([1.0, 0.0, 0.0], [1.0, 0.0, 0.0]), 0.0),
        (_straight([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]), 1.0, _straight([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]), 0.0),
        (_bezier([0, 0, 0], [0, 0, 0], [1, 1, 0], [2, 0, 0]), 0.0, _bezier([0, 0, 0], [1, 0, 0], [2, 1, 0], [3, 0, 0]), 0.0),
        (_bezier([0, 0, 0], [1, 0, 0], [2, 0, 0], [3, 0, 0]), 1.0, _bezier([3, 0, 0], [4, 0, 0], [5, 0, 0], [6, 0, 0]), 0.0),
        (_bezier([0, 0, 0], [1, 1, 0], [2, 1, 0], [3, 0, 0]), 1.0, _bezier([3, 0, 0], [4, -1, 0], [5, -1, 0], [6, 0, 0]), 0.0),
        (_bezier([0, 0, 0], [1, 1, 0], [2, 1, 0], [3, 0, 0]), 1.0, _bezier([3, 0, 0], [3, 0, 0], [3, 0, 0], [6, 1, 0]), 0.0),
        (_bezier([0, 0, 0], [1, 2, 0], [2, 2, 1], [3, 0, 1]), 0.5, _bezier([0, 0, 0], [1, 2, 0], [2, 2, 1], [3, 0, 1]), 0.5),
    ]


def _surface(kind: int, p: list[float], d: list[float], r1: float = 0.0, r2: float = 0.0) -> list[float]:
    return [kind, *p, *d, r1, r2]


def _surface_points() -> list[tuple[list[float], list[float], list[float]]]:
    """(surface1, surface2, u1 v1 u2 v2)."""
    z = [0.0, 0.0, 1.0]
    origin = [0.0, 0.0, 0.0]
    plane = _surface(0, [0.0, 0.0, 1.0], z)
    cyl = _surface(1, origin, z, 1.0)
    sphere = _surface(2, [0.0, 0.0, 0.0], z, 1.0)
    return [
        (plane, plane, [0.0, 0.0, 0.0, 0.0]),
        (plane, sphere, [0.0, 0.0, 0.0, math.pi / 2]),
        (cyl, _surface(0, [1.0, 0.0, 0.0], [1.0, 0.0, 0.0]), [0.0, 0.5, 0.0, 0.5]),
        (cyl, cyl, [0.3, 0.2, 0.3, 0.2]),
        (cyl, _surface(1, origin, z, 1.5), [0.0, 0.0, 0.0, 0.0]),
        (sphere, sphere, [0.4, 0.3, 0.4, 0.3]),
        (sphere, _surface(2, [0.0, 0.0, 0.0], z, 2.0), [0.4, 0.3, 0.4, 0.3]),
        (sphere, cyl, [0.0, 0.0, 0.0, 0.0]),
        (_surface(3, origin, z, 3.0, 1.0), _surface(3, origin, z, 3.0, 1.0), [0.5, 0.25, 0.5, 0.25]),
        (_surface(3, origin, z, 3.0, 1.0), _surface(1, origin, z, 4.0), [0.0, 0.0, 0.0, 0.0]),
        (sphere, sphere, [0.0, math.pi / 2, 1.0, math.pi / 2]),
    ]


def append_localanalysis_requests(requests: list[str]) -> None:
    """Append the LocalAnalysis requests: curve pairs, then surface pairs."""
    for c1, u1, c2, u2 in _curve_pairs():
        for order in range(7):
            for eps in (DEFAULT_CURVE_EPS, TIGHT_CURVE_EPS):
                requests.append(_line('la_curve', c1 + [u1] + c2 + [u2, order] + eps))
    for s1, s2, params in _surface_points():
        for order in range(7):
            for eps in (DEFAULT_SURF_EPS, TIGHT_SURF_EPS):
                requests.append(_line('la_surface', [0] + s1 + s2 + [order] + eps + params))
            requests.append(_line('la_surface', [2] + s1 + s2 + [order] + DEFAULT_SURF_EPS + params))
    plane = _surface(0, [0.0, 0.0, 1.0], [0.0, 0.0, 1.0])
    sphere = _surface(2, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1.0)
    for order in range(7):
        lines = [0.0, 0.0, 1.0, 0.0, 0.0, math.pi / 2, 1.0, 0.0, 0.0]
        requests.append(_line('la_surface', [1] + plane + sphere + [order] + DEFAULT_SURF_EPS + lines))
