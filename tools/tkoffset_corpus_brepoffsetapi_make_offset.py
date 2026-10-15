"""Deterministic requests for BRepOffsetAPI_MakeOffset
(tools/oracle/tkoffset_probe_brepoffsetapi_make_offset.cxx; the record is documented there).

The spine faces of the BRepFill_OffsetWire corpus (tools/tkbool_corpus_brepfill_offset_wire.py):
each is offset from the face and from its wires, with both joins, outward then inward
(and inward then outward), the B-spline bump with and without the arcs-and-segments
approximation; the open polyline gives one domain of open wires; a maker without spine
raises StdFail_NotDone; the small circles of the BRepOffset_MakeOffset GTests are offset
across their collapse boundary.
"""
from __future__ import annotations
import math
from tkbool_corpus_brepfill_offset_wire import ARC, INTERSECTION, FACES, open_polyline, xy_plane, arc_edge, face_of_edges
from tktopalgo_shape_program import ShapeProgram

FROM_FACE, FROM_WIRES, NO_SPINE = 0, 1, 2


def circle(p: ShapeProgram, radius: float) -> int:
    """A disc of the radius around the origin: one closed circle edge."""
    s = xy_plane(p)
    v = p.vertex((radius, 0.0, 0.0))
    return face_of_edges(p, s, [arc_edge(p, s, (0.0, 0.0), radius, (0.0, 2.0*math.pi), (v, v))])


def request(build, mode: int, join: int, offsets, is_open: int = 0, approx: int = 0) -> str:
    """Two Performs of one maker."""
    p = ShapeProgram()
    face = build(p)
    (o1, a1), (o2, a2) = offsets
    return f'oa_make_offset {p.tokens()} {face} {mode} {join} {is_open} {approx} {o1} {a1} {o2} {a2}'


def append_make_offset_requests(requests: list[str]) -> None:
    """Every spine face from the face and from its wires, both joins and both sides."""
    for name in sorted(FACES):
        for mode in (FROM_FACE, FROM_WIRES):
            for join in (ARC, INTERSECTION):
                requests.append(request(FACES[name], mode, join, ((0.5, 0.0), (-0.3, 0.0))))
                requests.append(request(FACES[name], mode, join, ((-0.2, 0.4), (0.7, 0.0))))
    for mode in (FROM_FACE, FROM_WIRES):
        requests.append(request(FACES['bump'], mode, ARC, ((0.3, 0.0), (-0.2, 0.0)), 0, 1))
        requests.append(request(FACES['stadium'], mode, INTERSECTION, ((0.3, 0.0), (0.3, 0.0)), 0, 1))
    for join in (ARC, INTERSECTION):
        for mode in (FROM_FACE, FROM_WIRES):
            requests.append(request(open_polyline, mode, join, ((0.5, 0.0), (-0.5, 0.0)), 1))
            requests.append(request(open_polyline, mode, join, ((0.5, 0.0), (-0.5, 0.0)), 0))
    requests.append(request(FACES['disc'], NO_SPINE, ARC, ((0.5, 0.0), (0.5, 0.0))))
    taper = math.tan(-1.0*math.pi/180.0)*3.0
    for diameter in (1.0, 2.0, 2.01):
        for offset in (taper, 0.05, -1.0, -0.5*diameter, -0.49*diameter):
            for join in (ARC, INTERSECTION):
                build = lambda p, r=diameter/2.0: circle(p, r)
                requests.append(request(build, FROM_WIRES, join, ((offset, 0.0), (-offset, 0.0))))
