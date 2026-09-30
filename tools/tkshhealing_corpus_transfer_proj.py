"""Projection transfer over real 3D/pcurve pairs and non-manifold vertex representations."""

from itertools import product
from tkshhealing_corpus_analysis_edge import edge_face_pairs
from tkshhealing_corpus_transfer_parameters import transfer_record
from tkshhealing_corpus_edge_divide import independent_range_edge
from tkshhealing_scene_edges import EdgeScene, edge_scene
from tktopalgo_shape_program import INTERNAL, EXTERNAL, FORWARD, REVERSED


def projection_record(
    scene: EdgeScene,
    edge: int,
    face: int,
    constructor: int,
    operations: list[list[float | int]],
) -> str:
    """Use the shared transfer protocol with projection-specific operations."""
    return transfer_record(scene, edge, face, constructor, operations).replace(
        "sh_transfer_parameters ", "sh_transfer_parameters_proj ", 1
    )


def projection_history(
    edge: int, face: int, force: int, tolerance: float
) -> list[list[float | int]]:
    """Increasing and descending sequences expose interval and closed-tail handling."""
    operations = [[5, force], [4, tolerance]]
    operations += [
        [0, value, direction]
        for value, direction in product([-1.0, -0.0, 0.25, 0.5, 1.0, 8.0], [0, 1])
    ]
    operations += [
        [1, direction, 4, *values]
        for values, direction in product(
            [[0.0, 0.25, 0.75, 1.0], [1.0, 0.75, 0.25, 0.0], [0.0, 0.5, 0.5, 1.0]],
            [0, 1],
        )
    ]
    operations += [[1, 1, 0], [3, edge, -1], [0, 0.5, 1], [3, edge, face], [0, 0.5, 0]]
    return operations


def append_transfer_projection_candidates(requests: list[str]) -> None:
    """Account every candidate before comparison; no native failure is silently removed."""
    scene = edge_scene()
    for (edge_name, face_name), constructor, policy in product(
        edge_face_pairs(), [0, 1], [(0, 1.0), (1, 1.0), (0, 0.0)]
    ):
        edge, face = scene.shapes[edge_name], scene.shapes[face_name]
        operations = projection_history(edge, face, *policy)
        requests.append(projection_record(scene, edge, face, constructor, operations))
    append_projected_ranges(requests)
    append_vertex_copies(requests)
    append_nonlinear_transfers(requests)
    append_independent_vertex_copies(requests)
    append_pcurve_only_vertex_requests(requests)


def append_projected_ranges(requests: list[str]) -> None:
    """Projection range correction includes independent maps and collapsed intervals."""
    scene = edge_scene()
    scene.shapes["independent"] = independent_range_edge(scene)
    pairs = [
        ("line", "f_plane"),
        ("bspline", "f_plane"),
        ("circle", "f_cyl"),
        ("seam_rev", "f_cyl"),
        ("independent", "f_plane"),
    ]
    for (edge_name, face_name), bounds, direction, force in product(
        pairs,
        [(0.0, 1.0), (0.25, 0.75), (0.75, 0.25), (0.5, 0.5), (2.5, 4.5), (11.0, 15.0)],
        [0, 1],
        [0, 1],
    ):
        edge, face = scene.shapes[edge_name], scene.shapes[face_name]
        operations = [[5, force], [2, *bounds, direction]]
        requests.append(projection_record(scene, edge, face, 1, operations))


def represented_vertex(scene: EdgeScene, orientation: int, representations: int) -> int:
    """Attach distinct stored representations while retaining the geometric point."""
    program = scene.program
    vertex = program.oriented(program.vertex((0.4, 0.2, 0.3), 1e-7), orientation)
    if representations > 0:
        program.vertex_on_curve(vertex, 0.4, scene.shapes["line"])
        program.vertex_on_face(vertex, 0.4, 0.2, scene.shapes["f_plane"])
    if representations > 1:
        program.vertex_on_curve(vertex, 0.7, scene.shapes["long"])
        program.vertex_on_pcurve(
            vertex, 0.4, scene.shapes["line"], scene.surfaces["plane"], 0
        )
        program.vertex_on_face(vertex, 0.2, 0.3, scene.shapes["f_cyl"])
    return vertex


def append_vertex_copies(requests: list[str]) -> None:
    """Both overloads retain unrelated representations and update their own geometry."""
    targets = [
        (6, "line", "line"),
        (6, "long", "line"),
        (6, "bspline", "line"),
        (6, "moved", "line"),
        (7, "f_plane", "f_plane"),
        (7, "f_plane_moved", "f_plane"),
        (7, "f_cyl", "f_plane"),
    ]
    for orientation, representations, (operation, target, source) in product(
        [FORWARD, REVERSED, INTERNAL, EXTERNAL], [0, 1, 2], targets
    ):
        scene = edge_scene()
        vertex = represented_vertex(scene, orientation, representations)
        operations = [[operation, vertex, scene.shapes[target], scene.shapes[source]]]
        requests.append(
            projection_record(
                scene, scene.shapes["line"], scene.shapes["f_plane"], 0, operations
            )
        )


def nonlinear_transfer_edge(
    scene: EdgeScene, family: int, location: int
) -> tuple[int, int]:
    """The spatial line and a cubic pcurve describe the same path with different speed."""
    p = scene.program
    curve = nonlinear_pcurve(scene, family)
    edge = p.edge()
    p.edge_curve(edge, scene.curves["line"], location)
    p.edge_pcurve(edge, curve, scene.surfaces["plane"], location)
    p.edge_range(edge, 2.0, 5.0, True)
    p.edge_range_on(edge, scene.surfaces["plane"], location, 0.0, 1.0)
    return edge, p.face(scene.surfaces["plane"], location)


def nonlinear_pcurve(scene: EdgeScene, family: int) -> int:
    """Build wrapped polynomial pcurves, including preserved undefined-offset fixtures."""
    p = scene.program
    if family < 2:
        curve = p.pcurve(3, 4, 2.0, 0.0, 2.0, 0.0, 2.0, 0.0, 5.0, 0.0)
    else:
        curve = p.pcurve(
            6, 3, 4, 2.0, 0.0, 2.0, 0.0, 2.0, 0.0, 5.0, 0.0, 2, 0.0, 4, 1.0, 4
        )
    if family == 1:
        curve = p.pcurve(4, curve, 0.0, 1.0)
    if family == 4:
        curve = p.pcurve(
            6, 3, 4, 2.0, 0.0, 2.25, 0.0, 3.0, 0.0, 5.0, 0.0, 2, 0.0, 4, 1.0, 4
        )
    if family in (3, 4):
        curve = p.pcurve(5, curve, 0.05)
    return curve


def append_nonlinear_transfers(requests: list[str]) -> None:
    """Projection must differ from the linear estimate, including located and wrapped pcurves."""
    for family, moved, policy in product(
        range(5), [False, True], [(0, 1.0), (1, 1.0), (0, 0.0)]
    ):
        scene = edge_scene()
        location = scene.locations["move"] if moved else 0
        edge, face = nonlinear_transfer_edge(scene, family, location)
        operations = [[5, policy[0]], [4, policy[1]]]
        operations += [[0, value, 1] for value in [2.0, 2.375, 3.5, 5.0]]
        operations += [[0, value, 0] for value in [0.0, 0.25, 0.5, 1.0]]
        operations += [[1, 1, 4, 2.0, 2.375, 3.5, 5.0], [1, 0, 4, 0.0, 0.25, 0.5, 1.0]]
        operations += [
            [2, *bounds, direction]
            for bounds, direction in product(
                [(2.375, 4.625), (0.5, 0.75), (0.5, 0.5), (0.0, 0.0), (1.0, 1.0)],
                [0, 1],
            )
        ]
        requests.append(projection_record(scene, edge, face, 1, operations))


def append_independent_vertex_copies(requests: list[str]) -> None:
    """Replay direct Rust vertex geometry and tolerance regressions natively."""
    scene = edge_scene()
    p = scene.program
    edge = independent_range_edge(scene)
    face = scene.shapes["f_plane"]
    vertex = p.oriented(p.vertex((3.5, 0.0, 0.0), 1e-7), INTERNAL)
    p.vertex_on_curve(vertex, 3.5, edge)
    p.vertex_on_face(vertex, 3.5, 0.0, face)
    target_edge = independent_range_edge(scene)
    p.edge_range(target_edge, 2.0, 6.0, True)
    target_face = p.face(p.surface(0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0))
    operations = [
        [6, vertex, edge, edge],
        [6, vertex, target_edge, edge],
        [7, vertex, target_face, face],
        [5, 1],
        [2, 1e101, 2e101, 0],
    ]
    requests.append(projection_record(scene, edge, face, 1, operations))


def append_pcurve_only_vertex_requests(requests: list[str]) -> None:
    """WireDivide reaches pcurve-only targets; retain their native null-curve crashes."""
    for source, target in [
        ("pc_only", "line"),
        ("pc_only", "pc_only"),
        ("line", "pc_only"),
    ]:
        scene = edge_scene()
        program = scene.program
        vertex = program.oriented(program.vertex((0.4, 0.0, 0.0)), INTERNAL)
        program.vertex_on_pcurve(
            vertex, 0.4, scene.shapes[source], scene.surfaces["plane"], 0
        )
        operations = [[6, vertex, scene.shapes[target], scene.shapes[source]]]
        requests.append(
            projection_record(
                scene, scene.shapes["line"], scene.shapes["f_plane"], 1, operations
            )
        )


def transfer_projection_partition() -> tuple[list[str], list[str]]:
    """Manifest membership accounts for every request and rejects disappeared crash cases."""
    from native_crash_accounting import partition_crash_requests
    from tkshhealing_transfer_projection_crashes import NATIVE_CRASH_HASHES

    candidates: list[str] = []
    append_transfer_projection_candidates(candidates)
    return partition_crash_requests(candidates, NATIVE_CRASH_HASHES)


def append_transfer_projection_requests(requests: list[str]) -> None:
    """Append exact cases; verify_transfer_projection.py separately requires both crash replays."""
    exact, _ = transfer_projection_partition()
    requests.extend(exact)
