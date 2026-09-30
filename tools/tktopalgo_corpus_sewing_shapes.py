"""Shape records of the sewing requests (tools/oracle/tktopalgo_probe_sewing.cxx), spelled
as `tp_shape` records (tools/tktopalgo_corpus_shapes.py).

The faces to sew are loose: each face has its own vertices and edges, moved by an offset
so that the boundaries meet within or beyond a tolerance. The shapes cover gaps below
and above the tolerance, a boundary to cut, three faces on one edge, a face smaller than
the minimal tolerance, a short edge, an open cylinder with its seam split, located
faces, floating edges and closed solids with seams and degenerated edges.
"""
from __future__ import annotations
import math
from tktopalgo_corpus_shapes import (
    CIRCLE, CYLINDER, EDGE, FACE, FORWARD, IDENTITY, LINE, PCIRCLE, PLANE, PLINE, REVERSED, SHELL, TORUS,
    FaceRecord, ShapeSpec, translation)
from tktopalgo_corpus_solids import axis, cross, dot, line_edge, planar_face, sub

TWO_PI = 2.0 * math.pi


def ccw_points(points: list[list[float]], normal: list[float], x_dir: list[float]) -> list[list[float]]:
    """The points sorted counterclockwise about `normal` around their centre."""
    y_dir = cross(normal, x_dir)
    centre = [sum(p[i] for p in points) / len(points) for i in range(3)]
    return sorted(points, key=lambda p: math.atan2(dot(sub(p, centre), y_dir), dot(sub(p, centre), x_dir)))


def polygon_face(spec: ShapeSpec, points: list[list[float]], normal: list[float], x_dir: list[float],
                 offset: tuple[float, float, float] = (0., 0., 0.), tol: float = 1e-7) -> int:
    """A plane face with its own vertices and segment edges through the points (sorted
    counterclockwise about `normal`), moved by `offset`; its index."""
    ordered = ccw_points(points, normal, x_dir)
    corners = [spec.vertex([p[i] + offset[i] for i in range(3)], tol) for p in ordered]
    for i, a in enumerate(corners):
        line_edge(spec, a, corners[(i + 1) % len(corners)], tol)
    return spec.face(planar_face(spec, corners, normal, x_dir, tol))


def compound_of_faces(spec: ShapeSpec, faces: list[int]) -> ShapeSpec:
    """The record whose top items are the faces (a compound of loose faces)."""
    spec.shells = []
    spec.top = [(FACE, f) for f in faces]
    return spec


def box_face_points(size: float, a: int, side: int) -> list[list[float]]:
    """The corners of the face of the box [0, size]^3 normal to axis `a` on `side`."""
    points = []
    for k in range(8):
        if ((k >> a) & 1) == side:
            points.append([size if (k >> i) & 1 else 0.0 for i in range(3)])
    return points


def loose_box(size: float, offsets: list[tuple[float, float, float]], tol: float = 1e-7) -> ShapeSpec:
    """The six faces of the box [0, size]^3, each loose and moved by its offset."""
    spec = ShapeSpec()
    faces = []
    for a in range(3):
        for side in (0, 1):
            normal, x_dir = axis(a, 1.0 if side else -1.0), axis((a + 1) % 3)
            points = box_face_points(size, a, side)
            faces.append(polygon_face(spec, points, normal, x_dir, offsets[len(faces) % len(offsets)], tol))
    return compound_of_faces(spec, faces)


def squares(cells: list[tuple[float, float, float, float, float]], tol: float = 1e-7) -> ShapeSpec:
    """Loose rectangles [u0, u1] x [v0, v1] of the plane z = h: `(u0, v0, u1, v1, h)`."""
    spec = ShapeSpec()
    n, x = axis(2), axis(0)
    faces = []
    for u0, v0, u1, v1, h in cells:
        points = [[u0, v0, h], [u1, v0, h], [u1, v1, h], [u0, v1, h]]
        faces.append(polygon_face(spec, points, n, x, tol=tol))
    return compound_of_faces(spec, faces)


def t_junction(gap: float = 0.0) -> ShapeSpec:
    """Three loose rectangles on the segment [0, 1] of the x axis: two in the plane z = 0
    (y < 0 and y > 0) and one in the plane y = 0 (z > 0), the last moved by `gap` in z."""
    spec = ShapeSpec()
    a = polygon_face(spec, [[0., -1., 0.], [1., -1., 0.], [1., 0., 0.], [0., 0., 0.]], axis(2), axis(0))
    b = polygon_face(spec, [[0., 0., 0.], [1., 0., 0.], [1., 1., 0.], [0., 1., 0.]], axis(2), axis(0))
    c = polygon_face(spec, [[0., 0., gap], [0., 0., 1.], [1., 0., 1.], [1., 0., gap]], axis(1, -1.0), axis(2))
    return compound_of_faces(spec, [a, b, c])


def small_pieces(side: float, short: float) -> ShapeSpec:
    """A unit square, a square of side `side` away from it (smaller than the minimal
    tolerance), and a pentagon whose edge of length `short` touches the square."""
    spec = ShapeSpec()
    n, x = axis(2), axis(0)
    square = polygon_face(spec, [[0., 0., 0.], [1., 0., 0.], [1., 1., 0.], [0., 1., 0.]], n, x)
    tiny = polygon_face(spec, [[5., 5., 0.], [5. + side, 5., 0.], [5. + side, 5. + side, 0.], [5., 5. + side, 0.]], n, x)
    pentagon = polygon_face(spec, [[1., 0., 0.], [2., 0., 0.], [2., 1., 0.], [1., 1., 0.], [1., 1. - short, 0.]], n, x)
    return compound_of_faces(spec, [square, tiny, pentagon])


def open_cylinder(radius: float, height: float, gap: float = 0.0) -> ShapeSpec:
    """A z-axis cylinder whose lateral face has its seam split into two edges, with a
    loose floor and roof (their circles moved up by `gap`)."""
    spec = ShapeSpec()
    n, x = axis(2), axis(0)
    b, t = [radius, 0., 0.], [radius, 0., height]
    vb0, vb1, vt0, vt1 = spec.vertex(b), spec.vertex(b), spec.vertex(t), spec.vertex(t)
    bottom = spec.edge([CIRCLE, 0., 0., 0., *n, *x, radius], vb0, vb1, (0.0, TWO_PI))
    upper = spec.edge([CIRCLE, 0., 0., height, *n, *x, radius], vt0, vt1, (0.0, TWO_PI))
    seam1 = spec.edge([LINE, *b, *n], vb1, vt1, (0.0, height))
    seam0 = spec.edge([LINE, *b, *n], vb0, vt0, (0.0, height))
    lateral = FaceRecord([CYLINDER, 0., 0., 0., *n, *x, radius])
    lateral.wire([(bottom, FORWARD, [PLINE, 0.0, 0.0, 1.0, 0.0]), (seam1, FORWARD, [PLINE, TWO_PI, 0.0, 0.0, 1.0]),
                  (upper, REVERSED, [PLINE, 0.0, height, 1.0, 0.0]), (seam0, REVERSED, [PLINE, 0.0, 0.0, 0.0, 1.0])])
    faces = [spec.face(lateral)]
    for z, normal, sense, use in ((gap, axis(2, -1.0), 0, REVERSED), (height + gap, n, 1, FORWARD)):
        v = spec.vertex([radius, 0., z])
        circle = spec.edge([CIRCLE, 0., 0., z, *n, *x, radius], v, v, (0.0, TWO_PI))
        disc = FaceRecord([PLANE, 0., 0., z, *normal, *x])
        disc.wire([(circle, use, [PCIRCLE, 0.0, 0.0, radius, sense])])
        faces.append(spec.face(disc))
    return compound_of_faces(spec, faces)


def located_squares(dx: float) -> ShapeSpec:
    """A shell of two copies of the unit square, the second located by a translation of
    `dx` along x (a location on the face, not on its geometry)."""
    spec = ShapeSpec()
    f0 = polygon_face(spec, [[0., 0., 0.], [1., 0., 0.], [1., 1., 0.], [0., 1., 0.]], axis(2), axis(0))
    f1 = polygon_face(spec, [[0., 0., 0.], [1., 0., 0.], [1., 1., 0.], [0., 1., 0.]], axis(2), axis(0))
    spec.shells = [(FORWARD, False, [(f0, FORWARD, list(IDENTITY)), (f1, FORWARD, translation(dx, 0., 0.))])]
    spec.top = [(SHELL, 0)]
    return spec


def floating_edges(gap: float) -> ShapeSpec:
    """A compound of three loose segments forming an open chain with gaps of `gap`, and
    a closed triangle of three loose segments."""
    spec = ShapeSpec()
    chain = [[0., 0., 0.], [1., 0., 0.], [1., 1., 0.], [2., 1., 0.]]
    items = []
    for i in range(3):
        a = spec.vertex(chain[i])
        b = spec.vertex([chain[i + 1][0] + gap, chain[i + 1][1], chain[i + 1][2]])
        items.append(line_edge(spec, a, b))
    triangle = [[5., 0., 0.], [6., 0., 0.], [5., 1., 0.]]
    for i in range(3):
        a, b = spec.vertex(triangle[i]), spec.vertex(triangle[(i + 1) % 3])
        items.append(line_edge(spec, a, b))
    spec.top = [(EDGE, e) for e in items]
    return spec


def faces_and_edge(gap: float) -> ShapeSpec:
    """Two loose adjacent squares and a floating segment near their common side."""
    spec = squares([(0., 0., 1., 1., 0.), (1. + gap, 0., 2., 1., 0.)])
    a, b = spec.vertex([1., 0., gap]), spec.vertex([1., 1., gap])
    edge = line_edge(spec, a, b)
    spec.top.append((EDGE, edge))
    return spec


def bezier_bulge(delta: float) -> ShapeSpec:
    """Two loose unit squares side by side whose common side is a segment on the left
    face and a Bezier S-curve bulging by up to 0.866 `delta` on the right one (sampled
    8 times it looks nearer, so the merge reaches a larger tolerance)."""
    spec = squares([(0., 0., 1., 1., 0.)])
    n, x = axis(2), axis(0)
    corners = [spec.vertex(p) for p in ([1., 0., 0.], [2., 0., 0.], [2., 1., 0.], [1., 1., 0.])]
    bottom = line_edge(spec, corners[0], corners[1])
    right = line_edge(spec, corners[1], corners[2])
    top = line_edge(spec, corners[2], corners[3])
    poles = [[1., 0.], [1. + 3. * delta, 1. / 3.], [1. - 3. * delta, 2. / 3.], [1., 1.]]
    bulge = spec.edge([3, 4, *[c for p in poles for c in (*p, 0.)]], corners[0], corners[3], (0.0, 1.0))
    face = FaceRecord([PLANE, 0., 0., 0., *n, *x])
    face.wire([(bottom, FORWARD, [PLINE, 1., 0., 1., 0.]), (right, FORWARD, [PLINE, 2., 0., 0., 1.]),
               (top, FORWARD, [PLINE, 2., 1., -1., 0.]), (bulge, REVERSED, [3, 4, *[c for p in poles for c in p]])])
    spec.top.append((FACE, spec.face(face)))
    return spec


def sliver_junction(width: float, gap: float) -> ShapeSpec:
    """Loose rectangles on the segment [0, 1] of the x axis: one in the plane z = 0 on
    y < 0, a sliver of `width` on y > 0 (both of its long sides near the segment), one in
    the plane y = 0 on z > 0 moved by `gap`, and one on z < 0."""
    spec = ShapeSpec()
    faces = [polygon_face(spec, [[0., -1., 0.], [1., -1., 0.], [1., 0., 0.], [0., 0., 0.]], axis(2), axis(0)),
             polygon_face(spec, [[0., 0., 0.], [1., 0., 0.], [1., width, 0.], [0., width, 0.]], axis(2), axis(0)),
             polygon_face(spec, [[0., 0., gap], [0., 0., 1.], [1., 0., 1.], [1., 0., gap]], axis(1, -1.0), axis(2)),
             polygon_face(spec, [[0., 0., 0.], [0., 0., -1.], [1., 0., -1.], [1., 0., 0.]], axis(1, -1.0), axis(2))]
    return compound_of_faces(spec, faces)


def fin_on_cylinder(radius: float, height: float) -> ShapeSpec:
    """A cylinder solid (its seam shared by its lateral face twice) and a loose plane fin
    in y = 0 along the seam line."""
    from tktopalgo_corpus_solids import add_cylinder
    spec = ShapeSpec()
    faces = add_cylinder(spec, [0., 0., 0.], radius, height)
    fin = polygon_face(spec, [[radius, 0., 0.], [radius + 1., 0., 0.], [radius + 1., 0., height], [radius, 0., height]],
                       axis(1, -1.0), axis(2))
    return compound_of_faces(spec, faces + [fin])


def glued_chain(short: float, order: tuple[int, ...] = (0, 1, 2, 4, 3, 5, 6)) -> ShapeSpec:
    """A unit square with its top-right corner cut by three consecutive short edges,
    listed in the wire in `order` (out of order, so that groups of glued vertices meet
    or grow from either end), and a loose neighbour square."""
    spec = ShapeSpec()
    n, x = axis(2), axis(0)
    points = [[0., 0., 0.], [1., 0., 0.], [1., 1. - 3 * short, 0.], [1., 1. - 2 * short, 0.], [1., 1. - short, 0.],
              [1., 1., 0.], [0., 1., 0.]]
    corners = [spec.vertex(p) for p in points]
    for i, a in enumerate(corners):
        line_edge(spec, a, corners[(i + 1) % len(corners)])
    face = planar_face(spec, corners, n, x)
    orientation, uses = face.wires[0]
    face.wires[0] = (orientation, [uses[i] for i in order])
    faces = [spec.face(face), polygon_face(spec, [[1., 0., 0.], [2., 0., 0.], [2., 1., 0.], [1., 1., 0.]], n, x)]
    return compound_of_faces(spec, faces)


def floating_nodes(eps: float) -> ShapeSpec:
    """Floating edges whose ends glue in chains: segments starting at x = 0, 0.8, 0.5 and
    1.0 times `eps` (in that order), a circle with distinct start and end vertices, and
    a segment shorter than the tolerance."""
    spec = ShapeSpec()
    items = []
    for i, start in enumerate((0.0, 0.8, 0.5, 1.0)):
        a = spec.vertex([start * eps, 0., 0.])
        b = spec.vertex([start * eps + (i - 1.5), 1., 0.])
        items.append(line_edge(spec, a, b))
    n, x = axis(2), axis(0)
    c0, c1 = spec.vertex([7., 0., 0.]), spec.vertex([7., 0., 0.])
    items.append(spec.edge([CIRCLE, 5., 0., 0., *n, *x, 2.0], c0, c1, (0.0, TWO_PI)))
    s0, s1 = spec.vertex([10., 0., 0.]), spec.vertex([10. + 0.3 * eps, 0., 0.])
    items.append(line_edge(spec, s0, s1))
    spec.top = [(EDGE, e) for e in items]
    return spec


def bowed_fin(radius: float, height: float, bulge: float) -> ShapeSpec:
    """A cylinder solid and a loose plane fin in y = 0 whose side on the seam line is a
    Bezier curve bowed by 1.125 `bulge` (longer than the seam, so that the seam is the
    second section of the merge)."""
    from tktopalgo_corpus_solids import add_cylinder
    spec = ShapeSpec()
    faces = add_cylinder(spec, [0., 0., 0.], radius, height)
    p = [[radius, 0., 0.], [radius + 1., 0., 0.], [radius + 1., 0., height], [radius, 0., height]]
    v = [spec.vertex(q) for q in p]
    e0, e1, e2 = line_edge(spec, v[0], v[1]), line_edge(spec, v[1], v[2]), line_edge(spec, v[2], v[3])
    poles3d = [p[0], [radius + 1.5 * bulge, 0., height / 3.], [radius + 1.5 * bulge, 0., 2. * height / 3.], p[3]]
    e3 = spec.edge([3, 4, *[c for q in poles3d for c in q]], v[0], v[3], (0.0, 1.0))
    fin = FaceRecord([PLANE, radius, 0., 0., *axis(1, -1.0), *axis(2)])
    poles2d = [[0., 0.], [height / 3., -1.5 * bulge], [2. * height / 3., -1.5 * bulge], [height, 0.]]
    fin.wire([(e0, FORWARD, [PLINE, 0., 0., 0., -1.]), (e1, FORWARD, [PLINE, 0., -1., 1., 0.]),
              (e2, FORWARD, [PLINE, height, -1., 0., 1.]), (e3, REVERSED, [3, 4, *[c for q in poles2d for c in q]])])
    return compound_of_faces(spec, faces + [spec.face(fin)])


def reversed_pair(gap: float) -> ShapeSpec:
    """Two loose unit squares side by side, the second REVERSED."""
    spec = squares([(0., 0., 1., 1., 0.), (1. + gap, 0., 2., 1., 0.)])
    spec.faces[1].orientation = REVERSED
    return spec


def open_torus(major: float, minor: float) -> ShapeSpec:
    """A z-axis torus face bounded by four separate edges (its two seams split), all
    four vertices at the same point."""
    spec = ShapeSpec()
    n, x = axis(2), axis(0)
    point = [major + minor, 0., 0.]
    v00, v10, v11, v01 = (spec.vertex(point) for _ in range(4))
    ring = [CIRCLE, 0., 0., 0., *n, *x, major + minor]
    meridian = [CIRCLE, major, 0., 0., *axis(1, -1.0), *x, minor]
    ring0 = spec.edge(ring, v00, v10, (0.0, TWO_PI))
    meridian1 = spec.edge(meridian, v10, v11, (0.0, TWO_PI))
    ring1 = spec.edge(ring, v01, v11, (0.0, TWO_PI))
    meridian0 = spec.edge(meridian, v00, v01, (0.0, TWO_PI))
    face = FaceRecord([TORUS, 0., 0., 0., *n, *x, major, minor])
    face.wire([(ring0, FORWARD, [PLINE, 0.0, 0.0, 1.0, 0.0]), (meridian1, FORWARD, [PLINE, TWO_PI, 0.0, 0.0, 1.0]),
               (ring1, REVERSED, [PLINE, 0.0, TWO_PI, 1.0, 0.0]), (meridian0, REVERSED, [PLINE, 0.0, 0.0, 0.0, 1.0])])
    return compound_of_faces(spec, [spec.face(face)])


def split_fin(radius: float, height: float) -> ShapeSpec:
    """A cylinder solid and two loose plane fins in y = 0 along the lower and upper
    halves of its seam line (their middle vertex cuts the seam)."""
    from tktopalgo_corpus_solids import add_cylinder
    spec = ShapeSpec()
    faces = add_cylinder(spec, [0., 0., 0.], radius, height)
    for z0, z1 in ((0., height / 2.), (height / 2., height)):
        faces.append(polygon_face(spec, [[radius, 0., z0], [radius + 1., 0., z0], [radius + 1., 0., z1],
                                         [radius, 0., z1]], axis(1, -1.0), axis(2)))
    return compound_of_faces(spec, faces)


def holed_square(eps: float) -> ShapeSpec:
    """A square [0, 2]^2 with a square hole whose corner is `eps` from the outer corner
    (two wires of one face with nodes within the tolerance), and a loose neighbour."""
    spec = ShapeSpec()
    n, x = axis(2), axis(0)
    outer = [spec.vertex(p) for p in ([0., 0., 0.], [2., 0., 0.], [2., 2., 0.], [0., 2., 0.])]
    for i, a in enumerate(outer):
        line_edge(spec, a, outer[(i + 1) % 4])
    face = planar_face(spec, outer, n, x)
    hole = [spec.vertex(p) for p in ([eps, eps, 0.], [eps, 1., 0.], [1., 1., 0.], [1., eps, 0.])]
    uses = []
    for i, a in enumerate(hole):
        b = hole[(i + 1) % 4]
        edge = line_edge(spec, a, b)
        pa, pb = spec.vertices[a][:3], spec.vertices[b][:3]
        length = math.dist(pa, pb)
        uses.append((edge, FORWARD, [PLINE, pa[0], pa[1], (pb[0] - pa[0]) / length, (pb[1] - pa[1]) / length]))
    face.wire(uses)
    neighbour = polygon_face(spec, [[2., 0., 0.], [3., 0., 0.], [3., 2., 0.], [2., 2., 0.]], n, x)
    return compound_of_faces(spec, [spec.face(face), neighbour])


def tiny_gap(short: float) -> ShapeSpec:
    """A rectangle [0, 2] x [0, 1] whose top side has a short middle edge [1, 1 + short],
    and two loose squares above it on either side of that edge, `short` apart: once the
    squares are sewn, the short edge is a free wire alone."""
    spec = ShapeSpec()
    n, x = axis(2), axis(0)
    points = [[0., 0., 0.], [2., 0., 0.], [2., 1., 0.], [1. + short, 1., 0.], [1., 1., 0.], [0., 1., 0.]]
    corners = [spec.vertex(p) for p in points]
    for i, a in enumerate(corners):
        line_edge(spec, a, corners[(i + 1) % len(corners)])
    faces = [spec.face(planar_face(spec, corners, n, x)),
             polygon_face(spec, [[0., 1., 0.], [1., 1., 0.], [1., 2., 0.], [0., 2., 0.]], n, x),
             polygon_face(spec, [[1. + short, 1., 0.], [2., 1., 0.], [2., 2., 0.], [1. + short, 2., 0.]], n, x)]
    return compound_of_faces(spec, faces)
