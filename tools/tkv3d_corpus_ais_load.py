"""Deterministic native AIS load/status traces, including partial foreign-context failures."""

from itertools import product


def append_ais_load_requests(requests: list[str]) -> None:
    """Exercise defaults, reloads, children and affinity; append_ais_load_requests(rows)."""
    for default, own, accepted, highlight, selection in product(
        (-2, 0, 4), (-1, 2), (-999, 2), (-1, 7), (-2, -1, 0, 4)
    ):
        operations = [
            (9, 0, default, 0),
            (7, 0, own, 0),
            (10, 0, accepted, 0),
            (8, 0, highlight, 0),
            (12, 0, 3, 0),
            (6, 0, 0, 0),
            (14, 0, 0, 0),
            (0, 0, selection, 0),
            (4, 0, 0, 0),
            (3, 0, 0, 0),
            (1, 0, 8, 1),
            (1, 0, 7, 0),
        ]
        _load_trace(requests, operations)
    for own, propagation, foreign, state, mode in product(
        range(2), range(2), (-1, 0, 1, 2), range(3), (-2, 0, 4)
    ):
        operations = [(13, 0, 1, propagation), (13, 1, 2, propagation), (11, 0, own, 0)]
        if foreign != -1:
            operations.append((5, foreign, 1, 0))
        operations.extend(
            [
                (0, 0, mode, 0),
                (2, 0, state, mode),
                (4, 1, 1, 0),
                (0, 0, mode + 1, 0),
                (2, 0, 2, -2),
            ]
        )
        _load_trace(requests, operations)
    _load_trace(
        requests,
        [
            (0, 3, 0, 0),
            (1, 3, -1, 1),
            (6, 3, 0, 0),
            (4, 3, 0, 0),
            (4, 0, 1, 0),
            (6, 0, 0, 0),
        ],
    )


def _load_trace(
    requests: list[str], operations: list[tuple[int, int, int, int]]
) -> None:
    """Encode one fresh scene as a numeric probe request."""
    fields = " ".join(str(value) for operation in operations for value in operation)
    requests.append(f"v3_ais_load {len(operations)} {fields}")
