"""Deterministic BRepGraph algorithm request corpus for the TKBRep oracle (`br_balgo`).

A request is `br_balgo script nactions (action)...`: the script is a `BRepScript` of
tools/tkbrep_brep_corpus.py; the actions are the `br_bedit` actions (run on the current
of two graphs) and the algorithm actions of tools/oracle/tkbrep_probe_brepgraph_algo.cxx
and crates/tkbrep/examples/br_probe/brep_graph_algo.rs: 240 Copy::Perform into the other
graph, 241 CopyNode into the other graph, 242 CopyNode inside the current graph, 243
Compact::Perform, 244 Deduplicate::Perform, 245 switch the current graph and 246 the
self Copy::Perform. The probes then print both graphs: definitions, references, UIDs,
layers with the history records, caches and shapes.
"""
from __future__ import annotations
from tkbrep_brep_corpus import BRepScript, FORWARD, REVERSED, text
from tkbrep_brepgraph_corpus import ADD, PERFORM
from tkbrep_brepgraph_cache_corpus import meshed_solid_scene
from tkbrep_brepgraph_edit_corpus import HISTORY_ON, LOG, triangle_actions, triangle_geometry
from tkbrep_breptools_corpus import solid_scene

SOLID, SHELL, FACE, WIRE, EDGE, VERTEX, COMPOUND, COMPSOLID, COEDGE = range(9)
PRODUCT, OCCURRENCE = 10, 11
COPY, SHARE, DROP = range(3)
CACHE_DROP, CACHE_FRESH = range(2)
PERFORM_COPY, COPY_NODE, COPY_SELF, COMPACT, DEDUP, SWITCH, PERFORM_SELF = range(240, 247)
REMOVE, ADD_VERTEX, SET_OWNER, CACHE_WRITE = 208, 120, 62, 98
# Shapes().Add option bits of `bg_options`: 1 CreateAutoProduct.
AUTO_PRODUCT = 1
ANGULAR, CONFUSION = 1e-12, 1e-7


def request(script: BRepScript, actions: list[list[float]]) -> str:
    numbers = [*script.numbers(), len(actions), *[v for action in actions for v in action]]
    return f'br_balgo {text(numbers)}'


def policies(geom: int, mesh: int, cache: int) -> list[int]:
    return [geom, mesh, cache]


def dedup(analyze: bool, history: bool, merge: bool, comp: float = ANGULAR,
          hashing: float = CONFUSION) -> list[float]:
    return [DEDUP, int(analyze), int(history), int(merge), comp, hashing]


def full_copy_requests(requests: list[str]) -> None:
    """Identity copies of the meshed solid scene under every policy combination, the
    source carrying a logging layer, history, a lock owner and cache meshes."""
    s, geometry = meshed_solid_scene()
    compound = s.ids['compound']
    source = [LOG, HISTORY_ON, [PERFORM, compound, 0], [SET_OWNER, 0, FACE, 0, 1],
              [CACHE_WRITE, 0, geometry['tri']], [REMOVE, EDGE, 3]]
    for geom in (COPY, SHARE, DROP):
        for mesh in (COPY, SHARE, DROP):
            cache = CACHE_FRESH if geom == mesh else CACHE_DROP
            requests.append(request(s, [*source, [PERFORM_COPY, *policies(geom, mesh, cache)]]))
    # The self copy, an empty source and a copy back into the (now non-empty) source.
    requests.append(request(s, [[PERFORM_COPY, *policies(COPY, COPY, CACHE_DROP)], [PERFORM_SELF],
                                *source, [PERFORM_SELF], [PERFORM_COPY, *policies(SHARE, SHARE, CACHE_DROP)],
                                [SWITCH], [PERFORM_COPY, *policies(COPY, COPY, CACHE_FRESH)]]))


def appended_copy_requests(requests: list[str]) -> None:
    """Copies into a non-empty target: node copies first, then the whole graph, with
    removed nodes and references and an assembly."""
    s, geometry = meshed_solid_scene()
    ids = s.ids
    source = [LOG, HISTORY_ON, [ADD, ids['compound'], AUTO_PRODUCT], [REMOVE, FACE, 1],
              [SET_OWNER, 0, VERTEX, 2, 1]]
    nodes = [(FACE, 0), (FACE, 1), (SOLID, 0), (SHELL, 0), (WIRE, 0), (EDGE, 0), (VERTEX, 0),
             (COMPOUND, 0), (PRODUCT, 0), (OCCURRENCE, 0), (COEDGE, 0), (9, 0), (FACE, 99)]
    for geom, mesh in ((COPY, COPY), (SHARE, DROP), (DROP, SHARE)):
        actions = [*source]
        actions += [[COPY_NODE, kind, index, *policies(geom, mesh, CACHE_FRESH)] for kind, index in nodes]
        actions += [[PERFORM_COPY, *policies(geom, mesh, CACHE_FRESH)]]
        requests.append(request(s, actions))
    # Into a target that holds its own scene first.
    requests.append(request(s, [[SWITCH], [PERFORM, ids['solid'], 0], [SWITCH], *source,
                                [PERFORM_COPY, *policies(COPY, COPY, CACHE_DROP)]]))


def self_copy_requests(requests: list[str]) -> None:
    """Sub-graphs duplicated inside their own graph, with the layers copied in place."""
    s, _ = meshed_solid_scene()
    ids = s.ids
    source = [LOG, HISTORY_ON, [ADD, ids['compound'], AUTO_PRODUCT], [SET_OWNER, 0, EDGE, 1, 1]]
    nodes = [(FACE, 0), (SOLID, 0), (COMPOUND, 0), (PRODUCT, 0), (EDGE, 2), (VERTEX, 3), (FACE, 50)]
    actions = [*source]
    actions += [[COPY_SELF, kind, index, *policies(COPY, SHARE, CACHE_FRESH)] for kind, index in nodes]
    requests.append(request(s, actions))
    requests.append(request(s, [*source, [REMOVE, FACE, 0], [COPY_SELF, SOLID, 0, *policies(SHARE, COPY, CACHE_DROP)],
                                [COPY_SELF, COMPOUND, 0, *policies(DROP, DROP, CACHE_DROP)]]))


def compact_requests(requests: list[str]) -> None:
    """Compaction after removals, with and without history, root products and the two
    cache policies; a vertex added afterwards shows the restarted UID counter."""
    s, geometry = meshed_solid_scene()
    ids = s.ids
    cache = [[CACHE_WRITE, 0, geometry['tri']], [CACHE_WRITE, 1, geometry['tri2']]]
    removals = [[REMOVE, FACE, 0], [REMOVE, EDGE, 5], [REMOVE, VERTEX, 7]]
    for history in (1, 0):
        for mode in (CACHE_DROP, CACHE_FRESH):
            requests.append(request(s, [LOG, HISTORY_ON, [PERFORM, ids['compound'], 0], [SET_OWNER, 0, FACE, 1, 1],
                                        *cache, *removals, [COMPACT, history, mode],
                                        [ADD_VERTEX, 0.0, 0.0, 9.0, 1e-7], [COMPACT, history, mode]]))
    # Root products: only the reachable definitions survive.
    requests.append(request(s, [LOG, HISTORY_ON, [ADD, ids['compound'], AUTO_PRODUCT], [ADD, ids['high'], AUTO_PRODUCT],
                                [REMOVE, FACE, 0], [COMPACT, 1, CACHE_FRESH], [SWITCH],
                                [ADD, ids['solid'], 0], [REMOVE, SHELL, 0], [COMPACT, 1, CACHE_DROP]]))
    # Nothing removed, an empty graph, and a removed solid with its subgraph.
    requests.append(request(s, [[COMPACT, 1, CACHE_DROP], [PERFORM, ids['compound'], 0], [COMPACT, 1, CACHE_DROP],
                                [210, SOLID, 0], [COMPACT, 1, CACHE_DROP]]))


def duplicate_scene() -> tuple[BRepScript, list[list[float]]]:
    """A triangle built twice from scratch (vertices, edges, wires, faces) on equal
    but distinct lines and planes, the second one sharing the first face's plane
    handle for one face, in a shell."""
    s = BRepScript()
    g = triangle_geometry(s)
    twin = triangle_geometry(s)
    actions = [*triangle_actions(g)]
    second = triangle_actions(twin)
    # Renumber the second triangle's vertices, edges, coedges and wires.
    for action in second:
        code = action[0]
        if code == 121:
            action = [121, action[1] + 3, action[2] + 3, *action[3:]]
        elif code == 122:
            action = [122, action[1] + 3, action[2]]
        elif code in (124, 182) and len(action) > 2:
            action = [code, action[1], *[v + 3 for v in action[2:]]]
        elif code == 181:
            action = [181, action[1] + 3, 1]
        elif code == 175:
            action = [175, action[1] + 3, *action[2:]]
        elif code == 125:
            action = [125, action[1], 1, *action[3:]]
        actions.append(action)
    actions += [[126], [136, 0, 0, FORWARD], [136, 0, 1, REVERSED]]
    return s, actions


def dedup_requests(requests: list[str]) -> None:
    """Geometry canonicalization and entity merging, analyzed and applied."""
    s, build = duplicate_scene()
    for analyze in (True, False):
        for merge in (False, True):
            requests.append(request(s, [LOG, HISTORY_ON, *build, dedup(analyze, True, merge)]))
    requests.append(request(s, [LOG, *build, dedup(False, False, True, 1e-9, 1e-6), dedup(False, True, True)]))
    # A removed edge with coincident vertices: the vertex redirect takes each edge's
    # guard before its removed check, so the merge raises Standard_ProgramError.
    requests.append(request(s, [LOG, HISTORY_ON, *build, [REMOVE, EDGE, 5], dedup(False, True, True)]))
    # The ingested scenes: a merge pass over shared topology, and a removed edge
    # without coincident vertices (no redirect runs).
    m, _ = meshed_solid_scene()
    requests.append(request(m, [LOG, HISTORY_ON, [ADD, m.ids['compound'], AUTO_PRODUCT], dedup(False, True, True)]))
    requests.append(request(m, [LOG, HISTORY_ON, [PERFORM, m.ids['compound'], 0], [REMOVE, EDGE, 1],
                                dedup(False, True, True), dedup(False, True, False)]))
    # Deduplicate then compact the merged scene.
    requests.append(request(s, [LOG, HISTORY_ON, *build, dedup(False, True, True), [COMPACT, 1, CACHE_DROP]]))


def append_tkbrep_brepgraph_algo_requests(requests: list[str]) -> None:
    """Append the BRepGraph algorithm requests."""
    requests.append(request(BRepScript(), [[PERFORM_COPY, *policies(COPY, COPY, CACHE_DROP)],
                                           [COPY_NODE, FACE, 0, *policies(COPY, COPY, CACHE_DROP)],
                                           [COMPACT, 1, CACHE_DROP], dedup(False, True, True), [PERFORM_SELF]]))
    full_copy_requests(requests)
    appended_copy_requests(requests)
    self_copy_requests(requests)
    compact_requests(requests)
    dedup_requests(requests)
