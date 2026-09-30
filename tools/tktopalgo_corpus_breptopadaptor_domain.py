"""Deterministic requests for the BRepTopAdaptor domains of IntStart and IntPatch
(tools/oracle/tktopalgo_probe_breptopadaptor_domain.cxx, docs/topalgo-design.md section 8).

Every request carries its face records (tktopalgo_corpus_faces.py and
tktopalgo_corpus_adaptor_faces.py) and never relies on an earlier request:
- `bta_sob`: IntStart_SearchOnBoundaries of a quadric on the edges of planar, holed,
  trimmed, cylindrical, spherical, conical, toroidal, Bezier and B-spline faces, with two
  tolerance pairs, with and without the regularity recheck;
- `bta_inside`: IntStart_SearchInside of a quadric on the same faces;
- `bta_intersection`: IntPatch_Intersection of two faces built with BRep_Builder (planes,
  a holed square, a trimmed plane, cylinders, a sphere, a cone, a torus patch, Bezier and
  B-spline patches), in both orders, with three option sets.
"""
from __future__ import annotations
import math
from tktopalgo_corpus_adaptor_faces import adaptor_faces
from tktopalgo_corpus_faces import (CYLINDER, FORWARD, PLANE, SEGMENT_3D, FaceSpec, cylinder_face,
                                    polygon_wire, rectangle)

PLANE_Q, CYLINDER_Q, SPHERE_Q, CONE_Q = range(4)
# The faces of the searches, by name in adaptor_faces().
SEARCH_FACES = ('square', 'holed', 'trimmed', 'disc', 'cylinder', 'sphere', 'cone', 'torus_patch',
                'bezier_surface', 'bspline', 'bspline_c1', 'bspline_holed', 'toleranced_3d')
# (tolerance on the boundary, tangency tolerance) pairs.
SEARCH_TOLERANCES = ((1e-7, 1e-7), (1e-4, 1e-6))
# IntStart_SearchOnBoundaries.gxx (TreatLC) dereferences the null 3d curve of a straight
# edge without one when the edge crosses a cylinder, so the planar faces whose edges have
# only pcurves are not searched against the cylinders (native crash; the toleranced square,
# whose edges carry 3d lines, runs that branch).
NO_3D_LINE_FACES = ('square', 'holed', 'trimmed')
# (geomint, keeprline, postwl) option sets of IntPatch_Intersection::Perform.
INTERSECTION_OPTIONS = ((1, 0, 1), (0, 0, 1), (1, 1, 0))


def numbers(values: list[float]) -> str:
    """The fields of a request, with every double spelled exactly."""
    return ' '.join(repr(float(v)) for v in values)


def quadric(kind: int, location: tuple[float, float, float], direction: tuple[float, float, float],
            x_direction: tuple[float, float, float], r1: float = 0.0, r2: float = 0.0) -> list[float]:
    """A `ga_intsurf_quadric` record: kind, the gp_Ax3 frame and the radii."""
    return [kind, *location, *direction, *x_direction, r1, r2]


def search_quadrics() -> list[list[float]]:
    """Quadrics that cross the faces of the searches (their boundaries and insides), and
    planes through their vertices (x = 0 holds whole edges of the squares; x + y = 2 and
    x + y = 10 pass through corners of the squares and of the hole)."""
    return [quadric(PLANE_Q, (1.0, 0.0, 0.0), (1.0, 0.0, 0.0), (0.0, 1.0, 0.0)),
            quadric(PLANE_Q, (0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (0.0, 1.0, 0.0)),
            quadric(PLANE_Q, (1.0, 1.0, 0.0), (1.0, 1.0, 0.0), (0.0, 0.0, 1.0)),
            quadric(PLANE_Q, (5.0, 5.0, 0.0), (1.0, 1.0, 0.0), (0.0, 0.0, 1.0)),
            quadric(PLANE_Q, (0.0, 0.0, 0.3), (0.2, 0.1, 1.0), (1.0, 0.0, -0.2)),
            quadric(PLANE_Q, (5.0, 5.0, 0.0), (1.0, 0.3, 0.0), (0.0, 0.0, 1.0)),
            quadric(CYLINDER_Q, (0.0, 0.0, 0.0), (0.0, 0.0, 1.0), (1.0, 0.0, 0.0), 1.5),
            quadric(CYLINDER_Q, (5.0, 5.0, 0.0), (0.0, 0.0, 1.0), (1.0, 0.0, 0.0), 3.0),
            quadric(SPHERE_Q, (1.5, 1.5, 0.0), (0.0, 0.0, 1.0), (1.0, 0.0, 0.0), 1.2),
            quadric(CONE_Q, (0.0, 0.0, -1.0), (0.0, 0.0, 1.0), (1.0, 0.0, 0.0), 0.5, 1.0)]


def plane_square(location: tuple[float, float, float], normal: tuple[float, float, float],
                 half: float, with_3d: bool = False) -> FaceSpec:
    """The square [-half, half]^2 of the plane gp_Ax3(location, normal); its edges carry
    3d lines with `with_3d` (needed when an edge crosses a cylinder, see NO_3D_LINE_FACES)."""
    face = FaceSpec([PLANE, *location, *normal])
    if not with_3d:
        polygon_wire(face, rectangle(-half, -half, half, half))
        return face
    points = rectangle(-half, -half, half, half)
    vertices = [face.vertex(u, v) for u, v in points]
    edges = [face.edge(SEGMENT_3D, [*points[i], *points[(i + 1) % 4]], vertices[i], vertices[(i + 1) % 4])
             for i in range(4)]
    face.wire([(edge, FORWARD) for edge in edges])
    return face


def placed_cylinder(location: tuple[float, float, float], axis: tuple[float, float, float],
                    radius: float, height: float) -> FaceSpec:
    """The lateral face of cylinder_face() on the frame gp_Ax3(location, axis)."""
    face = cylinder_face(radius, height)
    face.surface = [CYLINDER, *location, *axis, radius]
    return face


def intersection_pairs() -> list[tuple[FaceSpec, FaceSpec]]:
    """Pairs of faces that intersect: plane-plane, plane-quadric, quadric-quadric,
    plane-spline and spline-spline."""
    faces = adaptor_faces()
    vertical = plane_square((5.0, 5.0, 0.0), (1.0, 0.0, 0.0), 20.0)
    return [(faces['holed'], vertical),
            (faces['holed'], plane_square((5.0, 5.0, 0.0), (1.0, 0.3, 0.2), 20.0)),
            (faces['holed'], plane_square((5.0, 5.0, 0.0), (0.0, 0.0, 1.0), 4.0)),
            (faces['holed'], plane_square((5.0, 5.0, 0.0), (1.0, 1.0, 0.0), 20.0)),
            (faces['square'], plane_square((0.0, 1.0, 0.0), (1.0, 0.0, 0.0), 5.0)),
            (plane_square((0.0, 0.0, 2.5), (0.0, 0.0, 1.0), 5.0), faces['cylinder']),
            (plane_square((0.0, 0.0, 2.5), (0.3, 0.1, 1.0), 6.0), faces['cylinder']),
            (plane_square((0.0, 0.0, 2.5), (1.0, 0.0, 0.0), 6.0, with_3d=True), faces['cylinder']),
            (plane_square((0.0, 0.0, 0.5), (0.0, 0.0, 1.0), 5.0), faces['sphere']),
            (plane_square((0.0, 0.0, 1.0), (0.0, 0.2, 1.0), 6.0), faces['cone']),
            (plane_square((0.0, 0.0, 0.5), (0.0, 0.0, 1.0), 6.0), faces['torus_patch']),
            (faces['bspline'], plane_square((3.0, 2.0, 0.0), (1.0, 0.0, 0.0), 10.0)),
            (faces['bspline_holed'], plane_square((3.0, 2.0, 0.0), (1.0, 0.0, 0.0), 10.0)),
            (faces['bspline_c1'], plane_square((3.0, 2.0, 0.0), (0.0, 1.0, 0.0), 10.0)),
            (faces['bezier_surface'], plane_square((1.5, 1.5, 0.0), (1.0, 0.2, 0.0), 10.0)),
            (faces['trimmed'], placed_cylinder((5.0, 5.0, -1.0), (0.0, 0.0, 1.0), 3.0, 5.0)),
            (faces['disc'], placed_cylinder((0.5, 0.0, -1.0), (0.0, 0.0, 1.0), 1.0, 3.0)),
            (faces['toleranced_3d'], plane_square((1.0, 1.0, 0.0), (1.0, 0.0, 0.0), 5.0)),
            (faces['cylinder'], placed_cylinder((-4.0, 0.0, 2.5), (1.0, 0.0, 0.0), 1.5, 8.0)),
            (faces['sphere'], placed_cylinder((0.5, 0.0, -3.0), (0.0, 0.0, 1.0), 1.0, 6.0)),
            (faces['bspline'], faces['bezier_surface'])]


def append_breptopadaptor_domain_requests(requests: list[str]) -> None:
    """Append the IntStart searches and the IntPatch intersections on face domains."""
    faces = adaptor_faces()
    for name in SEARCH_FACES:
        face = faces[name]
        for q in search_quadrics():
            crashes = name in NO_3D_LINE_FACES and q[0] == CYLINDER_Q
            for tolb, tolt in SEARCH_TOLERANCES:
                for recheck in (0, 1):
                    if not crashes:
                        requests.append('bta_sob ' + numbers([*face.numbers(), 1, *q, tolb, tolt, recheck]))
            requests.append('bta_inside ' + numbers([*face.numbers(), 1, *q, 1e-7, 1e-6]))
    for first, second in intersection_pairs():
        for a, b in ((first, second), (second, first)):
            for options in INTERSECTION_OPTIONS:
                requests.append('bta_intersection ' + numbers(
                    [*a.numbers(), 1, *b.numbers(), 1, 1e-7, 1e-7, *options]))
