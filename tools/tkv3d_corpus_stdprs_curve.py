"""All curve overloads, endpoint metrics, infinite bounds and knot-scaled sampling."""

from itertools import product
from math import cos, sin, sqrt


def append_stdprs_curve_requests(requests: list[str]) -> None:
    """Append deterministic native curve cases; append_stdprs_curve_requests(rows)."""
    for kind, variant, draw, arrow, count in product(
        range(8), range(4), range(2), range(2), (1, 4, 9)
    ):
        if arrow and kind >= 5:
            continue
        requests.append(
            f"v3_stdprs_add {kind} {variant} {draw} {arrow} {count} .25 .75"
        )
    for kind, variant in product((0, 5, 6, 7), (1, 3)):
        requests.append(f"v3_stdprs_add {kind} {variant} 0 0 4 -1e100 1e100")
    append_curve_matching(requests)


def append_curve_matching(requests: list[str]) -> None:
    """Exercise automatic and explicit ranges; append_curve_matching(rows)."""
    for kind, variant, count, x, y, distance in product(
        range(8),
        range(4),
        (2, 4, 9),
        (-4, -3, 0, 3, 6, 9, 12),
        (-2, 0, 2),
        (0, 0.25, 1),
    ):
        requests.append(
            f"v3_stdprs_match {kind} {variant} {count} {x} {y} 0 {distance} .25 .75"
        )
    for variant in range(4):
        angle = 2 * sqrt(0.8)
        requests.append(
            f"v3_stdprs_match 1 {variant} 4 {3*cos(angle)} {3*sin(angle)} 0 1e-10 0 .05"
        )
    for kind in (0, 5, 6, 7):
        requests.append(f"v3_stdprs_match {kind} 2 4 8 -2 0 .01 -1e100 1e100")
