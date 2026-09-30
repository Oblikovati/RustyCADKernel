"""The BRepCheck_Wire requests of the TKTopAlgo corpus: connected, closed (in 3d and on
the face), oriented and self-intersecting wires, figure-eight wires (the 2d choice of
ChoixUV), seams, and BRepCheck::SelfIntersection."""
from __future__ import annotations
from tktopalgo_corpus_brepcheck import CONTEXTS, FLAGS, IN_CONTEXT, STATUS, WIRE_CHECK, check
from tktopalgo_corpus_brepcheck_shapes import PlanarShapes, cylinder_face, square
from tktopalgo_shape_program import FORWARD, INTERNAL, REVERSED, WIRE, ShapeProgram, num

CLOSED, CLOSED2D, ORIENTATION, SELF_INTERSECT, SET_GCTRL, GCTRL, SET_STATUS = range(1, 8)
Z = (0.0, 0.0, 1.0)


def members(face: int, update: int) -> list:
    """Every check of the wire on `face`, then the flags and contexts."""
    return [(CLOSED, update), (CLOSED2D, face, update), (ORIENTATION, face, update),
            (SELF_INTERSECT, face, -1, -1, update), GCTRL]


def wire_requests(requests: list[str], p: ShapeProgram, wire: int, face: int) -> None:
    """The wire alone (each check, with and without update) and in its face."""
    for update in (0, 1):
        requests.append(check(WIRE_CHECK, p, wire, *members(face, update), STATUS))
    requests.append(check(WIRE_CHECK, p, wire, (IN_CONTEXT, face), FLAGS, CONTEXTS, *members(face, 0)))
    requests.append(check(WIRE_CHECK, p, wire, (SET_GCTRL, 0), (IN_CONTEXT, face), (SET_GCTRL, 1),
                          (CLOSED, 1), (SET_STATUS, 20), STATUS))
    requests.append(f'bck_selfint {p.tokens()} {wire} {face} -1 -1')


def polygon_face(points, loops, **options):
    """A face on z = 0 bounded by point loops; returns the shapes."""
    shapes = PlanarShapes(ShapeProgram(), [(*xy, 0.0) for xy in points])
    shapes.face(loops, normal=Z, **options)
    shapes.finish()
    return shapes


def planar_requests(requests: list[str]) -> None:
    """Squares (plain, holed, reversed hole), a bow tie, a figure eight, a T shape."""
    cases = [
        ([(0, 0), (1, 0), (1, 1), (0, 1)], [[0, 1, 2, 3]], {}),
        ([(0, 0), (1, 0), (1, 1), (0, 1)], [[0, 1, 2, 3]], {'orientations': [REVERSED]}),
        ([(0, 0), (1, 1), (1, 0), (0, 1)], [[0, 1, 2, 3]], {}),
        ([(0, 0), (1, 0), (1, 1), (2, 1), (2, 2), (1, 2), (0, 1)], [[0, 1, 2, 3, 4, 5, 2, 6]], {}),
        ([(0, 0), (2, 0), (2, 1), (1, 1), (1, 0), (1, -1), (0, -1)], [[0, 4, 1, 2, 3, 4, 5, 6]], {}),
        ([(0, 0), (4, 0), (4, 4), (0, 4), (1, 1), (1, 2), (2, 2), (2, 1)], [[0, 1, 2, 3], [4, 5, 6, 7]], {}),
        ([(0, 0), (3, 0), (3, 3), (0, 3), (1, 1), (1, 2), (2, 2), (2, 1)], [[0, 1, 2, 3], [4, 7, 6, 5]], {}),
        ([(0, 0), (1, 0), (1, 1), (0, 1), (0.5, 0.0)], [[0, 4, 1, 2, 3]], {'shifts': {(0, 4): (0.0, 1e-3)}}),
    ]
    for points, loops, options in cases:
        shapes = polygon_face(points, loops, **options)
        for wire in shapes.wires[0]:
            wire_requests(requests, shapes.p, wire, shapes.faces[0])


def topology_requests(requests: list[str]) -> None:
    """Open, disconnected, empty and redundant wires, INTERNAL and reversed edges."""
    sq = square(ShapeProgram())
    p, e = sq.p, sq.edges
    variants = [[(e[(0, 1)], FORWARD), (e[(1, 2)], FORWARD), (e[(2, 3)], REVERSED)],
                [(e[(0, 1)], FORWARD), (e[(2, 3)], FORWARD)],
                [],
                [(e[(0, 1)], FORWARD), (e[(0, 1)], FORWARD), (e[(1, 2)], FORWARD), (e[(2, 3)], REVERSED),
                 (e[(0, 3)], REVERSED)],
                [(e[(0, 1)], FORWARD), (e[(1, 2)], REVERSED), (e[(2, 3)], REVERSED), (e[(0, 3)], REVERSED)],
                [(e[(0, 1)], FORWARD), (e[(1, 2)], FORWARD), (e[(2, 3)], REVERSED), (e[(0, 3)], REVERSED),
                 (e[(1, 2)], INTERNAL)],
                [(e[(0, 1)], INTERNAL), (e[(1, 2)], INTERNAL)],
                [(e[(0, 1)], FORWARD), (e[(0, 1)], REVERSED)]]
    wires = []
    for variant in variants:
        wire = p.container(WIRE)
        for edge, orientation in variant:
            p.add(wire, edge, orientation)
        wires.append(wire)
    for wire in wires:
        wire_requests(requests, p, wire, sq.faces[0])


def curve_requests(requests: list[str]) -> None:
    """A seam wire on a cylinder, a 2d gap on it, and a wire with an edge whose pcurve
    loops over itself (a self-intersecting Bezier pcurve)."""
    for shift in (0.0, 1e-9, 0.2):
        p = ShapeProgram()
        cyl = cylinder_face(p, seam_shift=shift)
        wire_requests(requests, p, cyl['wire'], cyl['face'])
    p = ShapeProgram()
    plane = p.surface(0, 0, 0, 0, 0, 0, 1)
    v = p.vertex((0.0, 0.0, 0.0))
    loop = p.edge()
    p.edge_curve(loop, p.curve(3, 4, 0, 0, 0, 2, 2, 0, 2, -2, 0, 0, 0, 0))
    p.edge_pcurve(loop, p.pcurve(3, 4, 0, 0, 2, 2, 2, -2, 0, 0), plane)
    p.edge_range(loop, 0.0, 1.0)
    p.add(loop, v, FORWARD)
    p.add(loop, v, REVERSED)
    cross = p.edge()
    p.edge_curve(cross, p.curve(3, 4, 0, 0, 0, 3, 3, 0, 3, -3, 0, 0, 0, 0))
    p.edge_pcurve(cross, p.pcurve(3, 4, 0, 0, 3, 3, 3, -3, 0, 0), plane)
    p.edge_range(cross, 0.0, 1.0)
    p.add(cross, v, FORWARD)
    p.add(cross, v, REVERSED)
    wire = p.container(WIRE)
    p.add(wire, loop, FORWARD)
    face = p.face(plane)
    p.add(face, wire, FORWARD)
    wire_requests(requests, p, wire, face)
    wire2 = p.container(WIRE)
    p.add(wire2, loop, FORWARD)
    p.add(wire2, cross, FORWARD)
    face2 = p.face(plane)
    p.add(face2, wire2, FORWARD)
    wire_requests(requests, p, wire2, face2)


def missing_pcurve_requests(requests: list[str]) -> None:
    """Edges without pcurve on a cylinder (the first edge, a later edge)."""
    for missing in ('bottom', 'top'):
        p = ShapeProgram()
        cyl = cylinder_face(p)
        # A bare copy of the edge without pcurve replaces it in a new wire.
        bare = p.edge()
        p.edge_curve(bare, p.curve(1, 0, 0, 0 if missing == 'bottom' else 2, 0, 0, 1, 1.0))
        p.edge_range(bare, 0.0, 6.283185307179586)
        vertex = cyl['v0'] if missing == 'bottom' else cyl['v1']
        p.add(bare, vertex, FORWARD)
        p.add(bare, vertex, REVERSED)
        wire = p.container(WIRE)
        order = [('bottom', FORWARD), ('seam', FORWARD), ('top', REVERSED), ('seam', REVERSED)]
        for name, o in order:
            p.add(wire, bare if name == missing else cyl[name], o)
        face = p.face(cyl['surface'])
        p.add(face, wire, FORWARD)
        wire_requests(requests, p, wire, face)
    sq = square(ShapeProgram())
    requests.append(f"bck_selfint {sq.p.tokens()} {sq.wires[0][0]} {sq.faces[0]} {sq.edges[(0, 1)]} {sq.edges[(1, 2)]}")


def append_wire_requests(requests: list[str]) -> None:
    planar_requests(requests)
    topology_requests(requests)
    curve_requests(requests)
    missing_pcurve_requests(requests)


__all__ = ['append_wire_requests', 'num', 'INTERNAL']
