"""The BRepCheck_Vertex requests of the TKTopAlgo corpus: a vertex in the context of its
edges (the ends of the 3d curve, the points on curves and pcurves), of its faces (the
points on surfaces), of shapes that do not hold it, and its Tolerance."""
from __future__ import annotations
from tktopalgo_corpus_brepcheck import (CONTEXTS, FLAGS, INIT, IN_CONTEXT, IS_STATUS_ON, PARALLEL,
                                        STATUS, STATUS_ON, VERTEX, check, contexts_script)
from tktopalgo_corpus_brepcheck_shapes import square
from tktopalgo_shape_program import FORWARD, INTERNAL, REVERSED, WIRE, ShapeProgram

TOLERANCE = 1


def segment(p: ShapeProgram, start, end, v_tol=1e-7, e_tol=1e-7, location=0):
    """An edge on the line from `start` towards `end` (range [0, 1]) with its vertices."""
    va, vb = p.vertex(start, v_tol), p.vertex(end, v_tol)
    e = p.edge()
    direction = tuple(b-a for a, b in zip(start, end))
    p.edge_curve(e, p.curve(0, *start, *direction), location, e_tol)
    p.edge_range(e, 0.0, 1.0)
    p.add(e, va, FORWARD)
    p.add(e, vb, REVERSED)
    return va, vb, e


def edge_contexts(requests: list[str]) -> None:
    """The ends of a segment: exact, beyond the vertex tolerance, within the edge one;
    an INTERNAL vertex is not checked at an end."""
    for end, v_tol, e_tol in (((1, 0, 0), 1e-7, 1e-7), ((1, 1e-4, 0), 1e-3, 1e-7), ((1, 1, 0), 1e-7, 1e-7),
                              ((2, 3, -1), 0.5, 1e-7), ((1, 0, 1), 1e-7, 2.0), ((1, 0, 1e-7), 1e-7, 1e-7)):
        p = ShapeProgram()
        va, vb, e = segment(p, (0, 0, 0), (1, 0, 0), v_tol, e_tol)
        moved = p.vertex(end, v_tol)
        inner = p.vertex(end, v_tol)
        e2 = p.edge()
        p.edge_curve(e2, p.curve(0, 0, 0, 0, 1, 0, 0), 0, e_tol)
        p.edge_range(e2, 0.0, 1.0)
        p.add(e2, va, FORWARD)
        p.add(e2, moved, REVERSED)
        p.add(e2, inner, INTERNAL)
        for v in (va, vb, moved, inner):
            requests.append(check(VERTEX, p, v, *contexts_script(e, e2), (TOLERANCE,)))
    # A closed edge: one vertex FORWARD and REVERSED (both ends are checked).
    for gap in (0.0, 1e-3):
        p = ShapeProgram()
        v = p.vertex((1, 0, 0), 1e-7)
        e = p.edge()
        p.edge_curve(e, p.curve(1, 0, 0, gap, 0, 0, 1, 1.0))
        p.edge_range(e, 0.0, 6.283185307179586)
        p.add(e, v, FORWARD)
        p.add(e, v, REVERSED)
        requests.append(check(VERTEX, p, v, *contexts_script(e)))


def located_contexts(requests: list[str]) -> None:
    """Locations of the edge, of its curve and of the vertex (the check predivides)."""
    for loc_kind in range(4):
        p = ShapeProgram()
        move = p.location(0, 1.0, 2.0, 3.0)
        turn = p.location(1, 0, 0, 1, 0.5)
        both = p.location(2, move, turn)
        location = [0, move, turn, both][loc_kind]
        va, vb, e = segment(p, (0, 0, 0), (1, 0, 0), location=location)
        moved_e = p.located(e, move)
        moved_v = p.located(va, move)
        for v, context in ((va, e), (va, moved_e), (moved_v, moved_e), (moved_v, e), (vb, moved_e)):
            requests.append(check(VERTEX, p, v, *contexts_script(context)))


def representations(requests: list[str]) -> None:
    """Points on the curve, on a pcurve and on the surface, near and far."""
    for offset in (0.0, 1e-9, 0.25):
        sq = square(ShapeProgram())
        p = sq.p
        e = sq.edges[(0, 1)]
        v = sq.vertices[0]
        surface, face = sq.surfaces[0], sq.faces[0]
        p.vertex_on_curve(v, offset, e)
        p.vertex_on_pcurve(v, offset, e, surface)
        p.vertex_on_face(v, offset, 0.0, face)
        vmid = p.vertex((0.5, 0.0, 0.0))
        p.vertex_on_curve(vmid, 0.5+offset, e)
        p.vertex_on_face(vmid, 0.5, offset, face)
        wire = sq.wires[0][0]
        requests.append(check(VERTEX, p, v, *contexts_script(e, face, wire), (TOLERANCE,)))
        requests.append(check(VERTEX, p, vmid, *contexts_script(e, face), (TOLERANCE,)))
        requests.append(check(VERTEX, p, v, (IN_CONTEXT, face), (IN_CONTEXT, face), STATUS,
                              (STATUS_ON, face), (STATUS_ON, e), (IS_STATUS_ON, e), (PARALLEL, 1),
                              (IN_CONTEXT, e), CONTEXTS, (INIT, sq.vertices[1]), FLAGS, CONTEXTS))


def foreign_contexts(requests: list[str]) -> None:
    """Shapes that do not hold the vertex, compounds and wires that do."""
    p = ShapeProgram()
    va, vb, e = segment(p, (0, 0, 0), (1, 0, 0))
    other = p.vertex((5, 5, 5))
    wire = p.container(WIRE)
    p.add(wire, e, REVERSED)
    compound = p.container(3)
    p.add(compound, wire, FORWARD)
    for v in (va, other):
        requests.append(check(VERTEX, p, v, *contexts_script(e, wire, compound, vb)))


def append_vertex_requests(requests: list[str]) -> None:
    edge_contexts(requests)
    located_contexts(requests)
    representations(requests)
    foreign_contexts(requests)
