"""ComposeShell segment storage over real edges, shared handles and patch intervals."""
from itertools import product
from tkshhealing_scene_edges import EdgeScene, edge_scene
from tkshhealing_corpus_analysis_basic import request


def segment_record(scene: EdgeScene, edges: list[int], manifold: int, constructor: int,
                   orientation: int, operations: list[list[int]]) -> str:
    """Serialize operations before they execute so exceptions preserve input alignment."""
    steps: list[int] = []
    for target, opcode, *operands in operations:
        steps.extend([target, opcode, len(operands), *operands])
    return request("sh_wire_segment", scene.program.tokens(), manifold, len(edges), *edges,
                   constructor, orientation, len(operations), *steps)


def wire_segment_candidates() -> list[str]:
    """All histories, including the explicitly accounted native null-handle failure."""
    scene = edge_scene()
    requests: list[str] = []
    for manifold, constructor, orientation in product([0, 1], [0, 1], range(4)):
        for names in [[], ["line"], ["closed"], ["line_rev", "bare"], ["seam", "seam_rev"]]:
            edges = [scene.shapes[name] for name in names]
            operations = segment_history(scene)
            requests.append(segment_record(scene, edges, manifold, constructor, orientation, operations))
    append_segment_bounds(requests, scene)
    requests.append(segment_record(scene, [scene.shapes["line"]], 0, 1, 2, [[0, 16]]))
    requests.append(segment_record(scene, [], 0, 2, 0, []))
    return requests


def segment_history(scene: EdgeScene) -> list[list[int]]:
    """Copy, detach, reload, and edit through both segment and exposed wire handles."""
    edge, reverse, vertex = scene.shapes["line"], scene.shapes["line_rev"], scene.shapes["v0"]
    return [
        [0, 3, 0, edge], [1, 12], [0, 6, 1, 2, 3, 4, 4],
        [1, 7, 1, 3], [1, 8, 1, 2], [0, 9, 1, 5], [0, 10, 1, 2],
        [0, 11, vertex], [1, 2, 3], [1, 5, 1, reverse], [0, 0],
        [0, 15], [1, 14], [0, 1], [0, 13, edge, 0],
        [0, 4, 0, reverse, -2, -1, 4, 5], [1, 12],
        [0, 4, 1, -1, 8, 7, 4, 3], [0, 11, -1], [1, 0],
    ]


def append_segment_bounds(requests: list[str], scene: EdgeScene) -> None:
    """Invalid ranks preserve native exceptions and any prior edge insertion."""
    for index, opcode, bounds in product([-2, 0, 1, 2, 3, 99], [3, 4, 5, 6, 7, 8, 9, 10], [(-4, 8, -7, 2), (4, 4, 3, 4)]):
        edge = scene.shapes["line"]
        operands = segment_operands(opcode, index, edge, bounds)
        operations = [[0, 13, edge, 0], [0, opcode, *operands], [1, 12], [1, 0]]
        requests.append(segment_record(scene, [edge], 0, 1, 0, operations))


def segment_operands(opcode: int, index: int, edge: int, bounds: tuple[int, int, int, int]) -> list[int]:
    """Encode the overload-specific operand shape; segment_operands(4,1,edge,bounds)."""
    if opcode in [3, 5]:
        return [index, edge]
    if opcode == 4:
        return [index, edge, *bounds]
    if opcode == 6:
        return [index, *bounds]
    return [index, bounds[opcode - 7]]
