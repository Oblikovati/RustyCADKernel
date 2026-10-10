"""Actual selection debug geometry, retained structures and callback mutations."""
from __future__ import annotations
import itertools


def append_sensitive_display_requests(requests: list[str]) -> None:
    """Compare native graphics callbacks across geometry and display overloads."""
    for kind, transform, flags, repeat, fail in itertools.product(range(15), range(4), range(4), (1, 2), (-1, 1, 2, 3)):
        requests.append(f"v3_sensitive_prs {kind} {transform} {flags} {repeat} {fail}")
    for kind, transform, flags, mode, fail in itertools.product((0, 7, 10, 12, 13), range(4), (0, 1, 2, 3, 4, 7), range(4), (-1, 1, 2, 3)):
        requests.append(f"v3_sensitive_display {kind} {transform} {flags} {mode} 3 {fail}")
    append_sensitive_display_callback_requests(requests)


def append_sensitive_display_callback_requests(requests: list[str]) -> None:
    """Flipping callbacks append entities and selections without keeping caller locks."""
    for kind, transform, fail in itertools.product((0, 7, 12), range(4), (-1, 1, 2, 3)):
        requests.append(f"v3_sensitive_prs {kind} {transform} 9 2 {fail}")
        for flags, mode in itertools.product((9, 17, 25), (0, 1, 3)):
            requests.append(f"v3_sensitive_display {kind} {transform} {flags} {mode} 3 {fail}")
