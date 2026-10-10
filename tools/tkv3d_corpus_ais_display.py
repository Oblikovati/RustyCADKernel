"""AIS lifecycle traces exercise callbacks, selection state and retained failure effects."""

from itertools import product


def append_ais_display_requests(requests: list[str]) -> None:
    """Append independent default/explicit display and erase scenes."""
    _display_failure_requests(requests)
    _display_children_requests(requests)
    _display_highlight_requests(requests)
    _display_trace(
        requests, [(20, 3, 1), (21, 3, 0, 0, 1, 1), (22, 3, 1), (23, 3, 1), (24, 3)]
    )


def _display_failure_requests(requests: list[str]) -> None:
    """Cross first/repeated display with computation failures and active/inactive views."""
    for loaded, active, display, selection, failure in product(
        range(2), range(2), (-2, 0, 2), (-2, -1, 0, 1), (-999, -2, 0, 2, 8, 10, 11)
    ):
        operations = [(9, 0, display), (25, 0, active)]
        if active:
            operations.append((37, 0, 0))
        if loaded:
            operations.append((0, 0, 1))
        operations.extend(
            [
                (28, 0, failure),
                (21, 0, display, selection, active, 2),
                (20, 0, active),
                (28, 0, -999),
                (21, 0, 7, selection, active, 1),
                (20, 0, active),
                (21, 0, display + 1, selection + 1, active, 0),
                (23, 0, active),
                (22, 0, active),
            ]
        )
        _display_trace(requests, operations)


def _display_children_requests(requests: list[str]) -> None:
    """Exercise child contexts, absent own presentations and foreign-context failures."""
    for own, propagation, foreign, automatic in product(
        range(2), range(2), (-1, 0, 1, 2), range(2)
    ):
        operations = [
            (13, 0, 1, propagation),
            (13, 1, 2, propagation),
            (11, 0, own),
            (26, 0, automatic),
        ]
        if foreign != -1:
            operations.append((5, foreign, 1))
        operations.extend(
            [
                (20, 0, 1),
                (29, 1),
                (29, 2),
                (33, 1),
                (22, 0, 1),
                (20, 0, 1),
                (22, 1, 1),
                (21, 0, 2, 1, 1, 1),
            ]
        )
        _display_trace(requests, operations)


def _display_highlight_requests(requests: list[str]) -> None:
    """Check global highlighting, owner flags and activation outside the context's status list."""
    for automatic, custom, highlight, selected in product(range(2), repeat=4):
        operations = [(20, 0), (20, 1), (26, 0, automatic), (27, 0, custom), (8, 0, 2)]
        if selected:
            operations.extend([(29, 0), (29, 1)])
        if highlight:
            operations.append((30, 0))
        operations.extend(
            [
                (33, 0),
                (21, 0, 1, 0, 1, 2),
                (22, 0, 1),
                (20, 0),
                (31, 0),
                (24, 1),
                (23, 1),
            ]
        )
        _display_trace(requests, operations)
    _display_trace(
        requests,
        [(0, 0, 1), (35, 0, 0, 1), (36, 0, 0), (29, 0), (21, 0, 0, 0, 0, 0), (22, 0)],
    )
    _display_trace(
        requests, [(35, 0, 0, 1), (20, 0), (29, 0), (32, 0, 0), (24, 0), (22, 0)]
    )


def _display_trace(requests: list[str], operations: list[tuple[int, ...]]) -> None:
    """Pad each numeric operation to the probe's six fields."""
    fields = [
        value
        for operation in operations
        for value in (*operation, *([0] * (6 - len(operation))))
    ]
    requests.append(f"v3_ais_display {len(operations)} " + " ".join(map(str, fields)))
