"""Requests for ShapeUpgrade_UnifySameDomain (tools/oracle/tkshhealing_probe_unify.cxx).

`sh_unify` builds its input from sewn planar polygons (coplanar pairs and grids, rings
around a hole, L shapes, T junctions, triangles, nearly coplanar faces and split sides of
an open box), from sewn lateral faces of cylinders, cones and spheres split in angular
pieces (full turns that need a seam rebuilt, partial ones), from wires of segments, arcs
and Bezier curves, and from a box. Each input runs under the default settings and under
variations of Initialize, the unifications, the B-spline concatenation, internal edges,
the safe input mode, the tolerances and kept edges or vertices; the result digest and the
history of every input sub-shape are compared.
"""
from __future__ import annotations
import math

from tktopalgo_corpus_breplib_records import numbers

PI = math.pi
DEFAULT = (0, 1, 1, 0, 0, 1, 1e-7, 1e-12, [])
VARIANTS = [
    DEFAULT,
    (1, 1, 1, 0, 0, 1, 1e-7, 1e-12, []),
    (0, 1, 0, 0, 0, 1, 1e-7, 1e-12, []),
    (0, 0, 1, 0, 0, 1, 1e-7, 1e-12, []),
    (0, 1, 1, 1, 0, 1, 1e-7, 1e-12, []),
    (0, 1, 1, 0, 1, 1, 1e-7, 1e-12, []),
    (0, 1, 1, 0, 0, 0, 1e-7, 1e-12, []),
    (0, 1, 1, 0, 0, 1, 1e-4, 1e-2, []),
    (0, 1, 1, 0, 0, 1, 1e-7, 1e-12, [(0, 1), (1, 2)]),
    (0, 1, 1, 0, 1, 1, 1e-7, 1e-12, [(0, 3)]),
]


def square(x: float, y: float, size: float = 1.0, z: float = 0.0) -> list[tuple[float, float, float]]:
    return [(x, y, z), (x + size, y, z), (x + size, y + size, z), (x, y + size, z)]


def polygons(faces: list[list[tuple[float, float, float]]], tol: float = 1e-6) -> list[float]:
    fields: list[float] = [0, len(faces)]
    for face in faces:
        fields += [len(face)] + [c for p in face for c in p]
    return fields + [tol]


def polygon_inputs() -> list[list[float]]:
    grid = [square(float(i), float(j)) for i in range(3) for j in range(3)]
    ring = [f for f in grid if f[0][:2] != (1.0, 1.0)]
    side = [(0, 0, 0), (1, 0, 0), (1, 0, 1), (0, 0, 1)]
    wall = [(1, 0, 0), (1, 1, 0), (1, 1, 1), (1, 0, 1)]
    return [
        polygons([square(0, 0), square(1, 0)]),
        polygons(grid),
        polygons(ring),
        polygons([square(0, 0), square(1, 0), side, wall]),
        polygons([square(0, 0, 2.0), square(2, 0), square(2, 1)]),
        polygons([[(0, 0, 0), (1, 0, 0), (1, 1, 0)], [(0, 0, 0), (1, 1, 0), (0, 1, 0)]]),
        polygons([square(0, 0), square(1, 0, 1.0, 1e-8)]),
        polygons([square(0, 0), square(1, 0, 1.0, 5e-5)], 1e-4),
        polygons(open_box_split()),
    ]


def open_box_split() -> list[list[tuple[float, float, float]]]:
    """An open box whose bottom and two sides are each split into two coplanar faces."""
    return [
        square(0, 0, 1.0), [(1, 0, 0), (2, 0, 0), (2, 1, 0), (1, 1, 0)],
        [(0, 0, 0), (1, 0, 0), (1, 0, 1), (0, 0, 1)], [(1, 0, 0), (2, 0, 0), (2, 0, 1), (1, 0, 1)],
        [(0, 1, 0), (0, 1, 1), (1, 1, 1), (1, 1, 0)], [(1, 1, 0), (1, 1, 1), (2, 1, 1), (2, 1, 0)],
        [(0, 0, 0), (0, 0, 1), (0, 1, 1), (0, 1, 0)], [(2, 0, 0), (2, 1, 0), (2, 1, 1), (2, 0, 1)],
    ]


def revolved_inputs() -> list[list[float]]:
    pieces = [
        (0, PI / 2, 4, [1.0, 2.0]), (0, PI / 3, 3, [1.0, 2.0]), (0, 3 * PI / 4, 2, [1.5, 1.0]),
        (0, PI / 3, 6, [1.0, 1.0]), (1, PI / 2, 4, [1.0, 0.5, 2.0]), (1, PI / 2, 2, [1.0, 0.5, 2.0]),
        (1, PI / 2, 4, [1.0, 0.0, 2.0]), (2, PI / 2, 4, [1.0]), (2, 2 * PI / 3, 2, [1.0]),
        (2, 2 * PI / 3, 3, [1.0]),
    ]
    inputs = [[1, kind, angle, count, 0, *params] for kind, angle, count, params in pieces]
    inputs += [[1, kind, angle, count, 1, *params] for kind, angle, count, params in pieces[:5]]
    return inputs


def segment(a, b) -> list[float]:
    return [0, *a, *b]


def arc(center, radius: float, a1: float, a2: float) -> list[float]:
    return [1, *center, radius, a1, a2]


def bezier(poles) -> list[float]:
    return [2, len(poles)] + [c for p in poles for c in p]


def wire(edges: list[list[float]]) -> list[float]:
    return [2, len(edges)] + [c for e in edges for c in e]


def wire_inputs() -> list[list[float]]:
    o = (0.0, 0.0, 0.0)
    return [
        wire([segment(o, (1, 0, 0)), segment((1, 0, 0), (2, 0, 0)), segment((2, 0, 0), (3, 0, 0))]),
        wire([segment(o, (1, 0, 0)), segment((1, 0, 0), (1, 1, 0))]),
        wire([arc(o, 2.0, 0.0, 1.0), arc(o, 2.0, 1.0, 2.0), arc(o, 2.0, 2.0, PI)]),
        wire([arc(o, 1.0, 0.0, PI), arc(o, 1.0, PI, 2 * PI)]),
        wire([bezier([o, (0.5, 0.0, 0.0), (1.0, 1.0, 0.0)]), bezier([(1.0, 1.0, 0.0), (1.5, 2.0, 0.0), (2.0, 4.0, 0.0)])]),
        wire([segment(o, (1, 0, 0)), segment((1, 0, 0), (2, 1e-9, 0))]),
    ]


def face_inputs() -> list[list[float]]:
    """Planar faces bounded by a Bezier chain or arcs closed by segments."""
    o = (0.0, 0.0, 0.0)
    chain = [bezier([o, (0.5, 0.0, 0.0), (1.0, 1.0, 0.0)]), bezier([(1.0, 1.0, 0.0), (1.5, 2.0, 0.0), (2.0, 4.0, 0.0)])]
    closing = [segment((2.0, 4.0, 0.0), (0.0, 4.0, 0.0)), segment((0.0, 4.0, 0.0), o)]
    arcs = [arc(o, 2.0, 0.0, 1.0), arc(o, 2.0, 1.0, 2.0), segment((2 * math.cos(2.0), 2 * math.sin(2.0), 0.0), o),
            segment(o, (2.0, 0.0, 0.0))]
    lines = [segment(o, (1, 0, 0)), segment((1, 0, 0), (2, 0, 0)), segment((2, 0, 0), (2, 1, 0)), segment((2, 1, 0), o)]
    return [[4, *wire(chain + closing)[1:]], [4, *wire(arcs)[1:]], [4, *wire(lines)[1:]]]


def settings(variant) -> list[float]:
    init, ue, uf, concat, allow, safe, lin, ang, keep = variant
    return [init, ue, uf, concat, allow, safe, lin, ang, len(keep)] + [c for k in keep for c in k]


def append_unify_requests(requests: list[str]) -> None:
    """Append the ShapeUpgrade_UnifySameDomain requests."""
    inputs = polygon_inputs() + revolved_inputs() + wire_inputs() + face_inputs() + [[3, 1.0, 2.0, 3.0]]
    for shape in inputs:
        for variant in VARIANTS:
            requests.append('sh_unify ' + numbers(shape + settings(variant)))
    append_face_connect_requests(requests)


def face_connect(faces, pairs, sew: float = 1e-6, fix: float = 1e-7) -> str:
    fields: list[float] = [len(faces)]
    for face in faces:
        fields += [len(face)] + [c for p in face for c in p]
    fields += [len(pairs)] + [c for pair in pairs for c in pair] + [sew, fix]
    return 'sh_face_connect ' + numbers(fields)


def append_face_connect_requests(requests: list[str]) -> None:
    """ShapeFix_FaceConnect on separate polygon faces of one shell, in the cases whose
    result does not depend on the order of the faces in the connection map: natively the
    map iterates in the order of the TShape addresses (TopTools_ShapeMapHasher), so the
    face sewn first of a real pair, hence the orientation of the shared edge, changes
    from run to run. The translated GTests cover the real pairs."""
    a, b = square(0, 0), square(1, 0)
    far = square(1.01, 0)
    cases = [([a, b], [(0, 0)]), ([a, b], [(1, 1)]), ([a], [(0, 0)]), ([a, b], []), ([a, far], [(0, 1)])]
    for faces, pairs in cases:
        requests.append(face_connect(faces, pairs))
