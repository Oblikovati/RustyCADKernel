"""AdvApp2Var_ApproxF2var and class requests of the TKGeomBase numeric oracle.

The kernels run on the test functions of tkgeombase_probe_advapp2var_eval.cxx (ids 0..3;
id + 10 fails beyond a constant parameter of 0.5), with the point counts and Jacobi
degrees the approximation uses, odd and even counts, every continuity order, and the
error paths. The arrays come from the generator of tkgeombase_corpus_advapp2var.py.
"""
from __future__ import annotations
from tkgeombase_corpus_advapp2var import Values, line


def append_f2var_requests(requests: list[str]) -> None:
    """Append the AdvApp2Var_ApproxF2var requests in a fixed order."""
    rng = Values(29092026)
    for order in [0, 1, 2, 3, -1]:
        requests.append(line('aa_mma1her', order))
    for nu, nv in [(8, 10), (21, 30), (52, 50), (2, 3), (61, 60)]:
        requests.append(line('aa_mma2roo', nu, nv))
    for ndg, order in [(20, -1), (20, 0), (30, 1), (58, 2), (10, 3)]:
        requests.append(line('aa_mma2jmx', ndg, order))
    for ndg, npt, order in [(7, 8, -1), (9, 10, 0), (14, 15, 1), (19, 20, 2), (24, 25, 0), (29, 30, 1), (39, 40, 2),
                            (49, 50, -1), (60, 61, 0), (30, 61, 2), (60, 61, -1), (7, 9, 0), (7, 8, 3), (8, 8, 0), (1, 8, 0)]:
        requests.append(line('aa_mmapptt', ndg, npt, order))
    append_constraint_requests(requests, rng)
    append_square_requests(requests, rng)


def append_constraint_requests(requests: list[str], rng: Values) -> None:
    """mma2ac1_, mma2ac2_, mma2ac3_ and mma2can_ on random patches."""
    for nd, mu, mv, iu, iv in [(3, 5, 5, 0, 0), (3, 7, 6, 1, 1), (3, 9, 9, 2, 1), (2, 6, 8, 0, 2)]:
        nc = nd * (iu + 2) * (iv + 2)
        requests.append(line('aa_mma2ac1', nd, mu, mv, iu, iv, *[rng.take(nc) for _ in range(4)],
                             rng.take((mu + 1) * (mv + 1) * nd)))
    for op in ['aa_mma2ac2', 'aa_mma2ac3']:
        for nd, mu, mv, order, nl, n1, n2 in [(3, 6, 5, 1, 7, [5, 6], [6, 4]), (3, 8, 8, 0, 9, [9], [3]),
                                               (3, 9, 9, 2, 8, [8, 6, 1], [2, 8, 5])]:
            size = nl * nd * (order + 1)
            requests.append(line(op, nd, mu, mv, order, nl, n1, rng.take(size), n2, rng.take(size),
                                 rng.take((mu + 1) * (mv + 1) * nd)))
    for mu, mv, nd, iu, iv, nu, nv in [(9, 9, 3, 1, 1, 6, 5), (8, 8, 3, 0, 0, 8, 8), (10, 7, 3, 2, 1, 7, 4),
                                       (6, 6, 3, -1, -1, 3, 3), (5, 5, 3, 1, 1, 6, 5), (6, 6, 3, 3, 0, 4, 4)]:
        requests.append(line('aa_mma2can', mu, mv, nd, iu, iv, nu, nv, rng.take(nu * nv * nd)))


def append_square_requests(requests: list[str], rng: Values) -> None:
    """mma2cdi_, mma2ds1_, mma2ce1_, mma2fnc_ and mma2fx6_."""
    for nd, nu, nv, iu, iv in [(3, 8, 10, 0, 0), (3, 10, 10, 1, 1), (3, 21, 30, 2, 1), (3, 9, 11, 1, 0), (3, 20, 20, -1, -1),
                               (3, 12, 12, -1, 1), (3, 12, 13, 2, -1), (3, 8, 8, 3, 0)]:
        nc, su, sv = nd * (iu + 2) * (iv + 2), (nv // 2 + 1) * nd * (iu + 1), (nu // 2 + 1) * nd * (iv + 1)
        arrays = [rng.take(nc) for _ in range(4)] + [rng.take(su) for _ in range(4)] + [rng.take(sv) for _ in range(4)]
        requests.append(line('aa_mma2cdi', nd, nu, nv, iu, iv, *arrays))
    for nu, nv, iso, domain, fid in [(8, 10, 2, (0.0, 1.0, 0.0, 1.0), 0), (10, 10, 1, (0.0, 1.0, 0.0, 1.0), 1),
                                     (21, 20, 1, (-1.0, 2.0, 0.5, 1.5), 2), (20, 21, 2, (0.2, 0.9, -0.5, 0.5), 3),
                                     (9, 9, 3, (0.0, 1.0, 0.0, 1.0), 1), (10, 10, 2, (0.0, 1.0, 0.0, 1.0), 10),
                                     (10, 10, 1, (0.0, 1.0, 0.0, 1.0), 10)]:
        requests.append(line('aa_mma2ds1', nu, nv, iso, list(domain), fid))
    unit = [0.0, 1.0, 0.0, 1.0]
    for fields in [(0, 5, 1, 1, 20, 20, 15, 15, 12, 12, 3, 3, 1e-6, unit), (1, 0, 0, 0, 10, 10, 9, 9, 7, 7, 1, 1, 1e-3, unit),
                   (2, 3, 1, 1, 30, 30, 21, 21, 14, 14, 3, 3, 1e-5, unit), (3, 4, 2, 2, 40, 40, 29, 29, 20, 20, 5, 5, 1e-7, unit),
                   (1, 1, 2, 0, 25, 15, 20, 12, 10, 8, 5, 1, 1e-9, [-2.0, 1.0, -1.0, 1.0]),
                   (2, 2, 0, 1, 15, 25, 12, 20, 8, 10, 1, 3, 1e-9, [0.0, 3.0, 0.0, 2.0]),
                   (1, 0, 1, 1, 20, 20, 15, 15, 6, 6, 3, 3, 1e-12, [-2.0, 2.0, -2.0, 2.0]),
                   (0, 6, 1, 1, 20, 20, 15, 15, 12, 12, 3, 3, 1e-6, unit), (0, 5, 1, 1, 20, 20, 15, 15, 12, 12, 13, 3, 1e-6, unit),
                   (3, 5, -1, -1, 10, 8, 9, 7, 6, 5, 1, 1, 1e-4, [0.5, 1.5, -1.0, 0.0]),
                   (0, 5, -1, -1, 20, 20, 15, 15, 12, 12, 1, 1, 1e-6, unit), (2, 3, -1, -1, 30, 30, 21, 21, 14, 14, 1, 1, 1e-3, unit),
                   (2, 4, -1, -1, 30, 30, 21, 21, 4, 12, 1, 1, 1e-4, unit), (2, 1, -1, -1, 30, 30, 21, 21, 12, 4, 1, 1, 1e-4, unit),
                   (1, 2, -1, -1, 25, 25, 20, 20, 6, 6, 1, 1, 1e-5, [-1.0, 1.0, -1.0, 1.0])]:
        fid, numdec, iu, iv, nu, nv, ju, jv, gu, gv, mu, mv, eps, domain = fields
        requests.append(line('aa_mma2ce1', fid, numdec, iu, iv, nu, nv, ju, jv, gu, gv, mu, mv, eps, domain))
    for fields in [(0, 1, 0.3, unit, 1, 0, 20, 14, 12, 1, 1e-6), (1, 2, 0.7, [-1.0, 1.0, -0.5, 0.5], 2, 1, 30, 20, 15, 3, 1e-8),
                   (2, 1, 0.1, [0.0, 2.0, 0.0, 1.0], 0, 2, 40, 30, 21, 4, 1e-4), (3, 2, 0.5, unit, -1, 0, 10, 7, 8, 2, 1e-3),
                   (2, 1, 0.9, unit, 1, 0, 8, 5, 5, 3, 1e-12), (0, 1, 0.2, unit, 1, 0, 15, 12, 10, 2, 1e-6),
                   (1, 2, 0.4, unit, 2, 2, 25, 20, 16, 2, 1e-7), (1, 1, 0.4, unit, 2, 0, 61, 50, 30, 1, 1e-10),
                   (10, 1, 0.9, unit, 1, 0, 20, 14, 12, 1, 1e-6), (0, 3, 0.3, unit, 1, 0, 20, 14, 12, 1, 1e-6),
                   (0, 1, 0.3, [0.0, 0.0, 0.0, 1.0], 1, 0, 20, 14, 12, 1, 1e-6)]:
        fid, iso, tconst, domain, order, ideriv, nbroot, ndgjac, ncflim, nbcrmx, eps = fields
        requests.append(line('aa_mma2fnc', fid, iso, tconst, domain, order, ideriv, nbroot, ndgjac, ncflim, nbcrmx, eps))
    append_fx6_requests(requests, rng)
    requests.append(line('aa_mma2fnc_mut_order'))


def append_fx6_requests(requests: list[str], rng: Values) -> None:
    """mma2fx6_ on patch grids whose high constraint rows are tiny or not."""
    for mu, mv, nd, pu, pv, iu, iv, scale in [(6, 6, 3, 1, 1, 1, 1, 1e-12), (6, 6, 3, 2, 1, 2, 1, 1e-3),
                                              (8, 7, 3, 2, 2, 2, 2, 1e-15), (4, 4, 1, 1, 2, 1, 1, 1e-8)]:
        count = pu * pv
        patch = []
        for _ in range(count * nd):
            for kv in range(mv):
                for ku in range(mu):
                    patch.append(rng.next() * (scale if ku >= 2 or kv >= 2 else 1.0))
        requests.append(line('aa_mma2fx6', mu, mv, nd, pu, pv, iu, iv, 1e-3, [1e-2, 1e-2, 1e-3, 1e-2, 1e-1, 1e-2, 1e-2, 1e-2],
                             [min(mu, 2 * iu + 2)] * count, [min(mv, 2 * iv + 2)] * count, [1e-4] * count, patch))
