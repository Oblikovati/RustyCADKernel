"""Deterministic requests for BRepOffset_Inter3d (tools/oracle/tkoffset_probe_brepoffset_inter3d.cxx;
the records are documented there).

The primitives of tkoffset_corpus_brepoffset_tool.py analysed and offset both ways: the
intersections across their edges (and, for planar shapes, vertices), of all near faces,
and with a context face kept or extended.
"""
from __future__ import annotations
from tkoffset_corpus_brepoffset_tool import BOX, CONE, CYLINDER, LYING_CYLINDER, SPHERE, TILTED_BOX, TORUS

PRIMITIVES = [BOX, TILTED_BOX, CYLINDER, LYING_CYLINDER, CONE, SPHERE, TORUS, '2 0 1.5 2 1 1 1 0 0 1']


def append_inter3d_requests(requests: list[str]) -> None:
    """Append the BRepOffset_Inter3d requests in a fixed order."""
    for prim in PRIMITIVES:
        for off in (0.5, -0.25):
            for op in range(3):
                for planar in (0, 1):
                    requests.append(f'oi_inter {prim} {off} 0.01 {op} {planar}')
