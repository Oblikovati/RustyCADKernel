"""Public pick overloads on a real headless view, switching query types in place."""
from __future__ import annotations
import itertools


def append_selector_pick_requests(requests: list[str]) -> None:
    """Exercise camera reuse, area containment, clipping, persistence and failures."""
    for camera, persistence, overlap, clipping, tolerance in itertools.product((1, 2), (0, 6), range(2), range(4), (-1, 0, 5)):
        requests.append(f"v3_selector_pick {camera} {persistence} 0 0 {clipping} {overlap} 0 -1 6 {tolerance} 8 1 2 4 3 3 4 2 1")
    for entity, transform, flags in itertools.product(range(6), range(2), (0, 1, 2, 4, 8)):
        requests.append(f"v3_selector_pick 2 6 {entity} {transform} 3 1 {flags} -1 6 3 5 3 1 2 4 1")
    for failing, flags in itertools.product(range(9), (0, 8)):
        requests.append(f"v3_selector_pick 1 0 0 0 0 1 {flags} {failing} 3 -1 4 1 2 4 3")
