"""Deterministic requests for AppCont (tools/oracle/tkgeombase_probe_appcont.cxx).

Every entry of the five tables is read directly (`acc_table`), so the generated Rust
tables are proven bit-identical to what g++ compiled. The matrix providers run every
classe up to past their limits, VBernstein every classe x nbpoints pair, into
matrices with shifted bounds and too small (partial writes, then Standard_OutOfRange).
The fit runs 3D, 2D and mixed analytic functions (polynomial, trigonometric,
constant) over forward, reversed and tiny ranges, every pair of end constraints
(including CurvaturePoint, which the fit treats as a constrained-but-unfixed end),
degrees 1 to 25 with 1 to 40 Gauss points, failing derivatives, singular ends (jumps
below and above the 0.1 threshold) and periodic coordinates that wrap. Left out:
the paths that are undefined behavior natively (VBernstein at classe 1, i.e. degree
0; more than 61 Gauss points; periodic functions with fewer than 3 samples), and
empty ranges (degree < 0, no Gauss point, no coordinate), where the native
math_Matrix builds empty or negative-size tables on its stack buffer (degree -2
crashes) while the TKMath port refuses them with Standard_RangeError.
"""
from __future__ import annotations
import math
import random
from generate_appcont_tables import TABLES
from tkgeombase_corpus_gce import add

NO, PASS, TANGENT, CURVATURE = 0, 1, 2, 3
MMATRIX, INV_MMATRIX, IBP, IBT, VB = range(5)
TABLE_CHUNK = 500


def table_requests(requests: list[str]) -> None:
    """Every table entry, TABLE_CHUNK per request."""
    for kind, (_, _, size) in enumerate(TABLES):
        for start in range(0, size, TABLE_CHUNK):
            add(requests, 'acc_table', kind, start, min(TABLE_CHUNK, size-start))


def offset_requests(requests: list[str]) -> None:
    for classe in range(-3, 31):
        add(requests, 'acc_offsets', classe)


def square_dim(kind: int, classe: int) -> int:
    """The native result size of a square provider."""
    return classe - {MMATRIX: 0, INV_MMATRIX: 0, IBP: 2, IBT: 4}[kind]


def square_requests(requests: list[str]) -> None:
    """MMatrix, InvMMatrix, IBPMatrix, IBTMatrix over every classe, shifted and short."""
    for kind in (MMATRIX, INV_MMATRIX, IBP, IBT):
        for classe in range(-1, 29):
            dim = max(square_dim(kind, classe), 1)
            add(requests, 'acc_matrix', kind, classe, 0, 1, 1, dim, dim)
            if classe in (3, 7, 12, 24, 26):
                add(requests, 'acc_matrix', kind, classe, 0, -2, 5, dim, dim)
                add(requests, 'acc_matrix', kind, classe, 0, 1, 1, dim + 2, dim + 1)
                add(requests, 'acc_matrix', kind, classe, 0, 1, 1, max(dim - 1, 1), dim)


def v_bernstein_requests(requests: list[str]) -> None:
    """VBernstein for every classe x nbpoints (classe 1 reads before the table natively)."""
    for classe in [-2, 0, *range(2, 28)]:
        for nbpoints in range(-1, 26):
            add(requests, 'acc_matrix', VB, classe, nbpoints, 1, 1, max(classe, 1), max(nbpoints, 1))
    for classe, nbpoints in [(3, 4), (6, 12), (26, 24)]:
        add(requests, 'acc_matrix', VB, classe, nbpoints, 0, 0, classe, nbpoints)
        add(requests, 'acc_matrix', VB, classe, nbpoints, 1, 1, classe + 1, nbpoints + 3)
        add(requests, 'acc_matrix', VB, classe, nbpoints, 1, 1, classe, nbpoints - 1)


def function_spec(rng: random.Random, nb3d: int, nb2d: int, shape: str, periods=(), d1ok=1,
                  jumps=(0.0, 0.0), u=(0.0, 1.0)) -> list:
    """The `acc_ls` function fields: range, counts, D1 flag, jumps, periods, coefficients."""
    dims = max(3*nb3d + 2*nb2d, 0)
    coefficients = []
    for _ in range(dims):
        a = [rng.uniform(-2.0, 2.0) for _ in range(3)]
        trig = [rng.uniform(-1.0, 1.0), rng.uniform(0.5, 4.0)]
        if shape == 'poly':
            trig = [0.0, 1.0]
        elif shape == 'constant':
            a, trig = [a[0], 0.0, 0.0], [0.0, 0.0]
        coefficients += [*a, *trig]
    fields = [*u, nb3d, nb2d, d1ok, *jumps, len(periods)]
    for dim, period in periods:
        fields += [dim, period]
    return fields + coefficients


def functions(rng: random.Random) -> list[list]:
    """The fitted functions: 3D, 2D, mixed, special shapes, ranges and ends."""
    return [
        function_spec(rng, 1, 0, 'trig'),
        function_spec(rng, 0, 1, 'trig'),
        function_spec(rng, 1, 1, 'trig'),
        function_spec(rng, 2, 2, 'trig', u=(-2.0, 3.5)),
        function_spec(rng, 1, 1, 'poly'),
        function_spec(rng, 0, 2, 'constant'),
        function_spec(rng, 1, 0, 'trig', u=(1.0, 0.0)),
        function_spec(rng, 1, 1, 'trig', u=(1.0, 1.0001)),
        function_spec(rng, 1, 1, 'trig', d1ok=0),
        function_spec(rng, 1, 1, 'trig', jumps=(3.0, 0.05)),
        function_spec(rng, 0, 1, 'trig', jumps=(0.02, -0.5)),
    ]


# (degree, Gauss points): every resolution path and the native limits.
DEGREES = [(1, 4), (2, 3), (3, 8), (4, 10), (5, 12), (6, 24), (8, 16), (10, 20), (13, 24),
           (17, 24), (20, 24), (23, 24), (24, 24), (25, 24), (3, 1), (3, 2), (4, 25), (4, 40)]
PAIRS = [(a, b) for a in (NO, PASS, TANGENT) for b in (NO, PASS, TANGENT)] + [(CURVATURE, NO), (PASS, CURVATURE)]


def fit_requests(rng: random.Random, requests: list[str]) -> None:
    for spec in functions(rng):
        for first, last in PAIRS:
            for deg, nbpoints in DEGREES:
                add(requests, 'acc_ls', *spec, first, last, deg, nbpoints)


def periodic_requests(rng: random.Random, requests: list[str]) -> None:
    """Coordinates reduced into [0, P): sample jumps, first-sample and end-pole fixes."""
    specs = [
        function_spec(rng, 0, 1, 'poly', periods=[(1, 2*math.pi)]),
        function_spec(rng, 0, 1, 'poly', periods=[(1, 2*math.pi)]),
        function_spec(rng, 0, 1, 'trig', periods=[(1, 1.5), (2, 1.0)], jumps=(0.7, 0.9)),
        function_spec(rng, 1, 1, 'trig', periods=[(2, 0.8), (4, 2.0)], u=(-1.0, 2.0)),
    ]
    # A steep first coordinate that wraps twice, and one that wraps just after the
    # first of 3 samples (the first-sample fix).
    specs[0][-10:-5] = [0.3, 13.0, 0.0, 0.0, 1.0]
    specs[1][-10:-5] = [5.9, 1.0, 0.0, 0.0, 1.0]
    for spec in specs:
        for first, last in PAIRS:
            for deg, nbpoints in [(3, 3), (5, 12), (7, 24), (9, 20)]:
                add(requests, 'acc_ls', *spec, first, last, deg, nbpoints)


def append_appcont_requests(requests: list[str]) -> None:
    """Append the AppCont requests in a fixed order."""
    rng = random.Random(19950314)
    table_requests(requests)
    offset_requests(requests)
    square_requests(requests)
    v_bernstein_requests(requests)
    fit_requests(rng, requests)
    periodic_requests(rng, requests)
