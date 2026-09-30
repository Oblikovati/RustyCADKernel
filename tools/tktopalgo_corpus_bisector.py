"""Deterministic requests for Bisector (tools/oracle/tktopalgo_probe_bisector.cxx).

The bisectors of every pair of elements: two points, a point and a line, circle, conic or
B-spline, two lines, circles, conics or B-splines, meeting at a corner (the MAT2d use, with
the vectors taken from the curves) or not; the analytic Bisector_BisecAna of each overload;
the algorithmic Bisector_BisecCC and Bisector_BisecPC with their members; intersections of
the bisectors of triangle corners; and the functions, the polygon and IsConvex.
"""
from __future__ import annotations
import math
from tktopalgo_corpus_curves import (Point, arc, arc_point, bspline, circle, ellipse, line, numbers,
                                     parabola, segment, trimmed)

# The curves meeting at the corner (0, 0): the first ends there, the second starts there.
CORNER: Point = (0.0, 0.0)


def into_corner() -> list[tuple[str, list[float]]]:
    """Curves ending at the corner: lines, arcs (convex and concave), conics, B-splines."""
    c1 = (-1.0, 1.0)
    return [
        ('seg_left', segment((-2.0, 0.0), CORNER)),
        ('seg_down', segment((0.0, 2.0), CORNER)),
        ('seg_diag', segment((-1.5, -1.0), CORNER)),
        ('arc_ccw', arc((0.0, 1.0), 1.0, -math.pi, -math.pi / 2)),
        ('arc_cw', arc(c1, math.sqrt(2.0), 0.0, -math.pi / 4)),
        ('ellipse', trimmed(math.pi / 2, math.pi, ellipse((1.0, 0.0), (1.0, 0.0), 1.0, 0.5))),
        ('parabola', trimmed(-1.0, 0.0, parabola(CORNER, (0.0, 1.0), 0.5))),
        ('bspline', bspline([(-2.0, 1.0), (-1.0, 1.5), (-0.5, 0.2), CORNER], 3)),
        ('bspline2', bspline([(-1.0, -1.0), (-0.5, 0.5), CORNER], 2)),
    ]


def out_of_corner() -> list[tuple[str, list[float]]]:
    """Curves starting at the corner."""
    return [
        ('seg_right', segment(CORNER, (2.0, 0.0))),
        ('seg_up', segment(CORNER, (0.0, 1.5))),
        ('seg_diag', segment(CORNER, (1.0, -1.5))),
        ('seg_back', segment(CORNER, (-1.0, 0.0))),
        ('arc_ccw', arc((0.0, -1.0), 1.0, math.pi / 2, math.pi)),
        ('arc_cw', arc((1.0, 1.0), math.sqrt(2.0), -3 * math.pi / 4, -math.pi)),
        ('ellipse', trimmed(0.0, math.pi / 2, ellipse((-1.0, 0.0), (1.0, 0.0), 1.0, 0.7))),
        ('bspline', bspline([CORNER, (0.5, 0.8), (1.5, 0.5), (2.0, 1.0)], 3)),
        ('bspline2', bspline([CORNER, (1.0, -0.2), (1.5, -1.0)], 2)),
        ('short_linear', bspline([CORNER, (5e-5, 2e-5)], 1)),
    ]


def curves_requests() -> list[str]:
    """Bisector_Bisec of every pair of corner curves, both sides, on and off the curve."""
    requests: list[str] = []
    for i, (_, c1) in enumerate(into_corner()):
        for j, (_, c2) in enumerate(out_of_corner()):
            sense = 1.0 if (i + j) % 2 == 0 else -1.0
            join = (i + 2 * j) % 3
            requests.append('tp_bisec_curves ' + numbers([1] + c1 + c2 + [sense, join, 1e-7, 1]))
            requests.append('tp_bisec_curves ' + numbers([1] + c1 + c2 + [-sense, join, 1e-7, 1]))
    return requests


def curves_off_requests() -> list[str]:
    """Bisector_Bisec of curves that do not meet (a connexion of MAT2d): explicit point of
    passage and vectors, off the curve."""
    requests: list[str] = []
    pairs = [
        (segment((0.0, 0.0), (2.0, 0.0)), segment((2.0, 1.0), (0.0, 1.0)), (1.0, 0.5), (1.0, 0.0), (1.0, 0.0)),
        (segment((0.0, 0.0), (2.0, 0.0)), arc((1.0, 2.0), 1.0, -math.pi / 4, -3 * math.pi / 4), (1.0, 0.5), (0.0, 1.0), (0.0, -1.0)),
        (arc((0.0, 0.0), 1.0, 0.0, math.pi / 2), arc((0.0, 0.0), 2.0, math.pi / 2, 0.0), (1.5, 0.0), (1.0, 0.0), (-1.0, 0.0)),
        (arc((0.0, 0.0), 1.0, 0.0, math.pi), segment((-2.0, -1.0), (2.0, -1.0)), (0.0, -1.0), (-1.0, 0.0), (1.0, 0.0)),
        (bspline([(0.0, 0.0), (1.0, 0.5), (2.0, 0.0)], 2), segment((2.0, 2.0), (0.0, 2.0)), (1.0, 1.5), (1.0, 0.0), (-1.0, 0.0)),
        (segment((0.0, 0.0), (1.0, 0.0)), segment((1.0, 0.0), (0.0, 0.0)), (1.0, 0.0), (1.0, 0.0), (-1.0, 0.0)),
    ]
    for index, (c1, c2, p, v1, v2) in enumerate(pairs):
        for sense in (1.0, -1.0):
            passage = [p[0], p[1], v1[0], v1[1], v2[0], v2[1]]
            requests.append('tp_bisec_curves ' + numbers([0] + c1 + c2 + [sense, 0, 1e-7, 0] + passage))
            # The concentric arcs "on the curve" select no GccAna solution: the native
            # Bisector_Bisec then dereferences the null analytic bisector (a crash).
            if index != 2:
                requests.append('tp_bisec_curves ' + numbers([0] + c1 + c2 + [sense, 2, 1e-7, 1] + passage))
    return requests


def curve_point_requests() -> list[str]:
    """Bisector_Bisec of a curve ending (which 0) or starting (which 1) at the corner and a
    point, on both sides."""
    points = [(1.0, 0.5), (-0.5, 1.0), (0.3, -0.8), (2.0, 2.0)]
    requests: list[str] = []
    for which, curves in ((0, into_corner()), (1, out_of_corner())):
        for i, (_, c) in enumerate(curves):
            for k, p in enumerate(points[:2] if i % 2 else points[2:]):
                for sense in (1.0, -1.0):
                    requests.append('tp_bisec_curve_point ' + numbers([which, 1] + c + [p[0], p[1], sense, 1e-7, 1]))
                off = [p[0] * 0.5, p[1] * 0.5, 1.0, 0.0, 0.0, 1.0]
                requests.append('tp_bisec_curve_point ' + numbers([which, 0] + c + [p[0], p[1], 1.0, 1e-7, 0] + off))
    return requests


def points_requests() -> list[str]:
    """Bisector_Bisec of two points, from their middle and elsewhere, on and off the curve."""
    requests: list[str] = []
    pairs = [((0.0, 0.0), (2.0, 0.0)), ((1.0, 1.0), (-1.0, 3.0)), ((0.1, 0.2), (0.3, -0.4)), ((5.0, 5.0), (5.0, 5.0))]
    for a, b in pairs:
        mid = ((a[0] + b[0]) / 2, (a[1] + b[1]) / 2)
        for sense, oncurve in ((1.0, 1), (-1.0, 1), (1.0, 0)):
            passage = [mid[0], mid[1], b[1] - a[1], a[0] - b[0], a[1] - b[1], b[0] - a[0]]
            if a == b:
                passage = [mid[0], mid[1], 1.0, 0.0, 0.0, 1.0]
            requests.append('tp_bisec_points ' + numbers([a[0], a[1], b[0], b[1]] + passage + [sense, 0.0, oncurve]))
    return requests


def ana_requests() -> list[str]:
    """Bisector_BisecAna directly: every overload on circles, lines and points, including a
    Perform of curves that are not circles or lines (StdFail_NotDone)."""
    requests: list[str] = []
    analytic_in = [c for name, c in into_corner() if name.startswith(('seg', 'arc'))]
    analytic_out = [c for name, c in out_of_corner() if name.startswith(('seg', 'arc'))]
    for i, c1 in enumerate(analytic_in):
        for j, c2 in enumerate(analytic_out):
            if (i + j) % 2:
                continue
            sense = 1.0 if i % 2 == 0 else -1.0
            requests.append('tp_bisec_ana ' + numbers([0, 1] + c1 + c2 + [sense, (i + j) % 3, 1e-7, 1]))
    requests.append('tp_bisec_ana ' + numbers([0, 1] + into_corner()[7][1] + out_of_corner()[0][1] + [1.0, 0, 1e-7, 1]))
    same = circle((0.0, 0.0), 1.0)
    requests.append('tp_bisec_ana ' + numbers([0, 1] + trimmed(0.0, 1.0, same) + trimmed(1.0, 2.0, same) + [1.0, 0, 1e-7, 1]))
    requests.append('tp_bisec_ana ' + numbers([0, 1] + trimmed(0.0, 1.0, same) + trimmed(1.0, 2.0, same) + [-1.0, 0, 1e-7, 1]))
    for which, curves in ((1, analytic_in), (2, analytic_out)):
        for i, c in enumerate(curves):
            p = (0.7, 0.4) if i % 2 else (-0.4, -0.9)
            for sense in (1.0, -1.0):
                requests.append('tp_bisec_ana ' + numbers([which, 1] + c + [p[0], p[1], sense, 1e-7, 1]))
    for a, b in (((0.0, 0.0), (2.0, 0.0)), ((1.0, 1.0), (-1.0, 3.0))):
        mid = ((a[0] + b[0]) / 2, (a[1] + b[1]) / 2)
        passage = [mid[0], mid[1], b[1] - a[1], a[0] - b[0], a[1] - b[1], b[0] - a[0]]
        for sense in (1.0, -1.0):
            requests.append('tp_bisec_ana ' + numbers([3, a[0], a[1], b[0], b[1]] + passage + [sense, 0.0, 0]))
    return requests


def cc_requests() -> list[str]:
    """Bisector_BisecCC of curves facing each other or meeting at an end."""
    pairs = [
        (segment((0.0, 0.0), (2.0, 0.0)), arc((1.0, 2.0), 1.0, -math.pi / 4, -3 * math.pi / 4), (1.0, 0.5), 1.0, 1.0),
        (bspline([(0.0, 0.0), (1.0, 0.5), (2.0, 0.0)], 2), segment((2.0, 2.0), (0.0, 2.0)), (0.0, 1.0), 1.0, 1.0),
        (arc((0.0, 0.0), 2.0, math.pi / 2, 0.0), bspline([(0.0, 1.0), (0.6, 0.6), (1.0, 0.0)], 2), (0.0, 1.5), -1.0, 1.0),
        (bspline([(-2.0, 1.0), (-1.0, 1.5), (-0.5, 0.2), (0.0, 0.0)], 3), bspline([(0.0, 0.0), (0.5, 0.8), (1.5, 0.5), (2.0, 1.0)], 3), (0.0, 0.0), 1.0, 1.0),
        (bspline([(-2.0, 1.0), (-1.0, 1.5), (-0.5, 0.2), (0.0, 0.0)], 3), bspline([(0.0, 0.0), (0.5, 0.8), (1.5, 0.5), (2.0, 1.0)], 3), (0.0, 0.0), -1.0, -1.0),
        (trimmed(math.pi / 2, math.pi, ellipse((1.0, 0.0), (1.0, 0.0), 1.0, 0.5)), segment((0.0, 0.0), (0.0, 1.5)), (0.0, 0.0), 1.0, 1.0),
        (segment((0.0, 0.0), (3.0, 0.0)), bspline([(3.0, 2.0), (1.5, 1.0), (0.0, 2.5)], 2), (0.5, 1.0), 1.0, 1.0),
    ]
    requests: list[str] = []
    for c1, c2, origin, s1, s2 in pairs:
        for dmax in (500.0, 0.8):
            requests.append('tp_bisec_cc ' + numbers(c1 + c2 + [s1, s2, origin[0], origin[1], dmax]))
    return requests


def pc_requests() -> list[str]:
    """Bisector_BisecPC of curves and points, with DistMax and on bounds."""
    cases = [
        (circle((0.0, 0.0), 2.0), (0.5, 0.0), 1.0),
        (arc((0.0, 0.0), 2.0, 0.0, math.pi), (0.3, 0.4), 1.0),
        (arc((0.0, 0.0), 2.0, 0.0, math.pi), arc_point((0.0, 0.0), 2.0, math.pi), 1.0),
        (segment((0.0, 0.0), (2.0, 0.0)), (1.0, 1.0), 1.0),
        (bspline([(0.0, 0.0), (1.0, 1.0), (2.0, 0.0), (3.0, 1.0)], 3), (1.5, 2.0), 1.0),
        (bspline([(0.0, 0.0), (1.0, 1.0), (2.0, 0.0), (3.0, 1.0)], 3), (0.0, 0.0), -1.0),
        (bspline([(0.0, 0.0), (1.0, 1.0), (2.0, 0.0), (3.0, 1.0)], 3), (3.0, 1.0), 1.0),
        (trimmed(-1.0, 1.0, parabola((0.0, 0.0), (0.0, 1.0), 0.25)), (0.0, 1.0), 1.0),
    ]
    requests: list[str] = []
    for c, p, side in cases:
        for dmax in (500.0, 1.2):
            requests.append('tp_bisec_pc ' + numbers([0] + c + [p[0], p[1], side, dmax]))
        requests.append('tp_bisec_pc ' + numbers([1] + c + [p[0], p[1], side, 0.2, 0.8]))
    return requests


def inter_requests() -> list[str]:
    """Bisector_Inter of the bisectors of two corners of triangles and curved triangles."""
    a, b, c = (0.0, 0.0), (4.0, 0.0), (1.0, 3.0)
    straight = [segment(a, b), segment(b, c), segment(c, a)]
    curved = [segment(a, b), arc((2.5, 0.0), 1.5, 0.0, math.pi / 2), segment((2.5, 1.5), a)]
    splined = [segment(a, b), bspline([b, (3.0, 2.0), c], 2), segment(c, a)]
    requests: list[str] = []
    for curves in (straight, curved, splined):
        for i in range(3):
            first = [0, 1] + curves[i] + curves[(i + 1) % 3] + [1.0, 0, 1e-7, 1, 1e-7]
            second = [0, 1] + curves[(i + 1) % 3] + curves[(i + 2) % 3] + [1.0, 0, 1e-7, 1, 1e-7]
            for comun in (1, 0):
                requests.append('tp_bisec_inter ' + numbers(first + second + [1e-7, 1e-7, comun]))
    point_bis = [1, 1] + straight[0] + [1.0, 3.0, 1.0, 1e-7, 1, 1e-7]
    requests.append('tp_bisec_inter ' + numbers(point_bis + [0, 1] + straight[0] + straight[1] + [1.0, 0, 1e-7, 1, 1e-7, 1e-7, 1e-7, 1]))
    return requests


def small_requests() -> list[str]:
    """The functions, the polygon and IsConvex."""
    requests: list[str] = []
    curve = bspline([(0.0, 0.0), (1.0, 1.0), (2.0, 0.0), (3.0, 1.0)], 3)
    for u in (0.0, 0.3, 0.7, 1.0):
        requests.append('tp_bisec_function_h ' + numbers(curve + [0.5, 2.0, 1.0, 0.2, u]))
        requests.append('tp_bisec_function_inter ' + numbers(circle((0.0, 0.0), 2.0) + [0.5, 0.0, -0.3, 0.4, 1.0, u * 6.0]))
    requests.append('tp_bisec_function_h ' + numbers(segment((0.0, 1.0), (2.0, 1.0)) + [0.0, 0.0, 0.0, 0.0, 0.5]))
    poly = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 0.4, 0.5, 1.1, 0.5, 0.1, 1.0, 0.9, 1.0, 1.2, 1.0, 0.2,
            1.5, 1.3, 1.5, 1.3, 1.5, 0.3]
    for u in (-0.1, 0.0, 0.2, 0.5, 0.99, 1.5, 2.0):
        requests.append('tp_bisec_poly ' + numbers([4] + poly + [u]))
    requests.append('tp_bisec_poly ' + numbers([0, 0.5]))
    requests.append('tp_bisec_poly ' + numbers([1] + poly[:6] + [0.5]))
    for c in (circle((0.0, 0.0), 1.0), segment((0.0, 0.0), (1.0, 0.0)), curve, arc((0.0, 0.0), 1.0, 0.0, -1.0)):
        for sign in (1.0, -1.0):
            requests.append('tp_bisec_is_convex ' + numbers(c + [sign]))
    return requests


def append_bisector_requests(requests: list[str]) -> None:
    """Append every Bisector request family in a fixed order."""
    for family in (curves_requests, curves_off_requests, curve_point_requests, points_requests,
                   ana_requests, cc_requests, pc_requests, inter_requests, small_requests):
        requests.extend(family())
