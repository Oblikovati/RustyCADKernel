"""Deterministic requests for the IntWalk package
(tools/oracle/tkgeomalgo_probe_intwalk.cxx and tkgeomalgo_probe_iwalk.cxx).

`pw_walk` marches IntWalk_PWalking between pairs of parametric surfaces (planes,
cylinders, spheres, tori, cones, Bezier and B-spline surfaces): closed lines, open lines
ending on the boundaries, near-tangent and tangent contacts, start points without
solution, both constructors and both Performs (with a parameter box), several
deflections and increments, then PutToBoundary and SeekAdditionalPoints; the report
covers every consultation member and the out-of-range indices. `pw_first` runs
PerformFirstPoint; `pw_edit` the public stepping members (TestDeflection, TestArret,
RepartirOuDiviser, RemoveAPoint, AddAPoint) after a Perform; `pw_min` the protected
searches (the distance minimizations, the singular point, ComputePasInit,
SeekPointOnBoundary and ExtendLineInCommonZone). `iw_walk` instantiates
IntWalk_IWalking.gxx on a probe class with IntPatch_TheSurfFunction: quadrics walked on
patches from path points on the boundaries (open lines, passing and tangent points,
multiplicities on a seam), from interior points (closed lines, lines that open on the
border), reversed, with the hole filling, the open-line Perform after the full one and
an object never performed.
"""
from __future__ import annotations
import math

TWO_PI = 2.0 * math.pi
HALF_PI = math.pi / 2.0


def _line(op: str, values: list[float]) -> str:
    return op + ' ' + ' '.join(repr(float(v)) for v in values)


def _analytic(kind: int, loc: list[float], d: list[float], r1: float, r2: float, bounds: list[float]) -> list[float]:
    return [kind, *loc, *d, r1, r2, *bounds]


def _bezier(poles: list[list[list[float]]], bounds: list[float]) -> list[float]:
    flat = [c for row in poles for p in row for c in p]
    return [5, len(poles), len(poles[0]), *flat, *bounds]


def _bspline(du: int, dv: int, poles: list[list[list[float]]], bounds: list[float]) -> list[float]:
    flat = [c for row in poles for p in row for c in p]
    return [6, du, dv, len(poles), len(poles[0]), *flat, *bounds]


def _grid(n: int, m: int, height, scale: float = 1.0) -> list[list[list[float]]]:
    return [[[scale * i / (n - 1), scale * j / (m - 1), height(i, j)] for j in range(m)] for i in range(n)]


Z, X, Y = [0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]
PLANE = _analytic(0, [0.0, 0.0, 0.0], Z, 0.0, 0.0, [-2.0, 2.0, -2.0, 2.0])
PLANE_HALF = _analytic(0, [0.0, 0.0, 0.2], Z, 0.0, 0.0, [-0.8, 0.8, -0.8, 0.8])
TILTED = _analytic(0, [0.1, 0.0, 0.0], [1.0, 0.2, 0.4], 0.0, 0.0, [-5.0, 5.0, -5.0, 5.0])
CYL_X = _analytic(1, [0.0, 0.0, 0.0], X, 1.0, 0.0, [0.0, TWO_PI, -2.0, 2.0])
CYL_Z = _analytic(1, [0.0, 0.0, 0.0], Z, 0.8, 0.0, [0.0, TWO_PI, -2.0, 2.0])
CYL_Y_SHORT = _analytic(1, [0.0, 0.0, 0.0], Y, 0.7, 0.0, [0.5, 2.5, -1.0, 1.0])
SPHERE = _analytic(2, [0.0, 0.0, 0.3], Z, 1.0, 0.0, [0.0, TWO_PI, -HALF_PI, HALF_PI])
SPHERE2 = _analytic(2, [1.0, 0.2, 0.0], Z, 1.0, 0.0, [0.0, TWO_PI, -HALF_PI, HALF_PI])
SPHERE_CAP = _analytic(2, [0.0, 0.0, 0.0], Z, 1.5, 0.0, [0.0, TWO_PI, 0.0, HALF_PI])
TORUS = _analytic(3, [0.0, 0.0, 0.0], Z, 3.0, 1.0, [0.0, TWO_PI, 0.0, TWO_PI])
CONE = _analytic(4, [0.0, 0.0, -1.0], Z, 0.5, 0.4, [0.0, TWO_PI, 0.0, 3.0])
TANGENT_PLANE = _analytic(0, [0.0, 0.0, 1.3], Z, 0.0, 0.0, [-2.0, 2.0, -2.0, 2.0])
NEAR_TANGENT = _analytic(0, [0.0, 0.0, 1.2999], Z, 0.0, 0.0, [-2.0, 2.0, -2.0, 2.0])
BEZIER = _bezier(_grid(4, 4, lambda i, j: 0.4 * math.sin(1.3 * i + 0.7 * j) - 0.1), [0.0, 1.0, 0.0, 1.0])
BEZIER_FLAT = _bezier(_grid(3, 3, lambda i, j: 0.05 * i * j), [0.0, 1.0, 0.0, 1.0])
BSPLINE = _bspline(3, 3, _grid(6, 6, lambda i, j: 0.3 * math.cos(0.9 * i - 0.5 * j), 2.0), [0.0, 3.0, 0.0, 3.0])
BSPLINE2 = _bspline(2, 3, _grid(5, 4, lambda i, j: 0.2 * (i - 2) * (j - 1.5) - 0.05, 1.5), [0.0, 3.0, 0.0, 1.0])
CUT_PLANE_BEZ = _analytic(0, [0.0, 0.0, 0.05], Z, 0.0, 0.0, [-1.0, 2.0, -1.0, 2.0])
CUT_PLANE_BSP = _analytic(0, [0.0, 0.0, 0.1], Z, 0.0, 0.0, [-1.0, 3.0, -1.0, 3.0])
CUT_TILTED = _analytic(0, [0.5, 0.5, 0.0], [0.3, -0.2, 1.0], 0.0, 0.0, [-2.0, 2.0, -2.0, 2.0])
# A plane cutting a closed circle off the sphere, inside both parameter spaces.
PLANE_Y = _analytic(0, [0.0, 0.5, 0.0], Y, 0.0, 0.0, [-2.0, 2.0, -2.0, 2.0])
SPHERE_O = _analytic(2, [0.0, 0.0, 0.0], Z, 1.0, 0.0, [0.0, TWO_PI, -HALF_PI, HALF_PI])
# Viviani's curve: it crosses itself at a tangent point of the two surfaces.
SPHERE_V = _analytic(2, [0.0, 0.0, 0.0], Z, 2.0, 0.0, [0.0, TWO_PI, -HALF_PI, HALF_PI])
CYL_V = _analytic(1, [1.0, 0.0, 0.0], Z, 1.0, 0.0, [0.0, TWO_PI, -3.0, 3.0])
# A B-spline flat (z = 0) on its first spans: a tangent zone with the plane z = 0.
# A circle through the corner (0.5, 0.5) of the plane patch.
PLANE_CORNER = _analytic(0, [0.0, 0.0, 0.0], Z, 0.0, 0.0, [-1.0, 0.5, -1.0, 0.5])
SPHERE_C = _analytic(2, [0.0, 0.0, 0.3], Z, math.sqrt(0.59), 0.0, [0.0, TWO_PI, -HALF_PI, HALF_PI])
BSPLINE_FLAT = _bspline(2, 2, _grid(6, 4, lambda i, j: 0.0 if i <= 2 else 0.3 * (j - 1.5)), [0.0, 4.0, 0.0, 2.0])

# (s1, s2, start u1 v1 u2 v2): a start near a point of the intersection.
PAIRS = [
    (PLANE, SPHERE, [0.9539, 0.0, 0.0, -0.3047]),
    (SPHERE, PLANE, [1.2, -0.3, 0.34, 0.9]),
    (PLANE_HALF, SPHERE, [0.7036, 0.7036, 0.7854, -0.1002]),
    (CYL_X, PLANE, [1.0, 0.3, 0.3, 0.84]),
    (CYL_X, CYL_Z, [6.123, 0.784, 0.2, 0.987]),
    (CYL_Z, CYL_Y_SHORT, [1.0, 0.5, 1.2, 0.4]),
    (SPHERE, SPHERE2, [0.3, 0.6, 2.0, 0.7]),
    (TORUS, TILTED, [1.8338, 1.0, 1.1602, -3.477]),
    (TORUS, PLANE, [0.5, 1.6, 2.0, 1.5]),
    (CONE, PLANE, [1.0, 1.25, 0.2, 0.5]),
    (CONE, CYL_X, [1.5, 1.6, 1.4, 0.3]),
    (SPHERE_CAP, CYL_Z, [0.5, 0.6, 0.5, 1.0]),
    (BEZIER, CUT_PLANE_BEZ, [0.3, 0.3, 0.3, 0.3]),
    (CUT_PLANE_BEZ, BEZIER, [0.6, 0.2, 0.6, 0.2]),
    (BSPLINE, CUT_PLANE_BSP, [1.0, 1.0, 0.7, 0.7]),
    (BSPLINE, CUT_TILTED, [0.8, 1.5, 0.3, 0.2]),
    (BSPLINE2, BEZIER, [1.0, 0.5, 0.5, 0.5]),
    (BEZIER_FLAT, CUT_TILTED, [0.5, 0.5, 0.0, 0.0]),
    (SPHERE, NEAR_TANGENT, [0.0, 1.5, 0.0, 0.0]),
    (SPHERE, TANGENT_PLANE, [0.0, HALF_PI, 0.0, 0.0]),
    (SPHERE, SPHERE, [0.3, 0.2, 0.3, 0.2]),
    (PLANE, SPHERE2, [5.0, 5.0, 0.0, -1.0]),
    (PLANE_Y, SPHERE_O, [0.866, 0.0, HALF_PI, 1.0472]),
    (SPHERE_V, CYL_V, [math.pi / 4.0, math.pi / 4.0, HALF_PI, 1.414]),
    (CYL_V, SPHERE_V, [2.5, 1.9, 1.2, 0.9]),
    (BSPLINE_FLAT, PLANE, [3.5, 1.0, 0.8, 0.5]),
    (PLANE, BSPLINE_FLAT, [0.7, 0.5, 3.2, 1.0]),
    (PLANE_CORNER, SPHERE_C, [math.sqrt(0.5), 0.0, 0.0, -math.asin(0.3 / math.sqrt(0.59))]),
]
WALKS = [(1e-6, 1e-8, 0.01, 0.1), (1e-6, 1e-8, 0.001, 0.05), (1e-5, 1e-6, 0.1, 0.3), (1e-6, 1e-3, 0.05, 0.2),
         (1e-6, 1e-2, 0.1, 0.1), (1e-6, 1e-2, 0.001, 0.1), (1e-6, 0.05, 0.002, 0.2)]


def _pw_requests(requests: list[str]) -> None:
    for s1, s2, start in PAIRS:
        for tt, eps, defl, incr in WALKS:
            requests.append(_line('pw_walk', s1 + s2 + [tt, eps, defl, incr, 0, *start, 0, 0]))
        tt, eps, defl, incr = WALKS[0]
        requests.append(_line('pw_walk', s1 + s2 + [tt, eps, defl, incr, 2, *start, 3, 40]))
        box = [start[0] - 0.5, start[1] - 0.5, start[2] - 0.5, start[3] - 0.5,
               start[0] + 0.5, start[1] + 0.5, start[2] + 0.5, start[3] + 0.5]
        requests.append(_line('pw_walk', s1 + s2 + [tt, eps, defl, incr, 1, *start, *box, 1, 0]))
        requests.append(_line('pw_walk', s1 + s2 + [tt, eps, defl, 0.25, 0, *start, 2, 60]))
        requests.append(_line('pw_first', s1 + s2 + [tt, eps, defl, incr, *start]))
        for iso in range(4):
            requests.append(_line('pw_edit', s1 + s2 + [tt, eps, defl, incr, *start, iso, [1, 2, 9999, 0][iso]]))


def _pw_min_requests(requests: list[str]) -> None:
    tt, eps, defl, incr = WALKS[0]
    spline_pairs = [(BEZIER, BSPLINE2, [0.3, 0.4, 0.8, 0.6]), (BSPLINE, BEZIER, [1.1, 0.9, 0.4, 0.5]),
                    (BEZIER, CUT_PLANE_BEZ, [0.3, 0.3, 0.2, 0.3])]
    for s1, s2, pnt in spline_pairs + [(PLANE, SPHERE, [0.9, 0.1, 0.1, -0.3])]:
        for has, step in ((0, [0.0] * 4), (1, [1e-3, 2e-3, 1e-3, 5e-4])):
            requests.append(_line('pw_min', [0, *s1, *s2, tt, eps, defl, incr, *pnt, has, *step]))
            requests.append(_line('pw_min', [1, *s1, *s2, tt, eps, defl, incr, *pnt, has, *step[:2]]))
        requests.append(_line('pw_min', [3, *s1, *s2, tt, eps, defl, incr, *pnt, 0.1, 0.2, 1e3, 0.0]))
    singular = [(SPHERE, PLANE, [0.0, HALF_PI, 0.0, 1.3]), (PLANE, SPHERE, [-2.0, 0.3, 1.0, -0.2]),
                (BEZIER, CUT_PLANE_BEZ, [0.0, 0.2, 0.0, 0.2]), (CYL_X, PLANE, [0.0, 0.5, 0.5, 1.0])]
    for s1, s2, pnt in singular:
        for tol in (1e-7, 1e-3):
            requests.append(_line('pw_min', [2, *s1, *s2, tt, eps, defl, incr, *pnt, tol]))
    for s1, s2, start in PAIRS[:18]:
        for first in (0, 1):
            uv = [start[0] + 0.05, start[1] - 0.05, start[2], start[3] + 0.02]
            requests.append(_line('pw_min', [4, *s1, *s2, tt, eps, defl, incr, *start, *uv, first]))
        for iso, flag in ((0, 0), (1, 1), (2, 0), (3, 1)):
            requests.append(_line('pw_min', [5, *s1, *s2, tt, eps, defl, incr, *start, iso, flag]))


def _quadric(kind: int, loc: list[float], d: list[float], xd: list[float], r1: float, r2: float) -> list[float]:
    return [kind, *loc, *d, *xd, r1, r2]


def _path(u: float, v: float, passing: int = 0, tangent: int = 0, mult: list[tuple[float, float]] = ()) -> list[float]:
    return [u, v, passing, tangent, len(mult), *[c for uv in mult for c in uv]]


def _iw(surface, quadric, walk, flags, paths, loops) -> str:
    tol, eps, defl, step = walk
    fill, rev, mode = flags
    values = [*surface, *quadric, tol, eps, defl, step, fill, rev, mode, len(paths)]
    for p in paths:
        values += p
    values.append(len(loops))
    for u, v in loops:
        values += [u, v]
    return _line('iw_walk', values)


def _iw_cases() -> list[tuple[list[float], list[float], list[list[float]], list[tuple[float, float]]]]:
    square = _analytic(0, [0.0, 0.0, 0.0], Z, 0.0, 0.0, [-1.0, 1.0, -1.0, 1.0])
    big = _quadric(2, [0.0, 0.0, 0.3], Z, X, 1.2, 0.0)
    r = math.sqrt(1.44 - 0.09)
    c = math.sqrt(r * r - 1.0)
    corners = [_path(1.0, c), _path(1.0, -c), _path(-1.0, c), _path(-1.0, -c),
               _path(c, 1.0), _path(-c, 1.0), _path(c, -1.0), _path(-c, -1.0)]
    small = _quadric(2, [0.0, 0.0, 0.3], Z, X, 0.8, 0.0)
    rs = math.sqrt(0.64 - 0.09)
    cyl = _analytic(1, [0.0, 0.0, 0.0], Z, 1.0, 0.0, [0.0, TWO_PI, -2.0, 2.0])
    zplane = _quadric(0, [0.0, 0.0, 0.5], Z, X, 0.0, 0.0)
    tilted = _quadric(0, [0.0, 0.0, 0.2], [0.3, 0.1, 1.0], X, 0.0, 0.0)
    cap = _analytic(2, [0.0, 0.0, 0.0], Z, 1.5, 0.0, [0.0, TWO_PI, -HALF_PI, HALF_PI])
    xcyl = _quadric(1, [0.0, 0.0, 0.0], X, Y, 0.5, 0.0)
    cone = _quadric(3, [0.0, 0.0, -1.0], Z, X, 0.3, 0.5)
    torus = _analytic(3, [0.0, 0.0, 0.0], Z, 3.0, 1.0, [0.0, TWO_PI, 0.0, TWO_PI])
    # Two lines crossing at the apex of a cone of radius 0 (a singular point of the function).
    apex = _quadric(3, [0.0, 0.0, 0.0], X, Y, 0.0, 0.4)
    ta = math.tan(0.4)
    # A plane through the pole of the sphere: the line reaches the degenerated border.
    pole_plane = _quadric(0, [0.0, 0.0, 1.5], [1.0, 0.0, 0.5], Y, 0.0, 0.0)
    half_cyl = _analytic(1, [0.0, 0.0, 0.0], Z, 1.0, 0.0, [0.0, math.pi, -2.0, 2.0])
    vz = math.asin(0.5 / 1.5)
    return [
        (square, big, corners, []),
        (square, big, corners[:3], []),
        (square, small, [], [(rs, 0.0)]),
        (square, small, [], [(rs, 0.0), (-rs, 0.0), (0.0, rs)]),
        (square, big, corners[:4], [(0.0, 0.3)]),
        (square, xcyl, [_path(-1.0, 0.5), _path(1.0, 0.5), _path(-1.0, -0.5), _path(1.0, -0.5)], []),
        # A passing point that is not tangent (state 2, loaded as 11). A tangent passing
        # point (state 1) crossed by an open line loops forever natively (see the doc).
        (square, xcyl, [_path(-1.0, 0.5), _path(1.0, 0.5), _path(-1.0, -0.5), _path(0.3, -0.5, 1, 0),
                        _path(1.0, -0.5)], []),
        (cyl, zplane, [_path(0.0, 0.5, 0, 0, [(TWO_PI, 0.5)]), _path(TWO_PI, 0.5)], []),
        (cyl, zplane, [], [(math.pi, 0.5)]),
        (cyl, tilted, [], [(1.0, 0.2 - 0.3 * math.cos(1.0) - 0.1 * math.sin(1.0))]),
        (cap, zplane, [], [(1.0, vz)]),
        (cap, zplane, [_path(0.0, vz), _path(TWO_PI, vz)], [(2.0, vz)]),
        (square, cone, [], [(0.3 + math.tan(0.5), 0.0)]),
        (torus, zplane, [], [(0.3, math.pi / 6.0), (2.0, 5.0 * math.pi / 6.0)]),
        (square, _quadric(2, [0.0, 0.0, 1.2], Z, X, 1.2, 0.0), [], [(0.0, 0.0)]),
        (square, big, corners, [(r / math.sqrt(2.0), r / math.sqrt(2.0))]),
        (square, apex, [_path(1.0, ta), _path(1.0, -ta), _path(-1.0, ta), _path(-1.0, -ta)], []),
        (cap, pole_plane, [], [(math.atan2(math.sqrt(2.25 - 0.5625), 0.75), 0.0)]),
        (half_cyl, zplane, [], [(1.0, 0.5)]),
        (half_cyl, tilted, [], [(1.5, 0.2 - 0.1 * math.sin(1.5) - 0.3 * math.cos(1.5))]),
    ]


def _iw_requests(requests: list[str]) -> None:
    # The last walk has a function tolerance under the roots the solver reaches: the
    # steps fail and are divided until the lines stop on lonely ends.
    walks = [(1e-7, 1e-6, 0.01, 0.1), (1e-6, 1e-5, 0.05, 0.2), (1e-8, 1e-7, 0.002, 0.05), (1e-20, 1e-6, 0.01, 0.1)]
    for surface, quadric, paths, loops in _iw_cases():
        for walk in walks:
            requests.append(_iw(surface, quadric, walk, (0, 0, 0), paths, loops))
        requests.append(_iw(surface, quadric, walks[0], (1, 0, 0), paths, loops))
        requests.append(_iw(surface, quadric, walks[3], (1, 0, 0), paths, loops))
        requests.append(_iw(surface, quadric, walks[0], (0, 1, 0), paths, loops))
        requests.append(_iw(surface, quadric, walks[0], (0, 0, 1), paths, loops))
    requests.append(_iw(_iw_cases()[0][0], _iw_cases()[0][1], walks[0], (0, 0, 2), [], []))


def append_intwalk_requests(requests: list[str]) -> None:
    """Append the IntWalk requests: PWalking, its protected searches, then IWalking."""
    _pw_requests(requests)
    _pw_min_requests(requests)
    _iw_requests(requests)
