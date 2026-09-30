"""Curve projection with full output geometry, status and retained endpoint cache."""

from itertools import product
from math import pi
from tktopalgo_corpus_breplib_records import (
    numbers,
    geom_gp,
    geom_bezier,
    geom_bspline,
    geom_trimmed,
)
from tkshhealing_corpus_analysis_surface import analysis_surfaces


def projection_curves() -> list[tuple[list[float], tuple[float, float]]]:
    """Analytical, nested and uneven splines; projection_curves()."""
    circle = geom_gp(1, radius=5.0, p=(0.0, 0.0, 3.0))
    poles = [(0.0, 0.0, 0.0), (2.0, 3.0, 0.0), (5.0, 1.0, 2.0), (8.0, 2.0, 0.0)]
    spline = geom_bspline(2, False, poles, [(0.0, 3), (0.5, 1), (1.0, 3)])
    uneven = geom_bspline(2, False, poles, [(0.0, 3), (0.000001, 1), (1.0, 3)])
    return [
        (geom_gp(0, p=(5.0, 0.0, 0.0), d=(0.0, 0.0, 1.0)), (0.0, 10.0)),
        (circle, (0.0, 2.0 * pi)),
        (geom_gp(1, radius=10.0), (0.0, 2.0 * pi)),
        (geom_gp(1, radius=0.1, p=(0.0, 0.0, 10.0)), (0.0, 2.0 * pi)),
        (geom_gp(2, major=10.0, minor=5.0), (0.0, 2.0 * pi)),
        (geom_bezier(poles), (0.0, 1.0)),
        (spline, (0.0, 1.0)),
        (uneven, (0.0, 1.0)),
        (geom_trimmed(0.1, 0.4, geom_trimmed(0.0, 0.5, spline)), (0.1, 0.4)),
    ]


def append_project_curve_requests(requests: list[str]) -> None:
    """Append elementary/spline surfaces, cache cycles and native errors; append_project_curve_requests(rows)."""
    for surface, (curve, interval), operation in product(
        analysis_surfaces(), projection_curves(), [0, 1]
    ):
        request = [operation, 1, *surface, 1e-7, 1, *curve, *interval, -1.0, -1.0, 2]
        requests.append("sh_project_curve " + numbers(request))
    append_projection_modes(requests)
    append_polar_projections(requests)


def append_projection_modes(requests: list[str]) -> None:
    """Exercise initialization and degeneracy settings; append_projection_modes(rows)."""
    surfaces = analysis_surfaces()[:7]
    for surface, initialization, mode, tolerance in product(
        surfaces, range(4), [-1, 0, 1], [1e-15, 1e-4, 2.0]
    ):
        curve = geom_gp(1, radius=5.0, p=(0.0, 0.0, 3.0))
        request = [
            0,
            initialization,
            *surface,
            tolerance,
            mode,
            *curve,
            0.0,
            2.0 * pi,
            0.01,
            0.2,
            2,
        ]
        requests.append("sh_project_curve " + numbers(request))


def append_polar_projections(requests: list[str]) -> None:
    """Cross sphere poles and cylinder/torus seams; append_polar_projections(rows)."""
    surfaces = analysis_surfaces()[1:7]
    curves = [
        geom_gp(1, radius=10.0, n=(0.0, 1.0, 0.0)),
        geom_bezier([(10.0, 0.0, 0.0), (10.0, 0.0, 10.0), (0.0, 0.0, 10.0)]),
        geom_bezier([(0.0, 0.0, 10.0), (0.0, 0.0, 10.0), (5.0, 5.0, 5.0)]),
        geom_bspline(
            2,
            False,
            [
                (5.0, 0.0, 0.0),
                (5.0, 0.0, 3.0),
                (5.0, 0.0, 3.0),
                (0.0, 5.0, 6.0),
                (-5.0, 0.0, 9.0),
            ],
            [(0.0, 3), (0.5, 2), (1.0, 3)],
        ),
    ]
    for surface, curve, mode in product(surfaces, curves, [-1, 0, 1]):
        interval = (0.0, 2.0 * pi) if curve[0] == 1 else (0.0, 1.0)
        requests.append(
            "sh_project_curve "
            + numbers([0, 1, *surface, 1e-7, mode, *curve, *interval, 1.0, 1.0, 3])
        )
