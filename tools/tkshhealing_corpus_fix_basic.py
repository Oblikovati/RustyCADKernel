"""Deterministic requests for ShapeFix_Root, ShapeFix_ShapeTolerance and ShapeFix_EdgeConnect
(tools/oracle/tkshhealing_probe_fix_basic.cxx).

`sh_fixroot` runs scripts of the setters (precision above and below the limits, `Set`
from another root), the context, the three registrator kinds and the messages sent with
and without a shape, and `NeedFix` on the three flag positions.

`sh_fixtol` limits and forces the tolerances of the strip scene of
tools/tkshhealing_corpus_analysis_basic.py with each sub-shape type and interval.

`sh_edgeconnect` connects edges whose ends do not share their vertices: a triangle of
three separate lines with gaps, in open and closed wires, edges with curves on a plane
whose ends differ from their 3D curve ends, a closed circle edge, groups merged by a
later connection, repeated connections, `Clear` before `Build`.
"""
from __future__ import annotations
import math
from tkshhealing_corpus_analysis_basic import (T_EDGE, T_FACE, T_SHAPE, T_SHELL, T_VERTEX, T_WIRE, request,
                                               topo_scene)
from tktopalgo_shape_program import COMPOUND, FORWARD, REVERSED, WIRE, ShapeProgram


def ops_request(op: str, program: str, ops: list[list[float]], *tail: float) -> str:
    """One `op program nops (op)... tail` line."""
    return request(op, program, len(ops), *[x for o in ops for x in o], *tail)


def fixroot_requests() -> list[str]:
    """Setter, registrator and message scripts on a ShapeFix_Root."""
    p = ShapeProgram()
    v = p.vertex((0., 0., 0.))
    e = p.edge()
    prog = p.tokens()
    scripts = [
        [],
        [[0, 1e-3], [3, 0.5], [3, 1e-5]],
        [[1, 1e-5], [2, 0.1], [0, 1e-2], [0, 1.], [0, 1e-9], [3, 2.]],
        [[2, 1e-9], [3, 1.], [1, 5.], [0, 0.5]],
        [[4, 0.25, 0.01, 3.], [3, 10.], [5]],
        [[5], [4, 1e-3, 1e-4, 1e-2]],
        [[6, 2], [7, v, 1, 0], [7, v, 2, 1], [7, e, 3, 2], [8, 4, 0], [8, 5, 1], [8, 6, 2], [7, -1, 7, 0]],
        [[6, 0], [7, v, 1, 0], [8, 2, 2]],
        [[6, 1], [7, v, 1, 0], [8, 2, 1]],
        [[6, 2], [7, v, 1, 0], [4, 1e-3, 1e-4, 1.], [7, v, 2, 0]],
        [[9, -1, 1], [9, -1, 0], [9, 0, 1], [9, 1, 0], [9, 5, 0], [9, -7, 1]],
    ]
    return [ops_request('sh_fixroot', prog, ops) for ops in scripts]


def fixtol_requests() -> list[str]:
    """LimitTolerance and SetTolerance on the strip scene."""
    s = topo_scene()
    n, prog = s.n, s.program.tokens()
    out = []
    types = [T_VERTEX, T_EDGE, T_FACE, T_WIRE, T_SHAPE, T_SHELL]
    for name in ('f1', 'sh_good', 'c_shells', 'h0', 'a3', 'w_f2', 'fint', 'deg', 'c_empty'):
        for ty in types:
            for tmin, tmax in ((1e-5, 1e-3), (1e-4, 0.), (0., 1e-6), (2e-5, 2e-5), (-1., 1.)):
                out.append(ops_request('sh_fixtol', prog, [[0, n[name], tmin, tmax, ty]]))
            for preci in (1e-4, 0., -1.):
                out.append(ops_request('sh_fixtol', prog, [[1, n[name], preci, ty]]))
    out.append(ops_request('sh_fixtol', prog, [[0, -1, 0., 1., T_SHAPE], [1, -1, 1., T_SHAPE]]))
    out.append(ops_request('sh_fixtol', prog, [[1, n['so'], 1e-3, T_WIRE], [0, n['so'], 1e-6, 1e-4, T_SHAPE]]))
    return out


def line_edge(p: ShapeProgram, start, end, v1: int, v2: int, tol: float = 1e-7) -> int:
    """An edge on the line from start to end, on [0, length], with its vertices."""
    e = p.edge()
    length = math.dist(start, end)
    direction = tuple((b-a)/length for a, b in zip(start, end))
    p.edge_curve(e, p.line(start, direction), 0, tol)
    p.edge_range(e, 0., length)
    p.add(e, v1, FORWARD)
    p.add(e, v2, REVERSED)
    return e


def triangle(p: ShapeProgram, n: dict[str, int]) -> None:
    """Three lines of a triangle with separate vertices and gaps, and their wires."""
    pts = [(0., 0., 0.), (1., 0., 0.), (0., 1., 0.), (1., 1., 0.)]
    gap = [(0., 0., 0.), (1.002, 0.001, 0.), (0., 1.003, 0.), (1., 1., 0.004)]
    for i, (a, b) in enumerate(((0, 1), (1, 2), (2, 0))):
        start = pts[a] if i else gap[a]
        va = p.vertex(start, 1e-4)
        vb = p.vertex(gap[b], 2e-4)
        n[f'e{i}'] = line_edge(p, start, gap[b], va, vb)
    n['e3'] = line_edge(p, pts[3], pts[1], p.vertex(pts[3]), p.vertex((1., 0.001, 0.)))
    n['e1r'] = p.oriented(n['e1'], REVERSED)
    for name, closed in (('w_open', False), ('w_closed', True)):
        wire = p.container(WIRE)
        for i in range(3):
            p.add(wire, n[f'e{i}'])
        if closed:
            p.flag(wire, 0, True)
        n[name] = wire


def curves_on_plane(p: ShapeProgram, n: dict[str, int]) -> None:
    """Two lines with p-curves on a plane that end away from their 3D ends, and a closed
    circle edge in a closed wire."""
    plane = p.surface(0, 0., 0., 0., 0., 0., 1.)
    vp = [p.vertex((2., 0., 0.)), p.vertex((3., 0., 0.)), p.vertex((3.001, 0., 0.)), p.vertex((4., 0., 0.))]
    for name, (a, b), (x0, x1) in (('p0', (0, 1), (2., 3.)), ('p1', (2, 3), (3.001, 4.))):
        e = line_edge(p, (x0, 0., 0.), (x1, 0., 0.), vp[a], vp[b])
        p.edge_pcurve(e, p.line2d((x0, 0.0005), (1., 0.)), plane)
        n[name] = e
    vc = p.vertex((1., 0., 5.))
    circle = p.edge()
    p.edge_curve(circle, p.curve(1, 0., 0., 5., 0., 0., 1., 1.))
    p.edge_range(circle, 0., 2*math.pi)
    p.add(circle, vc, FORWARD)
    p.add(circle, vc, REVERSED)
    n['circle'] = circle
    cw = p.container(WIRE)
    p.add(cw, circle)
    p.flag(cw, 0, True)
    n['w_circle'] = cw


def connect_scene() -> tuple[ShapeProgram, dict[str, int]]:
    """Separate edges with gaps between their ends, and a compound of them all."""
    p = ShapeProgram()
    n: dict[str, int] = {}
    triangle(p, n)
    curves_on_plane(p, n)
    everything = p.container(COMPOUND)
    for name in ('e0', 'e1', 'e2', 'e3', 'p0', 'p1', 'circle'):
        p.add(everything, n[name])
    n['all'] = everything
    return p, n


def edgeconnect_requests() -> list[str]:
    """Scripts of Add, Build and Clear."""
    p, n = connect_scene()
    prog = p.tokens()
    e0, e1, e2, e3 = n['e0'], n['e1'], n['e2'], n['e3']
    scripts = [
        [[0, e0, e1], [0, e1, e2], [0, e2, e0], [2]],
        [[1, n['w_open']], [2]],
        [[1, n['w_closed']], [2]],
        [[0, e0, e1], [0, e2, e3], [0, e0, e3], [2]],
        [[0, e0, e1], [0, e0, e1], [0, e1, e0], [2]],
        [[0, n['p0'], n['p1']], [2]],
        [[1, n['w_circle']], [2]],
        [[0, e0, e1], [3], [2]],
        [[0, n['e1r'], e0], [2]],
        [[1, n['all']], [2]],
        [[0, e2, e3], [0, e3, e2], [0, e0, e1], [0, e1, e2], [2]],
        [[2]],
    ]
    return [ops_request('sh_edgeconnect', prog, ops, n['all']) for ops in scripts]


def append_fix_basic_requests(requests: list[str]) -> None:
    """Append the ShapeFix_Root, ShapeFix_ShapeTolerance and ShapeFix_EdgeConnect requests."""
    requests += fixroot_requests()
    requests += fixtol_requests()
    requests += edgeconnect_requests()
