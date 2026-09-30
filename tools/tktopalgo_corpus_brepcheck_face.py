"""The BRepCheck_Face requests of the TKTopAlgo corpus: faces without surface or wires,
holed faces (well and badly oriented, intersecting, outside, nested, touching at a
vertex), a wire met twice, thin faces, infinite faces and the four cases of
BRepCheck_Face_Test.cxx built with BRep_Builder."""
from __future__ import annotations
import math
from tktopalgo_corpus_brepcheck import CONTEXTS, FACE, FLAGS, IN_CONTEXT, STATUS, check
from tktopalgo_corpus_brepcheck_shapes import PlanarShapes, square
from tktopalgo_shape_program import AS_IS, FORWARD, REVERSED, SHELL, WIRE, ShapeProgram

INTERSECT, CLASSIFY, ORIENTATION, SET_UNORIENTABLE, UNORIENTABLE, SET_STATUS, SET_GCTRL, GCTRL = range(1, 9)
TWO_PI = 2*math.pi


def face_requests(requests: list[str], p: ShapeProgram, face: int, *contexts: int) -> None:
    """Each wire check (with and without update), in order and alone, and the context."""
    for update in (0, 1):
        requests.append(check(FACE, p, face, (ORIENTATION, update), (CLASSIFY, update), (INTERSECT, update),
                              UNORIENTABLE, STATUS))
    requests.append(check(FACE, p, face, (CLASSIFY, 0), UNORIENTABLE, (SET_UNORIENTABLE,), UNORIENTABLE, STATUS))
    requests.append(check(FACE, p, face, (INTERSECT, 1), (SET_GCTRL, 0), GCTRL, (SET_GCTRL, 1), (INTERSECT, 1),
                          (CLASSIFY, 1), (ORIENTATION, 1), (SET_STATUS, 34), STATUS))
    if contexts:
        requests.append(check(FACE, p, face, *[(IN_CONTEXT, c) for c in contexts], FLAGS, CONTEXTS))


def circle_hole(p: ShapeProgram, plane: int, face: int, centre, radius: float, reverse=True,
                offset=None) -> int:
    """A circular wire on the plane z = 0 (reversed to bound a hole); with `offset`, the
    pcurve is the offset of a smaller circle (BRepCheck_Face_Test, OffsetPCurve)."""
    v = p.vertex((centre[0]+radius, centre[1], 0.0))
    e = p.edge()
    p.edge_curve(e, p.curve(1, centre[0], centre[1], 0, 0, 0, 1, radius))
    if offset is None:
        pc = p.pcurve(1, centre[0], centre[1], radius, 1)
    else:
        pc = p.pcurve(5, p.pcurve(1, centre[0], centre[1], radius-offset, 1), offset)
    p.edge_pcurve(e, pc, plane)
    p.edge_range(e, 0.0, TWO_PI)
    p.add(e, v, FORWARD)
    p.add(e, v, REVERSED)
    wire = p.container(WIRE)
    p.add(wire, e, FORWARD)
    if reverse:
        wire = p.oriented(wire, REVERSED)
    p.add(face, wire, AS_IS)
    return wire


def holed_requests(requests: list[str]) -> None:
    """Square holes: inside and reversed, not reversed, crossing the outer wire,
    outside, nested, touching the outer wire at a vertex; a reversed outer wire."""
    holes = [((1, 1, 1),), ((1, 1, 1), (2.5, 2.5, 1)), ((3, 1, 2),), ((5, 5, 1),), ((1, 1, 2), (1.5, 1.5, 0.5)),
             ((0, 0, 1),)]
    for hole in holes:
        sq = square(ShapeProgram(), 4.0, holes=hole)
        face_requests(requests, sq.p, sq.faces[0])
    for orientations in ([FORWARD, FORWARD], [REVERSED, REVERSED], [REVERSED, FORWARD]):
        sq = square(ShapeProgram(), 4.0, holes=((1, 1, 1),), orientations=orientations)
        face_requests(requests, sq.p, sq.faces[0])
    for reversed_face in (False, True):
        sq = square(ShapeProgram(), 4.0, holes=((1, 1, 1),))
        face = sq.p.oriented(sq.faces[0], REVERSED) if reversed_face else sq.faces[0]
        face_requests(requests, sq.p, face)


def special_requests(requests: list[str]) -> None:
    """No surface, no wire, a wire met twice, two outer wires, an infinite face, a
    face in shells that hold it or not."""
    p = ShapeProgram()
    bare = p.face(-1)
    requests.append(check(FACE, p, bare, STATUS, FLAGS))
    plane = p.surface(0, 0, 0, 0, 0, 0, 1)
    empty = p.face(plane)
    face_requests(requests, p, empty)
    p.flag(empty, 1, True)
    face_requests(requests, p, empty)
    sq = square(ShapeProgram())
    twice = sq.p.face(sq.surfaces[0])
    sq.p.add(twice, sq.wires[0][0], FORWARD)
    sq.p.add(twice, sq.wires[0][0], REVERSED)
    face_requests(requests, sq.p, twice)
    two = PlanarShapes(ShapeProgram(), [(0, 0, 0), (1, 0, 0), (1, 1, 0), (0, 1, 0), (2, 0, 0), (3, 0, 0),
                                        (3, 1, 0), (2, 1, 0)])
    two.face([[0, 1, 2, 3], [4, 5, 6, 7]], normal=(0, 0, 1))
    two.finish()
    face_requests(requests, two.p, two.faces[0])
    sq = square(ShapeProgram())
    shell = sq.p.container(SHELL)
    sq.p.add(shell, sq.faces[0], REVERSED)
    other = sq.p.container(SHELL)
    face_requests(requests, sq.p, sq.faces[0], shell, other, sq.wires[0][0])


def thin_requests(requests: list[str]) -> None:
    """Faces bounded by two arcs between the same vertices (CheckThin)."""
    for second_orientation, sense in ((REVERSED, 1), (FORWARD, 1), (REVERSED, 0)):
        p = ShapeProgram()
        plane = p.surface(0, 0, 0, 0, 0, 0, 1)
        va, vb = p.vertex((1.0, 0.0, 0.0)), p.vertex((-1.0, 0.0, 0.0))
        edges = []
        for k, (start, end) in enumerate(((0.0, math.pi), (math.pi, TWO_PI))):
            e = p.edge()
            p.edge_curve(e, p.curve(1, 0, 0, 0, 0, 0, 1, 1.0))
            p.edge_pcurve(e, p.pcurve(1, 0.0, 0.0, 1.0, sense if k else 1), plane)
            p.edge_range(e, start, end)
            p.add(e, va if k == 0 else vb, FORWARD)
            p.add(e, vb if k == 0 else va, REVERSED)
            edges.append(e)
        wire = p.container(WIRE)
        p.add(wire, edges[0], FORWARD)
        p.add(wire, edges[1], second_orientation)
        face = p.face(plane)
        p.add(face, p.oriented(wire, REVERSED), AS_IS)
        face_requests(requests, p, face)


def gtest_requests(requests: list[str]) -> None:
    """The cases of BRepCheck_Face_Test.cxx: 40 circular holes, an offset pcurve, an
    open wire and an unbounded line outside the face (and the reversed face)."""
    sq = square(ShapeProgram(), 100.0)
    p, plane, face = sq.p, sq.surfaces[0], sq.faces[0]
    for row in range(5):
        for column in range(8):
            circle_hole(p, plane, face, (10.0+11.0*column, 10.0+11.0*row), 2.0)
    requests.append(check(FACE, p, face, (CLASSIFY, 0), STATUS))
    sq = square(ShapeProgram(), 100.0)
    circle_hole(sq.p, sq.surfaces[0], sq.faces[0], (50.0, 50.0), 5.0, offset=1.0)
    requests.append(check(FACE, sq.p, sq.faces[0], (CLASSIFY, 0), STATUS))
    for kind in ('open', 'line'):
        sq = square(ShapeProgram(), 100.0)
        p = sq.p
        e = p.edge()
        if kind == 'open':
            p.edge_curve(e, p.line((120.0, 10.0, 0.0), (1.0, 0.0, 0.0)))
            p.edge_range(e, 0.0, 10.0)
            p.add(e, p.vertex((120.0, 10.0, 0.0)), FORWARD)
            p.add(e, p.vertex((130.0, 10.0, 0.0)), REVERSED)
        else:
            p.edge_curve(e, p.line((0.0, 120.0, 0.0), (1.0, 0.0, 0.0)))
            p.edge_range(e, -2e100, 2e100)
        wire = p.container(WIRE)
        p.add(wire, e, FORWARD)
        p.add(sq.faces[0], wire, FORWARD)
        reversed_face = p.oriented(sq.faces[0], REVERSED)
        for face in (sq.faces[0], reversed_face):
            requests.append(check(FACE, p, face, (CLASSIFY, 0), STATUS))
            face_requests(requests, p, face)


def shared_vertex_requests(requests: list[str]) -> None:
    """Holes that share a vertex with the outer wire (the non-manifold case of
    Intersect): at a corner, and along a part of an outer edge; a hole whose first
    edge is too short to be sampled; half-infinite and Bezier edges."""
    points = [(0, 0, 0), (4, 0, 0), (4, 4, 0), (0, 4, 0), (1, 0.5, 0), (0.5, 1, 0), (3, 0, 0), (3.5, 1, 0),
              (2, 2, 0), (2, 2+1e-10, 0), (2.5, 3, 0), (3, 2, 0)]
    for holes in ([[0, 5, 4]], [[1, 7, 6]], [[0, 5, 4], [1, 7, 6]], [[8, 9, 10, 11]]):
        shapes = PlanarShapes(ShapeProgram(), points)
        shapes.face([[0, 1, 2, 3], *holes], normal=(0.0, 0.0, 1.0))
        shapes.finish()
        face_requests(requests, shapes.p, shapes.faces[0])
    for first, last in ((0.0, 2e100), (-2e100, 0.0)):
        sq = square(ShapeProgram(), 100.0)
        p = sq.p
        e = p.edge()
        p.edge_curve(e, p.line((0.0, 120.0, 0.0), (1.0, 0.0, 0.0)))
        p.edge_range(e, first, last)
        wire = p.container(WIRE)
        p.add(wire, e, FORWARD)
        p.add(sq.faces[0], wire, FORWARD)
        face_requests(requests, p, sq.faces[0])
    sq = square(ShapeProgram(), 4.0)
    p = sq.p
    v = p.vertex((2.0, 1.0, 0.0))
    e = p.edge()
    p.edge_curve(e, p.curve(3, 4, 2, 1, 0, 3, 3, 0, 1, 3, 0, 2, 1, 0))
    p.edge_pcurve(e, p.pcurve(3, 4, 2.0, 1.0, 3.0, 3.0, 1.0, 3.0, 2.0, 1.0), sq.surfaces[0])
    p.edge_range(e, -0.25, 1.25)
    p.add(e, v, FORWARD)
    p.add(e, v, REVERSED)
    wire = p.container(WIRE)
    p.add(wire, e, REVERSED)
    p.add(sq.faces[0], wire, FORWARD)
    face_requests(requests, p, sq.faces[0])


def append_face_requests(requests: list[str]) -> None:
    holed_requests(requests)
    shared_vertex_requests(requests)
    special_requests(requests)
    thin_requests(requests)
    gtest_requests(requests)


__all__ = ['append_face_requests', 'IN_CONTEXT']
