"""Deterministic requests for the FairCurve package
(tools/oracle/tkgeomalgo_probe_faircurve.cxx).

`fc_law` evaluates FairCurve_BattenLaw before and after its setters, with heights that
turn negative. `fc_density` evaluates the tension, sagging and jerk densities with
their gradients and packed Hessians (derivative orders 0 to 2, with and without the
free-sliding value, uniform tension) on a Bezier and a two-span B-spline of degree 9.
`fc_energy` evaluates FairCurve_EnergyOfBatten and FairCurve_EnergyOfMVC (Variable,
Value, Values, the Hessian) for every end-constraint order, then at perturbed variables
(the sliding guard: clamped and infinite slidings), and optionally minimizes them with
FairCurve_Newton. `fc_batten` computes batten and minimal-variation curves (end
angles that need several homotopy steps, free and imposed slidings, curvatures, null
heights, invalid heights and ratios), then moves their end points and computes again,
comparing every getter, the curve and the Dump text.
"""
from __future__ import annotations
import math


def _line(op: str, values: list[float]) -> str:
    return op + ' ' + ' '.join(repr(float(v)) for v in values)


def _bezier_arc(amplitude: float) -> list[float]:
    poles = [[3.0 * i / 9, amplitude * math.sin(math.pi * i / 9) + 0.1 * (i % 3)] for i in range(10)]
    return [2, 0.0, 1.0, 10, 10, 10] + [x for p in poles for x in p]


def _two_span(amplitude: float) -> list[float]:
    poles = [[2.0 * i / 15, amplitude * math.cos(math.pi * i / 15) * (1 + 0.05 * i)] for i in range(16)]
    return [3, 0.0, 0.5, 1.0, 10, 6, 10, 16] + [x for p in poles for x in p]


SPLINES = [_bezier_arc(0.4), _two_span(0.3)]
TS = [6, 0.0, 0.1, 0.37, 0.5, 0.9, 1.0]


def _law_requests(requests: list[str]) -> None:
    ts = [7, -0.5, 0.0, 0.25, 0.5, 0.75, 1.0, 1.5]
    for h, s, l, h2, s2, l2 in [
        (1.0, 0.0, 2.0, 2.0, 0.1, 3.0),
        (0.5, 0.3, 2.0, 0.1, -0.3, 1.0),
        (1.0, 1.5, 2.0, -1.0, 0.0, 2.0),
        (2.0, -3.0, 1.0, 0.0, 0.0, 0.0),
    ]:
        requests.append(_line('fc_law', [h, s, l, h2, s2, l2] + ts))


def _density_requests(requests: list[str]) -> None:
    for kind in range(3):
        for spline in SPLINES:
            for d in range(3):
                for aux in (0, 1):
                    for uniform in ((0, 1) if kind == 0 else (0,)):
                        d2 = 2 if d < 2 else 1
                        law = [1.0, 0.2, 3.0, 2.5]
                        requests.append(_line('fc_density', [kind] + spline + [d, aux] + law + [uniform, d2] + TS))
        # A law that vanishes inside the range: the density reports false there.
        requests.append(_line('fc_density', [kind] + SPLINES[0] + [2, 1, 0.2, 1.0, 3.0, 2.5, 0, 2] + TS))


ENERGY_ORDERS = [(0, 0), (1, 0), (0, 1), (1, 1), (2, 1), (1, 2), (2, 2)]


def _energy_requests(requests: list[str]) -> None:
    for kind in (0, 1):
        for spline in SPLINES:
            for o1, o2 in ENERGY_ORDERS:
                for free in (0, 1):
                    head = [kind] + spline + [o1, o2, free, 0.3, -0.2, 0.4, -0.1]
                    law = [1.0, 0.1, 3.2, 3.2]
                    ratio = 0.0 if (o1 + o2) % 2 else 0.5
                    requests.append(_line('fc_energy', head + law + [ratio, 0.01, 0, 1e-7, 1e-7, 40, 1e-6]))
        # The sliding guard: clamped (negative perturbation) and infinite slidings.
        for pert in (-0.07, 0.5):
            head = [kind] + SPLINES[0] + [1, 1, 1, 0.3, -0.2, 0.0, 0.0]
            requests.append(_line('fc_energy', head + [1.0, 0.1, 3.2, 3.2, 1.0, pert, 0, 1e-7, 1e-7, 40, 1e-6]))
        # The Newton minimization of the energies.
        for o1, o2 in [(0, 0), (1, 1), (2, 2)]:
            head = [kind] + SPLINES[0] + [o1, o2, 1, 0.2, 0.3, 0.2, 0.0]
            nt = [1e-4, 1e-4, 40, 1e-6]
            requests.append(_line('fc_energy', head + [1.0, 0.0, 3.2, 3.2, 0.3, 0.001, 1] + nt))
    # A physical ratio outside [0, 1] raises Standard_DomainError.
    requests.append(_line('fc_energy', [1] + SPLINES[0] + [1, 1, 0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 3.0, 3.0, 1.5,
                                                             0.01, 0, 1e-7, 1e-7, 40, 1e-6]))


def _batten(kind: int, p1: list[float], p2: list[float], h: float, slope: float, ratio: float,
            settings: list[float], moves: list[list[float]]) -> list[float]:
    return [kind] + p1 + p2 + [h, slope, ratio] + settings + [len(moves)] + [x for m in moves for x in m]


def _settings(free: int, o1: int, o2: int, a1: float, a2: float, factor: float = 1.0,
              c1: float = 0.0, c2: float = 0.0, h2: float = 1.0, nbiter: int = 50,
              tol: float = 1e-3) -> list[float]:
    return [free, o1, o2, a1, a2, factor, c1, c2, h2, nbiter, tol]


def _batten_requests(requests: list[str]) -> None:
    origin, unit = [0.0, 0.0], [1.0, 0.0]
    moves = [[0.0, 0.1, 1.5, 0.3]]
    for kind in (0, 1):
        ratio = 0.5 if kind else 0.0
        for o1, o2, a1, a2 in [(0, 0, 0.0, 0.0), (1, 1, 0.3, 0.2), (1, 1, -0.4, 0.3), (1, 0, 0.5, 0.0),
                               (0, 1, 0.0, 0.6), (1, 1, 1.2, 1.1), (1, 1, 2.5, -2.0)]:
            for free in (0, 1):
                requests.append(_line('fc_batten', _batten(kind, origin, unit, 1.0, 0.0, ratio,
                                                           _settings(free, o1, o2, a1, a2), [])))
        requests.append(_line('fc_batten', _batten(kind, origin, [2.0, 0.5], 0.5, 0.1, ratio,
                                                   _settings(0, 1, 1, 0.3, 0.3, factor=1.2), moves)))
        requests.append(_line('fc_batten', _batten(kind, origin, unit, 1.0, 0.0, ratio,
                                                   _settings(1, 1, 1, 0.8, 0.4, tol=1e-6), moves)))
        # Null height at an end (a steep slope), then an invalid height request.
        requests.append(_line('fc_batten', _batten(kind, origin, unit, 0.1, 3.0, ratio,
                                                   _settings(0, 1, 1, 0.3, 0.3, h2=0.1), [])))
        requests.append(_line('fc_batten', _batten(kind, origin, unit, 1.0, 0.0, ratio,
                                                   _settings(0, 1, 1, 0.2, 0.2, h2=-1.0), [])))
        # Confused points and a null height raise at construction; a move onto P2 raises.
        requests.append(_line('fc_batten', _batten(kind, origin, [0.0, 0.0], 1.0, 0.0, ratio,
                                                   _settings(0, 1, 1, 0.0, 0.0), [])))
        requests.append(_line('fc_batten', _batten(kind, origin, unit, 0.0, 0.0, ratio,
                                                   _settings(0, 1, 1, 0.0, 0.0), [])))
        requests.append(_line('fc_batten', _batten(kind, origin, unit, 1.0, 0.0, ratio,
                                                   _settings(0, 1, 1, 0.2, 0.1), [[1.0, 0.0, 1.0, 0.0]])))
    # Minimal variation: curvature constraints and physical ratios.
    for o1, o2, c1, c2 in [(2, 2, 0.5, -0.5), (2, 1, 1.0, 0.0), (1, 2, 0.0, 0.8), (2, 0, -0.7, 0.0)]:
        for ratio in (0.0, 0.3, 1.0):
            requests.append(_line('fc_batten', _batten(1, origin, unit, 1.0, 0.0, ratio,
                                                       _settings(0, o1, o2, 0.3, 0.2, c1=c1, c2=c2), [])))
    requests.append(_line('fc_batten', _batten(1, origin, [2.0, 1.0], 1.0, 0.05, 0.2,
                                               _settings(1, 2, 2, 0.4, -0.3, c1=0.2, c2=0.3), moves)))
    requests.append(_line('fc_batten', _batten(1, origin, unit, 1.0, 0.0, 1.5,
                                               _settings(0, 1, 1, 0.2, 0.2), [])))


def append_faircurve_requests(requests: list[str]) -> None:
    """Append the FairCurve requests: laws, densities, energies, then curves."""
    _law_requests(requests)
    _density_requests(requests)
    _energy_requests(requests)
    _batten_requests(requests)
