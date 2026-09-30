"""Deterministic requests for BRepLib_FuseEdges (tools/oracle/tktopalgo_probe_breplib_fuse.cxx).

The shapes are wires of pieces of one curve (lines, circles, ellipses, open and closed
B-splines, Bezier and trimmed curves, with a change of curve type), polygons with
collinear and doubling-back sides, planar faces of those wires, a face whose split side
has 3d lines and p-curves, and multi-patch shells; each runs with and without B-spline
concatenation, with avoided edges, and through the three call orders.
"""
from __future__ import annotations
import math
from tktopalgo_corpus_breplib_package import compound, face, shell, shells, spline_grid
from tktopalgo_corpus_breplib_records import geom_bezier, geom_bspline, geom_gp, geom_trimmed, numbers
from tktopalgo_corpus_faces import FORWARD, SEGMENT_3D, Z_PLANE, FaceSpec

TWO_PI = 2 * math.pi


def polygon(points, closed: bool = True) -> list[float]:
    return [5, len(points)] + [c for p in points for c in p] + [int(closed)]


def planar_face(wire: list[float], only_plane: bool = True) -> list[float]:
    return [10] + wire + [int(only_plane)]


def pieces(curve: list[float], params: list[float], edges: list[tuple[int, int, float, float]]) -> list[float]:
    """Record 11: vertices at `params`, then edges (vertex i, vertex j, first, last)."""
    return [11] + curve + [len(params)] + params + [len(edges)] + [c for e in edges for c in e]


def chain(curve: list[float], params: list[float], closed: bool = False) -> list[float]:
    """Consecutive pieces between the parameters, closed on the first vertex."""
    edges = [(i, i + 1, params[i], params[i + 1]) for i in range(len(params) - 1)]
    if closed:
        vertices = params[:-1]
        edges[-1] = (len(vertices) - 1, 0, params[-2], params[-1])
        return pieces(curve, vertices, edges)
    return pieces(curve, params, edges)


def split_square_face() -> FaceSpec:
    """The square [0, 2]^2 whose bottom side is two collinear edges with 3d lines."""
    spec = FaceSpec(list(Z_PLANE))
    points = [(0., 0.), (1., 0.), (2., 0.), (2., 2.), (0., 2.)]
    vertices = [spec.vertex(u, v) for u, v in points]
    n = len(points)
    edges = [spec.edge(SEGMENT_3D, [*points[i], *points[(i + 1) % n]], vertices[i], vertices[(i + 1) % n])
             for i in range(n)]
    spec.wire([(e, FORWARD) for e in edges])
    return spec


def closed_bspline() -> list[float]:
    poles = [(1., 0., 0.), (1., 1., 0.), (-1., 1., 0.), (-1., -1., 0.), (1., -1., 0.), (1., 0., 0.)]
    return geom_bspline(3, False, poles, [(0., 4), (0.5, 2), (1., 4)])


def open_bspline() -> list[float]:
    poles = [(0., 0., 0.), (1., 2., 0.), (2., -1., 0.), (3., 1., 0.), (4., 0., 1.)]
    return geom_bspline(3, False, poles, [(0., 4), (0.5, 1), (1., 4)])


def wires() -> list[list[float]]:
    circle, ellipse = geom_gp(1), geom_gp(2)
    line = geom_gp(0)
    bez = geom_bezier([(0., 0., 0.), (1., 1., 0.), (2., 0., 0.), (3., 1., 1.)])
    arcs_then_line = pieces(circle, [0., 2., 4.], [(0, 1, 0., 2.), (1, 2, 2., 4.)])
    return [chain(circle, [0., 2., 4., TWO_PI], closed=True), chain(circle, [0., 2., 4.]),
            chain(circle, [1., 3., 5., 1. + TWO_PI], closed=True), chain(ellipse, [0., 1., 3., TWO_PI], closed=True),
            chain(ellipse, [0.5, 1.5, 2.5]), chain(line, [0., 1., 2.5, 4.]), chain(geom_trimmed(0., 3., line), [0.5, 1., 2.]),
            chain(open_bspline(), [0., 0.4, 1.]), chain(open_bspline(), [0., 0.2, 0.5, 0.8, 1.]),
            chain(closed_bspline(), [0., 0.5, 1.], closed=True), chain(closed_bspline(), [0.25, 0.5, 0.75, 1.]),
            pieces(closed_bspline(), [0.25, 0.5, 0.75, 1.], [(0, 1, 0.25, 0.5), (1, 2, 0.5, 0.75), (2, 3, 0.75, 1.),
                                                             (3, 0, 0., 0.25)]),
            chain(bez, [0., 0.5, 1.]), chain(bez, [0., 0.3, 0.6, 1.]), arcs_then_line,
            polygon([(0., 0., 0.), (1., 0., 0.), (2., 0., 0.), (2., 1., 0.), (0., 1., 0.)]),
            polygon([(0., 0., 0.), (1., 0., 0.), (2., 0., 0.), (3., 0., 0.)], closed=False),
            polygon([(0., 0., 0.), (1., 0., 0.), (0., 0., 0.)], closed=False),
            polygon([(0., 0., 0.), (1., 0., 0.), (1., 1., 0.), (0., 1., 0.)])]


def shapes() -> list[list[float]]:
    faces = [planar_face(w) for w in wires()[:4] + wires()[15:16] + [wires()[9], wires()[18]]]
    return wires() + faces + [face(split_square_face()), shell(spline_grid()), shell(spline_grid(), segment=True),
                              shells()[0], compound([wires()[0], wires()[15]])]


def request(fields: list[float]) -> str:
    return 'blp_fuseedges ' + numbers(fields)


def append_breplib_fuse_requests(requests: list[str]) -> None:
    """Append the FuseEdges requests in a fixed order."""
    for shape in shapes():
        for concat in (0, 1):
            requests.append(request(shape + [concat, 0, 0]))
        requests += [request(shape + [0, 0, 1]), request(shape + [1, 0, 2])]
    for shape in [wires()[0], wires()[5], wires()[15], face(split_square_face())]:
        requests += [request(shape + [0, 1, 1, 0]), request(shape + [0, 1, 2, 0]), request(shape + [1, 2, 1, 2, 0])]
