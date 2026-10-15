"""AIS_ViewController event dispatch over three points displayed at pixels (9,49), (100,49) and
(190,49) of a 200x100 top view: point selection with every scheme, dynamic highlighting,
rubber-band, polygon and zoom-window selection, view keys and orientation requests, dragging
start/stop and flushing without handling (see tools/oracle/tkv3d_probe_ais_view_controller.cxx).

A request is `v3_ais_view_controller Count (Op A B C D)...` with the operations
0 select point (x, y, scheme), 1 highlight (x, y), 2 rubber band (x1, y1, x2, y2),
3 polygon point (x, y, append), 4 apply the tool (scheme), 5 set the tool, 6 key down,
7 orientation request (orientation, fit), 8 flush (handle), 9 drag start (x, y), 10 drag stop,
11 continuous redraw, 12 allow highlight, 13 reset previous highlight, 14 abort view animation.
"""
from __future__ import annotations

FLUSH = (8, 1, 0, 0, 0)
SEQUENCES = [
    [(0, 100, 49, 0, 0), FLUSH],
    [(0, 9, 49, 0, 0), FLUSH, (0, 190, 49, 1, 0), FLUSH, (0, 9, 49, 3, 0), FLUSH],
    [(0, 100, 49, 0, 0), (0, 190, 49, 1, 0), FLUSH, (0, 50, 10, 0, 0), FLUSH],
    [(0, 100, 49, 2, 0), FLUSH, (0, 100, 49, 4, 0), FLUSH, (0, 100, 49, 5, 0), FLUSH],
    [(1, 190, 49, 0, 0), FLUSH, (1, 190, 49, 0, 0), FLUSH, (1, 50, 10, 0, 0), FLUSH],
    [(1, 100, 49, 0, 0), (8, 0, 0, 0, 0), (8, 1, 0, 0, 0), (13, 0, 0, 0, 0), (1, 100, 49, 0, 0), FLUSH],
    [(12, 0, 0, 0, 0), (1, 100, 49, 0, 0), FLUSH, (9, 100, 49, 0, 0), FLUSH, (10, 0, 0, 0, 0), FLUSH],
    [(9, 100, 49, 0, 0), FLUSH, (10, 0, 0, 0, 0), FLUSH],
    [(1, 100, 49, 0, 0), (9, 100, 49, 0, 0), FLUSH, (10, 0, 0, 0, 0), FLUSH],
    [(2, 0, 0, 150, 99, ), FLUSH, (4, 0, 0, 0, 0), FLUSH],
    [(2, 150, 99, 0, 0), (4, 1, 0, 0, 0), FLUSH],
    [(2, 5, 30, 120, 70), (4, 0, 0, 0, 0), FLUSH, (2, 95, 30, 199, 70), (4, 1, 0, 0, 0), FLUSH],
    [(2, 5, 30, 120, 70), FLUSH, (2, 5, 30, 5, 30), FLUSH, (4, 2, 0, 0, 0), FLUSH],
    [(3, 0, 0, 1, 0), (3, 150, 0, 1, 0), (3, 150, 99, 1, 0), (3, 0, 99, 1, 0), FLUSH,
     (4, 0, 0, 0, 0), FLUSH],
    [(3, 0, 0, 1, 0), (3, 0, 0, 1, 0), (3, 199, 0, 1, 0), (3, 199, 99, 0, 0), (3, 199, 98, 0, 0),
     FLUSH, (4, 3, 0, 0, 0), FLUSH],
    [(3, 0, 0, 1, 0), (3, 199, 99, 1, 0), FLUSH, (4, 0, 0, 0, 0), FLUSH],
    [(2, 50, 20, 150, 80), (5, 3, 0, 0, 0), FLUSH, (4, 0, 0, 0, 0), FLUSH],
    [(2, 50, 20, 150, 80), (5, 3, 0, 0, 0), (4, 0, 0, 0, 0), FLUSH],
    [(5, 1, 0, 0, 0), FLUSH, (5, 2, 0, 0, 0), (4, 0, 0, 0, 0), FLUSH],
    [(6, 110, 0, 0, 0), FLUSH, (14, 0, 0, 0, 0)],
    [(6, 112, 0, 0, 0), (6, 115, 0, 0, 0), FLUSH, (14, 0, 0, 0, 0)],
    [(6, 116, 0, 0, 0), FLUSH, (14, 0, 0, 0, 0), (6, 114, 0, 0, 0), FLUSH, (14, 0, 0, 0, 0)],
    [(6, 106, 0, 0, 0), FLUSH, (14, 0, 0, 0, 0)],
    [(7, 4, 1, 0, 0), FLUSH, (7, 1, 0, 0, 0), FLUSH],
    [(7, 3, 1, 0, 0), (8, 0, 0, 0, 0), (8, 1, 0, 0, 0)],
    [(11, 1, 0, 0, 0), FLUSH, (0, 100, 49, 0, 0), FLUSH],
    [(0, 100, 49, 0, 0), FLUSH, (0, 100, 49, 1, 0), FLUSH, (2, 0, 0, 199, 99), (4, 3, 0, 0, 0),
     FLUSH, (0, 50, 10, 0, 0), FLUSH],
]


def append_ais_view_controller_requests(requests: list[str]) -> None:
    """Every operation sequence."""
    for ops in SEQUENCES:
        body = ' '.join(' '.join(map(str, (op + (0, 0, 0, 0))[:5])) for op in ops)
        requests.append(f'v3_ais_view_controller {len(ops)} {body}')
