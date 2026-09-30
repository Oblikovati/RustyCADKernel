"""State transitions for Contap point and line records, preserving native inputs."""
from itertools import product


def append_contap_requests(requests: list[str]) -> None:
    """Exercise all flag/transition pairs and stable insertion; append_contap_requests(lines)."""
    requests.extend(
        f'ct_point {mode} -0.0 2 -3 -4 5 -6 {mask} {null} {t1} {t2} -0.0'
        for mode, mask, null, t1, t2 in product(range(2), range(16), range(2), range(4), range(4)))
    sequences = [[], [0], [3, 2, 1], [1, 2, 3], [1, 1, 1], [0, -0.0, -2, 3, -2, 1]]
    requests.extend(
        f'ct_line {mode} {transition} {len(parameters)} '+ ' '.join(map(str, parameters))
        for mode, transition, parameters in product(range(4), range(4), sequences))
