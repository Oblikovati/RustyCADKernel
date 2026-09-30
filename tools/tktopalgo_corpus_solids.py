"""Named shapes of the TKTopAlgo oracle, spelled as `tp_shape` records (tktopalgo_corpus_shapes.py).

Boxes (with 3d polygons and triangulations on request), hollow boxes, open shells,
cylinders, spheres and tori, each a solid of closed shells whose faces are FORWARD with
outward normals and whose edges are shared; the pcurves follow the edge
parametrizations (lines on [0, length], circles on their angle).
"""
from __future__ import annotations
import math
from tktopalgo_corpus_shapes import (
    CIRCLE, CYLINDER, FORWARD, LINE, NO_CURVE, PCIRCLE, PLANE, PLINE, REVERSED, SPHERE, TORUS,
    FaceRecord, Mesh, ShapeSpec)

TWO_PI = 2.0 * math.pi
HALF_PI = 0.5 * math.pi


def sub(a: list[float], b: list[float]) -> list[float]:
    return [x - y for x, y in zip(a, b)]


def dot(a: list[float], b: list[float]) -> float:
    return sum(x * y for x, y in zip(a, b))


def cross(a: list[float], b: list[float]) -> list[float]:
    return [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]


def axis(index: int, sign: float = 1.0) -> list[float]:
    """The unit vector of axis `index` (0 x, 1 y, 2 z) times `sign`."""
    return [sign if i == index else 0.0 for i in range(3)]


def line_edge(spec: ShapeSpec, a: int, b: int, tol: float = 1e-7, polygon: float | None = None) -> int:
    """A segment edge from vertex a to vertex b on [0, length]; a 3d polygon of its ends
    with deflection `polygon` when given."""
    pa, pb = spec.vertices[a][:3], spec.vertices[b][:3]
    length = math.sqrt(dot(sub(pb, pa), sub(pb, pa)))
    direction = [c / length for c in sub(pb, pa)]
    nodes = None if polygon is None else ([pa, pb], polygon)
    return spec.edge([LINE, *pa, *direction], a, b, (0.0, length), tol, polygon=nodes)


def planar_face(spec: ShapeSpec, corners: list[int], normal: list[float], x_dir: list[float],
                tol: float = 1e-7, mesh: float | None = None) -> FaceRecord:
    """A plane face (frame at the first corner) bounded by the segment edges joining the
    corners (in counterclockwise order about `normal`); a two-triangle-per-corner fan mesh
    with polygons of the edges when `mesh` (its deflection) is given."""
    origin = spec.vertices[corners[0]][:3]
    y_dir = cross(normal, x_dir)
    uv = {c: (dot(sub(spec.vertices[c][:3], origin), x_dir), dot(sub(spec.vertices[c][:3], origin), y_dir))
          for c in corners}
    face = FaceRecord([PLANE, *origin, *normal, *x_dir], tol)
    uses, polygons = [], []
    for i, a in enumerate(corners):
        b = corners[(i + 1) % len(corners)]
        edge, orientation = find_edge(spec, a, b)
        start = a if orientation == FORWARD else b
        end = b if orientation == FORWARD else a
        du, dv = uv[end][0] - uv[start][0], uv[end][1] - uv[start][1]
        length = math.hypot(du, dv)
        uses.append((edge, orientation, [PLINE, *uv[start], du / length, dv / length]))
        polygons.append((edge, [corners.index(start) + 1, corners.index(end) + 1], mesh or 0.0))
    face.wire(uses)
    if mesh is not None:
        nodes = [spec.vertices[c][:3] for c in corners]
        triangles = [(1, i, i + 1) for i in range(2, len(corners))]
        face.mesh = Mesh(nodes, triangles, mesh, polygons)
    return face


def find_edge(spec: ShapeSpec, a: int, b: int) -> tuple[int, int]:
    """The segment edge joining vertices a and b, and its orientation from a to b."""
    for index, edge in enumerate(spec.edges):
        if edge[0] == LINE and edge[7:9] == [a, b]:
            return index, FORWARD
        if edge[0] == LINE and edge[7:9] == [b, a]:
            return index, REVERSED
    raise ValueError(f'no edge between vertices {a} and {b}; expected a segment edge')


def add_box(spec: ShapeSpec, low: list[float], high: list[float], tol: float = 1e-7,
            polygon: float | None = None, mesh: float | None = None) -> list[int]:
    """The 8 vertices, 12 edges and 6 faces of an axis-aligned box; the face indices."""
    base = len(spec.vertices)
    for k in range(8):
        spec.vertex([high[i] if (k >> i) & 1 else low[i] for i in range(3)], tol)
    for a in range(3):
        for k in range(8):
            if not (k >> a) & 1:
                line_edge(spec, base + k, base + (k | (1 << a)), tol, polygon)
    faces = []
    for a in range(3):
        for side in (0, 1):
            corners = [base + k for k in range(8) if ((k >> a) & 1) == side]
            normal = axis(a, 1.0 if side else -1.0)
            x_dir = axis((a + 1) % 3)
            faces.append(spec.face(planar_face(spec, ccw(spec, corners, normal, x_dir), normal, x_dir, tol, mesh)))
    return faces


def ccw(spec: ShapeSpec, corners: list[int], normal: list[float], x_dir: list[float]) -> list[int]:
    """The corners sorted counterclockwise about `normal` around their centre."""
    y_dir = cross(normal, x_dir)
    centre = [sum(spec.vertices[c][i] for c in corners) / len(corners) for i in range(3)]

    def angle(c: int) -> float:
        d = sub(spec.vertices[c][:3], centre)
        return math.atan2(dot(d, y_dir), dot(d, x_dir))
    return sorted(corners, key=angle)


def box(low: list[float], high: list[float], tol: float = 1e-7, polygon: float | None = None,
        mesh: float | None = None) -> ShapeSpec:
    """The solid of one axis-aligned box."""
    spec = ShapeSpec()
    spec.shell(add_box(spec, low, high, tol, polygon, mesh))
    return spec


def hollow_box(low: list[float], high: list[float], cavity_low: list[float], cavity_high: list[float]) -> ShapeSpec:
    """A box solid with a box cavity (the inner shell REVERSED)."""
    spec = ShapeSpec()
    spec.shell(add_box(spec, low, high))
    spec.shell(add_box(spec, cavity_low, cavity_high), orientation=REVERSED)
    return spec


def open_box(low: list[float], high: list[float], missing: int = 5) -> ShapeSpec:
    """The solid of a box shell missing face `missing` (not closed)."""
    spec = ShapeSpec()
    faces = add_box(spec, low, high)
    spec.shell([f for i, f in enumerate(faces) if i != missing], closed=False)
    return spec


def add_cylinder(spec: ShapeSpec, origin: list[float], radius: float, height: float, tol: float = 1e-7) -> list[int]:
    """A z-axis cylinder of `radius` and `height` from `origin`: its three faces (its
    vertices and edges of tolerance `tol`)."""
    n, x = axis(2), axis(0)
    top = [origin[0], origin[1], origin[2] + height]
    vb = spec.vertex([origin[0] + radius, origin[1], origin[2]], tol)
    vt = spec.vertex([top[0] + radius, top[1], top[2]], tol)
    bottom = spec.edge([CIRCLE, *origin, *n, *x, radius], vb, vb, (0.0, TWO_PI), tol)
    upper = spec.edge([CIRCLE, *top, *n, *x, radius], vt, vt, (0.0, TWO_PI), tol)
    seam = spec.edge([LINE, *spec.vertices[vb][:3], *n], vb, vt, (0.0, height), tol)
    lateral = FaceRecord([CYLINDER, *origin, *n, *x, radius])
    lateral.wire([(bottom, FORWARD, [PLINE, 0.0, 0.0, 1.0, 0.0]), (seam, FORWARD, [PLINE, TWO_PI, 0.0, 0.0, 1.0]),
                  (upper, REVERSED, [PLINE, 0.0, height, 1.0, 0.0]), (seam, REVERSED, [PLINE, 0.0, 0.0, 0.0, 1.0])])
    floor = FaceRecord([PLANE, *origin, *axis(2, -1.0), *x])
    floor.wire([(bottom, REVERSED, [PCIRCLE, 0.0, 0.0, radius, 0])])
    roof = FaceRecord([PLANE, *top, *n, *x])
    roof.wire([(upper, FORWARD, [PCIRCLE, 0.0, 0.0, radius, 1])])
    return [spec.face(lateral), spec.face(floor), spec.face(roof)]


def cylinder(origin: list[float], radius: float, height: float, tol: float = 1e-7) -> ShapeSpec:
    """The solid of a z-axis cylinder."""
    spec = ShapeSpec()
    spec.shell(add_cylinder(spec, origin, radius, height, tol))
    return spec


def add_sphere(spec: ShapeSpec, centre: list[float], radius: float, tol: float = 1e-7) -> list[int]:
    """A sphere face about the z axis (seam at u = 0, degenerated poles): [face]."""
    n, x = axis(2), axis(0)
    south = spec.vertex([centre[0], centre[1], centre[2] - radius], tol)
    north = spec.vertex([centre[0], centre[1], centre[2] + radius], tol)
    low = spec.edge([NO_CURVE], south, south, (0.0, TWO_PI), tol, degenerated=True)
    seam = spec.edge([CIRCLE, *centre, *axis(1, -1.0), *x, radius], south, north, (-HALF_PI, HALF_PI), tol)
    high = spec.edge([NO_CURVE], north, north, (0.0, TWO_PI), tol, degenerated=True)
    face = FaceRecord([SPHERE, *centre, *n, *x, radius])
    face.wire([(low, FORWARD, [PLINE, 0.0, -HALF_PI, 1.0, 0.0]), (seam, FORWARD, [PLINE, TWO_PI, 0.0, 0.0, 1.0]),
               (high, REVERSED, [PLINE, 0.0, HALF_PI, 1.0, 0.0]), (seam, REVERSED, [PLINE, 0.0, 0.0, 0.0, 1.0])])
    return [spec.face(face)]


def sphere(centre: list[float], radius: float, tol: float = 1e-7) -> ShapeSpec:
    """The solid of a sphere."""
    spec = ShapeSpec()
    spec.shell(add_sphere(spec, centre, radius, tol))
    return spec


def torus(centre: list[float], major: float, minor: float) -> ShapeSpec:
    """The solid of a z-axis torus: one face, two seams through one vertex."""
    spec = ShapeSpec()
    n, x = axis(2), axis(0)
    v0 = spec.vertex([centre[0] + major + minor, centre[1], centre[2]])
    ring = spec.edge([CIRCLE, *centre, *n, *x, major + minor], v0, v0, (0.0, TWO_PI))
    meridian_centre = [centre[0] + major, centre[1], centre[2]]
    meridian = spec.edge([CIRCLE, *meridian_centre, *axis(1, -1.0), *x, minor], v0, v0, (0.0, TWO_PI))
    face = FaceRecord([TORUS, *centre, *n, *x, major, minor])
    face.wire([(ring, FORWARD, [PLINE, 0.0, 0.0, 1.0, 0.0]), (meridian, FORWARD, [PLINE, TWO_PI, 0.0, 0.0, 1.0]),
               (ring, REVERSED, [PLINE, 0.0, TWO_PI, 1.0, 0.0]), (meridian, REVERSED, [PLINE, 0.0, 0.0, 0.0, 1.0])])
    spec.shell([spec.face(face)])
    return spec


def sphere_in_box(low: list[float], high: list[float], centre: list[float], radius: float) -> ShapeSpec:
    """A box solid with a spherical cavity (the sphere shell REVERSED)."""
    spec = ShapeSpec()
    spec.shell(add_box(spec, low, high))
    spec.shell(add_sphere(spec, centre, radius), orientation=REVERSED)
    return spec
