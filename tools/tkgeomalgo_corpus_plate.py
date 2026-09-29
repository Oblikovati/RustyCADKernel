"""Deterministic requests for the Plate package (tools/oracle/tkgeomalgo_probe_plate.cxx).

`plate_solve` loads pinpoint (value and derivative), linear XYZ and linear scalar
constraints into Plate_Plate, solves it for orders 2 to 9 and anisotropies around 1
(the three SolveTI systems, the invalid order, anisotropy and box cases), then reports
IsDone, Continuity, UVBox, Evaluate and EvaluateDerivative at sample points, CoefPol and
UVConstraints for the solved plate, its copy and an assigned plate.
`plate_constraint` builds the line, plane, global translation, sampled curve, GtoC
(G1/G2/G3, with and without projection) and FreeGtoC (G1/G2/G3, incremental loads and
orientations) constraints, prints their content, then solves a plate that loads them.
"""
from __future__ import annotations
import random


def _line(op: str, values: list[float]) -> str:
    return op + ' ' + ' '.join(repr(float(v)) for v in values)


def _ppc(u: float, v: float, p: list[float], iu: int = 0, iv: int = 0) -> list[float]:
    return [u, v, *p, iu, iv]


def _grid_ppcs(rng: random.Random, n: int) -> list[list[float]]:
    return [_ppc(rng.uniform(0, 1), rng.uniform(0, 1), [rng.uniform(-1, 1) for _ in range(3)]) for _ in range(n)]


def _solve(order: int, aniso: float, pponly: int, ppcs: list[list[float]], lxyz: list[list[float]],
           lsc: list[list[float]], uv: list[float]) -> str:
    fields = [order, aniso, pponly, len(ppcs)]
    for c in ppcs:
        fields += c
    fields.append(len(lxyz))
    for c in lxyz:
        fields += c
    fields.append(len(lsc))
    for c in lsc:
        fields += c
    return _line('plate_solve', fields + [len(uv), *uv])


def _corners() -> list[list[float]]:
    return [_ppc(0.0, 0.0, [0.0, 0.0, 1.0]), _ppc(1.0, 0.0, [1.0, 0.0, 0.0]), _ppc(0.0, 1.0, [0.0, 1.0, 0.0]),
            _ppc(1.0, 1.0, [1.0, 1.0, 0.5])]


def _solve_requests(requests: list[str], rng: random.Random) -> None:
    uv = [0.5, 0.5, 0.25, 0.75, 0.0, 0.0, 1.0, 1.0, 2.0, -1.0]
    corners = _corners()
    derivs = corners + [_ppc(0.5, 0.5, [0.1, 0.2, 0.3], 1, 0), _ppc(0.5, 0.5, [0.0, 0.1, 0.0], 0, 1),
                        _ppc(0.2, 0.8, [0.0, 0.0, 0.5], 1, 1), _ppc(0.8, 0.2, [0.0, 0.0, -0.5], 2, 0)]
    for order in range(1, 11):
        requests.append(_solve(order, 1.0, 0, corners, [], [], uv))
    for order in (2, 3, 4, 5, 6):
        requests.append(_solve(order, 1.0, 0, derivs, [], [], uv))
    for aniso in (1e-7, 0.5, 2.0, 1e7):
        requests.append(_solve(3, aniso, 0, corners, [], [], uv))
    requests.append(_solve(3, 1.0, 1, corners, [], [], uv))
    requests.append(_solve(3, 1.0, 0, [], [], [], uv))
    requests.append(_solve(3, 1.0, 0, [_ppc(0.5, 0.5, [1.0, 2.0, 3.0])], [], [], uv))
    requests.append(_solve(3, 1.0, 0, [_ppc(0.5, 0.5, [1.0, 2.0, 3.0]), _ppc(0.5, 0.5, [1.0, 2.0, 3.0])], [], [], uv))
    lxyz = [2, 3, *_ppc(0.3, 0.3, [0.0, 0.0, 0.0]), *_ppc(0.6, 0.3, [0.0, 0.0, 0.0]),
            *_ppc(0.6, 0.6, [0.0, 0.0, 0.0], 1, 0), 1.0, -1.0, 0.0, 0.0, 1.0, -0.5]
    lxyz2 = [1, 2, *_ppc(0.1, 0.9, [0.2, 0.0, 0.1]), *_ppc(0.9, 0.1, [0.0, 0.3, 0.0]), 0.5, 0.5]
    lsc = [2, 2, *_ppc(0.4, 0.2, [0.0, 0.0, 0.5]), *_ppc(0.2, 0.4, [0.0, 0.0, -0.5]),
           0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0]
    lsc2 = [1, 1, *_ppc(0.7, 0.7, [0.3, 0.3, 0.3], 0, 1), 0.0, 1.0, 1.0]
    for order in (2, 3, 4):
        requests.append(_solve(order, 1.0, 0, corners, [lxyz], [], uv))
        requests.append(_solve(order, 1.0, 0, corners, [lxyz, lxyz2], [], uv))
        requests.append(_solve(order, 1.0, 0, corners, [], [lsc], uv))
        requests.append(_solve(order, 1.0, 0, corners, [lxyz, lxyz2], [lsc, lsc2], uv))
    requests.append(_solve(3, 1.0, 0, [], [lxyz], [], uv))
    for seed in range(8):
        n = 5 + seed
        ppcs = _grid_ppcs(rng, n)
        requests.append(_solve(2 + seed % 6, rng.uniform(0.5, 2.0), seed % 2, ppcs, [lxyz2] if seed % 3 == 0 else [],
                               [lsc2] if seed % 4 == 1 else [], uv))


def _d(rng: random.Random, count: int) -> list[float]:
    return [rng.uniform(-1.0, 1.0) for _ in range(3 * count)]


def _constraint_requests(requests: list[str], rng: random.Random) -> None:
    fixed = [
        [0, 0.5, 0.5, 1.0, 2.0, 3.0, 0.0, 0.0, 1.0, 0, 0], [0, 0.5, 0.5, 1.0, 2.0, 3.0, 1.0, 0.0, 0.0, 1, 0],
        [0, 0.3, 0.6, 0.0, 0.0, 0.0, 0.6, 0.0, 0.8, 0, 0], [1, 0.5, 0.5, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0, 0],
        [1, 0.4, 0.5, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0, 1],
        [2, 0.0, 0.0, 6, 0.2, 0.2, 0.8, 0.2, 0.5, 0.9], [2, 0.0, 0.0, 2, 0.5, 0.5],
        [3, 0.0, 0.0, 4, *_ppc(0.1, 0.5, [0.0, 0.0, 0.1]), *_ppc(0.3, 0.5, [0.0, 0.0, 0.2]),
         *_ppc(0.6, 0.5, [0.0, 0.0, 0.1]), *_ppc(0.9, 0.5, [0.0, 0.0, 0.0]), 2],
        [3, 0.0, 0.0, 2, *_ppc(0.1, 0.5, [0.0, 0.0, 0.1]), *_ppc(0.3, 0.5, [0.0, 0.0, 0.2]), 3],
    ]
    for case in fixed:
        requests.append(_line('plate_constraint', case))
    s1 = [1.0, 0.0, 0.1, 0.0, 1.0, -0.1]
    t1 = [1.0, 0.1, 0.0, 0.0, 1.0, 0.2]
    ortho = [1.0, 0.0, 0.0, 0.0, 0.0, 1.0]
    flat = [1.0, 0.0, 0.0, 2.0, 0.0, 0.0]
    s2, t2 = [0.1, 0.0, 0.2, 0.0, 0.1, 0.0, 0.3, 0.0, 0.1], [0.0, 0.2, 0.1, 0.1, 0.0, 0.0, 0.0, 0.1, 0.3]
    s3, t3 = [0.01 * i for i in range(12)], [0.02 * (6 - i) for i in range(12)]
    np_ = [0.0, 1.0, 0.0]
    for kind in (4, 5):
        for d1s, d1t in ((s1, t1), (flat, t1), (s1, flat), (s1, ortho)):
            requests.append(_line('plate_constraint', [kind, 0.5, 0.5, *d1s, *d1t, *np_]))
    for kind in (6, 7):
        requests.append(_line('plate_constraint', [kind, 0.5, 0.5, *s1, *t1, *s2, *t2, *np_]))
        requests.append(_line('plate_constraint', [kind, 0.5, 0.5, *s1, *s1, *s2, *t2, *np_]))
    for kind in (8, 9):
        requests.append(_line('plate_constraint', [kind, 0.5, 0.5, *s1, *t1, *s2, *t2, *s3, *t3, *np_]))
    for load, orientation in ((1.0, 0), (0.5, 0), (0.5, 1), (0.25, -1), (1.0, 1)):
        requests.append(_line('plate_constraint', [10, 0.4, 0.6, *s1, *t1, load, orientation]))
        requests.append(_line('plate_constraint', [11, 0.4, 0.6, *s1, *t1, *s2, *t2, load, orientation]))
        requests.append(_line('plate_constraint', [12, 0.4, 0.6, *s1, *t1, *s2, *t2, *s3, *t3, load, orientation]))
    for kind in (10, 11, 12):
        higher = ([] if kind == 10 else [*s2, *t2]) + ([*s3, *t3] if kind == 12 else [])
        # A degenerate source normal, orthogonal normals, and parallel normals under an
        # incremental load (a null rotation axis).
        for d1s, d1t, load in ((flat, t1, 1.0), (flat, t1, 0.5), (s1, ortho, 1.0), (s1, s1, 0.5)):
            requests.append(_line('plate_constraint', [kind, 0.4, 0.6, *d1s, *d1t, *higher, load, 0]))
    for _ in range(12):
        kind = rng.choice([4, 5, 6, 7, 8, 9, 10, 11, 12])
        counts = {4: 5, 5: 5, 6: 11, 7: 11, 8: 19, 9: 19, 10: 4, 11: 10, 12: 18}[kind]
        tail = [rng.uniform(0.2, 1.0), rng.choice([-1, 0, 1])] if kind >= 10 else []
        requests.append(_line('plate_constraint', [kind, rng.uniform(0, 1), rng.uniform(0, 1), *_d(rng, counts), *tail]))


def append_plate_requests(requests: list[str]) -> None:
    """Append the Plate requests: plate solves, then constraint constructions."""
    rng = random.Random(20260932)
    _solve_requests(requests, rng)
    _constraint_requests(requests, rng)
