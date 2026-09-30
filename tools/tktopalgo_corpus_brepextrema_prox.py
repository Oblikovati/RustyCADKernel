"""Deterministic requests for the BRepExtrema proximity tools
(tools/oracle/tktopalgo_probe_brepextrema_prox.cxx, docs/topalgo-design.md section 13).

The shapes carry grid triangulations of their faces and 3D polygons of their edges
(tktopalgo_corpus_brepextrema_shapes.py):
- `bxp_tset`: triangle sets of faces and edges (with locations), their boxes, centers,
  vertex data and BVH nodes; Init twice (stale vertex data) and Swap/Clear;
- `bxp_overlap`: overlap tools on two sets and on one set, exact and with tolerances
  (the maps accumulate over the tolerances);
- `bxp_self`: self-intersection of meshed shapes (boxes, crossing and touching
  rectangles, a folded surface), with the filter of adjacent triangles, and `bxp_shared`
  on shared-vertex and shared-edge configurations;
- `bxp_pdist`, `bxp_border`, `bxp_pval`, `bxp_prox`: the proximity distance and value tools
  and shape proximity on edge pairs (arcs, circles, Bezier curves) and face pairs (planes,
  cylinder and sphere patches) with sample counts, additional vertices and repeated runs,
  the node and link border tests, and shape proximity in overlap mode;
- `bxp_insp`: the vertex inspector.
"""
from __future__ import annotations
import math
from tktopalgo_corpus_brepbndlib import numbers
from tktopalgo_corpus_brepclass3d import with_location
from tktopalgo_corpus_brepextrema import X, Y, Z, sub
from tktopalgo_corpus_brepextrema_shapes import (arc_shape, bezier_shape, cylinder_point, face_of, rectangle_shape,
                                                 segment_shape, sphere_point, with_mesh)
from tktopalgo_corpus_shapes import FACE, Mesh, ShapeSpec, rotation, translation
from tktopalgo_corpus_solids import box, cylinder, sphere

INF = 2e100


def meshed_box(low: list[float], high: list[float]) -> ShapeSpec:
    """A box solid with the fan meshes of its faces (two triangles each)."""
    return box(low, high, mesh=1e-3)


def rect(origin: list[float], normal: list[float], x_dir: list[float], size: tuple[float, float],
         cells: tuple[int, int] = (2, 2)) -> ShapeSpec:
    """A meshed plane rectangle."""
    return rectangle_shape(origin, normal, x_dir, size, mesh=cells, polygon=3)


def cyl_face(origin: list[float], radius: float, height: float, cells: tuple[int, int]) -> ShapeSpec:
    """The meshed lateral face of a z-axis cylinder."""
    return with_mesh(face_of(cylinder(origin, radius, height)), 0, cylinder_point(origin, radius, height), cells)


def sph_face(centre: list[float], radius: float, cells: tuple[int, int]) -> ShapeSpec:
    """The meshed face of a sphere."""
    return with_mesh(face_of(sphere(centre, radius)), 0, sphere_point(centre, radius), cells)


def edge_pairs() -> list[tuple[ShapeSpec, ShapeSpec]]:
    """Edge pairs with 3D polygons (the tests A1 to A5 and E1, E2 with coarse polygons)."""
    half = 0.5 * math.pi
    poles1 = [[10.0, 84.0, 0.0], [54.0, 96.0, 0.0], [145.0, 146.0, 0.0], [167.0, 167.0, 0.0],
              [185.0, 212.0, 0.0], [187.0, 234.0, 0.0], [176.0, 302.0, 0.0]]
    poles2 = [[120.0, 72.0, 0.0], [170.0, 87.0, 0.0], [227.0, 118.0, 0.0], [238.0, 126.0, 0.0],
              [243.0, 157.0, 0.0], [203.0, 216.0, 0.0], [134.0, 281.0, 0.0], [94.0, 324.0, 0.0]]
    return [
        (arc_shape([0.0] * 3, X, Y, 5.0, (half, math.pi), 9), arc_shape([0.0] * 3, X, Y, 5.0, (-half, 0.0), 9)),
        (arc_shape([0.0] * 3, X, Y, 1.0, (0.0, 2 * math.pi), 17), arc_shape([0.0] * 3, Y, Z, 1.0, (0.0, 2 * math.pi), 13)),
        (arc_shape([0.0, -1.0, 0.0], X, Y, 1.0, (-math.pi, 0.0), 11), arc_shape([0.0, 1.0, 0.0], X, Y, 1.0, (0.0, math.pi), 7)),
        (bezier_shape(poles1, 15), bezier_shape(poles2, 12)),
        (bezier_shape(poles1[1:], 9), bezier_shape(poles2[:-1], 21)),
        (segment_shape([0.0, 0.0, 0.0], [4.0, 0.0, 0.0], polygon=5), segment_shape([0.0, 1.0, 0.5], [4.0, 1.5, 0.0], polygon=3)),
        (segment_shape([0.0, 0.0, 0.0], [4.0, 0.0, 0.0], polygon=2), arc_shape([2.0, 0.0, 0.0], Z, X, 1.0, (0.2, 2.9), 25)),
    ]


def face_pairs() -> list[tuple[ShapeSpec, ShapeSpec]]:
    """Face pairs with grid meshes (the tests B1 to B4, C1 to C4, D1 and D4 with coarse meshes)."""
    return [
        (rect([-1.0, -1.0, 0.0], Z, X, (2.0, 2.0)), rect([-1.0, -1.0, 1.0], Z, X, (2.0, 2.0))),
        (rect([-1.0, -1.0, 0.0], Z, X, (2.0, 2.0), (4, 4)), rect([-1.0, -1.0, 1.0], Z, X, (2.0, 2.0), (1, 1))),
        (rect([-1.0, -1.0, 0.0], Z, X, (2.0, 2.0)), rect([0.0, -1.0, 0.0], X, Y, (2.0, 2.0))),
        (rect([-1.0, -1.0, 0.0], Z, X, (2.0, 2.0)), rect([0.0, -1.0, -0.5], X, Y, (2.0, 2.0), (3, 3))),
        (rect([-1.0, -1.0, 0.0], Z, X, (2.0, 2.0), (3, 2)), cyl_face([0.0, 0.0, 0.5], 0.3, 1.0, (8, 2))),
        (cyl_face([0.0, 0.0, 0.0], 0.5, 1.0, (6, 2)), cyl_face([0.0, 2.0, 0.0], 0.5, 1.0, (10, 3))),
        (sph_face([0.0, 1.0, 0.0], 1.0, (6, 4)), sph_face([0.0, -1.5, 0.0], 1.0, (8, 6))),
        (sph_face([0.0, 1.2, 0.0], 1.0, (4, 4)), rect([-1.0, 0.0, -1.0], Y, X, (2.0, 2.0), (2, 2))),
    ]


def triangle_sets() -> list[tuple[ShapeSpec, list[list[int]]]]:
    """Shapes with the sub-shape lists of their triangle sets (faces then edges)."""
    boxed = meshed_box([0.0, 0.0, 0.0], [1.0, 2.0, 3.0])
    located = with_location(meshed_box([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]), rotation([0.0] * 3, Z, 0.7))
    plane = rect([0.0, 0.0, 0.0], Z, X, (2.0, 1.0), (3, 2))
    return [
        (boxed, [sub(3, i) for i in range(6)]),
        (located, [sub(5, i) for i in range(6)]),
        (plane, [sub(3, 0), sub(2, 0), sub(2, 2)]),
        (plane, [sub(2, 1), sub(3, 0)]),
        (cyl_face([0.0, 0.0, 0.0], 1.0, 2.0, (5, 2)), [sub(3, 0)]),
        (arc_shape([0.0] * 3, Z, X, 2.0, (0.0, 3.0), 6), [sub(2, 0)]),
        (boxed, [sub(3, 0), sub(2, 0)]),
    ]


def _tset_requests(requests: list[str]) -> None:
    sets = triangle_sets()
    for spec, items in sets:
        flat = [c for item in items for c in item]
        requests.append('bxp_tset ' + numbers([*spec.numbers(), len(items), *flat, 0]))
        requests.append('bxp_tset ' + numbers([*spec.numbers(), len(items), *flat, 2]))
    boxed, items = sets[0]
    for first, second in ((items, items[3:]), (items[:2], items), (items[4:], items[:1])):
        flat1 = [c for item in first for c in item]
        flat2 = [c for item in second for c in item]
        requests.append('bxp_tset ' + numbers([*boxed.numbers(), len(first), *flat1, 1, len(second), *flat2]))


def overlap_list(spec: ShapeSpec) -> list[int]:
    """`n (kind index)...` of every face of a spec."""
    return [len(spec.faces), *[c for i in range(len(spec.faces)) for c in sub(3, i)]]


def _overlap_requests(requests: list[str]) -> None:
    cases = [
        (meshed_box([0.0] * 3, [1.0] * 3), meshed_box([0.5, 0.5, 0.5], [1.5, 1.5, 1.5])),
        (meshed_box([0.0] * 3, [1.0] * 3), meshed_box([1.2, 0.0, 0.0], [2.0, 1.0, 1.0])),
        (meshed_box([0.0] * 3, [1.0] * 3), meshed_box([1.0, 0.0, 0.0], [2.0, 1.0, 1.0])),
        (rect([0.0, 0.0, 0.0], Z, X, (2.0, 2.0), (3, 3)), rect([1.0, -0.5, -1.0], X, Y, (3.0, 2.0), (2, 2))),
        (rect([0.0, 0.0, 0.0], Z, X, (2.0, 2.0), (2, 2)), rect([0.5, 0.5, 0.0], Z, X, (2.0, 2.0), (2, 2))),
        (cyl_face([0.0] * 3, 1.0, 2.0, (8, 2)), rect([-2.0, -2.0, 1.0], Z, X, (4.0, 4.0), (2, 2))),
        (sph_face([0.0] * 3, 1.0, (6, 4)), sph_face([1.5, 0.0, 0.0], 1.0, (6, 4))),
    ]
    for a, b in cases:
        for tolerances in ([0.0], [0.05], [0.0, 0.3], [0.3, 0.0, 1.0]):
            requests.append('bxp_overlap ' + numbers([*a.numbers(), *overlap_list(a), *b.numbers(), *overlap_list(b),
                                                      len(tolerances), *tolerances]))
    self_box = meshed_box([0.0] * 3, [1.0] * 3)
    for tolerances in ([0.0], [0.1]):
        requests.append('bxp_overlap ' + numbers([*self_box.numbers(), *overlap_list(self_box), *ShapeSpec().numbers(),
                                                  0, len(tolerances), *tolerances]))


def crossing_rectangles() -> ShapeSpec:
    """A compound of two meshed rectangles crossing each other."""
    spec = rect([0.0, 0.0, 0.0], Z, X, (2.0, 2.0), (2, 2))
    other = rect([1.0, -0.5, -1.0], X, Y, (3.0, 2.0), (2, 2))
    return merged(spec, other)


def merged(a: ShapeSpec, b: ShapeSpec) -> ShapeSpec:
    """A compound of the faces of two single-face records (vertices and edges renumbered)."""
    spec = ShapeSpec(vertices=list(a.vertices), edges=[list(e) for e in a.edges], faces=list(a.faces))
    nv, ne = len(a.vertices), len(a.edges)
    spec.vertices += b.vertices
    for e in b.edges:
        edge = list(e)
        # Curve fields depend on the kind (LINE here: 1 + 6 numbers), then the two vertex indices.
        edge[7] += nv
        edge[8] += nv
        spec.edges.append(edge)
    for f in b.faces:
        wires = [(o, [(edge + ne, eo, pc) for edge, eo, pc in uses]) for o, uses in f.wires]
        mesh = f.mesh and Mesh(f.mesh.nodes, f.mesh.triangles, f.mesh.deflection,
                               [(edge + ne, idx, d) for edge, idx, d in f.mesh.polygons])
        spec.faces.append(type(f)(f.surface, f.tolerance, f.natural, wires, mesh, f.orientation))
    spec.top = [(FACE, 0), (FACE, 1)]
    return spec


def _self_requests(requests: list[str]) -> None:
    shapes = [meshed_box([0.0] * 3, [1.0] * 3), crossing_rectangles(),
              merged(rect([0.0, 0.0, 0.0], Z, X, (2.0, 2.0), (2, 2)), rect([0.5, 0.5, 0.0], Z, X, (1.0, 1.0), (1, 1))),
              merged(rect([0.0, 0.0, 0.0], Z, X, (2.0, 2.0), (2, 2)), rect([2.0, 0.0, 0.0], Z, X, (1.0, 2.0), (1, 2))),
              merged(rect([0.0, 0.0, 0.0], Z, X, (2.0, 2.0), (1, 1)), rect([2.0, 0.0, 0.0], X, Y, (2.0, 2.0), (1, 1))),
              merged(rect([0.0, 0.0, 0.0], Z, X, (2.0, 2.0), (1, 1)), rect([2.0, 0.0, 0.0], [-0.6, 0.0, 0.8],
                                                                           Y, (2.0, 2.0), (1, 1))),
              with_location(meshed_box([0.0] * 3, [1.0, 2.0, 1.0]), translation(1.0, 1.0, 1.0))]
    for spec in shapes:
        for tol in (0.0, 0.1):
            for mode in (0, 1):
                requests.append('bxp_self ' + numbers([*spec.numbers(), tol, mode, 40]))


def _shared_requests(requests: list[str]) -> None:
    o = [0.0, 0.0, 0.0]
    vertex_cases = [
        [o, [1, 0, 0], [0, 1, 0], [-1, 0, 0], [0, -1, 0]],        # coplanar, opposite
        [o, [1, 0, 0], [0, 1, 0], [1, 1, 0], [2, 1, 0]],          # coplanar, overlapping
        [o, [1, 0, 0], [0, 1, 0], [0, 0, 1], [1, 0, 1]],          # perpendicular
        [o, [1, 0, 0], [0, 1, 0], [1, 1, 1], [1, 1, -1]],         # crossing the plane
        [o, [1, 0, 0], [0, 1, 0], [-1, -1, 1], [-1, -1, -1]],
        [o, [1, 0, 0], [0, 1, 0], [0.5, 0.5, 0.0], [0.2, 0.9, 1.0]],
    ]
    edge_cases = [
        [o, [1, 0, 0], [0.5, 1, 0], [0.5, -1, 0]],
        [o, [1, 0, 0], [0.5, 1, 0], [0.7, 2, 0]],
        [o, [1, 0, 0], [0.5, 1, 0], [0.5, 0, 1]],
        [o, [1, 0, 0], [0.5, 0, 1], [0.5, 0, -1]],
    ]
    flat_v = [float(c) for case in vertex_cases for p in case for c in p]
    flat_e = [float(c) for case in edge_cases for p in case for c in p]
    requests.append('bxp_shared ' + numbers([len(vertex_cases), *flat_v, len(edge_cases), *flat_e]))
    for case in vertex_cases:
        requests.append('bxp_shared ' + numbers([1, *[float(c) for p in case for c in p], 0]))


def single(kind: int) -> list[int]:
    """The one-shape list `1 kind 0`."""
    return [1, *sub(kind, 0)]


def _pdist_requests(requests: list[str]) -> None:
    add = [[0.0, 0.5, 0.2, 1], [3.9, 1.0, 0.0, 0], [2.0, 2.0, 2.0, 2]]
    for a, b in edge_pairs():
        for ns in (0, 3):
            requests.append('bxp_pdist ' + numbers([*a.numbers(), *single(2), *b.numbers(), *single(2), ns, 0, ns % 2]))
        requests.append('bxp_pdist ' + numbers([*a.numbers(), *single(2), *b.numbers(), *single(2), 0, len(add),
                                                *[c for p in add for c in p], 1]))
    for a, b in face_pairs():
        requests.append('bxp_pdist ' + numbers([*a.numbers(), *single(3), *b.numbers(), *single(3), 0, 0, 0]))
        requests.append('bxp_pdist ' + numbers([*b.numbers(), *single(3), *a.numbers(), *single(3), 5, len(add),
                                                *[c for p in add for c in p], 1]))
    for a, _ in face_pairs()[:6]:
        requests.append('bxp_border ' + numbers([*a.numbers(), *sub(3, 0)]))
    requests.append('bxp_border ' + numbers([*meshed_box([0.0] * 3, [1.0] * 3).numbers(), *sub(3, 2)]))


def _pval_requests(requests: list[str]) -> None:
    for a, b in edge_pairs():
        for ns1, ns2, mode in ((0, 0, 0), (0, 0, 1), (20, 0, 0), (0, 30, 1), (5, 5, 2)):
            requests.append('bxp_pval ' + numbers([*a.numbers(), *single(2), *b.numbers(), *single(2), ns1, ns2, mode]))
    for a, b in face_pairs():
        for ns1, ns2, mode in ((0, 0, 0), (0, 0, 1), (4, 6, 0)):
            requests.append('bxp_pval ' + numbers([*a.numbers(), *single(3), *b.numbers(), *single(3), ns1, ns2, mode]))
    a, b = face_pairs()[0]
    requests.append('bxp_pval ' + numbers([*a.numbers(), *single(3), *edge_pairs()[0][1].numbers(), *single(2), 0, 0, 0]))


def _prox_requests(requests: list[str]) -> None:
    for a, b in edge_pairs() + face_pairs():
        for mode in (0, 1, 2):
            requests.append('bxp_prox ' + numbers([*a.numbers(), *b.numbers(), INF, 0, 0, mode]))
        requests.append('bxp_prox ' + numbers([*b.numbers(), *a.numbers(), INF, 3, 7, 1]))
    overlapping = [(meshed_box([0.0] * 3, [1.0] * 3), meshed_box([0.5, 0.5, 0.5], [1.5, 1.5, 1.5])),
                   (meshed_box([0.0] * 3, [1.0] * 3), meshed_box([1.2, 0.0, 0.0], [2.0, 1.0, 1.0])),
                   face_pairs()[2], face_pairs()[4]]
    for a, b in overlapping:
        for tol in (0.0, 0.1, 0.5):
            for mode in (0, 2):
                requests.append('bxp_prox ' + numbers([*a.numbers(), *b.numbers(), tol, 0, 0, mode]))


def _insp_requests(requests: list[str]) -> None:
    points = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1e-8, 0.0, 0.0], [0.5, 0.5, 0.5]]
    currents = [([0.0, 0.0, 0.0], [1, 2, 3]), ([1.0, 1e-9, 0.0], [2, 1]), ([0.5, 0.5, 0.5], [4, 1, 4]),
                ([3.0, 3.0, 3.0], [1, 2, 3, 4])]
    for tol in (-1.0, 0.0, 1e-4, 0.3):
        flat = [c for p in points for c in p]
        cur = []
        for p, targets in currents:
            cur += [*p, len(targets), *targets]
        requests.append('bxp_insp ' + numbers([tol, len(points), *flat, len(currents), *cur]))


def append_brepextrema_prox_requests(requests: list[str]) -> None:
    """Append the proximity request families."""
    _tset_requests(requests)
    _overlap_requests(requests)
    _self_requests(requests)
    _shared_requests(requests)
    _pdist_requests(requests)
    _pval_requests(requests)
    _prox_requests(requests)
    _insp_requests(requests)
