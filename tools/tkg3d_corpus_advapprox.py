"""TKG3d corpus: the AdvApprox package (tools/oracle/tkg3d_probe_advapprox.cxx).

Evaluator specs are `kind mode failAt` (kinds 0 polynomial, 1 trigonometric,
2 rational, 3 helix, 4 kink at 0.3, 5 exponential; modes 0 none, 1 fail on the
second derivative, 2 fail on call failAt). Cutting specs are 0 (default), 1
(DichoCutting) or 2 nrec rec... npref pref... weight (PrefAndRec).
"""
from __future__ import annotations
import random
from tkg3d_corpus import fmt

PREF_AND_REC = [2, 2, 0.55, 1.2, 2, 0.4, 0.9, 5.0]


def cut_spec(rng: random.Random):
    """A random cutting spec with non-empty PrefAndRec tables."""
    choice = rng.randrange(3)
    if choice < 2:
        return [choice]
    rec = sorted(rng.uniform(-1, 3) for _ in range(rng.randint(1, 3)))
    pref = sorted(rng.uniform(-1, 3) for _ in range(rng.randint(1, 3)))
    return [2, len(rec), *rec, len(pref), *pref, rng.choice([5.0, 2.0, 1.5])]


def append_function_requests(rng: random.Random, requests: list[str]) -> None:
    """AdvApprox_ApproxAFunction on every function, space layout, continuity and limit."""
    # The Geom_OffsetSurface iso-curve setting: one 3D space, C1, degree 14, 100 segments.
    for kind in range(6):
        for first, last in ((0.0, 2.0), (-1.0, 3.0), (0.2, 0.25)):
            requests.append('g3_advapprox ' + fmt([kind, 0, 0, 0, 0, 1, 0, 0, 1e-6, first, last, 1, 14, 100, 0]))
    layouts = [(1, 0, 0), (0, 1, 0), (0, 0, 1), (1, 1, 1), (2, 0, 1), (0, 2, 0), (1, 0, 2)]
    for index in range(60):
        n1, n2, n3 = rng.choice(layouts)
        tol = rng.choice([1e-2, 1e-4, 1e-7, 1e-10])
        cont = rng.choice([0, 1, 2, 1, 2])
        deg = rng.choice([1, 3, 6, 9, 14, 20])
        seg = rng.choice([1, 2, 5, 20, 100])
        first = rng.uniform(-1.5, 0.5)
        last = first + rng.choice([0.5, 1.0, 2.5, 4.0])
        evaluator = [rng.randrange(6), 0, 0]
        requests.append('g3_advapprox ' + fmt([*evaluator, n1, n2, n3, tol, tol * 2, tol * 3, first, last,
                                              cont, deg, seg, *cut_spec(rng)]))
    invalid = [
        ([0, 0, 0], (0, 0, 1), 0.0, 1.0, 3, 14, 10, [0]),     # C3: Standard_ConstructionError
        ([0, 0, 0], (0, 0, 1), 0.0, 1.0, 4, 14, 10, [0]),     # G1
        ([0, 0, 0], (0, 0, 0), 0.0, 1.0, 1, 14, 10, [0]),     # no subspace
        ([0, 0, 0], (-1, 0, 1), 0.0, 1.0, 1, 14, 10, [0]),    # negative count
        ([0, 0, 0], (0, 0, 1), 1.0, 0.0, 1, 14, 10, [0]),     # Last < First
        ([0, 0, 0], (0, 0, 1), 1.0, 1.0, 1, 14, 10, [0]),     # empty interval: error code 1
        ([0, 0, 0], (0, 0, 1), 0.0, 1.0, 1, 0, 10, [0]),      # MaxDeg < 1
        ([0, 0, 0], (0, 0, 1), 0.0, 1.0, 1, 14, -1, [0]),     # MaxSeg < 0
        ([0, 0, 0], (0, 0, 1), 0.0, 1.0, 1, 14, 0, [0]),      # no segment: error code 1
        ([1, 1, 0], (0, 0, 1), 0.0, 2.0, 2, 14, 10, [0]),     # evaluator fails on D2
        ([1, 1, 0], (1, 0, 0), 0.0, 2.0, 1, 14, 10, [0]),     # D2 never asked in C1
        ([4, 2, 5], (0, 0, 1), 0.0, 1.0, 1, 14, 10, [0]),     # fails during the constraints
        ([4, 2, 60], (0, 0, 1), 0.0, 1.0, 1, 14, 10, [0]),    # fails after a cut
        ([4, 0, 0], (0, 0, 1), 0.0, 1.0, 2, 3, 100, [2, 1, 0.3, 1, 0.31, 0.5]),  # PrefAndRec weight too small
        ([4, 0, 0], (1, 0, 0), -1.0, 1.0, 1, 2, 40, PREF_AND_REC),
    ]
    for evaluator, (n1, n2, n3), first, last, cont, deg, seg, cut in invalid:
        requests.append('g3_advapprox ' + fmt([*evaluator, n1, n2, n3, 1e-7, 1e-7, 1e-7, first, last, cont, deg, seg, *cut]))


def append_static_requests(rng: random.Random, requests: list[str]) -> None:
    """AdvApprox_ApproxAFunction::Approximation with its raw tables and codes."""
    for index in range(30):
        dims = rng.choice([[1], [3], [1, 2], [3, 3], [2, 1, 3]])
        order = rng.choice([0, 1, 2])
        coeffs = rng.choice([2 * order + 2, 8, 12, 15])
        code = rng.choice([1, 2, 3, 4, -1, -3])
        evaluator = [rng.randrange(6), 0, 0]
        requests.append('g3_advapprox_static ' + fmt([*evaluator, len(dims), *dims, sum(dims), rng.uniform(-1, 0),
                                                     rng.uniform(0.5, 3.0), order, coeffs, rng.choice([1, 4, 30]),
                                                     rng.choice([1e-3, 1e-6, 1e-9]), code, *cut_spec(rng)]))
    edge = [
        ([0, 0, 0], [3], 2, 0.0, 1.0, 1, 10, 5, 1e-6, 1),      # inconsistent total dimension
        ([0, 0, 0], [3], 3, 0.0, 1.0, 3, 10, 5, 1e-6, 1),      # order 3: Standard_ConstructionError
        ([0, 0, 0], [3], 3, 0.0, 1.0, 1, 10, 5, 1e-6, 0),      # code 0: Standard_ConstructionError
        ([0, 0, 0], [3], 3, 0.0, 1.0, 2, 4, 5, 1e-6, 1),       # too few coefficients
        ([0, 0, 0], [1], 1, 0.0, 1.0, 1, 10, 0, 1e-6, 1),      # no segment
        ([5, 2, 30], [1, 2], 3, 0.0, 2.0, 1, 10, 30, 1e-12, 1),  # evaluator error after cuts
        ([4, 0, 0], [1], 1, -1.0, 1.0, 0, 3, 50, 1e-12, 1),    # the stack fills up
    ]
    for evaluator, dims, given, first, last, order, coeffs, seg, tol, code in edge:
        requests.append('g3_advapprox_static ' + fmt([*evaluator, len(dims), *dims, given, first, last, order,
                                                     coeffs, seg, tol, code, 0]))


def append_simple_requests(rng: random.Random, requests: list[str]) -> None:
    """AdvApprox_SimpleApprox on one interval, odd and even Gauss point counts."""
    for index in range(30):
        dims = rng.choice([[1], [2], [3], [1, 3], [2, 1]])
        cont = rng.choice([0, 1, 2])
        code = rng.choice([1, 2, 3, 4, -2, -5])
        maxcoef = rng.choice([2 * cont + 1, 7, 11, 13, 20])
        evaluator = [rng.randrange(6), rng.choice([0, 0, 0, 1, 2]), rng.randint(1, 40)]
        requests.append('g3_advapprox_simple ' + fmt([*evaluator, len(dims), *dims, cont, code, maxcoef,
                                                     rng.uniform(-1, 0), rng.uniform(0.3, 2.0),
                                                     rng.choice([1, 5, 13]), rng.choice([1e-3, 1e-8]), ]))
    requests.append('g3_advapprox_simple ' + fmt([0, 0, 0, 1, 1, 3, 1, 13, 0.0, 1.0, 13, 1e-6]))


def append_cut_requests(rng: random.Random, requests: list[str]) -> None:
    """The two cutting strategies on long, short and degenerate intervals."""
    for a, b in ((0.0, 1.0), (0.0, 1e-8), (0.0, 2.5e-8), (1.0, 0.0), (-1.0, 3.0), (0.4, 0.6)):
        requests.append('g3_advapprox_cut ' + fmt([a, b, *PREF_AND_REC]))
        requests.append('g3_advapprox_cut ' + fmt([a, b, *cut_spec(rng)]))
    requests.append('g3_advapprox_cut ' + fmt([0.0, 1.0, 2, 1, 0.5, 1, 0.5, 1.0]))


def append_advapprox_requests(requests: list[str]) -> None:
    """Every AdvApprox request, with its own generator so other families keep their inputs."""
    rng = random.Random(0xADA9)
    append_function_requests(rng, requests)
    append_static_requests(rng, requests)
    append_simple_requests(rng, requests)
    append_cut_requests(rng, requests)
