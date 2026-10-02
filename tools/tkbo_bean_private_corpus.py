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
