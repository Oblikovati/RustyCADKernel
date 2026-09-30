"""WireDivide histories retain the original edge scene and real splitting criteria."""

from itertools import product
from tkshhealing_scene_edges import EdgeScene, edge_scene
from tkshhealing_corpus_analysis_edge import edge_face_pairs
from tkshhealing_corpus_analysis_basic import request


def wire_record(
    scene: EdgeScene,
    edge: int,
    face: int,
    modes: tuple[int, int],
    mode: int = 2,
    tolerance: float = 1e-7,
    context: int = 1,
    count: int = 2,
) -> str:
    """Serialize one real geometry history; wire_record(scene, edge, face, (1,1))."""
    return request(
        "sh_wire_divide",
        scene.program.tokens(),
        *modes,
        edge,
        face,
        mode,
        tolerance,
        context,
        count
    )


def wire_divide_candidates() -> list[str]:
    """Return every candidate, including inputs whose native behavior needs accounting."""
    scene = edge_scene()
    requests = basic_wire_histories(scene)
    append_wire_states(requests, scene)
    append_wire_internal(requests)
    append_wire_independent_ranges(requests)
    append_wire_small_prefix(requests)
    append_wire_uninitialized(requests)
    return requests


def basic_wire_histories(scene: EdgeScene) -> list[str]:
    """Repeat division over every existing edge/face geometry pair."""
    requests: list[str] = []
    for (edge, face), modes, mode, tolerance in product(
        edge_face_pairs(), product([0, 1], repeat=2), [0, 1, 2], [1e-7, 2.0]
    ):
        requests.append(
            wire_record(
                scene, scene.shapes[edge], scene.shapes[face], modes, mode, tolerance
            )
        )
    return requests


def append_wire_states(requests: list[str], scene: EdgeScene) -> None:
    """Exercise all public configuration overloads with native geometry."""
    for surface, location, modes, mode in product(
        ["plane", "cyl", "sphere"],
        ["id", "move"],
        product([0, 1], repeat=2),
        [0, 2, -7],
    ):
        requests.append(
            request(
                "sh_wire_state",
                scene.program.tokens(),
                scene.shapes["line"],
                scene.shapes["f_plane"],
                scene.surfaces[surface],
                scene.locations[location],
                *modes,
                mode
            )
        )


def append_wire_internal(requests: list[str]) -> None:
    """Retain internal vertices at endpoints, inside and outside split pieces."""
    for parameter, orientation, representation in product(
        [-0.1, 0.0, 0.25, 0.5, 0.75, 1.0, 1.1], [2, 3], [0, 1]
    ):
        scene = edge_scene()
        edge = scene.shapes["line"]
        attach_wire_internal(scene, edge, parameter, orientation, representation)
        requests.append(wire_record(scene, edge, scene.shapes["f_plane"], (1, 1)))


def attach_wire_internal(
    scene: EdgeScene, edge: int, parameter: float, orientation: int, representation: int
) -> None:
    """Store the same internal point on a curve or pcurve representation."""
    vertex = scene.program.vertex((parameter, 0.0, 0.0))
    scene.program.add(edge, vertex, orientation)
    if representation == 0:
        scene.program.vertex_on_curve(vertex, parameter, edge)
    else:
        scene.program.vertex_on_pcurve(vertex, parameter, edge, scene.surfaces["plane"])


def append_wire_uninitialized(requests: list[str]) -> None:
    """Keep default-null context and pcurve-only internal-vertex behavior visible."""
    scene = edge_scene()
    requests.append(
        wire_record(
            scene,
            scene.shapes["line"],
            scene.shapes["f_plane"],
            (1, 1),
            context=0,
            count=1,
        )
    )
    edge = scene.shapes["pc_only"]
    attach_wire_internal(scene, edge, 0.25, 2, 1)
    requests.append(wire_record(scene, edge, scene.shapes["f_plane"], (1, 1), count=1))


def wire_divide_partition() -> tuple[list[str], list[str]]:
    """Require every declared crashing request to remain in the candidate corpus."""
    from native_crash_accounting import partition_crash_requests
    from tkshhealing_wire_divide_crashes import NATIVE_CRASH_HASHES

    return partition_crash_requests(wire_divide_candidates(), NATIVE_CRASH_HASHES)


def append_wire_divide_requests(requests: list[str]) -> None:
    """Add exact paths; tools/verify_wire_divide.py separately requires both crashes."""
    exact, _ = wire_divide_partition()
    requests.extend(exact)


def append_wire_independent_ranges(requests: list[str]) -> None:
    """Synchronize native independent [2,5] and [10,16] parameter ranges."""
    from tkshhealing_corpus_edge_divide import independent_range_edge

    scene = edge_scene()
    edge = independent_range_edge(scene)
    for parameter, orientation in [(2.0, 0), (5.0, 1)]:
        add_wire_endpoint(scene, edge, parameter, parameter, orientation)
    for modes, face in product(
        product([0, 1], repeat=2), [scene.shapes["f_plane"], -1]
    ):
        requests.append(wire_record(scene, edge, face, modes))


def add_wire_endpoint(
    scene: EdgeScene, edge: int, point: float, parameter: float, orientation: int
) -> None:
    """Build a real endpoint and store its curve parameter."""
    vertex = scene.program.vertex((point, 0.0, 0.0))
    scene.program.add(edge, vertex, orientation)
    scene.program.vertex_on_curve(vertex, parameter, edge)


def append_wire_small_prefix(requests: list[str]) -> None:
    """A collapsed Bezier prefix precedes a long piece, exercising midpoint repair."""
    for degree, tolerance in product([2, 4, 6], [1e-7, 4.0, 10.0, 50.0, 100.0]):
        scene = edge_scene()
        edge = wire_bezier_prefix(scene, degree)
        requests.append(
            wire_record(
                scene, edge, scene.shapes["f_plane"], (1, 1), tolerance=tolerance
            )
        )


def wire_bezier_prefix(scene: EdgeScene, degree: int) -> int:
    """Make identical spatial and planar Beziers with repeated start poles."""
    poles3 = [0.0, 0.0, 0.0] * degree + [100.0, 0.0, 0.0]
    poles2 = [0.0, 0.0] * degree + [100.0, 0.0]
    spatial = scene.program.curve(3, degree + 1, *poles3)
    planar = scene.program.pcurve(3, degree + 1, *poles2)
    edge = scene.program.edge()
    scene.program.edge_curve(edge, spatial)
    scene.program.edge_pcurve(edge, planar, scene.surfaces["plane"])
    scene.program.edge_range(edge, 0.0, 1.0)
    for point, parameter, orientation in [(0.0, 0.0, 0), (100.0, 1.0, 1)]:
        add_wire_endpoint(scene, edge, point, parameter, orientation)
    return edge
