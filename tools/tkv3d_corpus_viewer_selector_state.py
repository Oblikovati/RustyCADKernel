"""Selector state traces keep every std::sort comparison within strict ordering domains."""
from __future__ import annotations
import random


def append_viewer_selector_state_requests(requests: list[str]) -> None:
    """Exercise lifecycle, lazy ranks, tolerances, protected seeds and worker configuration."""
    def append(operations: list[tuple[int, int, int, int]]) -> None:
        requests.append('v3_viewer_selector_state '+str(len(operations))+' '+
                        ' '.join(str(value) for operation in operations for value in operation))
    append([])
    append([(11, 0, 0, 0), (19, 0, 0, 0), (14, 0, 0, 0), (19, 1, 0, 0), (14, 0, 1, 0)])
    append([(3, 0, 0, 0), (5, 0, 1, 1), (5, 0, 9, 9), (8, 1, 0, 0), (2, 0, 0, 0),
            (8, 1, 0, 0), (6, 0, 0, 0), (8, 1, 0, 0), (4, 0, 0, 0), (7, 0, 0, 0)])
    rng = random.Random(932)
    for _ in range(160):
        operations: list[tuple[int, int, int, int]] = []
        for _ in range(75):
            op = rng.randrange(29)
            a, b, c = rng.randrange(3), rng.randrange(2), rng.randrange(2)
            if op in (0, 26, 27): a = rng.choice([-3, 0, 2, 4, 8])
            if op == 1: b = rng.randrange(-5, 6)
            if op == 5: b, c = rng.randrange(-20, 21), rng.randrange(-5, 6)
            if op in (8, 9): a = rng.randrange(-2, 12)
            if op == 16: b = rng.randrange(-2, 2)
            if op == 19: a = 1  # Null-builder rebuild is undefined upstream; covered by ordered trace above.
            if op == 20: a, b = rng.randrange(2), rng.randrange(-2, 4)
            if op == 28: a = rng.choice([-1, 0, 1, 2])
            if op == 24: b = rng.randrange(-2, 2)
            operations.append((op, a, b, c))
        append(operations)
    for size in (1, 2, 15, 16, 17, 32, 64):
        operations = [(5, i % 3, (i * 7 + 3) % 5, 0) for i in range(size)]
        operations += [(6, 0, 0, 0)] + [(8, i + 1, 0, 0) for i in range(size)]
        append(operations)
