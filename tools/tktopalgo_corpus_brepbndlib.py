"""Deterministic requests for BRepBndLib (tools/oracle/tktopalgo_probe_brepbndlib.cxx).

Every request carries its shape record (tktopalgo_corpus_shapes.py), so it never relies
on an earlier request. `bbl_add` runs Add (with and without the meshes), AddClose and
AddOptimal (the four flag pairs) on solids (boxes with tolerances, 3d polygons and
triangulations, located and rotated boxes, hollow and open boxes, cylinders, spheres,
tori), single faces (a meshed face, an infinite plane, a mesh-only face, spherical and
Bezier patches, natural or bounded by iso edges), free edges (lines, circles with and
without a 3d polygon, Bezier and B-spline curves, polygon-only and degenerated edges, an
edge with only a polygon on a triangulation) and free vertices, into an empty box and
into boxes that already hold points.
"""
from __future__ import annotations
import math
from tktopalgo_corpus_shapes import (
    BEZIER, BEZIER_SURFACE, BSPLINE, CIRCLE, EDGE, FACE, FORWARD, LINE, NO_CURVE, NO_SURFACE,
    PLANE, PLINE, REVERSED, SPHERE, VERTEX, FaceRecord, Mesh, ShapeSpec, rotation, translation)
from tktopalgo_corpus_solids import (
    TWO_PI, add_box, box, cross, cylinder, hollow_box, open_box, sphere, sphere_in_box, torus)

# (mode, flag1, flag2): Add(S, B, true/false), AddClose, AddOptimal with its flag pairs.
MODES = [(0, 1, 0), (0, 0, 0), (1, 0, 0), (2, 1, 1), (2, 1, 0), (2, 0, 1), (2, 0, 0)]
# Initial boxes: none, a box around the origin with a gap, a far thin box.
INITIAL_BOXES = [[0], [1, -1.0, -1.0, -1.0, 1.0, 1.0, 1.0, 0.25], [1, 50.0, 50.0, 50.0, 50.0, 60.0, 50.0, 0.0]]


def numbers(values: list[float]) -> str:
    """The fields of a request, with every double spelled exactly."""
    return ' '.join(repr(float(v)) for v in values)


def meshed_box_face() -> ShapeSpec:
    """The top face of a meshed box, alone."""
    spec = box([0.0, 0.0, 0.0], [4.0, 3.0, 2.0], mesh=0.125)
    spec.top = [(FACE, 5)]
    return spec


def mesh_edge_only() -> ShapeSpec:
    """An edge of a meshed box without 3d curve use: its polygon on the triangulation."""
    spec = box([0.0, 0.0, 0.0], [4.0, 3.0, 2.0], mesh=0.0625)
    spec.top = [(EDGE, 3)]
    return spec


def infinite_plane() -> ShapeSpec:
    """A plane face without wires (the whole plane)."""
    spec = ShapeSpec(top=[(FACE, 0)])
    spec.face(FaceRecord([PLANE, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0]))
    return spec


def mesh_only_face() -> ShapeSpec:
    """A face without surface: its triangulation of five nodes."""
    spec = ShapeSpec(top=[(FACE, 0)])
    nodes = [[0.0, 0.0, 0.0], [2.0, 0.0, 0.5], [2.0, 3.0, 1.0], [0.0, 3.0, -0.5], [1.0, 1.5, 2.0]]
    record = FaceRecord([NO_SURFACE], 1e-3)
    record.mesh = Mesh(nodes, [(1, 2, 5), (2, 3, 5), (3, 4, 5), (4, 1, 5)], 0.01)
    spec.face(record)
    return spec


def sphere_patch(u1: float, u2: float, v1: float, v2: float, radius: float = 2.0) -> ShapeSpec:
    """A spherical patch bounded by two meridians and two parallels (not natural)."""
    spec = ShapeSpec(top=[(FACE, 0)])
    corners = [(u1, v1), (u2, v1), (u2, v2), (u1, v2)]
    ids = [spec.vertex(sphere_point(radius, u, v)) for u, v in corners]
    low = spec.edge(parallel(radius, v1), ids[0], ids[1], (u1, u2))
    east = spec.edge(meridian(radius, u2), ids[1], ids[2], (v1, v2))
    high = spec.edge(parallel(radius, v2), ids[3], ids[2], (u1, u2))
    west = spec.edge(meridian(radius, u1), ids[0], ids[3], (v1, v2))
    face = FaceRecord([SPHERE, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, radius])
    face.wire([(low, FORWARD, [PLINE, 0.0, v1, 1.0, 0.0]), (east, FORWARD, [PLINE, u2, 0.0, 0.0, 1.0]),
               (high, REVERSED, [PLINE, 0.0, v2, 1.0, 0.0]), (west, REVERSED, [PLINE, u1, 0.0, 0.0, 1.0])])
    spec.face(face)
    return spec


def sphere_point(radius: float, u: float, v: float) -> list[float]:
    return [radius * math.cos(v) * math.cos(u), radius * math.cos(v) * math.sin(u), radius * math.sin(v)]


def parallel(radius: float, v: float) -> list[float]:
    """The circle of the z-axis sphere at latitude v, parametrized by u."""
    return [CIRCLE, 0.0, 0.0, radius * math.sin(v), 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, radius * math.cos(v)]


def meridian(radius: float, u: float) -> list[float]:
    """The circle of the z-axis sphere at longitude u, parametrized by v."""
    x_dir = [math.cos(u), math.sin(u), 0.0]
    return [CIRCLE, 0.0, 0.0, 0.0, *cross(x_dir, [0.0, 0.0, 1.0]), *x_dir, radius]


def bezier_patch(natural: bool) -> ShapeSpec:
    """A 3 x 3 Bezier patch, flagged natural, or bounded by its four boundary pcurves."""
    spec = ShapeSpec(top=[(FACE, 0)])
    poles = [[0.0, 0.0, 0.0], [0.0, 1.0, 1.0], [0.0, 2.0, 0.0],
             [1.0, 0.0, 1.0], [1.0, 1.0, 3.0], [1.0, 2.0, 1.0],
             [2.0, 0.0, 0.0], [2.0, 1.0, -1.0], [2.0, 2.0, 0.5]]
    face = FaceRecord([BEZIER_SURFACE, 3, 3, *[c for p in poles for c in p]], 1e-7, natural)
    if not natural:
        ids = [spec.vertex(p) for p in (poles[0], poles[6], poles[8], poles[2])]
        rows = [[poles[0], poles[3], poles[6]], [poles[6], poles[7], poles[8]],
                [poles[2], poles[5], poles[8]], [poles[0], poles[1], poles[2]]]
        e = [spec.edge([BEZIER, 3, *[c for p in row for c in p]], a, b, (0.0, 1.0))
             for row, (a, b) in zip(rows, [(0, 1), (1, 2), (3, 2), (0, 3)])]
        face.wire([(e[0], FORWARD, [PLINE, 0.0, 0.0, 1.0, 0.0]), (e[1], FORWARD, [PLINE, 1.0, 0.0, 0.0, 1.0]),
                   (e[2], REVERSED, [PLINE, 0.0, 1.0, 1.0, 0.0]), (e[3], REVERSED, [PLINE, 0.0, 0.0, 0.0, 1.0])])
    spec.face(face)
    return spec


def circle_polygon_edge() -> tuple[list[float], list[list[float]]]:
    """The edge of OCC30292: the unit circle with a 26-segment 3d polygon of deflection 0.1."""
    nodes = [[math.cos(TWO_PI * k / 26), math.sin(TWO_PI * k / 26), 0.0] for k in range(27)]
    return [CIRCLE, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0], nodes


def free_edges() -> ShapeSpec:
    """A compound of free edges of every kind and two free vertices."""
    spec = ShapeSpec()
    a, b = spec.vertex([0.0, 0.0, 0.0]), spec.vertex([3.0, 4.0, 0.0], 1e-3)
    spec.edge([LINE, 0.0, 0.0, 0.0, 0.6, 0.8, 0.0], a, b, (0.0, 5.0))
    circle, nodes = circle_polygon_edge()
    spec.edge(circle, -1, -1, (0.0, TWO_PI))
    spec.edge([*circle[:10], 2.0], -1, -1, (0.0, TWO_PI), polygon=(nodes, 0.1))
    spec.edge([BEZIER, 4, 0.0, 0.0, 1.0, 1.0, 2.0, 1.0, 2.0, -1.0, 1.0, 3.0, 0.0, 2.0], -1, -1, (0.0, 1.0), 1e-4)
    spec.edge([BSPLINE, 3, 4, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 2, 0.0, 4, 1.0, 4],
              -1, -1, (0.0, 1.0))
    spec.edge([NO_CURVE], -1, -1, (0.0, 1.0), polygon=([[5.0, 5.0, 5.0], [6.0, 5.0, 7.0], [4.0, 8.0, 5.0]], 0.02))
    spec.edge([NO_CURVE], a, a, (0.0, 1.0), degenerated=True)
    c, d = spec.vertex([-2.0, 7.0, 1.0], 0.5), spec.vertex([9.0, -1.0, -3.0])
    spec.top = [(EDGE, i) for i in range(7)] + [(VERTEX, c), (VERTEX, d)]
    return spec


def single(kind: int, record: ShapeSpec, index: int = 0) -> ShapeSpec:
    """`record` with one top-level item."""
    record.top = [(kind, index)]
    return record


def empty_solid() -> ShapeSpec:
    """A solid without shells."""
    return ShapeSpec()


def located(spec: ShapeSpec, location: list[float]) -> ShapeSpec:
    """`spec` moved by `location`."""
    spec.location = location
    return spec


def face_located_box() -> ShapeSpec:
    """A meshed box whose top face is placed in its shell with a translation."""
    spec = ShapeSpec()
    faces = add_box(spec, [0.0, 0.0, 0.0], [2.0, 2.0, 2.0], mesh=0.01, polygon=0.02)
    orientation, closed, uses = FORWARD, False, [(f, FORWARD, [0]) for f in faces]
    uses[5] = (faces[5], FORWARD, translation(0.5, -0.25, 1.0))
    spec.shells.append((orientation, closed, uses))
    return spec


def bnd_shapes() -> list[ShapeSpec]:
    """Every shape of the BRepBndLib corpus."""
    return [
        box([0.0, 0.0, 0.0], [10.0, 10.0, 10.0]),
        box([-1.0, 2.0, -3.0], [4.0, 3.5, 7.0], tol=0.01),
        box([0.0, 0.0, 0.0], [1.0, 2.0, 3.0], polygon=0.05),
        box([0.0, 0.0, 0.0], [1.0, 2.0, 3.0], mesh=0.02),
        box([0.0, 0.0, 0.0], [1.0, 2.0, 3.0], polygon=0.05, mesh=0.02),
        located(box([0.0, 0.0, 0.0], [1.0, 2.0, 3.0], mesh=0.02), translation(10.0, -5.0, 2.5)),
        located(box([0.0, 0.0, 0.0], [1.0, 2.0, 3.0], polygon=0.05), rotation([0.0, 0.0, 0.0], [1.0, 1.0, 0.0], 0.7)),
        located(box([0.0, 0.0, 0.0], [1.0, 2.0, 3.0]), rotation([1.0, 0.0, 0.0], [0.0, 0.0, 1.0], 0.3)),
        face_located_box(),
        hollow_box([0.0, 0.0, 0.0], [10.0, 10.0, 10.0], [3.0, 3.0, 3.0], [7.0, 7.0, 7.0]),
        open_box([0.0, 0.0, 0.0], [10.0, 10.0, 10.0]),
        cylinder([1.0, 2.0, 3.0], 2.0, 5.0),
        located(cylinder([0.0, 0.0, 0.0], 1.0, 1.0), rotation([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.5)),
        sphere([0.0, 0.0, 0.0], 10.0),
        sphere([1.0, -2.0, 0.5], 0.75),
        torus([0.0, 0.0, 0.0], 5.0, 1.0),
        sphere_in_box([0.0, 0.0, 0.0], [10.0, 10.0, 10.0], [5.0, 5.0, 5.0], 2.0),
        empty_solid(),
        meshed_box_face(),
        mesh_edge_only(),
        infinite_plane(),
        mesh_only_face(),
        sphere_patch(0.2, 1.2, -0.3, 0.6),
        sphere_patch(-2.5, 2.5, -1.2, 1.4),
        bezier_patch(True),
        bezier_patch(False),
        single(FACE, cylinder([0.0, 0.0, 0.0], 3.0, 2.0), 0),
        single(FACE, sphere([0.0, 0.0, 0.0], 4.0), 0),
        single(FACE, torus([0.0, 0.0, 0.0], 5.0, 1.5), 0),
        free_edges(),
        single(EDGE, free_edges(), 2),
        single(VERTEX, free_edges(), 2),
    ]


def append_brepbndlib_requests(requests: list[str]) -> None:
    """Append every `bbl_add` request: each shape in each mode, then each solid of the
    first five into the non-empty initial boxes."""
    shapes = bnd_shapes()
    for spec in shapes:
        record = spec.numbers()
        for mode, f1, f2 in MODES:
            requests.append('bbl_add ' + numbers([*record, *INITIAL_BOXES[0], mode, f1, f2]))
    for spec in shapes[:5] + shapes[20:22]:
        record = spec.numbers()
        for initial in INITIAL_BOXES[1:]:
            for mode, f1, f2 in MODES[::3]:
                requests.append('bbl_add ' + numbers([*record, *initial, mode, f1, f2]))
