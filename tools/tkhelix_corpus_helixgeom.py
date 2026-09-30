"""Deterministic requests for HelixGeom (tools/oracle/tkhelix_probe_helixgeom.cxx).

A helix is `t1 t2 pitch r taper cw`. The families:
- `hx_curve`: HelixGeom_HelixCurve loads (valid, invalid, reloads that keep the stored
  fields and the previous tan(taper)), the adaptor queries with interval arrays of
  several bounds (out-of-range indices raise) and D0..D3/DN evaluations;
- `hx_appr_curve3d`: HelixGeom_Tools::ApprCurve3D on helices, on a circle adaptor and on
  the Adaptor3d_Curve base class, over tolerances, continuities, segment counts and
  degrees (including the settings AdvApprox refuses and the ones it cannot reach);
- `hx_appr_helix`: HelixGeom_Tools::ApprHelix over the helix parameters;
- `hx_coil` and `hx_helix`: the builders, run several times on the same object, with
  full turns, partial turns, reversed ranges, tapers, tolerances AdvApprox cannot reach
  (it still reports done) and settings it refuses.
"""
from __future__ import annotations
import math
import random

TWO_PI = 2.0 * math.pi


def numbers(values: list[float]) -> str:
    """The fields of a request, with every double spelled exactly."""
    return ' '.join(repr(float(v)) for v in values)


def curve_request(loads: list[tuple[int, list[float]]], bounds: tuple[int, int], order: int,
                  params: list[float]) -> str:
    """One `hx_curve nloads (mode helix)... lo up n nu (u)...` request line."""
    fields: list[float] = [len(loads)]
    for mode, helix in loads:
        fields += [mode] + helix
    fields += [bounds[0], bounds[1], order, len(params)] + params
    return 'hx_curve ' + numbers(fields)


def appr_curve3d_request(adaptor: list[float], tol: float, cont: int, seg: int, deg: int) -> str:
    """One `hx_appr_curve3d adaptor tol cont maxseg maxdeg` request line."""
    return 'hx_appr_curve3d ' + numbers(adaptor + [tol, cont, seg, deg])


def appr_helix_request(helix: list[float], tol: float) -> str:
    """One `hx_appr_helix helix tol` request line."""
    return 'hx_appr_helix ' + numbers(helix + [tol])


def run(helix: list[float], tol: float = 1e-4, cont: int = 4, deg: int = 8, seg: int = 150,
        apply: int = 1) -> list[float]:
    """The fields of one builder run: `set tol cont maxdeg maxseg helix`."""
    return [apply, tol, cont, deg, seg] + helix


def coil_request(runs: list[list[float]]) -> str:
    """One `hx_coil nruns (run)...` request line."""
    return 'hx_coil ' + numbers([len(runs)] + [v for r in runs for v in r])


def helix_request(frame: list[float], runs: list[list[float]]) -> str:
    """One `hx_helix P N X nruns (run)...` request line."""
    return 'hx_helix ' + numbers(frame + [len(runs)] + [v for r in runs for v in r])


SAMPLES = [0.0, 0.5 * math.pi, math.pi, TWO_PI, -1.0, 10.0, 1e-9]
VALID_HELICES = [
    [0.0, TWO_PI, 5.0, 2.0, 0.0, 1], [0.0, 4.0 * math.pi, 10.0, 5.0, 0.0, 0],
    [-3.0, 5.0, 20.5, 7.5, 0.1, 1], [1.0, 1.5, 1.0, 1.0, -0.3, 0],
    [0.0, TWO_PI, 0.0, 3.0, 0.5, 1], [0.0, TWO_PI, 4.0, 0.0, 0.2, 1],
    [0.0, 6.0 * math.pi, 15.0, 4.0, 0.05, 1], [0.0, TWO_PI, 20.0, 5.0, 5e-5, 0],
    [0.0, 3.0, 2.0, 1.0, 1.2, 1], [-10.0, -9.0, 1e-3, 1e3, -1.5, 0],
]
INVALID_HELICES = [
    [2.0, 1.0, 5.0, 2.0, 0.0, 1], [1.0, 1.0, 5.0, 2.0, 0.0, 1], [0.0, TWO_PI, -1.0, 2.0, 0.0, 1],
    [0.0, TWO_PI, 5.0, -1.0, 0.0, 1], [0.0, TWO_PI, 5.0, 2.0, 0.5 * math.pi, 1],
    [0.0, TWO_PI, 5.0, 2.0, -0.5 * math.pi, 0], [0.0, TWO_PI, 5.0, 2.0, 2.0, 1],
]


def curve_requests() -> list[str]:
    """Loads, reloads, queries and evaluations of HelixGeom_HelixCurve."""
    requests = [curve_request([], (1, 2), 1, SAMPLES), curve_request([], (0, 3), 3, SAMPLES)]
    for order, helix in enumerate(VALID_HELICES + INVALID_HELICES):
        requests.append(curve_request([(0, helix)], (1, 2), order % 4, SAMPLES))
    for bounds in [(1, 1), (2, 3), (1, 0), (0, 1), (-2, 5)]:
        requests.append(curve_request([(0, VALID_HELICES[2])], bounds, 2, [0.25]))
    tapered, flat = VALID_HELICES[2], VALID_HELICES[1]
    reloads = [[(0, tapered), (0, flat)], [(0, tapered), (0, VALID_HELICES[7])],
               [(0, tapered), (0, INVALID_HELICES[2])], [(0, tapered), (0, INVALID_HELICES[0]), (1, flat)],
               [(1, flat)], [(0, INVALID_HELICES[4]), (1, flat), (0, flat), (1, flat)]]
    requests += [curve_request(loads, (1, 2), 2, SAMPLES) for loads in reloads]
    rng = random.Random(20261002)
    for _ in range(40):
        t1 = rng.uniform(-20.0, 20.0)
        helix = [t1, t1 + rng.uniform(1e-3, 40.0), rng.uniform(0.0, 50.0), rng.uniform(0.0, 20.0),
                 rng.uniform(-1.4, 1.4), rng.randint(0, 1)]
        requests.append(curve_request([(0, helix)], (1, 2), rng.randint(0, 3),
                                      [rng.uniform(-50.0, 50.0) for _ in range(4)]))
    return requests


def appr_requests() -> list[str]:
    """ApprCurve3D and ApprHelix over helices, other adaptors and settings."""
    requests = []
    settings = [(1e-4, 4, 150, 8), (1e-6, 2, 50, 6), (1e-6, 0, 30, 4), (1e-3, 4, 1, 3),
                (1e-12, 4, 2, 8), (1e-7, 2, 150, 14), (1e-4, 5, 150, 8), (1e-4, 1, 20, 8),
                (1e-4, 4, 150, 0), (1e-4, 4, -1, 8), (1e-4, 6, 150, 8), (0.0, 4, 10, 8)]
    for helix in VALID_HELICES[:7] + INVALID_HELICES[:2]:
        for tol, cont, seg, deg in settings[:4]:
            requests.append(appr_curve3d_request([0] + helix, tol, cont, seg, deg))
    for tol, cont, seg, deg in settings:
        requests.append(appr_curve3d_request([0] + VALID_HELICES[6], tol, cont, seg, deg))
    for tol, cont, seg, deg in settings[:6]:
        requests.append(appr_curve3d_request([1, 3.0, 0.0, 5.0], tol, cont, seg, deg))
    requests.append(appr_curve3d_request([1, 2.0, 1.0, 0.5], 1e-4, 4, 10, 8))
    requests.append(appr_curve3d_request([2], 1e-4, 4, 10, 8))
    for helix in VALID_HELICES + INVALID_HELICES[:3]:
        for tol in [1e-3, 1e-4, 1e-6, 1e-10]:
            requests.append(appr_helix_request(helix, tol))
    return requests


def coil_requests() -> list[str]:
    """HelixGeom_BuilderHelixCoil runs, alone and in sequences on one builder."""
    requests = [coil_request([]), coil_request([run([0.0] * 6, apply=0)])]
    for helix in VALID_HELICES + INVALID_HELICES[:3]:
        requests.append(coil_request([run(helix)]))
        requests.append(coil_request([run(helix, 1e-6, 2, 6, 50)]))
    good, tight, refused = run(VALID_HELICES[1]), run(VALID_HELICES[1], 1e-14, 4, 3, 1), run(VALID_HELICES[1], deg=0)
    for runs in [[good, tight], [good, refused], [good, run(INVALID_HELICES[0])], [tight, good],
                 [good, run([0.0] * 6, apply=0)]]:
        requests.append(coil_request(runs))
    return requests


FRAMES = [[0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0], [10.0, 20.0, 30.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
          [1.0, -2.0, 3.0, 0.3, -0.4, 0.5, 0.7, 0.1, -0.2], [0.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 1.0, 0.0]]
TURN_HELICES = [
    [0.0, TWO_PI, 10.0, 5.0, 0.0, 1], [0.0, 3.0 * TWO_PI, 10.0, 5.0, 0.0, 1],
    [0.0, 3.5 * TWO_PI, 4.0, 2.0, 0.0, 0], [1.0, 1.0 + 2.3 * TWO_PI, 6.0, 3.0, 0.2, 1],
    [0.0, 2.0 * TWO_PI + 1e-8, 5.0, 1.0, 0.0, 1], [0.0, 2.0 * TWO_PI + 1e-5, 5.0, 1.0, -0.1, 0],
    [0.0, 0.5 * math.pi, 3.0, 2.0, 0.0, 1], [5.0, -10.0, 3.0, 2.0, 0.0, 1], [5.0, 3.0, 3.0, 2.0, 0.0, 1],
    [-4.0, 4.0 * TWO_PI, 2.0, 7.0, 0.0, 1], [0.0, 2.0 * TWO_PI, 0.0, 4.0, 0.3, 1],
    [0.0, 5.0 * TWO_PI, 12.0, 3.0, 5e-5, 0], [0.0, 1e-9, 1.0, 1.0, 0.0, 1],
]


def helix_requests() -> list[str]:
    """HelixGeom_BuilderHelix runs over frames, turn counts and unreachable or refused settings."""
    requests = []
    for index, helix in enumerate(TURN_HELICES):
        requests.append(helix_request(FRAMES[index % len(FRAMES)], [run(helix)]))
        requests.append(helix_request(FRAMES[(index + 1) % len(FRAMES)], [run(helix, 1e-6, 2, 6, 40)]))
    unreachable = run(TURN_HELICES[3], 1e-14, 4, 3, 1)
    requests.append(helix_request(FRAMES[0], [unreachable]))
    requests.append(helix_request(FRAMES[1], [run(TURN_HELICES[1]), unreachable, run(TURN_HELICES[1], apply=0)]))
    requests.append(helix_request(FRAMES[2], [run(TURN_HELICES[2], deg=0)]))
    requests.append(helix_request(FRAMES[3], [run([0.0] * 6, apply=0)]))
    return requests


def append_helixgeom_requests(requests: list[str]) -> None:
    """Append the HelixGeom requests in a fixed order."""
    requests += curve_requests() + appr_requests() + coil_requests() + helix_requests()
