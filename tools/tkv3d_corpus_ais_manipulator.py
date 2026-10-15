"""AIS_Manipulator attached to a point at the pixel P of a 400x300 top view: attachment
options, parts, sizes, gaps, skins, zoom persistence, presentation geometry (traced), the
selection of every mode, detection and selection of parts at pixel offsets from P,
interactive translation (an unattached manipulator dereferences its null objects natively
and adjusting the size to a point box crashes natively, so neither is requested), rotation, scaling and plane dragging, behaviors, flat-skin camera
recomputation and detachment (see tools/oracle/tkv3d_probe_ais_manipulator.cxx).

A request is `v3_ais_manipulator Count (Op A B C)...`; the operations are listed in the probe.
"""
from __future__ import annotations

ATTACH = (0, 1, 0, 1)
DUMP = (19, 1, 0, 0)
NODUMP = (19, 0, 0, 0)
SEQUENCES = [
    [],
    [DUMP, ATTACH],
    [(0, 0, 0, 0), (0, 1, 0, 1)],
    [(1, 3, 1, 1), (1, 0, 2, 0), (2, 3, 0, 0), (3, 50, 0, 0), (4, 5, 0, 0), (3, 7, 0, 0), ATTACH],
    [ATTACH, (8, 1, 0, 0), (8, 2, 0, 0), (8, 3, 0, 0), (8, 4, 0, 0), (8, 5, 0, 0), (8, 0, 0, 0)],
    [(5, 1, 0, 0), DUMP, ATTACH, NODUMP, (8, 1, 0, 0), (8, 2, 0, 0), (8, 3, 0, 0), (8, 4, 0, 0)],
    [(6, 0, 0, 0), ATTACH, (6, 1, 0, 0), (17, 5, 6, 7)],
    [ATTACH, (9, 40, 0, 0), (10, 0, 0, 0), (11, 40, 0, 0), (12, 60, 0, 0), (12, 80, 5, 0), (13, 1, 0, 0)],
    [ATTACH, (9, 0, -40, 0), (10, 0, 0, 0), (11, 0, -40, 0), (12, 0, -70, 0), (13, 0, 0, 0)],
    [ATTACH, (9, 69, -69, 0), (10, 0, 0, 0), (11, 69, -69, 0), (12, 0, -98, 0), (12, -69, -69, 0), (13, 1, 0, 0)],
    [ATTACH, (9, 87, 0, 0), (10, 0, 0, 0), (11, 87, 0, 0), (12, 120, 0, 0), (12, 87, 0, 0), (13, 1, 0, 0)],
    [ATTACH, (9, 30, -30, 0), (10, 0, 0, 0), (11, 30, -30, 0), (12, 50, -10, 0), (13, 1, 0, 0)],
    [(7, 1, 0, 0), ATTACH, (9, 40, 0, 0), (9, 0, -40, 0), (18, 0, 0, -40), (18, 2, 0, -80), (18, 3, 0, -80)],
    [ATTACH, (9, 40, 0, 0), (10, 0, 0, 0), (18, 0, 40, 0), (18, 1, 0, 0), (18, 2, 70, 3), (18, 4, 0, 0)],
    [ATTACH, (22, 0, 0, 0), (9, 40, 0, 0), (10, 0, 0, 0), (11, 40, 0, 0), (12, 70, 0, 0), (13, 1, 0, 0)],
    [ATTACH, (9, 40, 0, 0), (10, 0, 0, 0), (15, 0, 0, 0), (20, 0, 0, 0), (14, 0, 0, 0), (14, 0, 0, 0)],
    [(5, 1, 0, 0), ATTACH, DUMP, (16, 0, 0, 0), NODUMP, (8, 1, 0, 0), (8, 3, 0, 0)],
    [ATTACH, (21, 2, 0, 0), (23, 0, 0, 0), (17, 0, 0, 0)],
    [(13, 0, 0, 0), (16, 0, 0, 0), (14, 0, 0, 0), (15, 0, 0, 0)],
    [ATTACH, (3, 30, 0, 0), (23, 0, 0, 0), (8, 1, 0, 0), (8, 4, 0, 0), (6, 0, 0, 0), (8, 4, 0, 0)],
]


def append_ais_manipulator_requests(requests: list[str]) -> None:
    """Every operation sequence."""
    for ops in SEQUENCES:
        body = ' '.join(' '.join(map(str, op)) for op in ops)
        requests.append(f'v3_ais_manipulator {len(ops)} {body}'.rstrip())
