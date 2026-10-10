"""Native context style graph and inline setter traces with null/shared handles."""
from __future__ import annotations
import random


def append_ais_context_requests(requests: list[str]) -> None:
    """Append independent constructor and setter traces; append_ais_context_requests(rows)."""
    requests.append("v3_ais_context 0")
    for slot in range(6):
        for value in range(8):
            requests.append(f"v3_ais_context 1 1 {slot} {value}")
    randomizer = random.Random(319)
    for _ in range(96):
        operations = []
        for _ in range(24):
            op = randomizer.randrange(9)
            limit = 8 if op < 4 else 2
            operations.extend((op, randomizer.randrange(6), randomizer.randrange(limit)))
        requests.append("v3_ais_context 24 " + " ".join(map(str, operations)))


def _object_trace(requests: list[str], active: int, operations: list[tuple[int, int, int]]) -> None:
    """Encode one independently constructed scene; _object_trace(rows, 0, [(0, 1, 0)])."""
    fields = " ".join(str(value) for operation in operations for value in operation)
    requests.append(f"v3_ais_object {active} {len(operations)} {fields}")


def append_ais_object_requests(requests: list[str]) -> None:
    """Append update/failure combinations and raw context-pointer transitions."""
    import itertools
    for active, registered, status, op, all_modes, failure in itertools.product(
        range(2), range(2), range(3), (5, 6, 7, 8, 16), range(2), (-999, 0, 11)
    ):
        operations = [(0, 1, 0), (2, status, 0), (3, 0, 1), (3, 1, 0), (4, 1, 1)]
        if registered:
            operations.extend([(1, 1, 0), (1, 2, 0)])
        operations.extend([(9, failure, 0), (op, 1, all_modes), (9, -999, 0), (op, 1, all_modes)])
        _object_trace(requests, active, operations)
    for active in range(2):
        _object_trace(requests, active, [(5, 1, 0), (11, 1, 0), (0, 1, 0), (12, 0, 0),
                      (0, 1, 0), (0, 0, 0), (13, 1, 0), (0, 1, 0), (11, 0, 0),
                      (17, 1, 0), (17, 0, 0), (18, 0, 0)])

    for kind, depth, groups in itertools.product(range(6), range(3), range(3)):
        operations = [(20, kind, 0), (19, depth, 0), (0, 1, 0), (11, 1, 0),
                      (19, depth, 0), (3, 0, 0), (10, 0, 0)]
        if groups != 1:
            operations.append((21, groups, 0))
        operations.extend([(20, kind, 0), (20, 0, 0), (20, 5, 0)])
        _object_trace(requests, 0, operations)
    _object_trace(requests, 1, [(17, 1, 0), (22, 0, 0), (22, 1, 0), (17, 0, 0)])

    for registered, selected, custom in itertools.product(range(2), repeat=3):
        requests.append(f"v3_ais_context_shutdown {registered} {selected} {custom}")
