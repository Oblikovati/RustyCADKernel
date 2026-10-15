"""Deterministic requests for BRepOffsetAPI_MakePipeShell
(tools/oracle/tkoffset_probe_brepoffsetapi_make_pipe_shell.cxx; the record is documented there).

The spines, frames and profiles of the BRepFill_PipeShell corpus
(tools/tkbool_corpus_brepfill_pipe_shell.py), driven through the builder API: each spine
with the constructor defaults (no SetTransitionMode) and each transition mode; the segment
and the bent polyline with every location mode, two sections, a profile at a spine vertex;
a maker without profile (not ready); the closed square with two sections at the same place
(not done, so MakeSolid raises StdFail_NotDone); the face boundary on its support.
"""
from __future__ import annotations
from tkbool_corpus_brepfill_location_law import PATHS
from tkbool_corpus_brepfill_pipe_shell import FRAMES, MODES, frame_square, guide_of, profile
from tktopalgo_corpus_brepcheck_shapes import square as square_face
from tktopalgo_shape_program import ShapeProgram

DEFAULT = -1


def request(spine: str, mode: str, transition: int, profiles: list[tuple[str, bool, int]],
            solid: int = 1, extra=None) -> str:
    """One pipe shell maker; the profiles are `(kind, at_end, vertex)`."""
    p = ShapeProgram()
    spine_index = PATHS[spine](p)
    names = extra(p) if extra else {}
    tokens = []
    for kind, at_end, vertex in profiles:
        index = profile(p, spine, kind, at_end)
        vertex_index = p.vertex(FRAMES[spine][0]) if vertex else -1
        tokens.append(f'{index} {vertex_index} 0 0')
    body = ' '.join([str(len(profiles)), *tokens])
    return f'oa_pipe_shell {p.tokens()} {spine_index} {mode.format(**names)} {transition} {body} {solid}'


def support_request() -> str:
    """The boundary of a square face swept on the face (Darboux trihedron)."""
    p = ShapeProgram()
    shapes = square_face(p, 2.0)
    spine = shapes.wires[0][0]
    section = frame_square(p, (0.0, 0.0, 0.0), (0.0, 1.0, 0.0), (0.0, 0.0, 1.0), 0.2)
    return f'oa_pipe_shell {p.tokens()} {spine} 6 {shapes.faces[0]} {DEFAULT} 1 {section} -1 0 0 1'


def append_pipe_shell_requests(requests: list[str]) -> None:
    """Every spine with the defaults and each transition, then the modes and variants."""
    for spine in sorted(FRAMES):
        for transition in (DEFAULT, 0, 1, 2):
            requests.append(request(spine, '0', transition, [('square', False, 0)]))
        requests.append(request(spine, '0', DEFAULT, [('circle', False, 0)]))
    for spine in ('segment', 'bent'):
        for mode in MODES:
            requests.append(request(spine, mode, DEFAULT, [('square', False, 0)], extra=guide_of(spine)))
        requests.append(request(spine, '0', DEFAULT, [('square', False, 0), ('small', True, 0)]))
        requests.append(request(spine, '0', DEFAULT, [('square', False, 0), ('vertex', True, 0)]))
        requests.append(request(spine, '0', DEFAULT, [('square', False, 1)]))
        requests.append(request(spine, '0', DEFAULT, []))
    requests.append(request('closed_square', '0', DEFAULT, [('square', False, 0), ('small', True, 0)]))
    requests.append(support_request())
