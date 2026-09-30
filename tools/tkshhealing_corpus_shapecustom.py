"""Deterministic requests for ShapeCustom (tools/oracle/tkshhealing_probe_shapecustom.cxx and
tools/oracle/tkshhealing_probe_modification.cxx).

`sc_curve2d` runs IsLinear on pole sets (none, one, coincident, collinear, near-collinear
within and beyond the tolerance, closer than PConfusion, zigzags), ConvertToLine2d on
linear and non-linear B-spline and Bezier p-curves (rational, periodic, reversed range,
equal ends, a huge tolerance) and on other curve kinds, and SimplifyBSpline2d on
B-splines with removable and non-removable knots of several degrees and multiplicities
(a straight cubic, whose null second derivative raises, included).

`sc_restriction` sets each field of ShapeCustom_RestrictionParameters and prints them all.

`sc_modif` builds one scene (tools/tkshhealing_corpus_shcommon.py prelude + a TKTopAlgo
shape program): faces on direct and indirect planes, cylinders, spheres, cones (of
positive and negative semi-angle), tori, trimmed and offset surfaces of them, a Bezier
patch and a face with a mirror location, each bounded by a wire whose edge has a 3D line
and a p-curve on the face (a seam on the cylinders), and a compound of all faces. Every
member of ShapeCustom_TrsfModification (under translations, rotations, scalings, negative
scalings, mirrors and products), ShapeCustom_DirectModification and
ShapeCustom_ConvertToRevolution runs on each face, its edges and vertices, then
BRepTools_Modifier rebuilds each face and the compound, and the messages sent to a
ShapeExtend_MsgRegistrator are printed.
"""
from __future__ import annotations
import math
from dataclasses import dataclass, field
from tkshhealing_corpus_shcommon import (CONE, CYLINDER, PLANE, SPHERE, TORUS, Prelude,
                                         mirror_axis, mirror_plane, mirror_point, product,
                                         rotation, scale, translation)
from tktopalgo_shape_program import COMPOUND, FORWARD, REVERSED, WIRE, ShapeProgram, num

PI = math.pi
Z: tuple[float, float, float] = (0., 0., 1.)
X: tuple[float, float, float] = (1., 0., 0.)
O: tuple[float, float, float] = (0., 0., 0.)


def request(op: str, *fields: float | int | str) -> str:
    return ' '.join([op, *[f if isinstance(f, str) else num(f) for f in fields]])


# ---------------------------------------------------------------- ShapeCustom_Curve2d
def is_linear_requests() -> list[str]:
    sets = [
        ([], 1e-7), ([(1., 2.)], 1e-7), ([(1., 1.), (1., 1.)], 1e-7), ([(0., 0.), (1e-10, 0.)], 1e-7),
        ([(0., 0.), (3., 4.)], 1e-7), ([(0., 0.), (1., 1.), (2., 2.)], 0.),
        ([(0., 0.), (1., 1.), (2., 2.), (3., 3.), (-4., -4.)], 1e-7),
        ([(0., 0.), (1., 1e-4), (2., 0.)], 1e-3), ([(0., 0.), (1., 1e-4), (2., 0.)], 1e-5),
        ([(0., 0.), (1., 1.), (2., 0.), (3., 1.)], 0.5), ([(0., 0.), (1., 1.), (2., 0.), (3., 1.)], 2.),
        ([(5., 5.), (0., 0.), (10., 10.), (2.5, 2.5 + 1e-9)], 1e-8), ([(0., 0.), (1., 0.), (0., 1.)], 1.),
        ([(-1., 3.), (2., -6.), (0.5, -1.5), (1e-3, -3e-3)], 1e-12), ([(0., 0.), (0., 0.), (0., 1.)], 0.),
    ]
    out = []
    for points, tol in sets:
        out.append(request('sc_curve2d', 0, len(points), *[c for p in points for c in p], tol))
    return out


def bspline2d(degree: int, poles, knots, periodic: bool = False) -> list[float | int]:
    """The tp_geom2d_curve B-spline record: 6 degree periodic n poles nk (knot mult)."""
    return [6, degree, periodic, len(poles), *[c for p in poles for c in p], len(knots),
            *[v for k in knots for v in k]]


def bezier2d(poles) -> list[float | int]:
    return [5, len(poles), *[c for p in poles for c in p]]


def convert_requests() -> list[str]:
    line3 = bspline2d(3, [(0., 0.), (1., 1.), (2., 2.), (3., 3.)], [(0., 4), (1., 4)])
    line1 = bspline2d(1, [(0., 1.), (2., 2.), (6., 4.)], [(0., 2), (0.5, 1), (2., 2)])
    uneven = bspline2d(2, [(1., 0.), (1., 5.), (1., 6.), (1., 20.)], [(0., 3), (0.1, 1), (3., 3)])
    curved = bspline2d(2, [(0., 0.), (1., 1.), (2., 0.)], [(0., 3), (1., 3)])
    nearly = bspline2d(3, [(0., 0.), (1., 1e-6), (2., -1e-6), (3., 0.)], [(0., 4), (1., 4)])
    periodic = bspline2d(2, [(0., 0.), (1., 0.), (1., 1.), (0., 1.)], [(0., 1), (1., 1), (2., 1), (3., 1), (4., 1)], True)
    bez = bezier2d([(0., 0.), (2., 1.), (4., 2.)])
    bez_curved = bezier2d([(0., 0.), (2., 3.), (4., 2.)])
    circle = [1, 0., 0., 1., 0., 2., 1]
    line = [0, 0., 0., 1., 1.]
    trimmed = [7, 0., 1., 1, *line3]
    cases = [
        (line3, 0., 1., 1e-7), (line3, 0.25, 0.75, 1e-7), (line3, 1., 0., 1e-7), (line3, 0.5, 0.5, 1e-7),
        (line3, 0., 1., 10.), (line3, -0.5, 2., 1e-7), (line1, 0., 2., 1e-7), (line1, 0.5, 1.5, 1e-9),
        (uneven, 0., 3., 1e-7), (uneven, 0.1, 2., 1e-7), (curved, 0., 1., 1e-7), (curved, 0., 1., 1.),
        (nearly, 0., 1., 1e-7), (nearly, 0., 1., 1e-5), (periodic, 0., 1., 1e-7), (periodic, 0., 1., 2.),
        (bez, 0., 1., 1e-7), (bez, 0.2, 0.9, 0.), (bez_curved, 0., 1., 1e-7), (bez_curved, 0., 1., 3.),
        (circle, 0., 1., 1e-7), (line, 0., 1., 1e-7), (trimmed, 0., 1., 1e-7),
    ]
    return [request('sc_curve2d', 1, *record, c1, c2, tol) for record, c1, c2, tol in cases]


def simplify_requests() -> list[str]:
    cubic_poly = bspline2d(3, [(0., 0.), (1., 2.), (2., -1.), (3., 1.), (4., 0.5), (5., 3.)],
                           [(0., 4), (1., 1), (2., 1), (3., 4)])
    # A single cubic with two knots inserted (removable).
    single = bspline2d(3, [(0., 0.), (1. / 3., 1.), (1., 5. / 3.), (2., 5. / 3.), (8. / 3., 1.), (3., 0.)],
                       [(0., 4), (1., 1), (2., 1), (3., 4)])
    quartic = bspline2d(4, [(0., 0.), (1., 2.), (2., 3.), (3., 1.), (4., 2.), (5., 0.), (6., 1.)],
                        [(0., 5), (1., 2), (2., 5)])
    quadratic = bspline2d(2, [(0., 0.), (1., 1.), (2., 0.), (3., 1.)], [(0., 3), (1., 1), (2., 3)])
    quintic = bspline2d(5, [(float(i), math.sin(i)) for i in range(8)], [(0., 6), (0.5, 1), (1.5, 1), (2., 6)])
    straight = bspline2d(3, [(float(i), 0.) for i in range(6)], [(0., 4), (1., 1), (2., 1), (3., 4)])
    periodic = bspline2d(3, [(1., 0.), (0., 1.), (-1., 0.), (0., -1.)], [(0., 1), (1., 1), (2., 1), (3., 1), (4., 1)], True)
    bare = bspline2d(3, [(0., 0.), (1., 1.), (2., -1.), (3., 0.)], [(0., 4), (1., 4)])
    cases = [(cubic_poly, 1e-7), (cubic_poly, 10.), (single, 1e-7), (single, 1e-12), (quartic, 1e-7),
             (quartic, 5.), (quadratic, 1e-7), (quintic, 1e-7), (quintic, 1.), (straight, 1e-7),
             (periodic, 1e-7), (periodic, 3.), (bare, 1e-7)]
    return [request('sc_curve2d', 2, *record, tol) for record, tol in cases]


def restriction_requests() -> list[str]:
    out = [request('sc_restriction', 0)]
    for field_index in range(16):
        value = 7 if field_index < 2 else 1
        out.append(request('sc_restriction', 2, field_index, value, field_index, 0 if field_index >= 2 else -3))
        out.append(request('sc_restriction', 1, field_index, value))
    out.append(request('sc_restriction', 16, *[v for k in range(16) for v in (k, k % 2)]))
    return out


# ---------------------------------------------------------------- the modification scene
@dataclass
class ModificationScene:
    """The prelude, the program and the named indices of the faces and their shapes."""
    prelude: Prelude
    program: ShapeProgram
    faces: dict[str, int] = field(default_factory=dict)
    edges: dict[str, list[int]] = field(default_factory=dict)
    vertices: dict[str, list[int]] = field(default_factory=dict)
    compound: int = -1
    other_edge: int = -1

    def tokens(self) -> str:
        return self.prelude.tokens()+' '+self.program.tokens()


def prelude_surfaces(pre: Prelude) -> dict[str, int]:
    """The surfaces of the scene, direct and indirect."""
    s: dict[str, int] = {}
    for indirect in (False, True):
        tag = 'ind' if indirect else 'dir'
        s['plane_'+tag] = pre.elementary(PLANE, O, Z, X, indirect)
        s['cyl_'+tag] = pre.elementary(CYLINDER, (0.5, -1., 0.), Z, X, indirect, 1.5)
        s['sph_'+tag] = pre.elementary(SPHERE, O, (0., 1., 0.), (1., 0., 0.), indirect, 2.)
        s['cone_'+tag] = pre.elementary(CONE, O, Z, X, indirect, 0.3, 1.)
        s['cone_neg_'+tag] = pre.elementary(CONE, (1., 1., 0.), (0., 0.6, 0.8), (1., 0., 0.), indirect, -0.4, 2.)
        s['tor_'+tag] = pre.elementary(TORUS, O, Z, X, indirect, 3., 1.)
    s['cyl_trim'] = pre.trimmed(s['cyl_ind'], 0., PI, -1., 2.)
    s['sph_trim'] = pre.trimmed(s['sph_dir'], 0.5, 4., -1., 1.)
    s['cone_neg_trim'] = pre.trimmed(s['cone_neg_dir'], 0., 5., 0., 1.)
    s['cyl_off'] = pre.offset(s['cyl_dir'], 0.5)
    s['sph_off'] = pre.offset(s['sph_ind'], -0.5)
    s['tor_off'] = pre.offset(s['tor_dir'], 0.25)
    return s


def bounded_face(s: ModificationScene, name: str, surface: int, location: int = 0, seam: bool = False) -> None:
    """A face with one wire: an edge with a 3D line and a p-curve line (a seam when asked)."""
    p = s.program
    f = p.face(surface, location, 1e-7 if name != 'plane_dir' else 1e-9)
    v1, v2 = p.vertex((1., 0., 0.), 1e-7 if name != 'plane_dir' else 1e-9), p.vertex((1., 1., 0.), 1e-6)
    e = p.edge()
    p.edge_curve(e, p.line((1., 0., 0.), (0., 1., 0.)), 0, 1e-7 if name != 'plane_dir' else 1e-9)
    p.pcurve_on_face(e, p.line2d((0.25, 0.), (0.6, 0.8)), f)
    p.range_on_face(e, f, 0., 1.)
    p.edge_range(e, 0., 1.)
    p.add(e, v1, FORWARD)
    p.add(e, v2, REVERSED)
    w = p.container(WIRE)
    p.add(w, e, FORWARD)
    edges = [e]
    if seam:
        se = p.edge()
        p.edge_curve(se, p.line((2., -1., 0.), Z))
        p.pcurves_on_face(se, p.line2d((0., 0.), (0., 1.)), p.line2d((2 * PI, 0.), (0., 1.)), f)
        p.range_on_face(se, f, 0., 1.)
        p.edge_range(se, 0., 1.)
        p.add(se, v1, FORWARD)
        p.add(se, v1, REVERSED)
        p.add(w, se, FORWARD)
        p.add(w, se, REVERSED)
        edges.append(se)
    p.add(f, w, FORWARD)
    s.faces[name], s.edges[name], s.vertices[name] = f, edges, [v1, v2]


def modification_scene() -> ModificationScene:
    """Every face of the ShapeCustom modification requests and their compound."""
    pre = Prelude()
    surfaces = prelude_surfaces(pre)
    mirrored = pre.location(mirror_plane((0., 0., 1.), (0.3, 0., 1.)))
    turned = pre.location(product(rotation(O, (1., 1., 0.), 0.7), translation((1., 2., 3.))))
    s = ModificationScene(pre, pre.program())
    for name, surface in surfaces.items():
        bounded_face(s, name, surface, seam=name in ('cyl_dir', 'cyl_ind', 'sph_ind'))
    bezier = s.program.surface(6, 0., 0., 0., 0., 0., 1., 2, 2, 0., 0., 0., 1., 0., 0., 0., 1., 0., 1., 1., 1.)
    bounded_face(s, 'bezier', bezier)
    bounded_face(s, 'cyl_ind_mirrored', surfaces['cyl_ind'], mirrored)
    bounded_face(s, 'cone_neg_turned', surfaces['cone_neg_dir'], turned)
    s.compound = s.program.container(COMPOUND)
    for face in s.faces.values():
        s.program.add(s.compound, face)
    s.other_edge = s.edges['bezier'][0]
    return s


MODIFICATIONS: list[list[float | int]] = [
    [0, *translation((0., 0., 0.)), 0],
    [0, *translation((1., -2., 0.5)), 1],
    [0, *rotation((1., 0., 0.), (0., 0., 1.), 0.9), 0],
    [0, *scale((0., 1., 0.), 2.5), 0],
    [0, *scale(O, -1.5), 0],
    [0, *mirror_point((0.5, 0.5, 0.5)), 0],
    [0, *mirror_axis(O, (0., 1., 0.)), 0],
    [0, *mirror_plane(O, (1., 0., 0.)), 0],
    [0, *product(rotation(O, (0., 1., 0.), 0.4), scale((1., 1., 1.), 0.5)), 0],
    [1], [2],
]


def face_ops(s: ModificationScene, name: str) -> list[int]:
    """Every member on the face, its edges and vertices, then the modifier on the face."""
    f, (v1, v2) = s.faces[name], s.vertices[name]
    ops = [0, f, 2, v1, 2, v2, 5, s.edges[name][0], f, f]
    for e in s.edges[name]:
        ops += [1, e, 3, e, f, e, f, 3, e, f, s.other_edge, f, 4, v1, e, 4, v2, e]
    return ops + [6, f, 7]


def modif_requests(s: ModificationScene) -> list[str]:
    out = []
    for modification in MODIFICATIONS:
        for name in s.faces:
            ops = face_ops(s, name)
            out.append(request('sc_modif', s.tokens(), *modification, 1, len(ops_count(ops)), *ops))
        tail = [6, s.compound, 7, 8, 10]
        out.append(request('sc_modif', s.tokens(), *modification, 1, 4, *tail))
        out.append(request('sc_modif', s.tokens(), *modification, 0, 5, 10, 9, s.faces['cyl_dir'], 3, 1, 7, 8, 6, s.compound))
    return out


def ops_count(ops: list[int]) -> list[int]:
    """The ops of an op list (each op code with its operand count), to count them."""
    widths = {0: 1, 1: 1, 2: 1, 3: 4, 4: 2, 5: 3, 6: 1, 7: 0, 8: 0, 9: 3, 10: 0}
    codes, k = [], 0
    while k < len(ops):
        codes.append(ops[k])
        k += 1 + widths[ops[k]]
    return codes


def message_requests(s: ModificationScene) -> list[str]:
    """SendMsg with and without a registrator, and a registrator dump."""
    f, e = s.faces['cyl_ind'], s.edges['cyl_ind'][0]
    ops = [9, f, 1, 2, 9, f, 2, 1, 9, e, 3, 4, 9, -1, 4, 0, 7, 10]
    return [request('sc_modif', s.tokens(), kind, reg, 6, *ops) for kind in (1, 2) for reg in (0, 1)]


def append_shapecustom_requests(requests: list[str]) -> None:
    """Append every ShapeCustom request family in a fixed order."""
    requests += is_linear_requests()
    requests += convert_requests()
    requests += simplify_requests()
    requests += restriction_requests()
    scene = modification_scene()
    requests += modif_requests(scene)
    requests += message_requests(scene)
