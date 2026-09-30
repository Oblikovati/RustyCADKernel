"""Deterministic requests for the BRepLib polygon, wire and solid makers
(tools/oracle/tktopalgo_probe_breplib_wire.cxx).

`blm_polygon` covers every BRepLib_MakePolygon constructor, Add of points and vertices,
Close, closing on a vertex that BRepTools::Compare finds equal to the first one, and
segments that BRepLib_MakeEdge refuses (identical points). `blm_wire` covers every
BRepLib_MakeWire constructor and Add: connected edges in any order and orientation, edges
connected through a vertex tolerance (copied onto the wire vertex, with gaps of several
sizes), branches (non-manifold), disconnected edges, closed and degenerated edges, null and
empty wires. `blm_solid` covers every BRepLib_MakeSolid constructor, Add and FaceStatus,
with shells of the face reader's faces and a compsolid sharing a face.
"""
from __future__ import annotations
import math
from tktopalgo_corpus_breplib_records import numbers, vertices
from tktopalgo_corpus_faces import cylinder_face, disc_face, square_face, holed_face, sphere_face

# Polygon vertices: a unit square, a vertex near the first one, a far one and a copy of
# vertex 1 (identical point, another vertex).
POLYGON_TABLE = [(0., 0., 0., 1e-7), (1., 0., 0., 1e-7), (1., 1., 0., 1e-7), (0., 1., 0., 1e-7),
                 (1e-8, 0., 0., 1e-7), (5., 5., 5., 1e-3), (1., 0., 0., 1e-7), (0., 0., 1e-4, 1e-3)]


def polygon_request(ctor: list[float], ops: list[list[float]], prec: float = 0.) -> str:
    """`blm_polygon prec vertices ctor args nops (op)...`."""
    fields = [prec] + vertices(POLYGON_TABLE) + ctor + [len(ops)] + [c for op in ops for c in op]
    return 'blm_polygon ' + numbers(fields)


def polygon_requests() -> list[str]:
    p = [(0., 0., 0.), (1., 0., 0.), (1., 1., 0.), (0., 1., 0.)]
    add = lambda q: [0, *q]  # noqa: E731 - one op record
    requests = [polygon_request([0], []), polygon_request([0], [[2]]), polygon_request([0], [add(p[0])]),
                polygon_request([0], [add(p[0]), [2]]), polygon_request([0], [add(p[0]), add(p[1]), [2]])]
    for close in (0, 1):
        requests += [polygon_request([2, *p[0], *p[1], *p[2], close], []),
                     polygon_request([3, *p[0], *p[1], *p[2], *p[3], close], []),
                     polygon_request([5, 0, 1, 2, close], []), polygon_request([6, 0, 1, 2, 3, close], []),
                     polygon_request([6, 0, 1, 6, 2, close], []), polygon_request([5, 0, 1, 0, close], []),
                     polygon_request([5, 0, 1, 4, close], []), polygon_request([6, 0, 1, 2, 7, close], []),
                     polygon_request([3, *p[0], *p[1], *p[1], *p[2], close], []),
                     polygon_request([3, *p[0], *p[1], *p[2], 1e-8, 0., 0., close], [])]
    requests += [polygon_request([1, *p[0], *p[1]], [add(p[2]), add(p[3]), [2], [2]]),
                 polygon_request([1, *p[0], *p[0]], [add(p[1]), add(p[2])]),
                 polygon_request([4, 0, 1], [[1, 2], [1, 3], [1, 0], add(p[1])]),
                 polygon_request([4, 0, 0], [[1, 1], [2]]), polygon_request([4, 1, 6], [[1, 2], [2]]),
                 polygon_request([4, 0, 1], [[1, 1], [1, 5], [2]]),
                 polygon_request([0], [[1, 0], [1, 1], [1, 2], [1, 4]]),
                 polygon_request([0], [[1, 0], [1, 1], [1, 2], [1, 4]], prec=1e-3),
                 polygon_request([1, *p[0], 1e-5, 0., 0.], [], prec=1e-3)]
    return requests


# Wire vertices: the square 0..3, then vertices near vertex 1 (in its tolerance or with
# their own), a far one, a branch point and one just outside every tolerance.
WIRE_TABLE = [(0., 0., 0., 1e-7), (1., 0., 0., 1e-7), (1., 1., 0., 1e-7), (0., 1., 0., 1e-7),
              (1., 1e-5, 0., 1e-3), (2., 0., 0., 1e-7), (0.5, 0.5, 0., 1e-7), (1.0002, 0., 0., 1e-7),
              (1., 0., 5e-4, 1e-3), (1. + 3e-4, 0., 0., 2e-4), (0., 0., 1e-4, 1e-3), (1.5, 0.5, 0., 1e-7)]
# Edges `kind vf vl ...`.
WIRE_EDGES = [[0, 0, 1], [0, 1, 2], [0, 2, 3], [0, 3, 0], [0, 4, 5], [0, 1, 6], [0, 7, 5],
              [1, 0, 0, -1., 0., 0., 0., 0., 1., 1., 0., 2 * math.pi], [2, 2, 2], [0, 8, 11],
              [0, 0, 2], [0, 9, 5], [0, 10, 3], [0, 5, 11], [1, 1, 2, 1., 0.5, 0., 0., 0., 1., 0.5, -math.pi / 2, math.pi / 2],
              [2, -1, -1]]


def edge_list() -> list[float]:
    return [len(WIRE_EDGES)] + [c for e in WIRE_EDGES for c in e]


def wire_request(ctor: list[float], ops: list[list[float]]) -> str:
    """`blm_wire vertices edges ctor args nops (op)...`."""
    fields = vertices(WIRE_TABLE) + edge_list() + ctor + [len(ops)] + [c for op in ops for c in op]
    return 'blm_wire ' + numbers(fields)


def wire_requests() -> list[str]:
    F, R = 0, 1
    square = [[0, F], [1, F], [2, F], [3, F]]
    flat = lambda refs: [c for ref in refs for c in ref]  # noqa: E731
    add = lambda e, o=F: [0, e, o]  # noqa: E731
    requests = [wire_request([0], []), wire_request([5, -1], []), wire_request([5, 0], []),
                wire_request([5, -1], [add(0)]), wire_request([6, -1, 0, F], [])]
    requests += [wire_request([1, *square[0]], []), wire_request([2, 0, F, 1, F], []),
                 wire_request([3, 0, F, 1, F, 2, F], []), wire_request([4, *flat(square)], []),
                 wire_request([4, 3, F, 2, F, 1, F, 0, F], []), wire_request([4, 0, R, 1, F, 2, R, 3, F], []),
                 wire_request([4, 1, F, 3, F, 0, F, 2, F], []), wire_request([4, 0, F, 2, F, 1, F, 3, F], []),
                 wire_request([2, 0, F, 4, F], []), wire_request([2, 0, F, 9, F], []),
                 wire_request([2, 0, F, 11, F], []), wire_request([2, 0, F, 6, F], []),
                 wire_request([3, 0, F, 1, F, 5, F], []), wire_request([2, 0, F, 10, F], []),
                 wire_request([1, 7, F], [add(0), add(1)]), wire_request([1, 8, F], [add(1), add(2)]),
                 wire_request([2, 0, F, 14, F], [add(2), add(3)]), wire_request([1, 15, F], [add(0)]),
                 wire_request([2, 0, F, 12, F], [add(2)]), wire_request([2, 3, R, 12, R], [])]
    requests += [wire_request([5, 4, *flat(square)], []), wire_request([5, 3, 0, F, 1, F, 6, F], []),
                 wire_request([6, 3, 0, F, 1, F, 2, F, 3, F], []), wire_request([6, 2, 0, F, 1, F, 13, F], []),
                 wire_request([5, 4, *flat(square)], [add(10), add(5)]),
                 wire_request([0], [[1, 2, 0, F, 1, F], [1, 2, 2, F, 3, F]]),
                 wire_request([0], [[1, 3, 0, F, 6, F, 1, F], add(3), add(2)]),
                 wire_request([0], [add(0), add(6), add(1)]), wire_request([1, 0, F], [add(4), add(13)]),
                 wire_request([1, 0, R], [add(4, R), add(1, R)]), wire_request([1, 10, F], [add(0), add(1), add(2)])]
    lst = lambda refs: [2, len(refs)] + flat(refs)  # noqa: E731 - an Add(L) op
    requests += [wire_request([0], [lst([[2, F], [0, F], [3, F], [1, F]])]), wire_request([0], [lst([])]),
                 wire_request([0], [lst([[0, F], [6, F]])]), wire_request([0], [lst([[0, F], [4, F], [13, F]])]),
                 wire_request([1, 0, F], [lst([[1, F], [2, F], [3, F]])]), wire_request([1, 0, F], [lst([[9, F], [11, F]])]),
                 wire_request([0], [lst([[0, F], [4, F], [9, F]])]), wire_request([0], [lst([[3, R], [2, R], [1, F]])]),
                 wire_request([2, 0, F, 1, F], [lst([[12, F], [2, F]])]), wire_request([0], [lst([[7, F], [0, F], [5, F]])]),
                 wire_request([0], [lst([[10, F], [0, F], [1, F], [5, F]])]), wire_request([1, 15, F], [lst([[0, F]])])]
    return requests


def solid_request(faces: list[list[float]], shells: list[list[int]], ctor: list[float], ops: list[int]) -> str:
    """`blm_solid nfaces (face)... nshells (n face-indices)... ctor args nops (shell)...`."""
    fields = [len(faces)] + [c for f in faces for c in f] + [len(shells)] + \
        [c for s in shells for c in [len(s), *s]] + ctor + [len(ops)] + ops
    return 'blm_solid ' + numbers(fields)


def solid_requests() -> list[str]:
    faces = [square_face().numbers(), holed_face().numbers(), disc_face().numbers(),
             cylinder_face().numbers(), sphere_face().numbers()]
    shells = [[0, 1], [2], [3, 4], [1, 2], []]
    requests = [solid_request(faces, shells, [0], []), solid_request(faces, shells, [0], [0, 1]),
                solid_request(faces, shells, [2, 0], []), solid_request(faces, shells, [3, 0, 1], [2]),
                solid_request(faces, shells, [4, 0, 1, 2], []), solid_request(faces, shells, [5, 2, 0, 2], []),
                solid_request(faces, shells, [6, 1, 1, 3], [4]), solid_request(faces, shells, [5, 0], []),
                solid_request(faces, shells, [2, 4], [])]
    requests += [solid_request(faces, shells, [1, 2, 0, 3], []), solid_request(faces, shells, [1, 1, 2], []),
                 solid_request(faces, shells, [1, 3, 0, 3, 1], []), solid_request(faces, shells, [1, 0], []),
                 solid_request(faces, shells, [1, 3, 0, 3, 3], [2])]
    return requests


def append_breplib_wire_requests(requests: list[str]) -> None:
    """Append the polygon, wire and solid requests in a fixed order."""
    requests += polygon_requests()
    requests += wire_requests()
    requests += solid_requests()
