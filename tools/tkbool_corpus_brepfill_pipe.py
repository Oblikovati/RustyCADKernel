"""Deterministic requests for BRepFill_Pipe (tools/oracle/tkbool_probe_brepfill_pipe.cxx; the
record is documented there).

The profiles of the pipe shell requests (a square and a circle in the plane normal to the
spine at its start, a vertex) and an edge are swept along the five spines with the
corrected Frenet trihedron; the segment and the bent polyline also sweep a square face
(a solid), a compound of two wires, and run with the Frenet, the discrete and an
unsupported (fixed: corrected Frenet) trihedron and the forced C1 approximation.
"""
from __future__ import annotations
from tkbool_corpus_brepfill_generator import polyline_wire
from tkbool_corpus_brepfill_location_law import PATHS
from tkbool_corpus_brepfill_pipe_shell import FRAMES, profile
from tkprim_corpus_sweep_shapes import segment
from tktopalgo_corpus_brepcheck_shapes import square as square_face
from tktopalgo_shape_program import COMPOUND, SOLID, ShapeProgram


def edge_profile(p: ShapeProgram, spine: str) -> int:
    """A segment across the spine at its start."""
    start, u, _, _ = FRAMES[spine]
    a = tuple(start[k] - 0.3*u[k] for k in range(3))
    b = tuple(start[k] + 0.3*u[k] for k in range(3))
    return segment(p, a, b)


def compound_profile(p: ShapeProgram, spine: str) -> int:
    """A compound of the square and a smaller square."""
    c = p.container(COMPOUND)
    p.add(c, profile(p, spine, 'square'))
    p.add(c, profile(p, spine, 'small'))
    return c


def solid_profile(p: ShapeProgram, spine: str) -> int:
    """An empty solid (refused)."""
    return p.container(SOLID)


def face_profile(p: ShapeProgram, spine: str) -> int:
    """A square face on z = 0 at the start of a spine along +Z."""
    return square_face(p, 0.4).faces[0]


PROFILES = {
    'square': lambda p, s: profile(p, s, 'square'),
    'circle': lambda p, s: profile(p, s, 'circle'),
    'vertex': lambda p, s: profile(p, s, 'vertex'),
    'edge': edge_profile,
}


def request(spine: str, builder, mode: int = 0, c1: int = 0) -> str:
    """One pipe of a profile along a spine, then the pipe line of a point near it."""
    p = ShapeProgram()
    spine_index = PATHS[spine](p)
    profile_index = builder(p, spine)
    start, u, _, _ = FRAMES[spine]
    point = ' '.join(repr(start[k] + 0.1*u[k]) for k in range(3))
    # The tables are null natively for a vertex profile (the table queries crash).
    tables = 0 if builder is PROFILES['vertex'] else 1
    return f'bf_pipe {p.tokens()} {spine_index} {profile_index} {mode} {c1} {point} {tables}'


def append_pipe_requests(requests: list[str]) -> None:
    """Every profile along every spine, then the variants."""
    for spine in sorted(FRAMES):
        for name in sorted(PROFILES):
            requests.append(request(spine, PROFILES[name]))
    for spine in ('segment', 'bent'):
        requests.append(request(spine, face_profile))
        requests.append(request(spine, compound_profile))
        for mode in (1, 2, 9):
            requests.append(request(spine, PROFILES['square'], mode))
        requests.append(request(spine, PROFILES['circle'], 0, 1))
    requests.append(request('segment', solid_profile))
