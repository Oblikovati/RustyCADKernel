"""Deterministic requests for BRepIntCurveSurface
(tools/oracle/tktopalgo_probe_brepintcurvesurface.cxx, docs/topalgo-design.md section 12).

Every request carries its shape record (tktopalgo_corpus_shapes.py, the named shapes of
tktopalgo_corpus_solids.py and tktopalgo_corpus_brepclass3d.py) and never relies on an
earlier request:
- `bics_line`: Init(shape, line, tol) on one object for lines through the centre, along
  the axes, through edges and vertices, tangent to the curved faces, in the plane of a
  face and missing the shape, on boxes (tolerances 1e-7, 1e-3 and 0.5, located and
  rotated, with a flipped face, with a B-spline top), cylinders, spheres, a torus, hollow
  boxes, an open box, a square sheet, a half-space, compounds and an empty solid; each
  exploration prints every point (face, point, parameters, state, transition) with
  Load/Init/More/Next, then the accessors past the end;
- `bics_curve`: Load (once or twice) then Init(curve) for several curves on one object,
  and Init(shape, curve, tol): finite and infinite line segments, circles, B-spline curves
  and parabolas (infinite ones only on analytic shapes: they have no curve box);
- `bics_state`: a fresh object, Init before Load, Load then Next with the previous curve,
  and the Next calls after the last face (the face index keeps growing).
"""
from __future__ import annotations
import math
from tktopalgo_corpus_brepbndlib import numbers
from tktopalgo_corpus_brepclass3d import (flipped_face_box, half_space, spline_top_box, square_sheet,
                                          two_boxes, with_location)
from tktopalgo_corpus_shapes import ShapeSpec, rotation, translation
from tktopalgo_corpus_solids import box, cylinder, hollow_box, open_box, sphere, sphere_in_box, torus

TOLERANCES = (1e-7, 1e-3, 0.5)
SQ = 1.0 / math.sqrt(3.0)


def unit(v: list[float]) -> list[float]:
    """The unit vector of v."""
    n = math.sqrt(sum(c * c for c in v))
    return [c / n for c in v]


def cube_lines(low: float, high: float) -> list[list[float]]:
    """Lines `P D` across a cube: through the centre along the axes and a diagonal, through
    a face centre, along and across edges, through vertices, in a face plane, near a face
    (1e-8 and 1e-4 outside) and missing it."""
    mid = (low + high) / 2.0
    return [[mid, mid, low - 5.0, 0.0, 0.0, 1.0], [low - 5.0, mid, mid, 1.0, 0.0, 0.0],
            [mid, low - 5.0, mid, 0.0, 1.0, 0.0], [mid, mid, mid, SQ, SQ, SQ],
            [low + 1.0, low + 2.0, high + 3.0, 0.0, 0.0, -1.0], [low, low, low - 3.0, 0.0, 0.0, 1.0],
            [low - 3.0, low, mid, 1.0, 0.0, 0.0], [low, mid, low - 2.0, 0.0, 0.0, 1.0],
            [mid, mid, high, 1.0, 0.0, 0.0], [low - 1e-8, mid, low - 4.0, 0.0, 0.0, 1.0],
            [low - 1e-4, mid, low - 4.0, 0.0, 0.0, 1.0], [high + 5.0, high + 5.0, mid, 0.0, 0.0, 1.0],
            [low - 2.0, low - 2.0, low - 2.0, *unit([1.0, 1.0, 1.0])], [mid, mid, mid, *unit([0.3, -0.2, 0.9])],
            [low - 1.0, mid, mid, *unit([1.0, 0.5, 0.0])]]


def round_lines(centre: list[float], radius: float, height: float) -> list[list[float]]:
    """Lines across a cylinder or sphere of axis z: through the axis, across it, tangent,
    along the axis and along the surface, and missing it."""
    cx, cy, cz = centre
    return [[cx - 3.0 * radius, cy, cz + 0.5 * height, 1.0, 0.0, 0.0], [cx, cy, cz - 2.0 * radius, 0.0, 0.0, 1.0],
            [cx - 3.0 * radius, cy + radius, cz + 0.4 * height, 1.0, 0.0, 0.0],
            [cx - 3.0 * radius, cy + 0.5 * radius, cz + 0.3 * height, 1.0, 0.0, 0.0],
            [cx + radius, cy, cz - 1.0, 0.0, 0.0, 1.0], [cx, cy, cz + 0.2 * height, *unit([1.0, 1.0, 0.3])],
            [cx - 3.0 * radius, cy + 2.0 * radius, cz, 1.0, 0.0, 0.0],
            [cx + 0.3, cy - 0.2, cz + 0.1, *unit([-0.2, 0.7, 0.6])]]


def line_shapes() -> list[tuple[ShapeSpec, list[list[float]]]]:
    """Each shape with the lines to intersect it with."""
    cube = cube_lines(0.0, 10.0)
    return [
        (box([0.0, 0.0, 0.0], [10.0, 10.0, 10.0]), cube),
        (box([0.0, 0.0, 0.0], [10.0, 10.0, 10.0], tol=1e-3), cube[:9]),
        (with_location(box([0.0, 0.0, 0.0], [10.0, 10.0, 10.0]), translation(1.0, 2.0, 3.0)), cube[:8]),
        (with_location(box([0.0, 0.0, 0.0], [10.0, 10.0, 10.0]), rotation([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 0.5)),
         cube[:8]),
        (flipped_face_box(), cube[:6]),
        (spline_top_box(), cube),
        (cylinder([1.0, 2.0, 3.0], 2.0, 5.0), round_lines([1.0, 2.0, 3.0], 2.0, 5.0)),
        (sphere([0.0, 0.0, 0.0], 10.0), round_lines([0.0, 0.0, 0.0], 10.0, 0.0)),
        (torus([0.0, 0.0, 0.0], 5.0, 1.0), [[-10.0, 0.0, 0.0, 1.0, 0.0, 0.0], [-10.0, 0.0, 0.5, 1.0, 0.0, 0.0],
                                           [5.0, 0.0, -3.0, 0.0, 0.0, 1.0], [-10.0, 0.0, 1.0, 1.0, 0.0, 0.0],
                                           [0.0, 0.0, -3.0, 0.0, 0.0, 1.0], [4.0, -8.0, 0.2, 0.0, 1.0, 0.0]]),
        (hollow_box([0.0, 0.0, 0.0], [10.0, 10.0, 10.0], [3.0, 3.0, 3.0], [7.0, 7.0, 7.0]), cube[:6]),
        (sphere_in_box([0.0, 0.0, 0.0], [10.0, 10.0, 10.0], [5.0, 5.0, 5.0], 2.0), cube[:6]),
        (open_box([0.0, 0.0, 0.0], [10.0, 10.0, 10.0]), cube[:6]),
        (square_sheet(), [[5.0, 5.0, -3.0, 0.0, 0.0, 1.0], [-3.0, 5.0, 0.0, 1.0, 0.0, 0.0],
                          [5.0, 5.0, -3.0, *unit([0.2, 0.1, 1.0])], [15.0, 5.0, -3.0, 0.0, 0.0, 1.0]]),
        (half_space(), [[3.0, 4.0, -5.0, 0.0, 0.0, 1.0], [0.0, 0.0, 1.0, 1.0, 0.0, 0.0],
                        [1e5, -1e5, 3.0, *unit([0.1, 0.0, -1.0])]]),
        (two_boxes(), [[-3.0, 1.0, 1.0, 1.0, 0.0, 0.0], [1.0, 1.0, -3.0, 0.0, 0.0, 1.0]]),
        (ShapeSpec(), [[0.0, 0.0, 0.0, 0.0, 0.0, 1.0]]),
    ]


def _line_requests(requests: list[str]) -> None:
    for spec, lines in line_shapes():
        for tol in TOLERANCES:
            flat = [c for line in lines for c in line]
            requests.append('bics_line ' + numbers([*spec.numbers(), tol, len(lines), *flat]))
        for line in lines:
            requests.append('bics_line ' + numbers([*spec.numbers(), 1e-7, 1, *line]))


def segment(p: list[float], d: list[float], first: float, last: float) -> list[float]:
    """A `tp_bics_curve` line record on [first, last]."""
    return [0, *p, *d, first, last]


def circle(centre: list[float], normal: list[float], x_dir: list[float], radius: float,
           first: float = 0.0, last: float = 2.0 * math.pi) -> list[float]:
    """A `tp_bics_curve` circle record."""
    return [1, *centre, *normal, *x_dir, radius, first, last]


def parabola(apex: list[float], normal: list[float], x_dir: list[float], focal: float,
             first: float, last: float) -> list[float]:
    """A `tp_bics_curve` parabola record."""
    return [3, *apex, *normal, *x_dir, focal, first, last]


def bspline_curve(poles: list[list[float]], first: float = 0.0, last: float = 1.0) -> list[float]:
    """A `tp_bics_curve` cubic B-spline record with uniform clamped knots."""
    inner = len(poles) - 3
    knots = [[i / inner, 4 if i in (0, inner) else 1] for i in range(inner + 1)]
    return [2, 1, 3, len(poles), *[c for p in poles for c in p], len(knots), *[x for k in knots for x in k],
            first, last]


def cube_curves(low: float, high: float) -> list[list[float]]:
    """Finite and infinite segments, circles, a B-spline and parabolas across a cube."""
    mid = (low + high) / 2.0
    wave = [[low - 2.0 + 2.8 * i, mid + 3.0 * math.sin(i), mid + 2.0 * math.cos(1.3 * i)] for i in range(6)]
    return [segment([mid, mid, low - 5.0], [0.0, 0.0, 1.0], 0.0, 10.0),
            segment([mid, mid, low - 5.0], [0.0, 0.0, 1.0], 0.0, 7.0),
            segment([mid, mid, low - 5.0], [0.0, 0.0, 1.0], -1e100, 1e100),
            segment([mid, mid, low - 5.0], [0.0, 0.0, 1.0], 6.0, 30.0),
            circle([mid, mid, mid], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0], 0.75 * (high - low)),
            circle([mid, mid, mid], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0], 0.75 * (high - low), 0.5, 2.0),
            circle([mid, mid, high + 1.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0], 3.0),
            bspline_curve(wave),
            bspline_curve(wave, 0.2, 0.7),
            parabola([mid, mid, mid], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0], 1.0, -8.0, 8.0),
            parabola([mid, mid, mid], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0], 1.0, -1e100, 1e100)]


def curve_shapes() -> list[tuple[ShapeSpec, list[list[float]]]]:
    """Each shape with the curves to intersect it with (infinite parabolas on analytic
    shapes only)."""
    cube = cube_curves(0.0, 10.0)
    finite = cube[:10]
    return [
        (box([0.0, 0.0, 0.0], [10.0, 10.0, 10.0]), cube),
        (spline_top_box(), finite),
        (cylinder([5.0, 5.0, 0.0], 3.0, 10.0), cube),
        (sphere([5.0, 5.0, 5.0], 5.0), cube),
        (torus([5.0, 5.0, 5.0], 4.0, 1.0), finite),
        (hollow_box([0.0, 0.0, 0.0], [10.0, 10.0, 10.0], [3.0, 3.0, 3.0], [7.0, 7.0, 7.0]), cube),
        (with_location(box([0.0, 0.0, 0.0], [10.0, 10.0, 10.0]), rotation([5.0, 5.0, 5.0], [0.0, 0.0, 1.0], 0.3)),
         finite),
    ]


def _curve_requests(requests: list[str]) -> None:
    for spec, curves in curve_shapes():
        flat = [c for curve in curves for c in curve]
        for tol in TOLERANCES:
            for mode, reload in ((0, 0), (0, 1), (1, 0)):
                requests.append('bics_curve ' + numbers([*spec.numbers(), tol, mode, reload, len(curves), *flat]))
        for curve in curves:
            requests.append('bics_curve ' + numbers([*spec.numbers(), 1e-7, 0, 0, 1, *curve]))


def _state_requests(requests: list[str]) -> None:
    shapes = [box([-1.0, -1.0, -0.5], [1.0, 1.0, 0.5]), box([2.0, 2.0, 2.0], [3.0, 3.0, 3.0]),
              sphere([0.0, 0.0, 0.0], 0.75), cylinder([0.0, 0.0, -2.0], 0.5, 4.0), ShapeSpec(), square_sheet()]
    for spec in shapes:
        for tol in TOLERANCES:
            requests.append('bics_state ' + numbers([*spec.numbers(), tol]))


def append_brepintcurvesurface_requests(requests: list[str]) -> None:
    """Append the BRepIntCurveSurface request families."""
    _line_requests(requests)
    _curve_requests(requests)
    _state_requests(requests)
