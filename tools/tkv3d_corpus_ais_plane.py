"""AIS_Plane constructor, cache, selection, presentation and ownership transitions."""

from itertools import product


def append_ais_plane_requests(requests: list[str]) -> None:
    """Exercise every plane category and state operation; append_ais_plane_requests(rows)."""
    sequences = (
        (0, 1, 3, 19, 17, 20, 18, 21),
        (7, 4, 6, 8, 5, 7, 8, 6, 6, 8),
        (9, 10, 9, 11, 12, 9, 0, 11),
        (5, 19, 0, 20, 4, 19, 1, 21),
        (12, 0, 1, 13, 0, 14, 19, 0, 20, 15, 1, 16, 0, 22, 3, 23),
    )
    for ctor, kind, linked, sequence in product(
        range(7), range(3), range(2), sequences
    ):
        # A detached native drawer has null aspects after UnsetSize/UnsetColor.
        if linked == 0:
            sequence = tuple(op for op in sequence if op not in (6, 8))
        operations = " ".join(map(str, sequence))
        requests.append(
            f"v3_ais_plane {ctor} {kind} {linked} {len(sequence)} {operations}"
        )
