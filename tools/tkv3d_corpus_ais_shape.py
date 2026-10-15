"""AIS_Shape: the state after appearance edits, the three presentation modes, the selection in
modes 0, 2 and 4, the deviation and texture parameters and the HLR presentations, on a vertex,
an edge, a cylinder patch, a compound, an empty compound and an infinite plane.

A request is `v3_ais_shape Shape Link Width Count Op...` (see
tools/oracle/tkv3d_probe_ais_shape.cxx); the HLR ops (21, 22) need a linked drawer. A null
shape is left out (its bounding box dereferences a null TShape natively), and so are the
appearance edits on own defaults without link (they dereference the missing link natively).
"""
from __future__ import annotations

SEQUENCES = (
    (8, 9, 10, 11, 12, 13, 99),
    (0, 8, 2, 9, 1, 3, 8, 9),
    (4, 6, 9, 5, 7, 9, 0, 6, 1, 7),
    (14, 16, 8, 15, 23, 17, 8, 9),
    (2, 0, 3, 1, 4, 5, 6, 7, 19, 9, 10),
    (20, 8, 10, 11, 12),
)
APPEARANCE = set(range(8))


def append_ais_shape_requests(requests: list[str]) -> None:
    """Every sequence on every shape with linked defaults (own defaults without appearance
    edits), then the HLR presentations."""
    for shape in (0, 1, 2, 3, 4, 6):
        for link in (0, 1):
            for ops in SEQUENCES:
                if link == 0 and APPEARANCE & set(ops):
                    continue
                requests.append(f'v3_ais_shape {shape} {link} 2.0 {len(ops)} {" ".join(map(str, ops))}')
        for ops in ((21,), (22,), (18, 21), (18, 22), (0, 2, 21)):
            requests.append(f'v3_ais_shape {shape} 1 1.5 {len(ops)} {" ".join(map(str, ops))}')
