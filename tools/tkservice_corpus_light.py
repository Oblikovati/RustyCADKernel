"""Deterministic requests for Graphic3d_CLight and Graphic3d_LightSet
(tools/oracle/tkservice_probe_light.cxx).

`sv_light` applies sequences of setters (in and out of range, for every light type, so the
type and range checks raise) and CopyFrom between types, and compares the packed parameters,
color, direction, position, revision and flags after each; `sv_light_set` fills sets with
lights of every type and state and compares the counters, ambient color, keys, the filtered
iterations, a removal (the last light takes its index) and the revision after a light change.
"""
from __future__ import annotations
import random


def numbers(values: list[float]) -> str:
    """The fields of a request, with every double spelled exactly."""
    return ' '.join(repr(float(v)) for v in values)


def light_op(rng: random.Random) -> list[float]:
    """One light operation with its arguments."""
    op = rng.randrange(1, 16)
    if op == 1:
        return [op] + [rng.choice([0.0, 0.25, 0.5, 1.0, 0.3]) for _ in range(3)]
    if op in (2, 3, 4):
        return [op, rng.randrange(2)]
    if op in (5, 6):
        return [op] + [rng.choice([0.0, 1.0, -2.5, 1e-18, 100.0]) for _ in range(3)]
    if op == 7:
        return [op] + rng.choice([[0, 0, -1], [1, 1, 0], [0, 0, 0], [0.3, -0.4, 0.5], [0, 1e-8, 0]])
    if op == 10:
        return [op, rng.choice([1.0, 0.0, -1.0, 0.5]), rng.choice([0.0, 0.25, -0.5, 2.0])]
    if op == 15:
        return [op, rng.randrange(4)]
    values = {8: [2.0, 0.0, -1.0, 20.0, 1e-9], 9: [0.5, 0.0, 3.2, 1.5, 3.14159],
              11: [0.0, 0.5, 1.0, 1.5, -0.1], 12: [0.0, 0.5, -1.0, 10.0],
              13: [0.0, 0.5, 1.5707964, 1.6, -0.2], 14: [0.0, 100.0, -1.0, 1e-9]}
    return [op, rng.choice(values[op])]


def set_light(rng: random.Random) -> list[float]:
    """One light of a set request: type, enabled, shadows, intensity, color."""
    return [rng.randrange(4), rng.choice([1, 1, 0]), rng.randrange(2), rng.choice([1.0, 2.5, 0.5]),
            *[rng.choice([0.0, 0.25, 1.0]) for _ in range(3)]]


def append_light_requests(requests: list[str]) -> None:
    """Append each light type alone, then seeded operation sequences and light sets."""
    for kind in range(4):
        requests.append('sv_light ' + numbers([kind, 0]))
    rng = random.Random(20261008)
    for _ in range(80):
        ops = [light_op(rng) for _ in range(rng.randrange(1, 8))]
        requests.append('sv_light ' + numbers([rng.randrange(4), len(ops)] + [v for op in ops for v in op]))
    requests.append('sv_light_set ' + numbers([0, 0, 0]))
    for _ in range(40):
        count = rng.randrange(1, 7)
        lights = [v for _ in range(count) for v in set_light(rng)]
        requests.append('sv_light_set ' + numbers([count] + lights + [rng.randrange(count + 1), rng.randrange(count + 1)]))
