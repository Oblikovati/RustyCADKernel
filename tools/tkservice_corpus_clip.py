"""Deterministic requests for Graphic3d_ClipPlane (tools/oracle/tkservice_probe_clip.cxx).

`sv_clip` builds planes through every constructor and setter (including null normals that
raise), chains of planes, and compares the equations, the reversed equation, the gp_Pln frame,
the chain lengths and the point and box tests of the chain union; `sv_clip_capping` applies
sequences of capping operations (color, material, hatch, flags, texture, clone) and compares
the capping state and the modification counters after each.
"""
from __future__ import annotations
import random

EQUATIONS = [[0, 0, 1, 0], [1, 0, 0, -2], [0, -1, 0, 3.5], [1, 1, 1, -1], [0.3, -0.7, 0.2, 5],
             [0, 0, 0, 1], [-2, 0, 0, 4], [1e-3, 0, 0, 0], [0, 0, -1, -10]]
POINTS = [[0, 0, 0, 1], [1, 2, 3, 1], [-5, 0, 0, 1], [2, 0, 0, 1], [0, 3.5, 0, 1], [10, -10, 10, 1],
          [0.5, 0.5, 0, 1], [1, 2, 3, 0]]
BOXES = [[-1, -1, -1, 1, 1, 1], [2, 2, 2, 3, 3, 3], [-10, -10, -10, -5, -5, -5], [0, 0, 0, 0, 0, 0],
         [1.5, -4, -4, 2.5, 4, 4], [3, 3, 3, -3, -3, -3]]


def numbers(values: list[float]) -> str:
    """The fields of a request, with every double spelled exactly."""
    return ' '.join(repr(float(v)) for v in values)


def clip_request(rng: random.Random) -> list[float]:
    """A random plane kind and equation, a chain, points and boxes."""
    chain = [rng.choice(EQUATIONS) for _ in range(rng.randrange(4))]
    points = [rng.choice(POINTS) for _ in range(rng.randrange(1, 5))]
    boxes = [rng.choice(BOXES) for _ in range(rng.randrange(1, 4))]
    values = [rng.randrange(5), *rng.choice(EQUATIONS), len(chain)] + [v for e in chain for v in e]
    values += [len(points)] + [v for p in points for v in p]
    values += [len(boxes)] + [v for b in boxes for v in b]
    return values


def capping_op(rng: random.Random) -> list[float]:
    """One capping operation with its arguments."""
    op = rng.randrange(1, 11)
    args = {1: lambda: [rng.choice([0.0, 0.25, 1.0, 0.5]) for _ in range(3)] if rng.random() < 0.9 else [1.5, 0, 0],
            2: lambda: [rng.randrange(25)], 3: lambda: [rng.randrange(13)],
            6: lambda: [rng.randrange(3), rng.randrange(2)], 7: lambda: [rng.randrange(2)],
            9: lambda: [rng.randrange(2)], 10: lambda: rng.choice(EQUATIONS)}
    return [op] + args.get(op, lambda: [])()


def append_clip_requests(requests: list[str]) -> None:
    """Append the default plane, then seeded planes, chains and capping sequences."""
    requests.append('sv_clip ' + numbers([4, 0, 0, 1, 0, 0, 1, *POINTS[1], 1, *BOXES[0]]))
    rng = random.Random(20261007)
    for _ in range(80):
        requests.append('sv_clip ' + numbers(clip_request(rng)))
    for _ in range(40):
        ops = [capping_op(rng) for _ in range(rng.randrange(1, 9))]
        requests.append('sv_clip_capping ' + numbers([len(ops)] + [v for op in ops for v in op]))
