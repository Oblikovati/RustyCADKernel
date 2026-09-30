"""Deterministic requests for ShapeExtend_WireData (tools/oracle/tkshhealing_probe_wiredata.cxx).

Each `sh_wiredata` request builds one scene with a TKTopAlgo shape program
(tools/tktopalgo_shape_program.py): a unit square wire on a plane face, the same wire
reversed, a wire with an INTERNAL edge, a broken (unchained) wire, a degenerated edge, and a
cylinder face whose wire holds a seam edge twice (FORWARD and REVERSED, with two
p-curves). A request then runs a script of wire data ops on it and prints the digest.
"""
from __future__ import annotations
import math
from dataclasses import dataclass
from tktopalgo_shape_program import (COMPOUND, EXTERNAL, FORWARD, INTERNAL, REVERSED, WIRE,
                                     ShapeProgram)

# The op codes of the probe (see the native file).
INIT_WIRE, ADD_EDGE, ADD_WIRE, ADD_SHAPE, ADD_ORIENTED, REMOVE, SET, REVERSE = range(8)
REVERSE_FACE, SET_LAST, SET_DEGENERATED_LAST, COMPUTE_SEAMS, CLEAR, MANIFOLD = range(8, 14)
INIT_OTHER, ADD_OTHER, INIT_SELF, KEEP_NONMANIFOLD = range(14, 18)


@dataclass
class WireScene:
    """The program and the table indices of its shapes."""
    program: ShapeProgram
    edges: list[int]
    square: int
    square_reversed: int
    with_internal: int
    broken: int
    plane_face: int
    internal_edge: int
    external_edge: int
    degenerated: int
    seam_wire: int
    seam_face: int
    seam_edge: int
    vertex: int
    compound: int


def square_edges(p: ShapeProgram, plane: int) -> tuple[list[int], int]:
    """The four edges of the unit square with 3d lines and plane p-curves; the first vertex."""
    corners = [(0., 0.), (1., 0.), (1., 1.), (0., 1.)]
    vertices = [p.vertex((x, y, 0.)) for x, y in corners]
    edges = []
    for i, (a, b) in enumerate(zip(corners, corners[1:] + corners[:1])):
        direction = (b[0]-a[0], b[1]-a[1])
        edge = p.edge()
        p.edge_curve(edge, p.line((a[0], a[1], 0.), (*direction, 0.)))
        p.edge_pcurve(edge, p.line2d(a, direction), plane)
        p.edge_range(edge, 0., 1.)
        p.add(edge, vertices[i], FORWARD)
        p.add(edge, vertices[(i+1) % 4], REVERSED)
        p.vertex_on_curve(vertices[i], 0., edge)
        p.vertex_on_curve(vertices[(i+1) % 4], 1., edge)
        edges.append(edge)
    return edges, vertices[0]


def wire_of(p: ShapeProgram, uses: list[tuple[int, int]]) -> int:
    """A new wire of the (edge, orientation) uses."""
    wire = p.container(WIRE)
    for edge, orientation in uses:
        p.add(wire, edge, orientation)
    return wire


def seam_scene(p: ShapeProgram) -> tuple[int, int, int]:
    """A cylinder face: bottom and top circles and a seam line with two p-curves."""
    cylinder = p.surface(1, 0., 0., 0., 0., 0., 1., 1.)
    bottom_v, top_v = p.vertex((1., 0., 0.)), p.vertex((1., 0., 1.))
    circles = []
    for height, vertex in ((0., bottom_v), (1., top_v)):
        edge = p.edge()
        p.edge_curve(edge, p.curve(1, 0., 0., height, 0., 0., 1., 1.))
        p.edge_pcurve(edge, p.line2d((0., height), (1., 0.)), cylinder)
        p.edge_range(edge, 0., 2*math.pi)
        p.add(edge, vertex, FORWARD)
        p.add(edge, vertex, REVERSED)
        circles.append(edge)
    seam = p.edge()
    p.edge_curve(seam, p.line((1., 0., 0.), (0., 0., 1.)))
    p.edge_pcurves(seam, p.line2d((2*math.pi, 0.), (0., 1.)), p.line2d((0., 0.), (0., 1.)), cylinder)
    p.edge_range(seam, 0., 1.)
    p.add(seam, bottom_v, FORWARD)
    p.add(seam, top_v, REVERSED)
    wire = wire_of(p, [(circles[0], FORWARD), (seam, FORWARD), (circles[1], REVERSED), (seam, REVERSED)])
    face = p.face(cylinder)
    p.add(face, wire)
    return wire, face, seam


def scene() -> WireScene:
    """The shapes every wire data request works on."""
    p = ShapeProgram()
    plane = p.surface(0, 0., 0., 0., 0., 0., 1.)
    edges, vertex = square_edges(p, plane)
    square = wire_of(p, [(e, FORWARD) for e in edges])
    internal, external = p.oriented(edges[0], INTERNAL), p.oriented(edges[1], EXTERNAL)
    with_internal = wire_of(p, [(edges[0], FORWARD), (internal, -1), (edges[1], FORWARD), (external, -1)])
    broken = wire_of(p, [(edges[0], FORWARD), (edges[2], FORWARD), (edges[1], FORWARD)])
    plane_face = p.face(plane)
    p.add(plane_face, square)
    degenerated = p.edge()
    p.edge_flags(degenerated, True, True, True)
    p.add(degenerated, vertex, FORWARD)
    p.add(degenerated, vertex, REVERSED)
    seam_wire, seam_face, seam_edge = seam_scene(p)
    compound = p.container(COMPOUND)
    p.add(compound, edges[0])
    return WireScene(p, edges, square, p.oriented(square, REVERSED), with_internal, broken,
                     plane_face, internal, external, degenerated, seam_wire, seam_face,
                     seam_edge, vertex, compound)


def request(s: WireScene, ctor: list[int], ops: list[list[int]]) -> str:
    """One `sh_wiredata program ctor nops (op)...` line."""
    fields = [*ctor, len(ops), *[v for op in ops for v in op]]
    return ' '.join(['sh_wiredata', s.program.tokens(), *map(str, fields)])


def init_scripts(s: WireScene) -> list[tuple[list[int], list[list[int]]]]:
    """Construction from each wire in both modes, chained or not."""
    scripts = []
    for wire in (s.square, s.square_reversed, s.with_internal, s.broken, s.seam_wire, -1):
        for chained in (1, 0):
            for manifold in (1, 0):
                scripts.append(([1, wire, chained, manifold], []))
                scripts.append(([0], [[INIT_WIRE, wire, chained, manifold]]))
    return scripts


def edit_scripts(s: WireScene) -> list[tuple[list[int], list[list[int]]]]:
    """Adding, removing, replacing and rotating edges, with bad indices and bad casts."""
    e, new = s.edges, [0]
    return [
        (new, [[ADD_EDGE, e[0], 0], [ADD_EDGE, e[1], 0], [ADD_EDGE, e[2], 1], [ADD_EDGE, e[3], 2]]),
        (new, [[ADD_EDGE, s.internal_edge, 0], [ADD_EDGE, s.external_edge, 0], [ADD_EDGE, -1, 0]]),
        (new, [[MANIFOLD, 0], [ADD_EDGE, s.internal_edge, 0], [ADD_EDGE, -1, 0], [ADD_EDGE, e[0], 0]]),
        (new, [[ADD_EDGE, e[0], 5], [ADD_EDGE, e[1], -1], [ADD_EDGE, e[2], 0]]),
        (new, [[ADD_WIRE, s.square, 0], [ADD_WIRE, s.with_internal, 2], [ADD_WIRE, -1, 0]]),
        (new, [[MANIFOLD, 0], [ADD_WIRE, s.with_internal, 0], [ADD_WIRE, s.square, 3]]),
        # A null shape is left out: ShapeType() dereferences its null TShape natively.
        (new, [[ADD_SHAPE, s.square, 0], [ADD_SHAPE, e[0], 1], [ADD_SHAPE, s.vertex, 0],
               [ADD_SHAPE, s.plane_face, 0]]),
        (new, [[ADD_ORIENTED, 0, e[0], 0], [ADD_ORIENTED, 0, e[1], 1], [ADD_ORIENTED, 0, e[2], 2],
               [ADD_ORIENTED, 0, e[3], 3], [ADD_ORIENTED, 0, e[0], -1], [ADD_ORIENTED, 0, -1, 0]]),
        (new, [[ADD_ORIENTED, 1, s.square, 1], [ADD_ORIENTED, 1, s.square, 3], [ADD_ORIENTED, 2, e[0], 5],
               [ADD_ORIENTED, 2, s.square, 0], [ADD_ORIENTED, 2, s.vertex, 1], [ADD_ORIENTED, 1, -1, 0]]),
        ([1, s.square, 1, 1], [[REMOVE, 2], [REMOVE, 0], [REMOVE, 5], [REMOVE, -1]]),
        ([0], [[REMOVE, 0]]),
        ([1, s.square, 1, 1], [[SET, e[2], 1], [SET, e[3], 0], [SET, e[0], 9], [SET, e[0], -2]]),
        ([1, s.square, 1, 1], [[SET, s.internal_edge, 1], [SET, s.external_edge, 3], [SET, s.internal_edge, 0]]),
        ([1, s.square, 1, 0], [[SET, s.internal_edge, 2], [SET, s.plane_face, 1]]),
        ([1, s.square, 1, 1], [[SET_LAST, 2]]),
        ([1, s.square, 1, 1], [[SET_LAST, 0], [SET_LAST, 4], [SET_LAST, 7]]),
        ([1, s.square, 1, 1], [[SET_LAST, -1]]),
        ([0], [[SET_LAST, -2]]),
        ([1, s.square, 1, 1], [[ADD_EDGE, s.degenerated, 2], [SET_DEGENERATED_LAST]]),
        ([1, s.square, 1, 1], [[SET_DEGENERATED_LAST]]),
        ([1, s.square, 1, 1], [[REVERSE]]),
        ([1, s.broken, 1, 1], [[REVERSE], [REVERSE]]),
        ([0], [[REVERSE], [ADD_EDGE, e[1], 0], [REVERSE]]),
        ([1, s.square, 1, 1], [[CLEAR], [ADD_EDGE, e[3], 0]]),
    ]


def seam_scripts(s: WireScene) -> list[tuple[list[int], list[list[int]]]]:
    """Seams: computing them, reversing a wire on its face, Index of a seam."""
    e, new = s.edges, [0]
    return [
        ([1, s.seam_wire, 1, 1], [[COMPUTE_SEAMS, 0]]),
        ([1, s.seam_wire, 1, 1], [[COMPUTE_SEAMS, 1], [ADD_EDGE, s.seam_edge, 0]]),
        ([1, s.seam_wire, 1, 1], [[REVERSE_FACE, s.seam_face]]),
        ([1, s.seam_wire, 1, 1], [[REVERSE_FACE, -1]]),
        ([1, s.seam_wire, 1, 1], [[REVERSE_FACE, s.seam_face], [REVERSE_FACE, s.seam_face]]),
        ([1, s.square, 1, 1], [[REVERSE_FACE, s.plane_face]]),
        ([1, s.square, 1, 1], [[REVERSE_FACE, e[0]]]),
        (new, [[ADD_EDGE, e[0], 0], [ADD_ORIENTED, 0, e[0], 1], [ADD_EDGE, e[1], 0],
               [ADD_ORIENTED, 0, e[1], 1], [ADD_EDGE, e[0], 0]]),
        (new, [[ADD_ORIENTED, 0, e[0], 1], [ADD_EDGE, e[0], 0], [ADD_ORIENTED, 0, e[0], 1],
               [ADD_EDGE, e[0], 0], [COMPUTE_SEAMS, 1], [REVERSE]]),
    ]


def other_scripts(s: WireScene) -> list[tuple[list[int], list[list[int]]]]:
    """Copies from another wire data, from itself, and the shared non-manifold handle."""
    return [
        ([0], [[INIT_OTHER, s.with_internal, 1, 1]]),
        ([0], [[INIT_OTHER, s.with_internal, 1, 0]]),
        ([1, s.square, 1, 1], [[INIT_OTHER, s.seam_wire, 1, 1], [ADD_EDGE, s.edges[0], 0]]),
        ([1, s.square, 1, 1], [[ADD_OTHER, s.with_internal, 1, 1, 0]]),
        ([1, s.square, 1, 1], [[ADD_OTHER, s.with_internal, 1, 0, 2]]),
        ([1, s.square, 1, 0], [[ADD_OTHER, s.with_internal, 1, 1, 1]]),
        ([1, s.square, 1, 0], [[ADD_OTHER, s.with_internal, 1, 0, 0]]),
        ([1, s.square, 1, 1], [[ADD_OTHER, s.square, 1, 1, -3]]),
        ([1, s.with_internal, 1, 1], [[INIT_SELF]]),
        ([1, s.with_internal, 1, 1], [[KEEP_NONMANIFOLD]]),
        ([1, s.with_internal, 1, 0], [[KEEP_NONMANIFOLD], [MANIFOLD, 0], [ADD_EDGE, s.internal_edge, 0]]),
    ]


def append_wiredata_requests(requests: list[str]) -> None:
    """Append the ShapeExtend_WireData requests."""
    s = scene()
    for ctor, ops in init_scripts(s) + edit_scripts(s) + seam_scripts(s) + other_scripts(s):
        requests.append(request(s, ctor, ops))
