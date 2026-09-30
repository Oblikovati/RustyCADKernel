"""Edge criteria over real curve/pcurve geometry and native no-op approximation state."""

from itertools import product
from tkshhealing_scene_edges import EdgeScene, edge_scene
from tkshhealing_corpus_analysis_edge import edge_face_pairs
from tkshhealing_corpus_analysis_basic import request


def divide_record(
    scene: EdgeScene, modes: tuple[int, int], operations: list[list[float | int]]
) -> str:
    """Serialize one independent criterion history; divide_record(scene,(0,0),[[2]])."""
    flattened = [value for operation in operations for value in operation]
    return request(
        "sh_edge_divide", scene.program.tokens(), *modes, len(operations), *flattened
    )


def append_edge_divide_requests(requests: list[str]) -> None:
    """Keep original scene geometry and exercise all public edge criterion methods."""
    scene = edge_scene()
    for (edge, face), modes in product(edge_face_pairs(), product([0, 1], repeat=2)):
        for operations in edge_histories(scene.shapes[edge], scene.shapes[face]):
            requests.append(divide_record(scene, modes, operations))
    append_edge_piece_histories(requests, scene)
    append_edge_tool_replacements(requests, scene)
    append_independent_ranges(requests)


def edge_histories(edge: int, face: int) -> list[list[list[int]]]:
    """Keep free and face-bound histories independent; edge_histories(1,2)."""
    return [
        [[0, face], [1, edge], [2], [3, 1, 2], [3, 0, 4], [1, edge]],
        [[3, 0, 2], [1, edge], [0, face], [1, edge], [0, -1], [1, edge]],
    ]


def append_edge_piece_histories(requests: list[str], scene: EdgeScene) -> None:
    """Build geometric pieces after virtual criteria and mutate exposed knot arrays."""
    pairs = [
        ("line", "f_plane"),
        ("bspline", "f_plane"),
        ("circle", "f_cyl"),
        ("seam_rev", "f_cyl"),
    ]
    for (edge, face), modes, segment in product(
        pairs, product([0, 1], repeat=2), [0, 1]
    ):
        operations = [
            [0, scene.shapes[face]],
            [1, scene.shapes[edge]],
            [6, segment],
            [1, scene.shapes[edge]],
            [4, 1, 1, 0.1],
            [4, 0, 2, 0.9],
            [4, 0, 0, 8.0],
            [2],
        ]
        requests.append(divide_record(scene, modes, operations))


def append_edge_tool_replacements(requests: list[str], scene: EdgeScene) -> None:
    """Replace tools after discovery and preserve detached parameter handles."""
    for modes in product([-1, 0, 1], repeat=2):
        operations = [
            [0, scene.shapes["f_plane"]],
            [1, scene.shapes["line"]],
            [5, *modes],
            [2],
            [5, 1, 1],
            [1, scene.shapes["bspline"]],
        ]
        requests.append(divide_record(scene, (0, 0), operations))


def append_fix_small_curve_requests(requests: list[str]) -> None:
    """The native base Approx preserves real geometry and parameter bits."""
    scene = edge_scene()
    geometries = [
        (scene.curves["line"], scene.pcurves["line"], scene.pcurves["line_back"]),
        (scene.curves["bspline"], scene.pcurves["bspline"], -1),
        (scene.curves["circle"], scene.pcurves["circle"], scene.pcurves["trimmed"]),
        (-1, -1, -1),
    ]
    for geometry, bounds, status in product(
        geometries, [(-0.0, 16.0), (12.0, -3.0), (-1e100, 1e100)], [0, 1, 4, 256, 257]
    ):
        operations = small_curve_history(scene, status)
        flattened = [value for operation in operations for value in operation]
        requests.append(
            request(
                "sh_fix_small_curves",
                scene.program.tokens(),
                *geometry,
                *bounds,
                len(operations),
                *flattened
            )
        )


def small_curve_history(scene: EdgeScene, status: int) -> list[list[int]]:
    """Preserve status through initialization and approximation; small_curve_history(scene,1)."""
    return [
        [2],
        [0, scene.shapes["line"], scene.shapes["f_plane"]],
        [1, 1, 1],
        [3, status],
        [2],
        [0, scene.shapes["seam_rev"], scene.shapes["f_cyl"]],
        [2],
        [1, 0, 1],
        [0, -1, -1],
        [1, 0, 0],
        [2],
    ]


def append_independent_ranges(requests: list[str]) -> None:
    """Split identical geometry in independent 3D and 2D parameter spaces."""
    scene = edge_scene()
    edge = independent_range_edge(scene)
    for modes, segment in product(product([0, 1], repeat=2), [0, 1]):
        operations = [[0, scene.shapes["f_plane"]], [1, edge], [6, segment], [1, edge]]
        requests.append(divide_record(scene, modes, operations))


def independent_range_edge(scene: EdgeScene) -> int:
    """The pcurve [10,16] describes the spatial line [2,5]."""
    program = scene.program
    pcurve = program.pcurve(6, 1, 2, 2.0, 0.0, 5.0, 0.0, 2, 10.0, 2, 16.0, 2)
    edge = program.edge()
    program.edge_curve(edge, scene.curves["line"])
    program.edge_pcurve(edge, pcurve, scene.surfaces["plane"])
    program.edge_range(edge, 2.0, 5.0, True)
    program.range_on_face(edge, scene.shapes["f_plane"], 10.0, 16.0)
    program.edge_flags(edge, False, False, False)
    return edge
