"""Deterministic requests for BRepSweep (tools/oracle/tkprim_probe_brepsweep.cxx and
tkprim_probe_brepsweep_members.cxx; the records are documented there).

`bs_prism` sweeps every generating shape of tkprim_corpus_sweep_shapes.py along vectors
(axis-aligned, oblique, reversed, of the limit length Precision::Confusion() and below it)
and directions (infinite and semi-infinite), with every copy and canonize flag, then prints
the result and the history of every program shape (a foreign vertex included, whose first
and last shapes raise Standard_OutOfRange). `bs_revol` revolves the revolution shapes
(vertices on and off the axis, segments parallel, inclined, radial and on the axis, arcs
centered on and off the axis, B-splines, profiles, faces touching the axis or not) around
three axes by quarter, half, complete, negative, reduced (3PI, 7PI + 0.1) and too small
angles, with both copy flags. `bs_members` calls the members of BRepSweep_Translation and
BRepSweep_Rotation directly on custom directing edges, and `bs_tools` the Tool, Iterator
and Builder.
"""
from __future__ import annotations
import math
from tkprim_corpus_sweep_shapes import PRISM_SHAPES, REVOL_SHAPES

PI = math.pi
EDGE = 6


def numbers(values) -> str:
    """The fields of a request; integers stay integers."""
    return ' '.join(str(v) if isinstance(v, int) else repr(float(v)) for v in values)


def program_fields(builder) -> str:
    program, index = builder()
    return program.tokens() + ' ' + str(index)


PRISM_SWEEPS = [
    [0, 0.0, 0.0, 1.0], [0, 1.0, 2.0, 3.0], [0, -0.5, 0.25, -2.0], [0, 1e-7, 0.0, 0.0],
    [0, 1e-8, 0.0, 0.0], [1, 0.0, 0.0, 1.0, 1], [1, 0.6, 0.0, 0.8, 0],
]


def append_prism_requests(requests: list[str]) -> None:
    """Every prism shape along every vector or direction, with the flags varied."""
    for index, (name, builder) in enumerate(sorted(PRISM_SHAPES.items())):
        fields = program_fields(builder)
        for k, sweep in enumerate(PRISM_SWEEPS):
            copy, canonize = (index + k) % 2, (index + k) // 2 % 2
            requests.append(f'bs_prism {fields} {numbers(sweep + [copy, canonize])}')
        requests.append(f'bs_prism {fields} {numbers([0, 1.0, 1.0, 1.0, 1, 1])}')
        requests.append(f'bs_prism {fields} {numbers([0, 1.0, 1.0, 1.0, 0, 0])}')


AXES = [[0.0, 0.0, 0.0, 0.0, 0.0, 1.0], [0.0, 0.0, 0.0, 1.0, 0.0, 0.0], [0.5, 0.0, 0.0, 0.0, 0.0, 1.0]]
ANGLES = [PI / 2, PI, 2 * PI, -PI / 2, 3 * PI, 7 * PI + 0.1, 1e-13, 0.0]


def append_revol_requests(requests: list[str]) -> None:
    """Every revolution shape around every axis by every angle (and the complete
    constructor), both copy flags in turn."""
    for index, (name, builder) in enumerate(sorted(REVOL_SHAPES.items())):
        fields = program_fields(builder)
        for a, axis in enumerate(AXES):
            for k, angle in enumerate(ANGLES):
                copy = (index + a + k) % 2
                requests.append(f'bs_revol {fields} {numbers([0] + axis + [angle, copy])}')
            requests.append(f'bs_revol {fields} {numbers([1] + axis + [(index + a) % 2])}')


# Directing edges: (index, type, closed, begInf, endInf).
DIRS = [[2, EDGE, 0, 0, 0], [1, EDGE, 0, 0, 1], [0, EDGE, 0, 1, 1], [2, EDGE, 1, 0, 0], [1, EDGE, 0, 1, 0]]


def append_members_requests(requests: list[str]) -> None:
    """The translation and rotation members on several shapes and directing edges."""
    prism_names = ['vertex', 'segment', 'circle', 'closed_wire', 'square', 'cylinder_face', 'two_faces']
    for index, name in enumerate(prism_names):
        fields = program_fields(PRISM_SHAPES[name])
        for k, n in enumerate(DIRS):
            copy, canonize = (index + k) % 2, (index + k) // 2 % 2
            requests.append(f'bs_members 0 {fields} {numbers(n + [0.0, 0.5, 2.0, copy, canonize])}')
    revol_names = ['vertex', 'axis_vertex', 'inclined_segment', 'radial_segment', 'axis_segment',
                   'meridian_arc', 'offset_arc', 'ring_square', 'axis_square']
    for index, name in enumerate(revol_names):
        fields = program_fields(REVOL_SHAPES[name])
        for k, n in enumerate(DIRS):
            angle = 2 * PI if n[2] else [PI / 2, 1.0, PI][(index + k) % 3]
            requests.append(f'bs_members 1 {fields} {numbers(n + AXES[0] + [angle, (index + k) % 2])}')


def append_tools_requests(requests: list[str]) -> None:
    """The tool, iterator and builder on every prism shape."""
    for name, builder in sorted(PRISM_SHAPES.items()):
        requests.append(f'bs_tools {program_fields(builder)}')


def append_brepsweep_requests(requests: list[str]) -> None:
    """Append the BRepSweep requests."""
    append_prism_requests(requests)
    append_revol_requests(requests)
    append_members_requests(requests)
    append_tools_requests(requests)
