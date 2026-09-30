"""Native surface singularity, closure, projection and partial-output comparisons."""

from itertools import product
from tktopalgo_corpus_breplib_records import numbers, geom_bezier, geom_gp

SURFACE_ANALYSIS_FRAME: list[float] = [0.0, 0.0, 0.0, 0.0, 0.0, 1.0]


def analytic_analysis_surfaces() -> list[list[float]]:
    """Return elementary surface records; analytic_analysis_surfaces()."""
    frame = SURFACE_ANALYSIS_FRAME
    return [
        [0, *frame],
        [1, *frame, 5.0],
        [2, *frame, 10.0],
        [3, *frame, -1.0, 3.0, 2.0, 8.0],
        [4, *frame, 5.0, 0.4],
        [5, *frame, 5.0, 2.0],
        [5, *frame, 2.0, 5.0],
    ]


def polynomial_analysis_surfaces(count: int) -> list[list[float]]:
    """Include nonsquare grids to guard read order; polynomial_analysis_surfaces(4)."""
    poles = [
        coordinate
        for u, v in product(range(count), range(3))
        for coordinate in [float(u), float(v), float(u * v) / 10.0]
    ]
    bezier = [6, *SURFACE_ANALYSIS_FRAME, count, 3, *poles]
    spline = [
        7,
        *SURFACE_ANALYSIS_FRAME,
        count - 1,
        2,
        count,
        3,
        *poles,
        2,
        0.0,
        count,
        1.0,
        count,
        2,
        0.0,
        3,
        1.0,
        3,
    ]
    return [bezier, spline]


def analysis_surfaces() -> list[list[float]]:
    """Return elementary, bounded, offset and swept surfaces; analysis_surfaces()."""
    records = analytic_analysis_surfaces()
    for count in [2, 3, 4]:
        records.extend(polynomial_analysis_surfaces(count))
    wrapped = [[0, *record] for record in records]
    wrapped.extend([[1, 0.1, *records[3]], [1, 0.1, *records[-1]]])
    curve = geom_bezier([(1.0, 0.0, 0.0), (2.0, 1.0, 1.0), (1.0, 2.0, 2.0)])
    wrapped.extend([[2, *curve], [3, *curve], [2, *geom_gp(0)]])
    return wrapped


def append_analysis_request(requests: list[str], values: list[float]) -> None:
    """Encode one independent request; append_analysis_request(rows, values)."""
    request = "sh_analysis_surface " + numbers(values)
    requests.append(request)


def append_analysis_projections(requests: list[str], surface: list[float]) -> None:
    """Exercise global and hinted projection; append_analysis_projections(rows, surface)."""
    hints = [(0.0, 0.0), (0.2, 0.3), (1.0, 2.0)]
    points = [(0.0, 0.0, 0.0), (1.0, 2.0, 3.0), (10.0, 0.0, 0.0), (0.0, 0.0, 10.0)]
    for hint, point, operation in product(hints, points, [2, 3]):
        append_analysis_request(
            requests, [operation, *surface, 1e-7, *hint, *point, 99.0, 98.0]
        )


def append_analysis_degenerate_points(
    requests: list[str], surface: list[float]
) -> None:
    """Keep one-point NaN and reversed traversal; append_analysis_degenerate_points(rows, surface)."""
    for direct, count, mixed in product([0, 1], [1, 2, 3], [False, True]):
        samples: list[float] = []
        for index in range(count):
            pole = 10.0 if not mixed or index != count // 2 else 0.0
            samples.extend([0.0, 0.0, pole, float(index * index), 1.5707963267948966])
        append_analysis_request(
            requests, [5, *surface, 1e-7, direct, count, count, *samples]
        )


def append_analysis_degenerate(requests: list[str], surface: list[float]) -> None:
    """Probe individual singular point state; append_analysis_degenerate(rows, surface)."""
    points = [(0.0, 0.0, 10.0), (0.0, 0.0, -10.0), (0.0, 0.0, 0.0)]
    for point, tolerance, latitude in product(
        points, [1e-7, 1.0, 10.0], [0.0, 1.5707963267948966]
    ):
        append_analysis_request(
            requests, [4, *surface, tolerance, *point, 2.0, latitude, 5.0, latitude]
        )


def append_analysis_surface_requests(requests: list[str]) -> None:
    """Append the complete independent corpus; append_analysis_surface_requests(rows)."""
    append_analysis_sequence_bounds(requests)
    for surface in analysis_surfaces():
        for tolerance, operation in product([-1.0, 0.0, 1e-7, 0.1, 1.0, 10.0], [0, 1]):
            append_analysis_request(requests, [operation, *surface, tolerance])
        append_analysis_projections(requests, surface)
        append_analysis_degenerate(requests, surface)
        append_analysis_degenerate_points(requests, surface)


def append_analysis_sequence_bounds(requests: list[str]) -> None:
    """Probe count independently from sequence length; append_analysis_sequence_bounds(rows)."""
    for surface, direct, count, stored in product(
        analysis_surfaces()[:3], [0, 1], [-1, 0, 1, 2, 4], [0, 1, 3]
    ):
        samples = [0.0, 0.0, 10.0, 1.0, 1.5707963267948966] * stored
        append_analysis_request(
            requests, [5, *surface, 1e-7, direct, count, stored, *samples]
        )
