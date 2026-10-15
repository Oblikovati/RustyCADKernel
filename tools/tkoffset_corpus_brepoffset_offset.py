"""Deterministic requests for BRepOffset_Offset (tools/oracle/tkoffset_probe_brepoffset_offset.cxx;
the records are documented there).

The primitives of tkoffset_corpus_brepoffset_tool.py: every face offset outward and
inward, with each join and with shared images; the pipes along the edges between the
offset faces (with their connectors given back, polynomial or not, at several
tolerances and continuities) and the spheres at the box corners; the tubes around
every edge.
"""
from __future__ import annotations
from tkoffset_corpus_brepoffset_tool import (BOX, CONE, CROSSING_CYLINDER, CYLINDER, LYING_CYLINDER,
                                             SPHERE, TILTED_BOX, TORUS)

PRIMITIVES = [BOX, TILTED_BOX, CYLINDER, CROSSING_CYLINDER, LYING_CYLINDER, CONE, SPHERE, TORUS,
              '2 2 0 3 0 0 0 0 0 1', '2 0 1.5 2 1 1 1 0 0 1', '3 1 0 0 0 0 0 0 0 1']


def face_requests() -> list[str]:
    """The offset faces of every primitive."""
    return [f'oo_faces {prim} {off} {outside} {join} {shared}'
            for prim in PRIMITIVES for off in (0.5, -0.25, 3)
            for outside in (1, 0) for join in (0, 1) for shared in (0, 1)]


def pipe_requests() -> list[str]:
    """The pipes and spheres of the primitives with edges between faces."""
    requests = []
    for prim in (BOX, TILTED_BOX, CYLINDER, LYING_CYLINDER, CONE, '2 0 1.5 2 1 1 1 0 0 1'):
        for off in (0.5, -0.25):
            for poly, tol, conti in ((0, 1e-4, 2), (1, 1e-4, 2), (0, 1e-6, 4), (1, 1e-3, 0)):
                for connectors in (0, 1):
                    requests.append(f'oo_pipes {prim} {off} {poly} {tol} {conti} {connectors}')
    return requests


def tube_requests() -> list[str]:
    """The tubes around the edges of every primitive."""
    return [f'oo_tubes {prim} {off}' for prim in PRIMITIVES for off in (0.5, -0.25)]


def append_offset_requests(requests: list[str]) -> None:
    """Append the BRepOffset_Offset request families in a fixed order."""
    requests += face_requests()
    requests += pipe_requests()
    requests += tube_requests()
