"""Exact frustum projection and clipping-range requests, including native output states."""
from __future__ import annotations
from itertools import product


def append_selection_helper_requests(requests: list[str]) -> None:
    """Exercise cameras, viewport invalidation, range order and plane chains."""
    for present, perspective, zero, viewport in product((0, 1), repeat=4):
        for x, y, z in ((0, 0, 0), (400, 300, 1), (-10, 900, .5), (123, 231, .2)):
            requests.append(f'v3_frustum_builder {present} {perspective} {zero} 800 600 '
                            f'{viewport} .1 .2 .9 .8 {x} {y} {z} 2 -3 4 5 -6 7')
    append_clip_requests(requests)


def append_clip_requests(requests: list[str]) -> None:
    """Cross finite/void bounds with ordered sub-ranges and independent/linked planes."""
    clips = ('0', '1 0 5', '3 0 2 4 6 1 5', '3 0 2 1 5 4 6', '2 -3 0 3 7')
    planes = ('0', '1 0 0 1 -2 1 0', '1 0 0 -1 5 1 0',
              '2 0 0 1 -2 1 0 0 0 -1 5 1 1',
              '2 0 0 1 -2 0 0 0 0 -1 5 1 1',
              '1 1 0 0 -2 1 0', '1 1 0 0 2 1 0')
    queries = '5 -8 -4 -1 1 1 2 4 8 3 2'
    for bounds, clipped, plane, ray in product(('0 -10 10', '1 0 0', '0 0 6'), clips, planes,
                                             ('0 0 0 0 0 1', '0 0 3 0 0 -1')):
        requests.append(f'v3_clip_range {bounds} {clipped} {plane} {ray} {queries}')
