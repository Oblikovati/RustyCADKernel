"""Deterministic requests for BRepFill_CompatibleWires
(tools/oracle/tkbool_probe_brepfill_compatible.cxx; the record is documented there).

Closed families reach ComputeOrigin (sections of one edge count with corners: squares
shifted and turned, two-edge sections with their sample points) and the polar method
(different edge counts: squares, triangles, pentagons and circles, with punctual ends,
looping sections); open families reach SearchOrigin (straight segments, polylines
reversed or not, chords of zero length) and SameNumberByACR (different edge counts,
smooth B-spline sections); mixed open and closed sections give NotSameTopology. Every
family runs with and without rotation and with two percents.
"""
from __future__ import annotations
import math
from tkbool_corpus_brepfill_generator import (bspline_wire, circle_wire, point_wire,
                                              polyline_wire, square, wire_of)
from tkprim_corpus_sweep_shapes import segment
from tktopalgo_shape_program import FORWARD, REVERSED, ShapeProgram


def polygon(z: float, radius: float, sides: int, turn: float = 0.0, center=(0.0, 0.0)):
    """The corners of a regular polygon at height `z`."""
    return [(center[0]+radius*math.cos(turn+2*math.pi*k/sides),
             center[1]+radius*math.sin(turn+2*math.pi*k/sides), z) for k in range(sides)]


def two_edge_wire(p: ShapeProgram, z: float, radius: float, turn: float = 0.0) -> int:
    """A closed section of two edges: the segment between two opposite points of a
    circle of `radius` and a quadratic B-spline back through a third point."""
    a = (radius*math.cos(turn), radius*math.sin(turn), z)
    b = (-radius*math.cos(turn), -radius*math.sin(turn), z)
    c = (radius*math.cos(turn+math.pi/2), radius*math.sin(turn+math.pi/2), z)
    va, vb = p.vertex(a), p.vertex(b)
    e1 = segment(p, a, b, (va, vb))
    e2 = p.edge()
    p.edge_curve(e2, p.curve(5, 2, 3, *b, *c, *a, 2, 0.0, 3, 1.0, 3))
    p.add(e2, vb, FORWARD)
    p.add(e2, va, REVERSED)
    p.edge_range(e2, 0.0, 1.0)
    return wire_of(p, [(e1, FORWARD), (e2, FORWARD)], True)


FAMILIES = {
    'squares_same': lambda p: [polyline_wire(p, square(0.0, 2.0), True), polyline_wire(p, square(1.0, 2.0), True)],
    'squares_turned': lambda p: [polyline_wire(p, square(0.0, 2.0), True),
                                 polyline_wire(p, square(1.0, 2.0, 0.6), True),
                                 polyline_wire(p, square(2.0, 1.5, 1.4, (0.3, 0.1)), True)],
    'squares_reversed': lambda p: [polyline_wire(p, square(0.0, 2.0), True),
                                   polyline_wire(p, square(1.5, 2.0, 0.2), True, True)],
    'two_edges': lambda p: [two_edge_wire(p, 0.0, 1.0), two_edge_wire(p, 1.0, 1.2, 0.5)],
    'square_triangle': lambda p: [polyline_wire(p, square(0.0, 2.0), True),
                                  polyline_wire(p, polygon(1.0, 1.0, 3), True)],
    'triangle_pentagon': lambda p: [polyline_wire(p, polygon(0.0, 1.0, 3, 0.2), True),
                                    polyline_wire(p, polygon(1.0, 1.5, 5), True),
                                    polyline_wire(p, polygon(2.5, 1.0, 4, 0.7), True)],
    'square_circle': lambda p: [polyline_wire(p, square(0.0, 2.0), True), circle_wire(p, (0, 0, 2.0), 1.0)],
    'circles': lambda p: [circle_wire(p, (0, 0, 0), 1.0), circle_wire(p, (0, 0, 1.0), 2.0)],
    'apex_square_triangle': lambda p: [point_wire(p, (0, 0, -1.0)), polyline_wire(p, square(0.0, 2.0), True),
                                       polyline_wire(p, polygon(1.0, 1.0, 3), True), point_wire(p, (0, 0, 2.0))],
    'apex_squares': lambda p: [point_wire(p, (0, 0, -1.0)), polyline_wire(p, square(0.0, 2.0), True),
                               polyline_wire(p, square(1.0, 2.0, 0.3), True), point_wire(p, (0, 0, 2.0))],
    'looping': lambda p: looping(p),
    'open_segments': lambda p: [polyline_wire(p, [(0, 0, 0), (2, 0, 0)], False),
                                polyline_wire(p, [(2, 0.2, 1), (0, 0.1, 1)], False)],
    'open_polylines': lambda p: [polyline_wire(p, [(0, 0, 0), (1, 0, 0), (2, 1, 0)], False),
                                 polyline_wire(p, [(2, 1, 1), (1.5, 0.2, 1.2), (1, 0, 1), (0, 0.2, 1)], False)],
    'open_reversed': lambda p: [polyline_wire(p, [(0, 0, 0), (1, 1, 0), (2, 0, 0)], False),
                                polyline_wire(p, [(0, 0, 1), (1, 1, 1), (2, 0, 1)], False, True)],
    'open_bsplines': lambda p: [bspline_wire(p, [(0, 0, 0), (1, 1, 0), (2, -1, 0), (3, 0, 0)]),
                                polyline_wire(p, [(0, 0, 1), (1.5, 0.5, 1), (3, 0, 1)], False)],
    'open_bent': lambda p: [polyline_wire(p, [(0, 0, 0), (1, 0, 0), (1, 1, 0)], False),
                            polyline_wire(p, [(0, 0, 1), (0.5, 0.2, 1.1), (1, 0, 1), (1, 1, 1)], False)],
    'mixed': lambda p: [polyline_wire(p, square(0.0, 2.0), True), polyline_wire(p, [(0, 0, 1), (1, 0, 1)], False)],
}


def looping(p: ShapeProgram) -> list[int]:
    """Looping sections: the last section is the first one."""
    first = polyline_wire(p, square(0.0, 2.0), True)
    return [first, polyline_wire(p, polygon(1.0, 1.2, 3), True), first]


def append_compatible_requests(requests: list[str]) -> None:
    """Every family with rotation on and off and two percents."""
    for name, builder in sorted(FAMILIES.items()):
        p = ShapeProgram()
        wires = builder(p)
        fields = ' '.join([p.tokens(), str(len(wires)), *map(str, wires)])
        for percent, rotation in ((0.1, 1), (0.01, 0), (1.5, 1)):
            requests.append(f'bf_compatible {fields} {percent!r} {rotation}')
