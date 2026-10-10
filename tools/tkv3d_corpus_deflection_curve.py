"""Adaptive drawing and matching across analytic, Bezier and C1-split B-spline curves."""

from itertools import product


def append_deflection_curve_requests(requests: list[str]) -> None:
    """Exercise every public overload and both deviation policies; append_deflection_curve_requests(rows)."""
    for kind, variant, draw, arrow, relative in product(
        range(8), range(5), range(2), range(2), range(2)
    ):
        requests.append(
            f"v3_deflection_add {kind} {variant} {draw} {arrow} {relative} .25 .75"
        )
    for kind, variant, relative, x, y, distance in product(
        range(8), range(4), range(2), (-4, 0, 3, 8), (-2, 0, 2), (0, 0.25, 1)
    ):
        requests.append(
            f"v3_deflection_match {kind} {variant} {relative} {x} {y} 0 {distance} .25 .75"
        )
    for first, last in [(1, 0), (0, 0), (-1e100, 1e100)]:
        for variant in (1, 4):
            requests.append(f"v3_deflection_add 0 {variant} 1 1 0 {first} {last}")
