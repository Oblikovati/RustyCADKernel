"""Deterministic requests for BRepOffsetAPI_MiddlePath
(tools/oracle/tkoffset_probe_brepoffsetapi_middle_path.cxx; the records are documented there).

The pipes of the square and circle profiles along the spines of the pipe shell corpus (the
open polylines give linear middle edges, the arc after a segment and the B-spline
interpolated ones, the closed square a ring), the solid pipes of a square face (between the
faces and their outer wires), and the primitives between opposite faces (box, tilted box,
cylinder, cone, partial tori with arc paths) and adjacent faces of the box.
"""
from __future__ import annotations
from tkbool_corpus_brepfill_location_law import PATHS
from tkbool_corpus_brepfill_pipe import face_profile
from tkbool_corpus_brepfill_pipe_shell import FRAMES, profile
from tkoffset_corpus_brepoffset_tool import BOX, CONE, CYLINDER, TILTED_BOX
from tktopalgo_shape_program import ShapeProgram


def pipe_request(spine: str, builder, wires: int = 0) -> str:
    """The middle path of the pipe of a profile along a spine."""
    p = ShapeProgram()
    spine_index = PATHS[spine](p)
    profile_index = builder(p, spine)
    return f'oa_middle {p.tokens()} {spine_index} {profile_index} {wires}'


def append_middle_path_requests(requests: list[str]) -> None:
    """The pipes and primitives in a fixed order."""
    for spine in sorted(FRAMES):
        for kind in ('square', 'circle'):
            requests.append(pipe_request(spine, lambda p, s, k=kind: profile(p, s, k)))
    for spine in ('segment', 'bent'):
        for wires in (0, 1):
            requests.append(pipe_request(spine, face_profile, wires))
    for prim, start, end in ((BOX, 0, 1), (BOX, 4, 5), (BOX, 0, 2), (TILTED_BOX, 0, 1),
                             (CYLINDER, 1, 2), (CONE, 1, 2)):
        for wires in (0, 1):
            requests.append(f'oa_middle_prim {prim} {start} {end} {wires}')
    # Partial tori: the paths are arcs, so the middle edges are arcs.
    for angle, frame in ((1.5, '0 0 0 0 0 1'), (3.0, '0 0 0 0 0 1'), (4.5, '1 2 0.5 1 1 1')):
        for wires in (0, 1):
            requests.append(f'oa_middle_prim 5 3 1 {angle} {frame} 1 2 {wires}')
