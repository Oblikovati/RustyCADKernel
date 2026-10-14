"""DsgPrs: every presentation overload and the package layout computations on several point,
direction and value sets, every arrow side, and degenerate inputs (confused points, parallel
directions, points on circles and axes) that reach the native exceptions.

A request is `v3_dsgprs Kind ArrowLength Side P1..P5 D1..D3 V1..V4 F1 F2`; the kinds are
listed in tools/oracle/tkv3d_probe_dsg_prs.cxx (100 and above are the package computations).
"""
from __future__ import annotations
from itertools import product

# Point sets: generic, planar, symmetric about the Y axis, and with coincidences.
POINT_SETS = (
    ('0 0 0', '10 0 0', '5 6 0', '3 9 0', '-4 2 0'),
    ('1 2 3', '7 -1 4', '-2 5 1', '4 4 8', '0 -3 2'),
    ('-5 3 0', '5 3 0', '0 -2 0', '0 7 0', '0 0 0'),
    ('0 0 0', '0 0 0', '2 2 0', '2 2 0', '1 0 0'),
    ('3 0 0', '0 3 0', '0 0 3', '1 1 1', '-3 0 0'),
)
DIRECTION_SETS = (
    ('0 0 1', '1 0 0', '0 1 0'),
    ('1 0 0', '0 1 0', '0 0 1'),
    ('0 0 1', '0 0 1', '1 1 0'),
    ('0.6 0.8 0', '-0.8 0.6 0', '0 0 1'),
)
VALUE_SETS = ('5 2 1 0.5', '0.785 3 0.5 2', '3.5 1 4 0', '0 0.01 6 1.57')


def append_dsg_prs_requests(requests: list[str]) -> None:
    """Presentations (0..52, 48 the plain symbol, 49..52 the shape directions) on every combination, then the package computations (100..105)."""
    sides = (0, 1, 2, 3, 4, 5, 6, 7, 8)
    for kind in range(53):
        for index, (points, directions, values) in enumerate(product(POINT_SETS, DIRECTION_SETS, VALUE_SETS)):
            side = sides[(kind + index) % len(sides)]
            flags = f'{index % 2} {(index // 2) % 2}'
            arrow = (0.5, 2.0, 0.0)[index % 3]
            requests.append(f'v3_dsgprs {kind} {arrow} {side} {" ".join(points)} {" ".join(directions)} {values} {flags}')
    for kind in range(100, 106):
        for index, (points, directions, values) in enumerate(product(POINT_SETS, DIRECTION_SETS, VALUE_SETS)):
            requests.append(f'v3_dsgprs {kind} 1.0 0 {" ".join(points)} {" ".join(directions)} {values} {index % 2} 0')
