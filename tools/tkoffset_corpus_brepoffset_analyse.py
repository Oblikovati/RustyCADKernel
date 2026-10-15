"""Deterministic requests for BRepOffset_Analyse (tools/oracle/tkoffset_probe_brepoffset_analyse.cxx;
the record is documented there).

The primitives of tkoffset_corpus_brepoffset_tool.py and the slot prism (two planes
tangent to two half cylinders) analysed with a uniform offset, then the slot with
different offsets on tangent faces (the artificial faces of TreatTangentFaces: one
plane, both planes, a half cylinder, both half cylinders, inward) and a box with a
face offset (no tangent edge).
"""
from __future__ import annotations
from tkoffset_corpus_brepoffset_make_offset import SLOT
from tkoffset_corpus_brepoffset_tool import BOX, CONE, CYLINDER, LYING_CYLINDER, SPHERE, TILTED_BOX, TORUS


def request(prim: str, off: float, faces: list[tuple[int, float]], angle: float = 0.01) -> str:
    body = ' '.join(f'{f} {v}' for f, v in faces)
    return f'oan_analyse {prim} {angle} {off} {len(faces)} {body}'.rstrip()


def append_analyse_requests(requests: list[str]) -> None:
    """The analyses in a fixed order."""
    for prim in (BOX, TILTED_BOX, CYLINDER, LYING_CYLINDER, CONE, SPHERE, TORUS, SLOT):
        requests.append(request(prim, 0.3, []))
    for off, faces in ((0.3, [(0, 0.5)]), (0.3, [(0, 0.5), (2, 0.2)]), (0.3, [(1, 0.5)]),
                       (0.25, [(1, 0.4), (3, 0.4)]), (-0.2, [(0, -0.4)]), (0.3, [(0, 0.5), (2, 0.5)])):
        requests.append(request(SLOT, off, faces))
    requests.append(request(SLOT, 0.3, [(0, 0.5)], 0.5))
    requests.append(request(BOX, 0.5, [(0, 1.0)]))
