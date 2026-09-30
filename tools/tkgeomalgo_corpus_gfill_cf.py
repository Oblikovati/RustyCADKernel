"""Deterministic requests for the constrained filling of the GeomFill filling part
(tools/oracle/tkgeomalgo_probe_gfill_cf.cxx).

`gfill_bound` evaluates `GeomFill_SimpleBound` (lines, arcs, B-splines),
`GeomFill_DegeneratedBound` and `GeomFill_BoundWithSurf` (2d segments on planes,
cylinders and spheres) before and after `Reparametrize` (plain, reversed, with imposed
end derivatives, on other ranges), with the normals (Standard_Failure without a
constraint surface) and new tolerances. `gfill_corner` drives `GeomFill_CornerState`.
`gfill_patch` evaluates `GeomFill_CoonsAlgPatch` and the `GeomFill_TgtOnCoons` fields of
its four sides, then swaps its laws. `gfill_cfill` runs `GeomFill_ConstrainedFilling` on
three and four boundaries (constrained, free and mixed, given in any order and
orientation, with gaps and a degenerated side), with and without the corner checks,
with `SetDomain` and `ReBuild`, the `Check` members' text, and a second `Init` on the
same framework (which keeps the tangent fields and corner states of the first).
"""
from __future__ import annotations

import math

from tkgeomalgo_corpus_gfill import bspline, line, trimmed, circle

Point = tuple[float, float, float]


def _line(op: str, values: list[float]) -> str:
    return op + ' ' + ' '.join(repr(float(v)) for v in values)


Z: Point = (0.0, 0.0, 1.0)
X: Point = (1.0, 0.0, 0.0)
PLANE = [0, 0.0, 0.0, 0.0, *Z, *X]
TILTED = [0, 0.0, 0.0, 1.0, 0.0, 0.3, 1.0, 1.0, 0.0, 0.0]
CYLINDER = [1, 0.0, 0.0, 0.0, *Z, *X, 2.0]
SPHERE = [2, 0.0, 0.0, 0.0, *Z, *X, 3.0]


def with_surf(surface: list[float], p: tuple[float, float], d: tuple[float, float], length: float,
              tol3d: float = 1e-4, tolang: float = 1e-2) -> list[float]:
    """Kind 2: a BoundWithSurf on the 2d segment p + t d, t in [0, length]."""
    return [2, tol3d, tolang, *surface, *p, *d, 0.0, length]


def simple(curve: list[float], first: float = 0.0, last: float = 0.0,
           tol3d: float = 1e-4, tolang: float = 1e-2) -> list[float]:
    """Kind 0: a SimpleBound on the curve (on [first, last] when first < last)."""
    return [0, tol3d, tolang, first, last, *curve]


def degenerated(point: Point, first: float = 0.0, last: float = 1.0) -> list[float]:
    return [1, 1e-4, 1e-2, *point, first, last]


def _cyl(u: float, z: float) -> Point:
    return (2.0 * math.cos(u), 2.0 * math.sin(u), z)


def _segment(a: Point, b: Point) -> list[float]:
    d = (b[0] - a[0], b[1] - a[1], b[2] - a[2])
    return simple(line(a, d), 0.0, 1.0)


# The rectangle u in [0.3, 1.5], v in [0, 3] of the cylinder, as four chained sides.
CYL_SIDES = [
    with_surf(CYLINDER, (0.3, 0.0), (1.0, 0.0), 1.2),
    with_surf(CYLINDER, (1.5, 0.0), (0.0, 1.0), 3.0),
    with_surf(CYLINDER, (1.5, 3.0), (-1.0, 0.0), 1.2),
    with_surf(CYLINDER, (0.3, 3.0), (0.0, -1.0), 3.0),
]
CYL_FREE = [
    CYL_SIDES[0],
    _segment(_cyl(1.5, 0.0), _cyl(1.5, 3.0)),
    CYL_SIDES[2],
    _segment(_cyl(0.3, 3.0), _cyl(0.3, 0.0)),
]
# The rectangle u in [0.2, 1.2], v in [-0.5, 0.6] of the sphere.
SPH_SIDES = [
    with_surf(SPHERE, (0.2, -0.5), (1.0, 0.0), 1.0),
    with_surf(SPHERE, (1.2, -0.5), (0.0, 1.0), 1.1),
    with_surf(SPHERE, (1.2, 0.6), (-1.0, 0.0), 1.0),
    with_surf(SPHERE, (0.2, 0.6), (0.0, -1.0), 1.1),
]
# A square of the plane with B-spline and straight sides, and a triangle.
PLANE_SIDES = [
    simple(bspline(3, [0.0, 1.0], [4, 4], [(0, 0, 0), (1, -0.3, 0.2), (2, 0.3, -0.1), (3, 0, 0)])),
    _segment((3, 0, 0), (3, 3, 0)),
    simple(bspline(2, [0.0, 0.5, 1.0], [3, 1, 3], [(3, 3, 0), (2, 3.2, 0.3), (1, 2.9, 0.1), (0, 3, 0)])),
    _segment((0, 3, 0), (0, 0, 0)),
]
TRIANGLE = [
    with_surf(TILTED, (0.0, 0.0), (1.0, 0.0), 2.0),
    with_surf(TILTED, (2.0, 0.0), (-0.7071067811865476, 0.7071067811865476), 2.8284271247461903),
    with_surf(TILTED, (0.0, 2.0), (0.0, -1.0), 2.0),
]
TRIANGLE_FREE = [
    _segment((0, 0, 0), (2, 0, 0)),
    simple(bspline(2, [0.0, 1.0], [3, 3], [(2, 0, 0), (1.2, 1.2, 0.5), (0, 2, 0)])),
    _segment((0, 2, 0), (0, 0, 0)),
]


def _bound_requests(requests: list[str]) -> None:
    bounds = [*CYL_SIDES[:2], CYL_FREE[1], SPH_SIDES[0], SPH_SIDES[1], PLANE_SIDES[0], PLANE_SIDES[2],
              TRIANGLE[1], degenerated((1, 2, 3), 0.5, 2.0),
              simple(trimmed(0.3, 2.0, circle((0, 0, 1), (0, 0, 1), 2.0))),
              simple(circle((1, 1, 0), (0, 1, 0), 0.5)),
              simple(line((0, 0, 0), (1, 1, 1)), -1.0, 2.0)]
    ts = [5, 0.0, 0.25, 0.5, 0.8, 1.0]
    reparams = [[0, 1, 0, 0, 1, 1, 0], [0, 1, 0, 0, 1, 1, 1], [0, 1, 1, 1, 0.5, 2.0, 1],
                [2, 5, 1, 0, 1.5, 1, 0], [0, 1, 0, 1, 1, 0.25, 0]]
    for k, bound in enumerate(bounds):
        for reparam in reparams:
            requests.append(_line('gfill_bound', [*bound, *ts, *reparam, 1e-3 * (k + 1), 0.05]))


def _corner_requests(requests: list[str]) -> None:
    scripts = [
        [0, 0.1, 1, 0.2, 2, 0.3],
        [3, 0, 4, 0.5, 0, 1e-9],
        [4, 0.25, 3, 0, 2, 2.9],
        [3, 0, 3, 0, 4, 0.0, 4, 1.5],
        [1, -1.0, 0, 1e300, 2, 3.14159],
    ]
    for script in scripts:
        requests.append(_line('gfill_corner', [len(script), *script]))


def _patch_requests(requests: list[str]) -> None:
    uv = [8, 0.0, 0.0, 0.5, 0.5, 1.0, 0.3, 0.2, 1.0]
    sets = [CYL_SIDES, CYL_FREE, SPH_SIDES, PLANE_SIDES,
            [TRIANGLE[0], TRIANGLE[1], degenerated((0, 2, 0.6)), TRIANGLE[2]],
            [CYL_SIDES[1], CYL_SIDES[3], CYL_SIDES[0], CYL_SIDES[2]]]
    for sides in sets:
        requests.append(_line('gfill_patch', [v for side in sides for v in side] + uv))


def _cfill(sides: list[list[float]], *, deg: int = 8, seg: int = 3, nocheck: int = 0, checks: int = 0,
           doms: list[float] | None = None, again: list[list[float]] | None = None) -> str:
    values: list[float] = [deg, seg, nocheck, checks, len(sides)]
    for side in sides:
        values += side
    values += [len(doms or []), *(doms or [])]
    values.append(int(again is not None))
    if again is not None:
        values.append(len(again))
        for side in again:
            values += side
    return _line('gfill_cfill', values)


def _cfill_requests(requests: list[str]) -> None:
    sets = [CYL_SIDES, CYL_FREE, SPH_SIDES, PLANE_SIDES,
            [CYL_SIDES[2], CYL_SIDES[0], CYL_SIDES[3], CYL_SIDES[1]],
            [CYL_FREE[3], CYL_FREE[1], CYL_FREE[0], CYL_FREE[2]]]
    for sides in sets:
        for nocheck in (0, 1):
            requests.append(_cfill(sides, nocheck=nocheck, checks=1))
        requests.append(_cfill(sides, deg=10, seg=6))
    for sides in (TRIANGLE, TRIANGLE_FREE, [TRIANGLE[1], TRIANGLE[2], TRIANGLE[0]]):
        for nocheck in (0, 1):
            requests.append(_cfill(sides, nocheck=nocheck, checks=1))
    requests.append(_cfill(CYL_SIDES, doms=[0, 0.5, 2, 0.3]))
    requests.append(_cfill(CYL_SIDES, doms=[1, -0.7, 3, 0.4]))
    requests.append(_cfill(SPH_SIDES, doms=[0, 0.6, 1, 0.6, 2, 0.6, 3, 0.6]))
    requests.append(_cfill(CYL_FREE, doms=[0, 0.5, 1, 0.5]))
    requests.append(_cfill(TRIANGLE, doms=[0, 0.5]))
    # A second Init that constrains fewer sides than the first (CYL_SIDES then
    # PLANE_SIDES or TRIANGLE) is left out: it keeps the tangent poles of the first,
    # which no longer fit the grid, and the native code reads past its temporary
    # array there (undefined behaviour; Standard_OutOfRange in the port).
    requests.append(_cfill(PLANE_SIDES, again=CYL_SIDES))
    requests.append(_cfill(TRIANGLE_FREE, again=CYL_FREE))
    requests.append(_cfill(TRIANGLE, again=SPH_SIDES))
    requests.append(_cfill([CYL_SIDES[0], CYL_SIDES[1], degenerated(_cyl(0.3, 3.0)), CYL_SIDES[3]]))


def append_gfill_cf_requests(requests: list[str]) -> None:
    """Append the constrained-filling requests: boundaries, corners, patches, fillings."""
    _bound_requests(requests)
    _corner_requests(requests)
    _patch_requests(requests)
    _cfill_requests(requests)
