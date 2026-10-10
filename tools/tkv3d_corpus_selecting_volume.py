"""Manager transitions, clipping retention, native assertions and query delegation."""
from __future__ import annotations
from itertools import product


def append_selecting_volume_requests(requests: list[str]) -> None:
    """Compare every volume transition and copy with seeded pick/inside outputs."""
    for kind, built, next_kind, allow, late, copy in product(range(6), (0, 1), range(-1, 6), (0, 1), (0, 1), range(4)):
        requests.append(f'v3_selecting_volume {kind} {built} {next_kind} {allow} {late} {copy} 0 -1 -1 0 0 5 1')
    for kind, clip, world, depth, point, radius in product((0, 1, 2, 3, 4, 5), range(1, 5), (-1, 4, 5), (-1, 0, 1, 2),
                                                        ('0 0 0', '0 0 5', '5 5 5'), (-1, 0, 1, 6)):
        requests.append(f'v3_selecting_volume {kind} 1 -1 0 1 0 {clip} {world} {depth} {point} {radius}')
