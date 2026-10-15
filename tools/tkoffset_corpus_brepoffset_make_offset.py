"""Deterministic requests for BRepOffset_MakeOffset (tools/oracle/tkoffset_probe_brepoffset_make_offset.cxx;
the records are documented there).

The primitives of tkoffset_corpus_brepoffset_tool.py: the offset shapes of the solids and
of single faces both ways with arc and intersection joins, the thick solids with a face
removed, the thickened faces, and per-face offsets on boxes, a cylinder and a slot prism.
"""
from __future__ import annotations
from tkoffset_corpus_brepoffset_tool import BOX, CONE, CYLINDER, LYING_CYLINDER, SPHERE, TILTED_BOX, TORUS

SOLIDS = [BOX, TILTED_BOX, CYLINDER, LYING_CYLINDER, CONE, SPHERE, TORUS]
# The prism of a slot (two planes tangent to two half cylinders), kind 6 of ot_primitive.
SLOT = '6 2 1 3 0 0 0 0 0 1'
# The prisms of an L polygon (kind 7, arms 1.6 wide) and of a U polygon (kind 8, a gap 1
# wide between its arms) of ot_primitive: the planar non-convex solids of the complete
# intersection mode, whose inward (L) and outward (U) offsets invert faces.
L_PRISM = '7 4 4 2 0 0 0 0 0 1'
U_PRISM = '8 5 4 2 0 0 0 0 0 1'


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
    # The complete intersection mode (intersection join with the inter flag) of planar
    # solids: the splits of BRepOffset_MakeOffset_1 (BuildSplitsOfExtendedFaces for the
    # offset shapes, BuildSplitsOfTrimmedFaces for the walls of the thick solids); the
    # large inward offsets invert faces and make the invalid splits rebuilt.
    for prim in (BOX, TILTED_BOX):
        for off in (0.5, -0.25, -0.9, -1.2):
            for removed in (-1, 0, 4):
                requests.append(f'om_offset {prim} -1 {removed} {off} 1 1 0 0 0')
            requests.append(f'om_offset {prim} -1 -1 {off} 1 1 0 1 0')
    for prim, offsets in ((L_PRISM, (0.5, -0.5, -0.9)), (U_PRISM, (0.3, 0.6, -0.3, -0.6))):
        for off in offsets:
            for removed in (-1, 1):
                requests.append(f'om_offset {prim} -1 {removed} {off} 1 1 0 0 0')
            requests.append(f'om_offset {prim} -1 -1 {off} 1 1 0 1 0')
    # Per-face offsets: on the slot prism the tangent edges between faces of different
    # offsets get the artificial faces of BRepOffset_Analyse::TreatTangentFaces. At most
    # one distinct face offset value per request: UpdateFaceOffset spreads each value
    # through the connected faces in the iteration order of a hashed map (the shape
    # addresses natively), so two different values make the result address-dependent.
    for prim, removed, off, faces in (
            (SLOT, -1, 0.3, [(0, 0.5)]), (SLOT, -1, 0.3, [(0, 0.5), (2, 0.5)]),
            (SLOT, -1, 0.3, [(1, 0.5)]), (SLOT, -1, -0.2, [(0, -0.4)]), (SLOT, 5, -0.2, [(0, -0.4)]),
            (SLOT, 4, 0.25, [(1, 0.4), (3, 0.4)]),
            (BOX, -1, 0.5, [(0, 1.0)]), (BOX, -1, 0.5, [(0, 1.0), (1, 1.0)]), (BOX, 5, -0.25, [(0, -0.5)]),
            (CYLINDER, -1, 0.5, [(1, 1.0)]), (CYLINDER, -1, 0.5, [(0, 0.25)])):
        for join in (0, 1):
            body = ' '.join(f'{f} {v}' for f, v in faces)
            requests.append(f'om_face_offsets {prim} {removed} {off} {join} 0 {len(faces)} {body}')
    for prim in SOLIDS:
        for sub in (0, 1, -2, -3):
            for off in (2.0, -0.25):
                for solid in (0, 1):
                    requests.append(f'om_simple {prim} {sub} {off} {solid}')
