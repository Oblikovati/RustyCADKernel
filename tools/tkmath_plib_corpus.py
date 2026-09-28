"""Deterministic PLib differential requests for tools/verify_tkmath_numeric.py.

Usage: append_plib_requests(requests) after the other TKMath requests.
Every request stays inside the native defined domain: buffers are sized as the
C++ code reads them, and no request relies on undefined pointer arithmetic.
"""
from __future__ import annotations
import random

GAUSS_COUNTS = [8, 10, 15, 20, 25, 30, 40, 50, 61]


def line(op: str, values: list) -> str:
    """Format one request; line('plib_poly', [0.5, 1])."""
    return op+' '+' '.join(map(str, values))


def reals(rng: random.Random, count: int, scale: float = 10.0) -> list[float]:
    """Uniform values in [-scale, scale]; reals(rng, 3)."""
    return [rng.uniform(-scale, scale) for _ in range(count)]


def polynomial_requests(rng: random.Random) -> list[str]:
    """Horner branches: fused orders 1-2, general order 0 and >= 3, dims past 15."""
    requests = []
    for case in range(384):
        dr, deg = [0, 1, 2, 3, 5][case % 5], rng.randint(0, 8)
        dim = [1, 2, 3, 4, 7, 15, 16, 20][case % 8]
        scale = rng.choice([1e-300, 1.0, 1.0, 1e150])
        u = rng.choice([rng.uniform(-3, 3), 0.0, -0.0, 1e308, 1e-200])
        coefficients = [x*scale for x in reals(rng, (deg+1)*dim)]
        if case % 11 == 0: coefficients = [rng.choice([0.0, -0.0]) for _ in coefficients]
        requests.append(line('plib_poly', [u, dr, deg, dim]+coefficients))
    for case in range(128):
        udr, vdr, udeg, vdeg, dim = case % 4, (case//4) % 4, rng.randint(0, 4), rng.randint(0, 4), 1+case % 4
        requests.append(line('plib_poly2', [rng.uniform(-2, 2), rng.uniform(-2, 2), udr, vdr, udeg, vdeg, dim]
                             + reals(rng, (udeg+1)*(vdeg+1)*dim)))
    return requests


def rational_requests(rng: random.Random) -> list[str]:
    """RationalDerivative(s) with the unrolled Dimension==3 path and orders beyond Degree."""
    requests = []
    for case in range(192):
        deg, dr, dim, every = rng.randint(-1, 6), rng.randint(0, 5), [1, 2, 3, 3, 4, 5][case % 6], case % 2
        rows = max(min(dr, deg)+1, 1)
        ders = []
        for _ in range(rows):
            ders += reals(rng, dim)+[rng.choice([rng.uniform(0.2, 3), rng.uniform(-3, 3)])]
        if case % 23 == 0: ders[dim] = 0.0
        requests.append(line('plib_rational', [deg, dr, dim, every]+ders))
    return requests


def interpolation_requests(rng: random.Random) -> list[str]:
    """Lagrange (including coincident nodes) and cubic Hermite interpolation."""
    requests = []
    for case in range(192):
        deg, dr, dim = rng.randint(0, 6), rng.randint(-1, 7), 1+case % 4
        nodes = sorted(rng.uniform(-5, 5) for _ in range(deg+1))
        if case % 9 == 0 and deg > 0: nodes[deg] = nodes[deg-1]
        if case % 13 == 0 and deg > 0: nodes[0], nodes[1] = 1e-308, 1.5e-308
        requests.append(line('plib_lagrange', [rng.uniform(-6, 6), dr, deg, dim]+reals(rng, (deg+1)*dim)+nodes))
    for case in range(128):
        dr, dim = rng.randint(-1, 5), 1+case % 4
        first = rng.uniform(-5, 5)
        last = first if case % 17 == 0 else first+rng.choice([-1, 1])*rng.uniform(0.1, 4)
        requests.append(line('plib_cubic', [rng.uniform(-6, 6), dr, dim]+reals(rng, 4*dim)+[first, last]))
    return requests


def hermite_requests(rng: random.Random) -> list[str]:
    """Hermite bases (validity rejections included) and constrained interpolation."""
    requests = []
    bounds = [(-1.0, 1.0), (0.0, 1.0), (-100.0, 100.0), (-101.0, 1.0), (0.0, 0.001), (2.0, 2.0), (1.0, 1.005), (-3.0, 7.0)]
    for case in range(96):
        first, last = bounds[case % len(bounds)]
        orders = [(fo, lo) for fo in range(-1, 4) for lo in range(-1, 4) if fo+lo+2 >= 1]
        fo, lo = orders[case % len(orders)]
        requests.append(line('plib_hermite_coeffs', [first, last, fo, lo]))
    for case in range(128):
        dim, fo, lo = 1+case % 3, case % 3, (case//3) % 3
        first = rng.uniform(-3, 3)
        last = first if case % 19 == 0 else rng.uniform(-3, 3)
        requests.append(line('plib_hermite_interp', [dim, first, last, fo, lo]+reals(rng, dim*(fo+lo+2))))
    return requests


def curve_requests(rng: random.Random) -> list[str]:
    """CoefficientsPoles and Trimming on flat, point and scalar overloads, custom bounds."""
    requests = []
    for op in ['plib_coef_poles', 'plib_trim']:
        for case in range(160):
            dim, rational, n = [1, 2, 3, 4][case % 4], (case//4) % 2, rng.randint(1, 8)
            if case % 37 == 0: n = 27
            lower = [1, -2, 5][case % 3]
            wlen = n+1 if case % 29 == 0 else n
            weights = [rng.uniform(0.3, 3) for _ in range(wlen)]
            u1, u2 = rng.uniform(-2, 2), rng.uniform(-2, 2)
            requests.append(line(op, [dim, rational, n, lower, wlen, u1, u2]+reals(rng, n*dim)+weights))
    for case in range(96):
        rows, cols, rational = rng.randint(1, 5), rng.randint(1, 5), case % 2
        lr, lc = [1, 0, -3][case % 3], [1, 2, -1][(case//3) % 3]
        requests.append(line('plib_grid', [rows, cols, rational, lr, lc, rng.uniform(-2, 2), rng.uniform(-2, 2)]
                             + reals(rng, 3*rows*cols)+[rng.uniform(0.3, 3) for _ in range(rows*cols)]))
    return requests


def scalar_requests(rng: random.Random) -> list[str]:
    """JacobiParameters partial writes, constraint conversions, binomials and arc length."""
    requests = []
    for case in range(160):
        requests.append(line('plib_params', [case % 7, rng.randint(-1, 60), rng.randint(-6, 5), rng.randint(-1, 3),
                                             rng.randint(-1, 27), rng.randint(-1, 27)]))
    for case in range(96):
        deg = rng.choice([0, 1, 2, 3, 5, 8, 12, 20, 36, 40])
        dim = 1+case % 3
        tol = [1e-3, 1e-9, 0.0][case % 3] if case % 12 else 0.0
        requests.append(line('plib_length', [deg, dim, rng.uniform(-2, 1), rng.uniform(-1, 2), tol]+reals(rng, (deg+1)*dim, 3)))
    return requests


def basis_request(rng: random.Random, op: str, shape: int, work: int) -> str:
    """One Jacobi or Hermite-Jacobi request; basis_request(rng, 'plib_jacobi', 2, 10)."""
    nb = rng.choice(GAUSS_COUNTS+[9, 12, 60, -3])
    u = rng.choice([rng.uniform(-1, 1), -1.0, 1.0, 0.0, 0.99999])
    dim = rng.randint(1, 3)
    new_degree, max_degree = rng.randint(-1, work+1), rng.randint(0, work+2)
    tol = rng.choice([1e-3, 1e-6, 1e-12, 0.0])
    to_degree = rng.randint(0, max(work, 0))
    scale = rng.choice([1.0, 1e-6, 1e-11])
    coefficients = [x*scale for x in reals(rng, (max(work, 0)+1)*dim, 1)]
    return line(op, [work, shape, nb, u, dim, new_degree, max_degree, tol, to_degree]+coefficients)


def basis_requests(rng: random.Random) -> list[str]:
    """Both bases over every constraint level, degree 0..30 and the rejected extremes."""
    requests = []
    for op in ['plib_jacobi', 'plib_hermit']:
        for case in range(160):
            shape = [0, 2, 4][case % 3] if case % 16 else rng.choice([1, 3, 5, 6])
            niv = {0: 0, 2: 1, 4: 2}.get(shape, 0)
            work = 2*(niv+1)+rng.randint(0, 30)
            if case % 21 == 0: work = 2*(niv+1)+rng.choice([-1, 31])
            requests.append(basis_request(rng, op, shape, work))
    return requests


def append_plib_requests(requests: list[str]) -> None:
    """Extend the corpus with seeded PLib requests; append_plib_requests(requests)."""
    rng = random.Random(19950828)
    for family in [polynomial_requests, rational_requests, interpolation_requests, hermite_requests,
                   curve_requests, scalar_requests, basis_requests]:
        requests.extend(family(rng))
