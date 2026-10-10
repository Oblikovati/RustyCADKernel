"""Deterministic requests for BRepFill_Draft (tools/oracle/tkbool_probe_brepfill_draft.cxx; the
record is documented there).

A closed square wire, a square face and a shell of that face, and an open polyline are
drafted up along Z at a small angle (and straight), with each transition, outward and
inward, up to a length, up to a plane (keeping either side) and up to a stop face
above them.
"""
from __future__ import annotations
from tkbool_corpus_brepfill_generator import polyline_wire, square
from tktopalgo_corpus_brepcheck_shapes import square as square_face
from tktopalgo_shape_program import SHELL, ShapeProgram


def wire_shape(p: ShapeProgram) -> int:
    """A closed square wire of side 2 around the Z axis."""
    return polyline_wire(p, square(0.0, 2.0), True)


def face_shape(p: ShapeProgram) -> int:
    """The square face [0, 2] x [0, 2] on z = 0."""
    return square_face(p, 2.0).faces[0]


def shell_shape(p: ShapeProgram) -> int:
    """A shell of the square face."""
    s = p.container(SHELL)
    p.add(s, square_face(p, 2.0).faces[0])
    return s


def open_shape(p: ShapeProgram) -> int:
    """An open polyline."""
    return polyline_wire(p, [(0.0, 0.0, 0.0), (2.0, 0.0, 0.0), (2.0, 2.0, 0.0)], False)


SHAPES = {'wire': wire_shape, 'face': face_shape, 'shell': shell_shape, 'open': open_shape}


def stop_face(p: ShapeProgram) -> int:
    """A square face of side 8 moved to z = 2 above the shapes."""
    face = square_face(p, 8.0).faces[0]
    return p.located(face, p.location(0, -3.0, -3.0, 2.0))


def request(shape: str, mode: str, style: int = 1, internal: int = 0, angle: float = 0.1) -> str:
    """One draft of a shape along +Z; `mode` is `length`, `plane keep` or `stop keep`."""
    p = ShapeProgram()
    index = SHAPES[shape](p)
    kind, *rest = mode.split()
    if kind == 'length':
        tail = '0 3.0'
    elif kind == 'plane':
        tail = f'1 0.0 0.0 2.0 0.0 0.0 1.0 {rest[0]}'
    else:
        tail = f'2 {stop_face(p)} {rest[0]}'
    return f'bf_draft {p.tokens()} {index} 0.0 0.0 1.0 {angle} {style} 0.01 3.0 {internal} {tail}'


def append_draft_requests(requests: list[str]) -> None:
    """Every shape up to a length with each style, outward and inward, then the stops."""
    for shape in sorted(SHAPES):
        for style in (0, 1, 2):
            requests.append(request(shape, 'length', style))
        requests.append(request(shape, 'length', 1, 1))
        requests.append(request(shape, 'length', 1, 0, 0.0))
        for keep in (0, 1):
            requests.append(request(shape, f'plane {keep}'))
            requests.append(request(shape, f'stop {keep}'))
