"""Real AIS detection: filter policy, callbacks, failures, cursors and public picks."""
from itertools import product


def append_ais_detection_requests(requests: list[str]) -> None:
    """Append deterministic native detection traces; append_ais_detection_requests(rows)."""
    steps = [(0, 7), (1, 1), (3, 0), (4, 0), (5, 1), (6, 1),
             (1, 1), (7, 0), (1, 1), (2, 1), (4, 0), (3, 0), (5, 1)]
    for flags, mask, style, mutation in product(
            (0, 1, 3, 5, 8, 16, 17, 24, 32, 64, 128, 144, 256, 384, 512, 768), range(8), (-1, 0, 2), range(3)):
        requests.append(_detection_request(flags, mask, style, mutation, 0, steps))
    failure_steps = [(0, 7), (1, 1), (8, 0), (1, 1), (5, 1), (6, 1), (2, 1)]
    for flags, mask, failing in product((0, 1, 8, 16), (3, 6, 7), range(1, 7)):
        requests.append(_detection_request(flags, mask, 2, 0, failing, failure_steps))
    public_steps = [(11, 0), (9, 0), (10, 0), (12, 0), (9, 200), (10, 200), (9, 0), (2, 1)]
    for flags, mask in product((0, 1, 5, 8, 16, 32, 256, 384, 512, 768), range(8)):
        requests.append(_detection_request(flags, mask, 2, 0, 0, public_steps))

    style_steps = [(13, -2), (13, -1), (13, 8), (14, 6), (13, -2),
                   (13, -1), (15, 1), (0, 2), (1, 0), (15, 0), (2, 0)]
    for flags, style, mutation in product((0, 32, 128), (-1, 0, 2), range(3)):
        requests.append(_detection_request(flags, 7, style, mutation, 0, style_steps))

    missing_steps = [(0, 7), (1, 0), (0, 0), (5, 1), (6, 1), (2, 0)]
    requests.append(_detection_request(0, 7, -1, 0, 0, missing_steps))


def _detection_request(flags: int, mask: int, style: int, mutation: int,
                       failing: int, steps: list[tuple[int, int]]) -> str:
    """Encode one fixture request with explicit step count and arguments."""
    operations = ' '.join(f'{op} {value}' for op, value in steps)
    return f'v3_ais_detection {flags} {mask} {style} {mutation} {failing} {len(steps)} {operations}'
