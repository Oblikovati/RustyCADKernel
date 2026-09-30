"""A scene of edges and faces for the ShapeAnalysis, ShapeConstruct and ShapeFix probes.

The scene is built with a TKTopAlgo shape program (tools/tktopalgo_shape_program.py): a
plane, a cylinder and a sphere (faces on them, one moved, one REVERSED), and edges with
3D curves and p-curves that agree, disagree (a vertex off its curve, a p-curve shifted or
reversed along a circle), a seam, p-curve only and bare edges, a degenerated edge at a
sphere pole, edges with a located curve or a located shape, an edge without vertices, a
closed circle, a periodic circle range beyond its period, B-spline curves, p-curves whose
derivatives vanish at an end, and lines that overlap.
"""
from __future__ import annotations
import math
from dataclasses import dataclass, field
from tktopalgo_shape_program import FORWARD, INTERNAL, REVERSED, ShapeProgram

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
    """The surfaces, locations, curves and p-curves."""
    p = s.program
    s.surfaces['plane'] = p.surface(0, 0., 0., 0., 0., 0., 1.)
    s.surfaces['cyl'] = p.surface(1, 0., 0., 0., 0., 0., 1., 1.)
    s.surfaces['sphere'] = p.surface(2, 0., 0., 0., 0., 0., 1., 1.)
    s.locations['id'] = 0
    s.locations['move'] = p.location(0, 0.5, -1., 2.)
    s.locations['turn'] = p.location(1, 0., 0., 1., 0.3)
    c = s.curves
    c['line'] = p.line((0., 0., 0.), (1., 0., 0.))
    c['circle'] = p.curve(1, 0., 0., 0., 0., 0., 1., 1.)
    c['zline'] = p.line((1., 0., 0.), (0., 0., 1.))
    c['long'] = p.line((0., 0., 0.), (1., 0., 0.))
    c['near'] = p.line((0.5, 0., 0.0005), (1., 0., 0.))
    c['far'] = p.line((0., 3., 0.), (1., 0., 0.))
    c['bspline'] = p.curve(5, 2, 4, 0., 0., 0., 1., 1., 0., 2., 1., 0., 3., 0., 0., 3, 0., 3, 0.5, 1, 1., 3)
    pc = s.pcurves
    pc['line'] = p.line2d((0., 0.), (1., 0.))
    pc['line_up'] = p.line2d((0., 0.001), (1., 0.))
    pc['line_back'] = p.line2d((PI, 0.), (-1., 0.))
    pc['vline0'] = p.line2d((0., 0.), (0., 1.))
    pc['vline2pi'] = p.line2d((2*PI, 0.), (0., 1.))
    pc['pole'] = p.line2d((0., PI/2), (1., 0.))
    pc['circle'] = p.pcurve(1, 0.5, 0.5, 0.25, 1)
    pc['trimmed'] = p.pcurve(4, pc['circle'], 0., PI)
    pc['bspline'] = p.pcurve(6, 2, 4, 0., 0., 1., 1., 2., 1., 3., 0., 3, 0., 3, 0.5, 1, 1., 3)
    pc['flat1'] = p.pcurve(3, 3, 0., 0., 0., 0., 1., 0.)
    pc['flat2'] = p.pcurve(3, 4, 0., 0., 0., 0., 0., 0., 1., 1.)
    pc['point'] = p.pcurve(3, 3, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5)


def edge(s: EdgeScene, name: str, vertices: tuple[str | None, str | None], curve: str | None,
         span: tuple[float, float], pcurves: list[tuple[str, str]], location: str = 'id') -> int:
    """An edge with a 3D curve (or none), p-curves (name, surface) and its vertices."""
    p = s.program
    e = p.edge()
    if curve:
        p.edge_curve(e, s.curves[curve], s.locations[location])
    for pcurve, surface in pcurves:
        p.edge_pcurve(e, s.pcurves[pcurve], s.surfaces[surface], s.locations[location])
    p.edge_range(e, *span)
    first, last = vertices
    if first:
        p.add(e, s.shapes[first], FORWARD)
    if last:
        p.add(e, s.shapes[last], REVERSED)
    s.shapes[name] = e
    return e


def vertices(s: EdgeScene) -> None:
    p, n = s.program, s.shapes
    for name, point, tol in (('v0', (0., 0., 0.), 1e-7), ('v1', (1., 0., 0.), 1e-7), ('v1off', (1.001, 0., 0.), 1e-7),
                             ('vc0', (1., 0., 0.), 1e-7), ('vc1', (-1., 0., 0.), 2e-4), ('vtop', (1., 0., 1.), 1e-7),
                             ('vpole', (0., 0., 1.), 1e-5), ('vm0', (0.5, -1., 2.), 1e-7), ('vm1', (1.5, -1., 2.), 1e-7),
                             ('vb0', (0., 0., 0.), 1e-7), ('vb1', (3., 0., 0.), 1e-7), ('vl0', (0., 0., 0.), 1e-7),
                             ('vl1', (2., 0., 0.), 1e-7), ('vn0', (0.5, 0., 0.0005), 1e-7), ('vn1', (1.5, 0., 0.0005), 1e-7),
                             ('vf0', (0., 3., 0.), 1e-7), ('vf1', (1., 3., 0.), 1e-7), ('vlate', (0.7539022543433046, 0.6569865987187891, 0.), 1e-7)):
        n[name] = p.vertex(point, tol)


def edges(s: EdgeScene) -> None:
    """The edges of the scene."""
    edge(s, 'line', ('v0', 'v1'), 'line', (0., 1.), [('line', 'plane')])
    edge(s, 'line_off', ('v0', 'v1off'), 'line', (0., 1.), [('line', 'plane')])
    edge(s, 'circle', ('vc0', 'vc1'), 'circle', (0., PI), [('line', 'cyl')])
    edge(s, 'circle_up', ('vc0', 'vc1'), 'circle', (0., PI), [('line_up', 'cyl')])
    edge(s, 'circle_back', ('vc0', 'vc1'), 'circle', (0., PI), [('line_back', 'cyl')])
    seam = edge(s, 'seam', ('vc0', 'vtop'), 'zline', (0., 1.), [])
    s.program.edge_pcurves(seam, s.pcurves['vline2pi'], s.pcurves['vline0'], s.surfaces['cyl'])
    edge(s, 'pc_only', ('v0', 'v1'), None, (0., 1.), [('line', 'plane')])
    edge(s, 'bare', ('v0', 'v1'), None, (0., 1.), [])
    deg = edge(s, 'deg', ('vpole', 'vpole'), None, (0., 2*PI), [('pole', 'sphere')])
    s.program.edge_flags(deg, True, True, True)
    edge(s, 'moved', ('vm0', 'vm1'), 'line', (0., 1.), [('line', 'plane')], 'move')
    edge(s, 'novert', (None, None), 'line', (0., 1.), [('line', 'plane')])
    edge(s, 'closed', ('vc0', 'vc0'), 'circle', (0., 2*PI), [('line', 'cyl')])
    edge(s, 'late', ('vlate', 'vc1'), 'circle', (7., 8.), [('line', 'cyl')])
    edge(s, 'bspline', ('vb0', 'vb1'), 'bspline', (0., 1.), [('bspline', 'plane')])
    nsp = edge(s, 'bspline_nsp', ('vb0', 'vb1'), 'bspline', (0., 1.), [('bspline', 'plane')])
    s.program.edge_flags(nsp, False, True, False)
    for name in ('flat1', 'flat2', 'point'):
        edge(s, 'pc_'+name, ('v0', 'v1'), None, (0., 1.), [(name, 'plane')])
    edge(s, 'long', ('vl0', 'vl1'), 'long', (0., 2.), [])
    edge(s, 'near', ('vn0', 'vn1'), 'near', (0., 1.), [])
    edge(s, 'far', ('vf0', 'vf1'), 'far', (0., 1.), [])
    internal = edge(s, 'with_internal', ('v0', 'v1'), 'line', (0., 1.), [])
    s.program.add(internal, s.shapes['vc1'], INTERNAL)


def faces_and_copies(s: EdgeScene) -> None:
    p, n, sf, loc = s.program, s.shapes, s.surfaces, s.locations
    n['f_plane'] = p.face(sf['plane'])
    n['f_cyl'] = p.face(sf['cyl'])
    n['f_sphere'] = p.face(sf['sphere'])
    n['f_plane_moved'] = p.face(sf['plane'], loc['move'])
    n['f_cyl_rev'] = p.oriented(n['f_cyl'], REVERSED)
    for name in ('line', 'circle', 'seam', 'circle_back', 'pc_only'):
        n[name+'_rev'] = p.oriented(n[name], REVERSED)
    n['line_located'] = p.located(n['line'], loc['turn'])


def edge_scene() -> EdgeScene:
    """The shapes every edge request works on."""
    s = EdgeScene(ShapeProgram())
    geometry(s)
    vertices(s)
    edges(s)
    faces_and_copies(s)
    return s
