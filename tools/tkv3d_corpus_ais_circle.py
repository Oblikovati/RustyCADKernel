"""Circle and arc constructors, setter transitions, sensitivity and appearance."""

from itertools import product
from math import tau


def append_ais_circle_requests(requests: list[str]) -> None:
    """Append native circle lifecycles; append_ais_circle_requests(rows)."""
    spans = (
        (0, 0, tau),
        (1, 0, tau),
        (1, tau, 0),
        (1, 0.25, 2),
        (1, 2, 0.25),
        (1, 0, 0),
    )
    sequences = (
        (8, 9, 10, 11, 9),
        (0, 8, 2, 8, 3, 8, 1, 8),
        (4, 9, 5, 8, 9),
        (6, 8, 7, 9, 8, 12, 8),
        (2, 3, 0, 8),
        (0, 2, 1, 8, 3),
    )
    for (variant, first, last), filled, link, width, ops in product(
        spans, range(2), range(3), (-2, 0, 2.75), sequences
    ):
        requests.append(
            f"v3_ais_circle {variant} {filled} {link} {first} {last} {width} {len(ops)} "
            + " ".join(map(str, ops))
        )
