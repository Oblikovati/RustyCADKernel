"""Deterministic requests for the Law package (tools/oracle/tkgeomalgo_probe_law.cxx).

`law_bspline` builds Law_BSpline laws (clamped, rational, periodic, piecewise Bezier,
uniform, C0, invalid data) and evaluates every query and evaluator at knots, inside,
at the ends and outside the range. `law_edit` applies each editing member (degree
elevation, knot insertion and removal, reversal, segmentation, periodicity, origin,
weights, knots, poles, MovePointAndTangent, Copy) and prints the resulting law.
`law_function` drives every Law_Function (constant, linear, S, B-spline, interpolated,
composite) through Bounds, Continuity, NbIntervals/Intervals for every GeomAbs_Shape,
Value, D1, D2 and Trim. `law_mix` covers the Law class functions and
Law_BSplineKnotSplitting, `law_interpolate` covers Law_Interpolate with and without
parameters and tangents, periodic or not.
"""
from __future__ import annotations
import random


def _arr(values: list[float]) -> list[float]:
    return [len(values), *values]


def _law(deg: int, poles: list[float], knots: list[float], mults: list[int], periodic: bool = False,
         weights: list[float] | None = None) -> list[float]:
    head = [deg, int(periodic), int(weights is not None), *_arr(poles)]
    if weights is not None:
        head += _arr(weights)
    return head + _arr(knots) + _arr(mults)


def _line(op: str, values: list[float]) -> str:
    return op + ' ' + ' '.join(repr(float(v)) for v in values)


# name -> (law fields, parameters to evaluate)
LAWS = {
    'cubic': (_law(3, [0.0, 1.0, 3.0, 2.0, 4.0], [0.0, 0.5, 1.0], [4, 1, 4]), [0.0, 0.25, 0.5, 0.75, 1.0, -0.5, 1.5]),
    'rational': (_law(2, [1.0, 2.0, 0.5, 3.0], [0.0, 1.0, 2.0], [3, 1, 3], weights=[1.0, 2.0, 0.5, 1.0]),
                 [0.0, 0.3, 1.0, 1.7, 2.0]),
    'equal_weights': (_law(2, [1.0, 2.0, 3.0], [0.0, 1.0], [3, 3], weights=[2.0, 2.0, 2.0]), [0.0, 0.5, 1.0]),
    'periodic': (_law(3, [1.0, 3.0, 2.0, 5.0], [0.0, 1.0, 2.0, 3.0, 4.0], [1, 1, 1, 1, 1], periodic=True),
                 [0.0, 0.5, 2.0, 3.99, 4.0, 5.5, -1.25, 9.0]),
    'periodic_rational': (_law(2, [1.0, 3.0, 2.0], [0.0, 1.0, 2.0, 3.0], [2, 1, 1, 2], periodic=True,
                                weights=[1.0, 3.0, 1.0]), [0.0, 1.5, 3.0, 4.2]),
    'linear': (_law(1, [0.0, 2.0, 1.0], [0.0, 1.0, 3.0], [2, 1, 2]), [0.0, 1.0, 2.0, 3.0]),
    'bezier_pieces': (_law(2, [0.0, 1.0, 0.5, 2.0, 1.0], [0.0, 1.0, 2.0], [3, 2, 3]), [0.0, 1.0, 1.5, 2.0]),
    'uniform': (_law(2, [0.0, 1.0, 0.5], [0.0, 1.0, 2.0, 3.0, 4.0, 5.0], [1, 1, 1, 1, 1, 1]), [2.0, 2.5, 3.0]),
    'c0': (_law(3, [0.0, 1.0, 2.0, 3.0, 2.0, 1.0, 0.0], [0.0, 1.0, 2.0], [4, 3, 4]), [0.5, 1.0, 1.5]),
    'quintic': (_law(5, [0.0, 1.0, -1.0, 2.0, 0.5, 1.0, 3.0], [0.0, 0.4, 1.0], [6, 1, 6]), [0.1, 0.4, 0.9]),
}
INVALID = [
    _law(0, [1.0, 2.0], [0.0, 1.0], [1, 1]),
    _law(3, [1.0, 2.0, 3.0], [0.0, 1.0], [4, 4]),
    _law(2, [1.0, 2.0, 3.0], [1.0, 0.0], [3, 3]),
    _law(2, [1.0, 2.0, 3.0], [0.0, 1.0], [3, 3], weights=[1.0, 0.0, 1.0]),
    _law(2, [1.0, 2.0, 3.0], [0.0, 1.0], [3, 3], weights=[1.0, 2.0]),
    _law(26, [1.0] * 27, [0.0, 1.0], [27, 27]),
]


def _bspline_requests(requests: list[str]) -> None:
    for fields, params in LAWS.values():
        requests.append(_line('law_bspline', fields + _arr(params)))
    for fields in INVALID:
        requests.append(_line('law_bspline', fields + _arr([0.5])))


# (op, a, c, d) edits; op codes follow ga_law_edit_case.
EDITS = [
    (0, 3, 0, 0), (0, 5, 0, 0), (0, 1, 0, 0), (0, 30, 0, 0), (1, 0.25, 1, 0.0), (1, 0.5, 2, 0.0),
    (1, 0.75, 3, 1e-3), (1, 5.0, 1, 0.0), (2, 2, 0, 1e-7), (2, 2, 0, 10.0), (2, 1, 0, 1e-7), (2, 2, -1, 0.0),
    (3, 0, 0, 0), (4, 0.2, 0.8, 0), (4, 0.8, 0.2, 0), (4, 0.0, 1.0, 0), (5, 0, 0, 0), (6, 0, 0, 0), (7, 2, 0, 0),
    (7, 9, 0, 0), (8, 2, 3.0, 0), (8, 1, 1.0, 0), (8, 2, -1.0, 0), (9, 2, 0.45, 0), (9, 2, 2.0, 0), (9, 7, 0.1, 0),
    (10, 0.3, 2.0, 0.5), (11, 2, 3, 0), (12, 1, 2, 1), (13, 2, 1.5, 2.0), (13, 9, 1.0, 1.0), (14, 2, 0.4, 2),
    (15, 1, 2, 2), (16, 0, 0, 0),
]


def _edit_requests(requests: list[str]) -> None:
    for name in ('cubic', 'rational', 'periodic', 'periodic_rational', 'c0'):
        fields, params = LAWS[name]
        u = params[len(params) // 2]
        for op, a, c, d in EDITS:
            if op == 10 and name.startswith('periodic'):
                # Native MovePointAndTangent writes past the poles of a periodic law
                # (undefined behaviour); the port raises Standard_OutOfRange instead.
                continue
            requests.append(_line('law_edit', fields + [op, a, c, d, u]))


def _function_requests(requests: list[str], rng: random.Random) -> None:
    cubic, _ = LAWS['cubic']
    periodic, _ = LAWS['periodic']
    c0, _ = LAWS['c0']
    pts = [0.0, 1.0, 0.5, 2.0, 1.25, 1.5, 2.0, 3.0]
    cases = [
        [0, 2.5, 0.0, 1.0, 0.5], [1, 0.0, 1.0, 2.0, 5.0, 0.7], [1, 1.0, 1.0, 1.0, 5.0, 1.0],
        [2, 0.0, 1.0, 2.0, 3.0, 0.4], [3, 0.0, 1.0, 0.5, 2.0, 3.0, -1.0, 1.2],
        [4, *cubic, 0.0, 1.0, 0.5], [4, *cubic, 0.25, 0.75, 0.25], [4, *cubic, 0.25, 0.75, 0.75],
        [4, *c0, 0.0, 2.0, 1.0], [4, *c0, 0.5, 1.5, 0.5], [4, *periodic, 0.0, 4.0, 4.0], [4, *periodic, 0.5, 3.5, 2.0],
        [5, 0, *_arr(pts), 0.0, 0.0, 1.0], [5, 0, *_arr(pts[:6]), 0.0, 0.0, 0.7], [5, 0, *_arr(pts[:4]), 0.0, 0.0, 0.3],
        [5, 1, *_arr(pts + [4.0, 1.0]), 0.0, 0.0, 2.5], [6, 0, *_arr(pts), 1.0, -1.0, 1.0],
        [7, 0, *_arr(pts), 10.0, 20.0, 15.0], [7, 1, *_arr(pts + [4.0, 1.0]), 0.0, 1.0, 0.5],
        [8, 0.0, 3.0, 1e-9, 0, *_arr([0.0, 1.0, 1.0, 2.0, 1.0, 2.0, 2.0, 0.0, 2.0, 0.0, 3.0, 1.0]), 1.0],
        [8, 0.0, 3.0, 1e-9, 0, *_arr([0.0, 1.0, 1.0, 2.0, 1.0, 2.0, 2.0, 0.0, 2.0, 0.0, 3.0, 1.0]), 2.5],
        [8, 0.0, 3.0, 1e-3, 1, *_arr([0.0, 1.0, 1.0, 2.0, 1.0, 2.0, 2.0, 0.0, 2.0, 0.0, 3.0, 1.0]), 4.5],
        [8, 0.0, 3.0, 1e-9, 0, *_arr([0.0, 1.0, 1.0, 2.0, 1.0, 2.0, 2.0, 0.0, 2.0, 0.0, 3.0, 1.0]), -2.0],
        [8, 0.0, 1.0, 0.0, 0, *_arr([]), 0.5],
    ]
    for case in cases:
        requests.append(_line('law_function', case))
    for _ in range(10):
        a, b = sorted(rng.uniform(-2.0, 2.0) for _ in range(2))
        requests.append(_line('law_function', [2, a, rng.uniform(-1, 1), b + 0.5, rng.uniform(-1, 1), rng.uniform(a, b)]))


def _mix_requests(requests: list[str]) -> None:
    cubic, _ = LAWS['cubic']
    c0, _ = LAWS['c0']
    cases = [
        [0, 0.0, 1.0, 2.0, 3.0, 0.3], [0, -1.0, 5.0, 4.0, -2.0, 3.9],
        [1, 3, *_arr([0.0, 0.5, 1.0]), *_arr([4, 1, 4]), 0.0, 1.0, 1.0, 2.0],
        [1, 2, *_arr([0.0, 1.0]), *_arr([3, 3]), 0.0, 1.0, 1.0, 2.0],
        [2, 3, *_arr([0.0, 0.5, 1.0]), *_arr([4, 2, 4]), 1, 2], [2, 3, *_arr([0.0, 0.5, 1.0]), *_arr([4, 2, 4]), 0, 2],
        [2, 3, *_arr([0.0, 0.25, 0.5, 1.0]), *_arr([4, 1, 2, 4]), 1, 3],
    ]
    for hf in (0, 1):
        for hl in (0, 1):
            cases.append([3, 0.0, 2.0, hf, hl, 0.5, 2.5])
            cases.append([4, 0.0, 2.0, hf, hl, 0.5, 2.5])
    for rev in (0, 1):
        for hdf, hdl in ((0, 0), (1, 0), (0, 1), (1, 1)):
            cases.append([5, 2.0, 0.0, 1.0, hdf, hdl, 0.5, -0.25, rev, 8])
    cases.append([5, 1.0, -1.0, 3.0, 0, 0, 0.0, 0.0, 0, 4])
    for cont in range(0, 4):
        cases.append([6, *cubic, cont])
        cases.append([6, *c0, cont])
    cases.append([6, *cubic, -1])
    for case in cases:
        requests.append(_line('law_mix', case))


def _interpolate_requests(requests: list[str]) -> None:
    pts = [0.0, 1.0, 3.0, 2.0, 2.5]
    par = [0.0, 1.0, 1.5, 3.0, 4.0]
    cases = [
        [0, *_arr(pts), 0, *_arr([]), 0, 0.0, 0.0, *_arr([]), *_arr([])],
        [0, *_arr(pts), 1, *_arr(par), 0, 0.0, 0.0, *_arr([]), *_arr([])],
        [0, *_arr(pts), 1, *_arr(par), 1, 1.0, -1.0, *_arr([]), *_arr([])],
        [0, *_arr(pts), 1, *_arr(par), 2, 0.0, 0.0, *_arr([0.5, 0.0, 2.0, 0.0, -1.0]), *_arr([1, 0, 1, 0, 1])],
        [0, *_arr(pts), 1, *_arr(par), 2, 0.0, 0.0, *_arr([0.5, 0.0, 2.0, 0.0, -1.0]), *_arr([0, 0, 1, 0, 0])],
        [0, *_arr(pts[:3]), 0, *_arr([]), 0, 0.0, 0.0, *_arr([]), *_arr([])],
        [0, *_arr(pts[:2]), 0, *_arr([]), 0, 0.0, 0.0, *_arr([]), *_arr([])],
        [0, *_arr(pts[:2]), 0, *_arr([]), 1, 1.0, 2.0, *_arr([]), *_arr([])],
        [0, *_arr(pts[:3]), 1, *_arr(par[:3]), 1, 1.0, 2.0, *_arr([]), *_arr([])],
        [1, *_arr(pts[:4]), 0, *_arr([]), 0, 0.0, 0.0, *_arr([]), *_arr([])],
        [1, *_arr(pts[:4]), 1, *_arr(par), 0, 0.0, 0.0, *_arr([]), *_arr([])],
        [1, *_arr(pts[:4]), 1, *_arr(par), 2, 0.0, 0.0, *_arr([0.0, 1.0, 0.0, 2.0]), *_arr([0, 1, 0, 1])],
        [1, *_arr(pts[:4]), 1, *_arr(par[:4]), 0, 0.0, 0.0, *_arr([]), *_arr([])],
        [0, *_arr(pts), 1, *_arr([0.0, 1.0, 1.0, 3.0, 4.0]), 0, 0.0, 0.0, *_arr([]), *_arr([])],
        [0, *_arr(pts), 1, *_arr(par), 2, 0.0, 0.0, *_arr([1.0, 2.0]), *_arr([1, 1])],
    ]
    for case in cases:
        requests.append(_line('law_interpolate', case))


def append_law_requests(requests: list[str]) -> None:
    """Append the Law requests: B-spline laws, edits, functions, class functions, interpolations."""
    rng = random.Random(20260931)
    _bspline_requests(requests)
    _edit_requests(requests)
    _function_requests(requests, rng)
    _mix_requests(requests)
    _interpolate_requests(requests)
