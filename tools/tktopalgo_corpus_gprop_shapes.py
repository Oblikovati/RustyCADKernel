"""Shapes of the BRepGProp requests (tools/tktopalgo_corpus_brepgprop.py), spelled as
`tp_shape` records (tktopalgo_corpus_shapes.py).

Solids the other corpora do not build (a cone frustum, a thick ring whose end faces are
holed, the same solid shared by several located and reversed instances), and single
faces on every surface kind the shape reader knows: planes with holes, B-spline and
ellipse pcurves, B-spline patches, revolutions, extrusions (bounded, and whole, with
infinite V bounds) and offsets, plus curves the linear properties integrate (ellipses,
parabolas, an edge with only a B-spline pcurve).
"""
from __future__ import annotations
import math
from tktopalgo_corpus_shapes import (
    AGAIN, BEZIER, BSPLINE, BSPLINE_SURFACE, CIRCLE, CONE, CYLINDER, EDGE, ELLIPSE, EXTRUSION,
    FACE, FORWARD, IDENTITY, LINE, NO_CURVE, OFFSET, PARABOLA, PBSPLINE, PCIRCLE, PELLIPSE, PLANE,
    PLINE, REVERSED, REVOLUTION, SOLID, FaceRecord, ShapeSpec)
from tktopalgo_corpus_solids import TWO_PI, add_box, axis, box

Z, X = [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]


def frustum(radius: float, top_radius: float, height: float) -> ShapeSpec:
    """The solid of a z-axis cone frustum from `radius` at z = 0 to `top_radius` at z =
    height (the cone frame at the origin, its V along the generatrix)."""
    spec = ShapeSpec()
    semi_angle = math.atan2(top_radius - radius, height)
    slant = math.hypot(top_radius - radius, height)
    vb, vt = spec.vertex([radius, 0.0, 0.0]), spec.vertex([top_radius, 0.0, height])
    bottom = spec.edge([CIRCLE, 0.0, 0.0, 0.0, *Z, *X, radius], vb, vb, (0.0, TWO_PI))
    top = spec.edge([CIRCLE, 0.0, 0.0, height, *Z, *X, top_radius], vt, vt, (0.0, TWO_PI))
    generatrix = [math.sin(semi_angle), 0.0, math.cos(semi_angle)]
    seam = spec.edge([LINE, radius, 0.0, 0.0, *generatrix], vb, vt, (0.0, slant))
    lateral = FaceRecord([CONE, 0.0, 0.0, 0.0, *Z, *X, radius, semi_angle])
    lateral.wire([(bottom, FORWARD, [PLINE, 0.0, 0.0, 1.0, 0.0]), (seam, FORWARD, [PLINE, TWO_PI, 0.0, 0.0, 1.0]),
                  (top, REVERSED, [PLINE, 0.0, slant, 1.0, 0.0]), (seam, REVERSED, [PLINE, 0.0, 0.0, 0.0, 1.0])])
    floor = FaceRecord([PLANE, 0.0, 0.0, 0.0, *axis(2, -1.0), *X])
    floor.wire([(bottom, REVERSED, [PCIRCLE, 0.0, 0.0, radius, 0])])
    roof = FaceRecord([PLANE, 0.0, 0.0, height, *Z, *X])
    roof.wire([(top, FORWARD, [PCIRCLE, 0.0, 0.0, top_radius, 1])])
    spec.shell([spec.face(lateral), spec.face(floor), spec.face(roof)])
    return spec


def cylinder_side(spec: ShapeSpec, radius: float, height: float) -> tuple[int, int, int]:
    """The lateral face of a z-axis cylinder and its bottom and top circle edges."""
    vb, vt = spec.vertex([radius, 0.0, 0.0]), spec.vertex([radius, 0.0, height])
    bottom = spec.edge([CIRCLE, 0.0, 0.0, 0.0, *Z, *X, radius], vb, vb, (0.0, TWO_PI))
    top = spec.edge([CIRCLE, 0.0, 0.0, height, *Z, *X, radius], vt, vt, (0.0, TWO_PI))
    seam = spec.edge([LINE, radius, 0.0, 0.0, *Z], vb, vt, (0.0, height))
    lateral = FaceRecord([CYLINDER, 0.0, 0.0, 0.0, *Z, *X, radius])
    lateral.wire([(bottom, FORWARD, [PLINE, 0.0, 0.0, 1.0, 0.0]), (seam, FORWARD, [PLINE, TWO_PI, 0.0, 0.0, 1.0]),
                  (top, REVERSED, [PLINE, 0.0, height, 1.0, 0.0]), (seam, REVERSED, [PLINE, 0.0, 0.0, 0.0, 1.0])])
    return spec.face(lateral), bottom, top


def ring(outer: float, inner: float, height: float) -> ShapeSpec:
    """A thick z-axis ring: the outer cylinder, the inner one REVERSED in the shell, and
    two annular end faces, each a plane face with a circular hole."""
    spec = ShapeSpec()
    outer_side, outer_bottom, outer_top = cylinder_side(spec, outer, height)
    inner_side, inner_bottom, inner_top = cylinder_side(spec, inner, height)
    floor = FaceRecord([PLANE, 0.0, 0.0, 0.0, *axis(2, -1.0), *X])
    floor.wire([(outer_bottom, REVERSED, [PCIRCLE, 0.0, 0.0, outer, 0])])
    floor.wire([(inner_bottom, FORWARD, [PCIRCLE, 0.0, 0.0, inner, 0])])
    roof = FaceRecord([PLANE, 0.0, 0.0, height, *Z, *X])
    roof.wire([(outer_top, FORWARD, [PCIRCLE, 0.0, 0.0, outer, 1])])
    roof.wire([(inner_top, REVERSED, [PCIRCLE, 0.0, 0.0, inner, 1])])
    faces = [outer_side, inner_side, spec.face(floor), spec.face(roof)]
    spec.shells.append((FORWARD, True, [(f, REVERSED if f == inner_side else FORWARD, list(IDENTITY)) for f in faces]))
    return spec


def shared_solids(location: list[float], orientation: int, extra: bool = False) -> ShapeSpec:
    """A compound of a 2 x 3 x 4 box solid and the same solid (one TShape) again with
    `location` and `orientation`, then an exact duplicate; `extra` adds a free face."""
    spec = box([0.0, 0.0, 0.0], [2.0, 3.0, 4.0])
    spec.top = [(SOLID, 0), (AGAIN, 0, *location, orientation), (AGAIN, 0, *IDENTITY, FORWARD)]
    if extra:
        spec.top.append((FACE, 5))
    return spec


def holed_plate() -> ShapeSpec:
    """A 4 x 4 plane face at z = 1 with a circular hole of radius 1 at its centre."""
    spec = ShapeSpec(top=[(FACE, 0)])
    corners = [spec.vertex(p) for p in ([0.0, 0.0, 1.0], [4.0, 0.0, 1.0], [4.0, 4.0, 1.0], [0.0, 4.0, 1.0])]
    uv = [(0.0, 0.0), (4.0, 0.0), (4.0, 4.0), (0.0, 4.0)]
    uses = []
    for k in range(4):
        (u0, v0), (u1, v1) = uv[k], uv[(k + 1) % 4]
        du, dv = (u1 - u0) / 4.0, (v1 - v0) / 4.0
        e = spec.edge([LINE, u0, v0, 1.0, du, dv, 0.0], corners[k], corners[(k + 1) % 4], (0.0, 4.0))
        uses.append((e, FORWARD, [PLINE, u0, v0, du, dv]))
    face = FaceRecord([PLANE, 0.0, 0.0, 1.0, *Z, *X])
    face.wire(uses)
    vh = spec.vertex([3.0, 2.0, 1.0])
    hole = spec.edge([CIRCLE, 2.0, 2.0, 1.0, *Z, *X, 1.0], vh, vh, (0.0, TWO_PI))
    face.wire([(hole, REVERSED, [PCIRCLE, 2.0, 2.0, 1.0, 1])])
    spec.face(face)
    return spec


def spline_blob() -> ShapeSpec:
    """A plane face bounded by a two-span cubic B-spline edge (a 3d curve and a B-spline
    pcurve) below the X axis and the segment that closes it."""
    spec = ShapeSpec(top=[(FACE, 0)])
    a, b = spec.vertex([0.0, 0.0, 0.0]), spec.vertex([4.0, 0.0, 0.0])
    poles = [(0.0, 0.0), (0.5, -1.5), (2.0, -2.5), (3.5, -1.5), (4.0, 0.0)]
    knots = [0.0, 4, 0.5, 1, 1.0, 4]
    curve = [BSPLINE, 3, len(poles), *[c for u, v in poles for c in (u, v, 0.0)], 3, *knots]
    spline = spec.edge(curve, a, b, (0.0, 1.0))
    line = spec.edge([LINE, 4.0, 0.0, 0.0, -1.0, 0.0, 0.0], b, a, (0.0, 4.0))
    face = FaceRecord([PLANE, 0.0, 0.0, 0.0, *Z, *X])
    face.wire([(spline, FORWARD, [PBSPLINE, 3, len(poles), *[c for p in poles for c in p], 3, *knots]),
               (line, FORWARD, [PLINE, 4.0, 0.0, -1.0, 0.0])])
    spec.face(face)
    return spec


def ellipse_disc(tilted: bool = False) -> ShapeSpec:
    """A plane face bounded by one closed ellipse edge (a 3d ellipse and its pcurve); the
    plane is tilted about X when `tilted`."""
    spec = ShapeSpec(top=[(FACE, 0)])
    normal = [0.0, -math.sin(0.4), math.cos(0.4)] if tilted else Z
    v = spec.vertex([3.0, 0.0, 0.0])
    e = spec.edge([ELLIPSE, 0.0, 0.0, 0.0, *normal, *X, 3.0, 1.5], v, v, (0.0, TWO_PI))
    face = FaceRecord([PLANE, 0.0, 0.0, 0.0, *normal, *X])
    face.wire([(e, FORWARD, [PELLIPSE, 0.0, 0.0, 3.0, 1.5, 1])])
    spec.face(face)
    return spec


def spline_patch_numbers() -> list[float]:
    """A bicubic-by-quadratic B-spline surface over [0, 2] x [0, 1] with an interior knot
    in each direction (`tp_bspline_surface` fields)."""
    nu, nv = 5, 4
    poles = []
    for i in range(nu):
        for j in range(nv):
            poles.extend([i * 0.75, j * 0.8, 0.3 * math.sin(i + 0.5 * j) + 0.1 * i * j])
    return [BSPLINE_SURFACE, 3, 2, nu, nv, *poles, 3, 0.0, 4, 1.0, 1, 2.0, 4, 3, 0.0, 3, 0.5, 1, 1.0, 3]


def spline_patch(natural: bool) -> ShapeSpec:
    """The B-spline patch, natural, or bounded by its four iso pcurves (edges without 3d
    curve)."""
    spec = ShapeSpec(top=[(FACE, 0)])
    face = FaceRecord(spline_patch_numbers(), 1e-7, natural)
    if not natural:
        e = [spec.edge([NO_CURVE], -1, -1, span) for span in ((0.0, 2.0), (0.0, 1.0), (0.0, 2.0), (0.0, 1.0))]
        face.wire([(e[0], FORWARD, [PLINE, 0.0, 0.0, 1.0, 0.0]), (e[1], FORWARD, [PLINE, 2.0, 0.0, 0.0, 1.0]),
                   (e[2], REVERSED, [PLINE, 0.0, 1.0, 1.0, 0.0]), (e[3], REVERSED, [PLINE, 0.0, 0.0, 0.0, 1.0])])
    spec.face(face)
    return spec


def wide_spline_face() -> ShapeSpec:
    """The natural face of the upstream AdaptiveNaturalBSplineSurface case: a degree 1
    B-spline surface with GaussPointsMax + 1 = 62 U intervals (poles (i, 0, 0) and
    (i, 2, 0)), more knots than the interval arrays of the adaptive Gauss integration
    hold at first."""
    count = 63
    poles = [c for i in range(count) for pole in ((float(i), 0.0, 0.0), (float(i), 2.0, 0.0)) for c in pole]
    knots = [c for i in range(count) for c in (float(i), 2 if i in (0, count - 1) else 1)]
    spec = ShapeSpec(top=[(FACE, 0)])
    spec.face(FaceRecord([BSPLINE_SURFACE, 1, 1, count, 2, *poles, count, *knots, 2, 0.0, 2, 1.0, 2], 1e-7))
    return spec


def revolution_face() -> ShapeSpec:
    """A natural face on the revolution of a cubic Bezier profile about Z."""
    profile = [BEZIER, 4, 1.0, 0.0, 0.0, 2.0, 0.0, 1.0, 1.5, 0.0, 2.0, 0.5, 0.0, 3.0]
    spec = ShapeSpec(top=[(FACE, 0)])
    spec.face(FaceRecord([REVOLUTION, *profile, 0.0, 0.0, 0.0, *Z]))
    return spec


def extrusion_curve() -> list[float]:
    """A quadratic B-spline curve with three spans in the XY plane."""
    poles = [[0.0, 0.0, 0.0], [1.0, 1.0, 0.0], [2.0, 0.0, 0.0], [3.0, 1.0, 0.0], [4.0, 0.0, 0.0]]
    return [BSPLINE, 2, 5, *[c for p in poles for c in p], 4, 0.0, 3, 1.0, 1, 2.0, 1, 3.0, 3]


def extrusion_face(natural: bool) -> ShapeSpec:
    """The extrusion along Z of the B-spline curve: whole (V unbounded) or the band
    V in [0, 2] bounded by iso pcurves (edges without 3d curve)."""
    spec = ShapeSpec(top=[(FACE, 0)])
    face = FaceRecord([EXTRUSION, *extrusion_curve(), *Z], 1e-7, natural)
    if not natural:
        e = [spec.edge([NO_CURVE], -1, -1, span) for span in ((0.0, 3.0), (0.0, 2.0), (0.0, 3.0), (0.0, 2.0))]
        face.wire([(e[0], FORWARD, [PLINE, 0.0, 0.0, 1.0, 0.0]), (e[1], FORWARD, [PLINE, 3.0, 0.0, 0.0, 1.0]),
                   (e[2], REVERSED, [PLINE, 0.0, 2.0, 1.0, 0.0]), (e[3], REVERSED, [PLINE, 0.0, 0.0, 0.0, 1.0])])
    spec.face(face)
    return spec


def offset_face(basis: list[float], distance: float) -> ShapeSpec:
    """A natural face on the offset of the surface `basis` by `distance`."""
    spec = ShapeSpec(top=[(FACE, 0)])
    spec.face(FaceRecord([OFFSET, *basis, distance]))
    return spec


def side_face() -> ShapeSpec:
    """The lateral face of a cylinder of radius 1.5 and height 2, alone."""
    spec = ShapeSpec(top=[(FACE, 0)])
    cylinder_side(spec, 1.5, 2.0)
    return spec


def curve_edges() -> ShapeSpec:
    """A compound of free edges on an ellipse, an arc of ellipse, a parabola arc and a
    reversed parabola arc."""
    spec = ShapeSpec()
    spec.edge([ELLIPSE, 1.0, 2.0, 0.0, *Z, *X, 3.0, 1.0], -1, -1, (0.0, TWO_PI))
    spec.edge([ELLIPSE, 0.0, 0.0, 1.0, 0.0, 0.6, 0.8, *X, 2.0, 0.5], -1, -1, (0.3, 2.9))
    spec.edge([PARABOLA, 0.0, 0.0, 0.0, *Z, *X, 0.5], -1, -1, (-2.0, 3.0))
    spec.edge([PARABOLA, 1.0, 1.0, 1.0, *X, 0.0, 1.0, 0.0, 2.0], -1, -1, (-1.0, 1.0))
    spec.top = [(EDGE, i) for i in range(4)]
    return spec


def pcurve_only_edge() -> ShapeSpec:
    """OCC regression: a degenerated edge whose only representation is a cubic B-spline
    pcurve on a plane (the face holds it); the edge alone."""
    spec = ShapeSpec(top=[(EDGE, 0)])
    e = spec.edge([NO_CURVE], -1, -1, (0.0, 1.0), 1e-7, degenerated=True)
    face = FaceRecord([PLANE, 0.0, 0.0, 0.0, *Z, *X])
    face.wire([(e, FORWARD, [PBSPLINE, 3, 4, 0.0, 0.0, 0.1, 0.05, 0.2, -0.05, 0.3, 0.0, 2, 0.0, 4, 1.0, 4])])
    spec.face(face)
    return spec


def meshed_reversed_box() -> ShapeSpec:
    """A meshed 3 x 2 x 1 box whose shell holds its bottom face REVERSED (with its mesh)."""
    spec = ShapeSpec()
    faces = add_box(spec, [0.0, 0.0, 0.0], [3.0, 2.0, 1.0], mesh=0.05)
    uses = [(f, REVERSED if k == 0 else FORWARD, list(IDENTITY)) for k, f in enumerate(faces)]
    spec.shells.append((FORWARD, True, uses))
    return spec
