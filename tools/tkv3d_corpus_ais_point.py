"""Point datum appearance, live geometry, selection, highlight and matching contracts."""

from itertools import product


def append_ais_point_requests(requests: list[str]) -> None:
    """Append native point sequences; append_ais_point_requests(rows)."""
    sequences = (
        (8, 9, 10, 11, 12, 13, 14),
        (0, 8, 4, 8, 1, 8, 5, 8),
        (2, 4, 8, 0, 8, 3, 5, 8),
        (4, 2, 0, 1, 3, 5),
        (8, 6, 14, 8, 7, 11, 12, 13, 14, 8),
        (4, 5, 0, 1, 9),
    )
    for link, width, kind, ops in product(
        range(3), (-2, 0, 1, 2.5, 3.00000001), range(-1, 14), sequences
    ):
        requests.append(
            f"v3_ais_point {link} {width} {kind} {len(ops)} " + " ".join(map(str, ops))
        )
    for xyz, distance in product(
        ((0, 0, 0), (1, 2, 3), (-1e100, 2e100, -3e100)), (-1, 0, 1, 2, 5, 1e300)
    ):
        requests.append(
            "v3_standard_point " + " ".join(map(str, (*xyz, *xyz, distance)))
        )
    for distance in (0, 1.9999999999999998, 2, 2.0000000000000004):
        requests.append(f"v3_standard_point 1 2 3 1 2 5 {distance}")
