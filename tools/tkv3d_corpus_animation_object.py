"""Native object animation and protected-base cases; append_object_requests(rows)."""

from itertools import product

FLAGS = [0, 1, 2, 4, 8, 16, 24, 32, 40, 48, 88, 120, 152, 216]


def append_object_requests(requests: list[str]) -> None:
    """Cover exact transforms, selection and Z-fit policy; append_object_requests([])."""
    for flags, kind, duration, start, pts in product(
        FLAGS, range(4), [-1.0, 0.0, 2.0], [0.0, 1.0], [-1.0, 0.0, 0.5, 1.0, 2.0, 3.0]
    ):
        requests.append(f"v3_animation_object {flags} {kind} {duration} {start} {pts}")
    for flags, kind in product([flag for flag in FLAGS if flag & 2 == 0], range(4)):
        requests.append(f"v3_base_animation_object {flags} {kind}")
    for flags, angles, duration, pts in product(
        FLAGS, [(0.0, 3.14159), (-1.0, 0.5), (2.0, 2.0)], [0.0, 2.0], [-1.0, 0.0, 0.7, 2.0, 3.0]
    ):
        requests.append(f"v3_animation_axis_rotation {flags} {angles[0]} {angles[1]} {duration} {pts}")
