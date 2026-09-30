"""Deterministic requests for BRepLib_MakeFace, BRepLib_FindSurface, BRepLib::SameParameter
and BRepLib::ExtendFace (tools/oracle/tktopalgo_probe_breplib_face.cxx).

The surfaces are those of the face reader plus offset surfaces, linear extrusions and
revolutions; the wires are polygons and the wires of bounded faces; the edges for
SameParameter are faces whose 3d curves BuildCurves3d approximates, edges whose p-curve
is parametrized differently from the 3d curve (Bezier, C0 B-spline, shifted ranges),
shells and bounded faces.
"""
from __future__ import annotations
import math
from tktopalgo_corpus_breplib_package import compound, edge, face, pcurve_faces, shell, shells, spline_grid
from tktopalgo_corpus_breplib_records import (
    INF, bezier_surface, cone, cylinder, geom2d_bezier, geom2d_bspline, geom2d_gp, geom_bezier, geom_bspline,
    geom_gp, geom_trimmed, numbers, plane, sphere, torus, trimmed_plane)
from tktopalgo_corpus_faces import square_face

TWO_PI = 2 * math.pi


def s0(surface: list[float]) -> list[float]:
    """A surface2 record of a face-reader surface."""
    return [0] + surface


def offset(value: float, basis: list[float]) -> list[float]:
    return [1, value] + basis


def extrusion(direction, curve: list[float]) -> list[float]:
    return [2, *direction] + curve


def revolution(p, d, curve: list[float]) -> list[float]:
    return [3, *p, *d] + curve


def bez_surface() -> list[float]:
    return bezier_surface([[(0., 0., 0.), (0., 1., 0.2), (0., 2., 0.)], [(1., 0., 0.3), (1., 1., 1.), (1., 2., 0.2)],
                           [(2., 0., 0.), (2., 1., 0.1), (2., 2., 0.)]])


def pinched_surface() -> list[float]:
    """A Bezier surface whose first row of poles is one point (a degenerated U side)."""
    return bezier_surface([[(0., 0., 0.), (0., 0., 0.), (0., 0., 0.)], [(1., 0., 0.3), (1., 1., 1.), (1., 2., 0.2)],
                           [(2., 0., 0.), (2., 1., 0.1), (2., 2., 0.)]])


MERIDIAN = geom_bezier([(1., 0., 0.), (1.5, 0., 1.), (1., 0., 2.)])
LINE_X = geom_gp(0, p=(2., 0., 0.), d=(0., 0., 1.))


def elementary() -> list[list[float]]:
    return [plane(), cylinder(), sphere(), cone(), torus()]


def surfaces2() -> list[list[float]]:
    """Every surface2 kind: face-reader surfaces, offsets, extrusions and revolutions."""
    return [s0(s) for s in elementary() + [bez_surface(), spline_grid(), trimmed_plane(-1., 1., -2., 2.)]] + [
        offset(0.5, s0(plane())), offset(0.3, s0(cylinder(radius=2.))), offset(0.2, s0(bez_surface())),
        extrusion((0., 0., 1.), MERIDIAN), revolution((0., 0., 0.), (0., 0., 1.), MERIDIAN),
        revolution((0., 0., 0.), (0., 0., 1.), LINE_X),
        offset(0.2, extrusion((0., 1., 0.), MERIDIAN)), offset(0.1, revolution((0., 0., 0.), (0., 0., 1.), MERIDIAN)),
        offset(0.1, revolution((0., 0., 0.), (0., 0., 1.), LINE_X))]


def polygon(points, closed: bool = True) -> list[float]:
    return [5, len(points)] + [c for p in points for c in p] + [int(closed)]


def face_wire(surface2: list[float], bounds, reversed_: bool = False) -> list[float]:
    return [6] + surface2 + list(bounds) + [int(reversed_)]


def made_face(surface2: list[float], bounds, tol: float = 1e-7) -> list[float]:
    return [9] + surface2 + list(bounds) + [tol]


SQUARE = [(0., 0., 0.), (1., 0., 0.), (1., 1., 0.), (0., 1., 0.)]
TILTED = [(0., 0., 0.), (1., 0., 0.5), (1., 1., 1.), (0., 1., 0.5)]


def polygons() -> list[list[float]]:
    """Planar, reversed, tilted, slightly and strongly non-planar, open and collinear."""
    return [polygon(SQUARE), polygon(SQUARE[::-1]), polygon(TILTED), polygon([(0., 0., 0.), (2., 0., 0.), (1., 1., 0.)]),
            polygon(SQUARE[:3] + [(0., 1., 1e-8)]), polygon(SQUARE[:3] + [(0., 1., 0.5)]),
            polygon(SQUARE[:3] + [(0., 1., 1e-6)]), polygon(SQUARE, closed=False),
            polygon([(0., 0., 0.), (1., 1., 1.), (2., 2., 2.), (3., 3., 3.)], closed=False),
            polygon([(0., 0., 0.), (1., 0., 0.), (2., 0.1, 0.), (3., 0., 0.), (3., 2., 0.), (0., 2., 0.)])]


def face_wires() -> list[list[float]]:
    """Wires of bounded faces: planes, cylinders, spheres (with poles), cones (with apex), Bezier."""
    apex = -1. / math.sin(0.5)
    return [face_wire(s0(plane()), (0., 2., 0., 1.)), face_wire(s0(plane()), (0., 2., 0., 1.), True),
            face_wire(s0(cylinder()), (0., TWO_PI, 0., 1.)), face_wire(s0(cylinder()), (0., 2., 0., 1.)),
            face_wire(s0(sphere()), (0., TWO_PI, -math.pi / 2, math.pi / 2)),
            face_wire(s0(sphere()), (0., 1., -0.5, 0.5)), face_wire(s0(cone()), (0., TWO_PI, apex, 0.)),
            face_wire(s0(bez_surface()), (0.2, 0.8, 0.1, 0.9)), face_wire(s0(torus()), (0., 1., 0., 1.))]


def gp_face_requests() -> list[str]:
    apex = -1. / math.sin(0.5)
    bounded = {0: [(0., 1., 0., 2.), (1., 0., 2., 0.), (-INF, INF, 0., 1.), (0., 1., -INF, INF), (-INF, INF, -INF, INF),
                   (0., INF, 0., 1.), (-INF, 0., -INF, 1.)],
               1: [(0., math.pi, 0., 1.), (0., TWO_PI, -1., 1.), (7., 8., 0., 1.), (1., 0., 0., 1.), (0., TWO_PI, -INF, INF)],
               2: [(0., TWO_PI, -math.pi / 2, math.pi / 2), (0., 1., -0.5, 0.5), (0., 1., 0.5, -0.5), (-1., 1., -2., 0.)],
               3: [(0., TWO_PI, apex, 0.), (0., TWO_PI, 0., 1.), (1., 2., apex, 1.)],
               4: [(0., TWO_PI, 0., TWO_PI), (0., 1., 0., 1.), (0., TWO_PI, 0., 1.), (3., 1., 0., 1.)]}
    requests = [request('face', [0] + s0(s)) for s in elementary() + [bez_surface()]]
    for i, s in enumerate(elementary()):
        requests += [request('face', [1] + s0(s) + list(b)) for b in bounded[i]]
    wires = polygons()[:3] + face_wires()
    for s, w in [(plane(), wires[0]), (plane(), wires[1]), (plane(), wires[3]), (cylinder(), face_wires()[2]),
                 (cylinder(), face_wires()[3]), (sphere(), face_wires()[4]), (sphere(), face_wires()[5]),
                 (plane(), face_wires()[0]), (plane(), face_wires()[1])]:
        requests += [request('face', [4] + s0(s) + w + [1]), request('face', [4] + s0(s) + w + [0])]
    return requests


def geom_face_requests() -> list[str]:
    requests = [request('face', [2] + s + [1e-7]) for s in surfaces2()]
    requests += [request('face', [2] + s0(sphere()) + [1e-3]), request('face', [2] + s0(pinched_surface()) + [1e-7])]
    boxes = [(0.2, 0.8, 0.1, 0.9), (0.8, 0.2, 0.9, 0.1), (5., -1., 0., 1.), (0., 1., 0., 1.), (-1., 1., -1., 2.)]
    for s in surfaces2():
        requests += [request('face', [3] + s + list(b) + [1e-7]) for b in boxes]
    requests += [request('face', [3] + s0(pinched_surface()) + [0., 1., 0., 1., 1e-3]),
                 request('face', [3] + s0(sphere()) + [0., TWO_PI, -math.pi / 2, math.pi / 2, 1e-3])]
    return requests


def wire_face_requests() -> list[str]:
    requests = []
    for s, w in [(s0(bez_surface()), face_wires()[7]), (s0(plane()), polygons()[0]), (s0(plane()), polygons()[1]),
                 (s0(cylinder()), face_wires()[3]), (s0(torus()), face_wires()[8])]:
        requests += [request('face', [5] + s + w + [1]), request('face', [5] + s + w + [0])]
    for w in polygons() + face_wires():
        requests += [request('face', [6] + w + [0]), request('face', [6] + w + [1])]
    faces = [face(square_face()), made_face(s0(plane()), (0., 1., 0., 1.)), made_face(s0(cylinder()), (0., 1., 0., 1.))]
    requests += [request('face', [7] + f) for f in faces]
    requests += [request('face', [8] + f + polygons()[0]) for f in faces[:2]]
    requests += [request('face', [9] + s0(plane()) + [0, 1e-7, 1] + polygons()[0]),
                 request('face', [9] + s0(plane()) + [1, 1e-7, 0]), request('face', [9] + s0(bez_surface()) + [1, 1e-7, 0]),
                 request('face', [9] + s0(plane()) + [0, 1e-7, 2] + polygons()[0] + polygons()[3])]
    good, bad = (0., 1., 0., 1.), (5., -1., 0., 1.)
    grid = s0(spline_grid())
    requests += [request('face', [10] + grid + list(good) + [1e-7] + grid + list(bad) + [1e-7]),
                 request('face', [10] + grid + list(bad) + [1e-7] + grid + list(good) + [1e-7])]
    return requests


def degenerated_requests() -> list[str]:
    curves = [(geom_gp(1, radius=1e-8), 1e-7), (geom_gp(1, radius=1.), 1e-7), (geom_gp(1, radius=1e-3), 1e-2),
              (geom_bezier([(0., 0., 0.), (1e-9, 0., 0.), (0., 1e-9, 0.)]), 1e-7),
              (geom_bezier([(0., 0., 0.), (1., 0., 0.)]), 1e-7),
              (geom_bspline(1, False, [(0., 0., 0.), (1e-8, 0., 0.), (2e-8, 0., 0.)], [(0., 2), (0.5, 1), (1., 2)]), 1e-7),
              (geom_bspline(1, False, [(0., 0., 0.), (1., 0., 0.), (2., 0., 0.)], [(0., 2), (0.5, 1), (1., 2)]), 1e-7),
              (geom_gp(0), 1e-7), (geom_gp(2), 1e-7)]
    return [request('facedegen', c + [t]) for c, t in curves]


def find_surface_shapes() -> list[list[float]]:
    circle = edge(geom_gp(1), 0., TWO_PI, 1e-7, 1e-7)
    return polygons() + face_wires() + [face(square_face()), made_face(s0(cylinder()), (0., 1., 0., 1.)),
                                        edge(geom_gp(0), 0., 1.), circle, shells()[0], shells()[3],
                                        compound([circle, edge(geom_gp(0), 0., 1.)])]


def find_surface_requests() -> list[str]:
    requests = []
    for shape in find_surface_shapes():
        for tol, only_plane, only_closed in [(-1., 0, 0), (1e-7, 0, 0), (-1., 1, 0), (-1., 0, 1), (1e-3, 1, 1)]:
            requests.append(request('findsurface', shape + [tol, only_plane, only_closed]))
    return requests


def pcurve_edges() -> list[list[float]]:
    """Edges on surfaces whose p-curve parametrization differs from the 3d curve's."""
    seg = geom_gp(0)
    reparam = geom2d_bezier([(0., 0.), (0.8, 0.), (1., 0.)])
    c0 = geom2d_bspline(1, False, [(0., 0.), (0.3, 0.), (1., 0.)], [(0., 2), (0.5, 1), (1., 2)])
    c0_bad = geom2d_bspline(2, False, [(0., 0.), (0.1, 0.), (0.3, 0.), (0.9, 0.), (1., 0.)],
                            [(0., 3), (0.01, 2), (1., 3)])
    circle3d = geom_gp(1, radius=1.)
    return [[7] + seg + [0., 1.] + reparam + [0., 1.] + plane() + [1e-7, 0, 1],
            [7] + seg + [0., 1.] + reparam + [0., 1.] + plane() + [1e-7, 1, 1],
            [7] + seg + [0., 1.] + geom2d_gp(0) + [0., 2.] + plane() + [1e-7, 0, 0],
            [7] + seg + [0., 1.] + geom2d_gp(0) + [0.5, 1.5] + plane() + [1e-7, 0, 0],
            [7] + seg + [0., 1.] + c0 + [0., 1.] + plane() + [1e-7, 0, 1],
            [7] + seg + [0., 1.] + c0_bad + [0., 1.] + plane() + [1e-7, 0, 1],
            [7] + circle3d + [0., math.pi] + geom2d_gp(0, p=(0., 0.), d=(1., 0.)) + [0., math.pi] + cylinder() + [1e-7, 0, 1],
            [7] + circle3d + [0., math.pi] + geom2d_gp(0, p=(0., 0.1), d=(1., 0.)) + [0., math.pi] + cylinder() + [1e-7, 0, 1],
            [7] + seg + [0., 1.] + reparam + [0., 1.] + plane(p=(0., 0., 0.3)) + [1e-7, 0, 1]]


def same_parameter_shapes() -> list[list[float]]:
    rebuilt = [[8] + f + [1, 1] for f in pcurve_faces()[:12]]
    return pcurve_edges() + rebuilt + [[8] + s + [0, 1] for s in shells()[:6]] + [
        [8] + made_face(s, (0., 1., 0., 1.)) + [0, 1] for s in surfaces2()[:8]] + [
        made_face(offset(0.1, revolution((0., 0., 0.), (0., 0., 1.), LINE_X)), (0., 1., 0., 1.)),
        compound(pcurve_edges()[:3])]


def same_parameter_requests() -> list[str]:
    requests = []
    for shape in same_parameter_shapes():
        requests += [request('sameparam', shape + [0, 1e-5, 0]), request('sameparam', shape + [0, 1e-7, 1]),
                     request('sameparam', shape + [1, 1e-5, 1]), request('sameparam', shape + [2, 1e-5, 0]),
                     request('sameparam', shape + [3, 1e-5, 0]), request('sameparam', shape + [4, 1e-5, 0])]
    return requests


def extend_face_requests() -> list[str]:
    shapes = [made_face(s, (0.2, 0.8, 0.1, 0.9)) for s in surfaces2()[:8]] + [
        made_face(s0(cylinder()), (1., 2., 0., 1.)), made_face(s0(cylinder()), (0., 6.2, 0., 1.)),
        made_face(s0(sphere()), (0., TWO_PI, -1., 1.)), made_face(s0(torus()), (0., 6.2, 0., 6.2)),
        face(square_face()), shell(spline_grid()), made_face(offset(0.5, s0(plane())), (0., 1., 0., 1.))]
    flags = [[1, 1, 1, 1], [1, 0, 0, 0], [0, 1, 0, 0], [0, 0, 1, 0], [0, 0, 0, 1], [0, 0, 0, 0]]
    return [request('extendface', s + [0.5] + f) for s in shapes for f in flags] + [
        request('extendface', s + [10.] + [1, 1, 1, 1]) for s in shapes]


def request(op: str, fields: list[float]) -> str:
    prefix = 'blm_' if op in ('face', 'facedegen') else 'blp_'
    return prefix + op + ' ' + numbers(fields)


def append_breplib_face_requests(requests: list[str]) -> None:
    """Append the MakeFace, FindSurface, SameParameter and ExtendFace requests in a fixed order."""
    requests += gp_face_requests()
    requests += geom_face_requests()
    requests += wire_face_requests()
    requests += degenerated_requests()
    requests += find_surface_requests()
    requests += same_parameter_requests()
    requests += extend_face_requests()
