"""Plane presentation callbacks and infinite-plane distance matching at pinned OCCT."""

from itertools import product


def append_plane_presentation_requests(requests: list[str]) -> None:
    """Include every visibility combination; append_plane_presentation_requests(rows)."""
    for kind, flags, lengths in product(
        range(3), range(16), ((4, 6, 2), (3.7, 2.4, 0.3), (1, 1, 10), (-4, -6, 2))
    ):
        x, y, spacing = lengths
        requests.append(f"v3_plane_add {kind} 0 {flags} {x} {y} {spacing}")
    for variant in (1, 2):
        requests.append(f"v3_plane_add 0 {variant} 0 1 1 0.5")
    for kind, z, distance in product(range(3), (-1, 0, 1), (-1, 0, 1, 2)):
        requests.append(f"v3_plane_match {kind} 1000 0 {z} {distance}")

    for kind, flags in product(range(2), (17, 20, 31)):
        requests.append(f"v3_plane_add {kind} 0 {flags} 4 6 2")
