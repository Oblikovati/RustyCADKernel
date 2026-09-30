"""Deterministic requests for the fillings of the GeomFill filling part
(tools/oracle/tkgeomalgo_probe_gfill_fill.cxx).

`gfill_generator` builds the ruled surface through profiled sections.
`gfill_scheme` runs `GeomFill_Stretch`, `GeomFill_Coons` and `GeomFill_Curved` (four
rows and two rows) on pole and weight rows of many sizes, with rows of mismatched
lengths (Standard_DomainError), rows of lower bound 0 (Standard_OutOfRange after a
partial grid) and a second non-rational `Init` that keeps the weights of the first.
`gfill_bezier` and `gfill_bspline` fill two, three and four boundary curves in the
three styles: chains given in every order and orientation, rational boundaries
(the weight rescaling of SetSameWeights), knot vectors of different ranges and
distributions, closed boundaries, curves that do not join, too few poles for the
Coons style, the two-pole curves of OCC27704, and the four ways two curves touch in
the curved style (including the reversal of the caller's first curve).
"""
from __future__ import annotations

from tkgeomalgo_corpus_gfill import SECTIONS, bezier, bspline, curves

Point = tuple[float, float, float]


def _line(op: str, values: list[float]) -> str:
    return op + ' ' + ' '.join(repr(float(v)) for v in values)


def _generator_requests(requests: list[str]) -> None:
    for name, items in SECTIONS.items():
        for ptol in (1e-9, 1e-6):
            requests.append(_line('gfill_generator', [ptol, *curves(items)]))


def _row(lower: int, points: list[Point]) -> list[float]:
    values: list[float] = [lower, len(points)]
    for p in points:
        values += list(p)
    return values


def _wrow(lower: int, weights: list[float]) -> list[float]:
    return [lower, len(weights), *weights]


def _patch_rows(nu: int, nv: int) -> list[list[Point]]:
    """The four boundary rows of a warped patch: P1 and P3 of `nu` points (V = 0 and
    V = 1), P2 and P4 of `nv` points (U = 1 and U = 0), sharing the corners."""
    def at(u: float, v: float) -> Point:
        return (3.0 * u + 0.2 * v * v, 2.0 * v - 0.3 * u * v, 0.5 * u * (1 - v) + 0.7 * u * u * v - 0.4 * v)
    su = [i / max(1, nu - 1) for i in range(nu)]
    sv = [j / max(1, nv - 1) for j in range(nv)]
    return [[at(u, 0.0) for u in su], [at(1.0, v) for v in sv], [at(u, 1.0) for u in su], [at(0.0, v) for v in sv]]


def _weights(n: int, seed: float) -> list[float]:
    return [1.0 + 0.3 * ((seed * (k + 1)) % 1.0) for k in range(n)]


def _scheme(scheme: int, rows: list[list[Point]], weights: list[list[float]] | None = None,
            lowers: list[int] | None = None, again: int = 0) -> str:
    lowers = lowers or [1] * len(rows)
    values: list[float] = [scheme, int(weights is not None)]
    for lower, row in zip(lowers, rows):
        values += _row(lower, row)
    for w in weights or []:
        values += _wrow(1, w)
    return _line('gfill_scheme', values + [again])


def _scheme_requests(requests: list[str]) -> None:
    for nu, nv in ((2, 2), (3, 3), (4, 4), (5, 3), (3, 6), (6, 7), (8, 5)):
        rows = _patch_rows(nu, nv)
        weights = [_weights(len(r), 0.37 + 0.11 * k) for k, r in enumerate(rows)]
        for scheme in (0, 1, 2):
            requests.append(_scheme(scheme, rows))
            requests.append(_scheme(scheme, rows, weights, again=1))
        requests.append(_scheme(3, [rows[0], rows[1]]))
        requests.append(_scheme(3, [rows[0], rows[1]], [weights[0], weights[1]], again=1))
    rows = _patch_rows(1, 4)
    for scheme in (0, 2):
        requests.append(_scheme(scheme, rows))
    rows = _patch_rows(4, 5)
    weights = [_weights(len(r), 0.21 * (k + 1)) for k, r in enumerate(rows)]
    for scheme in (0, 1, 2):
        requests.append(_scheme(scheme, [rows[0], rows[1], rows[2][:3], rows[3]]))
        requests.append(_scheme(scheme, rows, [weights[0], weights[1], weights[2], weights[3][:4]]))
        requests.append(_scheme(scheme, rows, [weights[0][:3], weights[1], weights[2][:3], weights[3]]))
        requests.append(_scheme(scheme, rows, lowers=[0, 1, 1, 1]))
        requests.append(_scheme(scheme, rows, lowers=[1, 1, 1, 0]))
        requests.append(_scheme(scheme, rows, weights, lowers=[1, 2, 1, 1]))
    requests.append(_scheme(3, [rows[0], rows[1]], lowers=[0, 1]))
    requests.append(_scheme(3, [rows[0], rows[1]], [weights[0][:2], weights[1]]))


# The corners of a warped quadrilateral and the four chained boundaries A->B->C->D->A.
CORNERS: list[Point] = [(0.0, 0.0, 0.0), (3.0, 0.0, 0.5), (3.2, 2.0, 0.0), (-0.2, 2.1, -0.5)]


def _edge(a: Point, b: Point, degree: int, lift: float) -> list[Point]:
    points = []
    for k in range(degree + 1):
        t = k / degree
        bump = lift * t * (1 - t) * 4
        points.append((a[0] + t * (b[0] - a[0]), a[1] + t * (b[1] - a[1]) + 0.3 * bump, a[2] + t * (b[2] - a[2]) + bump))
    return points


def _reverse(points: list[Point]) -> list[Point]:
    return list(reversed(points))


def _bezier_chain(degrees: list[int], rational: list[bool]) -> list[list[float]]:
    edges = []
    for k in range(4):
        pts = _edge(CORNERS[k], CORNERS[(k + 1) % 4], degrees[k], 0.4 + 0.1 * k)
        w = [1.0 + 0.25 * ((j * (k + 2)) % 3) for j in range(len(pts))] if rational[k] else None
        if w:
            w[0], w[-1] = 1.0 + 0.1 * k, 1.2 - 0.05 * k
        edges.append((pts, w))
    return [bezier(p, w) for p, w in edges]


def _bezier(style: int, items: list[list[float]]) -> str:
    return _line('gfill_bezier', [style, len(items), *[v for item in items for v in item]])


def _bezier_requests(requests: list[str]) -> None:
    plans = [([1, 1, 1, 1], [False] * 4), ([2, 3, 2, 1], [False] * 4), ([3, 2, 4, 2], [False] * 4),
             ([2, 2, 2, 2], [True, False, False, False]), ([3, 2, 3, 2], [True, True, True, True]),
             ([2, 3, 1, 2], [False, True, False, True])]
    for degrees, rational in plans:
        chain = _bezier_chain(degrees, rational)
        for style in (0, 1, 2):
            requests.append(_bezier(style, chain))
            requests.append(_bezier(style, [chain[0], chain[2], chain[1], chain[3]]))
            requests.append(_bezier(style, chain[:3]))
            requests.append(_bezier(style, [chain[0], chain[2]]))
    edge = [_edge(CORNERS[k], CORNERS[(k + 1) % 4], 2, 0.3) for k in range(4)]
    flipped = [bezier(edge[0]), bezier(_reverse(edge[1])), bezier(edge[2]), bezier(_reverse(edge[3]))]
    for style in (0, 1, 2):
        requests.append(_bezier(style, flipped))
        requests.append(_bezier(style, [flipped[0], flipped[3], flipped[1], flipped[2]]))
        broken = [bezier(edge[0]), bezier(edge[1]), bezier([(9, 9, 9), (10, 9, 9), (11, 9, 9)]), bezier(edge[3])]
        requests.append(_bezier(style, broken))
    base = _edge((0, 0, 0), (2, 0, 0), 3, 0.5)
    other = _edge((0, 0, 0), (0, 2, 0.3), 2, 0.2)
    for style in (0, 1, 2):
        requests.append(_bezier(style, [bezier(base), bezier(other)]))
        requests.append(_bezier(style, [bezier(base), bezier(_reverse(other))]))
        requests.append(_bezier(style, [bezier(_reverse(base)), bezier(other)]))
        requests.append(_bezier(style, [bezier(_reverse(base)), bezier(_reverse(other))]))
        requests.append(_bezier(style, [bezier(base, [1, 2, 2, 1]), bezier(other, [1, 0.5, 1])]))
        requests.append(_bezier(style, [bezier(base), bezier(_edge((5, 5, 5), (6, 5, 5), 1, 0))]))


def _bspline_edge(a: Point, b: Point, degree: int, inner: list[float], lift: float,
                  span: tuple[float, float] = (0.0, 1.0), weights: bool = False) -> list[float]:
    knots = [span[0], *[span[0] + t * (span[1] - span[0]) for t in inner], span[1]]
    mults = [degree + 1, *[1] * len(inner), degree + 1]
    n = sum(mults) - degree - 1
    pts = _edge(a, b, n - 1, lift) if n > 1 else [a]
    w = [1.0 + 0.2 * ((k * 7) % 4) for k in range(n)] if weights else None
    return bspline(degree, knots, mults, pts, w)


def _bspline(style: int, items: list[list[float]]) -> str:
    return _line('gfill_bspline', [style, len(items), *[v for item in items for v in item]])


def _bspline_chain(specs: list[tuple]) -> list[list[float]]:
    return [_bspline_edge(CORNERS[k], CORNERS[(k + 1) % 4], *spec) for k, spec in enumerate(specs)]


def _bspline_requests(requests: list[str]) -> None:
    plans = [
        [(3, [], 0.4), (3, [], 0.3), (3, [], 0.2), (3, [], 0.5)],
        [(2, [0.5], 0.4), (3, [0.3, 0.7], 0.3), (2, [0.25], 0.2), (3, [0.5], 0.5)],
        [(3, [0.4], 0.4, (0.0, 2.0)), (2, [0.5], 0.3, (1.0, 1.5)), (3, [0.6], 0.2), (2, [], 0.5, (0.0, 4.0))],
        [(2, [0.5], 0.4, (0.0, 1.0), True), (2, [0.5], 0.3), (3, [0.2, 0.8], 0.2, (0.0, 1.0), True), (2, [0.3], 0.5)],
        [(1, [], 0.0), (1, [], 0.0), (1, [], 0.0), (1, [], 0.0)],
        [(3, [0.3, 0.6], 0.4, (0.0, 3.0), True), (3, [0.5], 0.3, (0.0, 3.0), True),
         (3, [0.5], 0.2, (0.0, 3.0), True), (3, [0.2, 0.4, 0.8], 0.5, (0.0, 3.0), True)],
    ]
    for plan in plans:
        chain = _bspline_chain(plan)
        for style in (0, 1, 2):
            requests.append(_bspline(style, chain))
            requests.append(_bspline(style, [chain[0], chain[3], chain[1], chain[2]]))
            requests.append(_bspline(style, chain[:3]))
            requests.append(_bspline(style, [chain[0], chain[2]]))
    _bspline_special_requests(requests)


def _two_pole(a: Point, b: Point, last_knot: float) -> list[float]:
    return bspline(1, [0.0, last_knot], [2, 2], [a, b], [1.0, 1.0])


def _bspline_special_requests(requests: list[str]) -> None:
    # OCC27704: two-pole curves, the fourth not joining; the Coons style raises.
    p1 = (-24033.3957701043, -6337.90755953146, -16577.8188547128)
    p2 = (-23933.3957701044, -6337.90755953146, -16577.8362547128)
    p3 = (-24033.3957701043, -6371.01755953146, -16577.8188547128)
    p4 = (-23933.3957701044, -6371.01755953146, -16577.8362547128)
    occ27704 = [_two_pole(p1, p2, 100.000001513789), _two_pole(p2, p1, 100.000001513789),
                _two_pole(p3, p1, 33.1099999999979), _two_pole(p3, p4, 100.000001513789)]
    for style in (0, 1, 2):
        requests.append(_bspline(style, occ27704))
    # A closed boundary chained before an open one at the same point.
    loop = bspline(2, [0, 1, 2], [3, 1, 3], [(0, 0, 0), (1, 1, 0), (-1, 1, 0.2), (0, 0, 0)])
    edges = [_bspline_edge((0, 0, 0), (2, 0, 0), 2, [], 0.2), _bspline_edge((2, 0, 0), (0, 0, 0), 2, [0.5], 0.4)]
    for style in (0, 2):
        requests.append(_bspline(style, [edges[0], edges[1], loop, loop]))
        requests.append(_bspline(style, [edges[0], loop, edges[1], loop]))
    # Too few poles for the Coons style, and three curves on a short middle range.
    chain = _bspline_chain([(1, [], 0.0), (2, [], 0.3), (1, [], 0.0), (2, [], 0.3)])
    requests.append(_bspline(1, chain))
    requests.append(_bspline(1, chain[:3]))
    # Two curves: the four joins of the curved style, rational and periodic rulings,
    # different ranges, and curves that do not touch.
    base = _bspline_edge((0, 0, 0), (2, 0, 0), 3, [0.5], 0.5, (0.0, 2.0))
    other = _bspline_edge((0, 0, 0), (0, 2, 0.3), 2, [0.3, 0.6], 0.2, (1.0, 3.5))
    rbase = _bspline_edge((2, 0, 0), (0, 0, 0), 3, [0.5], 0.5, (0.0, 2.0))
    rother = _bspline_edge((0, 2, 0.3), (0, 0, 0), 2, [0.3, 0.6], 0.2, (1.0, 3.5))
    far = _bspline_edge((5, 5, 5), (6, 5, 5), 1, [], 0.0)
    weighted = _bspline_edge((0, 0, 0), (0, 2, 0.3), 2, [0.3], 0.2, (0.0, 1.0), True)
    periodic = bspline(2, [0, 1, 2, 3, 4], [1, 1, 1, 1, 1], [(1, 0, 0), (0, 1, 0), (-1, 0, 0), (0, -1, 0)],
                       [1, 2, 1, 2], periodic=True)
    periodic2 = bspline(2, [0, 1, 2, 3, 4], [1, 1, 1, 1, 1], [(2, 0, 1), (0, 2, 1), (-2, 0, 1), (0, -2, 1)],
                        periodic=True)
    for style in (0, 1, 2):
        for pair in ([base, other], [base, rother], [rbase, other], [rbase, rother], [base, far],
                     [base, weighted], [periodic, periodic2]):
            requests.append(_bspline(style, pair))


def append_gfill_fill_requests(requests: list[str]) -> None:
    """Append the GeomFill filling requests: generator, schemes, curve fillings."""
    _generator_requests(requests)
    _scheme_requests(requests)
    _bezier_requests(requests)
    _bspline_requests(requests)
