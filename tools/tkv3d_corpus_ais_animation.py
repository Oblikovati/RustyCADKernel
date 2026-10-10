"""Deterministic AIS timeline and frozen timer inputs; append_animation_requests(rows)."""

from itertools import product


def append_animation_requests(requests: list[str]) -> None:
    """Exercise offsets, duration caches and boundary frames; append_animation_requests([])."""
    for start, duration, child_start, child_duration in product(
        [-2.0, 0.0, 1.0], [0.0, 1.0, 4.0], [-1.0, 0.0, 2.0], [0.0, 0.5, 3.0]
    ):
        for frames in [(-3.0, 0.0, 10.0), (0.25, 0.5, 1.0), (2.0, 4.0, 8.0)]:
            numbers = (start, duration, child_start, child_duration, *frames)
            requests.append("v3_ais_animation " + " ".join(map(str, numbers)))
    for duration, pts, speed in product(
        [0.0, 1.0, 5.0], [-2.0, 0.0, 0.5, 1.0, 8.0], [-2.0, 0.0, 1.0, 3.0]
    ):
        requests.append(f"v3_ais_animation_timer {duration} {pts} {speed}")

    for first, second in product([-1.0, 0.0, 2.0, 8.0], repeat=2):
        requests.append(f"v3_ais_animation_edits {first} {second}")
