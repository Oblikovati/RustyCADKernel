"""Deterministic TopLoc_Location differential requests for tools/verify_tkmath_numeric.py.

Usage: append_toploc_requests(requests) after the Poly requests.
Each request builds up to four datums (rotation about a random axis, uniform scale and
translation) and a chain of powered datums, with repeated datums so that adjacent powers
cancel, zero powers and identity chains. The probe compares cached transformations of
the chain, its inverse, powers, quotients and predivisions, equality, the NextLocation
tail, the DumpJson text and the HashCode recurrence (over synthetic datum ids, and
against the native value for the real datum addresses). A zero axis or a null scale
checks the datum construction failures.
"""
from __future__ import annotations
import random


def line(op: str, values: list) -> str:
    """Format one request with round-trip float text; line('toploc', [0, 0, 0])."""
    return op+' '+' '.join(str(value) for value in values)


def datum(rng: random.Random, case: int) -> list:
    """Point, axis, angle, scale and translation of one datum; datum(rng, 3)."""
    point = [rng.uniform(-2, 2) for _ in range(3)]
    axis = [rng.uniform(-1, 1) for _ in range(3)]
    if case % 41 == 0:
        axis = [0.0, 0.0, 0.0]
    angle = rng.choice([0.0, rng.uniform(-3.2, 3.2), 3.141592653589793])
    scale = rng.choice([1.0, 1.0, 1.0, -1.0, 2.0, rng.uniform(0.1, 3)])
    if case % 53 == 0:
        scale = 0.0
    translation = [rng.choice([0.0, rng.uniform(-10, 10)]) for _ in range(3)]
    return point + axis + [angle, scale] + translation


def append_toploc_requests(requests: list[str]) -> None:
    """Append 360 location requests; append_toploc_requests(requests)."""
    rng = random.Random(19910121)
    for case in range(360):
        count = rng.randint(1, 4)
        values = [count]
        for _ in range(count):
            values += datum(rng, case)
        items = rng.randint(0, 6)
        values.append(items)
        previous = None
        for _ in range(items):
            identifier = previous if previous is not None and rng.random() < 0.4 else rng.randrange(count)
            values += [identifier, rng.choice([1, 1, -1, 2, -2, 0, 3])]
            previous = identifier
        values.append(rng.choice([0, 1, 2, 3, -1, -2, -3]))
        requests.append(line('toploc', values))
