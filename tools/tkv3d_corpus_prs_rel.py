"""PrsDim relations displayed in a context (see tools/oracle/tkv3d_probe_prs_rel.cxx): every
construction of each relation kind with its state, then property edits (position, arrow size,
symbol, color, bounding box, value, automatic position, extension shape, text) each followed
by the traced presentation and the selection.

A request is `v3_prsrel Kind Variant Count (Op A B C)...`.
"""
from __future__ import annotations

SHOW = (16, 0, 0, 0)
# Kind: number of constructions.
KINDS = {0: 4, 1: 4, 2: 3, 3: 7, 4: 7, 5: 7, 6: 7, 7: 7, 8: 5, 9: 10, 10: 9, 11: 9, 12: 10, 13: 7, 14: 17}
EDITS = [
    [SHOW],
    [SHOW, (0, 12, -5, 3), SHOW, (8, 1, 0, 0), SHOW],
    [(1, 25, 0, 0), SHOW, (2, 3, 0, 0), SHOW, (2, 6, 0, 0), SHOW],
    [(3, 0, 0, 0), SHOW, (3, 1, 0, 0), (4, 0, 0, 0), SHOW, (4, 0, 0, 0), SHOW],
    [(5, -5, -5, -5), SHOW, (6, 0, 0, 0), SHOW],
    [(7, 35, 0, 0), SHOW, (7, 400, 0, 0), SHOW, (9, 1, 0, 0), SHOW],
    [(10, 2, 0, 0), SHOW, (10, 0, 0, 0), SHOW, (8, 0, 0, 0), SHOW],
]


def append_prs_rel_requests(requests: list[str]) -> None:
    """Every edit sequence on every construction of every kind."""
    for kind, variants in KINDS.items():
        for variant in range(variants):
            for ops in EDITS:
                body = ' '.join(' '.join(map(str, op)) for op in ops)
                requests.append(f'v3_prsrel {kind} {variant} {len(ops)} {body}')
