"""AIS_Triangulation: the mode 0 triangle array with and without normals and node colors,
transparency edits (own shading aspect copied from the link), colors and the empty selection
(see tools/oracle/tkv3d_probe_ais_triangulation.cxx).

A request is `v3_ais_triangulation Normals Colors Link Count Op...`.
"""
from __future__ import annotations
from itertools import product

SEQUENCES = ((0,), (0, 1, 0, 2, 0), (1, 0, 3, 0, 2), (4, 5, 0, 1, 1, 2), (3, 0, 1, 0))


def append_ais_triangulation_requests(requests: list[str]) -> None:
    """Every sequence with and without normals and colors, own or linked defaults."""
    for normals, colors, link, ops in product((0, 1), (0, 1), (0, 1), SEQUENCES):
        requests.append(f'v3_ais_triangulation {normals} {colors} {link} {len(ops)} {" ".join(map(str, ops))}')
