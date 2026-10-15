"""Deterministic requests for BRepOffset_Tool (tools/oracle/tkoffset_probe_brepoffset_tool.cxx;
the records are documented there).

The faces are faces of primitives: an axis-aligned and a tilted box (faces 0..5 are
x-min, x-max, y-min, y-max, z-min, z-max), a cylinder, a cone and a sphere at the origin,
a torus, and a cylinder crossing the box. The sections of the face pairs of one
primitive (adjacent, opposite, lateral and cap) and of two primitives run through
Inter3D (with and without the reference edge), PipeInter and InterOrExtent on both
sides, on the faces and on their enlargements; EnLargeFace runs every option on every
face kind; Inter2d intersects edge pairs of the faces; the queries cover common shapes,
plane normals, section orientations and curve sizes; Deboucle3D sees closed and open
shells with and without the removed face as boundary.
"""
from __future__ import annotations

BOX = '0 2 3 4 0 0 0 0 0 1'
TILTED_BOX = '0 1 1.5 2 0.5 -0.25 1 1 2 3'
CYLINDER = '1 1.5 3 0 0 0 0 0 0 1'
CROSSING_CYLINDER = '1 0.75 6 0 1 1.5 -1 0 0 1'
LYING_CYLINDER = '1 1 4 0 -1 1.5 2 1 0 0'
CONE = '2 2 1 3 0 0 0 0 0 1'
SPHERE = '3 2 0 0 0 0 0 0 0 1'
CROSSING_SPHERE = '3 2.5 0 0 0 0 0 0 0 1'
TORUS = '4 3 1 0 0 0 0 0 0 1'

# Face pairs of one primitive.
SAME_PAIRS = [
    (BOX, 0, 2), (BOX, 0, 4), (BOX, 1, 5), (BOX, 0, 1), (BOX, 3, 4),
    (TILTED_BOX, 0, 2), (TILTED_BOX, 2, 5), (TILTED_BOX, 4, 5),
    (CYLINDER, 0, 1), (CYLINDER, 0, 2), (CYLINDER, 1, 2),
    (CONE, 0, 1), (CONE, 0, 2),
]
# Face pairs of two primitives.
CROSS_PAIRS = [
    (BOX, 4, CROSSING_CYLINDER, 0), (BOX, 5, CROSSING_CYLINDER, 0),
    (CROSSING_CYLINDER, 0, BOX, 2), (BOX, 2, LYING_CYLINDER, 0),
    (BOX, 4, CROSSING_SPHERE, 0), (BOX, 1, CROSSING_SPHERE, 0),
    (CYLINDER, 0, CROSSING_SPHERE, 0), (BOX, 5, CONE, 0), (BOX, 4, TORUS, 0),
    (TILTED_BOX, 0, BOX, 3), (TILTED_BOX, 5, BOX, 5),
]
ENLARGE_OPTIONS = [
    '1 0 1 1 1 1 -1 -1 -1 -1', '0 0 1 1 1 1 -1 -1 -1 -1', '1 1 1 1 1 1 -1 -1 -1 -1',
    '1 0 0 1 1 1 -1 -1 -1 -1', '1 0 1 0 1 1 -1 -1 -1 -1', '1 0 1 1 0 1 -1 -1 -1 -1',
    '1 0 1 1 1 2 -1 -1 -1 -1', '0 0 1 1 1 2 -1 -1 -1 -1', '1 0 1 1 1 2 0.5 1 1.5 2',
    '1 1 1 1 1 2 0.5 0.25 2 1', '0 1 0 0 0 2 3 3 3 3',
]
FACES = [(BOX, i) for i in range(6)] + [(TILTED_BOX, 3), (CYLINDER, 0), (CYLINDER, 1),
         (CONE, 0), (CONE, 2), (SPHERE, 0), (TORUS, 0), (LYING_CYLINDER, 0)]


def inter3d_requests() -> list[str]:
    """Inter3D, PipeInter and InterOrExtent of the face pairs."""
    requests = []
    for prim, f1, f2 in SAME_PAIRS:
        for op in range(3):
            for enl1, enl2 in ((0, 0), (1, 1), (1, 0)):
                for side in (0, 1):
                    for ref in ((0, 1) if op == 0 else (0,)):
                        requests.append(f'ot_inter3d {op} {prim} {f1} {enl1} 1 {f2} {enl2} {side} {ref}')
    for p1, f1, p2, f2 in CROSS_PAIRS:
        for op in range(3):
            for enl1, enl2 in ((0, 0), (1, 1)):
                for side in (0, 1):
                    requests.append(f'ot_inter3d {op} {p1} {f1} {enl1} 0 {p2} {f2} {enl2} {side} 0')
    return requests


def enlarge_requests() -> list[str]:
    """EnLargeFace with every option on every face kind."""
    return [f'ot_enlarge {prim} {face} {options}' for prim, face in FACES for options in ENLARGE_OPTIONS]


def inter2d_requests() -> list[str]:
    """Inter2d of the edge pairs of faces and of their enlargements."""
    requests = []
    for prim, face, edges in ((BOX, 0, 4), (TILTED_BOX, 2, 4), (CYLINDER, 0, 4), (CYLINDER, 1, 1),
                              (CONE, 0, 4), (SPHERE, 0, 4), (TORUS, 0, 4)):
        for e1 in range(edges):
            for e2 in range(edges):
                for enl in (0, 1):
                    for tol in ('1e-07', '0.001'):
                        requests.append(f'ot_inter2d {prim} {face} {enl} {e1} {e2} {tol}')
    return requests


def queries_requests() -> list[str]:
    """Common shapes, plane normals, orientations and curve sizes of face pairs."""
    requests = []
    for prim, count in ((BOX, 6), (TILTED_BOX, 6), (CYLINDER, 3), (CONE, 3), (SPHERE, 1), (TORUS, 1)):
        for f1 in range(count):
            for f2 in range(count):
                requests.append(f'ot_queries {prim} {f1} {f2}')
    return requests


def deboucle_requests() -> list[str]:
    """Deboucle3D of closed and open shells."""
    requests = []
    for prim, count in ((BOX, 6), (TILTED_BOX, 6), (CYLINDER, 3), (CONE, 3), (SPHERE, 1), (TORUS, 1)):
        for removed in range(-1, count):
            for bound in (0, 1):
                requests.append(f'ot_deboucle {prim} {removed} {bound}')
    return requests


def append_tool_requests(requests: list[str]) -> None:
    """Append the BRepOffset_Tool request families in a fixed order."""
    requests += inter3d_requests()
    requests += enlarge_requests()
    requests += inter2d_requests()
    requests += queries_requests()
    requests += deboucle_requests()
