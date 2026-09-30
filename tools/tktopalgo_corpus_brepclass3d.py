"""Deterministic requests for BRepClass3d (tools/oracle/tktopalgo_probe_brepclass3d.cxx).

Every request carries its shape record (tktopalgo_corpus_shapes.py, named shapes of
tktopalgo_corpus_solids.py), so it never relies on an earlier request:
- `bc3_classify`: BRepClass3d_SolidClassifier on points inside, outside, on and near
  the faces, edges and vertices (and seeded points) of boxes (with small and large
  tolerances, located, rotated, with a flipped face), cylinders, spheres, a torus,
  hollow boxes, a box with a spherical cavity, an open box, a solid without shells, a
  half-space bounded by an infinite plane and compounds of solids, with the Load,
  constructor and infinite-point variants, then Destroy;
- `bc3_infinite`: PerformInfinitePoint (SolidClassifier and SClassifier twice) and
  BRepClass3d::OuterShell, with shells listed outer or inner first and an INTERNAL shell;
- `bc3_sclassifier`: the SClassifier constructor on one explorer, point after point;
- `bc3_segment`: Segment and OtherSegment in turn (the explorer's rays, its face index
  and the restarts with the next edge parameter);
- `bc3_explorer`, `bc3_findpoint`, `bc3_pointinface`, `bc3_uvpoint`: the explorer
  members, the FindAPointInTheFace and PointInTheFace overloads and ClassifyUVPoint;
- `bc3_tree`: the point and line selectors of the UB-tree (points and lines through
  vertices, along and across edges), with out-of-range result indices;
- `bc3_inter3d`: BRepClass3d_Intersector3d on lines across faces, FORWARD or REVERSED.
"""
from __future__ import annotations
import math
import random
from tktopalgo_corpus_brepbndlib import numbers
from tktopalgo_corpus_shapes import (
    BSPLINE_SURFACE, FORWARD, INTERNAL, PLANE, REVERSED, SHELL, FaceRecord, ShapeSpec, rotation,
    translation)
from tktopalgo_corpus_solids import (
    add_box, add_cylinder, box, ccw, cross, cylinder, hollow_box, line_edge, open_box, planar_face,
    sphere, sphere_in_box, torus)

TOLERANCES = [1e-7, 1e-3, 0.5]


def flipped_face_box() -> ShapeSpec:
    """A unit-10 box whose top face is REVERSED (an inside-out face)."""
    spec = ShapeSpec()
    faces = add_box(spec, [0.0, 0.0, 0.0], [10.0, 10.0, 10.0])
    spec.faces[faces[5]].orientation = REVERSED
    spec.shell(faces)
    return spec


def half_space() -> ShapeSpec:
    """A solid bounded by the whole plane z = 0 (no wire), its matter below."""
    spec = ShapeSpec()
    spec.shell([spec.face(FaceRecord([PLANE, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0]))], closed=False)
    return spec


def two_boxes() -> ShapeSpec:
    """A compound of two disjoint box solids (the explorer sees both shells)."""
    spec = ShapeSpec()
    spec.shell(add_box(spec, [0.0, 0.0, 0.0], [2.0, 2.0, 2.0]))
    spec.shell(add_box(spec, [5.0, 0.0, 0.0], [7.0, 2.0, 2.0]))
    spec.top = [(SHELL, 0), (SHELL, 1)]
    return spec


def inner_first() -> ShapeSpec:
    """A hollow box whose cavity shell is listed before the outer shell."""
    spec = ShapeSpec()
    cavity = add_box(spec, [3.0, 3.0, 3.0], [7.0, 7.0, 7.0])
    outer = add_box(spec, [0.0, 0.0, 0.0], [10.0, 10.0, 10.0])
    spec.shell(cavity, orientation=REVERSED)
    spec.shell(outer)
    return spec


def internal_shell_first() -> ShapeSpec:
    """A box solid with a first shell whose first face is INTERNAL, then the box."""
    spec = ShapeSpec()
    inner = add_box(spec, [3.0, 3.0, 3.0], [7.0, 7.0, 7.0])
    outer = add_box(spec, [0.0, 0.0, 0.0], [10.0, 10.0, 10.0])
    spec.shell(inner, face_orientation=INTERNAL)
    spec.shell(outer)
    return spec


def cylinder_in_box() -> ShapeSpec:
    """A box solid with a cylindrical cavity."""
    spec = ShapeSpec()
    spec.shell(add_box(spec, [0.0, 0.0, 0.0], [10.0, 10.0, 10.0]))
    spec.shell(add_cylinder(spec, [5.0, 5.0, 2.0], 2.0, 6.0), orientation=REVERSED)
    return spec


def flat_box() -> ShapeSpec:
    """A box 1e-10 thick: its side faces have degenerate UV bounds."""
    return box([0.0, 0.0, 0.0], [10.0, 10.0, 1e-10])


def spline_top_box() -> ShapeSpec:
    """The box [0, 10]^3 whose top face is a planar degree 1 B-spline patch with the
    parametrization of the plane (so the edge pcurves stay the same)."""
    spec = box([0.0, 0.0, 0.0], [10.0, 10.0, 10.0])
    face = spec.faces[5]
    origin, normal, x_dir = face.surface[1:4], face.surface[4:7], face.surface[7:10]
    y_dir = cross(normal, x_dir)
    corners = {v for _, uses in face.wires for e, _, _ in uses for v in spec.edges[e][7:9]}
    uv = [(sum((spec.vertices[c][i] - origin[i]) * x_dir[i] for i in range(3)),
           sum((spec.vertices[c][i] - origin[i]) * y_dir[i] for i in range(3))) for c in corners]
    (u0, u1), (v0, v1) = (min(p[0] for p in uv), max(p[0] for p in uv)), (min(p[1] for p in uv), max(p[1] for p in uv))
    poles = [[origin[i] + u * x_dir[i] + v * y_dir[i] for i in range(3)] for u in (u0, u1) for v in (v0, v1)]
    face.surface = [BSPLINE_SURFACE, 1, 1, 2, 2, *[c for p in poles for c in p], 2, u0, 2, u1, 2, 2, v0, 2, v1, 2]
    return spec


def square_sheet() -> ShapeSpec:
    """A solid of one open shell holding the square [0, 10]^2 at z = 0."""
    spec = ShapeSpec()
    corners = [spec.vertex(p) for p in ([0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [10.0, 10.0, 0.0], [0.0, 10.0, 0.0])]
    for i in range(4):
        line_edge(spec, corners[i], corners[(i + 1) % 4])
    normal, x_dir = [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]
    spec.shell([spec.face(planar_face(spec, ccw(spec, corners, normal, x_dir), normal, x_dir))], closed=False)
    return spec


def two_cavities() -> ShapeSpec:
    """A solid of two REVERSED box shells (no outer shell)."""
    spec = ShapeSpec()
    spec.shell(add_box(spec, [0.0, 0.0, 0.0], [2.0, 2.0, 2.0]), orientation=REVERSED)
    spec.shell(add_box(spec, [5.0, 0.0, 0.0], [7.0, 2.0, 2.0]), orientation=REVERSED)
    return spec


def box_with_spare_shell() -> ShapeSpec:
    """The shell of a box, with the faces of a second box that it does not hold."""
    spec = ShapeSpec()
    spec.shell(add_box(spec, [0.0, 0.0, 0.0], [10.0, 10.0, 10.0]))
    spec.shell(add_box(spec, [20.0, 0.0, 0.0], [22.0, 2.0, 2.0]))
    spec.top = [(SHELL, 0)]
    return spec


def with_location(spec: ShapeSpec, location: list[float]) -> ShapeSpec:
    spec.location = location
    return spec


def box_points(low: float, high: float) -> list[list[float]]:
    """Points inside, outside, on and near the faces, edges and vertices of a cube."""
    mid, span = (low + high) / 2.0, high - low
    points = [[mid, mid, mid], [low + 0.1 * span, low + 0.2 * span, low + 0.7 * span],
              [low - 1.0, mid, mid], [high + 5.0, high + 5.0, high + 5.0], [mid, mid, low - 1e-3],
              [low, mid, mid], [high, 0.3 * span + low, 0.4 * span + low], [mid, mid, high],
              [low + 1e-8, mid, mid], [low - 1e-6, mid, mid], [low + 1e-6, mid, mid],
              [low, low, mid], [high, mid, low], [low, low, low], [high, high, high],
              [mid, low + 0.25 * span, low + 0.25 * span], [low + 0.5 * span, high - 1e-9, mid]]
    return points


def seeded_points(rng: random.Random, low: float, high: float, count: int) -> list[list[float]]:
    return [[rng.uniform(low, high) for _ in range(3)] for _ in range(count)]


def classify_shapes() -> list[tuple[ShapeSpec, list[list[float]]]]:
    """Each shape with the points to classify on it."""
    rng = random.Random(3031)
    cube = box_points(0.0, 10.0)
    on_planes = [[0.0, 15.0, 5.0], [0.0, -3.0, -3.0], [0.0, 12.0, 12.0], [10.0, 20.0, 5.0]]
    return [
        (box([0.0, 0.0, 0.0], [10.0, 10.0, 10.0]), cube + on_planes + seeded_points(rng, -2.0, 12.0, 10)),
        (box([0.0, 0.0, 0.0], [10.0, 10.0, 10.0], tol=1e-3), cube + seeded_points(rng, -5.0, 15.0, 20)),
        (box([0.0, 0.0, 0.0], [10.0, 10.0, 10.0], tol=0.3), seeded_points(rng, -5.0, 15.0, 40)),
        (cylinder([1.0, 2.0, 3.0], 2.0, 5.0, tol=0.2), seeded_points(rng, -3.0, 10.0, 20)),
        (sphere([0.0, 0.0, 0.0], 10.0, tol=0.2), seeded_points(rng, -12.0, 12.0, 20)),
        (flat_box(), [[5.0, 5.0, 5.0], [5.0, 5.0, 0.0], [5.0, 5.0, -1.0], [20.0, 5.0, 0.0]]),
        (spline_top_box(), cube + seeded_points(rng, -2.0, 12.0, 10)),
        (square_sheet(), [[5.0, 5.0, 1.0], [5.0, 5.0, -1.0], [5.0, 5.0, 0.0], [15.0, 5.0, 0.0]]),
        (two_cavities(), [[1.0, 1.0, 1.0], [3.0, 1.0, 1.0], [6.0, 1.0, 1.0]]),
        (box_with_spare_shell(), [[5.0, 5.0, 5.0], [21.0, 1.0, 1.0]]),
        (with_location(box([0.0, 0.0, 0.0], [10.0, 10.0, 10.0]), translation(1.0, 2.0, 3.0)),
         [[p[0] + 1.0, p[1] + 2.0, p[2] + 3.0] for p in cube]),
        (with_location(box([0.0, 0.0, 0.0], [10.0, 10.0, 10.0]), rotation([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 0.5)),
         cube + seeded_points(rng, -12.0, 12.0, 8)),
        (flipped_face_box(), cube),
        (cylinder([1.0, 2.0, 3.0], 2.0, 5.0),
         [[1.0, 2.0, 5.5], [2.5, 2.0, 4.0], [3.0, 2.0, 5.0], [3.0 + 1e-8, 2.0, 5.0], [1.0, 4.0, 3.0],
          [1.0, 2.0, 3.0], [1.0, 2.0, 8.0], [1.0, 2.0, 8.5], [4.0, 2.0, 5.0], [1.0, 0.0, 6.0],
          [-1.0, 2.0, 3.0], [1.0 + math.sqrt(2.0), 2.0 + math.sqrt(2.0), 4.0]] + seeded_points(rng, -2.0, 9.0, 8)),
        (sphere([0.0, 0.0, 0.0], 10.0),
         [[0.0, 0.0, 0.0], [0.0, 0.0, 10.0], [0.0, 0.0, -10.0], [0.0, 0.0, 9.999], [10.0, 0.0, 0.0],
          [6.0, 6.0, 6.0], [5.0, 5.0, 5.0], [0.0, 10.0 + 1e-8, 0.0], [-3.0, 2.0, -9.0], [0.0, 0.0, 12.0]]
         + seeded_points(rng, -12.0, 12.0, 8)),
        (torus([0.0, 0.0, 0.0], 5.0, 1.0),
         [[5.0, 0.0, 0.0], [0.0, 0.0, 0.0], [6.0, 0.0, 0.0], [4.0, 0.0, 0.0], [5.0, 0.0, 1.0],
          [0.0, 5.0, 0.5], [-5.5, 0.0, 0.2], [3.0, 3.0, 0.0], [0.0, 0.0, 3.0]]),
        (hollow_box([0.0, 0.0, 0.0], [10.0, 10.0, 10.0], [3.0, 3.0, 3.0], [7.0, 7.0, 7.0]),
         [[5.0, 5.0, 5.0], [1.0, 1.0, 1.0], [3.0, 5.0, 5.0], [2.0, 5.0, 5.0], [8.0, 8.0, 5.0], [11.0, 5.0, 5.0],
          [3.0, 3.0, 3.0], [6.5, 3.2, 4.1]]),
        (inner_first(), [[5.0, 5.0, 5.0], [1.0, 1.0, 1.0], [11.0, 5.0, 5.0]]),
        (sphere_in_box([0.0, 0.0, 0.0], [10.0, 10.0, 10.0], [5.0, 5.0, 5.0], 2.0),
         [[5.0, 5.0, 5.0], [1.0, 1.0, 1.0], [7.0, 5.0, 5.0], [5.0, 5.0, 3.0], [5.0, 5.0, 2.5], [8.0, 8.0, 8.0]]),
        (cylinder_in_box(), [[5.0, 5.0, 5.0], [5.0, 5.0, 1.0], [1.0, 1.0, 1.0], [7.0, 5.0, 5.0], [5.0, 5.0, 9.0]]),
        (open_box([0.0, 0.0, 0.0], [10.0, 10.0, 10.0]), [[5.0, 5.0, 5.0], [5.0, 5.0, 11.0], [5.0, 5.0, 10.0], [-1.0, 5.0, 5.0]]),
        (ShapeSpec(), [[0.0, 0.0, 0.0], [1.0, 2.0, 3.0]]),
        (half_space(), [[0.0, 0.0, -1.0], [3.0, 4.0, 1.0], [3.0, 4.0, 0.0], [1e5, -1e5, -2.0]]),
        (two_boxes(), [[1.0, 1.0, 1.0], [6.0, 1.0, 1.0], [3.5, 1.0, 1.0], [2.0, 1.0, 1.0]]),
    ]


def append_classify(requests: list[str]) -> None:
    """`bc3_classify`: every shape, tolerance and mode (modes 2 to 4 on the first tolerance)."""
    for spec, points in classify_shapes():
        record = spec.numbers()
        flat = [c for p in points for c in p]
        for tol in TOLERANCES:
            for mode in (0, 1):
                requests.append('bc3_classify ' + numbers([*record, mode, tol, len(points), *flat]))
        for mode in (2, 3, 4):
            requests.append('bc3_classify ' + numbers([*record, mode, 1e-7, min(4, len(points)), *flat[:12]]))


def append_infinite(requests: list[str]) -> None:
    """`bc3_infinite` on every classified shape and the shell-order variants."""
    shapes = [spec for spec, _ in classify_shapes()] + [internal_shell_first()]
    for spec in shapes:
        record = spec.numbers()
        for mode in (0, 1):
            requests.append('bc3_infinite ' + numbers([*record, 1e-7, mode]))


def append_sclassifier(requests: list[str]) -> None:
    """`bc3_sclassifier`: point after point on one explorer, per shape."""
    for spec, points in classify_shapes():
        flat = [c for p in points for c in p]
        requests.append('bc3_sclassifier ' + numbers([*spec.numbers(), 1e-7, len(points), *flat]))


def append_segment(requests: list[str]) -> None:
    """`bc3_segment`: Segment then OtherSegment 12 times, from several points."""
    for spec, points in classify_shapes():
        record = spec.numbers()
        for point in points[:6]:
            requests.append('bc3_segment ' + numbers([*record, *point, 12]))


def face_shapes() -> list[ShapeSpec]:
    """The shapes whose faces are probed one by one."""
    return [box([0.0, 0.0, 0.0], [10.0, 10.0, 10.0]), cylinder([1.0, 2.0, 3.0], 2.0, 5.0),
            sphere([0.0, 0.0, 0.0], 10.0), torus([0.0, 0.0, 0.0], 5.0, 1.0), half_space(),
            flipped_face_box(), spline_top_box(), box_with_spare_shell(), square_sheet()]


def append_faces(requests: list[str]) -> None:
    """`bc3_explorer`, `bc3_findpoint`, `bc3_pointinface` and `bc3_uvpoint`."""
    for spec, _ in classify_shapes():
        requests.append('bc3_explorer ' + numbers(spec.numbers()))
    for spec in face_shapes():
        record = spec.numbers()
        for face in range(len(spec.faces)):
            for param in (0.1, 0.5, 0.9, 0.0, 1.0):
                requests.append('bc3_findpoint ' + numbers([*record, face, param]))
            for u, v, index, lift in [(0.5, 0.5, 0, 0.0), (0.5, 0.5, 1, 1.0), (2.0, 3.0, 5, 0.0), (-50.0, 1.0, 0, 0.0),
                                      (1.0, 1.0, 40, 2.0), (0.1, 0.2, 300, 0.0), (0.5, 0.5, 2000, 0.0)]:
                requests.append('bc3_pointinface ' + numbers([*record, face, u, v, index, 4, lift]))
            uvs = [0.5, 0.5, 0.0, 0.0, 1.0, 0.0, 5.0, 5.0, 10.0, 3.0, -1.0, 2.0, 3.14159, 0.25, 6.2, 0.7, 20.0, 20.0]
            requests.append('bc3_uvpoint ' + numbers([*record, face, len(uvs) // 2, *uvs]))


def append_tree(requests: list[str]) -> None:
    """`bc3_tree`: the point selector on points near edges and vertices, the line
    selector on lines through vertices, along and across edges."""
    cube = box([0.0, 0.0, 0.0], [10.0, 10.0, 10.0], tol=1e-3)
    points = [[0.0, 0.0, 0.0], [0.0, 0.0, 5.0], [5e-4, 0.0, 5.0], [2e-3, 0.0, 5.0], [5.0, 5.0, 5.0], [10.0, 10.0, 10.0 + 1e-4],
              [20.0, 0.0, 0.0]]
    lines = [[-1.0, -1.0, -1.0, 1.0, 1.0, 1.0, 20.0], [0.0, 0.0, -2.0, 0.0, 0.0, 1.0, 20.0],
             [-5.0, 0.0, 5.0, 1.0, 0.0, 0.0, 30.0], [5.0, 5.0, 5.0, 1.0, 0.0, 0.0, 10.0],
             [5.0, -5.0, 10.0, 0.0, 1.0, 0.0, 30.0], [5.0, 5.0, 5.0, 0.0, 1.0, 1.0, 3.0],
             [5.0, 5.0, 5.0, 1.0, 1.0, 1.0, -1.0], [3.0, 0.0, -1.0, 0.0, 0.0, 1.0, 0.5]]
    for spec in (cube, cylinder([1.0, 2.0, 3.0], 2.0, 5.0), sphere([0.0, 0.0, 0.0], 10.0)):
        record = spec.numbers()
        for point in points:
            requests.append('bc3_tree ' + numbers([*record, 0, *point]))
        for line in lines:
            requests.append('bc3_tree ' + numbers([*record, 1, *line]))


def append_inter3d(requests: list[str]) -> None:
    """`bc3_inter3d`: groups of lines across the faces of the face shapes."""
    lines = [[5.0, 5.0, -5.0, 0.0, 0.0, 1.0, 1.0, 1e-7], [5.0, 5.0, 20.0, 0.0, 0.0, -1.0, 1.0, 1e-7],
             [0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1e-7], [-20.0, 1.0, 1.0, 1.0, 0.0, 0.0, 5.0, 0.5],
             [1.0, 2.0, 4.0, 1.0, 0.0, 0.0, 1.0, 1e-7], [5.0, 5.0, 10.0, 1.0, 0.0, 0.0, 1.0, 1e-3],
             [3.0, 2.0, 3.0, 0.0, 0.0, 1.0, 1.0, 1e-7], [0.0, 0.0, 0.0, 0.0, 0.6, 0.8, 1.0, 20.0]]
    for spec in face_shapes():
        record = spec.numbers()
        for face in range(len(spec.faces)):
            requests.append('bc3_inter3d ' + numbers([*record, face, len(lines), *[c for line in lines for c in line]]))


def append_brepclass3d_requests(requests: list[str]) -> None:
    """Append every BRepClass3d request family in a fixed order."""
    append_classify(requests)
    append_infinite(requests)
    append_sclassifier(requests)
    append_segment(requests)
    append_faces(requests)
    append_tree(requests)
    append_inter3d(requests)
