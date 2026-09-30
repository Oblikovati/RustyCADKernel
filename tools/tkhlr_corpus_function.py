"""Contour values, gradients and retained/cached state."""
from itertools import product


def append_function_requests(requests: list[str]) -> None:
    """Exercise all function modes using elementary and polynomial surfaces."""
    frames = ['0 0 0 0 0 1 1 0 0', '1 -2 3 1 2 3 3 -1 1']
    for kind, frame, radius, uv, tolerance, angle in product(
            range(6), frames, [0, 2], ['0 0', '0.3 0.5', '-2 -3'],
            [0, 1e-7, 2], [0, 0.3]):
        requests.append(f'ct_function {kind} {frame} 0 {radius} 0.4 -4 4 -4 4 {uv} {tolerance} {angle} 1 2 3 4 -2 3')
