"""Deterministic requests for BRepExtrema, distance group
(tools/oracle/tktopalgo_probe_brepextrema.cxx and tktopalgo_probe_brepextrema_ext.cxx,
docs/topalgo-design.md section 13); the proximity group is in
tktopalgo_corpus_brepextrema_prox.py.

Every request carries its shape records (tktopalgo_corpus_brepextrema_shapes.py and the
named solids) and never relies on an earlier request:
- `bxd_dist`: DistShapeShape on every pair of a vertex, segment, arc, C0 B-spline, Bezier,
  half-infinite line, rectangle, infinite plane, cylinder, sphere, shell, box, cylinder,
  sphere, hollow box and compound, apart, touching, crossing and inside (inner solutions),
  with the deflections 1e-7, 1e-3 and 2, the three construction modes, the multithread
  flag (only where at most one vertex can be inside a solid: several tasks may otherwise
  each report an inner vertex in an order that follows the thread timing), the obsolete
  flag and algorithm, and a second Perform then swapped shapes; every solution accessor
  and the accessors at 0 and N+1;
- `bxd_dump`: Dump of performed and empty distances;
- `bxd_ss`: DistanceSS on every pair of sub-shape kinds in both orders, with void and real
  boxes, start distances below, at and above the minimum, deflections and the point-face
  flags and algorithms; infinite edges and faces (trimmed), C0 edges (break points);
- `bxd_poly`: Poly on meshed and unmeshed shapes;
- `bxx_pc`, `bxx_pf`, `bxx_cc`, `bxx_cf`, `bxx_ff`: the Ext wrappers built, initialized and
  performed in their modes (repeated Performs, no Initialize, another edge or face).
"""
from __future__ import annotations
import math
from tktopalgo_corpus_brepbndlib import numbers
from tktopalgo_corpus_brepclass3d import two_boxes, with_location
from tktopalgo_corpus_brepextrema_shapes import (INFINITE, arc_shape, bezier_shape, c0_bspline_shape,
                                                 infinite_line_shape, natural_plane_shape, points_shape,
                                                 polygon_edge_shape, rectangle_shape, segment_shape, shell_of,
                                                 face_of, vertex_shape, with_mesh, cylinder_point, sphere_point)
from tktopalgo_corpus_shapes import ShapeSpec, rotation, translation
from tktopalgo_corpus_solids import box, cylinder, hollow_box, sphere, torus

Z, X, Y = [0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]


def named_shapes() -> dict[str, ShapeSpec]:
    """The shapes of the distance pairs, around the unit box [0, 2]^3 (factories)."""
    return {
        'vertex': lambda: vertex_shape([1.0, 1.0, 5.0]),
        'segment': lambda: segment_shape([-1.0, 0.5, 4.0], [3.0, 1.5, 4.5]),
        'arc': lambda: arc_shape([1.0, 1.0, 4.0], Z, X, 1.5, (0.3, 2.4)),
        'circle': lambda: arc_shape([1.0, 1.0, -3.0], X, Y, 1.0),
        'c0spline': lambda: c0_bspline_shape([[-1.0, -1.0, 3.5], [0.0, 1.0, 3.0], [1.0, 1.0, 3.2],
                                              [2.0, 0.0, 3.8], [3.0, 2.0, 3.1]]),
        'bezier': lambda: bezier_shape([[-1.0, 3.0, 0.0], [0.5, 5.0, 1.0], [2.0, 3.5, 2.0], [3.0, 4.0, 0.5]]),
        'halfline': lambda: infinite_line_shape([1.0, 4.0, 1.0], [1.0, 0.0, 0.0], first=0.0),
        'rectangle': lambda: rectangle_shape([-1.0, -1.0, 3.0], Z, X, (3.0, 2.5)),
        'plane': lambda: natural_plane_shape([0.0, 0.0, -1.5], Z, X),
        'cylface': lambda: face_of(cylinder([4.0, 1.0, 0.0], 0.8, 2.0)),
        'sphface': lambda: face_of(sphere([1.0, -3.0, 1.0], 1.0)),
        'shell': lambda: shell_of(box([4.0, 4.0, 0.0], [5.0, 5.5, 1.0])),
        'box': lambda: box([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]),
        'cylinder': lambda: cylinder([5.0, -2.0, 0.0], 1.0, 3.0),
        'sphere': lambda: sphere([-3.0, 1.0, 1.0], 1.2),
        'hollow': lambda: hollow_box([-6.0, -6.0, -6.0], [-3.0, -3.0, -3.0], [-5.0, -5.0, -5.0], [-4.0, -4.0, -4.0]),
        'compound': lambda: with_location(two_boxes(), translation(-1.0, 6.0, 0.0)),
    }


# The pairs whose second shape may hold several vertices inside a solid of the first
# (or the reverse): no multithread request for them.
INNER_PAIRS = {('box', 'inner'), ('inner', 'box')}


def inner_cases() -> list[tuple[str, ShapeSpec, str, ShapeSpec]]:
    """Inner solutions: vertices, edges, faces and solids inside or across a solid."""
    big = lambda: box([0.0, 0.0, 0.0], [4.0, 4.0, 4.0])
    return [
        ('box', big(), 'inner', vertex_shape([1.0, 2.0, 3.0])),
        ('box', big(), 'inner', segment_shape([1.0, 1.0, 1.0], [3.0, 2.0, 3.0])),
        ('box', big(), 'inner', segment_shape([1.0, 1.0, 1.0], [6.0, 2.0, 3.0])),
        ('box', big(), 'inner', box([1.0, 1.0, 1.0], [2.0, 2.0, 2.0])),
        ('box', big(), 'inner', rectangle_shape([1.0, 1.0, 2.0], Z, X, (1.0, 1.0))),
        ('inner', vertex_shape([2.0, 2.0, 2.0]), 'box', big()),
        ('inner', points_shape([[1.0, 1.0, 1.0], [9.0, 9.0, 9.0], [2.0, 3.0, 1.0]]), 'box', big()),
        ('box', sphere([2.0, 2.0, 2.0], 1.5), 'inner', vertex_shape([2.1, 2.2, 1.9])),
        ('box', hollow_box([0.0, 0.0, 0.0], [6.0, 6.0, 6.0], [2.0, 2.0, 2.0], [4.0, 4.0, 4.0]), 'inner',
         vertex_shape([3.0, 3.0, 3.0])),
        ('box', hollow_box([0.0, 0.0, 0.0], [6.0, 6.0, 6.0], [2.0, 2.0, 2.0], [4.0, 4.0, 4.0]), 'inner',
         vertex_shape([1.0, 3.0, 3.0])),
        ('box', cylinder([0.0, 0.0, 0.0], 2.0, 3.0), 'inner', box([-0.5, -0.5, 1.0], [0.5, 0.5, 2.0])),
    ]


def dist_request(a: ShapeSpec, b: ShapeSpec, defl: float, mode: int, mt: int = 0, flag: int = 2, algo: int = 0,
                 again: int = 0) -> str:
    """A `bxd_dist` request."""
    return 'bxd_dist ' + numbers([*a.numbers(), *b.numbers(), defl, mode, mt, flag, algo, again])


def _dist_pairs(requests: list[str]) -> None:
    shapes = named_shapes()
    names = list(shapes)
    for i, first in enumerate(names):
        for second in names[i:]:
            requests.append(dist_request(shapes[first](), shapes[second](), 1e-7, 0))
            if (i + names.index(second)) % 3 == 0:
                requests.append(dist_request(shapes[second](), shapes[first](), 1e-3, 1, mt=1))
            if (i * 7 + names.index(second)) % 5 == 0:
                requests.append(dist_request(shapes[first](), shapes[second](), 2.0, 2, again=1))


def _dist_variants(requests: list[str]) -> None:
    shapes = named_shapes()
    pairs = [('box', 'segment'), ('box', 'rectangle'), ('arc', 'circle'), ('c0spline', 'rectangle'),
             ('cylinder', 'sphere'), ('vertex', 'plane'), ('halfline', 'cylface'), ('bezier', 'segment'),
             ('shell', 'box'), ('compound', 'box')]
    for first, second in pairs:
        for defl in (1e-7, 1e-3, 0.5, 2.0):
            for mode, mt in ((0, 0), (1, 0), (1, 1), (2, 0)):
                requests.append(dist_request(shapes[first](), shapes[second](), defl, mode, mt))
        for flag, algo in ((0, 0), (1, 1), (2, 1)):
            requests.append(dist_request(shapes[first](), shapes[second](), 1e-7, 1, 0, flag, algo, 1))
    for first_name, first, second_name, second in inner_cases():
        for mode, mt in ((0, 0), (1, 0), (1, 1), (2, 0)):
            if mt and len(second.vertices) > 1 and len(first.vertices) > 1:
                continue
            requests.append(dist_request(first, second, 1e-7, mode, mt, again=1))
    located = with_location(box([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]), rotation([1.0, 1.0, 1.0], Z, 0.4))
    requests.append(dist_request(located, shapes['segment'](), 1e-7, 1, 1, again=1))
    requests.append(dist_request(ShapeSpec(top=[]), shapes['box'](), 1e-7, 1))
    requests.append(dist_request(shapes['box'](), ShapeSpec(top=[]), 1e-7, 0))
    # Touching and parallel configurations (several solutions at one distance).
    requests.append(dist_request(box([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]), box([1.0, 0.0, 0.0], [2.0, 1.0, 1.0]), 1e-7, 0))
    requests.append(dist_request(box([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]), box([3.0, 0.0, 0.0], [4.0, 1.0, 1.0]), 1e-7, 1, 1))
    requests.append(dist_request(segment_shape([0.0, 0.0, 0.0], [4.0, 0.0, 0.0]),
                                 segment_shape([0.0, 1.0, 0.0], [4.0, 1.0, 0.0]), 1e-7, 0))
    requests.append(dist_request(segment_shape([0.0, 0.0, 0.0], [4.0, 0.0, 0.0]),
                                 segment_shape([2.0, -1.0, 1.0], [2.0, 1.0, 1.0]), 1e-7, 1, 1, again=1))


def _dump_requests(requests: list[str]) -> None:
    shapes = named_shapes()
    for first, second in (('box', 'segment'), ('vertex', 'rectangle'), ('arc', 'circle'), ('cylinder', 'sphere'),
                          ('halfline', 'box'), ('c0spline', 'plane')):
        for defl in (1e-7, 0.5):
            requests.append('bxd_dump ' + numbers([*shapes[first]().numbers(), *shapes[second]().numbers(), defl]))
    requests.append('bxd_dump ' + numbers([*box([0.0] * 3, [4.0] * 3).numbers(), *vertex_shape([1.0] * 3).numbers(),
                                           1e-7]))


def sub(kind: int, index: int) -> list[int]:
    """A `tp_bxe_sub` selector: 0 shape, 1 vertex, 2 edge, 3 face, 5/6/7 explored face/edge/vertex."""
    return [kind, index]


def ss_request(a: ShapeSpec, sa: list[int], b: ShapeSpec, sb: list[int], boxes: int, dst: float,
               defl: float = 1e-7, flag: int = 2, algo: int = 0) -> str:
    """A `bxd_ss` request."""
    return 'bxd_ss ' + numbers([*a.numbers(), *sa, *b.numbers(), *sb, boxes, dst, defl, flag, algo])


def ss_items() -> list[tuple[ShapeSpec, list[int]]]:
    """Sub-shapes of every kind for DistanceSS."""
    shapes = named_shapes()
    return [
        (shapes['vertex'](), sub(1, 0)), (shapes['box'](), sub(1, 3)),
        (shapes['segment'](), sub(2, 0)), (shapes['arc'](), sub(2, 0)), (shapes['c0spline'](), sub(2, 0)),
        (shapes['halfline'](), sub(2, 0)), (infinite_line_shape([0.0, 3.0, 3.0], [1.0, 1.0, 0.0]), sub(2, 0)),
        (shapes['box'](), sub(2, 5)), (shapes['cylinder'](), sub(2, 0)), (shapes['sphere'](), sub(2, 0)),
        (shapes['rectangle'](), sub(3, 0)), (shapes['plane'](), sub(3, 0)), (shapes['cylface'](), sub(3, 0)),
        (shapes['sphface'](), sub(3, 0)), (shapes['box'](), sub(5, 2)),
        (with_location(box([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]), translation(0.0, 0.0, -4.0)), sub(5, 4)),
    ]


def _ss_requests(requests: list[str]) -> None:
    items = ss_items()
    for i, (a, sa) in enumerate(items):
        for j, (b, sb) in enumerate(items):
            if (i + 2 * j) % 3 != 0 and i != j:
                continue
            requests.append(ss_request(a, sa, b, sb, 1, 1e30))
            requests.append(ss_request(a, sa, b, sb, (i + j) % 2, 1e30, 1e-3, 2 - (i % 3), j % 2))
    vertex, rect = vertex_shape([0.5, 0.5, 2.0]), rectangle_shape([0.0, 0.0, 0.0], Z, X, (1.0, 1.0))
    for dst in (0.5, 2.0 - 1e-9, 2.0, 2.0 + 1e-9, 5.0):
        requests.append(ss_request(vertex, sub(1, 0), rect, sub(3, 0), 1, dst))
        requests.append(ss_request(rect, sub(3, 0), vertex, sub(1, 0), 0, dst, 1e-3))
    for flag in range(3):
        for algo in range(2):
            requests.append(ss_request(vertex, sub(1, 0), rect, sub(3, 0), 1, 1e30, 1e-7, flag, algo))
    # Degenerated edges (sphere poles) and a non-geometric edge give nothing.
    sph = named_shapes()['sphere']()
    requests.append(ss_request(sph, sub(2, 0), sph, sub(2, 1), 0, 1e30))
    requests.append(ss_request(vertex, sub(1, 0), sph, sub(2, 0), 0, 1e30))
    poly = polygon_edge_shape([[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 1.0, 0.0]])
    requests.append(ss_request(poly, sub(2, 0), rect, sub(3, 0), 0, 1e30))


def _poly_requests(requests: list[str]) -> None:
    rect = lambda z, cells: rectangle_shape([0.0, 0.0, z], Z, X, (2.0, 1.0), mesh=cells)
    cyl = with_mesh(face_of(cylinder([4.0, 0.0, 0.0], 1.0, 2.0)), 0, cylinder_point([4.0, 0.0, 0.0], 1.0, 2.0), (8, 2))
    sph = with_mesh(face_of(sphere([0.0, 3.0, 0.0], 1.0)), 0, sphere_point([0.0, 3.0, 0.0], 1.0), (6, 4))
    cases = [(rect(0.0, (1, 1)), rect(1.0, (2, 2))), (rect(0.0, (3, 2)), cyl), (cyl, sph), (sph, rect(-2.0, (2, 1))),
             (rect(0.0, (1, 1)), with_location(rect(0.0, (2, 2)), translation(0.5, 0.5, 3.0))),
             (rect(0.0, (1, 1)), rectangle_shape([0.0, 0.0, 1.0], Z, X, (1.0, 1.0))),
             (box([0.0] * 3, [1.0] * 3), cyl)]
    for a, b in cases:
        requests.append('bxd_poly ' + numbers([*a.numbers(), *b.numbers()]))


def append_brepextrema_requests(requests: list[str]) -> None:
    """Append the BRepExtrema request families (distance, then Ext, then proximity)."""
    _dist_pairs(requests)
    _dist_variants(requests)
    _dump_requests(requests)
    _ss_requests(requests)
    _poly_requests(requests)
    __import__('tktopalgo_corpus_brepextrema_ext').append_brepextrema_ext_requests(requests)
    __import__('tktopalgo_corpus_brepextrema_prox').append_brepextrema_prox_requests(requests)
