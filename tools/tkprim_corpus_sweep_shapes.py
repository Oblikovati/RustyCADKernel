"""The generating shapes of the BRepSweep and BRepPrimAPI sweep corpora, built as shape
programs (tktopalgo_shape_program.py, read by tktopalgo_probe_shapes.cxx).

Each builder returns `(program, index)`: the program and the table index of the shape to
sweep. The shapes are vertices, segments, arcs, closed circles and B-spline edges, open
and closed wires, planar faces (with a hole), the lateral face of a cylinder (with its
seam), shells, compounds, solids (which the sweeps refuse) and a located face; the
program also holds foreign shapes (a vertex not in the swept shape), which the history
queries of the corpora reach.
"""
from __future__ import annotations
import math
from tktopalgo_corpus_brepcheck_shapes import CUBE_FACES, cube, cylinder_face, square
from tktopalgo_shape_program import COMPOUND, FORWARD, REVERSED, WIRE, ShapeProgram, sub, unit

Point = tuple[float, float, float]


def segment(p: ShapeProgram, a: Point, b: Point, vertices=None) -> int:
    """The edge from `a` to `b` on its 3d line, with its two vertices (or `vertices`)."""
    va, vb = vertices or (p.vertex(a), p.vertex(b))
    length = math.dist(a, b)
    e = p.edge()
    p.edge_curve(e, p.line(a, unit(sub(b, a))))
    p.add(e, va, FORWARD)
    p.add(e, vb, REVERSED)
    p.edge_range(e, 0.0, length)
    return e


def arc(p: ShapeProgram, center: Point, radius: float, first: float, last: float) -> int:
    """The arc of the circle of `radius` around `center` in a plane z = const (normal +Z,
    X direction +X), between the angles `first` and `last`."""
    points = [(center[0]+radius*math.cos(t), center[1]+radius*math.sin(t), center[2]) for t in (first, last)]
    e = p.edge()
    p.edge_curve(e, p.curve(1, *center, 0.0, 0.0, 1.0, radius))
    p.add(e, p.vertex(points[0]), FORWARD)
    p.add(e, p.vertex(points[1]), REVERSED)
    p.edge_range(e, first, last)
    return e


def circle(p: ShapeProgram, center: Point, radius: float) -> int:
    """The closed circle edge of `radius` around `center` (normal +Z) with one vertex."""
    v = p.vertex((center[0]+radius, center[1], center[2]))
    e = p.edge()
    p.edge_curve(e, p.curve(1, *center, 0.0, 0.0, 1.0, radius))
    p.add(e, v, FORWARD)
    p.add(e, v, REVERSED)
    p.edge_range(e, 0.0, 2*math.pi)
    return e


def bspline(p: ShapeProgram, poles: list[Point]) -> int:
    """The cubic B-spline edge of 4 poles (knots 0 and 1, multiplicity 4)."""
    e = p.edge()
    p.edge_curve(e, p.curve(5, 3, len(poles), *[c for pole in poles for c in pole], 2, 0.0, 4, 1.0, 4))
    p.add(e, p.vertex(poles[0]), FORWARD)
    p.add(e, p.vertex(poles[-1]), REVERSED)
    p.edge_range(e, 0.0, 1.0)
    return e


def polyline(p: ShapeProgram, points: list[Point], closed: bool) -> int:
    """The wire of the segments between `points` (closed back to the first one)."""
    vertices = [p.vertex(point) for point in points]
    wire = p.container(WIRE)
    count = len(points) if closed else len(points)-1
    for k in range(count):
        j = (k+1) % len(points)
        p.add(wire, segment(p, points[k], points[j], (vertices[k], vertices[j])), FORWARD)
    if closed:
        p.flag(wire, 0, True)
    return wire


def with_foreign(builder):
    """A builder whose program ends with a foreign vertex."""
    def build():
        p = ShapeProgram()
        index = builder(p)
        p.vertex((7.0, 7.0, 7.0))
        return p, index
    return build


def xz_square(p: ShapeProgram, x0: float, size: float) -> int:
    """A square face in the XZ plane from x = x0 (normal -Y, so +Z revolves outward)."""
    from tktopalgo_corpus_brepcheck_shapes import PlanarShapes
    shapes = PlanarShapes(p, [(x0, 0, 0), (x0+size, 0, 0), (x0+size, 0, size), (x0, 0, size)])
    face = shapes.face([[0, 1, 2, 3]])
    shapes.finish()
    return face


def compound_of(p: ShapeProgram) -> int:
    """A compound of a vertex and a segment."""
    c = p.container(COMPOUND)
    p.add(c, p.vertex((3.0, 0.0, 0.0)), FORWARD)
    p.add(c, segment(p, (1.0, 0.0, 0.0), (2.0, 0.0, 1.0)), FORWARD)
    return c


def located_square(p: ShapeProgram) -> int:
    """A square face located by a translation."""
    face = square(p, 1.0).faces[0]
    return p.located(face, p.location(0, 0.5, -0.25, 1.0))


def cube_shell(p: ShapeProgram, faces) -> int:
    """The shell of some faces of the unit cube."""
    shapes = cube(p, 1.0, faces)
    return shapes.shell()


def solid_of_cube(p: ShapeProgram) -> int:
    """A solid holding the shell of the unit cube (refused by the sweeps)."""
    from tktopalgo_corpus_brepcheck_shapes import solid_of
    return solid_of(p, cube_shell(p, None))


PRISM_SHAPES = {
    'vertex': with_foreign(lambda p: p.vertex((1.0, 2.0, 3.0))),
    'segment': with_foreign(lambda p: segment(p, (0.0, 0.0, 0.0), (1.0, 1.0, 0.0))),
    'arc': with_foreign(lambda p: arc(p, (0.0, 0.0, 0.0), 2.0, 0.0, 1.5)),
    'circle': with_foreign(lambda p: circle(p, (1.0, 0.0, 0.0), 0.5)),
    'bspline': with_foreign(lambda p: bspline(p, [(0, 0, 0), (1, 2, 0), (2, -1, 0), (3, 0, 0.5)])),
    'open_wire': with_foreign(lambda p: polyline(p, [(0, 0, 0), (1, 0, 0), (1, 1, 0), (2, 1, 0)], False)),
    'closed_wire': with_foreign(lambda p: polyline(p, [(0, 0, 0), (1, 0, 0), (1, 1, 0), (0, 1, 0)], True)),
    'square': with_foreign(lambda p: square(p, 2.0).faces[0]),
    'holed_square': with_foreign(lambda p: square(p, 3.0, holes=[(1.0, 1.0, 1.0)]).faces[0]),
    'cylinder_face': with_foreign(lambda p: cylinder_face(p, 1.0, 2.0)['face']),
    'two_faces': with_foreign(lambda p: cube_shell(p, [CUBE_FACES[0], CUBE_FACES[2]])),
    'compound': with_foreign(compound_of),
    'located_square': with_foreign(located_square),
    'solid': with_foreign(solid_of_cube),
}

REVOL_SHAPES = {
    'vertex': with_foreign(lambda p: p.vertex((2.0, 0.0, 1.0))),
    'axis_vertex': with_foreign(lambda p: p.vertex((0.0, 0.0, 1.0))),
    'parallel_segment': with_foreign(lambda p: segment(p, (1.0, 0.0, 0.0), (1.0, 0.0, 2.0))),
    'inclined_segment': with_foreign(lambda p: segment(p, (2.0, 0.0, 0.0), (1.0, 0.0, 2.0))),
    'radial_segment': with_foreign(lambda p: segment(p, (0.0, 0.0, 1.0), (2.0, 0.0, 1.0))),
    'axis_segment': with_foreign(lambda p: segment(p, (0.0, 0.0, 0.0), (0.0, 0.0, 2.0))),
    'meridian_arc': with_foreign(lambda p: xz_arc(p, (0.0, 0.0, 0.0), 1.0)),
    'offset_arc': with_foreign(lambda p: xz_arc(p, (3.0, 0.0, 0.0), 1.0)),
    'bspline': with_foreign(lambda p: bspline(p, [(1, 0, 0), (2, 0, 1), (1.5, 0, 2), (1, 0, 3)])),
    'profile_wire': with_foreign(lambda p: polyline(p, [(0, 0, 0), (1, 0, 0), (1, 0, 1), (0, 0, 1)], False)),
    'ring_square': with_foreign(lambda p: xz_square(p, 2.0, 1.0)),
    'axis_square': with_foreign(lambda p: xz_square(p, 0.0, 1.0)),
    'compound': with_foreign(compound_of),
    'solid': with_foreign(solid_of_cube),
}


def xz_arc(p: ShapeProgram, center: Point, radius: float) -> int:
    """The half circle in the XZ plane around `center` from -Z to +Z through +X (normal
    -Y; gp_Ax2 then takes the X direction -Z, so the parameters are 0..PI)."""
    e = p.edge()
    p.edge_curve(e, p.curve(1, *center, 0.0, -1.0, 0.0, radius))
    bottom = (center[0], center[1], center[2]-radius)
    top = (center[0], center[1], center[2]+radius)
    p.add(e, p.vertex(bottom), FORWARD)
    p.add(e, p.vertex(top), REVERSED)
    p.edge_range(e, 0.0, math.pi)
    return e
