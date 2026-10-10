"""Native object/entity traversal over real point BVHs and five persistence subsets."""
from __future__ import annotations
import itertools


def append_selector_traversal_requests(requests: list[str]) -> None:
    """Append clipping, containment, persistence, failures and repeat traversals."""
    for kind, camera, persistent, overlap, clipping in itertools.product(range(1, 5), (1, 2), range(7), range(2), range(4)):
        requests.append(f"v3_selector_traversal {kind} {camera} {persistent} 0 0 {clipping} {overlap} 0 -1 6 1 -1")
    for kind, entity_persistent, transform, flags in itertools.product(range(1, 5), range(6), range(2), (0, 1, 2, 4, 8)):
        requests.append(f"v3_selector_traversal {kind} 1 0 {entity_persistent} {transform} 1 1 {flags} -1 3 0 2")
        requests.append(f"v3_selector_traversal {kind} 2 6 {entity_persistent} {transform} 3 1 {flags} -1 6 1 2")
    for kind, failing, flags in itertools.product((1, 2, 3, 4), range(9), (0, 8)):
        requests.append(f"v3_selector_traversal {kind} 1 0 0 0 0 1 {flags} {failing} 3 2 -1")
    for kind, persistence, count in itertools.product((0, 3), range(7), (0, 1, 6)):
        requests.append(f"v3_selector_traversal {kind} 0 {persistence} 0 0 0 1 0 -1 {count} 2 -1")
    for kind, camera, persistence, entity in itertools.product((1, 2, 4), (1, 2), (0, 1, 6), range(6)):
        requests.append(f"v3_selector_traversal {kind} {camera} {persistence} {entity} 0 0 1 16 -1 6 1 -1")
