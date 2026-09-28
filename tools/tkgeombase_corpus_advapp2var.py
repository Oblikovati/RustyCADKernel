"""AdvApp2Var requests of the TKGeomBase numeric oracle (tkgeombase_probe_advapp2var*.cxx).

The MathBase requests feed each public kernel normal, boundary and error inputs;
the arrays come from a fixed linear congruential generator, printed with repr so
both probes parse the same doubles. The ApproxF2var requests live in
tkgeombase_corpus_advapp2var_approx.py, the class requests in
tkgeombase_corpus_advapp2var_class.py.
"""
from __future__ import annotations


class Values:
    """Deterministic pseudo-random doubles in [-1, 1); Values(7).take(3)."""

    def __init__(self, seed: int) -> None:
        self.state = seed

    def next(self) -> float:
        """The next value of the generator; Values(1).next()."""
        self.state = (self.state * 6364136223846793005 + 1442695040888963407) % 2**64
        return ((self.state >> 11) / 2**53) * 2.0 - 1.0

    def take(self, count: int, scale: float = 1.0) -> list[float]:
        """`count` values times `scale`; Values(1).take(4, 10.0)."""
        return [self.next() * scale for _ in range(count)]


def line(op: str, *fields: object) -> str:
    """One request: numbers printed exactly; line('aa_norms', 1, [0.5])."""
    tokens = [op]
    for field in fields:
        items = field if isinstance(field, (list, tuple)) else [field]
        tokens += [repr(float(item)) if isinstance(item, float) else str(item) for item in items]
    return ' '.join(tokens)


def append_advapp2var_requests(requests: list[str]) -> None:
    """Append every AdvApp2Var request family in a fixed order."""
    append_math_requests(requests)
    approx = __import__('tkgeombase_corpus_advapp2var_approx')
    approx.append_f2var_requests(requests)
    __import__('tkgeombase_corpus_advapp2var_class').append_class_requests(requests)


def append_math_requests(requests: list[str]) -> None:
    """Append the AdvApp2Var_MathBase requests in a fixed order."""
    rng = Values(20260928)
    for n in [1, 2, 3, 5]:
        requests.append(line('aa_norms', n, rng.take(n, 3.0), rng.take(n), 1.7, n - 2))
    requests.append(line('aa_norms', 3, [1e-300, 2e-300, 0.5], [1.0, 1.0, 1.0], 2.0, -3))
    for ndim, ncofmx, ncoeff in [(1, 6, 6), (3, 8, 5), (2, 4, 1), (2, 5, 2)]:
        requests.append(line('aa_mmapcmp', ndim, ncofmx, ncoeff, rng.take(ncofmx * ndim)))
    for iordre, ndimen, ncoeff in [(0, 1, 4), (1, 3, 6), (2, 2, 7), (3, 1, 5), (-1, 1, 3), (2, 1, 0)]:
        requests.append(line('aa_mmdrc11', iordre, ndimen, ncoeff, rng.take(max(ncoeff, 0) * ndimen)))
    for nd, nu, nv, ndx, nux in [(3, 2, 2, 3, 4), (2, 3, 2, 3, 3), (3, 4, 3, 3, 4), (1, 2, 3, 2, 5)]:
        for inplace in [0, 1]:
            requests.append(line('aa_mmfmca8', nd, nu, nv, ndx, nux, inplace, rng.take(nd * nu * nv)))
    for ndx, nux, nd, nu, nv in [(3, 4, 3, 2, 2), (3, 3, 2, 3, 2), (3, 4, 3, 4, 3), (2, 5, 1, 2, 3)]:
        for inplace in [0, 1]:
            requests.append(line('aa_mmfmca9', ndx, nux, nd, nu, nv, inplace, rng.take(ndx * nux * nv)))
    for isenmsc, ndx, n1, nc, n2, nd in [(1, 3, 4, 3, 5, 2), (-1, 3, 4, 4, 4, 3), (1, 1, 4, 4, 4, 1),
                                         (-1, 1, 3, 2, 3, 1), (0, 2, 3, 3, 3, 2), (1, 2, 3, 4, 5, 2)]:
        requests.append(line('aa_mmfmcb5', isenmsc, ndx, n1, nc, n2, nd, rng.take(ndx * n1), rng.take(n2 * nd)))
    for m1, i1, j1, m2 in [(3, 3, 4, 4), (4, 2, 3, 3), (2, 3, 2, 2), (3, 2, 5, 4)]:
        requests.append(line('aa_mmfmtb1', m1, i1, j1, m2, rng.take(m1 * j1)))
    append_mmfmcar(requests, rng)
    append_curve_requests(requests, rng)
    append_basis_requests(requests, rng)
    append_system_requests(requests, rng)


def append_mmfmcar(requests: list[str], rng: Values) -> None:
    """mmfmcar_ over identity, U, V and UV restrictions, reversals and errors."""
    ranges = [(0.0, 1.0, 0.0, 1.0), (0.25, 0.75, 0.0, 1.0), (0.0, 1.0, 0.1, 0.6), (0.2, 0.9, 0.3, 0.8),
              (1.0, 0.0, 1.0, 0.0), (-0.5, 1.5, 0.0, 1.0)]
    for nd, nx, nu, nv in [(3, 4, 4, 3), (3, 6, 4, 5), (2, 5, 5, 4), (1, 3, 2, 2)]:
        for u1, u2, v1, v2 in ranges:
            requests.append(line('aa_mmfmcar', nd, nx, nu, nv, u1, u2, v1, v2, rng.take(nd * nx * nv)))
    requests.append(line('aa_mmfmcar', 3, 2, 4, 2, 0.0, 0.5, 0.0, 1.0, rng.take(12)))
    requests.append(line('aa_mmfmcar', 1, 3, 3, 62, 0.0, 0.5, 0.0, 1.0, rng.take(186)))


def append_curve_requests(requests: list[str], rng: Values) -> None:
    """Curve length, sorting, derivatives, constraints and truncation requests."""
    for ndx, nd, nc, t0, t1, eps in [(3, 3, 4, 0.0, 1.0, 1e-9), (2, 2, 6, -1.0, 1.0, 1e-12), (3, 2, 3, 0.5, 0.5, 1e-9),
                                     (1, 1, 9, 0.0, 2.0, 1e-15), (3, 0, 2, 0.0, 1.0, 1e-9), (3, 3, 44, 0.0, 1.0, 1e-6)]:
        requests.append(line('aa_mmcglc1', ndx, nd, nc, t0, t1, eps, rng.take(ndx * nc)))
    for nc, nl, key in [(5, 3, 2), (1, 2, 1), (8, 1, 1), (6, 4, 4)]:
        requests.append(line('aa_mmbulld', nc, nl, key, rng.take(nc * nl)))
    for n, is_, key in [(5, 3, 2), (1, 2, 1), (40, 1, 1), (25, 3, 3)]:
        requests.append(line('aa_mvsheld', n, is_, key, rng.take(n * is_)))
    for nd, nc, ideriv in [(3, 5, 1), (2, 6, 3), (1, 4, 4), (3, 7, 0)]:
        requests.append(line('aa_mmcdriv', nd, nc, ideriv, rng.take(nd * nc)))
    for nc, nd, ideriv, t in [(5, 3, 1, 0.3), (6, 2, 3, -0.7), (4, 1, 4, 0.2), (7, 3, 0, 1.0), (30, 1, 22, 0.1)]:
        requests.append(line('aa_mmdrvck', nc, nd, ideriv, t, rng.take(nd * nc)))
    for nd, nx, nv in [(3, 4, 1), (2, 6, 2), (1, 2, 0), (3, 8, 3)]:
        requests.append(line('aa_mmcvctx', nd, nx, nv, rng.take(nd * 2 * (nv + 1))))
    for n in [101, 150]:
        requests.append(line('aa_mdsptpt', n, rng.take(n, 5.0), rng.take(n, 5.0)))
    for nx, nd, nc, io, nn in [(8, 3, 8, -1, 3), (10, 2, 10, 0, 4), (12, 3, 11, 1, 6), (14, 1, 14, 2, 8),
                               (8, 1, 8, 3, 2), (59, 1, 59, 0, 2)]:
        requests.append(line('aa_mmaperx', nx, nd, nc, io, nn, rng.take(nx * nd, 1e-3)))
    for nx, nd, nc, eps, io in [(8, 3, 8, 1e-4, -1), (10, 2, 10, 1e-3, 0), (12, 3, 11, 1e-5, 1), (14, 1, 14, 1e-2, 2),
                                (6, 1, 6, 10.0, 0), (6, 2, 6, 1e-12, 2)]:
        requests.append(line('aa_mmtrpjj', nx, nd, nc, eps, io, rng.take(nx * nd, 1e-3)))
    for nf, k, xd, xf, kind in [(1, 1, 0.0, 3.0, 0), (3, 4, -1.0, 2.0, 0), (5, 10, 0.2, 0.9, 0), (2, 11, 0.0, 1.0, 1),
                                (2, 0, 0.0, 1.0, 0), (3, 3, 0.0, 1.0, 2)]:
        requests.append(line('aa_mmgaus1', nf, k, xd, xf, kind))


def append_basis_requests(requests: list[str], rng: Values) -> None:
    """Hermite-Jacobi, Jacobi, evaluation and reparametrisation requests."""
    for nd, nk, oc, nl, ncf in [(3, 1, 0, 6, [6]), (2, 2, 1, 8, [8, 6]), (1, 3, 2, 10, [10, 7, 9]), (3, 1, -1, 5, [5]),
                                (1, 1, 3, 6, [6]), (1, 1, 1, 22, [22]), (2, 2, 1, 21, [21, 22])]:
        cuts = [0.0] + sorted(abs(v) * 3 for v in rng.take(nk))
        requests.append(line('aa_mmhjcan', nd, nk, oc, nl, ncf, cuts, rng.take(nd * nl * nk)))
    table = [1.0, 2.0, 1.0, 3.0, 2.0, 2.0] + [0.0] * 4
    lines = [[1.0, 2.5], [0.5, 9.0], [1.0, 2.0], [2.0, 2.0], [4.0, -1.0]]
    requests.append(line('aa_mminltt', 2, 5, 2, 3, table, len(lines), [v for pair in lines for v in pair]))
    requests.append(line('aa_mminltt', 3, 2, 2, 2, [1.0] * 6, 1, [0.0, 0.0]))
    for ideriv, ndeg in [(-1, 0), (0, 5), (1, 8), (2, 12), (0, 29), (-1, 30)]:
        requests.append(line('aa_mmjacan', ideriv, ndeg, rng.take((ndeg // 2 + 1) * 2)))
    for nc, nd, ider in [(4, 1, 0), (7, 3, 1), (10, 2, 2), (1, 1, -1)]:
        requests.append(line('aa_mmjaccv', nc, nd, ider, rng.take(nc * nd)))
    for t, io, nc, nv in [(0.3, -1, 5, 0), (0.5, 0, 6, 1), (-0.7, 1, 9, 2), (0.9, 2, 14, 3), (1.0, 2, 20, 3),
                          (0.1, 3, 8, 1), (0.1, 1, 21, 1), (0.1, 1, 8, 4)]:
        requests.append(line('aa_mmpobas', t, io, nc, nv))
    for nx, nc, nd, t in [(3, 5, 3, 0.4), (2, 4, 2, -0.3), (4, 6, 3, 0.0), (4, 6, 2, 1.0), (4, 6, 1, 0.7), (3, 0, 3, 0.5),
                          (3, 1, 3, 0.2), (2, 1, 2, 1.0), (3, 4, 3, 1.0), (2, 5, 2, 0.0)]:
        requests.append(line('aa_mmpocrb', nx, nc, nd, t, rng.take(nx * max(nc, 0))))
    for nx, nd, ng, t in [(5, 2, 5, 0.3), (6, 1, 3, -1.2), (4, 3, 0, 0.5)]:
        requests.append(line('aa_mmmpocur', nx, nd, ng, t, rng.take(nx * nd)))
    for u0, u1 in [(0.0, 2.0), (-1.0, 1.0), (0.0, 1.0), (0.5, 0.5), (-2.0, 3.0)]:
        requests.append(line('aa_mmarcin', 3, 3, 5, u0, u1, rng.take(15)))
    requests.append(line('aa_mmarcin', 4, 2, 6, 0.25, 1.5, rng.take(12)))
    requests.append(line('aa_mmarcin', 3, 3, 62, 0.0, 2.0, rng.take(186)))
    requests.append(line('aa_mmcvinv', 4, 5, 3, rng.take(15)))


def append_system_requests(requests: list[str], rng: Values) -> None:
    """Profile systems, root tables, searches, vectors and dense solves."""
    requests.append(line('aa_mmposui', 3, [0, 1, 1, 3, 1, 5]))
    requests.append(line('aa_mmposui', 4, [0, 1, 1, 3, 2, 6, 0, 7]))
    for h, g, bw in [(110, 0, 2), (110, 101, 2), (120, 105, 3)]:
        requests.append(line('aa_mmresol', h, g, bw))
    for n in [0, 1, 2, 5, 12, 61]:
        requests.append(line('aa_mmrtptt', n))
    table = [0.0, 0.5, 1.25, 2.0, 3.5]
    for t in [0.0, 0.7, 1.25, 3.5, 3.4999999999, -0.1]:
        requests.append(line('aa_mmsrre2', t, 5, table, 1e-6))
    requests.append(line('aa_mmsrre2', 0.5, 1, [0.5], 1e-6))
    for n, eps, v in [(3, 0.0, [0.0, -2.0, 1e-12]), (3, 0.0, [1.0, 2.0, 3.0]), (2, 1e-3, [1e-4, 0.0]), (3, 0.0, [0.0] * 3),
                      (4, 0.0, [3.0, 0.0, 0.0, 1e-20])]:
        requests.append(line('aa_mmunivt', n, eps, v))
    for n, v in [(3, [1.0, 0.0, 0.0]), (2, [0.0, 1.0]), (3, [1.0, 1.0, 1.0]), (3, [0.0] * 3), (1, [1.0]), (4, [1.0] * 4)]:
        requests.append(line('aa_mmvncol', n, v))
    for nx, no, nd in [(3, 3, 2), (6, 4, 3), (2, 2, 1)]:
        requests.append(line('aa_mmmrslwd', nx, no, nd, 1e-10, rng.take(nx * no), rng.take(nx * nd)))
    requests.append(line('aa_mmmrslwd', 2, 2, 1, 1e-10, [1.0, 2.0, 2.0, 4.0], [1.0, 1.0]))
    requests.append(line('aa_mmwprcs', 1e-6, 1e-8, 1e-5, 1e-4, 8, 40))
    requests.append(line('aa_mmtrpjj', 8, 1, 8, 1e-2, 0, [1.0, 5e-7, 0.0, 0.0, 1e-9, 1e-9, 1e-9, 1e-9]))
    requests.append(line('aa_mmwprcs', 1e-9, 1e-8, 1e-9, 1e-4, 8, 40))
