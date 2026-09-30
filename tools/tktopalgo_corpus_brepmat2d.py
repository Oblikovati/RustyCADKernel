"""Deterministic requests for BRepMAT2d (tools/oracle/tktopalgo_probe_brepmat2d.cxx).

Every request carries its face record (tktopalgo_corpus_shapes.py): a plane face in the
XY plane whose p-curves are the edges in UV = (x, y). The faces: a square, a triangle, an
L-shape (a concave vertex), a stadium (lines and half circles), a square with a
square hole, with a circular hole and with both, faces whose p-curves leave gaps above
1e-8 between consecutive edges (a line, a half circle after a line or after a half
circle, a REVERSED edge and the closing gap), an open polyline and a REVERSED square. `bm2_locus` sets the Closed flags of the
wires, then runs the explorer, the bisecting locus on both sides with the arc and the
intersection joins (from the first or the second contour, open result for open wires) and
the topological links.
"""
from __future__ import annotations
import math
from tktopalgo_corpus_brepbndlib import numbers
from tktopalgo_corpus_shapes import CIRCLE, FACE, FORWARD, LINE, PCIRCLE, PLANE, PLINE, REVERSED, FaceRecord, ShapeSpec

XY_PLANE = [PLANE, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0]
ARC, INTERSECTION = 0, 2


def segment(spec: ShapeSpec, a: int, b: int, shift: tuple[float, float] = (0.0, 0.0)):
    """A line edge from vertex a to b and its use; `shift` moves the p-curve (a gap)."""
    pa, pb = spec.vertices[a][:3], spec.vertices[b][:3]
    length = math.dist(pa, pb)
    d = [(pb[i] - pa[i]) / length for i in range(3)]
    edge = spec.edge([LINE, *pa, *d], a, b, (0.0, length))
    return edge, FORWARD, [PLINE, pa[0] + shift[0], pa[1] + shift[1], d[0], d[1]]


def arc(spec: ShapeSpec, a: int, b: int, centre: list[float], radius: float, span: tuple[float, float],
        shift: tuple[float, float] = (0.0, 0.0)):
    """A counterclockwise circle arc edge from vertex a to b over `span` and its use."""
    edge = spec.edge([CIRCLE, centre[0], centre[1], 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, radius], a, b, span)
    return edge, FORWARD, [PCIRCLE, centre[0] + shift[0], centre[1] + shift[1], radius, 1]


def polygon(spec: ShapeSpec, points: list[tuple[float, float]], shifts: dict[int, tuple[float, float]] | None = None):
    """The uses of a closed polygon through `points` (its own vertices)."""
    ids = [spec.vertex([x, y, 0.0]) for x, y in points]
    shifts = shifts or {}
    return [segment(spec, ids[i], ids[(i + 1) % len(ids)], shifts.get(i, (0.0, 0.0))) for i in range(len(ids))]


def make_face(spec: ShapeSpec, wires: list[list], orientation: int = FORWARD) -> ShapeSpec:
    """Adds the face of `wires` on the XY plane to `spec` as its only item."""
    face = FaceRecord(list(XY_PLANE))
    for uses in wires:
        face.wire(uses)
    face.orientation = orientation
    spec.face(face)
    spec.top = [(FACE, 0)]
    return spec


def square(shifts: dict[int, tuple[float, float]] | None = None, orientation: int = FORWARD) -> ShapeSpec:
    spec = ShapeSpec()
    return make_face(spec, [polygon(spec, [(0.0, 0.0), (4.0, 0.0), (4.0, 3.0), (0.0, 3.0)], shifts)], orientation)


def triangle() -> ShapeSpec:
    spec = ShapeSpec()
    return make_face(spec, [polygon(spec, [(0.0, 0.0), (5.0, 0.0), (1.0, 3.0)])])


def l_shape() -> ShapeSpec:
    spec = ShapeSpec()
    points = [(0.0, 0.0), (6.0, 0.0), (6.0, 2.0), (2.0, 2.0), (2.0, 5.0), (0.0, 5.0)]
    return make_face(spec, [polygon(spec, points)])


def stadium(arc_shift: tuple[float, float] = (0.0, 0.0)) -> ShapeSpec:
    """Two segments and two half circles of radius 1 around (0, 0) and (4, 0)."""
    spec = ShapeSpec()
    ids = [spec.vertex(p) for p in ([0.0, -1.0, 0.0], [4.0, -1.0, 0.0], [4.0, 1.0, 0.0], [0.0, 1.0, 0.0])]
    uses = [segment(spec, ids[0], ids[1]),
            arc(spec, ids[1], ids[2], [4.0, 0.0], 1.0, (-0.5 * math.pi, 0.5 * math.pi), arc_shift),
            segment(spec, ids[2], ids[3]),
            arc(spec, ids[3], ids[0], [0.0, 0.0], 1.0, (0.5 * math.pi, 1.5 * math.pi))]
    return make_face(spec, [uses])


def circle_use(spec: ShapeSpec, centre: list[float], radius: float, orientation: int):
    """A full circle edge with one vertex, used in `orientation`."""
    v = spec.vertex([centre[0] + radius, centre[1], 0.0])
    edge, _, pcurve = arc(spec, v, v, centre, radius, (0.0, 2.0 * math.pi))
    return edge, orientation, pcurve


def disc() -> ShapeSpec:
    spec = ShapeSpec()
    return make_face(spec, [[circle_use(spec, [1.0, 1.0], 2.0, FORWARD)]])


def circle_halves(shift: tuple[float, float] = (0.0, 0.0)) -> ShapeSpec:
    """A disc bounded by two half circles of radius 2; `shift` moves the p-curve of the
    second one (gaps closed through the B-spline form of a circle)."""
    spec = ShapeSpec()
    a, b = spec.vertex([2.0, 0.0, 0.0]), spec.vertex([-2.0, 0.0, 0.0])
    uses = [arc(spec, a, b, [0.0, 0.0], 2.0, (0.0, math.pi)),
            arc(spec, b, a, [0.0, 0.0], 2.0, (math.pi, 2.0 * math.pi), shift)]
    return make_face(spec, [uses])


def holed(square_hole: bool, circle_hole: bool) -> ShapeSpec:
    """A 10 x 8 square with a clockwise square hole and/or a circular hole."""
    spec = ShapeSpec()
    wires = [polygon(spec, [(0.0, 0.0), (10.0, 0.0), (10.0, 8.0), (0.0, 8.0)])]
    if square_hole:
        wires.append(polygon(spec, [(2.0, 2.0), (2.0, 5.0), (4.0, 5.0), (4.0, 2.0)]))
    if circle_hole:
        wires.append([circle_use(spec, [7.0, 4.0], 1.5, REVERSED)])
    return make_face(spec, wires)


def reversed_edge_gap() -> ShapeSpec:
    """A triangle whose second edge runs backwards (used REVERSED) with a shifted p-curve."""
    spec = ShapeSpec()
    ids = [spec.vertex(p) for p in ([0.0, 0.0, 0.0], [5.0, 0.0, 0.0], [1.0, 3.0, 0.0])]
    e0 = segment(spec, ids[0], ids[1])
    edge, _, pcurve = segment(spec, ids[2], ids[1], (0.0, 2e-6))
    e2 = segment(spec, ids[2], ids[0])
    return make_face(spec, [[e0, (edge, REVERSED, pcurve), e2]])


def open_polyline() -> ShapeSpec:
    """A plane face bounded by an open polyline of three segments."""
    spec = ShapeSpec()
    ids = [spec.vertex(p) for p in ([0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [3.0, 2.0, 0.0], [5.0, 3.0, 0.0])]
    return make_face(spec, [[segment(spec, ids[i], ids[i + 1]) for i in range(3)]])


def faces() -> list[tuple[ShapeSpec, list[int]]]:
    """Every face with the Closed flags of its wires. The disc (one closed circle) is left
    out: the MAT2d computation of a one-curve contour crashes natively."""
    return [
        (square(), [1]),
        (triangle(), [1]),
        (l_shape(), [1]),
        (stadium(), [1]),
        (holed(True, False), [1, 1]),
        (holed(False, True), [1, 1]),
        (holed(True, True), [1, 1, 1]),
        (square({1: (1e-5, 0.0)}), [1]),
        (square({0: (0.0, 3e-6)}), [1]),
        (stadium((2e-6, 0.0)), [1]),
        (circle_halves(), [1]),
        (circle_halves((3e-6, -1e-6)), [1]),
        (reversed_edge_gap(), [1]),
        (square(orientation=REVERSED), [1]),
        (square(), [0]),
        (open_polyline(), [0]),
    ]


def request(spec: ShapeSpec, closed: list[int], side: int, join: int, open_result: int, line: int) -> str:
    return 'bm2_locus ' + numbers([*spec.numbers(), len(closed), *closed, side, join, open_result, line, 1])


def append_brepmat2d_requests(requests: list[str]) -> None:
    """Append every `bm2_locus` request: each face on both sides with both joins (and
    the open result for the open wires), and the holed faces from their second contour."""
    for spec, closed in faces():
        opens = (0, 1) if 0 in closed else (0,)
        for side in (0, 1):
            for join in (ARC, INTERSECTION):
                for open_result in opens:
                    requests.append(request(spec, closed, side, join, open_result, 1))
        if len(closed) > 1:
            requests.append(request(spec, closed, 0, ARC, 0, 2))
