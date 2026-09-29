"""The editing requests of the BRepTools corpus: ReShape, History, Substitution, Quilt and
PurgeLocations commands over the solid scene of tools/tkbrep_breptools_corpus.py.

A request is `op script ncmd (command)...` (tools/oracle/tkbrep_probe_breptools_reshape.cxx);
the command codes are those of the probe. The scene's shape indices come from the
`ids` the scene records.
"""
from __future__ import annotations
from typing import Callable
from tkbrep_brep_corpus import BRepScript, text


def request(op: str, script: BRepScript, commands: list[list[float]]) -> str:
    """One `op script ncmd (command)...` line."""
    flat = [c for command in commands for c in command]
    return f'{op} {text(script.numbers())} {len(commands)} {text(flat)}'.rstrip()


def reshape_commands(ids: dict) -> list[list[list[float]]]:
    """ReShape sequences: replacements, removals, merges, chains, vertex copies, errors."""
    le, he, lv = ids['low_edges'], ids['high_edges'], ids['low_vertices']
    return [
        [[0, ids['low'], ids['high']], [1, le[1]], [4, lv[0], -1.0], [5, lv[1], 0.5, 0.25, 0.0, 0.01]],
        [[0, le[0], he[0]], [0, he[0], he[1]], [2, 2, le[2], le[3], he[2]], [1, lv[2]]],
        [[3, 0], [0, ids['moved'], ids['high']], [0, ids['shell'], ids['solid']], [6], [1, ids['low']]],
        [[4, ids['low'], 1.0], [0, -1, ids['high']], [1, -1], [2, 0, ids['low']], [0, lv[3], lv[0]]],
    ]


def history_commands(ids: dict) -> list[list[list[float]]]:
    """History sequences on the first history, the second one and their merge."""
    le, he = ids['low_edges'], ids['high_edges']
    low, high = ids['low'], ids['high']
    return [
        [[1, low, high], [0, le[0], low], [2, le[1]], [1, le[2], he[2]], [4, le[2], he[3]],
         [3, le[0], high], [0, ids['shell'], high], [1, ids['solid'], ids['compound']]],
        [[1, low, high], [11, high, low], [12, he[0]], [10, he[1], ids['solid']], [20],
         [15, 0], [21], [2, high], [1, high, low]],
        [[1, le[0], he[0]], [11, he[0], he[1]], [11, he[0], he[2]], [21], [5, 0], [20]],
    ]


def subst_commands(ids: dict) -> list[list[list[float]]]:
    """Substitutions of edges, vertices and faces, built from several roots."""
    le, he, lv = ids['low_edges'], ids['high_edges'], ids['low_vertices']
    return [
        [[0, le[0], 1, he[0]], [0, lv[1], 0], [1, ids['low']], [1, ids['compound']]],
        [[0, ids['low'], 2, ids['high'], ids['low']], [1, ids['solid']], [0, ids['low'], 0]],
        [[0, le[1], 2, he[1], he[2]], [1, ids['shell']], [2], [1, ids['low']]],
    ]


def quilt_commands(ids: dict) -> list[list[list[float]]]:
    """Quilts of the two squares, with and without bound edges and vertices."""
    le, he, lv = ids['low_edges'], ids['high_edges'], ids['low_vertices']
    return [
        [[2, ids['low']], [2, ids['high']]],
        [[0, le[0], he[0]], [1, lv[0], lv[1]], [2, ids['low']], [2, ids['high']], [2, ids['moved']]],
        [[2, ids['solid']], [2, ids['compound']], [0, le[1], ids['low']]],
    ]


def append_tkbrep_breptools_edit_requests(requests: list[str], solid: Callable[[], BRepScript],
                                          others: list[BRepScript]) -> None:
    """Append the editing requests over the solid scene (and PurgeLocations on others)."""
    ids = solid().ids
    for op, sequences in (('br_tools_reshape', reshape_commands(ids)),
                          ('br_tools_history', history_commands(ids)),
                          ('br_tools_subst', subst_commands(ids)),
                          ('br_tools_quilt', quilt_commands(ids))):
        for commands in sequences:
            requests.append(request(op, solid(), commands))
    for script in [solid(), *others]:
        requests.append(f'br_tools_purge {text(script.numbers())}')
