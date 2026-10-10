"""Native plane trihedron state, selection snapshots, labels and presentation."""

from itertools import product


def append_plane_trihedron_requests(requests: list[str]) -> None:
    """Append fixed transitions; append_plane_trihedron_requests(rows)."""
    sequences = (
        (0, 1, 2, 3, 4),
        (5, 0, 1, 2, 12),
        (6, 0, 1, 2, 12),
        (7, 0, 1, 2, 12),
        (12, 8, 12, 9, 12, 10, 12),
        (11, 0, 2, 12),
        (12, 5, 12, 6, 12, 7, 12),
    )
    for kind, system, length, ops in product(
        range(3), range(2), (-3, 0, 0.125, 5, 100), sequences
    ):
        requests.append(
            f"v3_plane_trihedron {kind} {system} {length} {len(ops)} "
            + " ".join(map(str, ops))
        )
