"""Deterministic requests for ShapeBuild (tools/oracle/tkshhealing_probe_shapebuild.cxx and
tools/oracle/tkshhealing_probe_reshape.cxx).

`sh_vertex` combines two spheres (point, tolerance): one inside the other, apart,
touching, at the same point, with signed-zero and tiny coordinates, through points and
through vertices.

`sh_edge` works on one scene built with a TKTopAlgo shape program
(tools/tktopalgo_shape_program.py): edges on a plane and on a cylinder with 3D curves and
p-curves (a line, a periodic circle, a trimmed circle, a seam with two p-curves), edges
with a p-curve only, a bare edge, a located edge and an edge with an INTERNAL vertex, and
faces on both surfaces (one located). Every ShapeBuild_Edge member runs on them, and
TransformPCurve on lines, circles, ellipses, Bezier, B-spline, trimmed and offset
p-curves with each 2D transformation and U factor.

`sh_reshape` records replacements and removals on shells, solids and compounds of faces
and edges, then applies them (both Apply members, Status, Value) with and without the
location mode.
"""
from __future__ import annotations
import math
from dataclasses import dataclass, field
from tktopalgo_shape_program import (COMPOUND, FORWARD, INTERNAL, REVERSED, SHELL, SOLID,
                                     WIRE, ShapeProgram, num)

PI = math.pi


@dataclass
class EdgeScene:
    """The program and the named indices of its shapes and geometries."""
    program: ShapeProgram
    shapes: dict[str, int] = field(default_factory=dict)
    surfaces: dict[str, int] = field(default_factory=dict)
    curves: dict[str, int] = field(default_factory=dict)
    pcurves: dict[str, int] = field(default_factory=dict)
    locations: dict[str, int] = field(default_factory=dict)


def geometry(s: EdgeScene) -> None:
    """Surfaces, curves, p-curves and locations of the scene."""
    p = s.program
    s.surfaces['plane'] = p.surface(0, 0., 0., 0., 0., 0., 1.)
    s.surfaces['cyl'] = p.surface(1, 0., 0., 0., 0., 0., 1., 1.)
    s.locations['id'] = 0
    s.locations['move'] = p.location(0, 0.5, -1., 2.)
    s.locations['turn'] = p.location(1, 0., 0., 1., 0.3)
    c = s.curves
    c['line'] = p.line((0., 0., 0.), (1., 0., 0.))
    c['circle'] = p.curve(1, 0., 0., 0., 0., 0., 1., 1.)
    c['arc'] = p.curve(4, c['circle'], 0., PI)
    c['bezier'] = p.curve(3, 3, 0., 0., 0., 1., 1., 0., 2., 0., 0.)
    c['bspline'] = p.curve(5, 2, 4, 0., 0., 0., 1., 1., 0., 2., 1., 0., 3., 0., 0., 3, 0., 3, 0.5, 1, 1., 3)
    pc = s.pcurves
    pc['line'] = p.line2d((0., 0.), (1., 0.))
    pc['vline'] = p.line2d((0., 0.), (0., 1.))
    pc['seam2'] = p.line2d((2*PI, 0.), (0., 1.))
    pc['circle'] = p.pcurve(1, 0.5, 0.5, 0.25, 1)
    pc['ellipse'] = p.pcurve(2, 0., 0., 2., 1.)
    pc['bezier'] = p.pcurve(3, 3, 0., 0., 0.5, 1., 1., 0.)
    pc['trimmed'] = p.pcurve(4, pc['circle'], 0., PI)
    pc['offset'] = p.pcurve(5, pc['bezier'], 0.1)
    pc['bspline'] = p.pcurve(6, 2, 4, 0., 0., 1., 1., 2., 1., 3., 0., 3, 0., 3, 0.5, 1, 1., 3)


def edge(s: EdgeScene, name: str, first: int, last: int, curve: str | None, span: tuple[float, float],
         pcurves: list[tuple[str, str]], location: str = 'id') -> int:
    """An edge with a 3D curve (or none), p-curves (name, surface) and its vertices."""
    p = s.program
    e = p.edge()
    if curve:
        p.edge_curve(e, s.curves[curve])
    for pcurve, surface in pcurves:
        p.edge_pcurve(e, s.pcurves[pcurve], s.surfaces[surface], s.locations[location])
    p.edge_range(e, *span)
    p.add(e, first, FORWARD)
    p.add(e, last, REVERSED)
    s.shapes[name] = e
    return e


def shapes(s: EdgeScene) -> None:
    """The vertices, edges and faces of the scene."""
    p, n = s.program, s.shapes
    n['v0'], n['v1'], n['v2'] = p.vertex((0., 0., 0.)), p.vertex((1., 0., 0.)), p.vertex((-1., 0., 0.), 1e-3)
    n['vc'] = p.vertex((1., 0., 0.))
    edge(s, 'line', n['v0'], n['v1'], 'line', (0., 1.), [('line', 'plane')])
    edge(s, 'circle', n['vc'], n['v2'], 'circle', (0., PI), [('line', 'cyl')])
    edge(s, 'late', n['vc'], n['v2'], 'circle', (7., 8.), [('line', 'cyl')])
    edge(s, 'arc', n['vc'], n['v2'], 'arc', (0.5, 2.), [])
    edge(s, 'pc_plane', n['v0'], n['v1'], None, (0., 1.), [('line', 'plane')])
    edge(s, 'pc_cyl', n['vc'], n['v2'], None, (0., PI), [('line', 'cyl')])
    edge(s, 'bare', n['v0'], n['v1'], None, (0., 1.), [])
    edge(s, 'moved', n['v0'], n['v1'], 'line', (0., 1.), [('line', 'plane')], 'move')
    seam = edge(s, 'seam', n['vc'], n['vc'], 'line', (0., 1.), [])
    p.edge_pcurves(seam, s.pcurves['seam2'], s.pcurves['vline'], s.surfaces['cyl'])
    internal = edge(s, 'internal', n['v0'], n['v1'], 'line', (0., 1.), [])
    p.add(internal, n['v2'], INTERNAL)
    n['line_rev'] = p.oriented(n['line'], REVERSED)
    n['line_int'] = p.oriented(n['line'], INTERNAL)
    n['f_plane'] = p.face(s.surfaces['plane'])
    n['f_cyl'] = p.face(s.surfaces['cyl'])
    n['f_plane_moved'] = p.face(s.surfaces['plane'], s.locations['move'])
    n['f_cyl_rev'] = p.oriented(n['f_cyl'], REVERSED)


def edge_scene() -> EdgeScene:
    """The shapes every edge request works on."""
    s = EdgeScene(ShapeProgram())
    geometry(s)
    shapes(s)
    return s


def edge_request(s: EdgeScene, op: int, *fields: float) -> str:
    """One `sh_edge program op ...` line."""
    return ' '.join(['sh_edge', s.program.tokens(), str(op), *map(num, fields)])


def copy_requests(s: EdgeScene) -> list[str]:
    """CopyReplaceVertices, CopyRanges, SetRange3d, CopyPCurves and Copy."""
    n = s.shapes
    edges = ['line', 'circle', 'late', 'arc', 'pc_plane', 'pc_cyl', 'bare', 'moved', 'seam', 'internal',
             'line_rev', 'line_int']
    out = []
    for e in edges:
        for v1, v2 in ((-1, -1), (n['v2'], -1), (-1, n['v0']), (n['v1'], n['v0'])):
            out.append(edge_request(s, 0, n[e], v1, v2))
        out += [edge_request(s, 4, n[e], share) for share in (1, 0)]
        out.append(edge_request(s, 2, n[e], 0.25, 0.75))
    pairs = [('circle', 'late'), ('late', 'circle'), ('line', 'moved'), ('moved', 'line'), ('arc', 'late'),
             ('pc_cyl', 'circle'), ('seam', 'seam'), ('bare', 'line'), ('line', 'line_rev')]
    for to, frm in pairs:
        out += [edge_request(s, 1, n[to], n[frm], a, b) for a, b in ((0., 1.), (0.25, 0.5), (-1., 2.))]
        out.append(edge_request(s, 3, n[to], n[frm]))
    return out


def pcurve_requests(s: EdgeScene) -> list[str]:
    """RemovePCurve, ReplacePCurve, ReassignPCurve, RemoveCurve3d and BuildCurve3d."""
    n, sf, loc, pc = s.shapes, s.surfaces, s.locations, s.pcurves
    out = []
    for e in ('line', 'circle', 'seam', 'moved', 'pc_plane', 'pc_cyl', 'bare', 'line_rev'):
        out += [edge_request(s, 5, n[e], n[f]) for f in ('f_plane', 'f_cyl', 'f_plane_moved')]
        out += [edge_request(s, 6, n[e], sf[k], loc[l]) for k in ('plane', 'cyl') for l in ('id', 'move')]
        out += [edge_request(s, 7, n[e], sf[k]) for k in ('plane', 'cyl')]
        out += [edge_request(s, 8, n[e], pc['bezier'], n[f]) for f in ('f_plane', 'f_cyl', 'f_cyl_rev')]
        out += [edge_request(s, 9, n[e], n[a], n[b]) for a, b in
                (('f_plane', 'f_cyl'), ('f_cyl', 'f_plane'), ('f_cyl', 'f_cyl_rev'), ('f_plane', 'f_plane_moved'))]
        out += [edge_request(s, op, n[e]) for op in (11, 12)]
    return out


# 2D transformations: none, a translation, a rotation, a scale.
TRSF2D = [[0], [1, 0.5, -2.], [2, 1., 1., 0.4], [3, 0., 1., 2.]]


def transform_requests(s: EdgeScene) -> list[str]:
    """TransformPCurve of every p-curve kind."""
    out = []
    for name, pcurve in s.pcurves.items():
        span = (0., PI) if name in ('circle', 'ellipse', 'trimmed') else (0., 1.)
        for trsf in TRSF2D:
            for u_fact in (1., 2., 0.5):
                out.append(edge_request(s, 10, pcurve, *trsf, u_fact, *span))
    return out


def make_requests(s: EdgeScene) -> list[str]:
    """The six MakeEdge members."""
    n, c, pc, sf, loc = s.shapes, s.curves, s.pcurves, s.surfaces, s.locations
    out = []
    for curve in c.values():
        for l in loc.values():
            out.append(edge_request(s, 13, curve, l))
            out.append(edge_request(s, 14, curve, l, 0.25, 0.75))
    for pcurve in (pc['line'], pc['bezier'], pc['trimmed']):
        for f in ('f_plane', 'f_cyl', 'f_plane_moved'):
            out.append(edge_request(s, 15, pcurve, n[f]))
            out.append(edge_request(s, 16, pcurve, n[f], 0.1, 0.9))
        for surface in sf.values():
            for l in loc.values():
                out.append(edge_request(s, 17, pcurve, surface, l))
                out.append(edge_request(s, 18, pcurve, surface, l, 0.2, 0.3))
    out.append(edge_request(s, 16, pc['line'], n['f_plane'], 0.5, 0.5))
    return out


def vertex_requests() -> list[str]:
    """CombineVertex through points and through vertices."""
    cases = [((0., 0., 0.), (1., 0., 0.), 0.1, 0.1), ((0., 0., 0.), (0.5, 0., 0.), 1., 0.1),
             ((0., 0., 0.), (0.5, 0., 0.), 0.1, 1.), ((1., 2., 3.), (1., 2., 3.), 1e-7, 1e-7),
             ((-0., 0., -0.), (0., -0., 0.), 0., 0.), ((1e-300, 0., 0.), (-1e-300, 0., 0.), 1e-7, 2e-7),
             ((0., 0., 0.), (3., 4., 0.), 1., 1.), ((0.1, 0.2, 0.3), (0.3, 0.2, 0.1), 0.05, 0.3)]
    out = []
    for mode in (0, 1):
        for p1, p2, t1, t2 in cases:
            for factor in (1.0001, 1., 2.):
                out.append(' '.join(['sh_vertex', str(mode), *map(num, [*p1, *p2, t1, t2, factor])]))
    return out


def reshape_scene() -> tuple[ShapeProgram, dict[str, int]]:
    """Faces, a shell, a solid, compounds, edges and a wire for the re-shaper."""
    p = ShapeProgram()
    n: dict[str, int] = {}
    plane = p.surface(0, 0., 0., 0., 0., 0., 1.)
    for k in range(4):
        n[f'f{k}'] = p.face(plane)
    n['v0'], n['v1'] = p.vertex((0., 0., 0.)), p.vertex((1., 0., 0.))
    for k in range(3):
        e = p.edge()
        p.edge_curve(e, p.line((0., 0., 0.), (1., 0., 0.)))
        p.edge_range(e, 0., 1.)
        p.add(e, n['v0'], FORWARD)
        p.add(e, n['v1'], REVERSED)
        n[f'e{k}'] = e
    n['wire'] = p.container(WIRE)
    p.add(n['wire'], n['e0'])
    p.add(n['wire'], n['e1'])
    n['shell'] = p.container(SHELL)
    for k in range(3):
        p.add(n['shell'], n[f'f{k}'])
    n['solid'] = p.container(SOLID)
    p.add(n['solid'], n['shell'])
    n['faces'] = p.container(COMPOUND)
    p.add(n['faces'], n['f0'])
    p.add(n['faces'], n['f3'])
    n['shell2'] = p.container(SHELL)
    p.add(n['shell2'], n['f3'])
    n['mixed'] = p.container(COMPOUND)
    for k in ('solid', 'e2', 'wire'):
        p.add(n['mixed'], n[k])
    n['f0_rev'] = p.oriented(n['f0'], REVERSED)
    n['f1_moved'] = p.located(n['f1'], p.location(0, 0., 0., 1.))
    n['e2_rev'] = p.oriented(n['e2'], REVERSED)
    return p, n


# TopAbs_ShapeEnum values used as `until`.
T_COMPOUND, T_SOLID, T_SHELL, T_FACE, T_WIRE, T_EDGE, T_VERTEX, T_SHAPE = 0, 2, 3, 4, 5, 6, 7, 8


def reshape_request(p: ShapeProgram, requests: list[list[int]], applies: list[list[int]]) -> str:
    """One `sh_reshape program nreq (req)... napply (apply)...` line."""
    fields = [len(requests), *[v for r in requests for v in r], len(applies), *[v for a in applies for v in a]]
    return ' '.join(['sh_reshape', p.tokens(), *map(str, fields)])


def reshape_requests() -> list[str]:
    """Replacement scripts, each applied to every container with every member."""
    p, n = reshape_scene()
    scripts = [
        [],
        [[0, n['f1'], n['f3']]],
        [[1, n['f1']]],
        [[0, n['f0'], n['faces']], [1, n['f2']]],
        [[0, n['f0_rev'], n['f3']], [0, n['e0'], n['e2_rev']]],
        [[0, n['f1'], n['shell2']]],
        [[0, n['f1'], n['e2']]],
        [[0, n['shell'], n['shell2']], [0, n['e1'], n['wire']]],
        [[2, 1], [0, n['f1_moved'], n['f3']], [0, n['f0'], n['f1']]],
        [[0, n['f0'], n['f1']], [0, n['f1'], n['f0']]],
        [[0, n['e0'], -1], [0, n['f2'], n['f0']]],
    ]
    targets = [n[k] for k in ('solid', 'shell', 'faces', 'mixed', 'wire', 'f0', 'f0_rev', 'f1_moved', 'e0')] + [-1]
    out = []
    for script in scripts:
        applies = [[0, t, until] for t in targets for until in (T_SHAPE, T_FACE, T_SHELL, T_EDGE)]
        applies += [[1, n[k], T_FACE, mode] for k in ('solid', 'shell', 'faces', 'mixed') for mode in (0, 1, 2)]
        applies += [[1, n['f0'], T_FACE, 0], [1, -1, T_FACE, 0]]
        applies += [[2, t, last] for t in targets for last in (0, 1)] + [[3, t] for t in targets]
        out.append(reshape_request(p, script, applies))
    return out


def append_shapebuild_requests(requests: list[str]) -> None:
    """Append the ShapeBuild requests."""
    s = edge_scene()
    requests.extend(vertex_requests())
    requests.extend(copy_requests(s) + pcurve_requests(s) + transform_requests(s) + make_requests(s))
    requests.extend(reshape_requests())
