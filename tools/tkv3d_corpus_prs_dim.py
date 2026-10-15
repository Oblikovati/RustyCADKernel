"""PrsDim dimensions displayed in a context (see tools/oracle/tkv3d_probe_prs_dim.cxx): every
construction of each dimension kind with its measured geometry, value, plane and label
position, then property edits (flyout, custom values, label positions, arrows, 3D text, units,
custom planes, directions, special symbols, formats) each followed by the traced presentation
and the selection of every mode.

A request is `v3_prsdim Kind Variant Count (Op A B C)...`.
"""
from __future__ import annotations

SHOW = (16, 0, 0, 0)
KINDS = {0: 11, 1: 8, 2: 7}
EDITS = [
    [SHOW],
    [(0, 50, 0, 0), SHOW, (0, -30, 0, 0), SHOW],
    [(1, 42, 0, 0), SHOW, (2, 0, 0, 0), SHOW],
    [(4, 0, 0, 0), SHOW, (4, 1, 0, 0), SHOW, (4, 2, 0, 0), SHOW],
    [(5, 0, 0, 0), SHOW, (5, 1, 0, 0), SHOW, (5, 2, 0, 0), SHOW],
    [(6, 0, 0, 0), SHOW, (6, 1, 0, 0), SHOW],
    [(13, 1, 0, 0), SHOW, (18, 25, 0, 0), SHOW, (18, 25, 1, 0), SHOW],
    [(11, 1, 82, 0), SHOW, (11, 2, 35, 0), SHOW, (12, 1, 0, 0), SHOW, (12, 2, 0, 0), SHOW],
    [(17, 80, 30, 0), (6, 1, 0, 0), SHOW, (15, 25, 0, 0), SHOW],
    [SHOW, (3, 5, 4, 0), SHOW, (3, -6, 3, 0), SHOW, (3, 20, -2, 0), SHOW, (14, 0, 0, 0), SHOW],
    [(9, 1, 0, 0), SHOW, (9, 2, 0, 0), SHOW, (9, 0, 0, 0), SHOW],
    [(10, 1, 1, 0), SHOW, (10, 0, 1, 0), SHOW, (10, 0, 0, 0), SHOW],
]


def append_prs_dim_requests(requests: list[str]) -> None:
    """Every edit sequence on every construction of every kind."""
    for kind, variants in KINDS.items():
        for variant in range(variants):
            for ops in EDITS:
                body = ' '.join(' '.join(map(str, op)) for op in ops)
                requests.append(f'v3_prsdim {kind} {variant} {len(ops)} {body}')
