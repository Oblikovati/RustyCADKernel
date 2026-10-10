"""Defined native axis constructor, appearance, snapshot selection and presentation transitions."""

from itertools import product


def append_ais_axis_requests(requests: list[str]) -> None:
    """Append deterministic axis sequences; append_ais_axis_requests(rows)."""
    sequences = (
        (10, 11, 13),
        (0, 2, 13, 1, 13, 3, 13),
        (10, 12, 10, 4, 10, 13),
        (6, 10, 13, 7, 10, 13, 8, 10, 13),
        (15, 13, 1, 13, 0, 13),
        (14, 13, 9, 13, 5, 10, 13),
        (16, 10, 13),
        (6, 17, 10, 13),
    )
    for kind, system, width, ops in product(
        range(6), range(2), (-2, 0, 1, 2.5, 3.00000001), sequences
    ):
        requests.append(
            f"v3_ais_axis {kind} {system} 4 {width} {len(ops)} "
            + " ".join(map(str, ops))
        )
    for length, system in product((-2, 0, -1, 0.001, 100), range(2)):
        requests.append(f"v3_ais_axis 5 {system} {length} 1 2 10 13")
    for styled, label, length, angle in product(
        range(2), range(2), (-3, 0, 1, 4, 100), (-1, 0, 0.1, 0.3, 1.5, 2)
    ):
        requests.append(f"v3_xyz_axis {styled} {label} {length} {angle}")
