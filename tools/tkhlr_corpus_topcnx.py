"""Deterministic requests for TopBas and TopCnx (tools/oracle/tkhlr_probe_topcnx.cxx).

- `tb_interference`: the TopBas_TestInterference record set by its setters, by its
  constructor, and edited through ChangeIntersection and ChangeBoundary, over every
  orientation and boundary signed-zero, huge and negative values (a default record is
  never read: its fields are uninitialized natively);
- `tc_transition`: TopCnx_EdgeFaceTransition before any reset, after the two resets,
  and over seeded sequences of interferences with tangent, normal and opposite boundary
  directions, curvatures, tolerances and every orientation, transition and boundary
  transition.
"""
from __future__ import annotations
import itertools
import random

DIRECTIONS = [(1.0, 0.0, 0.0), (0.0, 1.0, 0.0), (0.0, 0.0, 1.0), (-1.0, 0.0, 0.0),
              (0.0, -1.0, 0.0), (1.0, 1.0, 0.0), (1.0, 1e-9, 0.0), (0.3, -0.4, 0.5)]


def numbers(values: list[float]) -> str:
    """The fields of a request, with every double spelled exactly."""
    return ' '.join(repr(float(v)) for v in values)


def interference_requests() -> list[str]:
    """tb_interference over the three modes."""
    requests: list[str] = []
    values = [(0.0, 0), (-0.0, -3), (1e300, 7), (0.1, 2147483647)]
    for (inter, bound), (o, t, bt) in itertools.product(values, [(0, 1, 2), (3, 2, 1), (1, 1, 3)]):
        for mode in (0, 1):
            requests.append('tb_interference ' + numbers([mode, inter, bound, o, t, bt]))
        requests.append('tb_interference ' + numbers([2, inter, bound, o, t, bt, 2.5, -4]))
    for o in range(4):
        requests.append('tb_interference ' + numbers([1, 0.5, 1, o, o, o]))
    return requests


def interference_command(rng: random.Random) -> list[float]:
    """One AddInterference command with random directions, curvature and orientations."""
    tangent, normal = rng.choice(DIRECTIONS), rng.choice(DIRECTIONS)
    return ([2, rng.choice((1e-7, 0.1)), *tangent, *normal, rng.choice((0.0, 0.5, -0.5, 2.0))]
            + [rng.randrange(4) for _ in range(3)])


def reset_command(rng: random.Random) -> list[float]:
    """Reset(T, N, C) or Reset(T) with random directions."""
    if rng.random() < 0.5:
        return [1, *rng.choice(DIRECTIONS)]
    return [0, *rng.choice(DIRECTIONS), *rng.choice(DIRECTIONS), rng.choice((0.0, 1.0, -2.0))]


def transition_requests(rng: random.Random) -> list[str]:
    """tc_transition: fixed sequences, then seeded random ones."""
    requests = ['tc_transition 0.0',
                'tc_transition ' + numbers([1, 1, 1, 0, 0]),
                'tc_transition ' + numbers([2, 0, 1, 0, 0, 0, 1, 0, 0.5, 2, 1e-7, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1])]
    for o, tr, bt in itertools.product(range(4), repeat=3):
        requests.append('tc_transition ' + numbers([2, 1, 1, 0, 0, 2, 1e-7, 0, 1, 0, 0, 0, 1, 0.0, o, tr, bt]))
    for _ in range(120):
        commands = [reset_command(rng)]
        commands += [interference_command(rng) if rng.random() < 0.85 else reset_command(rng)
                     for _ in range(rng.randint(1, 6))]
        requests.append('tc_transition ' + numbers([len(commands)] + [v for c in commands for v in c]))
    return requests


def append_topcnx_requests(requests: list[str]) -> None:
    """Append the TopBas and TopCnx requests in a fixed order."""
    rng = random.Random(20261004)
    requests.extend(interference_requests())
    requests.extend(transition_requests(rng))
