"""Face records of the BRepTopAdaptor and IntCurvesFace corpora (`tp_face` records).

They add, to the BRepClass faces of tktopalgo_corpus_faces.py, the surfaces the curve-face
intersections meet: the lateral face of a cone, a whole torus (two seams) and a torus
patch, a Bezier surface patch, B-spline surface patches (one with a C1 knot, so that
IntCurvesFace samples it by C2 intervals, and one with a hole), and squares whose edges
carry 3d lines of tolerance 0.1 (and the same edges without 3d curves), for the boundary
tolerance of IntCurvesFace_Intersector; and, for the FClass2d paths, a thin half ring
(refined polygons), a wire the WireExplorer cannot walk, a one-segment wire and a sphere
whose pole edges are not flagged degenerated.
"""
from __future__ import annotations
import math
from tktopalgo_corpus_faces import (ARC, BEZIER_SURFACE, BSPLINE_SURFACE, CONE, FORWARD, LINE, REVERSED, SEAM,
                                    SEGMENT, SEGMENT_3D, SPHERE, TORUS, Z_PLANE, FaceSpec, named_faces, polygon_wire,
                                    rectangle)

TWO_PI = 2.0 * math.pi


def cone_face(radius: float = 2.0, semi_angle: float = 0.4, height: float = 3.0) -> FaceSpec:
    """The lateral face of a cone, u in [0, 2pi], v in [0, height], with its seam."""
    face = FaceSpec([CONE, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, radius, semi_angle])
    bottom, top = face.vertex(0.0, 0.0), face.vertex(0.0, height)
    e_bottom = face.edge(LINE, [0.0, 0.0, 1.0, 0.0, 0.0, TWO_PI], bottom, bottom)
    e_seam = face.edge(SEAM, [TWO_PI, 0.0, TWO_PI, height, -TWO_PI, 0.0], bottom, top)
    e_top = face.edge(LINE, [0.0, height, 1.0, 0.0, 0.0, TWO_PI], top, top)
    face.wire([(e_bottom, FORWARD), (e_seam, FORWARD), (e_top, REVERSED), (e_seam, REVERSED)])
    return face


def torus_face(major: float = 3.0, minor: float = 1.0) -> FaceSpec:
    """A whole torus: the seams v = 0 (= 2pi) and u = 2pi (= 0) through one vertex."""
    face = FaceSpec([TORUS, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, major, minor])
    corner = face.vertex(0.0, 0.0)
    s_v = face.edge(SEAM, [0.0, 0.0, TWO_PI, 0.0, 0.0, TWO_PI], corner, corner)
    s_u = face.edge(SEAM, [TWO_PI, 0.0, TWO_PI, TWO_PI, -TWO_PI, 0.0], corner, corner)
    face.wire([(s_v, FORWARD), (s_u, FORWARD), (s_v, REVERSED), (s_u, REVERSED)])
    return face


def torus_patch_face(major: float = 3.0, minor: float = 1.0) -> FaceSpec:
    """The patch [0, pi] x [-pi/2, pi/2] of a torus (its outer half)."""
    face = FaceSpec([TORUS, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, major, minor])
    polygon_wire(face, rectangle(0.0, -0.5 * math.pi, math.pi, 0.5 * math.pi))
    return face


def grid_poles(nu: int, nv: int, size: tuple[float, float], bump: float) -> list[float]:
    """nu x nv poles over [0, size] with a height pattern, U rows first."""
    values: list[float] = []
    for i in range(nu):
        for j in range(nv):
            x, y = size[0] * i / (nu - 1), size[1] * j / (nv - 1)
            values += [x, y, bump * math.sin(1.3 * i + 0.7 * j) * (1 + 0.1 * i)]
    return values


def bezier_surface_face() -> FaceSpec:
    """A 4 x 4 Bezier patch over [0, 3]^2 with its four boundary edges."""
    face = FaceSpec([BEZIER_SURFACE, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 4, 4, *grid_poles(4, 4, (3.0, 3.0), 0.8)])
    polygon_wire(face, rectangle(0.0, 0.0, 1.0, 1.0))
    return face


def bspline_surface_face(c1_knot: bool = False, hole: bool = False) -> FaceSpec:
    """A bicubic x biquadratic B-spline patch on [0, 3]^2 (7 x 5 poles, a double knot
    at u = 2 with `c1_knot`), with its boundary edges and optionally the hole [1, 2]^2."""
    if c1_knot:
        u_knots = [4, 0.0, 4, 1.0, 1, 2.0, 2, 3.0, 4]
        nu = 7
    else:
        u_knots = [4, 0.0, 4, 1.0, 1, 2.0, 1, 3.0, 4]
        nu = 6
    v_knots = [4, 0.0, 3, 1.0, 1, 2.0, 1, 3.0, 3]
    record = [BSPLINE_SURFACE, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 3, 2, nu, 5,
              *grid_poles(nu, 5, (6.0, 4.0), 0.9), *u_knots, *v_knots]
    face = FaceSpec(record)
    polygon_wire(face, rectangle(0.0, 0.0, 3.0, 3.0))
    if hole:
        polygon_wire(face, rectangle(1.0, 1.0, 2.0, 2.0), orientation=REVERSED)
    return face


def toleranced_square_face(with_3d: bool = True, tol: float = 0.1) -> FaceSpec:
    """The square [0, 2]^2 whose edges have the tolerance `tol` (the face 1e-7): with 3d
    lines (SEGMENT_3D) or with pcurves only."""
    face = FaceSpec(list(Z_PLANE))
    points = rectangle(0.0, 0.0, 2.0, 2.0)
    vertices = [face.vertex(u, v, tol) for u, v in points]
    kind = SEGMENT_3D if with_3d else SEGMENT
    edges = [face.edge(kind, [*points[i], *points[(i + 1) % 4]], vertices[i], vertices[(i + 1) % 4], tol=tol)
             for i in range(4)]
    face.wire([(edge, FORWARD) for edge in edges])
    return face


def crescent_face(radius: float = 10.0, width: float = 0.001) -> FaceSpec:
    """A thin half ring: the upper halves of the circles of radius `radius` and
    `radius - width`, so thin that the FClass2d polygon is refined."""
    face = FaceSpec(list(Z_PLANE))
    inner = radius - width
    a, b, c, d = face.vertex(radius, 0.0), face.vertex(-radius, 0.0), face.vertex(-inner, 0.0), face.vertex(inner, 0.0)
    edges = [face.edge(ARC, [0.0, 0.0, radius, 0.0, math.pi, 1], a, b),
             face.edge(SEGMENT, [-radius, 0.0, -inner, 0.0], b, c),
             face.edge(ARC, [0.0, 0.0, inner, -math.pi, 0.0, 0], c, d),
             face.edge(SEGMENT, [inner, 0.0, radius, 0.0], d, a)]
    face.wire([(edge, FORWARD) for edge in edges])
    return face


def disconnected_wire_face() -> FaceSpec:
    """The square [0, 2]^2 and a second wire of two of its edges that do not touch: the
    WireExplorer misses one, so every classification is exact."""
    face = FaceSpec(list(Z_PLANE))
    edges = polygon_wire(face, rectangle(0.0, 0.0, 2.0, 2.0))
    face.wire([(edges[0], FORWARD), (edges[2], FORWARD)])
    return face


def lone_edge_face() -> FaceSpec:
    """The square [0, 4]^2 and a wire of one segment (a polygon of two points)."""
    face = FaceSpec(list(Z_PLANE))
    polygon_wire(face, rectangle(0.0, 0.0, 4.0, 4.0))
    a, b = face.vertex(1.0, 1.0), face.vertex(3.0, 2.0)
    face.wire([(face.edge(SEGMENT, [1.0, 1.0, 3.0, 2.0], a, b), FORWARD)])
    return face


def open_pole_sphere_face(radius: float = 2.0) -> FaceSpec:
    """The sphere face whose pole edges are not flagged degenerated (FClass2d finds
    them degenerated by sampling their 3d points)."""
    face = FaceSpec([SPHERE, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, radius])
    half = 0.5 * math.pi
    south, north = face.vertex(0.0, -half), face.vertex(0.0, half)
    e_south = face.edge(LINE, [0.0, -half, 1.0, 0.0, 0.0, TWO_PI], south, south)
    e_seam = face.edge(SEAM, [TWO_PI, -half, TWO_PI, half, -TWO_PI, 0.0], south, north)
    e_north = face.edge(LINE, [0.0, half, 1.0, 0.0, 0.0, TWO_PI], north, north)
    face.wire([(e_south, FORWARD), (e_seam, FORWARD), (e_north, REVERSED), (e_seam, REVERSED)])
    return face


def adaptor_faces() -> dict[str, FaceSpec]:
    """The BRepClass faces and the faces above, by name, in a fixed order."""
    faces = named_faces()
    faces.update({'cone': cone_face(), 'torus': torus_face(), 'torus_patch': torus_patch_face(),
                  'bezier_surface': bezier_surface_face(), 'bspline': bspline_surface_face(),
                  'bspline_c1': bspline_surface_face(c1_knot=True),
                  'bspline_holed': bspline_surface_face(hole=True),
                  'toleranced_3d': toleranced_square_face(), 'toleranced_2d': toleranced_square_face(False),
                  'crescent': crescent_face(), 'disconnected': disconnected_wire_face(),
                  'lone_edge': lone_edge_face(), 'sphere_open': open_pole_sphere_face()})
    return faces


# The UV window of each face, around its boundary.
ADAPTOR_WINDOWS = {
    'square': (0.0, 2.0, 0.0, 2.0), 'square_reversed': (0.0, 2.0, 0.0, 2.0), 'holed': (0.0, 10.0, 0.0, 10.0),
    'holed_edges': (0.0, 10.0, 0.0, 10.0), 'disc': (-2.0, 2.0, -2.0, 2.0), 'cylinder': (0.0, TWO_PI, 0.0, 5.0),
    'sphere': (0.0, TWO_PI, -0.5 * math.pi, 0.5 * math.pi), 'bezier': (-6.0, 6.0, -6.0, 6.0),
    'gap': (0.0, 4.0, 0.0, 4.0), 'strip': (-5.0, 5.0, 0.0, 1.0), 'trimmed': (0.0, 10.0, 0.0, 10.0),
    'internal': (0.0, 4.0, 0.0, 4.0), 'empty': (-1.0, 1.0, -1.0, 1.0), 'cusp': (0.0, 4.0, 0.0, 3.0),
    'half_strip': (0.0, 5.0, 0.0, 1.0), 'no_pcurve': (0.0, 2.0, 0.0, 2.0), 'flat': (0.0, 4.0, 0.0, 2.0),
    'cylinder_flipped': (0.0, TWO_PI, 0.0, 5.0), 'cone': (0.0, TWO_PI, 0.0, 3.0), 'torus': (0.0, TWO_PI, 0.0, TWO_PI),
    'torus_patch': (0.0, math.pi, -0.5 * math.pi, 0.5 * math.pi), 'bezier_surface': (0.0, 1.0, 0.0, 1.0),
    'bspline': (0.0, 3.0, 0.0, 3.0), 'bspline_c1': (0.0, 3.0, 0.0, 3.0), 'bspline_holed': (0.0, 3.0, 0.0, 3.0),
    'toleranced_3d': (0.0, 2.0, 0.0, 2.0), 'toleranced_2d': (0.0, 2.0, 0.0, 2.0),
    'crescent': (-10.0, 10.0, -0.1, 10.0), 'disconnected': (0.0, 2.0, 0.0, 2.0), 'lone_edge': (0.0, 4.0, 0.0, 4.0),
    'sphere_open': (0.0, TWO_PI, -0.5 * math.pi, 0.5 * math.pi)}
