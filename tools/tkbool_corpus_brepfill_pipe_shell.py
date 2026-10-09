"""Deterministic requests for BRepFill_PipeShell (tools/oracle/tkbool_probe_brepfill_pipe_shell.cxx;
the record is documented there).

The spines are the paths of the location law requests (a segment, a bent polyline, a
closed square, a tangent arc after a segment, a B-spline). The profiles lie in the plane
normal to the spine at its start (a square, a circle), with a smaller square or a vertex
at the end for two sections. Each spine is swept with the default trihedron in each
transition; the segment and the bent polyline also run with every location mode (Frenet,
corrected Frenet, discrete, fixed, constant binormal, guides by abscissa and by plane,
with and without contact), with a homothety law, with the profile at a spine vertex, with
the degree and segment limits and the forced C1 approximation; closed profiles are made
solid.
"""
from __future__ import annotations
import math
from tkbool_corpus_brepfill_generator import circle_wire, point_wire, polyline_wire
from tkbool_corpus_brepfill_location_law import PATHS
from tktopalgo_corpus_brepcheck_shapes import square as square_face
from tktopalgo_shape_program import ShapeProgram

# The start point and two directions of the plane normal to each spine at its start, and
# its end point with the same plane directions.
FRAMES = {
    'segment': ((0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (0.0, 1.0, 0.0), (0.0, 0.0, 3.0)),
    'bent': ((0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (0.0, 1.0, 0.0), (1.0, 1.0, 4.0)),
    'closed_square': ((-2.0, -2.0, 0.0), (0.0, 1.0, 0.0), (0.0, 0.0, 1.0), (-2.0, -2.0, 0.0)),
    'arc_after_segment': ((0.0, 0.0, 0.0), (0.0, 1.0, 0.0), (0.0, 0.0, 1.0), (2.0, 1.0, 0.0)),
    'bspline': ((0.0, 0.0, 0.0), (1/math.sqrt(2), -1/math.sqrt(2), 0.0),
                (1/math.sqrt(6), 1/math.sqrt(6), -2/math.sqrt(6)), (3.0, 0.0, 3.0)),
}


def frame_square(p: ShapeProgram, center, u, v, half: float) -> int:
    """The closed square of half side `half` around `center` in the plane of `u`, `v`."""
    corners = [(-1, -1), (1, -1), (1, 1), (-1, 1)]
    points = [tuple(center[k] + half*(a*u[k] + b*v[k]) for k in range(3)) for a, b in corners]
    return polyline_wire(p, points, True)


def profile(p: ShapeProgram, spine: str, kind: str, at_end: bool = False) -> int:
    """A profile of the spine: `square`, `small` (half size), `circle` or `vertex`."""
    start, u, v, end = FRAMES[spine]
    center = end if at_end else start
    if kind == 'vertex':
        return p.vertex(center)
    if kind == 'circle':
        normal = (0.0, 0.0, 1.0) if u == (1.0, 0.0, 0.0) else (1.0, 0.0, 0.0)
        return circle_wire(p, center, 0.3, normal)
    return frame_square(p, center, u, v, 0.15 if kind == 'small' else 0.3)


# transition angmin angmax maxdeg maxseg c1 (the native defaults otherwise).
def settings(transition: int, maxdeg: int = 11, maxseg: int = 100, c1: int = 0) -> str:
    """The SetTransition, SetMaxDegree, SetMaxSegments and SetForceApproxC1 tokens."""
    return f'{transition} 0.01 6.0 {maxdeg} {maxseg} {c1}'


def request(spine: str, mode: str, transition: int, profiles: list[tuple[str, bool, int]],
            law: str = '0', solid: int = 1, extra=None, tune: str | None = None) -> str:
    """One pipe shell: the profiles are `(kind, at_end, vertex)` with `vertex` 1 to place
    the profile at the first spine vertex; `extra(p)` adds shapes used by the mode
    (`{guide}` or `{support}` in the mode tokens)."""
    p = ShapeProgram()
    spine_index = PATHS[spine](p)
    names = extra(p) if extra else {}
    tokens = []
    for kind, at_end, vertex in profiles:
        index = profile(p, spine, kind, at_end)
        vertex_index = p.vertex(FRAMES[spine][0]) if vertex else -1
        tokens.append(f'{index} {vertex_index} 0 0')
    mode_tokens = mode.format(**names)
    body = f'{len(profiles)} ' + ' '.join(tokens)
    return f'bf_pipe_shell {p.tokens()} {spine_index} {mode_tokens} {tune or settings(transition)} {body} {law} {solid}'


def guide_of(spine: str):
    """A guide parallel to the spine, shifted by 1 along the first plane direction."""
    def build(p: ShapeProgram) -> dict:
        _, u, _, _ = FRAMES[spine]
        points = {'segment': [(0, 0, 0), (0, 0, 3)], 'bent': [(0, 0, 0), (0, 0, 2), (1, 0, 3), (1, 1, 4)]}[spine]
        shifted = [tuple(q[k] + u[k] for k in range(3)) for q in points]
        return {'guide': polyline_wire(p, shifted, False)}
    return build


MODES = ['0', '1', '2', '3', '4 0 0 0 0 0 1 1 0 0', '5 1 0 0',
         '7 {guide} 1 0', '7 {guide} 0 0', '7 {guide} 1 1', '7 {guide} 0 1']


def append_pipe_shell_requests(requests: list[str]) -> None:
    """Every spine with each transition, then the location modes and the variants."""
    for spine in sorted(FRAMES):
        for transition in (0, 1, 2):
            requests.append(request(spine, '0', transition, [('square', False, 0)]))
        requests.append(request(spine, '0', 0, [('circle', False, 0)]))
    for spine in ('segment', 'bent'):
        for mode in MODES:
            requests.append(request(spine, mode, 0, [('square', False, 0)], extra=guide_of(spine)))
        requests.append(request(spine, '0', 0, [('square', False, 0), ('small', True, 0)]))
        requests.append(request(spine, '0', 0, [('square', False, 0), ('vertex', True, 0)]))
        requests.append(request(spine, '0', 0, [('square', False, 1)]))
        requests.append(request(spine, '0', 0, [('square', False, 0)], law='1 0.0 1.0 1.0 0.5'))
        requests.append(request(spine, '0', 1, [('square', False, 0)], tune=settings(1, 6, 10, 1)))
    # Two sections at the same place of the closed square (not increasing: not done)
    # and the automatic homothety of the contact on the border.
    requests.append(request('closed_square', '0', 0, [('square', False, 0), ('small', True, 0)]))
    for spine in ('segment', 'bent'):
        requests.append(request(spine, '7 {guide} 1 2', 0, [('square', False, 0)], extra=guide_of(spine)))
    requests.append(support_request())


def support_request() -> str:
    """The boundary of a square face swept on the face (Darboux trihedron)."""
    p = ShapeProgram()
    shapes = square_face(p, 2.0)
    spine = shapes.wires[0][0]
    section = frame_square(p, (0.0, 0.0, 0.0), (0.0, 1.0, 0.0), (0.0, 0.0, 1.0), 0.2)
    return (f'bf_pipe_shell {p.tokens()} {spine} 6 {shapes.faces[0]} {settings(0)} '
            f'1 {section} -1 0 0 0 1')
