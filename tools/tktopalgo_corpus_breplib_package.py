"""Deterministic requests for the BRepLib package functions
(tools/oracle/tktopalgo_probe_breplib_package.cxx).

The shapes are faces of the face readers (pcurve-only edges on planes, cylinders, spheres,
cones, tori and spline patches, edges with 3d lines), BRepLib_MakeShell shells (planes,
cylinders, spheres, tori, cones, Bezier and B-spline surfaces with several C2 patches,
trimmed surfaces), compounds of them and BRepLib_MakeEdge edges with enlarged vertices.
Each family runs one package function on each shape and prints the shape digest after it.
"""
from __future__ import annotations
import math
from tktopalgo_corpus_adaptor_faces import (
    bezier_surface_face, bspline_surface_face, cone_face, toleranced_square_face, torus_face, torus_patch_face)
from tktopalgo_corpus_breplib_records import (
    bezier_surface, bspline_surface, cone, cylinder, geom_bezier, geom_bspline, geom_gp, geom_trimmed, numbers,
    plane, sphere, torus, trimmed_plane, vertices)
from tktopalgo_corpus_faces import (
    bezier_polygon_face, cusp_face, cylinder_face, disc_face, flat_face, flipped_cylinder_face, gap_face,
    holed_face, internal_edge_face, sphere_face, square_face, trimmed_face)


def face(spec) -> list[float]:
    return [0] + spec.numbers()


def shell(surface: list[float], bounds=None, segment: bool = False) -> list[float]:
    head = [1] + surface
    return head + ([0] if bounds is None else [1, *bounds]) + [int(segment)]


def compound(children: list[list[float]]) -> list[float]:
    return [2, len(children)] + [c for child in children for c in child]


def edge(curve: list[float], p1: float, p2: float, t1: float = 1e-7, t2: float = 1e-7) -> list[float]:
    return [3] + curve + [p1, p2, t1, t2]


def spline_grid() -> list[float]:
    """A bicubic B-spline surface with two C1 knots in U and one in V (several C2 patches)."""
    poles = [[(float(i), float(j), 0.3 * math.sin(i + 2 * j)) for j in range(6)] for i in range(7)]
    return bspline_surface(3, 3, poles, [(0., 4), (1., 2), (2., 1), (3., 4)], [(0., 4), (1., 2), (2., 4)])


def pcurve_faces() -> list[list[float]]:
    """Faces whose edges have only curves on surface (and two with 3d lines)."""
    return [face(s) for s in (square_face(), holed_face(), disc_face(), cylinder_face(), sphere_face(),
                              bezier_polygon_face(sides=6), gap_face(), trimmed_face(), internal_edge_face(),
                              cusp_face(), flat_face(), flipped_cylinder_face(), cone_face(), torus_face(),
                              torus_patch_face(), bezier_surface_face(), bspline_surface_face(),
                              toleranced_square_face(), toleranced_square_face(tol=1e-4))]


def shells() -> list[list[float]]:
    """BRepLib_MakeShell shells of every surface kind."""
    bez = bezier_surface([[(0., 0., 0.), (0., 1., 0.2), (0., 2., 0.)], [(1., 0., 0.3), (1., 1., 1.), (1., 2., 0.2)],
                          [(2., 0., 0.), (2., 1., 0.1), (2., 2., 0.)]])
    return [shell(plane(), (0., 2., 0., 1.)), shell(cylinder(), (0., 2 * math.pi, 0., 3.)),
            shell(cylinder(), (0., 2., -1., 1.)), shell(sphere(radius=2.)), shell(torus()), shell(cone(), (0., 3., 0., 2.)),
            shell(bez), shell(spline_grid()), shell(spline_grid(), segment=True),
            shell(trimmed_plane(-1., 1., -2., 2.)), shell(sphere(), (0., 1., -0.5, 0.5))]


def edges() -> list[list[float]]:
    """BRepLib_MakeEdge edges with enlarged vertices."""
    line, circle = geom_gp(0), geom_gp(1)
    bez = geom_bezier([(0., 0., 0.), (1., 1., 0.), (2., 0., 0.), (3., 1., 1.)])
    spl = geom_bspline(3, False, [(0., 0., 0.), (1., 2., 0.), (2., -1., 0.), (3., 1., 0.), (4., 0., 1.)],
                       [(0., 4), (0.5, 1), (1., 4)])
    return [edge(line, 0., 1., 0.1, 0.2), edge(line, 0., 1., 0.6, 0.6), edge(circle, 0., math.pi, 0.3, 1e-7),
            edge(circle, 0., 2 * math.pi, 0.5, 0.5), edge(bez, 0., 1., 0.2, 0.05), edge(spl, 0., 1., 0.4, 0.4),
            edge(geom_trimmed(0., 3., line), 0.5, 2., 0.1, 0.1), edge(line, -1e5, 1e5, 1., 1.)]


def all_shapes() -> list[list[float]]:
    shapes = pcurve_faces() + shells() + edges()
    return shapes + [compound(shells()[:2] + edges()[:2]), compound([pcurve_faces()[0], compound(edges()[2:4])])]


def request(op: str, fields: list[float]) -> str:
    return f'blp_{op} ' + numbers(fields)


def build3d_requests() -> list[str]:
    requests = []
    for shape in pcurve_faces() + shells()[:4] + edges()[:2]:
        requests.append(request('build3d', shape + [0, 1e-5, 2, 14, 0]))
        requests.append(request('build3d', shape + [2, 1e-3, 1, 8, 30]))
    for shape in pcurve_faces()[:4]:
        requests.append(request('build3d', shape + [1, 0., 0, 0, 0]))
    return requests


def per_shape(op: str, tail: list[float], shapes=None) -> list[str]:
    return [request(op, shape + tail) for shape in (all_shapes() if shapes is None else shapes)]


def regularity_requests() -> list[str]:
    requests = []
    for shape in shells() + [face(cylinder_face()), face(sphere_face())] + [compound(shells()[:3])]:
        requests += [request('regularity', shape + [0, 1e-10, 0]), request('regularity', shape + [2, 1e-2, 0]),
                     request('regularity', shape + [1, 1e-10, 2, 1, 2])]
    return requests


def valid_range2_requests() -> list[str]:
    line, circle = geom_gp(0), geom_gp(1)
    bez = geom_bezier([(0., 0., 0.), (1., 1., 0.), (2., 0., 0.)])
    cases = [(line, 0., 10., 1e-7, (0., (0., 0., 0.), 0.5), (10., (10., 0., 0.), 0.5)),
             (line, 0., 10., 1e-7, (0., (0., 0., 0.), 6.), (10., (10., 0., 0.), 6.)),
             (line, 0., 10., 1e-7, (0., (5., 5., 0.), 0.5), (10., (10., 0., 0.), 0.5)),
             (circle, 0., 3., 1e-7, (0., (1., 0., 0.), 0.1), (3., (math.cos(3.), math.sin(3.), 0.), 0.1)),
             (bez, 0., 1., 1e-7, (0., (0., 0., 0.), 0.3), (1., (2., 0., 0.), 0.3)),
             (bez, 0., 1., 1e-7, (0., (0., 0., 0.), 3.), (1., (2., 0., 0.), 3.)),
             (line, -2e100, 2e100, 1e-7, (-2e100, (0., 0., 0.), 1.), (2e100, (0., 0., 0.), 1.)),
             (line, 0., 1e-8, 1e-7, (0., (0., 0., 0.), 1e-9), (1e-8, (1e-8, 0., 0.), 1e-9))]
    return [request('validrange2', c + [a, b, tol, v1[0], *v1[1], v1[2], v2[0], *v2[1], v2[2]])
            for c, a, b, tol, v1, v2 in cases]


def bounding_requests() -> list[str]:
    table = [(0., 0., 0., 0.1), (0.05, 0., 0., 0.2), (1., 0., 0., 0.1), (0., 0., 0., 0.5), (3., 1., -2., 0.25),
             (0., 0., 1e-20, 0.1), (-1., 2., 0.5, 1.)]
    lists = [[], [0], [0, 1], [1, 0], [0, 2], [0, 3], [3, 0], [0, 5], [0, 1, 2], [2, 1, 0], [0, 1, 2, 3, 4, 5, 6],
             [4, 6, 0], [0, 0, 1]]
    return ['blp_bounding ' + numbers(vertices(table) + [len(l)] + l) for l in lists]


def shell_requests() -> list[str]:
    surfaces = [(plane(), (0., 2., 0., 1.)), (cylinder(), None), (cylinder(), (0., 2 * math.pi, 0., 3.)),
                (sphere(), None), (torus(), None), (cone(), (0., 3., 0., 2.)), (spline_grid(), None),
                (spline_grid(), (0.5, 2.5, 0.2, 1.8)), (trimmed_plane(-1., 1., -2., 2.), None), (plane(), None)]
    requests = []
    for s, bounds in surfaces:
        b = [0] if bounds is None else [1, *bounds]
        for segment in (0, 1):
            for mode in (0, 1):
                requests.append(request('shell', s + b + [segment, mode]))
    return requests


def mesh_requests() -> list[str]:
    shapes = shells()[:8] + [face(square_face()), face(cylinder_face()), compound(shells()[:3])]
    requests = []
    for shape in shapes:
        requests += [request('normals', shape + [4, 1, 0.001, 0]), request('normals', shape + [3, 1, 0.5, 1]),
                     request('normals', shape + [3, 0, 0.001, 0]), request('deflection', shape + [4, 1]),
                     request('deflection', shape + [3, 0]), request('trinormals', shape + [3, 1, 0]),
                     request('trinormals', shape + [3, 1, 1]), request('trinormals', shape + [2, 0, 1])]
    return requests


def append_breplib_package_requests(requests: list[str]) -> None:
    """Append the package-function requests in a fixed order."""
    requests += build3d_requests()
    requests += per_shape('samerange', [1e-12, 1e-5])
    # UpdateEdgeTol on a SameParameter edge without 3d curve reads an unloaded adaptor
    # natively (a crash): only shapes whose edges have 3d curves.
    with_3d = shells() + edges() + [face(toleranced_square_face())]
    requests += per_shape('edgetol', [0, 1e-5, 1.], with_3d) + per_shape('edgetol', [1, 1e-4, 1e-2], with_3d)
    requests += regularity_requests()
    requests += per_shape('continuity', [1e-10], shells()) + per_shape('continuity', [0.1], shells()[:4])
    requests += per_shape('inner', [])
    requests += per_shape('sort', [])
    requests += bounding_requests()
    requests += per_shape('validrange', [])
    requests += valid_range2_requests()
    requests += per_shape('pcurveplane', [0]) + per_shape('pcurveplane', [1], pcurve_faces()[:6])
    requests += per_shape('pcurveplane', [2], pcurve_faces()[:6])
    requests += per_shape('ccos', [0], shells() + [face(toleranced_square_face())]) + per_shape('ccos', [1], shells()[:2])
    requests += shell_requests()
    requests += mesh_requests()
