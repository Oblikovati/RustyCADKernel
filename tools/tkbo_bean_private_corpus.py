"""Deterministic native and Rust requests for BeanFace's spline grid helper."""
from itertools import product


def grid_requests() -> list[str]:
    """Retain knot clipping, degenerate intervals, reused cache state and curvature cases."""
    rows = []
    bounds = [
        (0, 1, 0, 1), (0.2, 0.8, 0.3, 0.9), (-1, 2, -2, 3), (1, 0, 1, 0),
        (0.5, 0.5, 0.5, 0.5), (0, 1, 1, 0), (2, 3, 0, 1), (-3, -2, 0, 1),
    ]
    for degree, spans, height, interval, tolerance in product(
        [1, 2, 3], [1, 3, 15], [0, 0.5, 2], range(8), [-0.01, 0, 1e-7, 0.1]
    ):
        rows.append("bean_grid " + " ".join(map(str, [degree, spans, height, *bounds[interval], tolerance])))
    return rows


def line_plane_requests() -> list[str]:
    """Cover parallel, coincident, bounded and tolerance-expanded intersections."""
    from itertools import product
    origins = [(0, 0, 0), (0.5, -0.5, 1), (3, 4, 1e-8)]
    directions = [(1, 0, 0), (0, 0, 1), (1, 1, 1), (1, 0, 1e-10), (1, 0, 1e-9), (1, 0, 1e-8)]
    ranges = [(-1, 1, -1, 1, -1, 1), (0, 1, 0, 1, 0, 1), (1, -1, -1, 1, -1, 1)]
    rows = []
    for origin, direction, height, reverse, bounds, tolerances in product(
        origins, directions, [-1, 0, 1], [0, 1], ranges, [(0, 0), (1e-7, 2e-7), (0.5, 0.5)]
    ):
        values = [*origin, *direction, height, reverse, *bounds, *tolerances]
        rows.append("bean_line_plane " + " ".join(map(str, values)))
    return rows


def fast_analytic_requests() -> list[str]:
    """Cover supported analytic type pairs and each strict distance boundary."""
    rows = []
    origins = [(0, 0, 0), (0.5, 0, 1), (3, 4, 0)]
    directions = [((0, 0, 1), (0, 0, 1)), ((0, 0, 1), (0, 0, -1)),
                  ((0, 0, 1), (1, 0, 0)), ((1, 0, 0), (0, 0, 1))]
    radii = [(0.25, 0.25), (1.0, 1.0), (2.0, 0.5)]
    for curve_kind, surface_kind, origin_index, direction_pair, radius_pair, criteria in product(
        range(8), range(3), range(len(origins)), directions, radii, [0.0, 1e-7],
    ):
        curve_origin = origins[origin_index]
        surface_origin = origins[(origin_index + surface_kind) % len(origins)]
        curve_direction, surface_direction = direction_pair
        curve_radius, surface_radius = radius_pair
        first, last = (-0.75, 1.25) if curve_kind == 0 else (0.0, 1.0)
        values = [
            curve_kind, surface_kind, *curve_origin, *curve_direction, curve_radius,
            *surface_origin, *surface_direction, surface_radius, first, last, criteria,
        ]
        rows.append("bean_fast_analytic " + " ".join(map(str, values)))
    return rows
