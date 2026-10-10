"""Native context removal, bulk display and append-only object queries."""

from itertools import product


def append_ais_lifecycle_requests(requests: list[str]) -> None:
    """Append independent observable lifecycle traces; append_ais_lifecycle_requests(rows)."""
    for operation, update, selected, own, highlighted in product(
        (44, 46, 47, 48), range(2), range(2), range(2), range(2)
    ):
        operations = [
            (37, 0, 0),
            (11, 0, own),
            (20, 0),
            (21, 0, 2, 1, 0, 2),
            (4, 0, 0, 0),
        ]
        if selected:
            operations.append((29, 0))
        if highlighted:
            operations.append((30, 0))
        operations += [
            (
                (operation, 0, update)
                if operation in (44, 46)
                else (operation, 0, 2, update)
            ),
            (20, 0),
            (44, 0),
            (44, 0),
            (20, 0),
        ]
        _lifecycle_trace(requests, operations)
    for operation, index, foreign in product((44, 46, 47, 48), range(4), range(2)):
        if operation == 46 and index == 3:
            continue  # Native ClearGlobal(null) dereferences null in SelectionManager::Remove.
        operations = [(35, 0, 0, 1)]
        if index != 3 and foreign:
            operations.append((5, index, 1))
        _lifecycle_trace(requests, operations + [(operation, index, 0, 1), (20, 0)])
    _lifecycle_trace(
        requests,
        [
            (0, 0),
            (0, 1),
            (61, 0),
            (61, 1),
            (52, 0, 0, -1),
            (53, 0, 0, -1),
            (55, 0, 0, -1, 2),
            (56, 0, 0, 0),
        ],
    )
    _detection_traces(requests)
    _bulk_traces(requests)
    _query_traces(requests)


def _bulk_traces(requests: list[str]) -> None:
    """Exercise shared children, selected owners, legacy flags and repeated removal."""
    for update, child, automatic in product(range(2), range(2), range(2)):
        operations = [(59, 0), (37, 0, 0), (26, 0, automatic)]
        if child:
            operations.append((13, 0, 1, 0))
        operations += [
            (49, 0, 0, 0, 0, 2),
            (49, 1, 0, 0, 0, 2),
            (0, 2),
            (40, 0, update),
            (41, 0, update),
            (29, 0),
            (29, 1),
            (42, 0, update),
            (43, 0, update),
            (41, 0, update),
            (45, 0, update),
            (45, 0, update),
            (41, 0, update),
        ]
        _lifecycle_trace(requests, operations)
    for failed, operation in product((0, 10), (41, 42)):
        # One failing object avoids dependence on address-based hash traversal between processes.
        _lifecycle_trace(
            requests,
            [
                (0, 0),
                (35, 0, 0),
                (29, 0),
                (28, 0, failed),
                (operation, 0, 1),
                (28, 0, -999),
                (operation, 0, 1),
                (45, 0, 1),
            ],
        )
    _lifecycle_trace(
        requests, [(40, 0, 1), (41, 0, 1), (42, 0, 1), (43, 0, 1), (45, 0, 1)]
    )


def _query_traces(requests: list[str]) -> None:
    """Distinguish state None, exact kind None, wildcard signature and affinity filtering."""
    for first, second, kind, signature, status in product(
        range(3), range(3), range(3), (-1, 0, 1), range(3)
    ):
        operations = [(2, 0, first, 0), (2, 1, second, 2), (14, 0, 0)]
        operations += [(query, 0, kind, signature, status) for query in range(50, 57)]
        operations += [
            (57, 0, view, visible, status)
            for view, visible in product(range(2), range(2))
        ]
        _lifecycle_trace(requests, operations)


def _lifecycle_trace(requests: list[str], operations: list[tuple[int, ...]]) -> None:
    """Pad operation arguments to the shared six-field probe protocol."""
    arguments = [
        value
        for operation in operations
        for value in (*operation, *([0] * (6 - len(operation))))
    ]
    requests.append(
        "v3_ais_lifecycle " + " ".join(map(str, [len(operations), *arguments]))
    )


def _detection_traces(requests: list[str]) -> None:
    """Remove duplicate picked ranks while preserving the native cursor adjustment rules."""
    for current, highlighted, removed in product(range(8), range(8), range(3)):
        _lifecycle_trace(
            requests,
            [
                (20, 0),
                (20, 1),
                (20, 2),
                (33, removed),
                (58, 0, current, highlighted),
                (44, removed, 1),
                (44, (removed + 1) % 3),
                (45, 0),
            ],
        )
