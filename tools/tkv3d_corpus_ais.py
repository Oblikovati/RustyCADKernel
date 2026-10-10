"""Deterministic AIS selection callbacks, cursor mutation and context status."""
from __future__ import annotations
import random


def append_ais_requests(requests: list[str]) -> None:
    """Compare native selected-owner traces, including exceptions and duplicate picks."""
    def selection(operations: list[tuple[int, int, int, int]]) -> None:
        requests.append(f'v3_ais_selection {len(operations)} '+
                        ' '.join(str(value) for operation in operations for value in operation))
    selection([])
    selection([(0,0,0,0),(0,1,0,0),(5,0,0,0),(8,0,0,1),(1,0,1,1),(1,1,1,1)])
    selection([(0,0,0,0),(10,0,0,0),(1,0,1,1),(2,0,0,0)])
    selection([(0,0,0,0),(0,1,0,0),(11,0,5,0)])
    selection([(6,3,0,0),(1,0,1,1),(1,1,1,0),(6,4,0,0),(1,0,1,1)])
    rng = random.Random(639843)
    for _ in range(200):
        operations: list[tuple[int, int, int, int]] = []
        for _ in range(60):
            op = rng.choice([0,1,2,3,4,5,6,7,8,9,11])
            a, b, c = rng.randrange(4), rng.randrange(-1,6), rng.randrange(2)
            if op in (0,1,3,4,6,11): a = rng.randrange(5)
            if op == 11: c = rng.randrange(5)
            operations.append((op,a,b,c))
        selection(operations)
    for _ in range(80):
        operations = [(rng.randrange(8), rng.randrange(-3,5)) for _ in range(60)]
        operations = [(op, value % 3 if op == 3 else value) for op,value in operations]
        requests.append(f'v3_ais_status {len(operations)} '+' '.join(str(v) for op in operations for v in op))

    for kind in range(4):
        for disabled in range(2):
            for modes in ([], [0], [1], [3], [0,3], [1,3], [3,0], [0,1,3], [1,0,3]):
                requests.append(f"v3_ais_filters {kind} {disabled} {len(modes)} "+" ".join(map(str,modes)))
