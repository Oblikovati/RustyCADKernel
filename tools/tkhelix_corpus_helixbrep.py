"""Deterministic requests for HelixBRep (tools/oracle/tkhelix_probe_helixbrep.cxx).

`hxb_build P N X indirect nsteps (step)...` runs SetParameters (`0 mode data`, every
overload), SetApproxParameters (`1 tol deg cont`) and Perform (`2`) steps on one
HelixBRep_BuilderHelix and compares the state after each step: the statuses, the
reached tolerance and the wire edge by edge (curves pole by pole, vertices). The
families cover the upstream suite parameters, several parts with smoothed joints,
every error status (10 to 13, 1 without parameters), wrong array dimensions, arrays
with lower bounds other than 1 (absolute indices read other elements or raise), no
part at all, numbers of turns of 0, approximation settings, direct and indirect
frames, and repeated runs.
"""
from __future__ import annotations
import math
import random

TWO_PI = 2.0 * math.pi

STANDARD = [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0]
FRAMES = [STANDARD, [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1],
          [10.0, -5.0, 3.0, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0], [1.0, 2.0, 3.0, 0.3, -0.4, 0.5, 0.7, 0.1, -0.2, 1]]
PERFORM = [2.0]


def numbers(values: list[float]) -> str:
    """The fields of a request, with every double spelled exactly."""
    return ' '.join(repr(float(v)) for v in values)


def arr(values: list[float], lo: int = 1) -> list[float]:
    """An array field `lo n (v)...`."""
    return [lo, len(values)] + list(values)


def approx(tol: float, deg: int = 8, cont: int = 2) -> list[float]:
    """A SetApproxParameters step (cont is a GeomAbs_Shape code, 2 = C1)."""
    return [1, tol, deg, cont]


def general(diams, heights, pitches, is_pitches) -> list[float]:
    """SetParameters(axis, diams, heights, pitches, flags); arrays as `arr` fields."""
    return [0, 0] + diams + heights + pitches + is_pitches


def one_diam(diam, heights, pitches, is_pitches) -> list[float]:
    """SetParameters(axis, diam, heights, pitches, flags)."""
    return [0, 1, diam] + heights + pitches + is_pitches


def spiral(d1, d2, heights, pitches, is_pitches) -> list[float]:
    """SetParameters(axis, diam1, diam2, heights, pitches, flags)."""
    return [0, 2, d1, d2] + heights + pitches + is_pitches


def turns(diams, pitches, nb_turns) -> list[float]:
    """SetParameters(axis, diams, pitches, turns)."""
    return [0, 3] + diams + pitches + nb_turns


def diam_turns(diam, pitches, nb_turns) -> list[float]:
    """SetParameters(axis, diam, pitches, turns)."""
    return [0, 4, diam] + pitches + nb_turns


def spiral_turns(d1, d2, pitches, nb_turns) -> list[float]:
    """SetParameters(axis, diam1, diam2, pitches, turns)."""
    return [0, 5, d1, d2] + pitches + nb_turns


def build_request(frame: list[float], steps: list[list[float]]) -> str:
    """One `hxb_build` request line."""
    return 'hxb_build ' + numbers(frame + [len(steps)] + [v for s in steps for v in s])


def suite_parameters() -> list[list[float]]:
    """The SetParameters steps of the upstream suites."""
    return [
        one_diam(100.0, arr([100.0]), arr([20.0]), arr([1])),
        spiral(100.0, 20.0, arr([100.0]), arr([10.0]), arr([1])),
        general(arr([100.0, 80.0, 60.0, 40.0]), arr([30.0, 40.0, 30.0]), arr([10.0, 15.0, 10.0]), arr([1, 1, 1])),
        diam_turns(100.0, arr([20.0]), arr([5.0])),
        one_diam(100.0, arr([100.0]), arr([5.0]), arr([0])),
        general(arr([100.0, 100.0]), arr([100.0]), arr([20.0]), arr([1])),
        spiral(100.0, 20.0, arr([20.0, 60.0, 20.0]), arr([2.0, 6.0, 2.0]), arr([0, 0, 0])),
        diam_turns(100.0, arr([10.0, 10.0, 10.0]), arr([2.0, 6.0, 2.0])),
        general(arr([150.0, 150.0, 150.0, 123.0, 123.0, 123.0]),
                arr([0.75 * 13.0, 2.1 * 64.0, 2.25 * 50.0, 2.5 * 45.0, 0.75 * 13.0]),
                arr([0.75, 2.1, 2.25, 2.5, 0.75]), arr([0, 0, 0, 0, 0])),
        general(arr([80.0, 60.0, 40.0]), arr([40.0, 40.0]), arr([8.0, 12.0]), arr([1, 1])),
        one_diam(50.0, arr([50.0]), arr([10.0]), arr([1])),
    ]


def more_parameters() -> list[list[float]]:
    """Further overloads, lower bounds and error statuses."""
    return [
        turns(arr([60.0, 40.0, 60.0]), arr([5.0, 8.0]), arr([1.5, 2.25])),
        spiral_turns(40.0, 90.0, arr([6.0, 9.0]), arr([2.0, 1.0])),
        turns(arr([60.0, 40.0], 0), arr([5.0], 3), arr([2.0], -2)),
        general(arr([100.0, 80.0, 60.0], 0), arr([30.0, 40.0]), arr([10.0, 15.0]), arr([1, 1])),
        general(arr([100.0, 80.0, 60.0]), arr([30.0, 40.0], 2), arr([10.0, 15.0]), arr([1, 1])),
        general(arr([100.0, 80.0]), arr([30.0]), arr([10.0], 0), arr([1], 5)),
        spiral(100.0, 20.0, arr([20.0, 60.0, 20.0], 0), arr([4.0, 6.0, 5.0]), arr([1, 1, 1])),
        spiral(100.0, 20.0, arr([]), arr([]), arr([])),
        general(arr([100.0]), arr([]), arr([]), arr([])),
        general(arr([100.0, 80.0, 60.0]), arr([30.0]), arr([10.0]), arr([1])),
        diam_turns(100.0, arr([10.0, 10.0]), arr([5.0])),
        spiral_turns(100.0, 20.0, arr([10.0]), arr([5.0, 1.0])),
        turns(arr([100.0, 20.0]), arr([10.0, 2.0]), arr([5.0])),
        one_diam(100.0, arr([1.0e-6]), arr([10.0]), arr([1])),
        one_diam(100.0, arr([100.0]), arr([0.0]), arr([1])),
        one_diam(0.0, arr([100.0]), arr([10.0]), arr([1])),
        general(arr([10.0, 1.0e9]), arr([1.0]), arr([0.5]), arr([1])),
        one_diam(100.0, arr([100.0]), arr([-4.0]), arr([0])),
        one_diam(100.0, arr([100.0]), arr([0.0]), arr([0])),
        one_diam(-80.0, arr([30.0]), arr([7.0]), arr([1])),
        general(arr([100.0, 0.0]), arr([40.0]), arr([10.0]), arr([1])),
        spiral(30.0, 90.0, arr([15.0, 5.0]), arr([3.0, 2.5]), arr([1, 0])),
    ]


def settings_requests() -> list[str]:
    """Approximation settings, frames and repeated runs on one parameter set."""
    requests = []
    two_parts = general(arr([80.0, 60.0, 40.0]), arr([40.0, 40.0]), arr([8.0, 12.0]), arr([1, 1]))
    for tol, deg, cont in [(1e-6, 8, 4), (1e-3, 3, 0), (1e-4, 14, 2), (1e-4, 0, 2), (1e-4, 8, 5),
                           (1e-2, 5, 4), (1e-9, 8, 2), (0.3, 2, 0)]:
        requests.append(build_request(STANDARD, [two_parts, approx(tol, deg, cont), PERFORM]))
    # Coarse settings give coils of fewer than 4 poles, where MovePointAndTangent fails
    # at the joints until the degree is raised (the tolerance stays below the heights
    # and pitches, which BuildPart compares with it).
    short_parts = general(arr([80.0, 60.0, 40.0]), arr([9.0, 10.0]), arr([8.0, 12.0]), arr([1, 1]))
    for tol, deg, cont in [(7.0, 2, 0), (7.0, 1, 0), (3.0, 1, 0), (5.0, 2, 2)]:
        requests.append(build_request(STANDARD, [short_parts, approx(tol, deg, cont), PERFORM]))
        requests.append(build_request(FRAMES[1], [two_parts, approx(tol, deg, cont), PERFORM]))
    # A last coil of one radian: a degree 2 span of 3 poles within the tolerance, which
    # MovePointAndTangent refuses until the degree is raised once.
    rest_parts = general(arr([80.0, 80.0, 60.0]), arr([8.0 * (1.0 + 1.0 / TWO_PI), 12.0 * (1.0 + 1.0 / TWO_PI)]),
                         arr([8.0, 12.0]), arr([1, 1]))
    for tol in [0.2, 0.5, 1.0, 2.0]:
        requests.append(build_request(STANDARD, [rest_parts, approx(tol, 2, 0), PERFORM]))
    for frame in FRAMES[1:]:
        requests.append(build_request(frame, [two_parts, PERFORM, PERFORM]))
    requests.append(build_request(STANDARD, [PERFORM]))
    requests.append(build_request(STANDARD, [two_parts, PERFORM, more_parameters()[9], PERFORM]))
    requests.append(build_request(FRAMES[1], [suite_parameters()[0], approx(1e-5), PERFORM,
                                              suite_parameters()[1], PERFORM]))
    return requests


def random_requests(count: int) -> list[str]:
    """Seeded composite helices of up to 3 parts in random frames."""
    rng = random.Random(20261003)
    requests = []
    for _ in range(count):
        parts = rng.randint(1, 3)
        diams = [rng.uniform(5.0, 120.0) for _ in range(parts + 1)]
        heights = [rng.uniform(2.0, 40.0) for _ in range(parts)]
        flags = [rng.randint(0, 1) for _ in range(parts)]
        pitches = [rng.uniform(2.0, 15.0) if f else rng.uniform(0.3, 4.0) for f in flags]
        frame = FRAMES[rng.randint(0, len(FRAMES) - 1)]
        steps = [general(arr(diams), arr(heights), arr(pitches), arr(flags)),
                 approx(10.0 ** rng.uniform(-6.0, -2.0), rng.randint(3, 10), rng.choice([0, 2, 4])), PERFORM]
        requests.append(build_request(frame, steps))
    return requests


def append_helixbrep_requests(requests: list[str]) -> None:
    """Append the HelixBRep requests in a fixed order."""
    for index, params in enumerate(suite_parameters() + more_parameters()):
        frame = FRAMES[index % len(FRAMES)]
        requests.append(build_request(frame, [params, approx(1e-4), PERFORM]))
    requests += settings_requests()
    requests += random_requests(24)
