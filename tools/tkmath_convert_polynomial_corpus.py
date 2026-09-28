"""Composite Bezier, composite polynomial and polynomial grid requests for the Convert corpus.

Usage: append_polynomial_requests(rng, requests) from tkmath_convert_corpus.py.
Coefficient arrays always hold every coefficient the native code reads through raw
pointers, and Bezier chains never contain an empty curve (the native degree
elevation of an empty pole array is undefined).
"""
from __future__ import annotations
import random


def line(op: str, values: list) -> str:
    """Format one request with round-trip float text; line('cv_bezier', [2, 1e-4])."""
    return op+' '+' '.join(str(value) for value in values)


def array(lower: int, values: list) -> list:
    """The "lower count values..." encoding; array(1, [0.5, 2.0])."""
    return [lower, len(values)]+values


def append_polynomial_requests(rng: random.Random, requests: list[str]) -> None:
    """Append the Bezier chains, composite curves and grids; append_polynomial_requests(rng, r)."""
    append_bezier_requests(rng, requests)
    append_comp_polynomial_requests(rng, requests)
    append_grid_requests(rng, requests)


def point(rng: random.Random, dim: int) -> list[float]:
    """Random pole in [-10, 10]^dim."""
    return [rng.uniform(-10, 10) for _ in range(dim)]


def chain(rng: random.Random, dim: int, case: int) -> list[list[list[float]]]:
    """Adjacent Bezier curves; the junction kind cycles through C1, antiparallel, C0,
    coincident poles, a tiny next chord (3D knot guard) and a single-pole curve."""
    curves = [[point(rng, dim) for _ in range(rng.randint(2, 5))]]
    for index in range(rng.randint(0, 4)):
        previous = curves[-1]
        start = previous[-1]
        tangent = [a-b for a, b in zip(previous[-1], previous[-2])]
        kind = (case+index) % 6
        scale = {0: rng.uniform(0.1, 3), 1: -rng.uniform(0.1, 3), 3: 0.0, 4: 1e-20}.get(kind)
        second = [s+scale*t for s, t in zip(start, tangent)] if scale is not None else point(rng, dim)
        rest = [point(rng, dim) for _ in range(rng.randint(0, 4))]
        curves.append([start, second]+rest)
    if case % 17 == 5:
        curves.append([curves[-1][-1]])
    return curves


def append_bezier_requests(rng: random.Random, requests: list[str]) -> None:
    """2D and 3D chains with angular tolerances around and at zero."""
    for case in range(160):
        dim = 2 if case % 2 else 3
        angular = [1e-4, 0.0, 1.0, 1e-12][case % 4]
        curves = chain(rng, dim, case)
        values = [dim, angular, len(curves)]
        for curve in curves:
            values += [[1, 1, 1, 1, 0, 5][case % 6], len(curve)]+[c for p in curve for c in p]
        requests.append(line('cv_bezier', values))
    # Junction at the origin with a 1e-20 chord: only the 3D conversion guards the ratio.
    for dim in (2, 3):
        zero = [0.0]*(dim-2)
        first = [-1.0, 0.0]+zero+[-0.5, 0.0]+zero+[0.0, 0.0]+zero
        second = [0.0, 0.0]+zero+[1e-20, 0.0]+zero+[1.0, 1.0]+zero
        requests.append(line('cv_bezier', [dim, 1e-4, 2, 1, 3]+first+[1, 3]+second))
    requests.append(line('cv_bezier', [3, 1e-4, 1, 1, 1, 1.0, 2.0, 3.0]))
    requests.append(line('cv_bezier', [2, 1e-4, 2, 1, 1, 0.0, 0.0, 1, 1, 0.0, 0.0]))
    requests.append(line('cv_bezier', [3, 1e-4, 0]))


def coefficients(rng: random.Random, count: int) -> list[float]:
    """Random canonical coefficients, sometimes exact small integers."""
    if rng.random() < 0.3:
        return [float(rng.randint(-3, 3)) for _ in range(count)]
    return [rng.uniform(-5, 5) for _ in range(count)]


def breakpoints(rng: random.Random, count: int) -> list[float]:
    """Increasing breakpoints starting anywhere in [-5, 5]."""
    values = [rng.uniform(-5, 5)]
    for _ in range(count-1):
        values.append(values[-1]+rng.uniform(0.2, 3))
    return values


def domains(rng: random.Random, count: int) -> list[float]:
    """Rows of polynomial domains [a, b], reversed now and then."""
    rows = []
    for _ in range(count):
        a = rng.uniform(-3, 3)
        rows += [a, a+rng.choice([1.0, rng.uniform(0.5, 4), -rng.uniform(0.5, 2)])]
    return rows


def append_comp_polynomial_requests(rng: random.Random, requests: list[str]) -> None:
    """Single spans, uniform and per-junction continuities, and the argument errors."""
    for case in range(80):
        # Degree 0 is left out: its Schoenberg points divide by the degree.
        dim, degree = rng.randint(1, 3), rng.randint(1, 6)
        max_degree = max(1, degree+rng.choice([0, 0, 1, 2, -1]))
        stride = (max(max_degree, degree)+1)*dim
        lower = [1, 0, 3][case % 3]
        requests.append(line('cv_comppoly', [2, dim, max_degree, degree]
                             + array(lower, coefficients(rng, stride))
                             + array(lower, domains(rng, 1)) + array(lower, breakpoints(rng, 2))))
    for case in range(120):
        curves, dim, max_degree = rng.randint(1, 4), rng.randint(1, 3), rng.randint(1, 6)
        counts = [rng.randint(1, max_degree+1) for _ in range(curves)]
        counts[rng.randrange(curves)] = rng.randint(2, max_degree+1)
        degree = max(counts)-1
        stride = (max(max_degree, degree)+1)*dim
        coeffs = coefficients(rng, curves*stride)
        grid = [1, curves, 1, 2]+domains(rng, curves)
        trues = breakpoints(rng, curves+1)
        continuity = rng.randint(0, degree+(1 if case % 11 == 0 else 0))
        mask = 1 << rng.randint(0, 3) if case % 13 == 0 else 0
        requests.append(line('cv_comppoly', [0, curves, continuity, dim, max_degree, mask]
                             + array(1, counts)+array(1, coeffs)+grid+array(1, trues)))
        continuities = [rng.randint(0, degree+(1 if case % 9 == 0 else 0)) for _ in range(curves)]
        requests.append(line('cv_comppoly', [1, curves, dim, max_degree]+array(1, continuities)
                             + array(1, counts)+array(1, coeffs)+grid+array(1, trues)))
    append_comp_polynomial_errors(requests)


def append_comp_polynomial_errors(requests: list[str]) -> None:
    """Non-positive counts, domain tables without two columns, short continuity arrays."""
    good = array(1, [2])+array(1, [1.0, 2.0])+[1, 1, 1, 2, 0.0, 1.0]+array(1, [0.0, 1.0])
    for nc, cont, dim, maxdeg in [(0, 0, 1, 1), (1, -1, 1, 1), (1, 0, 0, 1), (1, 0, 1, 0)]:
        requests.append(line('cv_comppoly', [0, nc, cont, dim, maxdeg, 0]+good))
    requests.append(line('cv_comppoly', [0, 1, 0, 1, 1, 0]+array(1, [2])+array(1, [1.0, 2.0])
                         + [1, 1, 1, 3, 0.0, 1.0, 2.0]+array(1, [0.0, 1.0])))
    two = array(1, [2, 2])+array(1, [1.0, 2.0, 0.0, 1.0])+[1, 2, 1, 2, 0.0, 1.0, 0.0, 1.0]+array(1, [0.0, 1.0, 2.0])
    requests.append(line('cv_comppoly', [1, 2, 1, 1]+array(1, [0])+two))
    requests.append(line('cv_comppoly', [1, 2, 1, 1]+array(2, [0])+two))
    requests.append(line('cv_comppoly', [1, 2, 1, 1]+array(1, [0, 3])+two))
    requests.append(line('cv_comppoly', [2, 1, 1, 1]+array(1, [1.0, 2.0])+array(1, [0.0, 1.0, 2.0])+array(1, [0.0, 1.0])))
    requests.append(line('cv_comppoly', [2, 1, 0, 1]+array(1, [1.0, 2.0])+array(1, [0.0, 1.0])+array(1, [0.0, 1.0])))
    requests.append(line('cv_comppoly', [2, 1, 1, 1]+array(1, [1.0, 2.0])+array(1, [0.0, 1.0])+array(1, [0.0])))


def append_grid_requests(rng: random.Random, requests: list[str]) -> None:
    """Single patches and grids of patches with continuity, plus the size errors."""
    for case in range(40):
        mu, mv = rng.randint(1, 4), rng.randint(1, 4)
        counts = [rng.randint(2, mu+1), rng.randint(2, mv+1)]
        coeffs = coefficients(rng, 3*(mu+1)*(mv+1))
        requests.append(line('cv_grid', [0, mu, mv]+array(1, counts)+array(1, coeffs)
                             + array(1, domains(rng, 1))+array(1, domains(rng, 1))))
    for case in range(60):
        nu, nv, cu, cv = rng.randint(1, 3), rng.randint(1, 3), rng.randint(0, 1), rng.randint(0, 1)
        mu, mv = rng.randint(1, 4), rng.randint(1, 4)
        ru, rv = max(mu, 2*cu+1), max(mv, 2*cv+1)
        patches = nu*nv
        # The first patch carries the full degrees so no multiplicity turns negative.
        counts = [ru+1, rv+1]+[c for _ in range(patches-1) for c in (rng.randint(1, ru+1), rng.randint(1, rv+1))]
        coeffs = coefficients(rng, 3*patches*(ru+1)*(rv+1))
        requests.append(line('cv_grid', [1, nu, nv, cu, cv, mu, mv, 1, patches, 1, 2]+counts
                             + array(1, coeffs)+array(1, domains(rng, 1))+array(1, domains(rng, 1))
                             + array(1, breakpoints(rng, nu+1))+array(1, breakpoints(rng, nv+1))))
    append_grid_errors(requests)


def append_grid_errors(requests: list[str]) -> None:
    """Wrong count or coefficient bounds, degrees above the maxima, coincident breakpoints."""
    coeffs = array(1, [0.0]*12)
    unit = array(1, [0.0, 1.0])
    requests.append(line('cv_grid', [0, 1, 1]+array(0, [2, 2])+coeffs+unit+unit))
    requests.append(line('cv_grid', [0, 1, 1]+array(1, [2, 2])+array(1, [0.0]*11)+unit+unit))
    requests.append(line('cv_grid', [0, 1, 1]+array(1, [3, 2])+coeffs+unit+unit))
    requests.append(line('cv_grid', [0, 1, 1]+array(1, [2, 3])+coeffs+unit+unit))
    grid = [1, 1, 1, 0, 0, 1, 1]
    requests.append(line('cv_grid', grid+[1, 2, 1, 2, 2, 2, 2, 2]+coeffs+unit+unit+unit+unit))
    requests.append(line('cv_grid', grid+[1, 1, 1, 2, 2, 2]+array(0, [0.0]*12)+unit+unit+unit+unit))
    requests.append(line('cv_grid', grid+[1, 1, 1, 2, 3, 2]+coeffs+unit+unit+unit+unit))
    requests.append(line('cv_grid', grid+[1, 1, 1, 2, 2, 2]+coeffs+unit+unit+array(1, [0.0, 0.0])+unit))
