"""Normal and derivative comparisons, including distinct cone-apex branches."""
from itertools import product
from math import sin


def append_contap_surface_requests(requests: list[str]) -> None:
    """Exercise elementary and generic surfaces; append_contap_surface_requests(lines)."""
    frames = ['0 0 0 0 0 1 1 0 0', '1 -2 3 1 2 3 3 -1 1']
    for kind, frame, reverse, radius, u, v in product(
            range(6), frames, range(2), [0, 1e-9, 2], [-2, -0.0, 0.3, 2], [-10, -0.0, 0.5, 10]):
        requests.append(f'ct_surface {kind} {frame} {reverse} {radius} 0.4 -20 20 -20 20 {u} {v}')
    for radius, semi, delta, lower in product([0, 2], [-0.4,0.4], [0,1e-13,-1e-13,1e-11], [-20,0]):
        apex = -radius / sin(semi)
        requests.append(f'ct_surface 3 {frames[0]} 0 {radius} {semi} -20 20 {lower} 20 0.3 {apex+delta}')
