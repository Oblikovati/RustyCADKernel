"""Stateful selection orchestration, including hierarchy and callback failures."""
from __future__ import annotations
import random


def append_selection_manager_requests(requests: list[str]) -> None:
    """Append native manager traces; every random stream starts with known selection states."""
    def append(hierarchy: int, operations: list[tuple[int, int, int, int]]) -> None:
        requests.append(f'v3_selection_manager {hierarchy} {len(operations)} '+
                        ' '.join(str(value) for operation in operations for value in operation))
    directed = [(0,0,0,0),(2,0,0,0),(3,0,-1,0),(7,0,-1,0),(1,0,0,0),(0,0,-1,0),(2,0,0,0)]
    for hierarchy in range(3):
        append(hierarchy, [])
        append(hierarchy, directed)
        append(hierarchy, [(2,2,2,0),(4,2,2,0)])
        append(hierarchy, [(15,1,0,0),(0,0,0,0),(15,1,-999,0),(2,0,0,0),(14,1,0,1),(3,0,-1,0),(2,0,0,0)])
    rng = random.Random(736211)
    for trace in range(240):
        operations: list[tuple[int, int, int, int]] = [(13,0,0,trace % 2)]
        # OCCT crashes after Activate(unloaded), Remove, Activate: Remove skips
        # selector cleanup outside myGlobal, leaving cleared sensitive wrappers.
        # Preserve a standalone unloaded-activation trace above; lifecycle traces
        # establish membership before activation, including descendant objects.
        for _ in range(80):
            op, a, b, c = rng.randrange(21), rng.randrange(3), rng.randrange(-1,3), rng.randrange(2)
            if op == 13: op = 12
            if op == 2:
                operations.extend((0,index,-1,0) for index in (2,1,0))
            if op in (9,10): c = rng.randrange(3)
            if op == 11: c = rng.choice([-1,0,1,2,4,8])
            if op == 15: b = rng.choice([-999,0,1,2])
            if op == 17: c = rng.randrange(5)
            if op == 18: c = rng.randrange(-1,2)
            operations.append((op,a,b,c))
        append(trace % 3, operations)

    for grouped in range(3):
        for counts in ([], [799999], [800000], [800001], [0,799999,800000,800001], [800000,800000,800000]):
            for fail_at in range(-1, len(counts)):
                requests.append(f'v3_prebuild_threshold {grouped} {len(counts)} {fail_at} '+' '.join(map(str, counts)))
