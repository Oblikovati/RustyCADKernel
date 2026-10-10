"""Native line state, geometry, units and appearance sequences, including failures."""

from itertools import product


def append_ais_line_requests(requests: list[str]) -> None:
    """Append independent line lifecycles; append_ais_line_requests(rows)."""
    sequences = (
        (8, 9, 10),
        (0, 8, 2, 8, 1, 8, 3, 8),
        (2, 3, 0, 8),
        (0, 2, 3, 8, 1),
        (8, 5, 8, 6, 8, 9, 4, 8, 9),
        (9, 7, 9, 7, 9),
        (8, 11, 8, 9),
        (2, 0, 1, 3),
        (5, 6, 9, 8),
        (4, 5, 4, 9),
    )
    for segment, system, link, width, ops in product(
        range(2), range(2), range(3), (-2, 0, 1, 2.5, 3.00000001), sequences
    ):
        # Native UnsetColor dereferences a null line after UnsetWidth leaves stale own_width.
        if link == 2 and ops == (2, 0, 1, 3) and width < 0:
            continue
        requests.append(
            f"v3_ais_line {segment} {system} {link} {width} {len(ops)} "
            + " ".join(map(str, ops))
        )
