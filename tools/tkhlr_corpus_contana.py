"""Analytic contour overloads, degeneracies, indirect axes and reused results."""
from itertools import product


def append_contana_requests(requests: list[str]) -> None:
    """Append elementary contour inputs; append_contana_requests(lines)."""
    frames = ['0 0 0 0 0 1 1 0 0', '1 -2 3 1 2 3 3 -1 1']
    views = ['1 0 0', '0 0 1', '1 2 3', '-4 7 -2', '0 0 0', '1e-16 0 1']
    for mode, warm, frame, reverse, radius, view in product(
            range(9), range(2), frames, range(2), [0, 1e-9, 2, -1], views):
        angles = [-2.0, -0.3, 0, 0.3, 2.0] if mode % 3 == 1 else [0]
        for angle in angles:
            requests.append(f'ct_analytic {mode} {warm} {frame} {reverse} {radius} 0.4 {view} {angle}')
    for semi, view, angle in product([-0.4,1e-6,1.5], views, [-0.5,0,0.5]):
        requests.append(f'ct_analytic 7 1 {frames[0]} 0 2 {semi} {view} {angle}')
