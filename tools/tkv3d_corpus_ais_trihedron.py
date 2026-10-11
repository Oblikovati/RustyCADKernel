"""Exact trihedron geometry, cached selection and native ownership transitions."""
from itertools import product


def append_trihedron_requests(requests: list[str]) -> None:
    """Append independent native transitions; append_trihedron_requests(rows)."""
    sequences = (
        (20, 21, 22, 23, 24, 16, 20, 21, 22, 23, 24),
        (0, 16, 20, 21, 22, 23, 1, 16),
        (0, 2, 1, 16, 20, 21, 22, 23),
        (4, 5, 6, 7, 3, 16),
        (8, 9, 10, 8, 16, 20, 21, 22, 23, 11, 16),
        (0, 16, 14, 20, 21, 22, 23, 16, 20, 21, 22, 23),
        (12, 16, 13, 16, 10, 16, 12, 16),
        (15, 16, 22, 10, 16, 22),
    )
    for kind, size, operations in product(range(2), (0.125, 5, 100), sequences):
        requests.append(f"v3_ais_trihedron {kind} {len(operations)} {size} " + " ".join(map(str, operations)))
