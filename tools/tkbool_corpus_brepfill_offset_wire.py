"""Deterministic requests for BRepFill_OffsetWire (tools/oracle/tkbool_probe_brepfill_offset_wire.cxx;
the record is documented there).

Plane faces in the XY plane whose p-curves are their edges in UV = (x, y): a rectangle,
a triangle, an L-shape (a concave vertex), a square with a square hole, a disc (one
closed circle), a stadium (lines and half circles) and a face with a B-spline edge; each
is offset outward and inward with the arc and intersection joins, at a null offset, and
moved along the normal; an open polyline face gives open results.
"""
from __future__ import annotations
import math
from tktopalgo_corpus_brepcheck_shapes import PlanarShapes
from tktopalgo_shape_program import FORWARD, REVERSED, WIRE, ShapeProgram

ARC, INTERSECTION = 0, 2


def polygon_face(p: ShapeProgram, points, holes=()) -> int:
    """A face bounded by the polygon (counter-clockwise) and clockwise hole polygons,
    its wires flagged Closed."""
    all_points = list(points)
    loops = [list(range(len(points)))]
    for hole in holes:
        base = len(all_points)
        all_points += hole
        loops.append(list(range(base, base+len(hole))))
    shapes = PlanarShapes(p, [(x, y, 0.0) for x, y in all_points])
    face = shapes.face(loops, normal=(0.0, 0.0, 1.0))
    shapes.finish()
    for wire in shapes.wires[-1]:
        p.flag(wire, 0, True)
    return face


def xy_plane(p: ShapeProgram) -> int:
    """The XY plane, UV = (x, y)."""
    return p.plane((0.0, 0.0, 0.0), (0.0, 0.0, 1.0), (1.0, 0.0, 0.0))


def arc_edge(p: ShapeProgram, surface: int, centre, radius: float, span, vertices) -> int:
    """A counter-clockwise circle arc edge on the XY plane with its p-curve."""
    e = p.edge()
    p.edge_curve(e, p.curve(1, centre[0], centre[1], 0.0, 0.0, 0.0, 1.0, radius))
    p.add(e, vertices[0], FORWARD)
    p.add(e, vertices[1], REVERSED)
    p.edge_pcurve(e, p.pcurve(1, centre[0], centre[1], radius, 1), surface)
    p.edge_range(e, span[0], span[1])
    return e


def line_edge(p: ShapeProgram, surface: int, a, b, vertices) -> int:
    """A segment edge on the XY plane with its p-curve."""
    length = math.dist(a, b)
    d = ((b[0]-a[0])/length, (b[1]-a[1])/length)
    e = p.edge()
    p.edge_curve(e, p.line((a[0], a[1], 0.0), (d[0], d[1], 0.0)))
    p.add(e, vertices[0], FORWARD)
    p.add(e, vertices[1], REVERSED)
    p.edge_pcurve(e, p.line2d(a, d), surface)
    p.edge_range(e, 0.0, length)
    return e


def face_of_edges(p: ShapeProgram, surface: int, edges, closed: bool = True) -> int:
    """The face of one wire of the edges (FORWARD)."""
    wire = p.container(WIRE)
    for e in edges:
        p.add(wire, e, FORWARD)
    if closed:
        p.flag(wire, 0, True)
    face = p.face(surface)
    p.add(face, wire, FORWARD)
    return face


def disc(p: ShapeProgram) -> int:
    """A disc of radius 2 around (1, 1): one closed circle edge."""
    s = xy_plane(p)
    v = p.vertex((3.0, 1.0, 0.0))
    return face_of_edges(p, s, [arc_edge(p, s, (1.0, 1.0), 2.0, (0.0, 2.0*math.pi), (v, v))])


def stadium(p: ShapeProgram) -> int:
    """Two segments and two half circles of radius 1 around (0, 0) and (4, 0)."""
    s = xy_plane(p)
    pts = [(0.0, -1.0), (4.0, -1.0), (4.0, 1.0), (0.0, 1.0)]
    v = [p.vertex((x, y, 0.0)) for x, y in pts]
    edges = [line_edge(p, s, pts[0], pts[1], (v[0], v[1])),
             arc_edge(p, s, (4.0, 0.0), 1.0, (-0.5*math.pi, 0.5*math.pi), (v[1], v[2])),
             line_edge(p, s, pts[2], pts[3], (v[2], v[3])),
             arc_edge(p, s, (0.0, 0.0), 1.0, (0.5*math.pi, 1.5*math.pi), (v[3], v[0]))]
    return face_of_edges(p, s, edges)


def bump(p: ShapeProgram) -> int:
    """A rectangle whose top side is a cubic Bezier bump (a B-spline-like edge)."""
    s = xy_plane(p)
    pts = [(0.0, 0.0), (4.0, 0.0), (4.0, 2.0), (0.0, 2.0)]
    v = [p.vertex((x, y, 0.0)) for x, y in pts]
    poles = [(4.0, 2.0), (3.0, 4.0), (1.0, 0.5), (0.0, 2.0)]
    top = p.edge()
    p.edge_curve(top, p.curve(3, 4, *[c for x, y in poles for c in (x, y, 0.0)]))
    p.add(top, v[2], FORWARD)
    p.add(top, v[3], REVERSED)
    p.edge_pcurve(top, p.pcurve(3, 4, *[c for x, y in poles for c in (x, y)]), s)
    p.edge_range(top, 0.0, 1.0)
    edges = [line_edge(p, s, pts[0], pts[1], (v[0], v[1])), line_edge(p, s, pts[1], pts[2], (v[1], v[2])),
             top, line_edge(p, s, pts[3], pts[0], (v[3], v[0]))]
    return face_of_edges(p, s, edges)


def open_polyline(p: ShapeProgram) -> int:
    """A plane face bounded by an open polyline of three segments."""
    s = xy_plane(p)
    pts = [(0.0, 0.0), (3.0, 0.0), (3.0, 2.0), (5.0, 3.0)]
    v = [p.vertex((x, y, 0.0)) for x, y in pts]
    return face_of_edges(p, s, [line_edge(p, s, pts[i], pts[i+1], (v[i], v[i+1])) for i in range(3)], False)


FACES = {
    'rectangle': lambda p: polygon_face(p, [(0.0, 0.0), (4.0, 0.0), (4.0, 3.0), (0.0, 3.0)]),
    'triangle': lambda p: polygon_face(p, [(0.0, 0.0), (5.0, 0.0), (1.0, 3.0)]),
    'l_shape': lambda p: polygon_face(p, [(0.0, 0.0), (6.0, 0.0), (6.0, 2.0), (2.0, 2.0), (2.0, 5.0), (0.0, 5.0)]),
    'holed': lambda p: polygon_face(p, [(0.0, 0.0), (10.0, 0.0), (10.0, 8.0), (0.0, 8.0)],
                                    [[(2.0, 2.0), (2.0, 5.0), (4.0, 5.0), (4.0, 2.0)]]),
    'disc': disc,
    'stadium': stadium,
    'bump': bump,
}


def request(name: str, join: int, offset: float, alt: float = 0.0, is_open: int = 0) -> str:
    """One offset of a face."""
    p = ShapeProgram()
    face = open_polyline(p) if name == 'open' else FACES[name](p)
    return f'bf_offset_wire {p.tokens()} {face} {join} {is_open} {offset} {alt}'


def append_offset_wire_requests(requests: list[str]) -> None:
    """Every closed face outward and inward with both joins, null and lifted; the open
    polyline with an open result."""
    for name in sorted(FACES):
        for join in (ARC, INTERSECTION):
            for offset in (0.5, -0.3, 1.5):
                requests.append(request(name, join, offset))
        requests.append(request(name, ARC, 0.0, 1.0))
        requests.append(request(name, ARC, 0.4, 0.7))
    for join in (ARC, INTERSECTION):
        for offset in (0.5, -0.5):
            requests.append(request('open', join, offset, 0.0, 1))
