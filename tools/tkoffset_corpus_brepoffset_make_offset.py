"""Deterministic requests for BRepOffset_MakeOffset (tools/oracle/tkoffset_probe_brepoffset_make_offset.cxx;
the records are documented there).

The primitives of tkoffset_corpus_brepoffset_tool.py: the offset shapes of the solids and
of single faces both ways with arc and intersection joins, the thick solids with a face
removed, the thickened faces.
"""
from __future__ import annotations
from tkoffset_corpus_brepoffset_tool import BOX, CONE, CYLINDER, LYING_CYLINDER, SPHERE, TILTED_BOX, TORUS

SOLIDS = [BOX, TILTED_BOX, CYLINDER, LYING_CYLINDER, CONE, SPHERE, TORUS]


def append_make_offset_requests(requests: list[str]) -> None:
    """Append the BRepOffset_MakeOffset requests in a fixed order."""
    for prim in SOLIDS:
        for off in (0.5, -0.25):
            for join in (0, 1):
                requests.append(f'om_offset {prim} -1 -1 {off} {join} 0 0 0 0')
                requests.append(f'om_offset {prim} -1 0 {off} {join} 0 0 0 0')
                requests.append(f'om_offset {prim} 0 -1 {off} {join} 0 0 0 0')
                requests.append(f'om_offset {prim} 0 -1 {off} {join} 0 1 0 0')
                requests.append(f'om_offset {prim} -2 -1 {off} {join} 0 0 0 0')
                requests.append(f'om_offset {prim} -3 -1 {off} {join} 0 0 0 0')
                requests.append(f'om_offset {prim} -2 -1 {off} {join} 0 1 0 0')
    for prim in SOLIDS:
        for sub in (0, 1, -2, -3):
            for off in (2.0, -0.25):
                for solid in (0, 1):
                    requests.append(f'om_simple {prim} {sub} {off} {solid}')
