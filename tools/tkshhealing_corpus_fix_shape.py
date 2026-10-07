"""Requests for the shape-level ShapeFix tools (tools/oracle/tkshhealing_probe_fix_shape.cxx).

`sh_fix_shape` runs ShapeFix_Shape::Perform, ShapeFix::SameParameter (plain and
enforced), ShapeFix::FixVertexPosition, ShapeFix::RemoveSmallEdges, ShapeFix_Wireframe
(wire gaps then small edges), ShapeFix_FixSmallFace, ShapeFix_FixSmallSolid (remove and
merge) and ShapeFix_Shell / ShapeFix_Solid on each shell / solid, then ShapeAnalysis_CheckSmallFace,
ShapeAnalysis_FreeBoundsProperties and the ShapeFix_Wire passes, over the inputs of the
`sh_unify` and `sh_fix_face` requests, the primitive solids, Bezier faces and planar faces
whose wires only BRep_Builder joins, at the default precision and at a coarse one.
"""
from __future__ import annotations
import math

from tktopalgo_corpus_breplib_records import numbers
from tkshhealing_corpus_fix_face import lateral_inputs, loop_inputs, planar_inputs
from tkshhealing_corpus_unify import (PI, arc, face_inputs, polygon_inputs, polygons, revolved_inputs, segment, square,
                                      wire, wire_inputs)


def solid_inputs() -> list[list[float]]:
    return [[0, 1.0, 2.0], [1, 1.0, 0.5, 2.0], [1, 1.0, 0.0, 2.0], [2, 1.0], [3, 3.0, 1.0]]


def small_edge_inputs() -> list[list[float]]:
    """Faces and free wires with edges small at the coarse precision (5e-4 long) or at
    both precisions (5e-8 long): collinear with a neighbour, at a corner, on the boundary
    shared by two faces (alone, so that it is dropped in drop mode), closing a triangle,
    inside a free wire or at its end."""
    o = (0.0, 0.0, 0.0)
    e, t = 5e-4, 5e-8
    left = [(0, 0, 0), (1, 0, 0), (1, e, 0), (1, 1, 0), (0, 1, 0)]
    right = [(1, 0, 0), (2, 0, 0), (2, 1, 0), (1, 1, 0), (1, e, 0)]
    return [
        polygons([left]),
        polygons([left, right]),
        polygons([[(0, 0, 0), (1, 0, 0), (1, e, 0), (0, 1, 0)], [(1, 0, 0), (2, 0, 0), (2, 1, 0), (1, e, 0)]]),
        polygons([[(0, 0, 0), (1, 0, 0), (1, t, 0), (0, 1, 0)], [(1, 0, 0), (2, 0, 0), (2, 1, 0), (1, t, 0)]], 1e-9),
        polygons([[(0, 0, 0), (1, 0, 0), (1 + e, e, 0), (1, 1, 0), (0, 1, 0)]]),
        polygons([[(0, 0, 0), (1, 0, 0), (1, t, 0), (0, 1, 0)]], 1e-9),
        polygons([[(0, 0, 0), (1, 0, 0), (1, t, 0), (1, 1, 0), (0, 1, 0)], [(1, 0, 0), (2, 0, 0), (2, 1, 0), (1, 1, 0), (1, t, 0)]], 1e-9),
        wire([segment(o, (1, 0, 0)), segment((1, 0, 0), (1 + e, 0, 0)), segment((1 + e, 0, 0), (2, 0, 0))]),
        wire([segment(o, (1, 0, 0)), segment((1, 0, 0), (1, e, 0))]),
        wire([segment(o, (1, 0, 0)), segment((1, 0, 0), (1, e, 0)), segment((1, e, 0), (1, 1, 0)), segment((1, 1, 0), o)]),
        wire([arc(o, 1.0, 0.0, PI), arc(o, 1.0, PI, PI + 1e-4), arc(o, 1.0, PI + 1e-4, 2 * PI)]),
        wire([segment(o, (1, 0, 0)), segment((1, 0, 0), (1 + t, 0, 0)), segment((1 + t, 0, 0), (2, 0, 0))]),
        [4, *wire([segment(o, (1, 0, 0)), segment((1, 0, 0), (1, e, 0)), segment((1, e, 0), (1, 1, 0)), segment((1, 1, 0), o)])[1:]],
    ]


def small_face_inputs() -> list[list[float]]:
    """Small faces: a lone spot face, a spot face sewn into a larger face, a strip between
    two squares, thin and needle boxes."""
    s, w = 1e-5, 1e-5
    big = [(s, 0, 0), (1, 0, 0), (1, 1, 0), (0, 1, 0), (0, s, 0), (s, s, 0)]
    strip = [(0, 1, 0), (1, 1, 0), (1, 1 + w, 0), (0, 1 + w, 0)]
    return [
        polygons([square(0, 0, s)], 1e-7),
        polygons([square(0, 0, s), big], 1e-7),
        polygons([square(0, 0), strip, square(0, 1 + w)], 1e-7),
        [3, 1.0, 1.0, w],
        [3, w, w, 1.0],
    ]


def bezier_face(rows: list[list[tuple[float, float, float]]]) -> list[float]:
    return [3, len(rows), len(rows[0])] + [c for row in rows for p in row for c in p]


def bezier_inputs() -> list[list[float]]:
    """Bezier faces: flat, with a pin row spread within the coarse precision (smooth, with and
    without repeated poles), with an exactly collapsed row, twisted (bow tie) and folded."""
    e = 1e-5
    grid = [[(float(i), float(j), 0.0) for j in range(3)] for i in range(3)]
    smooth = [[(0.0, 0.0, 0.0), (0.0, e, 0.0), (0.0, 2 * e, 0.0)]] + grid[1:]
    repeated = [[(0.0, 0.0, 0.0), (0.0, 0.0, 0.0), (0.0, e, 0.0)]] + grid[1:]
    collapsed = [[(0.0, 0.0, 0.0)] * 3] + grid[1:]
    folded = [grid[0], [(1.0, 0.0, 4.0), (1.0, 1.0, 4.0), (1.0, 2.0, 4.0)], [(0.5, 0.0, 0.0), (0.5, 1.0, 0.0), (0.5, 2.0, 0.0)]]
    return [bezier_face(r) for r in (grid, smooth, repeated, collapsed, folded)] + [
        bezier_face([[(0, 0, 0), (0, 1, 0)], [(1, 1, 0), (1, 0, 0)]]),
        bezier_face([[(0, 0, 0), (0, 1, 1)], [(1, 0, 1), (1, 1, 0)]]),
    ]


def raw_face(points: list[tuple[float, float, float]], gap: float = 0.0) -> list[float]:
    """A planar face whose wire collects one segment per side, each shortened by `gap`."""
    edges = []
    for a, b in zip(points, points[1:] + points[:1]):
        end = tuple(q - gap * (q - p) for p, q in zip(a, b))
        edges.append(segment(a, end))
    return [4, len(edges)] + [c for e in edges for c in e]


def raw_face_inputs() -> list[list[float]]:
    """Faces with tails, notches, crossings, gaps and arcs that only BRep_Builder joins, and a
    face without a wire."""
    o = (0.0, 0.0, 0.0)
    tail = [o, (2, 0, 0), (2, 2, 0), (1, 2, 0), (1, 3, 0), (1 + 1e-4, 2, 0), (0, 2, 0)]
    notch = [o, (2, 0, 0), (2, 1, 0), (0, 1, 0), (1, 1, 0)]
    first_notch = [(0, 1, 0), (1, 1, 0), o, (2, 0, 0), (2, 1, 0)]
    bow = [o, (2, 0, 0), (0, 1, 0), (2, 1, 0)]
    arcs = [4, 4, *arc(o, 1.0, 0.0, PI / 2), *segment((0, 1, 0), (-1, 0, 0)), *arc(o, 1.0, PI, 1.5 * PI),
            *segment((0, -1, 0), (1.001, 0, 0))]
    return [raw_face(tail), raw_face(notch), raw_face(first_notch), raw_face(bow), raw_face(square(0, 0, 2.0)),
            raw_face(square(0, 0, 2.0), 1e-4), raw_face(square(0, 0, 2.0), 0.02), raw_face(tail, 1e-5), arcs, [4, 0]]


def sphere_face(arcs: list[tuple]) -> list[float]:
    return [5, len(arcs)] + [c for a in arcs for c in (*a[0], *a[1], a[2], a[3], a[4])]


def sphere_inputs() -> list[list[float]]:
    """Faces on the unit sphere bounded by circle arcs without pcurves: a meridian over the
    north pole closed by half the equator, a lune between two meridians through both poles,
    and a cap below a parallel."""
    o, y, z = (0.0, 0.0, 0.0), (0.0, 1.0, 0.0), (0.0, 0.0, 1.0)
    over_pole = [(o, y, 1.0, PI, 2 * PI), (o, z, 1.0, PI, 2 * PI)]
    lune = [(o, y, 1.0, 0.0, PI), (o, (0.6, 0.8, 0.0), 1.0, PI, 2 * PI)]
    cap = [((0.0, 0.0, 0.6), z, 0.8, 0.0, 2 * PI)]
    return [sphere_face(over_pole), sphere_face(lune), sphere_face(cap)]


def inputs() -> list[list[float]]:
    unify = polygon_inputs() + revolved_inputs() + wire_inputs() + face_inputs() + [[3, 1.0, 2.0, 3.0]]
    unify += small_edge_inputs() + small_face_inputs()
    faces = planar_inputs() + loop_inputs() + lateral_inputs()
    return ([[0, *s] for s in unify] + [[1, *f] for f in faces] + [[2, *s] for s in solid_inputs()]
            + bezier_inputs() + raw_face_inputs() + sphere_inputs())


def append_fix_shape_requests(requests: list[str]) -> None:
    """Append the shape-level ShapeFix requests."""
    for shape in inputs():
        for op in (*range(31), 32):
            requests.append('sh_fix_shape ' + numbers(shape + [1e-7, 1.0, op]))
        for op in (0, 3, 4, 5, 6, 7, 11, 12, 13, 14, 15, *range(16, 31), 32):
            requests.append('sh_fix_shape ' + numbers(shape + [1e-3, 0.1, op]))
