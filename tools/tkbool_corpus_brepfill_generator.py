"""Deterministic requests for BRepFill_Generator (tools/oracle/tkbool_probe_brepfill_generator.cxx;
the record is documented there).

Each family is a list of section wires built as one shape program
(tktopalgo_shape_program.py): coaxial circles (cylinders, cones, a torus that is no
particular case), arcs of equal and different spans, punctual sections (apex first,
last, both: a closed shell), squares (planes) and squares of other sizes or turned
(general ruled surfaces), open polylines, B-spline sections, edges stored REVERSED,
located edges, an edge without 3d curve (the Null3DCurve status), wires of different
edge counts, one wire and no wire. Every family runs with a mutable and an immutable
input.
"""
from __future__ import annotations
import math
from tkprim_corpus_sweep_shapes import bspline, segment
from tktopalgo_shape_program import FORWARD, REVERSED, WIRE, ShapeProgram

Point = tuple[float, float, float]


def wire_of(p: ShapeProgram, edges: list[tuple[int, int]], closed: bool = False) -> int:
    """A wire of `(edge, orientation)` pairs, flagged Closed when asked."""
    wire = p.container(WIRE)
    for edge, orientation in edges:
        p.add(wire, edge, orientation)
    if closed:
        p.flag(wire, 0, True)
    return wire


def circle_wire(p: ShapeProgram, center: Point, radius: float, normal: Point = (0.0, 0.0, 1.0)) -> int:
    """The wire of one closed circle edge (one vertex at the parameter 0)."""
    e = p.edge()
    p.edge_curve(e, p.curve(1, *center, *normal, radius))
    v = p.vertex(circle_point(center, radius, normal, 0.0))
    p.add(e, v, FORWARD)
    p.add(e, v, REVERSED)
    p.edge_range(e, 0.0, 2*math.pi)
    return wire_of(p, [(e, FORWARD)], True)


def circle_point(center: Point, radius: float, normal: Point, t: float) -> Point:
    """The point of parameter `t` of a circle of normal +Z (X direction +X) or +X (gp_Ax2
    then takes the X direction +Z and the Y direction -Y)."""
    if normal == (0.0, 0.0, 1.0):
        return (center[0]+radius*math.cos(t), center[1]+radius*math.sin(t), center[2])
    return (center[0], center[1]-radius*math.sin(t), center[2]+radius*math.cos(t))


def arc_wire(p: ShapeProgram, center: Point, radius: float, first: float, last: float, orientation: int = FORWARD) -> int:
    """The wire of one arc of a +Z circle between two angles."""
    e = p.edge()
    p.edge_curve(e, p.curve(1, *center, 0.0, 0.0, 1.0, radius))
    p.add(e, p.vertex(circle_point(center, radius, (0.0, 0.0, 1.0), first)), FORWARD)
    p.add(e, p.vertex(circle_point(center, radius, (0.0, 0.0, 1.0), last)), REVERSED)
    p.edge_range(e, first, last)
    return wire_of(p, [(e, orientation)])


def point_wire(p: ShapeProgram, point: Point, with_range: bool = True) -> int:
    """A punctual section: one degenerated edge without 3d curve on one vertex."""
    v = p.vertex(point)
    e = p.edge()
    p.add(e, v, FORWARD)
    p.add(e, v, REVERSED)
    p.edge_flags(e, True, True, True)
    if with_range:
        p.edge_range(e, 0.0, 2*math.pi)
    return wire_of(p, [(e, FORWARD)], True)


def polyline_wire(p: ShapeProgram, points: list[Point], closed: bool, reversed_edges: bool = False) -> int:
    """The wire of the segments between `points`; with `reversed_edges` each segment is
    stored from its end to its start and added REVERSED."""
    vertices = [p.vertex(point) for point in points]
    count = len(points) if closed else len(points)-1
    edges = []
    for k in range(count):
        j = (k+1) % len(points)
        if reversed_edges:
            edges.append((segment(p, points[j], points[k], (vertices[j], vertices[k])), REVERSED))
        else:
            edges.append((segment(p, points[k], points[j], (vertices[k], vertices[j])), FORWARD))
    return wire_of(p, edges, closed)


def square(z: float, size: float, turn: float = 0.0, center=(0.0, 0.0)) -> list[Point]:
    """The corners of a square of `size` at height `z`, turned by `turn` radians."""
    h = size/2
    corners = [(-h, -h), (h, -h), (h, h), (-h, h)]
    c, s = math.cos(turn), math.sin(turn)
    return [(center[0]+c*x-s*y, center[1]+s*x+c*y, z) for x, y in corners]


def bspline_wire(p: ShapeProgram, poles: list[Point]) -> int:
    """The wire of one cubic B-spline edge."""
    return wire_of(p, [(bspline(p, poles), FORWARD)])


def located_segment_wire(p: ShapeProgram, a: Point, b: Point, dz: float) -> int:
    """The wire of a segment located by a translation of `dz` along Z."""
    e = segment(p, a, b)
    located = p.located(e, p.location(0, 0.0, 0.0, dz))
    return wire_of(p, [(located, FORWARD)])


def curveless_wire(p: ShapeProgram, a: Point, b: Point) -> int:
    """The wire of an edge without 3d curve (not degenerated)."""
    e = p.edge()
    p.add(e, p.vertex(a), FORWARD)
    p.add(e, p.vertex(b), REVERSED)
    p.edge_range(e, 0.0, 1.0)
    return wire_of(p, [(e, FORWARD)])


def family(builder):
    """A family builder: the program and the table indices of its wires."""
    def build():
        p = ShapeProgram()
        wires = builder(p)
        return p, wires
    return build


FAMILIES = {
    'cylinder': family(lambda p: [circle_wire(p, (0, 0, 0), 1.0), circle_wire(p, (0, 0, 2.0), 1.0)]),
    'cone': family(lambda p: [circle_wire(p, (0, 0, 0), 1.0), circle_wire(p, (0, 0, 3.0), 2.0)]),
    'cylinder_cone': family(lambda p: [circle_wire(p, (0, 0, 0), 1.0), circle_wire(p, (0, 0, 1.0), 1.0),
                                       circle_wire(p, (0, 0, 2.5), 0.5)]),
    'downward_cone': family(lambda p: [circle_wire(p, (0, 0, 2.0), 1.5), circle_wire(p, (0, 0, 0.0), 0.5)]),
    'torus': family(lambda p: [circle_wire(p, (0, 0, 0), 1.0), circle_wire(p, (3.0, 0, 0), 1.0, (1.0, 0.0, 0.0))]),
    'offset_circles': family(lambda p: [circle_wire(p, (0, 0, 0), 1.0), circle_wire(p, (0.5, 0, 2.0), 1.0)]),
    'apex_last': family(lambda p: [circle_wire(p, (0, 0, 0), 1.0), point_wire(p, (0, 0, 2.0))]),
    'apex_first': family(lambda p: [point_wire(p, (0, 0, -1.0)), circle_wire(p, (0, 0, 1.0), 2.0)]),
    'apex_off_axis': family(lambda p: [circle_wire(p, (0, 0, 0), 1.0), point_wire(p, (0.5, 0, 2.0))]),
    'closed_ends': family(lambda p: [point_wire(p, (0, 0, -1.0)), circle_wire(p, (0, 0, 0), 1.0),
                                     circle_wire(p, (0, 0, 1.0), 1.0), point_wire(p, (0, 0, 2.0))]),
    'arcs_same_span': family(lambda p: [arc_wire(p, (0, 0, 0), 1.0, 0.0, 1.5), arc_wire(p, (0, 0, 1.0), 1.0, 0.0, 1.5)]),
    'arcs_cone': family(lambda p: [arc_wire(p, (0, 0, 0), 1.0, 0.5, 2.0), arc_wire(p, (0, 0, 1.0), 2.0, 0.5, 2.0)]),
    'arcs_other_span': family(lambda p: [arc_wire(p, (0, 0, 0), 1.0, 0.0, 1.5), arc_wire(p, (0, 0, 1.0), 1.0, 0.0, 2.5)]),
    'arcs_reversed': family(lambda p: [arc_wire(p, (0, 0, 0), 1.0, 0.0, 1.5, REVERSED),
                                       arc_wire(p, (0, 0, 2.0), 1.0, 0.0, 1.5, REVERSED)]),
    'squares': family(lambda p: [polyline_wire(p, square(0.0, 2.0), True), polyline_wire(p, square(1.0, 2.0), True)]),
    'squares_three': family(lambda p: [polyline_wire(p, square(0.0, 2.0), True), polyline_wire(p, square(1.0, 2.0), True),
                                       polyline_wire(p, square(2.0, 2.0, center=(0.5, 0.0)), True)]),
    'squares_sizes': family(lambda p: [polyline_wire(p, square(0.0, 2.0), True), polyline_wire(p, square(1.5, 1.0), True)]),
    'squares_turned': family(lambda p: [polyline_wire(p, square(0.0, 2.0), True),
                                        polyline_wire(p, square(1.0, 2.0, 0.3), True)]),
    'squares_reversed': family(lambda p: [polyline_wire(p, square(0.0, 2.0), True, True),
                                          polyline_wire(p, square(1.0, 2.0), True, True)]),
    'square_apex': family(lambda p: [polyline_wire(p, square(0.0, 2.0), True), point_wire(p, (0, 0, 1.5))]),
    'apex_square': family(lambda p: [point_wire(p, (0, 0, -1.5)), polyline_wire(p, square(0.0, 2.0), True)]),
    'open_polylines': family(lambda p: [polyline_wire(p, [(0, 0, 0), (1, 0, 0), (2, 0.5, 0)], False),
                                        polyline_wire(p, [(0, 1, 1), (1, 1, 1), (2, 1.5, 1.5)], False)]),
    'bsplines': family(lambda p: [bspline_wire(p, [(0, 0, 0), (1, 1, 0), (2, -1, 0), (3, 0, 0)]),
                                  bspline_wire(p, [(0, 0, 2), (1, 2, 2.5), (2, -2, 2), (3, 0, 2.2)])]),
    'segment_bspline': family(lambda p: [polyline_wire(p, [(0, 0, 0), (3, 0, 0)], False),
                                         bspline_wire(p, [(0, 0, 1), (1, 1, 1), (2, -1, 1), (3, 0, 1)])]),
    'segment_apex': family(lambda p: [polyline_wire(p, [(0, 0, 0), (2, 0, 0)], False), point_wire(p, (1, 1, 1))]),
    'apex_segment': family(lambda p: [point_wire(p, (1, 1, -1)), polyline_wire(p, [(0, 0, 0), (2, 0, 0)], False)]),
    'located_segments': family(lambda p: [located_segment_wire(p, (0, 0, 0), (1, 0, 0), 0.0),
                                          located_segment_wire(p, (0, 0, 0), (1, 0, 0), 1.5)]),
    'curveless': family(lambda p: [polyline_wire(p, [(0, 0, 0), (1, 0, 0)], False), curveless_wire(p, (0, 0, 1), (1, 0, 1))]),
    'curveless_first': family(lambda p: [curveless_wire(p, (0, 0, 0), (1, 0, 0)), polyline_wire(p, [(0, 0, 1), (1, 0, 1)], False)]),
    'edge_counts': family(lambda p: [polyline_wire(p, [(0, 0, 0), (1, 0, 0), (2, 0, 0)], False),
                                     polyline_wire(p, [(0, 1, 1), (2, 1, 1)], False)]),
    'one_wire': family(lambda p: [circle_wire(p, (0, 0, 0), 1.0)]),
    'no_wire': family(lambda p: [] if p.vertex((0.0, 0.0, 0.0)) < 0 else []),
}


def append_generator_requests(requests: list[str]) -> None:
    """Every family with a mutable and an immutable input."""
    for name, builder in sorted(FAMILIES.items()):
        program, wires = builder()
        fields = ' '.join([program.tokens(), str(len(wires)), *map(str, wires)])
        for mutable in (1, 0):
            requests.append(f'bf_generator {fields} {mutable}')
