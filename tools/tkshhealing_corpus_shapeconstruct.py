"""ShapeConstruct_Curve branch, aliasing, exception and OCC90 knot-mutation corpus."""

from itertools import product
from tktopalgo_corpus_breplib_records import (
    geom_gp,
    geom_bezier,
    geom_bspline,
    geom_trimmed,
    geom_offset,
    geom2d_gp,
    geom2d_bezier,
    geom2d_bspline,
    geom2d_trimmed,
    numbers,
)

CONSTRUCT_PERIODIC_KNOTS: list[tuple[float, int]] = [
    (0.0, 1),
    (1.0, 1),
    (2.0, 1),
    (3.0, 1),
    (4.0, 1),
]


def construct_curves_3d() -> list[list[float]]:
    """All adjusted/converted kinds, including wrappers; construct_curves_3d()."""
    poles = [(0.0, 0.0, 0.0), (1.0, 2.0, 1.0), (3.0, 0.0, 2.0)]
    spline = geom_bspline(2, False, poles, [(0.0, 3), (1.0, 3)])
    periodic = geom_bspline(
        2,
        True,
        poles + [(2.0, -1.0, 0.0)],
        CONSTRUCT_PERIODIC_KNOTS,
    )
    return [
        geom_gp(0, p=(2.0, 3.0, 4.0)),
        spline,
        periodic,
        geom_bezier(poles),
        *[geom_gp(kind) for kind in range(1, 5)],
        geom_trimmed(0.1, 0.9, spline),
        geom_trimmed(0.0, 1.0, geom_gp(0)),
        geom_offset(0.2, (0.0, 0.0, 1.0), spline),
    ]


def construct_curves_2d() -> list[list[float]]:
    """The corresponding p-curve kinds; construct_curves_2d()."""
    poles = [(0.0, 0.0), (1.0, 2.0), (3.0, 0.0)]
    spline = geom2d_bspline(2, False, poles, [(0.0, 3), (1.0, 3)])
    periodic = geom2d_bspline(
        2,
        True,
        poles + [(2.0, -1.0)],
        CONSTRUCT_PERIODIC_KNOTS,
    )
    return [
        geom2d_gp(0, p=(2.0, 3.0)),
        spline,
        periodic,
        geom2d_bezier(poles),
        *[geom2d_gp(kind) for kind in range(1, 5)],
        geom2d_trimmed(0.1, 0.9, spline),
        geom2d_trimmed(0.0, 1.0, geom2d_gp(0)),
    ]


def construct_adjustments() -> list[str]:
    """Flags, equal endpoints and all segment bounds; construct_adjustments()."""
    requests: list[str] = []
    for dimension, curves in [(3, construct_curves_3d()), (2, construct_curves_2d())]:
        for curve, take1, take2, same in product(curves, (0, 1), (0, 1), (False, True)):
            first = [2.0, 1.0, -1.0][:dimension]
            last = first if same else [5.0, 4.0, 2.0][:dimension]
            fields = [0 if dimension == 3 else 2, *curve, *first, *last, take1, take2]
            requests.append("sh_construct " + numbers(fields))
    for curve, bounds in product(
        construct_curves_3d(),
        [(0.0, 1.0), (-1.0, 0.75), (0.25, 2.0), (1.0, 0.0), (0.5, 0.5), (2.0, 3.0)],
    ):
        requests.append(
            "sh_construct "
            + numbers([1, *curve, 2.0, 1.0, -1.0, 5.0, 4.0, 2.0, *bounds])
        )
    return requests


def construct_conversions() -> list[str]:
    """Unchanged handles, copies, constructor errors and approximation fallback; construct_conversions()."""
    requests: list[str] = []
    bounds = [
        (0.0, 1.0),
        (0.25, 0.75),
        (-0.5, 1.5),
        (1.0, 0.0),
        (0.5, 0.5),
        (2.0, 3.0),
        (0.5e-9, 1.0 - 0.5e-9),
        (2e-9, 1.0 - 2e-9),
    ]
    for operation, curves in [(3, construct_curves_3d()), (4, construct_curves_2d())]:
        for curve, (first, last), tolerance in product(curves, bounds, [1e-7, 1e-3]):
            requests.append(
                "sh_construct " + numbers([operation, *curve, first, last, tolerance])
            )
    return requests


def construct_knot_requests() -> list[str]:
    """Lower bounds and partial mutation before rank errors; construct_knot_requests()."""
    values = [
        [0.0],
        [-0.0, -0.0, 0.0],
        [0.0, 1.0, 2.0],
        [0.0, 0.0, 0.0],
        [1.0, 1.0, 0.0, 2.0],
        [1.0, 1.0 + 2.0**-52, 1.0 + 2.0**-51],
        [-2.0, -2.0, -3.0, 1.0],
        [1e100, 1e100, 1e100],
        [1e-300, 1e-300, 1e-300],
    ]
    return [
        "sh_construct " + numbers([operation, lower, len(knots), *knots])
        for operation, lower, knots in product((5, 6), (-2, 0, 1, 2), values)
    ]


def append_shapeconstruct_requests(requests: list[str]) -> None:
    """Append all seven APIs and their failure branches; append_shapeconstruct_requests(rows)."""
    requests.extend(construct_adjustments())
    requests.extend(construct_conversions())
    requests.extend(construct_knot_requests())
