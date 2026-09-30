"""Shapes of the BRepExtrema requests (tools/tktopalgo_corpus_brepextrema.py), spelled as
`tp_shape` records (tktopalgo_corpus_shapes.py).

Single vertices and edges (segments, arcs, circles, C0 B-splines, Bezier curves, infinite
lines, edges with only a 3D polygon), plane faces (bounded or natural, finite or
infinite), cylinder, sphere and torus faces, and the named solids of
tktopalgo_corpus_solids.py; the `meshed_*` shapes carry grid triangulations of their faces
and 3D polygons of their edges, for the proximity tools.
"""
from __future__ import annotations
import math
from tktopalgo_corpus_shapes import (BEZIER, BSPLINE, CIRCLE, EDGE, FACE, LINE, NO_CURVE, PLANE, SHELL, VERTEX,
                                     FaceRecord, Mesh, ShapeSpec)
from tktopalgo_corpus_solids import ccw, cross, dot, line_edge, planar_face, sub

INFINITE = 2e100


def unit(v: list[float]) -> list[float]:
    """The unit vector of v."""
    n = math.sqrt(dot(v, v))
    return [c / n for c in v]


def vertex_shape(point: list[float], tol: float = 1e-7) -> ShapeSpec:
    """One vertex."""
    spec = ShapeSpec()
    spec.vertex(point, tol)
    spec.top = [(VERTEX, 0)]
    return spec


def points_shape(points: list[list[float]]) -> ShapeSpec:
    """A compound of vertices."""
    spec = ShapeSpec()
    for p in points:
        spec.vertex(p)
    spec.top = [(VERTEX, i) for i in range(len(points))]
    return spec


def polyline(points: list[list[float]], n: int) -> list[list[float]]:
    """`n` points along a polyline through `points` (the first and last included)."""
    return [points[0]] if n < 2 else [[points[0][k] + (points[1][k] - points[0][k]) * i / (n - 1) for k in range(3)]
                                      for i in range(n)]


def segment_shape(a: list[float], b: list[float], tol: float = 1e-7, polygon: int = 0) -> ShapeSpec:
    """One segment edge from a to b; a 3D polygon of `polygon` points when given."""
    spec = ShapeSpec()
    va, vb = spec.vertex(a, tol), spec.vertex(b, tol)
    length = math.sqrt(dot(sub(b, a), sub(b, a)))
    nodes = None if polygon < 2 else (polyline([a, b], polygon), 1e-3)
    spec.edge([LINE, *a, *unit(sub(b, a))], va, vb, (0.0, length), tol, polygon=nodes)
    spec.top = [(EDGE, 0)]
    return spec


def arc_points(centre: list[float], normal: list[float], x_dir: list[float], radius: float,
               span: tuple[float, float], n: int) -> list[list[float]]:
    """`n` points of a circle arc on `span`."""
    y_dir = cross(normal, x_dir)
    ts = [span[0] + (span[1] - span[0]) * i / (n - 1) for i in range(n)]
    return [[centre[k] + radius * (math.cos(t) * x_dir[k] + math.sin(t) * y_dir[k]) for k in range(3)] for t in ts]


def arc_shape(centre: list[float], normal: list[float], x_dir: list[float], radius: float,
              span: tuple[float, float] = (0.0, 2.0 * math.pi), polygon: int = 0) -> ShapeSpec:
    """A circle arc edge (one vertex when closed); a 3D polygon of `polygon` points when given."""
    spec = ShapeSpec()
    ends = arc_points(centre, normal, x_dir, radius, span, 2)
    closed = abs(span[1] - span[0] - 2.0 * math.pi) < 1e-12
    v0 = spec.vertex(ends[0])
    v1 = v0 if closed else spec.vertex(ends[1])
    nodes = None if polygon < 2 else (arc_points(centre, normal, x_dir, radius, span, polygon), 1e-3)
    spec.edge([CIRCLE, *centre, *normal, *x_dir, radius], v0, v1, span, polygon=nodes)
    spec.top = [(EDGE, 0)]
    return spec


def bezier_value(poles: list[list[float]], t: float) -> list[float]:
    """The point of a Bezier curve (de Casteljau)."""
    points = [list(p) for p in poles]
    while len(points) > 1:
        points = [[(1.0 - t) * a[k] + t * b[k] for k in range(3)] for a, b in zip(points, points[1:])]
    return points[0]


def bezier_shape(poles: list[list[float]], polygon: int = 0) -> ShapeSpec:
    """A Bezier edge on [0, 1]; a 3D polygon of `polygon` points on the curve when given."""
    spec = ShapeSpec()
    v0, v1 = spec.vertex(poles[0]), spec.vertex(poles[-1])
    nodes = None if polygon < 2 else ([bezier_value(poles, i / (polygon - 1)) for i in range(polygon)], 1e-3)
    spec.edge([BEZIER, len(poles), *[c for p in poles for c in p]], v0, v1, (0.0, 1.0), polygon=nodes)
    spec.top = [(EDGE, 0)]
    return spec


def c0_bspline_shape(poles: list[list[float]]) -> ShapeSpec:
    """A degree 2 B-spline edge on [0, 1] with a C0 knot at 0.5 (a corner): five poles."""
    spec = ShapeSpec()
    v0, v1 = spec.vertex(poles[0]), spec.vertex(poles[-1])
    knots = [0.0, 3, 0.5, 2, 1.0, 3]
    spec.edge([BSPLINE, 2, len(poles), *[c for p in poles for c in p], 3, *knots], v0, v1, (0.0, 1.0))
    spec.top = [(EDGE, 0)]
    return spec


def infinite_line_shape(point: list[float], direction: list[float], first: float = -INFINITE,
                        last: float = INFINITE) -> ShapeSpec:
    """A line edge infinite at one end or both (no vertex at an infinite end)."""
    spec = ShapeSpec()
    d = unit(direction)
    v0 = -1 if first <= -INFINITE else spec.vertex([point[k] + first * d[k] for k in range(3)])
    v1 = -1 if last >= INFINITE else spec.vertex([point[k] + last * d[k] for k in range(3)])
    spec.edge([LINE, *point, *d], v0, v1, (first, last))
    spec.top = [(EDGE, 0)]
    return spec


def polygon_edge_shape(points: list[list[float]]) -> ShapeSpec:
    """An edge with only a 3D polygon (no curve: not geometric)."""
    spec = ShapeSpec()
    v0, v1 = spec.vertex(points[0]), spec.vertex(points[-1])
    spec.edge([NO_CURVE], v0, v1, (0.0, 1.0), polygon=(points, 1e-3))
    spec.top = [(EDGE, 0)]
    return spec


def grid_mesh(point, n: int, m: int, corner=True) -> tuple[list[list[float]], list[tuple[int, int, int]]]:
    """The nodes `point(i/n, j/m)` of an (n+1) x (m+1) grid and its two triangles per cell,
    counterclockwise in the parameters."""
    nodes = [point(i / n, j / m) for j in range(m + 1) for i in range(n + 1)]
    triangles = []
    for j in range(m):
        for i in range(n):
            a = j * (n + 1) + i + 1
            b, c, d = a + 1, a + n + 1, a + n + 2
            triangles += [(a, b, d), (a, d, c)] if corner else [(a, b, c), (b, d, c)]
    return nodes, triangles


def rectangle_shape(origin: list[float], normal: list[float], x_dir: list[float], size: tuple[float, float],
                    tol: float = 1e-7, mesh: tuple[int, int] | None = None, polygon: int = 0) -> ShapeSpec:
    """A plane rectangle face (corner at origin, sides along x_dir and normal ^ x_dir); a grid
    mesh of mesh[0] x mesh[1] cells and 3D polygons of `polygon` points on its edges when given."""
    spec = ShapeSpec()
    y_dir = cross(normal, x_dir)
    corner = lambda u, v: [origin[k] + u * size[0] * x_dir[k] + v * size[1] * y_dir[k] for k in range(3)]
    corners = [spec.vertex(corner(u, v), tol) for u, v in ((0, 0), (1, 0), (1, 1), (0, 1))]
    for i in range(4):
        a, b = corners[i], corners[(i + 1) % 4]
        line_edge(spec, a, b, tol)
        if polygon >= 2:
            spec.edges[-1][-1:] = [polygon, *[c for p in polyline([spec.vertices[a][:3], spec.vertices[b][:3]], polygon)
                                              for c in p], 1e-3]
    face = planar_face(spec, ccw(spec, corners, normal, x_dir), normal, x_dir, tol)
    if mesh:
        nodes, triangles = grid_mesh(corner, *mesh)
        face.mesh = Mesh(nodes, triangles, 1e-3)
    spec.face(face)
    spec.top = [(FACE, 0)]
    return spec


def natural_plane_shape(origin: list[float], normal: list[float], x_dir: list[float]) -> ShapeSpec:
    """The whole plane as a natural-restriction face (infinite, no wire)."""
    spec = ShapeSpec()
    spec.face(FaceRecord([PLANE, *origin, *normal, *x_dir], natural=True))
    spec.top = [(FACE, 0)]
    return spec


def with_mesh(spec: ShapeSpec, face: int, point, cells: tuple[int, int], corner: bool = True) -> ShapeSpec:
    """Attaches the grid mesh of `point(s, t)` on [0, 1]^2 to a face of the spec."""
    nodes, triangles = grid_mesh(point, *cells, corner)
    spec.faces[face].mesh = Mesh(nodes, triangles, 1e-3)
    return spec


def cylinder_point(origin: list[float], radius: float, height: float, span=(0.0, 2.0 * math.pi)):
    """The point of the z-axis cylinder face of add_cylinder at (s, t) of [0, 1]^2."""
    return lambda s, t: [origin[0] + radius * math.cos(span[0] + s * (span[1] - span[0])),
                         origin[1] + radius * math.sin(span[0] + s * (span[1] - span[0])), origin[2] + t * height]


def sphere_point(centre: list[float], radius: float):
    """The point of the sphere face of add_sphere at (s, t) of [0, 1]^2."""
    def point(s: float, t: float) -> list[float]:
        u, v = 2.0 * math.pi * s, -0.5 * math.pi + math.pi * t
        return [centre[0] + radius * math.cos(v) * math.cos(u), centre[1] + radius * math.cos(v) * math.sin(u),
                centre[2] + radius * math.sin(v)]
    return point


def shell_of(spec: ShapeSpec) -> ShapeSpec:
    """The same record with its first shell as the top shape."""
    spec.top = [(SHELL, 0)]
    return spec


def face_of(spec: ShapeSpec, face: int = 0) -> ShapeSpec:
    """The same record with one face as the top shape."""
    spec.top = [(FACE, face)]
    return spec
