"""Deterministic requests for the AppBlend package
(tools/oracle/tkgeomalgo_probe_appblend.cxx).

`ab_appsurf` instantiates `AppBlend_AppSurf.gxx` on a probe line and section generator
(rational quadratic sections swept along a parameter, with 0 to 2 extra 2d curves,
with or without derivatives) and compares every query after `Perform` (Bezier pieces),
`Perform(SpApprox)` (one B-spline), `PerformSmoothing` (variational) and
`Perform(NbMaxP)` (pieces joined into a B-spline), each run twice to show the kept
state (the two-section degree narrowing), with known parameters, every
parametrization and continuity, the no-tangent context switch, invalid criterion
weights and a null line. `ab_context` sets, reads and resets the two switches.
"""
from __future__ import annotations


def _line(op: str, values: list[float]) -> str:
    return op + ' ' + ' '.join(repr(float(v)) for v in values)


# n len a b c d
SWEEPS = [
    [5, 1.0, 1.0, 0.2, 0.3, 0.1],
    [12, 3.0, 0.5, -0.1, 0.2, 0.05],
    [2, 1.0, 1.0, 0.0, 0.0, 0.0],
    [40, 2.0, 1.0, 0.3, -0.2, 0.08],
    [3, 0.5, 0.0, 1.0, 1.0, 0.2],
]
WEIGHTS = [0.4, 0.2, 0.4]


def _appsurf(sweep: list[float], np2d: int, deriv: int, mode: int, *, degrees=(3, 8), tols=(1e-4, 1e-5),
             nbit: int = 5, knownp: int = 0, partype: int = 0, cont: int = 4, nbmaxp: int = 10,
             notgt: int = 0, weights=WEIGHTS) -> str:
    values = sweep + [np2d, deriv, mode, *degrees, *tols, nbit, knownp, partype, cont, nbmaxp, notgt, *weights]
    return _line('ab_appsurf', values)


def append_appblend_requests(requests: list[str]) -> None:
    """Append the AppBlend requests: the approximations, then the context switches."""
    for sweep in SWEEPS:
        for mode in range(4):
            for np2d, deriv in ((0, 1), (1, 0), (2, 1)):
                requests.append(_appsurf(sweep, np2d, deriv, mode))
            requests.append(_appsurf(sweep, 1, 1, mode, knownp=1))
    base = SWEEPS[1]
    for mode in range(4):
        requests.append(_appsurf(base, 1, 1, mode, notgt=1))
        requests.append(_appsurf(base, 0, 1, mode, degrees=(2, 5), tols=(1e-6, 1e-7), nbit=0))
        for partype in (1, 2):
            requests.append(_appsurf(base, 0, 1, mode, partype=partype))
        for cont in (0, 2, 5):
            requests.append(_appsurf(base, 1, 0, mode, cont=cont))
    requests.append(_appsurf(SWEEPS[3], 1, 1, 3, nbmaxp=4, knownp=1))
    requests.append(_appsurf(SWEEPS[3], 0, 1, 3, nbmaxp=7))
    # NbMaxP = 0 divides by zero natively (SIGFPE): the port's error is a test only.
    requests.append(_appsurf(SWEEPS[0], 0, 1, 0, weights=[-1.0, 0.2, 0.4]))
    for b1, b2 in ((0, 0), (1, 0), (0, 1), (1, 1)):
        requests.append(_line('ab_context', [b1, b2]))
