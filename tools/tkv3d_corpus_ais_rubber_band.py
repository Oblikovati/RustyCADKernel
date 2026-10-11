"""Martensite native rubber-band appearance, degeneracies and geometry-cache transitions."""
from itertools import product


def append_rubber_band_requests(requests: list[str]) -> None:
    """Append independent native scenarios; append_rubber_band_requests(rows)."""
    sequences = (
        (0, 16, 1, 16, 5, 16, 6, 16, 17),
        (0, 8, 15, 16, 1, 16, 7, 16, 8, 16),
        (3, 2, 3, 4, 15, 16),
        (0, 3, 15, 16, 3, 15, 16, 3, 15, 16, 3, 15, 16),
        (0, 9, 10, 11, 12, 13, 14, 16),
        (0, 2, 15, 16, 4, 15, 16),
        (18, 8, 15, 16, 19, 16, 1, 16, 4, 15),
    )
    for ctor, width, closed, transparency, value, operations in product(
        range(3), (0, 1, 2.5), (0, 1), (0, 0.25, 1, 2), (-1, 0, 0.125, 1, 2), sequences
    ):
        requests.append(f"v3_ais_rubber_band {ctor} {width} {closed} {transparency} {len(operations)} {value} " + " ".join(map(str, operations)))
