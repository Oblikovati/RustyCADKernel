"""Deterministic requests for the IntPatch analytic intersections of two quadrics
(tools/oracle/tkgeomalgo_probe_intpatch_impimp.cxx).

`ip_impimp` intersects every pair of plane, cylinder, cone, sphere and torus patches in
coaxial, secant, tangent, coincident, disjoint and near-degenerate positions, on bounded
domains (whose restrictions give vertices and restriction lines) and natural ones.
`ip_aline2wline` converts the analytic lines of the pairs that have some into walking
lines, with the default and changed tolerances, over the whole line and a range.
`ip_special` and `ip_special_pole` run IntPatch_SpecialPoints on poles, apexes and seams.
"""
from __future__ import annotations
import math
from tkgeomalgo_corpus_intcurvesurface import O, TWO_PI, X, Y, Z, bez_s, cone, cyl, line_of, plane, sad, sphere, torus

PI = math.pi
HALF_PI = PI / 2.0


def nd(v):
    """The unit vector of `v`."""
    n = math.sqrt(sum(c * c for c in v))
    return [c / n for c in v]


# Patches: (name, surface) with domains that cut the intersections.
P_Z = sad(plane(O, Z), (-3.0, 3.0, -3.0, 3.0))
P_Z_NAT = sad(plane(O, Z))
P_Z1 = sad(plane([0.0, 0.0, 1.0], Z), (-4.0, 4.0, -4.0, 4.0))
P_X = sad(plane([0.2, 0.0, 0.0], X), (-3.0, 3.0, -3.0, 3.0))
P_TILT = sad(plane([0.0, 0.0, 0.3], nd([0.3, 0.1, 1.0])), (-4.0, 4.0, -4.0, 4.0))
P_STEEP = sad(plane([0.0, 0.0, 0.2], nd([1.0, 0.0, 0.8])), (-5.0, 5.0, -5.0, 5.0))
P_TANG_CYL = sad(plane([1.0, 0.0, 0.0], X), (-3.0, 3.0, -3.0, 3.0))
P_FAR = sad(plane([0.0, 0.0, 9.0], Z), (-2.0, 2.0, -2.0, 2.0))
CY_Z = sad(cyl(O, Z, 1.0), (0.0, TWO_PI, -2.0, 2.0))
CY_Z_NAT = sad(cyl(O, Z, 1.0))
CY_Z_HALF = sad(cyl(O, Z, 1.0), (0.3, 3.5, -1.5, 1.8))
CY_X = sad(cyl([0.0, 0.0, 0.0], X, 0.6), (0.0, TWO_PI, -3.0, 3.0))
CY_X_BIG = sad(cyl([0.0, 0.0, 0.0], X, 1.0), (0.0, TWO_PI, -3.0, 3.0))
CY_X_OFF = sad(cyl([0.0, 0.3, 0.1], X, 0.5), (0.0, TWO_PI, -3.0, 3.0))
CY_SKEW = sad(cyl([0.1, 0.0, 0.2], nd([1.0, 0.4, 0.3]), 0.7), (0.0, TWO_PI, -3.0, 3.0))
CY_THIN = sad(cyl([0.0, 0.0, 0.0], Y, 0.2), (0.0, TWO_PI, -3.0, 3.0))
CY_Z2 = sad(cyl([0.5, 0.0, 0.0], Z, 0.8), (0.0, TWO_PI, -1.0, 1.0))
CY_Z_TANG = sad(cyl([2.0, 0.0, 0.0], Z, 1.0), (0.0, TWO_PI, -1.0, 1.0))
CY_Z_SAME = sad(cyl(O, Z, 1.0), (1.0, 4.0, -1.0, 1.0))
CY_Z_FAR = sad(cyl([5.0, 0.0, 0.0], Z, 1.0), (0.0, TWO_PI, -1.0, 1.0))
CO_Z = sad(cone([0.0, 0.0, -1.0], Z, 0.5, 1.0), (0.0, TWO_PI, -1.0, 3.0))
CO_Z_NAT = sad(cone([0.0, 0.0, -1.0], Z, 0.5, 1.0))
CO_X = sad(cone([-1.0, 0.0, 0.0], X, 0.4, 0.5), (0.0, TWO_PI, -2.0, 3.0))
CO_APEX = sad(cone([0.0, 0.0, 0.0], Z, 0.6, 0.0), (0.0, TWO_PI, -2.0, 2.0))
CO_SKEW = sad(cone([0.2, 0.1, -0.5], nd([0.3, 1.0, 0.2]), 0.35, 0.6), (0.0, TWO_PI, -2.0, 2.0))
CO_Z2 = sad(cone([0.0, 0.0, 0.5], Z, 0.3, 0.4), (0.0, TWO_PI, -1.0, 2.0))
SP = sad(sphere(O, Z, 1.5))
SP_OFF = sad(sphere([0.8, 0.2, 0.3], Z, 1.0))
SP_PART = sad(sphere([0.0, 0.0, 0.0], Z, 1.0), (0.5, 4.0, -0.5, 1.2))
SP_TANG = sad(sphere([0.0, 0.0, 2.5], Z, 1.0))
SP_SAME = sad(sphere(O, Z, 1.5), (1.0, 3.0, -0.5, 0.5))
SP_SMALL = sad(sphere([0.0, 0.0, 0.7], X, 0.5))
SP_POLE = sad(sphere([0.0, 0.0, 0.0], Z, 1.0))
TO = sad(torus(O, Z, 3.0, 1.0))
TO_X = sad(torus([0.0, 0.0, 0.0], X, 2.0, 0.5))
TO_SMALL = sad(torus([0.0, 0.0, 0.0], Z, 1.5, 0.4), (0.0, TWO_PI, 0.0, TWO_PI))

PAIRS = [
    # plane / plane
    (P_Z, P_X), (P_Z, P_Z1), (P_Z, sad(plane(O, Z), (-1.0, 1.0, -1.0, 1.0))), (P_Z, P_TILT),
    (P_Z, sad(plane(O, [0.0, 0.0, -1.0]), (-2.0, 2.0, -2.0, 2.0))), (P_Z_NAT, P_X),
    # plane / cylinder
    (P_Z, CY_Z), (CY_Z, P_Z), (P_TILT, CY_Z), (P_X, CY_Z), (P_TANG_CYL, CY_Z), (P_STEEP, CY_Z),
    (P_Z, CY_X), (P_Z1, CY_Z_NAT), (P_FAR, CY_Z), (P_Z, CY_Z_HALF), (P_TILT, CY_SKEW),
    # plane / cone
    (P_Z, CO_Z), (CO_Z, P_Z), (P_TILT, CO_Z), (P_X, CO_Z), (P_STEEP, CO_Z), (P_X, CO_APEX),
    (sad(plane([0.0, 0.0, 0.0], Z), (-3.0, 3.0, -3.0, 3.0)), CO_APEX),
    (sad(plane(O, nd([1.0, 0.0, math.tan(0.6)])), (-3.0, 3.0, -3.0, 3.0)), CO_APEX),
    (sad(plane([0.0, 0.0, 1.0], nd([1.0, 0.0, math.tan(0.6)])), (-3.0, 3.0, -3.0, 3.0)), CO_APEX),
    (sad(plane([0.0, 0.0, 0.5], nd([1.0, 0.0, 0.4])), (-4.0, 4.0, -4.0, 4.0)), CO_Z_NAT),
    (P_TILT, CO_SKEW),
    # plane / sphere
    (P_Z, SP), (SP, P_Z), (P_TILT, SP), (P_X, SP_OFF), (sad(plane([0.0, 0.0, 1.5], Z), (-2.0, 2.0, -2.0, 2.0)), SP),
    (P_FAR, SP), (P_Z, SP_PART), (P_X, SP_SMALL),
    # plane / torus
    (P_Z, TO), (TO, P_Z), (P_X, TO), (P_Z1, TO), (sad(plane([0.0, 0.0, 1.0], X), (-5.0, 5.0, -5.0, 5.0)), TO),
    (P_TILT, TO), (P_Z, TO_X), (P_FAR, TO_SMALL),
    # cylinder / cylinder
    (CY_Z, CY_X), (CY_X, CY_Z), (CY_Z, CY_X_BIG), (CY_Z, CY_X_OFF), (CY_Z, CY_SKEW), (CY_Z, CY_THIN),
    (CY_Z, CY_Z2), (CY_Z, CY_Z_TANG), (CY_Z, CY_Z_SAME), (CY_Z, CY_Z_FAR), (CY_Z_HALF, CY_X),
    (CY_Z_NAT, CY_X), (CY_SKEW, CY_X_OFF),
    (sad(cyl(O, Z, 3.0), (0.0, TWO_PI, -2.0, 2.0)), sad(cyl([0.0, 0.0, 0.0], X, 0.5), (0.0, TWO_PI, -4.0, 4.0))),
    (sad(cyl(O, Z, 1.0), (0.0, TWO_PI, -2.0, 2.0)), sad(cyl([0.0, 0.0, 0.0], nd([1.0, 0.0, 1.0]), 1.0), (0.0, TWO_PI, -3.0, 3.0))),
    # cylinder / cone
    (CY_Z, CO_Z), (CO_Z, CY_Z), (CY_X, CO_Z), (CY_Z, CO_SKEW), (CY_Z2, CO_Z2),
    (sad(cyl(O, Z, 0.5), (0.0, TWO_PI, -1.0, 3.0)), CO_Z),
    (sad(cyl([0.0, 0.0, 0.0], Z, 1.5), (0.0, TWO_PI, 0.0, 4.0)), CO_Z),
    # cylinder / sphere
    (CY_Z, SP), (SP, CY_Z), (CY_Z, SP_OFF), (CY_X, SP), (CY_Z, sad(sphere(O, Z, 1.0))),
    (CY_Z, sad(sphere([1.5, 0.0, 0.0], Z, 0.5))), (CY_Z_TANG, SP_OFF), (CY_Z, sad(sphere([1.0, 0.0, 0.0], Z, 1.0))),
    # cylinder / torus
    (CY_Z, TO), (TO, CY_Z), (sad(cyl(O, Z, 3.0), (0.0, TWO_PI, -2.0, 2.0)), TO), (CY_X, TO_SMALL),
    # cone / cone
    (CO_Z, CO_X), (CO_Z, CO_Z2), (CO_Z, CO_Z), (CO_Z, sad(cone([0.0, 0.0, -1.0], Z, 0.5, 1.0), (1.0, 3.0, 0.0, 1.0))),
    (CO_Z, CO_SKEW), (CO_APEX, sad(cone([0.0, 0.0, 0.0], [0.0, 0.0, -1.0], 0.6, 0.0), (0.0, TWO_PI, -2.0, 2.0))),
    (CO_APEX, sad(cone([0.0, 0.0, 0.0], nd([1.0, 0.0, 0.3]), 0.4, 0.0), (0.0, TWO_PI, -2.0, 2.0))),
    (CO_APEX, sad(cone([0.0, 0.0, 1.0], Z, 0.6, 0.0), (0.0, TWO_PI, -2.0, 2.0))),
    # cone / sphere
    (CO_Z, SP), (SP, CO_Z), (CO_APEX, SP_POLE), (CO_Z, SP_OFF), (CO_X, SP_SMALL),
    (CO_APEX, sad(sphere([0.0, 0.0, 1.5], Z, 0.5))),
    # cone / torus
    (CO_Z, TO), (TO, CO_Z), (CO_APEX, TO_SMALL),
    # sphere / sphere
    (SP, SP_OFF), (SP, SP_TANG), (SP, SP_SAME), (SP, sad(sphere(O, Z, 1.5))), (SP, sad(sphere([5.0, 0.0, 0.0], Z, 1.0))),
    (SP_PART, SP_OFF), (SP, sad(sphere(O, Z, 0.5))),
    # sphere / torus
    (SP, TO), (TO, SP), (sad(sphere([3.0, 0.0, 0.0], Z, 1.0)), TO), (SP_SMALL, TO_SMALL),
    # torus / torus
    (TO, sad(torus([0.0, 0.0, 0.0], Z, 3.0, 1.0))), (TO, sad(torus([0.0, 0.0, 1.0], Z, 3.0, 1.0))),
    (TO, TO_X), (TO_SMALL, sad(torus([0.0, 0.0, 0.2], Z, 1.5, 0.4))),
]

TOLS = [(1.0e-7, 1.0e-6), (1.0e-3, 1.0e-4)]

# Near-degenerate positions: almost parallel axes, almost tangent or coincident surfaces.
EPS_ANG = 1.0e-9
NEAR = [
    (CY_Z, sad(cyl([0.3, 0.0, 0.0], nd([EPS_ANG, 0.0, 1.0]), 1.0), (0.0, TWO_PI, -1.5, 1.5))),
    (CY_Z, sad(cyl(O, Z, 1.0 + 1.0e-10), (0.0, TWO_PI, -1.0, 1.0))),
    (CY_Z, sad(cyl([2.0 + 1.0e-9, 0.0, 0.0], Z, 1.0), (0.0, TWO_PI, -1.0, 1.0))),
    (sad(plane([1.0 - 1.0e-9, 0.0, 0.0], X), (-3.0, 3.0, -3.0, 3.0)), CY_Z),
    (sad(plane([0.0, 0.0, 0.0], nd([EPS_ANG, 0.0, 1.0])), (-3.0, 3.0, -3.0, 3.0)), sad(plane(O, Z), (-2.0, 2.0, -2.0, 2.0))),
    (SP, sad(sphere([0.0, 0.0, 2.5 + 1.0e-9], Z, 1.0))),
    (SP, sad(sphere([1.0e-11, 0.0, 0.0], Z, 1.5))),
    (CY_Z, sad(sphere([2.0 - 1.0e-9, 0.0, 0.0], Z, 1.0))),
    (sad(plane([0.0, 0.0, 1.5 - 1.0e-10], Z), (-2.0, 2.0, -2.0, 2.0)), SP),
    (sad(plane(O, nd([1.0, 0.0, math.tan(0.6) + 1.0e-9])), (-3.0, 3.0, -3.0, 3.0)), CO_APEX),
    (CO_Z, sad(cone([0.0, 0.0, -1.0], nd([EPS_ANG, 0.0, 1.0]), 0.5, 1.0), (0.0, TWO_PI, -1.0, 3.0))),
    (CY_Z, sad(cone([0.0, 0.0, -1.0], Z, 1.0e-9, 1.0), (0.0, TWO_PI, -1.0, 1.0))),
    (TO, sad(torus([0.0, 0.0, 1.0e-10], Z, 3.0, 1.0))),
    (sad(plane([0.0, 0.0, 1.0 - 1.0e-10], Z), (-5.0, 5.0, -5.0, 5.0)), TO),
]


def _impimp_requests(requests: list[str]) -> None:
    for s1, s2 in PAIRS:
        requests.append(line_of('ip_impimp', [*s1, *s2, *TOLS[0], 0]))
    for s1, s2 in PAIRS[::4]:
        requests.append(line_of('ip_impimp', [*s1, *s2, *TOLS[1], 1]))
    for s1, s2 in NEAR:
        requests.append(line_of('ip_impimp', [*s1, *s2, *TOLS[0], 0]))
    # A surface that is no quadric raises Standard_ConstructionError.
    requests.append(line_of('ip_impimp', [*P_Z, *sad(bez_s([[[0.0, 0.0, 0.0], [0.0, 1.0, 0.0]], [[1.0, 0.0, 0.0], [1.0, 1.0, 1.0]]])), *TOLS[0], 0]))


ALINE_PAIRS = [
    (CY_Z, SP_OFF), (CY_Z, CO_SKEW), (CO_Z, SP_OFF), (CO_Z, CO_SKEW), (CY_Z, sad(sphere([1.0, 0.0, 0.0], Z, 1.0))),
    (CO_APEX, SP_POLE), (CY_Z, CO_Z2), (sad(cyl(O, Z, 1.0)), sad(sphere([0.5, 0.0, 0.0], Z, 1.2))),
    (SP_OFF, CY_Z), (CO_SKEW, CO_Z),
]
SETTINGS = [
    (200, 1.0e-9, 1.0e-8, 1.0e-7, 0, 0.0, 0.0),
    (40, 1.0e-6, 1.0e-6, 1.0e-5, 0, 0.0, 0.0),
    (200, 1.0e-9, 1.0e-8, 1.0e-7, 1, 0.5, 2.5),
    (3, 1.0e-9, 1.0e-8, 1.0e-7, 0, 0.0, 0.0),
]


def _aline_requests(requests: list[str]) -> None:
    for s1, s2 in ALINE_PAIRS:
        for st in SETTINGS:
            requests.append(line_of('ip_aline2wline', [*s1, *s2, *TOLS[0], *st]))


def pnt2s(p, u1, v1, u2, v2):
    return [*p, u1, v1, u2, v2]


PLANE_UNDER_POLE = sad(plane([0.0, 0.0, 1.0], nd([0.2, 0.1, 1.0])), (-2.0, 2.0, -2.0, 2.0))
BEZ_UNDER = sad(bez_s([[[-1.0, -1.0, 1.0], [-1.0, 1.0, 1.0]], [[1.0, -1.0, 1.0], [1.0, 1.0, 1.0]]]))
SEAM_CYL_PLANE = sad(plane([0.0, 0.0, 0.0], nd([0.0, 1.0, 0.4])), (-3.0, 3.0, -3.0, 3.0))


def _special_requests(requests: list[str]) -> None:
    ref = pnt2s([0.0, 0.0, 1.0], 0.1, 1.4, 0.2, 0.3)
    for q, p, rev in [(SP_POLE, PLANE_UNDER_POLE, 0), (SP_POLE, BEZ_UNDER, 0), (CY_Z, SEAM_CYL_PLANE, 0),
                      (TO_SMALL, P_Z, 0), (SP_POLE, PLANE_UNDER_POLE, 1)]:
        for tol in (1.0e-7, 1.0e-2, 2.0):
            requests.append(line_of('ip_special', [0, *q, *p, rev, *ref, tol]))
    # AddPointOnUorVIso on the U seam of the cylinder and the V iso of the sphere.
    for q, p, isu, iso in [(CY_Z, SEAM_CYL_PLANE, 1, 0.0), (SP_POLE, PLANE_UNDER_POLE, 0, 1.2), (CY_Z, P_TILT, 1, 0.0)]:
        for init in ([0.1, 0.1, 0.1], [1.0, -0.5, 0.3], [-2.0, 2.0, 1.5]):
            requests.append(line_of('ip_special', [1, *q, *p, 0, *ref, isu, iso, 1e-9, 1e-9, 1e-9, *init,
                                                   -5.0, -5.0, -5.0, 5.0, 5.0, 5.0]))
    # AddSingularPole on the pole of a sphere and the apex of a cone; a cylinder raises.
    vtx = pnt2s([0.0, 0.0, 1.0], 0.0, 1.5, 0.1, 0.1)
    for q, p in [(SP_POLE, PLANE_UNDER_POLE), (SP_POLE, BEZ_UNDER), (CO_APEX, P_TILT), (CO_APEX, P_X), (CY_Z, P_Z)]:
        for rev in (0, 1):
            for chk in (0, 1):
                requests.append(line_of('ip_special', [2, *q, *p, rev, *ref, *vtx, 0.1, chk]))
    # ContinueAfterSpecialPoint for every special point type.
    new = pnt2s([0.0, 0.0, 1.0], 3.0, 1.5, 0.2, 0.2)
    for q, p in [(CO_APEX, P_TILT), (SP_POLE, PLANE_UNDER_POLE), (CY_Z, SEAM_CYL_PLANE)]:
        for t in range(6):
            for rev in (0, 1):
                requests.append(line_of('ip_special', [3, *q, *p, rev, *ref, t, 1e-6, *new]))
        requests.append(line_of('ip_special', [3, *q, *p, 0, *ref, 5, 1e-6, *ref]))
    # AdjustPointAndVertex with and without periods and vertex.
    for per in ([0.0, 0.0, 0.0, 0.0], [TWO_PI, 0.0, TWO_PI, TWO_PI], [PI / 2, PI / 2, 1.0, 3.0]):
        for wv in (0, 1):
            requests.append(line_of('ip_special', [4, *SP, *P_Z, 0, *ref, *per, *pnt2s([1.0, 2.0, 3.0], 13.0, -7.5, 0.4, -20.0), wv]))


def _pole_requests(requests: list[str]) -> None:
    iso = pnt2s([0.0, 0.0, 0.0], 0.7, 1.0, 5.5, 2.0)
    vecs = [([1.0, 0.0, 0.3], [0.0, 1.0, 0.2]), ([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]), ([0.2, 0.0, 1.0], [0.0, 0.3, 1.0]),
            ([1.0, 1.0, 1e-9], [1.0, 1.0, 1e-9]), ([0.3, -0.7, 0.0], [0.0, 0.0, 0.5]), ([1.0, 0.0, 0.0], [2.0, 0.0, 0.0])]
    for du, dv in vecs:
        for rev in (0, 1):
            requests.append(line_of('ip_special_pole', [0, *iso, *du, *dv, HALF_PI, rev, 0.0]))
            requests.append(line_of('ip_special_pole', [0, *iso, *du, *dv, -HALF_PI, rev, 0.0]))
            requests.append(line_of('ip_special_pole', [1, *iso, *du, *dv, 0.5, rev, 0.25]))
            requests.append(line_of('ip_special_pole', [1, *iso, *du, *dv, 1.2, rev, 0.25]))
    for n in ([0.0, 0.0, 1.0], nd([1.0, 0.0, 1.0]), nd([0.3, 0.9, 0.1]), [1.0, 0.0, 0.0], nd([math.cos(0.5), 0.0, -math.sin(0.5)]),
              nd([0.0, 1.0, 0.0])):
        for ang in (0.5, 0.2, 1.3):
            requests.append(line_of('ip_special_pole', [2, *iso, *n, 0.0, 0.0, 0.0, ang, 0, 0.0]))


def append_intpatch_impimp_requests(requests: list[str]) -> None:
    """Every request family of the analytic intersections of two quadrics."""
    _impimp_requests(requests)
    _aline_requests(requests)
    _special_requests(requests)
    _pole_requests(requests)
