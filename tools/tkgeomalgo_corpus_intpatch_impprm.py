"""Deterministic requests for the IntPatch quadric-parametric intersection
(tools/oracle/tkgeomalgo_probe_intpatch_impprm.cxx).

`ip_impprm` intersects plane, cylinder, cone and sphere patches with Bezier, B-spline,
offset, revolution, extrusion and trimmed patches (both orders), with restrictions that
give restriction lines and vertices, with a start point, and with two tolerance sets.
`ip_rstint` puts the vertices of the restrictions of a patch on given walking and
restriction lines (IntPatch_RstInt).
"""
from __future__ import annotations
import math
from tkgeomalgo_corpus_intcurvesurface import (O, TWO_PI, X, Y, Z, bez_s, bezier, bsp_s, circle, cone, cyl, extr,
                                               hyperbola, line, line_of, offs, parabola, plane, revol, rtrim, sad,
                                               sphere, torus)

PI = math.pi

# Quadric patches.
Q_PLANE = sad(plane([0.0, 0.0, 0.1], Z), (-3.0, 3.0, -3.0, 3.0))
Q_PLANE_TILT = sad(plane([0.0, 0.0, 0.2], [0.2, 0.1, 1.0]), (-4.0, 4.0, -4.0, 4.0))
Q_CYL = sad(cyl(O, Z, 1.0), (0.0, TWO_PI, -2.0, 2.0))
Q_CYL_X = sad(cyl([0.0, 0.0, 0.1], X, 0.7), (0.0, TWO_PI, -3.0, 3.0))
Q_CYL_HALF = sad(cyl(O, Z, 1.0), (0.5, 3.5, -1.0, 1.5))
Q_CONE = sad(cone([0.0, 0.0, -1.0], Z, 0.5, 1.0), (0.0, TWO_PI, -1.0, 3.0))
Q_CONE_APEX = sad(cone([0.0, 0.0, 0.0], Z, 0.6, 0.0), (0.0, TWO_PI, -2.0, 2.0))
Q_SPHERE = sad(sphere(O, Z, 1.2))
Q_SPHERE_PART = sad(sphere([0.2, 0.0, 0.0], Z, 1.0), (0.5, 4.0, -0.8, 1.2))
Q_TORUS = sad(torus(O, Z, 2.0, 0.5))

# Parametric patches.
GRID = [[[-2.0 + 4.0 * i / 3.0, -2.0 + 4.0 * j / 3.0, 0.5 * math.sin(i + 2 * j)] for j in range(4)] for i in range(4)]
FLAT = [[[-2.0 + 4.0 * i / 2.0, -2.0 + 4.0 * j / 2.0, 0.3 * (i - 1) * (j - 1)] for j in range(3)] for i in range(3)]
P_BEZ = sad(bez_s(GRID))
P_BEZ_FLAT = sad(bez_s(FLAT))
P_BEZ_PART = sad(bez_s(GRID), (0.1, 0.8, 0.2, 0.9))
P_BSP = sad(bsp_s(2, 2, GRID, [0.0, 1.0, 2.0], [3, 1, 3], [0.0, 0.5, 1.0], [3, 1, 3]))
P_BSP_C0 = sad(bsp_s(1, 1, FLAT, [0.0, 1.0, 2.0], [2, 1, 2], [0.0, 1.0, 2.0], [2, 1, 2]))
P_OFF_BEZ = sad(offs(0.2, bez_s(FLAT)))
P_OFF_CYL = sad(offs(0.3, cyl(O, X, 0.8)), (0.0, TWO_PI, -2.0, 2.0))
P_REV_LINE = sad(revol([0.3, 0.0, 0.0], X, line([0.0, 0.8, 0.0], [0.4, 0.0, 1.0])), (0.0, TWO_PI, -2.0, 2.0))
P_REV_BEZ = sad(revol(O, X, bezier([[-2.0, 0.5, 0.0], [0.0, 1.5, 0.0], [2.0, 0.6, 0.0]])))
P_REV_CIRC = sad(revol(O, Z, circle([2.0, 0.0, 0.0], Y, 0.6)))
P_EXT_BEZ = sad(extr(Z, bezier([[-2.0, -0.5, 0.0], [0.0, 2.5, 0.0], [2.0, -0.8, 0.0]])), (0.0, 1.0, -2.0, 2.0))
P_EXT_PARAB = sad(extr([0.1, 0.0, 1.0], parabola([0.0, -0.5, 0.0], Z, 0.6)), (-2.0, 2.0, -1.5, 1.5))
P_EXT_HYP = sad(extr(Y, hyperbola([0.0, 0.0, 0.0], Y, 0.7, 0.4)), (-1.5, 1.5, -2.0, 2.0))
P_TRIM_BEZ = sad(rtrim(0.2, 0.9, 0.1, 0.8, bez_s(GRID)))
P_BEZ_TANG = sad(bez_s([[[-2.0, -2.0, 0.1], [-2.0, 2.0, 0.1]], [[2.0, -2.0, 0.1], [2.0, 2.0, 0.1]]]))
P_BEZ_DOME = sad(bez_s([[[-1.5, -1.5, 0.0], [-1.5, 0.0, 0.0], [-1.5, 1.5, 0.0]],
                        [[0.0, -1.5, 0.0], [0.0, 0.0, 2.4], [0.0, 1.5, 0.0]],
                        [[1.5, -1.5, 0.0], [1.5, 0.0, 0.0], [1.5, 1.5, 0.0]]]))
P_BEZ_FAR = sad(bez_s([[[5.0, 5.0, 5.0], [5.0, 6.0, 5.0]], [[6.0, 5.0, 5.0], [6.0, 6.0, 5.5]]]))

# Planar patches through the pole of the sphere, the apex of the cone and the seam of the
# cylinder; a revolution patch whose edge lies on the cylinder; a patch with an edge on the
# plane.
P_PLANE_X0 = sad(bez_s([[[0.0, -2.0, -2.0], [0.0, -2.0, 2.0]], [[0.0, 2.0, -2.0], [0.0, 2.0, 2.0]]]))
P_PLANE_Y0 = sad(bez_s([[[-2.0, 0.0, -1.5], [-2.0, 0.0, 1.5]], [[2.0, 0.0, -1.5], [2.0, 0.0, 1.5]]]))
P_PLANE_SLANT = sad(bez_s([[[-2.0, -2.0, -1.0], [-2.0, 2.0, 1.2]], [[2.0, -2.0, -0.8], [2.0, 2.0, 1.0]]]))
P_REV_EDGE = sad(revol(O, Z, bezier([[1.0, 0.0, 0.3], [1.5, 0.0, 0.8], [2.0, 0.0, 1.0]])))
P_REV_EDGE_PART = sad(revol(O, Z, bezier([[1.0, 0.0, 0.3], [1.5, 0.0, 0.8], [2.0, 0.0, 1.0]])), (1.0, 2.0, 0.0, 1.0))
P_REV_EDGE_OVER = sad(revol(O, Z, bezier([[1.0, 0.0, 0.3], [1.5, 0.0, 0.8], [2.0, 0.0, 1.0]])), (-1.0, 2.5, 0.0, 1.0))
P_EDGE_ON_PLANE = sad(bez_s([[[-2.0, -2.0, 0.1], [-2.0, 0.0, 0.1], [-2.0, 2.0, 0.1]],
                             [[0.0, -2.0, 0.6], [0.0, 0.0, -0.4], [0.0, 2.0, 0.7]],
                             [[2.0, -2.0, 0.9], [2.0, 0.0, 0.2], [2.0, 2.0, 1.1]]]))
P_WAVE = sad(bez_s([[[-2.0 + i, -2.0 + j, 0.9 * math.cos(1.3 * i) * math.sin(0.9 * j + 0.4)] for j in range(5)]
                    for i in range(5)]))
TARGETED = [
    (Q_SPHERE, P_PLANE_X0), (P_PLANE_X0, Q_SPHERE), (Q_SPHERE_PART, P_PLANE_X0), (Q_CONE_APEX, P_PLANE_X0),
    (P_PLANE_Y0, Q_CONE_APEX), (Q_CONE, P_PLANE_X0), (Q_CYL, P_PLANE_Y0), (P_PLANE_Y0, Q_CYL), (Q_CYL, P_PLANE_X0),
    (Q_CYL_HALF, P_PLANE_SLANT), (Q_SPHERE_PART, P_PLANE_SLANT), (Q_CYL, P_REV_EDGE), (P_REV_EDGE, Q_CYL),
    (Q_CYL_HALF, P_REV_EDGE_PART), (Q_CYL, P_REV_EDGE_OVER), (Q_CYL_HALF, P_REV_EDGE), (Q_PLANE, P_EDGE_ON_PLANE),
    (P_EDGE_ON_PLANE, Q_PLANE), (Q_PLANE, P_BEZ_TANG), (Q_CYL_HALF, P_WAVE), (Q_SPHERE_PART, P_WAVE),
    (Q_CONE, P_WAVE), (Q_SPHERE, P_WAVE), (Q_CYL_X, P_WAVE),
    (sad(sphere([0.0, 0.0, 0.0], Z, 1.0), (0.0, TWO_PI, -1.2, 0.9)), P_PLANE_X0),
    (sad(cone([0.0, 0.0, 0.0], Z, 0.6, 0.0), (0.4, 5.0, -2.0, 2.0)), P_PLANE_SLANT),
    (sad(sphere(O, Z, 1.0), (0.3, 5.5, -1.0, 1.0)), P_PLANE_Y0),
]
FINE = (1.0e-7, 1.0e-6, 1.0e-3, 0.02)

QUADRICS = [Q_PLANE, Q_PLANE_TILT, Q_CYL, Q_CYL_X, Q_CYL_HALF, Q_CONE, Q_CONE_APEX, Q_SPHERE, Q_SPHERE_PART]
PARAMETRICS = [P_BEZ, P_BEZ_FLAT, P_BEZ_PART, P_BSP, P_BSP_C0, P_OFF_BEZ, P_OFF_CYL, P_REV_LINE, P_REV_BEZ,
               P_REV_CIRC, P_EXT_BEZ, P_EXT_PARAB, P_EXT_HYP, P_TRIM_BEZ, P_BEZ_DOME]
TOLS = [(1.0e-7, 1.0e-6, 0.01, 0.1), (1.0e-4, 1.0e-4, 0.05, 0.2)]


def _impprm_requests(requests: list[str]) -> None:
    for qi, q in enumerate(QUADRICS):
        for pi, p in enumerate(PARAMETRICS):
            # Both orders, alternating, so each pair runs once in the main tolerances.
            pair = [*q, *p] if (qi + pi) % 2 == 0 else [*p, *q]
            requests.append(line_of('ip_impprm', [*pair, *TOLS[0], 0, 0.0, 0.0]))
    for q in QUADRICS[::2]:
        for p in PARAMETRICS[::3]:
            requests.append(line_of('ip_impprm', [*p, *q, *TOLS[1], 1, 0.0, 0.0]))
    # A start point on the parametric patch, a second Perform, a tangent and a far patch.
    for q, p, mode, u, v in [(Q_SPHERE, P_BEZ, 2, 0.5, 0.5), (Q_CYL, P_BEZ_DOME, 2, 0.3, 0.6),
                             (Q_PLANE, P_BSP, 2, 1.0, 0.5), (Q_CONE, P_EXT_BEZ, 3, 0.0, 0.0),
                             (Q_PLANE, P_BEZ_TANG, 1, 0.0, 0.0), (Q_SPHERE, P_BEZ_FAR, 1, 0.0, 0.0),
                             (Q_CYL, P_BEZ_FAR, 0, 0.0, 0.0)]:
        requests.append(line_of('ip_impprm', [*q, *p, *TOLS[0], mode, u, v]))
    for q, p in TARGETED:
        requests.append(line_of('ip_impprm', [*q, *p, *TOLS[0], 0, 0.0, 0.0]))
    for q, p in TARGETED[::2]:
        requests.append(line_of('ip_impprm', [*q, *p, *FINE, 1, 0.0, 0.0]))
    # Two parametric patches, and a torus with a parametric one: no quadric.
    requests.append(line_of('ip_impprm', [*P_BEZ, *P_BSP, *TOLS[0], 0, 0.0, 0.0]))
    requests.append(line_of('ip_impprm', [*Q_TORUS, *P_BEZ, *TOLS[0], 1, 0.0, 0.0]))


def _circle_line(q_center_z: float, radius: float, n: int, u0: float, u1: float, cyl_first: bool):
    """Points of the circle z = `q_center_z` on the cylinder of axis Z and radius
    `radius`, with the cylinder parameters (u, z) and the plane ones (x, y)."""
    pts = []
    for k in range(n):
        u = u0 + (u1 - u0) * k / (n - 1)
        x, y = radius * math.cos(u), radius * math.sin(u)
        cyl_uv, plane_uv = [u, q_center_z], [x, y]
        uv = [*cyl_uv, *plane_uv] if cyl_first else [*plane_uv, *cyl_uv]
        pts.append([x, y, q_center_z, *uv])
    return pts


def _plane_segment(y: float, n: int, x0: float, x1: float, z: float):
    """Points of the segment y = `y`, z = `z` from x0 to x1 on the plane z (x, y) and on
    another plane z (x, y)."""
    pts = []
    for k in range(n):
        x = x0 + (x1 - x0) * k / (n - 1)
        pts.append([x, y, z, x, y, x, y])
    return pts


RST_CYL = sad(cyl(O, Z, 1.0), (0.3, 5.0, -1.0, 1.0))
RST_PLANE = sad(plane([0.0, 0.0, 0.3], Z), (-0.8, 0.9, -0.7, 0.6))
RST_PLANE_BIG = sad(plane([0.0, 0.0, 0.3], Z), (-3.0, 3.0, -3.0, 3.0))


def _rstint_requests(requests: list[str]) -> None:
    circle_cyl = _circle_line(0.3, 1.0, 40, 0.0, TWO_PI, True)
    circle_pln = _circle_line(0.3, 1.0, 40, 0.0, TWO_PI, False)
    arc_cyl = _circle_line(0.3, 1.0, 25, 0.1, 5.5, True)
    seg = _plane_segment(0.1, 30, -1.5, 1.6, 0.3)
    cases = [
        # The circle on the cylinder, restricted in U: vertices at the U bounds.
        (RST_CYL, RST_PLANE_BIG, 1, 0, 0, 0, 0, circle_cyl),
        (RST_CYL, RST_PLANE_BIG, 1, 0, 0, 1, 1, circle_cyl),
        (RST_CYL, RST_PLANE_BIG, 1, 0, 0, 0, 1, arc_cyl),
        # The plane patch restricted: the circle crosses its four edges.
        (RST_PLANE, RST_CYL, 0, 0, 0, 0, 0, circle_cyl),
        (RST_PLANE, RST_CYL, 0, 0, 0, 1, 1, circle_cyl),
        (RST_PLANE, sad(cyl(O, Z, 1.0)), 1, 0, 0, 0, 0, circle_pln),
        # A segment through the plane patch.
        (RST_PLANE, RST_PLANE_BIG, 1, 0, 0, 0, 0, seg),
        (RST_PLANE, RST_PLANE_BIG, 1, 0, 0, 1, 1, seg),
        # Restriction lines on an arc of the other patch.
        (RST_PLANE, RST_PLANE_BIG, 1, 1, 1, 0, 0, seg),
        (RST_PLANE, RST_PLANE_BIG, 1, 1, 3, 1, 1, seg),
        (RST_PLANE, RST_PLANE_BIG, 0, 1, 2, 0, 0, seg),
        (RST_PLANE, RST_PLANE_BIG, 0, 1, 0, 1, 0, seg),
    ]
    for tol in (1.0e-6, 1.0e-3):
        for s, o, onfirst, kind, arc, ends, twice, pts in cases:
            values = [*s, *o, onfirst, tol, kind, arc, ends, twice, len(pts), *[c for p in pts for c in p]]
            requests.append(line_of('ip_rstint', values))


def append_intpatch_impprm_requests(requests: list[str]) -> None:
    """Appends the IntPatch_ImpPrmIntersection and IntPatch_RstInt requests."""
    _impprm_requests(requests)
    _rstint_requests(requests)
