"""AIS_InteractiveContext attribute members: appearance edits through the context, display
modes, display priority and Z layer, selection mode activation with every concurrency, filters,
pixel tolerance, trihedron and plane sizes, hidden lines, sub-intensity, local attributes,
selection, erase/display, the per-object deviation and isoline members and the default
attributes, on a cylinder shape, an edge shape and a
point (see tools/oracle/tkv3d_probe_ais_context_attributes.cxx).

A request is `v3_ais_ctx_attr Count Op Object Arg...`.
"""
from __future__ import annotations

SEQUENCES = (
    ((0, 0, 0), (2, 0, 0), (4, 0, 0), (6, 0, 5), (1, 0, 0), (3, 0, 0), (5, 0, 0), (7, 0, 0)),
    ((0, 2, 0), (2, 2, 0), (6, 2, 3), (6, 2, 0), (8, 2, 0), (1, 2, 0)),
    ((9, 0, 1), (9, 1, 1), (11, 0, 1), (10, 0, 0), (11, 0, 0), (9, 2, 3), (10, 2, 0)),
    ((12, 0, 7), (12, 2, 9), (13, 1, -2), (13, 0, -3), (12, 1, 2)),
    ((14, 0, 2), (14, 0, 4), (21, 0, 1), (20, 0, 2), (16, 0, 2), (15, 0, 0), (17, 0, 1), (19, 0, 1), (18, 0, 0)),
    ((14, 1, 2), (14, 1, 0), (14, 1, 1), (16, 1, 1), (14, 2, 1), (20, 2, -1), (21, 1, -1)),
    ((22, 0, 1), (22, 0, 2), (24, 0, 0), (25, 0, 5), (23, 0, 0), (24, 0, 1)),
    ((26, 0, 50), (27, 0, 10), (28, 0, 30), (29, 0, 0), (30, 0, 0), (45, 0, 0), (34, 0, 0), (47, 0, 1)),
    ((31, 0, 0), (31, 2, 0), (43, 0, 0), (33, 0, 0), (32, 0, 0), (32, 2, 0), (46, 0, 0)),
    ((35, 0, 0), (37, 0, 0), (37, 2, 0), (38, 0, 4), (39, 0, 2), (39, 0, 1)),
    ((40, 1, 0), (0, 1, 0), (9, 1, 1), (12, 1, 6), (41, 1, 0), (42, 0, 0), (44, 2, 0)),
    ((33, 1, 0), (2, 1, 0), (31, 1, 0), (9, 1, 1), (10, 1, 0), (32, 1, 0), (46, 0, 0)),
    # Per-object deviation and triangulation isolines: shapes take them, the point (a datum)
    # does not; the edge (kind Shape) also takes the angle.
    ((48, 0, 4), (48, 1, 7), (48, 2, 2), (49, 0, 30), (49, 1, 25), (49, 2, 20), (37, 0, 0)),
    ((50, 0, 1), (50, 2, 1), (48, 0, 10), (50, 0, 0), (9, 0, 1), (48, 0, 3), (49, 0, 11)),
)


def append_ais_context_attribute_requests(requests: list[str]) -> None:
    """Every sequence as one request. UnsetLocalAttributes and appearance edits after
    SetLocalAttributes are left out: the local drawer has no link, which the native bodies
    dereference."""
    for sequence in SEQUENCES:
        body = ' '.join(f'{op} {obj} {arg}' for op, obj, arg in sequence)
        requests.append(f'v3_ais_ctx_attr {len(sequence)} {body}')
