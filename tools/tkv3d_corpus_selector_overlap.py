"""Overlap accumulation with client owner-slot changes and real AIS presentation priority."""
from __future__ import annotations
import itertools
import random


def append_selector_overlap_requests(requests: list[str]) -> None:
    """Append deterministic independent selector scenes; append_selector_overlap_requests(rows)."""
    generator = random.Random(428)
    for kind, persistent, presentation, mode, volume in itertools.product(range(3), range(4), range(2), range(2), range(4)):
        steps = [(0, 1, -1, -1, 1, 2, 0, 5, 1), (1, 1, -1, -1, 1, 2, 0, 1, 2),
                 (0, 1, -1, -1, 1, 2, 0, 9, 0), (0, 0, 2, -1, 1, 2, 0, 2, 1),
                 (1, 1, 2, 3, 1, 2, 1, 0, 1), (0, 1, 0, -1, 2, 2, 0, 1, 1)]
        for _ in range(10):
            steps.append((generator.randrange(2), generator.randrange(4), generator.randrange(-1, 4),
                          generator.randrange(-1, 4), generator.randrange(3), generator.randrange(3),
                          generator.randrange(5), generator.choice((-2, 0, 1, 1, 5, 9)), generator.randrange(3)))
        payload = " ".join(str(value) for step in steps for value in step)
        requests.append(f"v3_selector_overlap {kind} {persistent} {presentation} {mode} {volume} {len(steps)} {payload}")
