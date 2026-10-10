"""Camera animation endpoint/lifecycle combinations; append_camera_requests(rows)."""

from itertools import product


def append_camera_requests(requests: list[str]) -> None:
    """Cover native endpoint guards, view modes and timestamps; append_camera_requests([])."""
    for flags, duration, start, pts in product(
        [0, 1, 2, 4, 7, 8, 16, 32, 48, 64, 96],
        [-1.0, 0.0, 2.0],
        [-1.0, 0.0, 1.0],
        [-2.0, 0.0, 0.5, 1.0, 2.0, 5.0],
    ):
        requests.append(f"v3_animation_camera {flags} {duration} {start} {pts}")
