"""Requests for ShapeAlgo_AlgoContainer (tools/oracle/tkshhealing_probe_shape_algo.cxx).

`sh_algo_geom 0/1` runs ApproxBSplineCurve on degree-1 polylines with coincident poles at
the start, in the middle (single and repeated), at the end and nowhere, with short and
long pieces (the 3D approximation needs more than 2 poles, the 2D one more than 10), and
on curves of other degrees and kinds (returned as is, or not a B-spline). `2/3` split C0
B-splines (interior knots of full multiplicity) and C1 ones; `4` converts curves, `5`
surfaces on boxes and `6` closed rings to periodic, all through the container. The shape
requests run HomoWires on pairs of polylines, arcs, reversed and located edges and wires
of null total range, ConnectNextWire on wires meeting at each end pairing, far apart and
empty, OuterWire and GetFaceUVBounds on single-wire faces and C0ShapeToC1Shape on faces
of C0 and smooth surfaces.
"""
from __future__ import annotations
import math
from itertools import product

from tkshhealing_corpus_construct_package import ring_bspline, surfaces
from tkshhealing_corpus_shapecustom import ModificationScene, bounded_face
from tkshhealing_corpus_shcommon import CYLINDER, PLANE, Prelude, translation
from tktopalgo_corpus_breplib_records import (
    geom2d_bspline,
    geom2d_gp,
    geom_bezier,
    geom_bspline,
    geom_gp,
    numbers,
)
from tktopalgo_shape_program import FORWARD, REVERSED, WIRE, ShapeProgram, num

Z = (0.0, 0.0, 1.0)
X = (1.0, 0.0, 0.0)
O = (0.0, 0.0, 0.0)
C0, C1, C2 = 0, 2, 4


def polyline_knots(count: int) -> list[tuple[float, int]]:
    """The clamped degree-1 knots of `count` poles, unit spaced."""
    return [(float(i), 2 if i in (0, count - 1) else 1) for i in range(count)]


def with_duplicates(points: list[tuple], duplicates: list[int]) -> list[tuple]:
    """The points with the point of each index of `duplicates` repeated once more."""
    out: list[tuple] = []
    for i, point in enumerate(points):
        out.append(point)
        out.extend([point] * duplicates.count(i))
    return out


def polylines3d() -> list[list[float]]:
    base = [(float(i), math.sin(0.7 * i), 0.1 * i * i) for i in range(12)]
    layouts = [[], [0], [4], [4, 4], [11], [2, 7], [0, 5, 11], [1, 2], [5, 6, 7]]
    curves = []
    for dup in layouts:
        poles = with_duplicates(base, dup)
        curves.append(geom_bspline(1, False, poles, polyline_knots(len(poles))))
    short = with_duplicates(base[:4], [1])
    curves.append(geom_bspline(1, False, short, polyline_knots(len(short))))
    return curves


def polylines2d() -> list[list[float]]:
    base = [(float(i), math.cos(0.5 * i)) for i in range(24)]
    layouts = [[], [0], [12], [3, 15], [23], [11, 11], [5, 6], [0, 23]]
    curves = []
    for dup in layouts:
        poles = with_duplicates(base, dup)
        curves.append(geom2d_bspline(1, False, poles, polyline_knots(len(poles))))
    short = with_duplicates(base[:9], [4])
    curves.append(geom2d_bspline(1, False, short, polyline_knots(len(short))))
    return curves


POLES3 = [(0.0, 0.0, 0.0), (1.0, 2.0, 0.5), (3.0, 0.0, 1.0), (4.0, 1.0, 0.0), (5.0, 2.0, 1.0),
          (6.0, 0.0, 0.0), (7.0, 1.0, 0.5)]
POLES2 = [(x, y) for x, y, _ in POLES3]


def c0_curves3d() -> list[list[float]]:
    return [
        geom_bspline(3, False, POLES3, [(0.0, 4), (1.0, 3), (2.0, 4)]),
        geom_bspline(2, False, POLES3, [(0.0, 3), (1.0, 2), (2.0, 2), (3.0, 3)]),
        geom_bspline(3, False, POLES3, [(0.0, 4), (1.0, 1), (2.0, 1), (3.0, 1), (4.0, 4)]),
        geom_bspline(1, False, POLES3[:3], [(0.0, 2), (1.0, 1), (2.0, 2)]),
    ]


def c0_curves2d() -> list[list[float]]:
    return [
        geom2d_bspline(3, False, POLES2, [(0.0, 4), (1.0, 3), (2.0, 4)]),
        geom2d_bspline(2, False, POLES2, [(0.0, 3), (1.0, 2), (2.0, 2), (3.0, 3)]),
        geom2d_bspline(3, False, POLES2, [(0.0, 4), (1.0, 1), (2.0, 1), (3.0, 1), (4.0, 4)]),
        geom2d_bspline(1, False, POLES2[:3], [(0.0, 2), (1.0, 1), (2.0, 2)]),
    ]


def geom_requests() -> list[str]:
    requests: list[str] = []
    others3d = [geom_bspline(3, False, POLES3[:4], [(0.0, 4), (1.0, 4)]), geom_gp(1, radius=2.0),
                geom_bezier(POLES3[:3])]
    others2d = [geom2d_bspline(2, False, POLES2[:3], [(0.0, 3), (1.0, 3)]), geom2d_gp(0)]
    for curve in polylines3d() + others3d:
        requests.append('sh_algo_geom ' + numbers([0, *curve]))
    for curve in polylines2d() + others2d:
        requests.append('sh_algo_geom ' + numbers([1, *curve]))
    for curve in c0_curves3d() + [geom_gp(0)]:
        requests.append('sh_algo_geom ' + numbers([2, *curve]))
    for curve in c0_curves2d() + [geom2d_gp(1, radius=1.0)]:
        requests.append('sh_algo_geom ' + numbers([3, *curve]))
    for curve, (first, last) in product([geom_gp(1, radius=2.0), geom_gp(2), c0_curves3d()[0]],
                                        [(0.0, 1.0), (0.5, 2.5)]):
        requests.append('sh_algo_geom ' + numbers([4, *curve, first, last, 1e-4, C1, 9, 9]))
    for surface, box in surfaces():
        requests.append('sh_algo_geom ' + numbers([5, *surface, *box]))
    for surface in [ring_bspline(False), ring_bspline(True), [0, 0, *O, *Z, *X]]:
        requests.append('sh_algo_geom ' + numbers([6, *surface]))
    return requests


class WireScene:
    """A prelude and a program with polyline and arc wires built from 3D curves only."""

    def __init__(self) -> None:
        self.prelude = Prelude()
        self.shift = self.prelude.location(translation((0.5, -1.0, 2.0)))
        self.program: ShapeProgram = self.prelude.program()
        self.wires: dict[str, int] = {}

    def segment(self, a, b, start: float = 0.0, scale: float = 1.0, ends: tuple[int, int] | None = None) -> int:
        """An edge on the line from a to b, its range [start, start + |ab| * scale], on the
        vertices `ends` (new ones when None)."""
        p = self.program
        va, vb = ends if ends is not None else (p.vertex(a), p.vertex(b))
        d = [bb - aa for aa, bb in zip(a, b)]
        length = math.sqrt(sum(c * c for c in d))
        unit = [c / length for c in d]
        origin = [aa - start * u for aa, u in zip(a, unit)]
        e = p.edge()
        p.edge_curve(e, p.line(origin, unit))
        p.edge_range(e, start, start + length * scale)
        p.add(e, va, FORWARD)
        p.add(e, vb, REVERSED)
        return e

    def arc(self, center, radius: float, first: float, last: float) -> int:
        p = self.program
        e = p.edge()
        p.edge_curve(e, p.curve(1, *center, *Z, radius))
        p.edge_range(e, first, last)
        cx, cy, cz = center
        p.add(e, p.vertex((cx + radius * math.cos(first), cy + radius * math.sin(first), cz)), FORWARD)
        p.add(e, p.vertex((cx + radius * math.cos(last), cy + radius * math.sin(last), cz)), REVERSED)
        return e

    def wire(self, name: str, edges: list[tuple[int, int]]) -> int:
        w = self.program.container(WIRE)
        for edge, orientation in edges:
            self.program.add(w, edge, orientation)
        self.wires[name] = w
        return w

    def polyline(self, name: str, points: list[tuple], reverse_at: int = -1, start: float = 0.0) -> int:
        edges = []
        vertices = [self.program.vertex(point) for point in points]
        for i in range(len(points) - 1):
            if i == reverse_at:
                ends = (vertices[i + 1], vertices[i])
                edges.append((self.segment(points[i + 1], points[i], start, 1.0, ends), REVERSED))
            else:
                ends = (vertices[i], vertices[i + 1])
                edges.append((self.segment(points[i], points[i + 1], start, 1.0, ends), FORWARD))
        return self.wire(name, edges)

    def tokens(self) -> str:
        return self.prelude.tokens() + ' ' + self.program.tokens()


def wire_scene() -> WireScene:
    s = WireScene()
    s.polyline('a', [(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (1.0, 2.0, 0.0), (4.0, 2.0, 0.0)])
    s.polyline('b', [(0.0, 0.0, 1.0), (2.0, 0.0, 1.0), (2.0, 4.0, 1.0)], start=3.0)
    s.polyline('c', [(0.0, 0.0, 2.0), (5.0, 0.0, 2.0)])
    s.polyline('d', [(0.0, 1.0, 0.0), (2.0, 1.0, 0.0), (2.0, 3.0, 0.0), (3.0, 3.0, 0.0)], reverse_at=1)
    s.wire('e', [(s.arc((0.0, 0.0, 3.0), 2.0, 0.0, 1.5), FORWARD), (s.arc((0.0, 0.0, 3.0), 2.0, 1.5, 3.0), FORWARD)])
    zero = [s.segment((0.0, 0.0, 5.0), (1.0, 0.0, 5.0), 0.0, 0.0) for _ in range(2)]
    s.wire('f', [(e, FORWARD) for e in zero])
    moved = s.program.located(s.segment((0.0, 0.0, 0.0), (3.0, 0.0, 0.0)), s.shift)
    s.wire('g', [(moved, FORWARD), (s.segment((3.5, -1.0, 2.0), (3.5, 1.0, 2.0)), FORWARD)])
    s.polyline('tail', [(4.0, 2.0, 0.0), (5.0, 2.0, 0.0), (5.0, 3.0, 0.0)])
    s.polyline('tail_rev', [(5.0, 3.0, 0.0), (5.0, 2.0, 0.0), (4.0, 2.0, 0.0)])
    s.polyline('head', [(-1.0, -1.0, 0.0), (0.0, -1.0, 0.0), (0.0, 0.0, 0.0)])
    s.polyline('head_rev', [(0.0, 0.0, 0.0), (0.0, -1.0, 0.0), (-1.0, -1.0, 0.0)])
    s.polyline('near', [(4.0005, 2.0, 0.0), (6.0, 2.0, 0.0)])
    s.polyline('far', [(10.0, 10.0, 10.0), (11.0, 10.0, 10.0)])
    s.wire('empty', [])
    return s


def homo_requests(s: WireScene) -> list[str]:
    w = s.wires
    pairs = [('a', 'b'), ('b', 'a'), ('a', 'c'), ('c', 'c'), ('a', 'd'), ('d', 'e'), ('a', 'f'),
             ('f', 'a'), ('e', 'g'), ('a', 'a'), ('g', 'b'), ('c', 'e')]
    return ['sh_algo_shape ' + s.tokens() + ' 1 0 ' + num(w[x]) + ' ' + num(w[y]) for x, y in pairs]


def connect_requests(s: WireScene) -> list[str]:
    w = s.wires
    nexts = ['tail', 'tail_rev', 'head', 'head_rev', 'near', 'far', 'empty']
    requests = []
    for first, nxt, (prec, maxtol) in product(['a', 'empty'], nexts, [(1e-7, 1e-3), (1e-3, 10.0)]):
        fields = [1, w[first], w[nxt], prec, maxtol]
        requests.append('sh_algo_shape ' + s.tokens() + ' 1 ' + ' '.join(num(v) for v in fields))
    return requests


def face_scene() -> ModificationScene:
    """Single-wire faces on a plane, a cylinder and extrusions of C0 and C2 B-splines."""
    pre = Prelude()
    c0 = geom_bspline(3, False, POLES3, [(0.0, 4), (1.0, 3), (2.0, 4)])
    c2 = geom_bspline(3, False, POLES3[:4], [(0.0, 4), (1.0, 4)])
    found = {
        'plane': pre.elementary(PLANE, O, Z, X, False),
        'cyl': pre.elementary(CYLINDER, (0.5, -1.0, 0.0), Z, X, False, 1.5),
        'ext_c0': pre.extrusion(c0, Z),
        'ext_c2': pre.extrusion(c2, (0.0, 0.6, 0.8)),
    }
    scene = ModificationScene(pre, pre.program())
    for name, surface in found.items():
        bounded_face(scene, name, surface)
    return scene


def face_requests() -> list[str]:
    scene = face_scene()
    requests = []
    for name, face in scene.faces.items():
        ops = [2, face, 3, face, 1e-3, 3, face, 1e-7, 3, scene.edges[name][0], 1e-3]
        requests.append('sh_algo_shape ' + scene.tokens() + ' 4 ' + ' '.join(num(v) for v in ops))
    return requests


def append_shape_algo_requests(requests: list[str]) -> None:
    """Append the ShapeAlgo_AlgoContainer requests."""
    requests.extend(geom_requests())
    scene = wire_scene()
    requests.extend(homo_requests(scene))
    requests.extend(connect_requests(scene))
    requests.extend(face_requests())
