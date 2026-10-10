"""Stateful tolerance updates and rank branch thresholds, including f32 normal rounding."""
from __future__ import annotations
from itertools import product
from random import Random


def append_tolerance_requests(requests: list[str]) -> None:
    """Retained zero counts, unknown decrements, negative tolerances and custom resets."""
    actions = ('0 -4', '0 0', '0 2', '0 8', '1 -4', '1 0', '1 2', '1 8', '2 0', '2 3', '2 -5', '3 0')
    requests.append('v3_tolerance 0')
    requests += ['v3_tolerance 4 ' + ' '.join(trace) for trace in product(actions, repeat=4)]
    rng = Random(609)
    for _ in range(128):
        trace = [rng.choice(actions) for _ in range(64)]
        requests.append('v3_tolerance 64 ' + ' '.join(trace))


def criterion_record(normal: str, depth: float, minimum: float, tolerance: float, selection: int, display: int, layer: int, preference: int) -> str:
    """Encode every compared or stored public scalar using native input order."""
    return f'{normal} {depth} {minimum} {tolerance} {selection} {display} {layer} 7 {preference}'


def append_criterion_requests(requests: list[str]) -> None:
    """Near confusion/60-degree boundaries, float underflow and priority inversions."""
    normals = ('0 0 0', '0 0 1', '0 0 -1', '1 0 0', '1 1 1', '1.7320506 0 1',
               '1.7320508 0 1', '1.732051 0 1', '1e-22 0 0', '1e-24 0 0', '1e30 1e30 1e30', '1e40 0 0')
    requests.append('v3_criterion 0')
    for first, second, depth, tolerance, priority in product(normals, normals, (0, 0.999999e-7, 1e-7, 1.000001e-7, 0.5, 2.0000001), (-1, 0, 1, 1.0000001), (0, 1)):
        a = criterion_record(first, depth, 1, tolerance, priority, 1-priority, 0, priority)
        b = criterion_record(second, 0, 2, 1, 1-priority, priority, 0, 1-priority)
        requests.append(f'v3_criterion 1 {a} {b}')
    rng = Random(610)
    for _ in range(2048):
        pair = [criterion_record(rng.choice(normals), rng.uniform(-5, 5), rng.uniform(-1, 5),
                                 rng.uniform(-2, 4), rng.randrange(-3, 4), rng.randrange(-3, 4),
                                 rng.randrange(-1, 2), rng.randrange(2)) for _ in range(2)]
        requests.append('v3_criterion 1 ' + ' '.join(pair))


def append_selection_ranking_requests(requests: list[str]) -> None:
    """Append the independent selector bookkeeping and ranking families."""
    append_tolerance_requests(requests)
    append_criterion_requests(requests)
