"""Requests for the shape-level ShapeFix tools (tools/oracle/tkshhealing_probe_fix_shape.cxx).

`sh_fix_shape` runs ShapeFix_Shape::Perform, ShapeFix::SameParameter (plain and
enforced), ShapeFix::FixVertexPosition, ShapeFix::RemoveSmallEdges, ShapeFix_Wireframe
(wire gaps then small edges), ShapeFix_FixSmallFace, ShapeFix_FixSmallSolid (remove and
merge) and ShapeFix_Shell / ShapeFix_Solid on each shell / solid, over the inputs of the
`sh_unify` and `sh_fix_face` requests and the primitive solids, at the default precision
and at a coarse one.
"""
from __future__ import annotations
import math

from tktopalgo_corpus_breplib_records import numbers
from tkshhealing_corpus_fix_face import lateral_inputs, loop_inputs, planar_inputs
from tkshhealing_corpus_unify import (PI, arc, face_inputs, polygon_inputs, polygons, revolved_inputs, segment, wire,
                                      wire_inputs)


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


def inputs() -> list[list[float]]:
    unify = polygon_inputs() + revolved_inputs() + wire_inputs() + face_inputs() + [[3, 1.0, 2.0, 3.0]]
    unify += small_edge_inputs()
    faces = planar_inputs() + loop_inputs() + lateral_inputs()
    return [[0, *s] for s in unify] + [[1, *f] for f in faces] + [[2, *s] for s in solid_inputs()]


def append_fix_shape_requests(requests: list[str]) -> None:
    """Append the shape-level ShapeFix requests."""
    for shape in inputs():
        for op in range(13):
            requests.append('sh_fix_shape ' + numbers(shape + [1e-7, 1.0, op]))
        for op in (0, 3, 4, 5, 7, 11, 12):
            requests.append('sh_fix_shape ' + numbers(shape + [1e-3, 0.1, op]))
