"""AdvApp2Var class requests of the TKGeomBase numeric oracle (tkgeombase_probe_advapp2var_class.cxx).

The Context requests cover every continuity order and degree limit, the grid
requests the cutting of the network and the framework, the patch requests the
iso and patch approximations of one grid, and the approx requests run
AdvApp2Var_ApproxAFunc2Var end to end over the probe functions, with and without
a criterion. Continuities are GeomAbs_Shape values (C0 0, G1 1, C1 2, G2 3, C2 4,
C3 5), favourite isos GeomAbs_IsoType values (IsoU 0, IsoV 1, NoneIso 2).
"""
from __future__ import annotations

from tkgeombase_corpus_advapp2var import line

CONTEXTS = [
    (1, 0, 0, 5, 5, 0, 0, 0, 1),
    (2, 1, 1, 10, 10, 1, 0, 0, 1),
    (2, 2, 2, 15, 20, 2, 1, 1, 1),
    (1, 2, 1, 30, 30, 3, 0, 2, 1),
    (1, -1, -1, 4, 4, 0, 0, 0, 1),
    (2, 1, 2, 61, 61, 3, 0, 0, 2),
    (2, 0, 1, 25, 7, 5, 2, 0, 0),
    (2, 3, 0, 10, 10, 0, 0, 0, 1),
    (2, 0, 0, 1, 5, 0, 0, 0, 1),
    (2, 1, 1, 62, 10, 0, 0, 0, 1),
]

GRIDS = [
    (1, 1, 0.0, 1.0, 0.0, 1.0, [0]),
    (1, 1, 0.0, 1.0, 0.0, 1.0, [1, 0, 0.5]),
    (2, 0, -1.0, 2.0, 0.0, 3.0, [3, 0, 0.5, 1, 1.0, 0, 1.25]),
    (0, 2, 0.0, 1.0, 0.0, 1.0, [4, 1, 0.25, 1, 0.75, 0, 0.4, 1, 0.5]),
]

# function ifav iu iv ndu ndv iprecis u0 u1 v0 v1 numdec critmax cuts
PATCHES = [
    (0, 2, 1, 1, 10, 10, 1, [0.0, 1.0, 0.0, 1.0], 5, 1.0e-2, [0]),
    (1, 1, 0, 0, 8, 8, 0, [0.0, 2.0, 0.0, 1.0], 3, 0.5, [1, 0, 1.0]),
    (2, 2, 2, 2, 14, 14, 2, [-1.0, 1.0, -1.0, 1.0], 5, 1.0e-6, [2, 0, 0.0, 1, 0.0]),
    (3, 1, 1, 2, 6, 9, 3, [0.0, 1.0, 0.0, 1.0], 0, 1.0, [0]),
    (1, 2, 1, 1, 4, 4, 1, [0.0, 3.0, 0.0, 3.0], 5, 1.0e-3, [0]),
    (1, 2, 1, 1, 4, 4, 1, [0.0, 3.0, 0.0, 3.0], 1, 1.0e-3, [1, 1, 1.5]),
    (11, 2, 1, 1, 10, 10, 1, [0.0, 1.0, 0.0, 1.0], 5, 1.0e-2, [0]),
]

# function favor contU contV iprecis maxdegU maxdegV maxpatch u0 u1 v0 v1 cutkind critkind critrep critmax
APPROXES = [
    (0, 1, 2, 2, 1, 9, 9, 10, [0.0, 1.0, 0.0, 1.0], 0, 0, 0, 0.0),
    (1, 0, 0, 0, 0, 5, 5, 20, [0.0, 3.0, 0.0, 2.0], 0, 0, 0, 0.0),
    (2, 2, 4, 4, 2, 11, 11, 30, [-1.0, 1.0, -1.0, 1.0], 1, 0, 0, 0.0),
    (3, 1, 2, 4, 3, 7, 13, 16, [0.0, 1.0, 0.0, 2.0], 0, 0, 0, 0.0),
    (1, 1, 2, 2, 1, 4, 4, 1, [0.0, 3.0, 0.0, 3.0], 0, 0, 0, 0.0),
    (1, 1, 2, 2, 1, 3, 3, 50, [0.0, 3.0, 0.0, 3.0], 1, 0, 0, 0.0),
    (1, 0, 4, 0, -1, 8, 6, 12, [0.0, 10.0, 0.0, 10.0], 0, 0, 0, 0.0),
    (2, 1, 2, 2, 5, 14, 14, 40, [0.0, 2.0, 0.0, 2.0], 0, 0, 0, 0.0),
    (0, 1, 5, 2, 1, 9, 9, 10, [0.0, 1.0, 0.0, 1.0], 0, 0, 0, 0.0),
    (0, 1, 2, 1, 1, 9, 9, 10, [0.0, 1.0, 0.0, 1.0], 0, 0, 0, 0.0),
    (1, 1, 2, 2, 1, 9, 9, 20, [0.0, 3.0, 0.0, 2.0], 0, 1, 0, 1.0e-3),
    (1, 1, 2, 2, 1, 9, 9, 20, [0.0, 3.0, 0.0, 2.0], 0, 2, 1, 1.0e-4),
    (2, 1, 2, 2, 1, 9, 9, 20, [-1.0, 1.0, -1.0, 1.0], 1, 2, 0, 1.0e-5),
    (3, 0, 2, 2, 1, 9, 9, 20, [0.0, 1.0, 0.0, 1.0], 0, 1, 1, 10.0),
    (11, 1, 2, 2, 1, 9, 9, 10, [0.0, 1.0, 0.0, 1.0], 0, 0, 0, 0.0),
]


def append_class_requests(requests: list[str]) -> None:
    """Append the AdvApp2Var class requests in a fixed order."""
    for context in CONTEXTS:
        requests.append(line('aa_context', list(context)))
    for iu, iv, u0, u1, v0, v1, cuts in GRIDS:
        requests.append(line('aa_grid', iu, iv, u0, u1, v0, v1, cuts))
    for fid, ifav, iu, iv, ndu, ndv, prec, domain, numdec, crit, cuts in PATCHES:
        requests.append(line('aa_patch', fid, ifav, iu, iv, ndu, ndv, prec, domain, numdec, crit, cuts))
    for fid, fav, cu, cv, prec, mu, mv, mp, domain, cut, ck, cr, cm in APPROXES:
        requests.append(line('aa_approx', fid, fav, cu, cv, prec, mu, mv, mp, domain, cut, ck, cr, cm))
