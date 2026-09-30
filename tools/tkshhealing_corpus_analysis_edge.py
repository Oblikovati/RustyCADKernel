"""Deterministic ShapeAnalysis_Edge queries and checks on exact native geometry.

Use append_analysis_edge_requests(requests) to add independent edge requests.
The scene retains curve-only, p-curve-only, shifted, reversed, seam and degenerate edges.
"""
from __future__ import annotations
from itertools import product
from tkshhealing_scene_edges import EdgeScene, edge_scene
from tkshhealing_corpus_analysis_basic import request


def edge_face_pairs() -> tuple[tuple[str, str], ...]:
    """Name pairs cover matching and deliberately mismatched representations."""
    return (('line', 'f_plane'), ('line_off', 'f_plane'), ('circle', 'f_cyl'),
            ('circle_up', 'f_cyl'), ('circle_back', 'f_cyl'), ('seam', 'f_cyl'),
            ('pc_only', 'f_plane'), ('bare', 'f_plane'), ('deg', 'f_sphere'),
            ('moved', 'f_plane_moved'), ('closed', 'f_cyl'), ('late', 'f_cyl'),
            ('bspline', 'f_plane'), ('bspline_nsp', 'f_plane'),
            ('pc_flat1', 'f_plane'), ('pc_flat2', 'f_plane'), ('pc_point', 'f_plane'),
            ('line_rev', 'f_plane'), ('circle_rev', 'f_cyl'), ('seam_rev', 'f_cyl'),
            ('circle_back_rev', 'f_cyl_rev'), ('pc_only_rev', 'f_plane'),
            ('line_located', 'f_plane'), ('line', 'f_cyl'))


def emit_edge_request(out: list[str], scene: EdgeScene, *tail: float | int) -> None:
    """Append one request; emit_edge_request(out, scene, 0, edge, 1)."""
    out.append(request('sh_edgeana', scene.program.tokens(), *tail))


def query_edge_requests(out: list[str], scene: EdgeScene) -> None:
    """Curve and vertex queries include the vertex-free edge without checking its points."""
    edges = [name for name in scene.shapes if not name.startswith(('v', 'f_'))]
    for name in edges:
        index = scene.shapes[name]
        emit_edge_request(out, scene, 3, index)
        for oriented in (0, 1):
            emit_edge_request(out, scene, 0, index, oriented)
    for edge, face in edge_face_pairs():
        for oriented in (0, 1):
            emit_edge_request(out, scene, 1, scene.shapes[edge], scene.shapes[face], oriented)


def face_check_requests(out: list[str], scene: EdgeScene) -> None:
    """Endpoints, tolerance, curve agreement and sampled deviation for every face pair."""
    for edge, face in edge_face_pairs():
        e, f = scene.shapes[edge], scene.shapes[face]
        for operation in (9, 11):
            emit_edge_request(out, scene, operation, e, f)
        emit_edge_request(out, scene, 10, e)
        for precision, vertex in product((-1.0, 0.0, 1e-4, 0.01), (0, 1, 2)):
            emit_edge_request(out, scene, 6, e, precision, vertex)
            emit_edge_request(out, scene, 7, e, f, precision, vertex)
        for samples in (2, 3, 23):
            emit_edge_request(out, scene, 13, e, samples)
            emit_edge_request(out, scene, 14, e, f, samples)


def tangent_requests(out: list[str], scene: EdgeScene) -> None:
    """Both ends use derivative and chord paths, including vanishing derivatives."""
    for edge, face in edge_face_pairs():
        for at_end, fraction in product((0, 1), (0.0, 1e-8, 1e-6, 0.01, 0.25, 1.0)):
            emit_edge_request(out, scene, 4, scene.shapes[edge], scene.shapes[face], at_end, fraction)


def surface_check_requests(out: list[str], scene: EdgeScene) -> None:
    """Exercise the explicit surface/location overloads, including seam orientation."""
    pairs = (('line', 'plane', 'id'), ('circle', 'cyl', 'id'), ('seam_rev', 'cyl', 'id'),
             ('moved', 'plane', 'move'), ('circle_back', 'cyl', 'id'), ('bare', 'plane', 'id'))
    for edge, surface, location in pairs:
        e, s, loc = scene.shapes[edge], scene.surfaces[surface], scene.locations[location]
        emit_edge_request(out, scene, 12, e, s, loc)
        for oriented in (0, 1):
            emit_edge_request(out, scene, 2, e, s, loc, oriented)
        for precision, vertex in product((-1.0, 1e-3), (0, 1, 2)):
            emit_edge_request(out, scene, 8, e, s, loc, precision, vertex)
        for at_end, fraction in product((0, 1), (0.0, 0.01)):
            emit_edge_request(out, scene, 5, e, s, loc, at_end, fraction)


def range_and_overlap_requests(out: list[str], scene: EdgeScene) -> None:
    """Periodic and bounded p-curve ranges, then overlap tolerances and domains."""
    for curve in ('line', 'circle', 'trimmed', 'bspline', 'flat1', 'flat2', 'point'):
        for first, last in ((0.0, 1.0), (-1e-10, 1.0), (-1.0, 1.0), (0.0, 7.0), (1.0, 0.0)):
            emit_edge_request(out, scene, 15, first, last, scene.pcurves[curve])
    for pair, tolerance, domain in product((('long', 'near'), ('long', 'far'), ('line', 'line_rev')),
                                            (0.0, 1e-7, 1e-3, 1.0), (0.0, 0.1, 0.9)):
        emit_edge_request(out, scene, 16, scene.shapes[pair[0]], scene.shapes[pair[1]], tolerance, domain)


def append_analysis_edge_requests(requests: list[str]) -> None:
    """Add every query family; append_analysis_edge_requests(requests)."""
    scene = edge_scene()
    query_edge_requests(requests, scene)
    face_check_requests(requests, scene)
    tangent_requests(requests, scene)
    surface_check_requests(requests, scene)
    range_and_overlap_requests(requests, scene)
    preserved_output_requests(requests, scene)


def preserved_output_requests(out: list[str], scene: EdgeScene) -> None:
    """Nonzero output values distinguish untouched fields from assigned zeroes."""
    program = scene.program.tokens()
    for edge, face in (('bare', 'f_cyl'), ('novert', 'f_cyl'), ('deg', 'f_sphere'),
                       ('line', 'f_plane'), ('pc_point', 'f_plane')):
        e, f = scene.shapes[edge], scene.shapes[face]
        for first, last in ((17.0, -23.0), (-0.0, 1e-8)):
            out.append(request('sh_edgeoutputs', program, 0, e, f, 0, 0.0, first, last, 3.0, -5.0))
            out.append(request('sh_edgeoutputs', program, 1, e, scene.surfaces['cyl'], 0, 1, 0.01, first, last, 3.0, -5.0))
            out.append(request('sh_edgeoutputs', program, 2, e, f, first, last))
            out.append(request('sh_edgeoutputs', program, 3, e, first, last))
            out.append(request('sh_edgeoutputs', program, 4, e, f, 23, first))
            out.append(request('sh_edgeoutputs', program, 5, e, 23, first))
