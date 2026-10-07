"""Deterministic requests for the BRepAlgo package (tools/oracle/tkbool_probe_brepalgo.cxx;
the records are documented there).

Wires of segments (collinear runs, a closed square), of a segment and tangent arcs, of
two arcs of one circle and of a B-spline and a segment are concatenated in G1, C1 and
C0; a square, a disc, a stadium and a Bezier-edged face are converted into arcs and
segments; a cube shell and solid, a face and an open box are checked, alone and as
results of arguments.
"""
from __future__ import annotations
import math
from tkbool_corpus_brepfill_offset_wire import bump, disc, polygon_face, stadium
from tkprim_corpus_sweep_shapes import segment
from tktopalgo_corpus_brepcheck_shapes import cube, solid_of
from tktopalgo_shape_program import FORWARD, REVERSED, WIRE, ShapeProgram

G1, C1, C0 = 1, 2, 0


def arc(p: ShapeProgram, center, radius: float, span, vertices) -> int:
    """A counter-clockwise arc on z = 0 between shared vertices."""
    e = p.edge()
    p.edge_curve(e, p.curve(1, center[0], center[1], 0.0, 0.0, 0.0, 1.0, radius))
    p.add(e, vertices[0], FORWARD)
    p.add(e, vertices[1], REVERSED)
    p.edge_range(e, span[0], span[1])
    return e


def bspline(p: ShapeProgram, poles, vertices) -> int:
    """A cubic B-spline edge between shared vertices."""
    e = p.edge()
    p.edge_curve(e, p.curve(5, 3, len(poles), *[c for pole in poles for c in pole], 2, 0.0, 4, 1.0, 4))
    p.add(e, vertices[0], FORWARD)
    p.add(e, vertices[1], REVERSED)
    p.edge_range(e, 0.0, 1.0)
    return e


def wire(p: ShapeProgram, edges, closed: bool = False) -> int:
    w = p.container(WIRE)
    for e in edges:
        p.add(w, e, FORWARD)
    if closed:
        p.flag(w, 0, True)
    return w


def chain(p: ShapeProgram, points, closed: bool = False) -> int:
    """The wire of segments through the points (sharing their vertices)."""
    v = [p.vertex(q) for q in points]
    n = len(points) if closed else len(points) - 1
    return wire(p, [segment(p, points[k], points[(k+1) % len(points)], (v[k], v[(k+1) % len(points)])) for k in range(n)], closed)


def tangent_arcs(p: ShapeProgram) -> int:
    """A segment, a tangent quarter arc and a tangent segment."""
    pts = [(0.0, 0.0, 0.0), (2.0, 0.0, 0.0), (3.0, 1.0, 0.0), (3.0, 3.0, 0.0)]
    v = [p.vertex(q) for q in pts]
    return wire(p, [segment(p, pts[0], pts[1], (v[0], v[1])), arc(p, (2.0, 1.0), 1.0, (-0.5*math.pi, 0.0), (v[1], v[2])),
                    segment(p, pts[2], pts[3], (v[2], v[3]))])


def circle_halves(p: ShapeProgram) -> int:
    """Two arcs of the same circle (a closed wire)."""
    a, b = p.vertex((1.0, 0.0, 0.0)), p.vertex((-1.0, 0.0, 0.0))
    return wire(p, [arc(p, (0.0, 0.0), 1.0, (0.0, math.pi), (a, b)), arc(p, (0.0, 0.0), 1.0, (math.pi, 2.0*math.pi), (b, a))], True)


def spline_and_segment(p: ShapeProgram) -> int:
    pts = [(0.0, 0.0, 0.0), (3.0, 0.0, 0.0), (4.0, 1.0, 0.0)]
    v = [p.vertex(q) for q in pts]
    return wire(p, [bspline(p, [(0.0, 0.0, 0.0), (1.0, 1.0, 0.0), (2.0, -1.0, 0.0), (3.0, 0.0, 0.0)], (v[0], v[1])),
                    segment(p, pts[1], pts[2], (v[1], v[2]))])


WIRES = {
    'collinear': lambda p: chain(p, [(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (2.0, 0.0, 0.0), (3.0, 1.0, 0.0)]),
    'square': lambda p: chain(p, [(0.0, 0.0, 0.0), (2.0, 0.0, 0.0), (2.0, 2.0, 0.0), (0.0, 2.0, 0.0)], True),
    'single': lambda p: chain(p, [(0.0, 0.0, 0.0), (2.0, 1.0, 0.0)]),
    'tangent_arcs': tangent_arcs,
    'circle_halves': circle_halves,
    'spline_segment': spline_and_segment,
}
FACES = {
    'square': lambda p: polygon_face(p, [(0.0, 0.0), (4.0, 0.0), (4.0, 3.0), (0.0, 3.0)]),
    'disc': disc,
    'stadium': stadium,
    'bump': bump,
}


def valid_requests() -> list[str]:
    """The cube shell and solid alone and as results, a face, an open box."""
    out = []
    p = ShapeProgram()
    shapes = cube(p, 2.0)
    shell = shapes.shell()
    solid = solid_of(p, shell)
    face = shapes.faces[0]
    tokens = p.tokens()
    out.append(f'ba_valid {tokens} 0 {solid} 1 1')
    out.append(f'ba_valid {tokens} 1 {solid} {solid} 1 1')
    out.append(f'ba_valid {tokens} 1 {face} {shell} 0 1')
    out.append(f'ba_valid {tokens} 1 {face} {solid} 1 0')
    out.append(f'ba_valid {tokens} 0 {face} 0 1')
    q = ShapeProgram()
    open_box = cube(q, 1.0, faces=[[0, 3, 2, 1], [0, 1, 5, 4], [1, 2, 6, 5]])
    box_shell = open_box.shell()
    out.append(f'ba_valid {q.tokens()} 0 {box_shell} 1 1')
    out.append(f'ba_valid {q.tokens()} 1 {open_box.faces[0]} {box_shell} 1 1')
    return out


def append_brepalgo_requests(requests: list[str]) -> None:
    """The concatenations, the conversions and the validity checks."""
    for name in sorted(WIRES):
        p = ShapeProgram()
        w = WIRES[name](p)
        for option in (G1, C1):
            requests.append(f'ba_concat {p.tokens()} {w} {option} 0.0001')
        requests.append(f'ba_concat_c0 {p.tokens()} {w}')
    for name in sorted(FACES):
        p = ShapeProgram()
        f = FACES[name](p)
        for tol in (0.1, 0.01):
            requests.append(f'ba_convert {p.tokens()} {f} {tol}')
    requests.extend(valid_requests())
    # BRepAlgo_FaceRestrictor on the scenes of the TopOpeBRepBuild requests.
    __import__('tkbool_corpus_topopebrepbuild').append_face_restrictor_requests(requests)
