"""Deterministic requests for BRepAlgo_NormalProjection and BRepProj_Projection
(tools/oracle/tkbool_probe_brepproj.cxx; the records are documented there).

Segments, an arc and a B-spline above a square face and a cube shell are projected normally (limited by the faces or not, within a distance or not, with
or without 3d curves); a square and a circle wire above them are projected along -Z
and from a point.
"""
from __future__ import annotations
from tkbool_corpus_brepalgo import arc, bspline, chain
from tkbool_corpus_brepfill_offset_wire import polygon_face
from tkprim_corpus_sweep_shapes import segment
from tktopalgo_corpus_brepcheck_shapes import cube
from tktopalgo_shape_program import ShapeProgram


def targets(p: ShapeProgram) -> dict:
    """The square face [0, 4]^2 on z = 0 and the shell of the cube [0, 4]^3 moved down
    to z in [-4, 0]."""
    out = {'square': polygon_face(p, [(0.0, 0.0), (4.0, 0.0), (4.0, 4.0), (0.0, 4.0)])}
    shell = cube(p, 4.0).shell()
    out['cube'] = p.located(shell, p.location(0, 0.0, 0.0, -4.0))
    return out


def tools(p: ShapeProgram) -> dict:
    """Edges above the square: a crossing segment, an inner segment, an arc, a spline."""
    v = [p.vertex(q) for q in ((1.0, 1.0, 1.0), (3.0, 3.0, 1.5))]
    return {
        'crossing': segment(p, (-1.0, 2.0, 1.0), (5.0, 2.0, 1.0)),
        'inner': segment(p, (1.0, 1.0, 1.0), (3.0, 3.0, 1.5), (v[0], v[1])),
        'arc': arc(p, (2.0, 2.0), 1.5, (0.0, 2.0), (p.vertex((3.5, 2.0, 0.0)), p.vertex((2.0 + 1.5 * -0.4161468365471424, 2.0 + 1.5 * 0.9092974268256817, 0.0)))),
        'spline': bspline(p, [(0.5, 0.5, 2.0), (1.5, 3.0, 2.0), (2.5, 1.0, 2.0), (3.5, 3.5, 2.0)],
                          (p.vertex((0.5, 0.5, 2.0)), p.vertex((3.5, 3.5, 2.0)))),
    }


def normal_request(target: str, edges: list[str], maxdist: float = -1.0, with3d: int = 1, limit: int = 1) -> str:
    p = ShapeProgram()
    t = targets(p)
    e = tools(p)
    ids = ' '.join(str(e[name]) for name in edges)
    return f'ba_normal_projection {p.tokens()} {t[target]} {len(edges)} {ids} {maxdist} {with3d} {limit}'


def projection_request(target: str, wire: str, mode: int, xyz) -> str:
    p = ShapeProgram()
    t = targets(p)
    if wire == 'square':
        w = chain(p, [(1.0, 1.0, 3.0), (3.0, 1.0, 3.0), (3.0, 3.0, 3.0), (1.0, 3.0, 3.0)], True)
    else:
        e = tools(p)
        w = e['crossing']
    return f'bp_projection {p.tokens()} {w} {t[target]} {mode} {xyz[0]} {xyz[1]} {xyz[2]}'


def append_brepproj_requests(requests: list[str]) -> None:
    """The normal projections and the projections of wires."""
    for target in ('square', 'cube'):
        requests.append(normal_request(target, ['crossing', 'inner']))
        requests.append(normal_request(target, ['arc', 'spline']))
        requests.append(normal_request(target, ['crossing'], -1.0, 1, 0))
        requests.append(normal_request(target, ['spline'], -1.0, 0, 1))
        requests.append(normal_request(target, ['inner'], 0.5, 1, 1))
    for target in ('square', 'cube'):
        for wire in ('square', 'edge'):
            requests.append(projection_request(target, wire, 0, (0.0, 0.0, -1.0)))
            requests.append(projection_request(target, wire, 1, (2.0, 2.0, 6.0)))
    requests.append(projection_request('square', 'square', 0, (0.0, 0.0, 0.0)))
