"""Deterministic requests for the BRepFill package functions
(tools/oracle/tkbool_probe_brepfill_package.cxx; the records are documented there).

`bf_face` rules faces between segments, arcs, closed circles, B-splines, REVERSED edges
and located edges (the shared-location path and different locations); `bf_shell` rules
shells between squares, circles, open polylines and B-spline wires; `bf_axe` frames
profiles on a planar face, a closed wire, a cylinder face (Standard_NoSuchObject) and an
edge (Standard_DomainError), in vertex contact or not; `bf_search_origin` restarts closed
squares and circles near a vertex, inside an edge and off the wire, along both directions
(and refuses an open wire); `bf_compute_acr` and `bf_insert_acr` measure and cut
polylines, circles, punctual and REVERSED-edge wires.
"""
from __future__ import annotations
import math
from tkbool_corpus_brepfill_generator import (bspline_wire, circle_wire, point_wire,
                                              polyline_wire, square, wire_of)
from tkprim_corpus_sweep_shapes import bspline, segment
from tktopalgo_corpus_brepcheck_shapes import cylinder_face
from tktopalgo_corpus_brepcheck_shapes import square as square_face
from tktopalgo_shape_program import FORWARD, REVERSED, ShapeProgram


def numbers(values) -> str:
    """The fields of a request; integers stay integers."""
    return ' '.join(str(v) if isinstance(v, int) else repr(float(v)) for v in values)


def circle_edge(p: ShapeProgram, center, radius: float) -> int:
    """A closed +Z circle edge with one vertex at the parameter 0."""
    e = p.edge()
    p.edge_curve(e, p.curve(1, *center, 0.0, 0.0, 1.0, radius))
    v = p.vertex((center[0]+radius, center[1], center[2]))
    p.add(e, v, FORWARD)
    p.add(e, v, REVERSED)
    p.edge_range(e, 0.0, 2*math.pi)
    return e


def arc_edge(p: ShapeProgram, center, radius: float, first: float, last: float) -> int:
    """An arc of a +Z circle between two angles."""
    e = p.edge()
    p.edge_curve(e, p.curve(1, *center, 0.0, 0.0, 1.0, radius))
    for t, o in ((first, FORWARD), (last, REVERSED)):
        p.add(e, p.vertex((center[0]+radius*math.cos(t), center[1]+radius*math.sin(t), center[2])), o)
    p.edge_range(e, first, last)
    return e


def face_pairs():
    """Programs with two edges (table indices) for BRepFill::Face."""
    def segments(p):
        return segment(p, (0, 0, 0), (2, 0, 0)), segment(p, (0, 1, 1), (2, 1, 1))

    def segment_spline(p):
        return segment(p, (0, 0, 0), (3, 0, 0)), bspline(p, [(0, 0, 1), (1, 1, 1), (2, -1, 1), (3, 0, 1)])

    def circles(p):
        return circle_edge(p, (0, 0, 0), 1.0), circle_edge(p, (0, 0, 2.0), 1.5)

    def arcs(p):
        return arc_edge(p, (0, 0, 0), 1.0, 0.0, 1.5), arc_edge(p, (0, 0, 1.0), 2.0, 0.2, 1.7)

    def reversed_edges(p):
        a = p.oriented(segment(p, (2, 0, 0), (0, 0, 0)), REVERSED)
        b = p.oriented(segment(p, (2, 1, 1), (0, 1, 1)), REVERSED)
        return a, b

    def same_location(p):
        loc = p.location(0, 0.5, 0.0, 1.0)
        return (p.located(segment(p, (0, 0, 0), (2, 0, 0)), loc),
                p.located(segment(p, (0, 1, 1), (2, 1, 1)), loc))

    def other_locations(p):
        return (p.located(segment(p, (0, 0, 0), (2, 0, 0)), p.location(0, 0.5, 0.0, 1.0)),
                p.located(segment(p, (0, 1, 1), (2, 1, 1)), p.location(0, 0.0, 0.0, 2.0)))

    def degenerate_side(p):
        # Two segments with a common end: the lateral edge there is degenerated.
        v = p.vertex((0.0, 0.0, 0.0))
        a = segment(p, (0, 0, 0), (2, 0, 0), (v, p.vertex((2.0, 0.0, 0.0))))
        b = segment(p, (0, 0, 0), (2, 1, 1), (v, p.vertex((2.0, 1.0, 1.0))))
        return a, b

    return [segments, segment_spline, circles, arcs, reversed_edges, same_location,
            other_locations, degenerate_side]


def shell_pairs():
    """Programs with two wires for BRepFill::Shell."""
    return [
        lambda p: (polyline_wire(p, square(0.0, 2.0), True), polyline_wire(p, square(1.0, 1.0), True)),
        lambda p: (polyline_wire(p, square(0.0, 2.0), True), polyline_wire(p, square(1.0, 2.0, 0.3), True)),
        lambda p: (circle_wire(p, (0, 0, 0), 1.0), circle_wire(p, (0, 0, 2.0), 2.0)),
        lambda p: (polyline_wire(p, [(0, 0, 0), (1, 0, 0), (2, 1, 0)], False),
                   polyline_wire(p, [(0, 0, 1), (1, 0, 1.5), (2, 1, 1)], False)),
        lambda p: (bspline_wire(p, [(0, 0, 0), (1, 1, 0), (2, -1, 0), (3, 0, 0)]),
                   bspline_wire(p, [(0, 0, 2), (1, 2, 2.5), (2, -2, 2), (3, 0, 2.2)])),
        lambda p: (polyline_wire(p, square(0.0, 2.0), True, True), polyline_wire(p, square(1.0, 2.0), True)),
    ]


def axe_cases():
    """Programs with a spine, a profile wire and a tolerance for BRepFill::Axe."""
    def on_face_vertex(p):
        spine = square_face(p, 2.0).faces[0]
        return spine, polyline_wire(p, [(0, 0, 0), (0, 0, 1), (0.5, 0, 1.5)], False), 1e-7

    def near_face_edge(p):
        spine = square_face(p, 2.0).faces[0]
        return spine, polyline_wire(p, [(1.0, -0.25, 0.0), (1.0, -0.25, 1.0)], False), 0.5

    def far_from_wire(p):
        spine = polyline_wire(p, square(0.0, 2.0), True)
        return spine, polyline_wire(p, [(3.0, 0.5, 0.0), (3.0, 0.5, 1.0)], False), 1e-3

    def wire_vertex(p):
        spine = polyline_wire(p, square(0.0, 2.0), True)
        return spine, polyline_wire(p, [(-1, -1, 0), (-1, -1.5, 1)], False), 1e-7

    def cylinder(p):
        spine = cylinder_face(p, 1.0, 2.0)['face']
        return spine, polyline_wire(p, [(2, 0, 0), (2, 0, 1)], False), 1e-7

    def edge_spine(p):
        spine = segment(p, (0, 0, 0), (1, 0, 0))
        return spine, polyline_wire(p, [(2, 0, 0), (2, 0, 1)], False), 1e-7

    return [on_face_vertex, near_face_edge, far_from_wire, wire_vertex, cylinder, edge_spine]


ORIGINS = [
    ((-1.0, -1.0, 0.0), (1.0, 0.0, 0.0)), ((-1.0, -1.0, 0.0), (-1.0, 0.0, 0.0)),
    ((0.3, -1.0, 0.0), (1.0, 0.0, 0.0)), ((0.3, -1.2, 0.0), (-1.0, 0.1, 0.0)),
    ((1.0, 0.99999999, 0.0), (0.0, 1.0, 0.0)), ((5.0, 5.0, 0.0), (0.0, 0.0, 1.0)),
]


def append_face_requests(requests: list[str]) -> None:
    for builder in face_pairs():
        p = ShapeProgram()
        a, b = builder(p)
        requests.append(f'bf_face {p.tokens()} {a} {b}')
        requests.append(f'bf_face {p.tokens()} {b} {a}')


def append_shell_requests(requests: list[str]) -> None:
    for builder in shell_pairs():
        p = ShapeProgram()
        a, b = builder(p)
        requests.append(f'bf_shell {p.tokens()} {a} {b}')
        requests.append(f'bf_shell {p.tokens()} {b} {a}')


def append_axe_requests(requests: list[str]) -> None:
    for builder in axe_cases():
        p = ShapeProgram()
        spine, profile, tol = builder(p)
        requests.append(f'bf_axe {p.tokens()} {spine} {profile} {numbers([tol])}')


def append_origin_requests(requests: list[str]) -> None:
    wires = [
        lambda p: polyline_wire(p, square(0.0, 2.0), True),
        lambda p: polyline_wire(p, square(0.0, 2.0), True, True),
        lambda p: circle_wire(p, (0, 0, 0), 1.0),
        lambda p: polyline_wire(p, [(0, 0, 0), (1, 0, 0)], False),
    ]
    for builder in wires:
        p = ShapeProgram()
        w = builder(p)
        for point, direction in ORIGINS:
            for tol in (1e-7, 0.25):
                requests.append(f'bf_search_origin {p.tokens()} {w} {numbers([*point, *direction, tol])}')


ACR_WIRES = [
    lambda p: polyline_wire(p, square(0.0, 2.0), True),
    lambda p: polyline_wire(p, [(0, 0, 0), (1, 0, 0), (1, 3, 0), (2, 3, 1)], False),
    lambda p: polyline_wire(p, square(0.0, 2.0), True, True),
    lambda p: circle_wire(p, (0, 0, 0), 1.0),
    lambda p: bspline_wire(p, [(0, 0, 0), (1, 1, 0), (2, -1, 0), (3, 0, 0)]),
    lambda p: point_wire(p, (1.0, 2.0, 3.0)),
    lambda p: wire_of(p, [(segment(p, (0, 0, 0), (1, 0, 0)), FORWARD),
                          (segment(p, (1, 2, 0), (1, 0, 0)), REVERSED)]),
]
CUTS = [[0.1], [0.125, 0.3, 0.6, 0.9], [0.25, 0.5, 0.75], [0.2500001, 0.5], [0.0, 1.0]]


def append_acr_requests(requests: list[str]) -> None:
    for builder in ACR_WIRES:
        p = ShapeProgram()
        w = builder(p)
        requests.append(f'bf_compute_acr {p.tokens()} {w}')
        for k, cuts in enumerate(CUTS):
            prec = (1e-6, 1e-3)[k % 2]
            requests.append(f'bf_insert_acr {p.tokens()} {w} {len(cuts)} {numbers([*cuts, prec])}')


def append_package_requests(requests: list[str]) -> None:
    """Every BRepFill package request family in a fixed order."""
    append_face_requests(requests)
    append_shell_requests(requests)
    append_axe_requests(requests)
    append_origin_requests(requests)
    append_acr_requests(requests)
