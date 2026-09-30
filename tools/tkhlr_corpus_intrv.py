"""Deterministic requests for Intrv (tools/oracle/tkhlr_probe_intrv.cxx).

An interval is `mode s ts e te` (0 default, 1 bounds only, 2 bounds and tolerances raised
to Epsilon, 3 bounds and tolerances as given). The families:
- `iv_interval`: the constructors on zero, signed zero, subnormal, huge and RealFirst /
  RealLast bounds, and tolerances that are negative, zero, tiny or overflow a float;
- `iv_edit`: SetStart, FuseAtStart, CutAtStart, SetEnd, FuseAtEnd, CutAtEnd, including
  the infinite bounds they keep;
- `iv_relate`: Position and the 13 tests on a grid of intervals around a reference
  interval (every position, empty and reversed intervals, zero and wide tolerances) and
  seeded random pairs;
- `iv_set`: the eight boolean operations on sets built by Unite, fixed cases (empty set
  or tool, touching, nested, split, infinite intervals) and seeded random sets;
- `iv_value`: Value inside and outside 1..NbIntervals (Standard_OutOfRange).
"""
from __future__ import annotations
import random

REAL_LAST = 1.7976931348623157e308
Interval = tuple[int, float, float, float, float]


def numbers(values: list[float]) -> str:
    """The fields of a request, with every double spelled exactly."""
    return ' '.join(repr(float(v)) for v in values)


def fields(interval: Interval) -> list[float]:
    """The five fields of one interval."""
    return [float(v) for v in interval]


def set_fields(intervals: list[Interval]) -> list[float]:
    """`n interval...` of one set."""
    values: list[float] = [float(len(intervals))]
    for interval in intervals:
        values += fields(interval)
    return values


def interval_requests() -> list[str]:
    """iv_interval over the constructors and boundary bounds and tolerances."""
    bounds = [(0.0, 1.0), (-0.0, 0.0), (5e-324, 1e-310), (1e300, REAL_LAST),
              (-REAL_LAST, REAL_LAST), (3.0, -2.0), (0.1, 0.30000000000000004)]
    tolerances = [(0.0, 0.0), (-1.0, 0.5), (1e-45, 1e-30), (1e39, 1e40), (0.25, 3.0)]
    requests = ['iv_interval ' + numbers(fields((0, 0.0, 0.0, 0.0, 0.0)))]
    for s, e in bounds:
        for ts, te in tolerances:
            for mode in (1, 2, 3):
                requests.append('iv_interval ' + numbers(fields((mode, s, ts, e, te))))
    return requests


def edit_requests() -> list[str]:
    """iv_edit: each bound edit on finite, default and zero-tolerance intervals."""
    intervals: list[Interval] = [(2, 0.0, 0.1, 10.0, 0.1), (0, 0.0, 0.0, 0.0, 0.0),
                                 (3, 0.0, 0.0, 10.0, 0.0), (3, -REAL_LAST, 0.5, REAL_LAST, 0.5),
                                 (3, 1.0, -0.25, 2.0, 0.75)]
    edits = [(-0.5, 0.2), (0.05, 0.01), (0.0, 0.1), (11.0, 3.0), (9.95, 0.0), (1e300, 1e-3)]
    requests: list[str] = []
    for interval in intervals:
        for op in range(6):
            for value, tol in edits:
                requests.append('iv_edit ' + numbers(fields(interval) + [op, value, tol]))
    return requests


def relate_requests(rng: random.Random) -> list[str]:
    """iv_relate on a grid around the reference [0, 10] +- 0.5 and random pairs."""
    reference: Interval = (2, 0.0, 0.5, 10.0, 0.5)
    ends = [-3.0, -0.5, 0.0, 0.2, 0.5, 5.0, 9.5, 10.0, 10.4, 13.0]
    requests: list[str] = []
    for tol in (0.0, 0.5):
        for s in ends:
            for e in ends:
                a: Interval = (3, s, tol, e, tol)
                requests.append('iv_relate ' + numbers(fields(a) + fields(reference)))
    requests.append('iv_relate ' + numbers(fields((0, 0, 0, 0, 0)) + fields(reference)))
    requests.append('iv_relate ' + numbers(fields(reference) + fields((0, 0, 0, 0, 0))))
    for _ in range(80):
        pair = [random_interval(rng) for _ in range(2)]
        requests.append('iv_relate ' + numbers(fields(pair[0]) + fields(pair[1])))
    return requests


def random_interval(rng: random.Random) -> Interval:
    """A sorted interval of [-10, 10] with small tolerances, in mode 2 or 3."""
    s, e = sorted(rng.uniform(-10.0, 10.0) for _ in range(2))
    return (rng.choice((2, 3)), s, rng.choice((0.0, 0.01, 0.3)), e, rng.choice((0.0, 0.01, 0.3)))


def fixed_sets() -> list[tuple[list[Interval], list[Interval]]]:
    """Sets and tools for the boundary cases of the boolean operations."""
    a: Interval = (2, 0.0, 0.1, 10.0, 0.1)
    return [([], []), ([], [a]), ([a], []), ([a], [a]),
            ([a], [(2, 2.0, 0.1, 3.0, 0.1)]), ([a], [(2, -5.0, 0.1, 0.0, 0.1)]),
            ([a], [(2, 10.0, 0.1, 15.0, 0.1)]), ([a], [(2, -5.0, 0.1, 5.0, 0.1)]),
            ([a], [(2, 5.0, 0.1, 15.0, 0.1)]), ([a], [(2, -5.0, 0.1, 15.0, 0.1)]),
            ([a], [(2, 0.05, 0.1, 5.0, 0.1)]), ([a], [(2, 5.0, 0.1, 9.95, 0.1)]),
            ([a, (2, 12.0, 0.1, 14.0, 0.1)], [(2, 9.0, 0.1, 13.0, 0.1)]),
            ([(0, 0, 0, 0, 0)], [a]), ([a], [(0, 0, 0, 0, 0)]),
            ([(2, 0.0, 0.0, 1.0, 0.0), (2, 2.0, 0.0, 3.0, 0.0), (2, 4.0, 0.0, 5.0, 0.0)],
             [(2, 1.0, 0.0, 2.0, 0.0), (2, 3.0, 0.0, 4.0, 0.0)]),
            ([(3, 5.0, 0.0, 1.0, 0.0), (2, 2.0, 0.0, 3.0, 0.0)], [(3, 4.0, 0.0, 0.0, 0.0)])]


def set_requests(rng: random.Random) -> list[str]:
    """iv_set: every operation on the fixed sets, then on random sets."""
    pairs = fixed_sets()
    for _ in range(40):
        pairs.append(([random_interval(rng) for _ in range(rng.randint(1, 4))],
                      [random_interval(rng) for _ in range(rng.randint(1, 3))]))
    return ['iv_set ' + numbers([op] + set_fields(left) + set_fields(right))
            for left, right in pairs for op in range(8)]


def value_requests() -> list[str]:
    """iv_value inside and outside the sequence bounds."""
    two = [(2, 0.0, 0.0, 1.0, 0.0), (2, 2.0, 0.0, 3.0, 0.0)]
    return ['iv_value ' + numbers(set_fields(intervals) + [index])
            for intervals in ([], two) for index in (-1, 0, 1, 2, 3)]


def append_intrv_requests(requests: list[str]) -> None:
    """Append every Intrv request family in a fixed order."""
    rng = random.Random(20261003)
    requests.extend(interval_requests())
    requests.extend(edit_requests())
    requests.extend(relate_requests(rng))
    requests.extend(set_requests(rng))
    requests.extend(value_requests())
