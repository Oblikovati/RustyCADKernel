"""The BRepCheck and BRepLib_ValidateEdge requests of the TKTopAlgo numeric corpus.

The shapes are shape programs (tktopalgo_shape_program.py, built by
tktopalgo_corpus_brepcheck_shapes.py). `bck_check` runs a script of members on one
checker: the codes 90..99 are the BRepCheck_Result members, the lower codes those of
the checker (tools/oracle/tktopalgo_probe_brepcheck.cxx). The families of each
checker are in tktopalgo_corpus_brepcheck_<checker>.py.
"""
from __future__ import annotations
import math
from tktopalgo_shape_program import ShapeProgram, num

VERTEX, EDGE, WIRE_CHECK, FACE, SHELL_CHECK = 0, 1, 2, 3, 4
# The BRepCheck_Result members of a script.
IN_CONTEXT, BLIND, STATUS, STATUS_ON, IS_STATUS_ON, CONTEXTS, PARALLEL, FAIL, INIT, FLAGS = range(90, 100)


def script(*calls) -> str:
    """`n (code args)...`."""
    flat = [v for call in calls for v in (call if isinstance(call, (list, tuple)) else [call])]
    return ' '.join([str(len(calls)), *map(num, flat)])


def check(kind: int, program: ShapeProgram, index: int, *calls) -> str:
    return f'bck_check {kind} {program.tokens()} {index} {script(*calls)}'


def contexts_script(*contexts: int) -> list:
    """InContext in each shape, then the result flags, the contexts and a Blind."""
    calls: list = [(IN_CONTEXT, c) for c in contexts]
    calls += [FLAGS, CONTEXTS, BLIND, FLAGS]
    return calls


def status_requests(requests: list[str]) -> None:
    """`bck_status`: every enumerator and out-of-range values."""
    for s in range(-2, 40):
        requests.append(f'bck_status {s}')


def add_requests(requests: list[str]) -> None:
    """`bck_add`: sequences with and without NoError, repeated statuses."""
    sequences = [[0], [0, 0], [0, 13], [13, 0], [13, 13], [0, 13, 0, 14], [5, 4, 5, 0, 36],
                 [0, 0, 0, 7], [36, 35, 34, 33, 0], [1, 2, 3, 1, 2, 3], [0, 1, 0, 2, 0, 3],
                 [18, 0, 18], list(range(37)), list(range(36, -1, -1))]
    for seq in sequences:
        requests.append(f'bck_add {len(seq)} '+' '.join(map(str, seq)))


def prec_requests(requests: list[str]) -> None:
    """`bck_prec`: PrecCurve of lines, circles, ellipses (plain, trimmed, on a plane) and
    PrecSurface of planes, cones and other surfaces."""
    ellipses = [((0, 0, 0), 2.0, 1.0), ((-3.5, 1e-3, 7.0), 10.0, 0.25), ((1e10, -2e8, 3.0), 1e6, 1.0),
                ((1e-300, 0.0, -0.0), 1e-5, 1e-6), ((-7.25, -1e5, 123.456), 5.5, 5.5)]
    for centre, r1, r2 in ellipses:
        p = ShapeProgram()
        c = p.curve(2, *centre, 0, 0, 1, r1, r2)
        t = p.curve(4, c, 0.5, 2.0)
        requests.append(f'bck_prec {p.tokens()} 0 {c} 0')
        requests.append(f'bck_prec {p.tokens()} 0 {t} 0')
    p = ShapeProgram()
    others = [p.line((1, 2, 3), (0, 0, 1)), p.curve(1, 5, 5, 5, 0, 1, 0, 3.0),
              p.curve(3, 3, 0, 0, 0, 1, 1, 0, 2, 0, 1), p.curve(5, 2, 3, 0, 0, 0, 1, 2, 0, 2, 0, 0, 2, 0, 3, 1, 3)]
    for c in others:
        requests.append(f'bck_prec {p.tokens()} 0 {c} 0')
    surfaces = ShapeProgram()
    records = [(0, 0, 0, 0, 0, 0, 1), (4, 1, 2, 3, 0, 0, 1, 2.0, 0.5), (4, -1e7, 3e-3, 0, 1, 0, 0, 1e5, 0.1),
               (4, 0, 0, 0, 0, 0, 1, 0.0, 0.7), (1, 0, 0, 0, 0, 0, 1, 1.5), (2, 1, 1, 1, 0, 0, 1, 4.0),
               (3, 0, 0, 0, 0, 0, 1, -1, 1, -2, 2), (5, 0, 0, 0, 0, 0, 1, 3.0, 1.0)]
    for record in records:
        s = surfaces.surface(*record)
        requests.append(f'bck_prec {surfaces.tokens()} 1 {s} 0')
    on_plane = ShapeProgram()
    plane = on_plane.surface(0, 2, -3, 4, 0, 0, 1)
    for pc in (on_plane.pcurve(2, 1.0, -2.0, 3.0, 1.0), on_plane.pcurve(1, 5.0, 5.0, 2.0, 1),
               on_plane.line2d((0, 0), (1, 1))):
        requests.append(f'bck_prec {on_plane.tokens()} 2 {pc} {plane}')


def append_brepcheck_requests(requests: list[str]) -> None:
    """Every BRepCheck and BRepLib_ValidateEdge family, in a fixed order."""
    status_requests(requests)
    add_requests(requests)
    prec_requests(requests)
    for module, function in (('tktopalgo_corpus_brepcheck_vertex', 'append_vertex_requests'),
                             ('tktopalgo_corpus_brepcheck_edge', 'append_edge_requests'),
                             ('tktopalgo_corpus_brepcheck_wire', 'append_wire_requests'),
                             ('tktopalgo_corpus_brepcheck_face', 'append_face_requests'),
                             ('tktopalgo_corpus_brepcheck_shell', 'append_shell_requests'),
                             ('tktopalgo_corpus_brepcheck_solid', 'append_solid_requests'),
                             ('tktopalgo_corpus_brepcheck_validate', 'append_validate_requests')):
        getattr(__import__(module), function)(requests)


__all__ = ['append_brepcheck_requests', 'check', 'contexts_script', 'script', 'math']
