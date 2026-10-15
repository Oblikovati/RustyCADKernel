"""Deterministic requests for BRepOffsetAPI_DraftAngle and Draft_Modification
(tools/oracle/tkoffset_probe_brepoffsetapi_draft_angle.cxx; the record is documented there).

The lateral faces of a box drafted one, two and four at a time about the bottom, the top
and the middle plane, both ways and up or down; a tilted box along its axis; the lateral
faces of a cylinder (into cones), a lying cylinder, a cone (into another cone) and a slot
prism (tangent planes and half cylinders, spread or postponed); then
the failures: a face parallel to the neutral plane, a cylinder drafted across its axis,
a sphere, the second Add of a face, a removed face, and a negligible angle.
"""
from __future__ import annotations
from tkoffset_corpus_brepoffset_tool import BOX, CONE, CYLINDER, LYING_CYLINDER, SPHERE, TILTED_BOX

UP = '0 0 1'
SLOT = '6 2 1 3 0 0 0 0 0 1'
BOTTOM = '0 0 0 0 0 1'


def request(prim: str, adds: list[tuple[int, str, float, str, int]], removed: int = -1) -> str:
    """`adds` are `(face, direction, angle, plane point and normal, flag)`."""
    body = ' '.join(f'{f} {d} {a} {p} {fl}' for f, d, a, p, fl in adds)
    return f'oa_draft_angle {prim} {len(adds)} {body} {removed}'


def append_draft_angle_requests(requests: list[str]) -> None:
    """The drafts in a fixed order."""
    for angle in (0.1, -0.1, 0.3):
        for plane in (BOTTOM, '0 0 4 0 0 1', '0 0 2 0 0 1'):
            requests.append(request(BOX, [(0, UP, angle, plane, 1)]))
            requests.append(request(BOX, [(0, UP, angle, plane, 1), (2, UP, angle, plane, 1)]))
            requests.append(request(BOX, [(f, UP, angle, plane, 1) for f in range(4)]))
    requests.append(request(BOX, [(f, '0 0 -1', 0.1, BOTTOM, 1) for f in range(4)]))
    requests.append(request(BOX, [(f, UP, 0.1, '0 0 0 0 0 -1', 1) for f in range(4)]))
    tilted = '1 2 3'
    requests.append(request(TILTED_BOX, [(f, tilted, 0.15, '0.5 -0.25 1 1 2 3', 1) for f in range(4)]))
    for angle in (0.1, -0.2):
        requests.append(request(CYLINDER, [(0, UP, angle, BOTTOM, 1)]))
        requests.append(request(CYLINDER, [(0, UP, angle, '0 0 1.5 0 0 1', 1)]))
        requests.append(request(CONE, [(0, UP, angle, BOTTOM, 1)]))
    requests.append(request(LYING_CYLINDER, [(0, '1 0 0', 0.1, '0 -1 1.5 1 0 0', 1)]))
    # The slot prism: the planes and half cylinders are tangent, so the draft spreads
    # along them (Flag) or the cylinders are postponed between the drafted planes.
    for face in range(4):
        for fl in (1, 0):
            requests.append(request(SLOT, [(face, UP, 0.1, BOTTOM, fl)]))
    for fl in (1, 0):
        requests.append(request(SLOT, [(0, UP, 0.1, BOTTOM, fl), (2, UP, 0.1, BOTTOM, fl)]))
        requests.append(request(SLOT, [(f, UP, -0.15, '0 0 3 0 0 1', fl) for f in range(4)]))
    # Failures and edge cases.
    requests.append(request(BOX, [(5, UP, 0.1, BOTTOM, 1)]))
    requests.append(request(CYLINDER, [(0, '1 0 0', 0.1, BOTTOM, 1)]))
    requests.append(request(SPHERE, [(0, UP, 0.1, BOTTOM, 1)]))
    requests.append(request(BOX, [(0, UP, 0.1, BOTTOM, 1), (0, UP, 0.1, BOTTOM, 1)]))
    requests.append(request(BOX, [(0, UP, 0.1, BOTTOM, 1), (2, UP, 0.1, BOTTOM, 1)], 0))
    requests.append(request(BOX, [(5, UP, 0.1, BOTTOM, 1)], 5))
    requests.append(request(BOX, [(0, UP, 1.0e-5, BOTTOM, 1)]))
