"""Deterministic requests for the IntImp package
(tools/oracle/tkgeomalgo_probe_intimp.cxx).

`ii_choix` covers `ChoixRef` inside and outside 0..3. `ii_tangence` drives
`IntImp_ComputeTangence` through its early exits (null derivatives and normals),
tangent patches (small components and parallel normals) and the sorted
isoparametrics, with seeded random derivatives. The generic templates run through
their TKGeomAlgo instances: `ii_zerpar` (`IntImp_ZerParFunc` as
IntWalk_TheFunctionOfTheInt2S) and `ii_int2s` (`IntImp_Int2S` as IntWalk_TheInt2S, from
a start point and on each isoparametric, inside and beyond the patch bounds, on
tangent patches) on planes, cylinders, spheres and tori; `ii_zerimp`
(`IntImp_ZerImpFunc` as IntPatch_TheSurfFunction) on patches against quadrics,
including tangent ones; `ii_intcs` (`IntImp_ZerCSParFunc` and `IntImp_IntCS` as
IntCurveSurface_TheCSFunctionOfHInter and IntCurveSurface_TheExactHInter) on lines
and circles against patches, with and without margin, crossing and missing.
"""
from __future__ import annotations
import math
import random

TWO_PI = 2.0 * math.pi
HALF_PI = math.pi / 2.0


def _line(op: str, values: list[float]) -> str:
    return op + ' ' + ' '.join(repr(float(v)) for v in values)


def _surface(kind: int, loc: list[float], d: list[float], r1: float, r2: float,
             bounds: list[float]) -> list[float]:
    return [kind, *loc, *d, r1, r2, *bounds]


PLANE = _surface(0, [0.0, 0.0, 0.3], [0.0, 0.0, 1.0], 0.0, 0.0, [-2.0, 2.0, -2.0, 2.0])
SMALL_PLANE = _surface(0, [0.0, 0.0, 0.3], [0.0, 0.0, 1.0], 0.0, 0.0, [-0.2, 0.2, -0.2, 0.2])
TILTED = _surface(0, [0.1, 0.0, 0.0], [1.0, 0.2, 0.4], 0.0, 0.0, [-3.0, 3.0, -3.0, 3.0])
CYLINDER = _surface(1, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 1.0, 0.0, [0.0, TWO_PI, -2.0, 2.0])
CYLINDER_Z = _surface(1, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 0.8, 0.0, [0.0, TWO_PI, -2.0, 2.0])
SPHERE = _surface(2, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1.5, 0.0, [0.0, TWO_PI, -HALF_PI, HALF_PI])
SPHERE2 = _surface(2, [1.0, 0.2, 0.0], [0.0, 0.0, 1.0], 1.0, 0.0, [0.0, TWO_PI, -HALF_PI, HALF_PI])
TORUS = _surface(3, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 3.0, 1.0, [0.0, TWO_PI, 0.0, TWO_PI])
TANGENT_PLANE = _surface(0, [0.0, 0.0, 1.5], [0.0, 0.0, 1.0], 0.0, 0.0, [-2.0, 2.0, -2.0, 2.0])

# (s1, s2, start parameters u1 v1 u2 v2 near an intersection point)
PAIRS = [
    (PLANE, SPHERE, [1.2, 0.5, 0.4, 0.25]),
    (CYLINDER, PLANE, [1.0, 0.3, 0.3, 0.5]),
    (SPHERE, SPHERE2, [0.3, 0.6, 2.0, 0.7]),
    (CYLINDER, CYLINDER_Z, [0.7, 0.5, 0.2, 0.6]),
    (TORUS, TILTED, [0.4, 1.3, 0.8, 0.9]),
]


def _quadric(kind: int, loc: list[float], d: list[float], xd: list[float], r1: float, r2: float) -> list[float]:
    return [kind, *loc, *d, *xd, r1, r2]


def _tangence_requests(requests: list[str]) -> None:
    rng = random.Random(1402)
    for _ in range(30):
        dpuv = [round(rng.uniform(-2.0, 2.0), 6) for _ in range(12)]
        eps = [10.0 ** rng.randint(-9, -1) for _ in range(4)]
        requests.append(_line('ii_tangence', dpuv + eps))
    fixed = [
        [1, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 1],           # perpendicular patches
        [1, 0, 0, 0, 1, 0, 1, 0, 0, 0, 1, 0],           # the same tangent plane
        [1, 0, 0, 0, 1, 0, 1, 0, 1e-12, 0, 1, 0],       # almost the same plane
        [0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 1],           # a null derivative
        [1, 0, 0, 2, 0, 0, 1, 0, 0, 0, 0, 1],           # parallel derivatives: null normal
        [1, 0, 0, 0, 1, 0, 0, 1, 1, 1, 0, 0],
        [3, 1, 0, -1, 2, 0.5, 0.2, 0.1, 1, 0.5, 2, -1],
    ]
    for dpuv in fixed:
        for eps in ([1e-7] * 4, [0.5] * 4):
            requests.append(_line('ii_tangence', dpuv + eps))


def _intersection_requests(requests: list[str]) -> None:
    for s1, s2, start in PAIRS:
        for iso in range(4):
            requests.append(_line('ii_zerpar', s1 + s2 + [iso] + start))
        requests.append(_line('ii_int2s', s1 + s2 + [1e-6, 0, 0] + start))
        requests.append(_line('ii_int2s', s1 + s2 + [1e-3, 0, 0] + [x + 0.1 for x in start]))
        for iso in range(4):
            requests.append(_line('ii_int2s', s1 + s2 + [1e-6, 1, iso] + start))
    # Solutions beyond the bounds of a small patch, a tangent contact, no solution.
    requests.append(_line('ii_int2s', SMALL_PLANE + SPHERE + [1e-6, 0, 0, 0.15, 0.1, 0.4, 0.25]))
    requests.append(_line('ii_int2s', SMALL_PLANE + SPHERE + [1e-6, 0, 0, -0.18, 0.19, 2.5, 0.3]))
    requests.append(_line('ii_int2s', SPHERE + SMALL_PLANE + [1e-6, 0, 0, 0.4, 0.25, 0.15, 0.1]))
    requests.append(_line('ii_int2s', TANGENT_PLANE + SPHERE + [1e-6, 0, 0, 0.0, 0.0, 0.3, 1.5]))
    requests.append(_line('ii_int2s', PLANE + SPHERE2 + [1e-9, 0, 0, 5.0, 5.0, 0.0, -1.0]))
    requests.append(_line('ii_zerpar', TANGENT_PLANE + SPHERE + [2, 0.0, 0.0, 0.3, 1.5]))


def _zerimp_requests(requests: list[str]) -> None:
    z, x, y = [0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]
    sphere_q = _quadric(2, [0.0, 0.0, 0.0], z, x, 1.2, 0.0)
    plane_q = _quadric(0, [0.0, 0.0, 0.2], z, x, 0.0, 0.0)
    cylinder_q = _quadric(1, [0.0, 0.0, 0.0], y, z, 0.9, 0.0)
    cone_q = _quadric(3, [0.0, 0.0, -1.0], z, x, 0.5, 0.4)
    torus_q = _quadric(4, [0.0, 0.0, 0.0], x, y, 2.0, 0.5)
    cases = [
        (PLANE, sphere_q, [0.5, 0.4]), (PLANE, cylinder_q, [0.2, 0.9]),
        (CYLINDER, plane_q, [1.2, 0.1]), (SPHERE, cylinder_q, [0.7, 0.2]),
        (TORUS, plane_q, [0.3, 1.0]), (SPHERE, cone_q, [2.0, -0.4]),
        (CYLINDER_Z, torus_q, [0.5, 0.2]), (SPHERE, sphere_q, [0.1, 0.1]),
        (PLANE, _quadric(0, [0.0, 0.0, 0.0], z, x, 0.0, 0.0), [0.1, 0.2]),
        (TANGENT_PLANE, _quadric(2, [0.0, 0.0, 0.0], z, x, 1.5, 0.0), [0.0, 0.0]),
    ]
    for surface, quadric, uv in cases:
        for tol in (1e-7, 1e-3):
            requests.append(_line('ii_zerimp', surface + quadric + [tol] + uv))


def _intcs_requests(requests: list[str]) -> None:
    def curve(kind: int, loc: list[float], d: list[float], r: float, first: float, last: float) -> list[float]:
        return [kind, *loc, *d, r, first, last]
    down = curve(0, [0.2, 0.1, 2.0], [0.0, 0.0, -1.0], 0.0, 0.0, 4.0)
    slant = curve(0, [-2.0, -0.5, -1.0], [1.0, 0.25, 0.5], 0.0, 0.0, 6.0)
    circle = curve(1, [0.0, 0.0, 0.5], [1.0, 0.0, 0.0], 1.2, 0.0, TWO_PI)
    parallel = curve(0, [0.0, 0.0, 0.5], [1.0, 0.0, 0.0], 0.0, -2.0, 2.0)
    missing = curve(0, [0.0, 3.0, 0.0], [1.0, 0.0, 0.0], 0.0, -5.0, 5.0)
    cases = [
        (PLANE, down, [0.1, 0.2, 1.5]), (PLANE, slant, [0.0, 0.0, 2.0]),
        (SPHERE, down, [0.3, 1.0, 0.4]), (SPHERE, slant, [0.2, -0.3, 1.0]),
        (CYLINDER, circle, [0.5, 0.0, 1.0]), (CYLINDER_Z, slant, [3.5, -0.5, 1.2]),
        (TORUS, down, [0.1, 1.5, 1.0]), (PLANE, parallel, [0.0, 0.0, 0.0]),
        (SPHERE, missing, [1.5, 0.0, 0.0]), (SMALL_PLANE, slant, [0.0, 0.0, 2.6]),
    ]
    for surface, crv, start in cases:
        for tol, margin in ((1e-7, 0.0), (1e-3, 0.1)):
            requests.append(_line('ii_intcs', surface + crv + start + [tol, margin]))


def append_intimp_requests(requests: list[str]) -> None:
    """Append the IntImp requests: ChoixRef, ComputeTangence, then the templates."""
    for index in (-1, 0, 1, 2, 3, 4):
        requests.append(_line('ii_choix', [index]))
    _tangence_requests(requests)
    _intersection_requests(requests)
    _zerimp_requests(requests)
    _intcs_requests(requests)
