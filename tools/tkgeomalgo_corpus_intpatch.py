"""Deterministic requests for the IntPatch core (tools/oracle/tkgeomalgo_probe_intpatch*.cxx).

`ip_point` edits an IntPatch_Point by command scripts (values, parameters, vertices and
arcs of both domains with every transition kind, reversal, multiplicity) and dumps the
points that have no arc or vertex. `ip_iwline` and `ip_pathpoint` edit the walking line
of IntPatch_TheIWalking and the path points and segments of IntPatch_TheSOnBounds.
The line requests (tkgeomalgo_corpus_intpatch_lines.py) edit the four line kinds and run
their ComputeVertexParameters; the tool requests (tkgeomalgo_corpus_intpatch_tools.py)
run IntPatch_HInterTool, IntPatch_HCurve2dTool, IntPatch_ArcFunction, IntPatch_CSFunction
with IntPatch_CurvIntSurf, and the curvature radius of IntPatch_PointLine; the polyhedron
and polygon requests are in tkgeomalgo_corpus_intpatch_poly.py.
"""
from __future__ import annotations
from tkgeomalgo_corpus_intcurvesurface import line_of

# Transition codes of ga_ip_transition.
TRANSITIONS = range(8)


def p_value(x, y, z, tol=1.0e-7, tangent=0):
    """Command 1: SetValue(Pt, Tol, Tangent)."""
    return [1, x, y, z, tol, tangent]


def p_pnt(x, y, z):
    return [2, x, y, z]


def p_pon2s(x, y, z, u1, v1, u2, v2):
    return [3, x, y, z, u1, v1, u2, v2]


def p_tol(tol):
    return [4, tol]


def p_params(u1, v1, u2, v2):
    return [5, u1, v1, u2, v2]


def p_para(para):
    return [6, para]


def p_vertex(first, rank):
    return [7, first, rank]


def p_arc(first, rank, prm, tline=1, tarc=2):
    return [8, first, rank, prm, tline, tarc]


def p_mult(flag):
    return [9, flag]


P_REVERSE = [10]


def point(*commands) -> list[float]:
    """A point script `n (command args)*`."""
    return [len(commands), *[v for c in commands for v in c]]


def free_point(x, y, z, u1, v1, u2, v2, para, tol=1.0e-7):
    """A point on no domain at the parameter `para` on the line."""
    return point(p_value(x, y, z, tol), p_params(u1, v1, u2, v2), p_para(para))


def _point_requests(requests: list[str]) -> None:
    base = [p_value(1.0, 2.0, 3.0, 1.0e-6, 1), p_params(0.1, 0.2, 0.3, 0.4), p_para(2.5)]
    requests.append(line_of('ip_point', [1, *point()]))
    requests.append(line_of('ip_point', [1, *point(*base)]))
    requests.append(line_of('ip_point', [1, *point(*base, p_mult(1), p_tol(0.25), p_pnt(-1.0, 0.5, 7.0))]))
    requests.append(line_of('ip_point', [1, *point(p_pon2s(0.5, -0.5, 1.5, 1.0, 2.0, 3.0, 4.0), p_para(-1.0))]))
    requests.append(line_of('ip_point', [1, *point(p_mult(1), p_value(0.0, 0.0, 0.0, 0.5, 0))]))
    requests.append(line_of('ip_point', [1, *point(p_tol(-2.0), P_REVERSE, p_para(1.0e300))]))
    for tl in TRANSITIONS:
        for ta in (0, 2, 5, 6):
            arcs = [p_arc(1, 1, 0.25 * tl, tl, ta), p_arc(0, 2, -0.5, ta, tl)]
            requests.append(line_of('ip_point', [0, *point(*base, *arcs)]))
            requests.append(line_of('ip_point', [0, *point(*base, *arcs, P_REVERSE, P_REVERSE)]))
    for first in (0, 1):
        for rank in range(5):
            requests.append(line_of('ip_point', [0, *point(p_vertex(first, rank))]))
            requests.append(line_of('ip_point', [0, *point(p_vertex(first, rank), p_arc(1 - first, 3, 1.5))]))
    # SetValue(Pt, Tol, Tangent) clears the domain flags but keeps the arcs.
    requests.append(line_of('ip_point', [0, *point(p_arc(1, 1, 0.5), p_vertex(0, 2), p_value(1.0, 1.0, 1.0),
                                                   p_arc(0, 3, 0.75, 3, 4))]))
    requests.append(line_of('ip_point', [0, *point(p_arc(1, 0, 0.5, 7, 7), p_arc(1, 2, 0.6, 4, 3), P_REVERSE)]))


def _iwline_requests(requests: list[str]) -> None:
    pts = [[1, 0.1 * i, 0.2 * i, -0.1 * i, 0.5 * i, 1.0 - i, i * i, 2.0] for i in range(6)]
    status = [[3, 0, 1, 2, 0.0, 0.0, 0.0, 0.1, 0.2], [6, 1, 4, 1.0, 1.0, 1.0, 0.3, 0.4]]
    passing = [[7, 3], [1, 9.0, 9.0, 9.0, 0.0, 0.0, 0.0, 0.0], [7, 5]]
    tangent = [[8, 1.0, 2.0, 3.0, 4], [9, 1], [10, 1]]
    scripts = [
        [],
        pts,
        pts + status + passing + tangent,
        pts + passing + [[11]] + status,
        pts + passing + [[12, 3]] + [[11]],
        pts + [[12, 0]] + [[12, 10]] + [[2, 1, 1], [5, 1]],
        pts + [[4, 1, 1, 1]] + [[2, 0, 0]] + [[11], [11]],
        [[7, 1]] + pts[:2] + [[7, 2], [11], [12, 1]],
    ]
    for script in scripts:
        requests.append(line_of('ip_iwline', [len(script), *[v for c in script for v in c]]))


def _pathpoint_requests(requests: list[str]) -> None:
    for v, a, sa, f, l in ((1, 1, 2, 1, 2), (2, 3, 1, 2, 0), (0, 0, 0, 0, 1), (3, 2, 3, 0, 0), (4, 1, 1, 2, 2)):
        requests.append(line_of('ip_pathpoint', [1.0, 2.0, 3.0, 1.0e-6, v, a, 0.25 * v,
                                                 -1.0, 0.5, 0.0, 2.0e-7, 4 - a, 1.5, sa, f, l]))


def append_intpatch_requests(requests: list[str]) -> None:
    """Every IntPatch request, the records first."""
    _point_requests(requests)
    _iwline_requests(requests)
    _pathpoint_requests(requests)
    __import__('tkgeomalgo_corpus_intpatch_lines').append_intpatch_line_requests(requests)
    __import__('tkgeomalgo_corpus_intpatch_tools').append_intpatch_tool_requests(requests)
    __import__('tkgeomalgo_corpus_intpatch_poly').append_intpatch_poly_requests(requests)
    __import__('tkgeomalgo_corpus_intpatch_wltool').append_intpatch_wltool_requests(requests)
    __import__('tkgeomalgo_corpus_intpatch_prmprm').append_intpatch_prmprm_requests(requests)
    __import__('tkgeomalgo_corpus_intpatch_extend').append_intpatch_extend_requests(requests)
    __import__('tkgeomalgo_corpus_intpatch_inter').append_intpatch_inter_requests(requests)
