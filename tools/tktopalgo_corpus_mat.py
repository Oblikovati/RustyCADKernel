"""Deterministic requests for MAT and MAT2d (tools/oracle/tktopalgo_probe_mat.cxx and
tktopalgo_probe_mat_parts.cxx).

`tp_mat2d_locus` computes the bisecting locus of a figure as BRepMAT2d_BisectingLocus does
(the curves cut, the circuit, the MAT2d algorithm, the MAT graph and the fusion of the
elements of several lines) and prints the circuit and the whole graph. The figures are
polygons (convex and concave), rounded and curved contours (arcs, ellipse arcs, B-splines),
contours with holes, and open lines (the closed and the open algorithms), on both sides
and with the joins by arc and by intersection. The other requests exercise the classes
one by one: the lists, the bisectors, a hand-made graph piece, the connexions, the curve
cutting, the minimal path and the tool.
"""
from __future__ import annotations
import math
from tktopalgo_corpus_curves import Point, arc, bspline, ellipse, numbers, segment, trimmed

Line = list[list[float]]


def polygon(points: list[Point]) -> Line:
    """The segments of a closed polygon."""
    return [segment(points[i], points[(i + 1) % len(points)]) for i in range(len(points))]


def polyline(points: list[Point]) -> Line:
    """The segments of an open polyline."""
    return [segment(points[i], points[i + 1]) for i in range(len(points) - 1)]


def figure(lines: list[tuple[bool, Line]]) -> list[float]:
    """The numbers of a figure: its lines, each closed or not."""
    flat: list[float] = [len(lines)]
    for closed, items in lines:
        flat += [1 if closed else 0, len(items)]
        for item in items:
            flat += item
    return flat


def rounded_rectangle(w: float, h: float, r: float) -> Line:
    """A rectangle [0, w] x [0, h] with corners rounded by arcs of radius r (CCW)."""
    half = math.pi / 2
    return [
        segment((r, 0.0), (w - r, 0.0)), arc((w - r, r), r, -half, 0.0),
        segment((w, r), (w, h - r)), arc((w - r, h - r), r, 0.0, half),
        segment((w - r, h), (r, h)), arc((r, h - r), r, half, math.pi),
        segment((0.0, h - r), (0.0, r)), arc((r, r), r, math.pi, 3 * half),
    ]


def bulge() -> Line:
    """A segment closed by a half circle (a D shape, CCW)."""
    return [segment((-1.0, 0.0), (1.0, 0.0)), arc((0.0, 0.0), 1.0, 0.0, math.pi)]


def spline_contour() -> Line:
    """A B-spline arch closed by a segment (CCW)."""
    return [segment((0.0, 0.0), (4.0, 0.0)), bspline([(4.0, 0.0), (4.5, 2.0), (2.0, 3.5), (-0.5, 2.0), (0.0, 0.0)], 3)]


def wave_contour() -> Line:
    """A contour with an inflected B-spline side (cut at its inflection)."""
    return [segment((0.0, 0.0), (4.0, 0.0)), segment((4.0, 0.0), (4.0, 2.0)),
            bspline([(4.0, 2.0), (3.0, 3.0), (1.0, 1.5), (0.0, 2.0)], 3), segment((0.0, 2.0), (0.0, 0.0))]


def ellipse_contour() -> Line:
    """Half an ellipse closed by its major axis (CCW)."""
    return [segment((-2.0, 0.0), (2.0, 0.0)), trimmed(0.0, math.pi, ellipse((0.0, 0.0), (1.0, 0.0), 2.0, 1.0))]


def closed_figures() -> list[list[float]]:
    """Figures of one or more closed lines."""
    square = [(0.0, 0.0), (2.0, 0.0), (2.0, 2.0), (0.0, 2.0)]
    hole_cw = [(0.7, 0.7), (0.7, 1.3), (1.3, 1.3), (1.3, 0.7)]
    return [
        figure([(True, polygon([(0.0, 0.0), (4.0, 0.0), (1.0, 3.0)]))]),
        figure([(True, polygon(square))]),
        figure([(True, polygon([(0.0, 0.0), (5.0, 0.0), (5.0, 2.0), (0.0, 2.0)]))]),
        figure([(True, polygon([(0.0, 0.0), (4.0, 0.0), (4.0, 1.0), (1.0, 1.0), (1.0, 3.0), (0.0, 3.0)]))]),
        figure([(True, polygon([(0.0, 0.0), (3.0, -1.0), (5.0, 1.0), (3.0, 3.0), (1.0, 2.5)]))]),
        figure([(True, rounded_rectangle(4.0, 2.0, 0.5))]),
        figure([(True, bulge())]),
        figure([(True, spline_contour())]),
        figure([(True, wave_contour())]),
        figure([(True, ellipse_contour())]),
        figure([(True, polygon(square)), (True, polygon(hole_cw))]),
        figure([(True, polygon([(0.0, 0.0), (6.0, 0.0), (6.0, 3.0), (0.0, 3.0)])),
                (True, polygon([(1.0, 1.0), (1.0, 2.0), (2.0, 2.0), (2.0, 1.0)])),
                (True, polygon([(4.0, 1.0), (4.0, 2.0), (5.0, 2.0), (5.0, 1.0)]))]),
        figure([(True, rounded_rectangle(4.0, 3.0, 0.6)), (True, polygon([(1.5, 1.2), (1.5, 1.8), (2.5, 1.8), (2.5, 1.2)]))]),
    ]


def open_figures() -> list[list[float]]:
    """Figures of open lines."""
    return [
        figure([(False, polyline([(0.0, 0.0), (2.0, 0.0), (3.0, 1.0), (5.0, 1.0)]))]),
        figure([(False, polyline([(0.0, 0.0), (2.0, 0.0), (2.0, 2.0)]))]),
        figure([(False, [segment((0.0, 0.0), (2.0, 0.0)), arc((2.0, 1.0), 1.0, -math.pi / 2, 0.0)])]),
        figure([(False, [bspline([(0.0, 0.0), (1.0, 1.0), (2.0, -0.5), (3.0, 0.5)], 3)])]),
        figure([(True, polygon([(0.0, 0.0), (4.0, 0.0), (4.0, 3.0), (0.0, 3.0)])),
                (False, polyline([(1.0, 1.0), (2.0, 1.5), (3.0, 1.0)]))]),
    ]


def locus_requests() -> list[str]:
    """The locus of each figure on both sides; the joins by arc and by intersection; the
    open lines with the closed and the open algorithms."""
    requests: list[str] = []
    for k, fig in enumerate(closed_figures()):
        for side in (0, 1):
            requests.append('tp_mat2d_locus ' + numbers(fig + [side, 0, 0]))
        requests.append('tp_mat2d_locus ' + numbers(fig + [k % 2, 2, 0]))
    for k, fig in enumerate(open_figures()):
        for side, join, open_result in ((0, 0, 0), (1, 0, 0), (0, 0, 1), (0, 2, 1)):
            # Left out: the open algorithm with joins by arc on a figure that also has a
            # closed line. MAT2d_Tool2d reads a point item of the circuit as a curve (a null
            # down_cast) and dereferences it: a native crash (Rust: Standard_NullObject).
            if k == 4 and (join, open_result) == (0, 1):
                continue
            requests.append('tp_mat2d_locus ' + numbers(fig + [side, join, open_result]))
    return requests


def list_requests() -> list[str]:
    """Edit scripts of MAT_ListOfEdge (no edit dereferences a missing node)."""
    scripts = [
        (3, [(0, 0), (2, 0), (4, 3), (4, 1), (1, 0), (3, 0)]),
        (4, [(10, 0), (0, 0), (2, 0), (2, 0), (2, 0), (2, 0), (2, 0), (3, 0), (4, 2)]),
        (4, [(0, 0), (2, 0), (5, 0), (2, 0), (7, 9), (6, 8), (0, 0), (9, 0)]),
        (3, [(10, 0), (11, 2), (5, 0), (2, 0), (12, 7), (0, 0), (5, 0), (2, 0)]),
        (2, [(8, 5), (13, 6), (0, 0), (9, 0), (4, 4), (10, 0), (3, 0), (3, 0)]),
        (0, [(13, 1), (8, 2), (0, 0), (2, 0), (2, 0)]),
    ]
    requests: list[str] = []
    for n, ops in scripts:
        flat = [n, len(ops)]
        for op, arg in ops:
            flat += [op, arg]
        requests.append('tp_mat_list ' + numbers(flat))
    return requests


def parts_requests() -> list[str]:
    """The bisector dump, the graph piece, the connexions, the curve cuts, the minimal
    paths and the tool."""
    requests = ['tp_mat_bisector ' + numbers([s, lv]) for s, lv in ((0, 0), (0, 1), (2, 1))]
    requests += ['tp_mat_arc 0', 'tp_mat2d_tool 0']
    base = [1, 2, 3, 4, 1.5, 0.25, 0.5, 0.0, 0.0, 1.0, 1.0]
    for other, sense in (([1, 3, 3, 1, 2.0, 0.25, 0.1, 0.0, 0.0, 1.0, -1.0], 1.0),
                         ([1, 3, 3, 1, 2.0, 0.25, 0.1, 0.0, 0.0, 1.0, -1.0], -1.0),
                         ([1, 2, 2, 1, 1.0, 0.5, 0.1, 0.0, 0.0, 0.0, 1.0], 1.0),
                         ([2, 2, 3, 1, 1.0, 0.5, 0.1, 0.0, 0.0, 0.0, 1.0], 1.0)):
        requests.append('tp_mat2d_connexion ' + numbers(base + other + [sense]))
    cuts = [
        bspline([(0.0, 0.0), (1.0, 1.0), (2.0, -1.0), (3.0, 0.0)], 3),
        bspline([(0.0, 0.0), (1.0, 2.0), (2.0, 2.0), (3.0, 0.0)], 3),
        trimmed(0.0, math.pi, ellipse((0.0, 0.0), (1.0, 0.0), 2.0, 1.0)),
        trimmed(-1.0, 3.0, ellipse((0.0, 0.0), (1.0, 0.0), 2.0, 1.0)),
        arc((0.0, 0.0), 1.0, 0.0, 2.0),
        segment((0.0, 0.0), (1.0, 1.0)),
    ]
    requests += ['tp_mat2d_cut ' + numbers(c) for c in cuts]
    square = [(0.0, 0.0), (6.0, 0.0), (6.0, 6.0), (0.0, 6.0)]
    holes = [[(1.0, 1.0), (1.0, 2.0), (2.0, 2.0), (2.0, 1.0)], [(4.0, 4.0), (4.0, 5.0), (5.0, 5.0), (5.0, 4.0)],
             [(4.0, 1.0), (4.0, 2.0), (5.0, 1.5)]]
    fig = figure([(True, polygon(square))] + [(True, polygon(h)) for h in holes])
    for ind, sense in ((1, 1), (1, 0), (2, 1), (4, 0)):
        requests.append('tp_mat2d_minipath ' + numbers(fig + [ind, sense]))
    circles = figure([(True, [arc((0.0, 0.0), 1.0, 0.0, 6.0)]), (True, [arc((3.0, 0.5), 0.5, 0.0, 6.0)]),
                      (True, [bspline([(0.0, 3.0), (1.0, 4.0), (2.0, 3.0)], 2)])])
    requests.append('tp_mat2d_minipath ' + numbers(circles + [1, 1]))
    return requests


def append_mat_requests(requests: list[str]) -> None:
    """Append every MAT and MAT2d request family in a fixed order."""
    for family in (locus_requests, list_requests, parts_requests):
        requests.extend(family())
