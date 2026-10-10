"""Deterministic requests for BRepFill_Edge3DLaw, BRepFill_DraftLaw and
BRepFill_EdgeOnSurfLaw (tools/oracle/tkbool_probe_brepfill_location_law.cxx; the record is
documented there).

The paths are a segment, a bent polyline, a closed square, a tangent arc after a segment,
a B-spline and a polyline with REVERSED edges; each runs with the corrected Frenet, the
Frenet and the constant binormal trihedrons and as a draft law. The surface laws run on
the boundary of a square face (on the face, and on a foreign face: no result).
"""
from __future__ import annotations
import math
from tkbool_corpus_brepfill_generator import bspline_wire, polyline_wire, square, wire_of
from tkprim_corpus_sweep_shapes import segment
from tktopalgo_corpus_brepcheck_shapes import square as square_face
from tktopalgo_shape_program import FORWARD, REVERSED, ShapeProgram


def section(p: ShapeProgram) -> int:
    """The swept section: a small square wire around the origin."""
    return polyline_wire(p, square(0.0, 0.2), True)


def arc_after_segment(p: ShapeProgram) -> int:
    """A segment along X then the tangent quarter arc turning to +Y."""
    v0, v1, v2 = p.vertex((0, 0, 0)), p.vertex((1, 0, 0)), p.vertex((2, 1, 0))
    s = segment(p, (0, 0, 0), (1, 0, 0), (v0, v1))
    a = p.edge()
    p.edge_curve(a, p.curve(1, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0))
    p.add(a, v1, FORWARD)
    p.add(a, v2, REVERSED)
    p.edge_range(a, -math.pi/2, 0.0)
    return wire_of(p, [(s, FORWARD), (a, FORWARD)])


PATHS = {
    'segment': lambda p: polyline_wire(p, [(0, 0, 0), (0, 0, 3)], False),
    'bent': lambda p: polyline_wire(p, [(0, 0, 0), (0, 0, 2), (1, 0, 3), (1, 1, 4)], False),
    'closed_square': lambda p: polyline_wire(p, square(0.0, 4.0), True),
    'arc_after_segment': arc_after_segment,
    'bspline': lambda p: bspline_wire(p, [(0, 0, 0), (1, 1, 1), (2, -1, 2), (3, 0, 3)]),
    'reversed': lambda p: polyline_wire(p, [(0, 0, 0), (1, 0, 1), (2, 0, 1)], False, True),
}


def append_location_law_requests(requests: list[str]) -> None:
    """Every path with the three trihedrons and as a draft law; the surface laws."""
    for name, builder in sorted(PATHS.items()):
        p = ShapeProgram()
        path = builder(p)
        sec = section(p)
        for trihedron in (0, 1, 2):
            requests.append(f'bf_location_law {p.tokens()} {path} {sec} 0 {trihedron}')
        requests.append(f'bf_location_law {p.tokens()} {path} {sec} 1 0.0 0.0 1.0 0.2')
    for name in ('segment', 'bent'):
        p = ShapeProgram()
        path = PATHS[name](p)
        sec = section(p)
        guide = polyline_wire(p, [(1, 0, 0), (1.5, 0, 2), (2, 1, 4)], False)
        requests.append(f'bf_location_law {p.tokens()} {path} {sec} 3 {guide}')
    p = ShapeProgram()
    face_shapes = square_face(p, 2.0)
    other = square_face(p, 1.0).faces[0]
    sec = section(p)
    boundary = face_shapes.wires[0][0]
    for support in (face_shapes.faces[0], other):
        requests.append(f'bf_location_law {p.tokens()} {boundary} {sec} 2 {support}')
