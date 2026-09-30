"""The BRepLib_ValidateEdge requests of the TKTopAlgo corpus: a reference 3d curve (or a
curve on surface) against a curve on surface, with the same or another parametrization,
every control point count, with and without the early exit, and the exact method."""
from __future__ import annotations
import math
from tktopalgo_shape_program import ShapeProgram, num

(SET_EXACT, SET_PARALLEL, SET_POINTS, SET_EXIT, PROCESS, IS_DONE, MAX_DISTANCE, CHECK, UPDATE, IS_EXACT,
 IS_PARALLEL) = range(1, 12)


def validate(program: ShapeProgram, reference: tuple, other: tuple, same_parameter: bool, calls: list) -> str:
    """`ble_validate program reference other sameparameter script`."""
    flat = [v for call in calls for v in (call if isinstance(call, tuple) else (call,))]
    fields = [*reference, *other, same_parameter, len(calls), *flat]
    return f"ble_validate {program.tokens()} {' '.join(map(num, fields))}"


def readout(tolerance: float) -> list:
    """The results after Process."""
    return [IS_DONE, MAX_DISTANCE, (CHECK, tolerance), (CHECK, 0.0), (UPDATE, 0.0), (UPDATE, 1e9), IS_EXACT,
            IS_PARALLEL]


def option_scripts(tolerance: float) -> list[list]:
    """The option sets: defaults, point counts, early exit, exact and parallel."""
    scripts: list[list] = [[MAX_DISTANCE, IS_DONE, PROCESS]]
    for points in (0, -3, 1, 2, 7, 100):
        scripts.append([(SET_POINTS, points), PROCESS])
    for exit_tolerance in (1e-12, tolerance, 1.0):
        scripts.append([(SET_EXIT, exit_tolerance), PROCESS])
    scripts.append([(SET_EXACT, 1), PROCESS])
    scripts.append([(SET_EXACT, 1), (SET_PARALLEL, 1), (SET_EXIT, 1e-12), PROCESS])
    scripts.append([(SET_PARALLEL, 1), (SET_POINTS, 5), PROCESS, PROCESS])
    return [script+readout(tolerance) for script in scripts]


def line_cases(requests: list[str]) -> None:
    """A 3d line against lines on a plane: exact, shifted, on another range, not same
    parameter."""
    for shift in (0.0, 1e-8, 1e-3):
        p = ShapeProgram()
        c = p.line((0, 0, 0), (1, 0, 0))
        plane = p.surface(0, 0, 0, 0, 0, 0, 1)
        pc = p.line2d((0.0, shift), (1.0, 0.0))
        for other_range, same in (((0.0, 1.0), True), ((0.0, 1.0), False), ((0.0, 1.5), True), ((0.1, 1.0), True)):
            for calls in option_scripts(1e-7):
                requests.append(validate(p, (0, c, 0.0, 1.0), (pc, *other_range, plane), same, calls))


def conic_cases(requests: list[str]) -> None:
    """Circles and ellipses on planes and cones (the precisions of BRepCheck)."""
    p = ShapeProgram()
    circle = p.curve(1, 0, 0, 0, 0, 0, 1, 2.0)
    ellipse = p.curve(2, 1e3, -7, 0, 0, 0, 1, 3.0, 1.0)
    plane = p.surface(0, 0, 0, 0, 0, 0, 1)
    cone = p.surface(4, 0, 0, 0, 0, 0, 1, 2.0, 0.5)
    on_plane = p.pcurve(1, 0.0, 0.0, 2.0+1e-6, 1)
    on_cone = p.line2d((0.0, 0.0), (1.0, 0.0))
    ellipse_on_plane = p.pcurve(2, 1e3, -7.0, 3.0, 1.0)
    cases = [((0, circle, 0.0, 6.0), (on_plane, 0.0, 6.0, plane)),
             ((0, circle, 0.0, 6.0), (on_cone, 0.0, 6.0, cone)),
             ((0, ellipse, 1.0, 2.0), (ellipse_on_plane, 1.0, 2.0, plane)),
             ((0, ellipse, 1.0, 2.0), (ellipse_on_plane, 1.5, 2.5, plane)),
             ((1, on_cone, 0.0, 6.0, cone), (on_plane, 0.0, 6.0, plane))]
    for reference, other in cases:
        for same in (True, False):
            for calls in option_scripts(1e-7):
                requests.append(validate(p, reference, other, same, calls))


def spline_cases(requests: list[str]) -> None:
    """A B-spline 3d curve against a nearby Bezier pcurve on a plane."""
    p = ShapeProgram()
    spline = p.curve(5, 3, 5, 0, 0, 0, 1, 2, 0, 2, -1, 0, 3, 1, 0, 4, 0, 0, 3, 0, 4, 0.5, 1, 1, 4)
    plane = p.surface(0, 0, 0, 0, 0, 0, 1)
    bezier = p.pcurve(3, 4, 0.0, 0.0, 1.0, 2.1, 3.0, -1.0, 4.0, 0.0)
    for other_range in ((0.0, 1.0), (0.2, 0.9)):
        for calls in option_scripts(1e-2):
            requests.append(validate(p, (0, spline, 0.0, 1.0), (bezier, *other_range, plane), True, calls))


def append_validate_requests(requests: list[str]) -> None:
    line_cases(requests)
    conic_cases(requests)
    spline_cases(requests)


__all__ = ['append_validate_requests', 'math']
