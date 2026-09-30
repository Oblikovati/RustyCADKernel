"""Exact linear parameter maps and native edge-range copying over actual curves."""

from itertools import product
from tkshhealing_corpus_analysis_basic import request
from tkshhealing_corpus_analysis_edge import edge_face_pairs
from tkshhealing_corpus_edge_divide import independent_range_edge
from tkshhealing_scene_edges import EdgeScene, edge_scene


def transfer_record(
    scene: EdgeScene,
    edge: int,
    face: int,
    constructor: int,
    operations: list[list[float | int]],
) -> str:
    """Serialize one independent transfer history; transfer_record(scene,0,1,0,[])."""
    flattened = [value for operation in operations for value in operation]
    return request(
        "sh_transfer_parameters",
        scene.program.tokens(),
        edge,
        face,
        constructor,
        len(operations),
        *flattened
    )


def parameter_history(edge: int, face: int) -> list[list[float | int]]:
    """Exercise both directions, sequence ownership, tolerance storage and reinitialization."""
    values = [-0.0, 0.0, 1e-12, -8.0, 2.0, 3.5, 5.0, 10.0, 16.0, 1e100]
    operations = [[0, value, direction] for value, direction in product(values, [0, 1])]
    operations += [[1, direction, len(values), *values] for direction in [0, 1]]
    operations += [
        [1, 1, 0],
        [4, 0.125],
        [3, edge, -1],
        [0, 3.5, 1],
        [3, edge, face],
        [0, 3.5, 1],
    ]
    return operations


def append_transfer_parameter_requests(requests: list[str]) -> None:
    """Append valid public API histories, including independent and collapsed spans."""
    scene = edge_scene()
    for (edge_name, face_name), constructor in product(edge_face_pairs(), [0, 1]):
        edge, face = scene.shapes[edge_name], scene.shapes[face_name]
        requests.append(
            transfer_record(
                scene, edge, face, constructor, parameter_history(edge, face)
            )
        )
    append_transfer_range_requests(requests, scene)
    append_transfer_span_requests(requests)


def append_transfer_range_requests(requests: list[str], scene: EdgeScene) -> None:
    """Ranges copy to separate topology and preserve the original geometry."""
    pairs = [
        ("line", "f_plane"),
        ("bspline", "f_plane"),
        ("circle", "f_cyl"),
        ("seam_rev", "f_cyl"),
    ]
    bounds = [(0.25, 0.75), (0.75, 0.25), (-2.0, 8.0), (0.5, 0.5)]
    for (edge_name, face_name), constructor, interval, is_2d in product(
        pairs, [0, 1], bounds, [0, 1]
    ):
        edge, face = scene.shapes[edge_name], scene.shapes[face_name]
        operations = [[2, *interval, is_2d], [0, 0.5, 1]]
        requests.append(transfer_record(scene, edge, face, constructor, operations))


def append_transfer_span_requests(requests: list[str]) -> None:
    """Native small/reversed spatial spans use unit scale; collapsed pcurves yield zero scale."""
    spans = [
        (2.0, 5.0, 10.0, 16.0),
        (2.0, 2.0, 10.0, 16.0),
        (5.0, 2.0, 10.0, 16.0),
        (2.0, 5.0, 10.0, 10.0),
        (0.0, 1e-310, 10.0, 16.0),
        (2.0, 5.0, 16.0, 10.0),
    ]
    for (first, last, first_2d, last_2d), constructor in product(spans, [0, 1]):
        scene = edge_scene()
        edge, face = independent_range_edge(scene), scene.shapes["f_plane"]
        scene.program.edge_range(edge, first, last, True)
        scene.program.range_on_face(edge, face, first_2d, last_2d)
        operations = parameter_history(edge, face)
        requests.append(transfer_record(scene, edge, face, constructor, operations))
    append_independent_range_transfers(requests)


def append_independent_range_transfers(requests: list[str]) -> None:
    """Copy compatible subranges between independent curve parameterizations."""
    scene = edge_scene()
    edge, face = independent_range_edge(scene), scene.shapes["f_plane"]
    for constructor, bounds, is_2d in product(
        [0, 1], [(2.5, 4.5), (4.5, 2.5), (11.0, 15.0), (15.0, 11.0)], [0, 1]
    ):
        requests.append(
            transfer_record(scene, edge, face, constructor, [[2, *bounds, is_2d]])
        )
