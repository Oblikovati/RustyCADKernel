"""Deterministic requests for BRepOffsetAPI_MakePipe, MakeEvolved, MakeDraft, MakeFilling,
NormalProjection and FindContigousEdges (tools/oracle/tkoffset_probe_brepoffsetapi_wrappers.cxx;
the records are documented there).

Each wrapper drives the shapes of the corpus of its BRepFill or BRepAlgo algorithm: the
pipe spines and profiles (two-argument constructor and each trihedron), the evolved
spines and profiles (classic and volume, the profile placed by BRepFill::Axe, required on
the spine or not), the drafted wire, face, shell and open polyline (with and without
SetOptions, each limit), the fillings of the twisted quadrilateral and the square, the
normal projections on the square and the cube, and the contiguous edges of cube faces.
"""
from __future__ import annotations
from tkbool_corpus_brepfill_advanced_evolved import SPINES
from tkbool_corpus_brepfill_draft import SHAPES as DRAFT_SHAPES, stop_face
from tkbool_corpus_brepfill_evolved import PROFILES as EVOLVED_PROFILES
from tkbool_corpus_brepfill_filling import square_requests, twisted_requests
from tkbool_corpus_brepfill_generator import circle_wire, polyline_wire
from tkbool_corpus_brepfill_location_law import PATHS
from tkbool_corpus_brepfill_pipe import PROFILES as PIPE_PROFILES, face_profile
from tkbool_corpus_brepfill_pipe_shell import FRAMES
from tkbool_corpus_brepproj import targets, tools
from tktopalgo_corpus_brepcheck_shapes import cube
from tktopalgo_shape_program import ShapeProgram


def pipe_request(spine: str, builder, mode: int, c1: int = 0) -> str:
    """One pipe of a profile along a spine."""
    p = ShapeProgram()
    spine_index = PATHS[spine](p)
    profile_index = builder(p, spine)
    return f'oa_pipe {p.tokens()} {spine_index} {profile_index} {mode} {c1}'


def evolved_request(spine: str, profile: str, axe: int, solid: int, on_spine: int, volume: int, join: int = 0) -> str:
    """One evolved shape of a profile along a spine (wire, or circle)."""
    p = ShapeProgram()
    spine_index = circle_wire(p, (0., 0., 0.), 4.) if spine == 'circle' else polyline_wire(p, SPINES[spine], True)
    points, closed = EVOLVED_PROFILES[profile]
    profile_index = polyline_wire(p, points, closed)
    return f'oa_evolved {p.tokens()} {spine_index} {profile_index} {join} {axe} {solid} {on_spine} 1e-07 {volume}'


def draft_request(shape: str, style: int, kind: str, internal: int = 0, angle: float = 0.1) -> str:
    """One draft of a shape along +Z up to a length, a plane or a stop face."""
    p = ShapeProgram()
    index = DRAFT_SHAPES[shape](p)
    if kind == 'length':
        tail = '0 3.0'
    elif kind == 'plane':
        tail = '1 0.0 0.0 2.0 0.0 0.0 1.0 1'
    else:
        tail = f'2 {stop_face(p)} 1'
    return f'oa_draft {p.tokens()} {index} 0.0 0.0 1.0 {angle} {style} 0.01 3.0 {internal} {tail}'


def projection_request(target: str, edges: list[str], maxdist: float = -1.0, with3d: int = 1, limit: int = 1) -> str:
    """One normal projection of tool edges on a target."""
    p = ShapeProgram()
    t = targets(p)
    e = tools(p)
    ids = ' '.join(str(e[name]) for name in edges)
    return f'oa_normal_projection {p.tokens()} {t[target]} {len(edges)} {ids} {maxdist} {with3d} {limit}'


def contigous_request(count: int, tol: float, option: int) -> str:
    """The contiguous edges of the first `count` faces of a cube."""
    p = ShapeProgram()
    shapes = cube(p, 2.0)
    faces = shapes.faces[:count]
    return f'oa_contigous {p.tokens()} {len(faces)} {" ".join(map(str, faces))} {tol} {option}'


def append_wrapper_requests(requests: list[str]) -> None:
    """Every wrapper over the shapes of its algorithm's corpus."""
    for spine in sorted(FRAMES):
        for name in sorted(PIPE_PROFILES):
            requests.append(pipe_request(spine, PIPE_PROFILES[name], -1))
    for spine in ('segment', 'bent'):
        requests.append(pipe_request(spine, face_profile, -1))
        for mode in (0, 1, 2, 9):
            requests.append(pipe_request(spine, PIPE_PROFILES['square'], mode))
        requests.append(pipe_request(spine, PIPE_PROFILES['circle'], 0, 1))
    for spine in sorted(SPINES):
        for profile in ('vertical', 'outward', 'triangle'):
            for axe, on_spine in ((1, 0), (0, 0), (0, 1)):
                requests.append(evolved_request(spine, profile, axe, 1, on_spine, 0))
            requests.append(evolved_request(spine, profile, 1, 0, 0, 0))
    for join in (0, 1, 2):
        requests.append(evolved_request('square', 'vertical', 1, 1, 0, 0, join))
    for profile in ('vertical', 'outward'):
        requests.append(evolved_request('square', profile, 1, 1, 0, 1))
        requests.append(evolved_request('square', profile, 1, 0, 0, 1))
    for shape in sorted(DRAFT_SHAPES):
        for style in (-1, 0, 1, 2):
            requests.append(draft_request(shape, style, 'length'))
        requests.append(draft_request(shape, -1, 'length', 1))
        requests.append(draft_request(shape, -1, 'plane'))
        requests.append(draft_request(shape, -1, 'stop'))
    for line in twisted_requests() + square_requests():
        requests.append('oa_filling' + line[len('bf_filling'):])
    for target in ('square', 'cube'):
        requests.append(projection_request(target, ['crossing', 'inner', 'arc', 'spline']))
        requests.append(projection_request(target, ['crossing'], 0.5))
        requests.append(projection_request(target, ['spline'], -1.0, 0, 0))
    for count in (2, 3, 6):
        for tol, option in ((1e-6, 1), (0.1, 0)):
            requests.append(contigous_request(count, tol, option))
