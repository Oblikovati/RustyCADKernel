"""Deterministic requests for BRepPrimAPI and BRepPreviewAPI
(tools/oracle/tkprim_probe_brepprimapi.cxx and tkprim_probe_brepprimapi_sweeps.cxx; the
records are documented there).

`pa_box` builds every box constructor (signed, flat, tiny and framed sizes, two corners in
any order) and runs call sequences: the solid, the shell, the solid after the shell (a
type mismatch), the six faces, Build, and every Init before and after a build (the shape
and the done flag of a build stay). `pv_box` runs the same calls on the preview box: a
point, a segment, a rectangle in each plane or a solid, set by each Init. `pa_wedge` builds
the four wedge constructors on the wedge shapes of the BRepPrim corpus. `pa_oneaxis` builds
every constructor of the cylinder, cone, sphere, torus and revolution makers with valid,
degenerate and invalid values. `pa_prism` and `pa_revol` sweep the generating shapes of the
BRepSweep corpus (and profiles crossing or touching the axis) and print the history of
every program shape; `pa_halfspace` bounds half-spaces by faces and shells seen from points
inside, outside, beside and on them.
"""
from __future__ import annotations
import math
from tkprim_corpus_brepprim import FRAMES, wedge_shapes
from tkprim_corpus_sweep_shapes import PRISM_SHAPES, REVOL_SHAPES, polyline, segment, with_foreign
from tktopalgo_corpus_brepcheck_shapes import cube, cylinder_face, square

PI = math.pi


def numbers(values) -> str:
    """The fields of a request, with every double spelled exactly."""
    return ' '.join(repr(float(v)) for v in values)


def program_fields(builder) -> str:
    program, index = builder()
    return program.tokens() + ' ' + str(index)


# Box calls: (op, arguments).
SHAPE, SHELL, SOLID, BUILD, IS_DONE, WEDGE = [0], [1], [2], [9], [10], [15]
FACES = [[op] for op in range(3, 9)]
BOX_SEQUENCES = [
    [SHAPE], [SOLID, SHELL], [SHELL, SOLID], FACES + [WEDGE], [IS_DONE, BUILD, IS_DONE, SHAPE],
    [SHAPE, [11, 4.0, 5.0, 6.0], SHAPE, IS_DONE, BUILD, SHAPE],
    [[12, 1.0, 1.0, 1.0, -1.0, 2.0, 3.0], WEDGE, SOLID],
    [[13, 1.0, 2.0, 3.0, -1.0, 0.5, 0.0], SOLID, [3], [8]],
    [[14] + FRAMES[2] + [1.0, 2.0, 3.0], WEDGE, SHAPE],
    [SHELL, SHAPE, SOLID, WEDGE],
]


def box_constructors() -> list[list[float]]:
    """`kind values...` of every box constructor."""
    ctors = [[0]]
    ctors += [[1] + s for s in ([1.0, 2.0, 3.0], [-1.0, 2.0, -3.0], [0.0, 1.0, 1.0], [1e-8, 1.0, 1.0], [10.0, 20.0, 30.0])]
    ctors += [[2, 1.0, -2.0, 0.5] + s for s in ([2.0, 3.0, 4.0], [-2.0, 3.0, -4.0], [2.0, 0.0, 4.0])]
    ctors += [[3] + p for p in ([0, 0, 0, 1, 2, 3], [1, 2, 3, 0, 0, 0], [1, -1, 2, -1, 1, -2], [1, 1, 1, 1, 1, 1],
                                [0, 0, 0, 1, 1, 0])]
    ctors += [[4] + f + s for f in FRAMES[1:] for s in ([1.0, 2.0, 3.0], [-1.0, 2.0, 3.0])]
    return ctors


def calls_fields(sequence: list[list[float]]) -> list[float]:
    return [len(sequence)] + [v for call in sequence for v in call]


def append_box_requests(requests: list[str]) -> None:
    """Every box constructor with every call sequence."""
    for ctor in box_constructors():
        for sequence in BOX_SEQUENCES:
            requests.append('pa_box ' + numbers(ctor + calls_fields(sequence)))


PREVIEW_INITS = [
    [11, 0.0, 0.0, 0.0], [11, 1.0, 0.0, 0.0], [11, 0.0, 2.0, 0.0], [11, 0.0, 0.0, -3.0],
    [11, 1.0, 1.0, 0.0], [11, 0.0, 1.0, 2.0], [11, 2.0, 0.0, -1.0], [11, 1.0, 2.0, 3.0],
    [11, 1e-8, 1.0, 1.0], [11, 1e-8, 1e-8, 1.0], [12, 1.0, 2.0, 3.0, -1.0, 0.0, 2.0],
    [13, 1.0, 1.0, 1.0, 1.0, 3.0, 2.0], [13, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0], [13, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0],
    [14] + FRAMES[1] + [1.0, 2.0, 0.0], [14] + FRAMES[3] + [1.0, 2.0, 3.0], [14] + FRAMES[2] + [0.0, 0.0, 1.0],
]


def append_preview_requests(requests: list[str]) -> None:
    """Every preview Init with the shape, then the rebuild orders."""
    requests.append('pv_box ' + numbers(calls_fields([SHAPE, IS_DONE, WEDGE])))
    for init in PREVIEW_INITS:
        requests.append('pv_box ' + numbers(calls_fields([init, SHAPE, IS_DONE])))
        requests.append('pv_box ' + numbers(calls_fields([init, BUILD, FACES[0], SHELL, SHAPE])))
    sequences = [
        [[11, 1.0, 1.0, 1.0], SHAPE, [11, 1.0, 0.0, 0.0], SHAPE, BUILD, SHAPE],
        [[11, 1.0, 1.0, 1.0], BUILD, [11, 2.0, 2.0, 2.0], BUILD, SHAPE],
        [[11, 1.0, 0.0, 0.0], BUILD, [11, 2.0, 2.0, 2.0], BUILD, SHAPE, SOLID],
        [[11, 1.0, 1.0, 1.0], SHELL, BUILD, SHAPE, SOLID],
        [[11, 1.0, 1.0, 0.0], SOLID, IS_DONE],
    ]
    for sequence in sequences:
        requests.append('pv_box ' + numbers(calls_fields(sequence)))


WEDGE_SEQUENCES = [[0], [1, 2], [2, 1, 5], [4, 3, 4, 0], [5, 2]]


def wedge_maker_values() -> list[tuple[int, list[float]]]:
    """(kind, values) of MakeWedge from the BRepPrim wedge shapes: the ltx wedges and the
    limited ones (kind 3 of BRepPrim with the minima at the origin)."""
    values = []
    for kind, v in wedge_shapes():
        if kind == 2:
            values.append((0, v))
        elif kind == 3 and v[0] == v[1] == v[2] == 0.0:
            values.append((2, [v[5], v[6], v[7], v[4], v[3], v[9], v[8]]))
    values += [(0, [1.0, 2.0, 3.0, -2.0]), (0, [-1.0, 2.0, 3.0, 1.0]), (2, [1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0])]
    return values


def append_wedge_requests(requests: list[str]) -> None:
    """Every wedge maker on the reference frame, then on another frame, with every call
    sequence."""
    for index, (kind, values) in enumerate(wedge_maker_values()):
        for sequence in WEDGE_SEQUENCES:
            requests.append('pa_wedge ' + numbers([kind] + values + [len(sequence)] + sequence))
        frame = FRAMES[1 + index % 3]
        for sequence in WEDGE_SEQUENCES[:3]:
            requests.append('pa_wedge ' + numbers([kind + 1] + frame + values + [len(sequence)] + sequence))


ONE_AXIS_SEQUENCES = [[0], [1, 2, 3], [3, 1], [5, 4, 5, 6, 7], [2, 0, 3]]
F1, F2 = FRAMES[1], FRAMES[2]
TRIMMED_LINE = [7, 0.0, 2.0, 1, 0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]
SLANT = [7, 0.0, 1.0, 1, 0, 0.0, 0.0, 0.0, 0.6, 0.0, 0.8]
XZ_CIRCLE = [1, 3.0, 0.0, 0.0, 0.0, -1.0, 0.0, 1.0]
XZ_BSPLINE = [6, 3, 0, 4, 1.0, 0.0, 0.0, 2.0, 0.0, 1.0, 1.5, 0.0, 2.0, 1.0, 0.0, 3.0, 2, 0.0, 4, 1.0, 4]
LINE = [0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]


def one_axis_makers() -> list[list[float]]:
    """`maker kind values...` of every one-axis constructor."""
    makers = []
    for rh in ([2.0, 3.0], [-1.0, 2.0], [1.0, 0.0], [0.0, 1.0], [1e-8, 1.0]):
        makers += [[0, 0] + rh, [0, 2] + F1 + rh]
    for angle in (PI, PI / 2, 2 * PI, 3 * PI, -1.0, 0.0, 1e-13):
        makers += [[0, 1, 2.0, 3.0, angle], [0, 3] + F2 + [2.0, 3.0, angle]]
    for rrh in ([5.0, 2.0, 10.0], [0.0, 3.0, 4.0], [5.0, 0.0, 10.0], [2.0, 2.0, 1.0], [-1.0, 2.0, 1.0], [1.0, 2.0, 0.0]):
        makers += [[1, 0] + rrh, [1, 2] + F1 + rrh]
    for angle in (PI, 2 * PI, -1.0, 7.0):
        makers += [[1, 1, 5.0, 2.0, 10.0, angle], [1, 3] + F2 + [0.0, 3.0, 4.0, angle]]
    sphere_values = [[5.0], [-1.0], [0.0], [5.0, 1.0], [5.0, -1.0], [5.0, 2 * PI], [5.0, 3 * PI],
                     [5.0, 0.0, PI / 2], [5.0, -PI / 4, PI / 4], [5.0, PI / 2, 0.0], [5.0, -2.0, 2.0],
                     [5.0, 0.0, PI / 2, PI / 2], [5.0, -0.5, 0.5, -1.0]]
    for values in sphere_values:
        n = len(values) - 1
        makers += [[2, n] + values, [2, 4 + n, 1.0, -1.0, 2.0] + values, [2, 8 + n] + F2 + values]
    torus_values = [[10.0, 2.0], [2.0, 10.0], [10.0, -2.0], [0.0, 1.0], [10.0, 2.0, 1.0], [10.0, 2.0, 2 * PI],
                    [10.0, 2.0, 7.0], [10.0, 2.0, -PI / 2, PI / 2], [10.0, 2.0, PI / 2, 3 * PI / 2],
                    [10.0, 2.0, 0.0, 2 * PI], [10.0, 2.0, 1.0, 0.5], [10.0, 2.0, 0.0, PI, PI / 4],
                    [3.0, 1.0, -PI / 6, PI / 3, 2.0]]
    for values in torus_values:
        n = len(values) - 2
        makers += [[3, n] + values, [3, 4 + n] + F1 + values]
    meridians = [
        (TRIMMED_LINE, [[], [1.0], [0.0, 1.0], [0.5, 1.5, 2.0]]),
        (SLANT, [[], [PI], [0.0, 0.5], [0.0, 1.0, 1.0]]),
        (XZ_CIRCLE, [[], [1.5], [0.0, PI], [-1.0, 1.0, 2.0]]),
        (XZ_BSPLINE, [[], [2.0], [0.2, 0.8], [0.0, 1.0, 3 * PI]]),
        (LINE, [[0.0, 2.0], [-1.0, 1.0, 1.0]]),
    ]
    for curve, value_sets in meridians:
        for values in value_sets:
            n = len(values)
            makers += [[4, n] + curve + values, [4, 4 + n] + F1 + curve + values]
    return makers


def append_one_axis_requests(requests: list[str]) -> None:
    """Every one-axis maker with every call sequence."""
    for maker in one_axis_makers():
        for sequence in ONE_AXIS_SEQUENCES:
            requests.append('pa_oneaxis ' + numbers(maker + [len(sequence)] + sequence))


PRISM_KINDS = [[0, 0.0, 0.0, 1.0], [0, 1.0, 2.0, 3.0], [0, 1e-8, 0.0, 0.0], [1, 0.0, 0.0, 1.0, 1], [1, 0.6, 0.0, 0.8, 0]]


def append_prism_requests(requests: list[str]) -> None:
    """Every prism shape along every vector or direction, with the flags varied."""
    for index, (name, builder) in enumerate(sorted(PRISM_SHAPES.items())):
        fields = program_fields(builder)
        for k, kind in enumerate(PRISM_KINDS):
            copy, canonize = (index + k) % 2, (index + k) // 2 % 2
            requests.append(f'pa_prism {fields} {numbers(kind + [copy, canonize])}')


def triangle_face(p) -> int:
    """A triangle face in the XZ plane with one vertex on the Z axis."""
    from tktopalgo_corpus_brepcheck_shapes import PlanarShapes
    shapes = PlanarShapes(p, [(0, 0, 1), (1, 0, 0), (1, 0, 2)])
    face = shapes.face([[0, 1, 2]])
    shapes.finish()
    return face


REVOL_API_SHAPES = dict(REVOL_SHAPES)
REVOL_API_SHAPES.update({
    'crossing_segment': with_foreign(lambda p: segment(p, (-1.0, 0.0, 1.0), (1.0, 0.0, 1.0))),
    'touching_wire': with_foreign(lambda p: polyline(p, [(1, 0, 0), (0, 0, 1), (1, 0, 2)], False)),
    'touching_triangle': with_foreign(triangle_face),
})
REVOL_AXES = [[0.0, 0.0, 0.0, 0.0, 0.0, 1.0], [0.5, 0.0, 0.0, 0.0, 0.0, 1.0]]
REVOL_ANGLES = [PI / 2, 2 * PI, -PI / 2, 1e-13]


def append_revol_requests(requests: list[str]) -> None:
    """Every revolution shape around two axes by every angle and completely."""
    for index, (name, builder) in enumerate(sorted(REVOL_API_SHAPES.items())):
        fields = program_fields(builder)
        for a, axis in enumerate(REVOL_AXES):
            for k, angle in enumerate(REVOL_ANGLES):
                requests.append(f'pa_revol {fields} {numbers([0] + axis + [angle, (index + a + k) % 2])}')
            requests.append(f'pa_revol {fields} {numbers([1] + axis + [(index + a) % 2])}')


HALF_SPACE_SHAPES = {
    'square': (0, with_foreign(lambda p: square(p, 2.0).faces[0])),
    'cylinder_face': (0, with_foreign(lambda p: cylinder_face(p, 1.0, 2.0)['face'])),
    'cube_shell': (1, with_foreign(lambda p: cube(p, 1.0, None).shell())),
    'vertex': (0, with_foreign(lambda p: p.vertex((1.0, 2.0, 3.0)))),
}
HALF_SPACE_POINTS = [[1.0, 1.0, 1.0], [1.0, 1.0, -1.0], [3.0, 1.0, 1.0], [3.0, 3.0, 1.0], [1.0, 1.0, 0.0],
                     [0.5, 0.5, 0.5], [0.2, 0.3, 5.0], [0.0, 0.0, 1.0]]


def append_half_space_requests(requests: list[str]) -> None:
    """Every bounding shape from every point, and with the other cast (a type mismatch)."""
    for name, (kind, builder) in sorted(HALF_SPACE_SHAPES.items()):
        fields = program_fields(builder)
        for point in HALF_SPACE_POINTS:
            requests.append(f'pa_halfspace {fields} {numbers([kind] + point)}')
        requests.append(f'pa_halfspace {fields} {numbers([1 - kind, 1.0, 1.0, 1.0])}')


def append_brepprimapi_requests(requests: list[str]) -> None:
    """Append the BRepPrimAPI and BRepPreviewAPI requests."""
    append_box_requests(requests)
    append_preview_requests(requests)
    append_wedge_requests(requests)
    append_one_axis_requests(requests)
    append_prism_requests(requests)
    append_revol_requests(requests)
    append_half_space_requests(requests)
