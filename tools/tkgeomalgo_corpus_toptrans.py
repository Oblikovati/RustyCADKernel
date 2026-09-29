"""Deterministic requests for the TopTrans package (tools/oracle/tkgeomalgo_probe_toptrans.cxx).

`tt_curve` and `tt_surface` run a command sequence on one fresh transition and print the
states after each command; `tt_states` prints the static state tables. Directions are
drawn from discrete fans (multiples of 45 and 30 degrees, near-tolerance tilts) so that
equal angles and tangent faces, which drive the tie branches, occur often. A surface
request uses either only the plane Compare or only the curved one: the plane overload
leaves the sector curvature unwritten, and natively that storage is uninitialized
until the curved overload writes it, so mixing them could read indeterminate memory.
"""
from __future__ import annotations
import math
import random

FORWARD, REVERSED, INTERNAL, EXTERNAL = range(4)
Vector = tuple[float, float, float]


def numbers(values: list[float]) -> str:
    """The fields of a request, with every double spelled exactly."""
    return ' '.join(repr(float(v)) for v in values)


def fan(angle: float, tilt: float = 0.0) -> Vector:
    """A direction in the plane normal to +X at `angle`, tilted by `tilt` toward +X."""
    return (tilt, math.cos(angle), math.sin(angle))


def pick_direction(rng: random.Random) -> Vector:
    """A fan direction: a multiple of 45 or 30 degrees, sometimes tilted slightly."""
    step = rng.choice([math.pi / 4, math.pi / 6])
    tilt = rng.choice([0.0, 0.0, 0.0, 1e-9, 1e-3, 0.5])
    return fan(step * rng.randrange(12), tilt)


def curve_commands(rng: random.Random, count: int) -> list[float]:
    """Random Reset and Compare commands of a curve transition."""
    values: list[float] = [count]
    tangents: list[Vector] = [(1.0, 0.0, 0.0), (-1.0, 0.0, 0.0), (1.0, 1.0, 0.0), (0.0, 1.0, 0.0),
                              (1.0, -1.0, 0.0), (1.0, 1e-9, 0.0), (0.0, 0.0, 1.0)]
    curvatures = [0.0, 0.0, 0.5, 1.0, 2.0, -1.0]
    for _ in range(count):
        command = rng.choice([0, 1, 2, 2, 2, 2])
        values.append(command)
        if command == 0:
            values += [*rng.choice(tangents), *pick_direction(rng), rng.choice(curvatures)]
        elif command == 1:
            values += [*rng.choice(tangents)]
        else:
            values += [rng.choice([0.0, 1e-7, 0.1]), *rng.choice(tangents), *pick_direction(rng),
                       rng.choice(curvatures), rng.randrange(4), rng.randrange(4)]
    return values


def fixed_curve_requests() -> list[list[float]]:
    """Hand-made sequences: no reset, a line, a curve, INTERNAL and tie cases."""
    x, y, z = (1.0, 0.0, 0.0), (0.0, 1.0, 0.0), (0.0, 0.0, 1.0)
    d = (1.0, 1.0, 0.0)
    return [
        [1, 2, 1e-7, *y, *z, 0.0, FORWARD, FORWARD],
        [3, 1, *x, 2, 1e-7, *y, *z, 0.0, FORWARD, FORWARD, 2, 1e-7, *d, *z, 0.0, REVERSED, FORWARD],
        [2, 1, *x, 2, 1e-7, -1.0, 0.0, 0.0, *z, 0.0, INTERNAL, FORWARD],
        [4, 0, *x, *y, 1.0, 2, 1e-7, *y, *x, 0.5, FORWARD, INTERNAL, 2, 1e-7, *y, *x, 2.0, REVERSED, INTERNAL,
         2, 1e-7, *y, *z, 0.0, EXTERNAL, REVERSED],
        [4, 1, *x, 2, 0.0, *y, *z, 1.0, FORWARD, FORWARD, 2, 0.0, *y, *z, 2.0, REVERSED, FORWARD,
         2, 0.0, *y, *z, 0.5, INTERNAL, REVERSED],
    ]


def surface_commands(rng: random.Random, count: int, curved: bool) -> list[float]:
    """Random Reset and Compare commands of a surface transition (see the module note)."""
    values: list[float] = [count]
    tgt = (1.0, 0.0, 0.0)
    principal = [(1.0, 0.0, 0.0), (0.0, 1.0, 0.0), (1.0, 1.0, 0.0), (1.0, 1e-9, 0.0)]
    curvatures = [0.0, 1e-13, 0.5, 1.0, 2.0, -1.0]
    for _ in range(count):
        command = rng.choice([0, 1] + [3 if curved else 2] * 5)
        values.append(command)
        if command == 0:
            values += [*tgt, *pick_direction(rng)]
        elif command == 1:
            values += [*tgt, *pick_direction(rng), *rng.choice(principal), *rng.choice(principal),
                       rng.choice(curvatures), rng.choice(curvatures)]
        else:
            values += [rng.choice([0.0, -1.0, 1e-7, 1e-3, 1.0]), *pick_direction(rng)]
            if command == 3:
                values += [*rng.choice(principal), *rng.choice(principal),
                           rng.choice(curvatures), rng.choice(curvatures)]
            values += [rng.randrange(4), rng.randrange(4)]
    return values


def fixed_surface_requests() -> list[list[float]]:
    """The upstream GTest scenarios, parallel frames (raised errors) and tie cases."""
    x, y, z = (1.0, 0.0, 0.0), (0.0, 1.0, 0.0), (0.0, 0.0, 1.0)
    return [
        [2, 0, *x, *z, 2, 1.5e-12, *z, FORWARD, FORWARD],
        [1, 1, *x, *z, *x, *x, 1.0, 1.0],
        [2, 0, *x, *z, 3, 1.0, *z, 0.6, 0.8, 0.0, 0.6, -0.8, 0.0, 1.0, 0.5, FORWARD, FORWARD],
        [3, 0, *x, *x, 0, *x, *z, 2, 1e-7, *x, FORWARD, FORWARD],
        [3, 0, *x, *z, 2, 1e-7, *z, FORWARD, FORWARD, 2, 1e-7, *z, FORWARD, FORWARD],
        [3, 0, *x, *z, 2, 1e-7, *z, FORWARD, FORWARD, 2, 1e-7, *z, REVERSED, FORWARD],
        [2, 1, *x, *z, *y, *x, 2.0, 1.0, 3, 1e-7, *z, *y, *x, 1.0, 1.0, FORWARD, INTERNAL],
        [3, 1, *x, *z, *x, *y, 1e-13, 0.0, 2, 0.0, *y, INTERNAL, INTERNAL, 2, 0.0, *z, EXTERNAL, REVERSED],
    ]


def curved_tie_requests() -> list[list[float]]:
    """Two curved faces at the same angle on a curved reference: normal to the
    before-after direction (null cosine), where the curvature and the side (sine sign)
    decide, and at 45 degrees, where the curvature alone decides."""
    x, y, z = (1.0, 0.0, 0.0), (0.0, 1.0, 0.0), (0.0, 0.0, 1.0)
    requests: list[list[float]] = []
    for reference in (1.0, 1.5):
        for normal in ((0.0, 1.0, 0.0), (0.0, -1.0, 0.0), fan(math.pi / 4), fan(-3 * math.pi / 4)):
            for first, second in ((2.0, 0.5), (0.5, 2.0), (0.5, 0.50001), (1.5, 3.0)):
                for side in (FORWARD, REVERSED):
                    requests.append([3, 1, *x, *z, *y, *x, reference, 0.0,
                                     3, 1e-7, *normal, *y, *x, first, 0.0, FORWARD, FORWARD,
                                     3, 1e-7, *normal, *y, *x, second, 0.0, side, FORWARD])
    return requests


def append_toptrans_requests(requests: list[str]) -> None:
    """Append the TopTrans requests: the state tables, the fixed then the random sequences."""
    for orientation in range(4):
        requests.append(f'tt_states {orientation}')
    for values in fixed_curve_requests():
        requests.append('tt_curve ' + numbers(values))
    for values in fixed_surface_requests() + curved_tie_requests():
        requests.append('tt_surface ' + numbers(values))
    rng = random.Random(1997_03_04)
    for index in range(300):
        requests.append('tt_curve ' + numbers(curve_commands(rng, 1 + index % 8)))
    for index in range(600):
        requests.append('tt_surface ' + numbers(surface_commands(rng, 1 + index % 10, index % 2 == 1)))
