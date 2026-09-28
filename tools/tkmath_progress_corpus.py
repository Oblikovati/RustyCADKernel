"""Deterministic LU_Decompose / math_Gauss requests run under a cancelling progress indicator.

Each request starts a user indicator that reports a break once UserBreak() has been
polled more than `allowed_polls` times (-1: never). The corpus sweeps matrix sizes
1..6, every break point from before the first column to after the last, both
LU_Decompose overloads, and singular inputs whose early exits must still close the
progress scope (zero rows, a zero pivot mid-way, a pivot threshold above every pivot).
"""
import random


def append_progress_requests(requests: list[str]) -> None:
    """Append the progress requests with their own seed; append_progress_requests(requests)."""
    rng = random.Random(20201104)
    for n in range(1, 7):
        for allowed in range(-1, n + 2):
            for with_scales in (0, 1):
                cells = [rng.uniform(-5, 5) for _ in range(n * n)]
                requests.append(line('progress_lu', [n, allowed, with_scales, 1e-20, *cells]))
            cells = [rng.uniform(-5, 5) for _ in range(n * n)]
            requests.append(line('progress_gauss', [n, allowed, 1e-20, *cells]))
    append_singular_requests(requests)


def append_singular_requests(requests: list[str]) -> None:
    """Early singular exits under breaking and non-breaking indicators."""
    zero_row = [1, 2, 3, 0, 0, 0, 4, 5, 6]
    mid_pivot = [1, 2, 3, 2, 4, 6, 1, 0, 1]
    for allowed in (-1, 0, 1, 2):
        for with_scales in (0, 1):
            requests.append(line('progress_lu', [3, allowed, with_scales, 1e-20, *zero_row]))
            requests.append(line('progress_lu', [3, allowed, with_scales, 1e-20, *mid_pivot]))
            requests.append(line('progress_lu', [2, allowed, with_scales, 10.0, 4, 1, 1, 3]))
        requests.append(line('progress_gauss', [3, allowed, 1e-20, *mid_pivot]))
        requests.append(line('progress_gauss', [2, allowed, 10.0, 4, 1, 1, 3]))


def line(op: str, values: list) -> str:
    """Format one request with round-trip float text; line('progress_lu', [1, -1])."""
    return op + ' ' + ' '.join(str(value) for value in values)
