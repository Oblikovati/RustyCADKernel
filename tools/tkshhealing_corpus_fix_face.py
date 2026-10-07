"""Requests for ShapeFix_Face (tools/oracle/tkshhealing_probe_fix_face.cxx).

`sh_fix_face` runs Perform and each public Fix method (orientation, natural bound,
missing seam, small area wires, intersecting wires, wires with two coincident edges,
periodic degenerated edges, loop wires, split face) on planar faces with holes of either
orientation, holes crossing or touching the outer wire or each other, a tiny hole,
coincident wires, two outer wires, a reversed outer wire, faces of wires through one
vertex twice, and the lateral faces of whole and partial cylinders, cones (with apex),
spheres and tori, with and without a reshape context and at two precisions.
"""
from __future__ import annotations
import math

from tktopalgo_corpus_breplib_records import numbers
from tkshhealing_corpus_unify import segment, wire

PI = math.pi


def rect(x0: float, y0: float, x1: float, y1: float) -> list[tuple[float, float, float]]:
    return [(x0, y0, 0.0), (x1, y0, 0.0), (x1, y1, 0.0), (x0, y1, 0.0)]


def planar(wires: list[list[tuple[float, float, float]]], reversed_: list[int]) -> list[float]:
    fields: list[float] = [0, len(wires)]
    for w in wires:
        fields += [len(w)] + [c for p in w for c in p]
    return fields + reversed_


def planar_inputs() -> list[list[float]]:
    outer = rect(0, 0, 4, 4)
    hole = rect(1, 1, 2, 2)
    return [
        planar([outer, hole], [0, 1]),
        planar([outer, hole], [0, 0]),
        planar([outer, rect(3, 1, 5, 2)], [0, 1]),
        planar([outer, rect(1, 1, 2, 2), rect(2, 2, 3, 3)], [0, 1, 1]),
        planar([outer, rect(1, 1, 2, 2), rect(1.5, 1.5, 3, 3)], [0, 1, 1]),
        planar([outer, rect(5, 0, 6, 1)], [0, 0]),
        planar([outer], [1]),
        planar([outer, rect(1, 1, 1.0001, 1.0001)], [0, 1]),
        planar([outer, rect(0, 0, 4, 4)], [0, 1]),
        planar([outer, rect(-1, 1, 5, 2)], [0, 1]),
    ]


def loop_inputs() -> list[list[float]]:
    """Faces of wires passing twice through one point (a loop at a vertex)."""
    o, a, b, c, d = (0.0, 0.0, 0.0), (2.0, 0.0, 0.0), (1.0, 1.0, 0.0), (2.0, 2.0, 0.0), (0.0, 2.0, 0.0)
    eight = [segment(o, a), segment(a, b), segment(b, c), segment(c, d), segment(d, b), segment(b, o)]
    return [[2, *wire(eight)[1:]]]


def lateral_inputs() -> list[list[float]]:
    return [
        [1, 0, 2 * PI, 1.0, 2.0], [1, 0, PI, 1.0, 2.0], [1, 1, 2 * PI, 1.0, 0.0, 2.0],
        [1, 1, PI, 1.0, 0.5, 2.0], [1, 2, 2 * PI, 1.0], [1, 2, PI / 2, 1.0],
        [1, 3, 2 * PI, 3.0, 1.0], [1, 3, PI, 3.0, 1.0],
    ]


def append_fix_face_requests(requests: list[str]) -> None:
    """Append the ShapeFix_Face requests."""
    inputs = planar_inputs() + loop_inputs() + lateral_inputs()
    settings = [(0, 1e-7, 1.0), (1, 1e-7, 1.0), (1, 1e-3, 0.1)]
    for face in inputs:
        for ctx, prec, maxtol in settings:
            for op in range(15):
                requests.append('sh_fix_face ' + numbers(face + [ctx, prec, maxtol, op]))
