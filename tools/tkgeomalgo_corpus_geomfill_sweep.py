"""Deterministic requests for the sweep part of GeomFill (tools/oracle/tkgeomalgo_probe_geomfill_*.cxx).

`gfs_shape` covers GeomFill::GetShape, Knots, Mults, GetMinimalWeights and GetTolerance
for every Convert_ParameterisationType over angles from 0 to beyond a full turn, with
arrays of other bounds and too short arrays. `gfs_circle` runs the three
GeomFill::GetCircle members on consistent section circles (quarter, half, near-full and
reflex arcs, both orientations, a zero angle and cosines clamped outside [-1, 1]) for
every parameterisation, with the right, a short and a shifted pole array. `gfs_convert`
drives both arc convertors, initialized or not, over tiny, ordinary and near-2PI arcs.
`gfs_tensor` covers GeomFill_Tensor, including reads outside the array. The trihedron,
location, section and sweep families come from the tkgeomalgo_corpus_geomfill_*.py
modules appended at the end.
"""
from __future__ import annotations
import math
import random

TGT = [0, 1, 2, 3, 4]
QUASI, RATIONAL_C1, POLYNOMIAL = 5, 6, 7


def line_of(op: str, values: list[float]) -> str:
    """One request `op v1 v2 ...` with every value as a Python float literal."""
    return op + ' ' + ' '.join(repr(float(v)) for v in values)


def nb_poles(angle: float, conv: int) -> int:
    """The NbPoles of GeomFill::GetShape."""
    if conv == QUASI:
        return 7
    if conv == POLYNOMIAL:
        return 8
    return 2 * math.ceil(3.0 * abs(angle) / 2.0 / math.pi) + 1


def _shape(requests: list[str]) -> None:
    angles = [0.0, 1e-9, 0.5, math.pi / 2, 2.0, math.pi, 4.0, 1.5 * math.pi, 2 * math.pi, 3 * math.pi, -1.0]
    for conv in range(8):
        for angle in angles:
            n = max(nb_poles(angle, conv) // 2 + 1, 2)
            requests.append(line_of('gfs_shape', [angle, conv, 1, n, 0.1, 2.0, 1e-4, 1e-7]))
        requests.append(line_of('gfs_shape', [2.0, conv, 3, 4, 3.0, 0.5, 0.01, 0.0]))
        requests.append(line_of('gfs_shape', [1.0, conv, 1, 1, 0.0, 1.0, 1e-3, 1e-5]))


def _frame(angle: float, radius: float, sign: float) -> tuple:
    """A section circle of `angle` in the XY plane: ns1, ns2 point to the centre."""
    center = [0.5, -1.0, 2.0]
    nplan = [0.0, 0.0, sign]
    ns1 = [-1.0, 0.0, 0.0]
    ns2 = [-math.cos(angle), -sign * math.sin(angle), 0.0]
    pts1 = [center[i] - radius * ns1[i] for i in range(3)]
    pts2 = [center[i] - radius * ns2[i] for i in range(3)]
    return ns1, ns2, nplan, pts1, pts2, center


def _circle_values(angle: float, radius: float, sign: float, rng: random.Random) -> list[float]:
    ns1, ns2, nplan, pts1, pts2, center = _frame(angle, radius, sign)
    def vec() -> list[float]:
        return [rng.uniform(-0.5, 0.5) for _ in range(3)]
    d1 = [*vec(), *vec(), *vec(), *vec(), *vec(), rng.uniform(-0.3, 0.3), *vec()]
    d2 = [*vec(), *vec(), *vec(), *vec(), *vec(), rng.uniform(-0.3, 0.3), *vec()]
    return [*ns1, *ns2, *nplan, *pts1, *pts2, radius, *center, *d1, *d2]


def _circle(requests: list[str]) -> None:
    rng = random.Random(1402)
    cases = [(math.pi / 2, 1.0, 1.0), (math.pi / 2, 2.5, -1.0), (math.pi, 1.0, 1.0), (2.0, 0.5, 1.0),
             (4.5, 1.5, 1.0), (4.5, 1.5, -1.0), (2 * math.pi - 1e-3, 1.0, 1.0), (0.0, 1.0, 1.0), (1e-7, 3.0, 1.0)]
    for angle, radius, sign in cases:
        values = _circle_values(angle, radius, sign, rng)
        for conv in [1, 2, 3, QUASI, POLYNOMIAL, 0]:
            n = nb_poles(angle, conv)
            requests.append(line_of('gfs_circle', [conv, n, 1, *values]))
        requests.append(line_of('gfs_circle', [2, 5, 4, *values]))
        requests.append(line_of('gfs_circle', [QUASI, 5, 1, *values]))
        requests.append(line_of('gfs_circle', [POLYNOMIAL, 9, 0, *values]))
    # Normals longer than 1: the cosine is clamped to +-1 and the sine set to 0.
    values = _circle_values(math.pi / 3, 1.0, 1.0, rng)
    values[0:6] = [-2.0, 0.0, 0.0, -1.5, 0.1, 0.0]
    for conv in [1, QUASI, POLYNOMIAL]:
        requests.append(line_of('gfs_circle', [conv, nb_poles(math.pi, conv), 1, *values]))
    values[3:6] = [2.0, 0.1, 0.0]
    requests.append(line_of('gfs_circle', [2, 5, 1, *values]))


def _convert(requests: list[str]) -> None:
    rng = random.Random(2203)
    angles = [1e-8, -1e-7, 0.3, 1.0, math.pi / 2, math.pi, 4.0, 2 * math.pi - 1e-7, 2 * math.pi - 1e-5, 2 * math.pi]
    for angle in angles:
        def vec(scale: float = 1.0) -> list[float]:
            return [rng.uniform(-scale, scale) for _ in range(3)]
        arc = [*vec(2.0), *vec(0.5), *vec(0.5), *vec(1.0), *vec(0.5), *vec(0.5), 0.1, 0.2, 0.9, *vec(0.3), *vec(0.3),
               angle, rng.uniform(-1, 1), rng.uniform(-1, 1)]
        for kind, n in [(0, 8), (1, 7)]:
            requests.append(line_of('gfs_convert', [kind, 1, n, 1, *arc]))
        requests.append(line_of('gfs_convert', [0, 1, 6, 1, *arc]))
        requests.append(line_of('gfs_convert', [1, 1, 7, 3, *arc]))
    arc = [1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1.0, 0.5, 0.0]
    requests.append(line_of('gfs_convert', [0, 0, 8, 1, *arc]))
    requests.append(line_of('gfs_convert', [1, 1, 7, 1, *arc]))


def _tensor(requests: list[str]) -> None:
    rng = random.Random(77)
    for nr, nc, nm in [(1, 1, 1), (2, 3, 4), (3, 2, 2)]:
        sets = []
        for _ in range(6):
            sets += [rng.randint(1, nr), rng.randint(1, nc), rng.randint(1, nm), rng.uniform(-5, 5)]
        sets += [nr + 1, 1, 1, 9.0, 0, 1, 1, 8.0]
        vec = [rng.uniform(-2, 2) for _ in range(nm)]
        requests.append(line_of('gfs_tensor', [nr, nc, nm, 0.5, len(sets) // 4, *sets, len(vec), *vec]))
    requests.append(line_of('gfs_tensor', [2, 2, 2, 1.0, 0, 1, 3.0]))


def append_geomfill_sweep_requests(requests: list[str]) -> None:
    """Append every GeomFill sweep request family in a fixed order."""
    _shape(requests)
    _circle(requests)
    _convert(requests)
    _tensor(requests)
    __import__('tkgeomalgo_corpus_geomfill_trihedron').append_trihedron_requests(requests)
    __import__('tkgeomalgo_corpus_geomfill_laws').append_laws_requests(requests)
    __import__('tkgeomalgo_corpus_geomfill_placement').append_placement_requests(requests)
