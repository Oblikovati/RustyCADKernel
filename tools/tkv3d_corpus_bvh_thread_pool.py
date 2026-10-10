"""Completed-round pool traces, comparing counts and messages without scheduler ordering."""
from __future__ import annotations


def append_bvh_thread_pool_requests(requests: list[str]) -> None:
    """Cover clamped counts, lazy startup, duplicate builds, failures and restarts."""
    for workers in (-3, 0, 1, 2, 4):
        for count in (1, 3, 7):
            for rounds in (0, 1, 3):
                for seed in range(4):
                    requests.append(f'v3_bvh_thread_pool {workers} {count} {rounds} {seed}')
