"""Deterministic BRepGraph layer and cache request corpus for the TKBRep oracle (`br_blayer`).

A request is `br_blayer script nactions (action)...`: the script is a `BRepScript` of
tools/tkbrep_brep_corpus.py and the actions (tools/oracle/tkbrep_probe_brepgraph_layers*.cxx
and crates/tkbrep/examples/br_probe/brep_graph_layers*.rs) ingest scene shapes (the
`br_bgraph` codes 0-5), record history, dispatch registry events to logging layers, attach
supplement shapes, import BRepTools histories, bind cache entries and fill a standalone
deferred layer. The probes then print the registry, the history events and every query,
the supplement attachments, the cache entries and generations, the deferred entries in
map order and every node's shapes, and, after the graph is dropped, the detachment events.
"""
from __future__ import annotations
from tkbrep_brep_corpus import BRepScript, INTERNAL, text
from tkbrep_brepgraph_corpus import (ADD, ADD_UNDER, AUTO, COMPOUND_KIND, FACE_KIND, PERFORM,
                                     SHELL_KIND, SOLID_KIND, TRACK)
from tkbrep_breptools_corpus import solid_scene
from tkbrep_brepgraph_corpus import free_scene

EDGE_KIND, VERTEX_KIND, WIRE_KIND, PRODUCT_KIND = 4, 5, 3, 10
FACE_REF, WIRE_REF, VERTEX_REF, CHILD_REF = 1, 2, 3, 5
MODIFIED, GENERATED, DELETED, REPLACED = range(4)
ALL_KINDS, ALL_REF_KINDS = 0xFFF, 0x7F


def label(word: str) -> list[int]:
    """A text as `count byte...`."""
    return [len(word), *word.encode()]


def nodes(*pairs) -> list[int]:
    """A node list as `count (kind index)...`."""
    return [len(pairs), *[v for pair in pairs for v in pair]]


def uids(*pairs) -> list[int]:
    """A UID list as `count (kind counter)...` (item UIDs pass triples)."""
    return [len(pairs), *[v for pair in pairs for v in pair]]


def request(script: BRepScript, actions: list[list[float]]) -> str:
    numbers = [*script.numbers(), len(actions), *[v for action in actions for v in action]]
    return f'br_blayer {text(numbers)}'


def history_requests(requests: list[str]) -> None:
    """Every history record kind on the solid scene, with its edge cases."""
    s = solid_scene()
    compound = s.ids['compound']
    e = [(EDGE_KIND, i) for i in range(8)]
    records = [
        [10, *label('Step1'), *e[0], *nodes(e[1], e[0], e[1]), MODIFIED],
        [10, *label('Step2'), *e[1], *nodes(e[2]), GENERATED],
        [10, *label('Gone'), *e[3], *nodes(), GENERATED],
        [10, *label('Bad'), *e[4], *nodes(e[5]), DELETED],
        [11, *label('Batch'), *nodes(e[2], e[3], e[4]), *nodes(e[5], e[3], e[6]), *label('x'), MODIFIED],
        [11, *label('Short'), *nodes(e[0], e[1]), *nodes(e[6]), *label(''), GENERATED],
        [11, *label('Empty'), *nodes(), *nodes(e[1]), *label(''), MODIFIED],
        [12, *label('Del'), *nodes(e[7], e[7], (FACE_KIND, 1), (FACE_KIND, 99))],
        [13, *label('Swap'), FACE_KIND, 0, FACE_KIND, 1],
        [13, *label('Invalid'), 9, 0, FACE_KIND, 1],
        [14, *label('Batch2'), *nodes((VERTEX_KIND, 0), (VERTEX_KIND, 1)),
         *nodes((VERTEX_KIND, 2), (VERTEX_KIND, 1)), *label('info')],
        [21, EDGE_KIND, 2],
        [21, 9, 0],
    ]
    requests.append(request(s, [[PERFORM, compound, 0], *records]))
    requests.append(request(s, [[ADD, compound, AUTO | TRACK], *records[:5], [19, 0], *records[5:8],
                                [19, 1], [20], *records[8:]]))


def uid_requests(requests: list[str]) -> None:
    """The UID and item-UID records, standalone and on the graph's UIDs."""
    s = solid_scene()
    actions = [
        [PERFORM, s.ids['compound'], 0],
        [15, *label('U'), FACE_KIND, 1, *uids((FACE_KIND, 2), (FACE_KIND, 0), (FACE_KIND, 2)), REPLACED],
        [15, *label('U0'), FACE_KIND, 3, *uids((FACE_KIND, 0)), MODIFIED],
        [15, *label('Uempty'), EDGE_KIND, 4, *uids(), GENERATED],
        [15, *label('Uinv'), FACE_KIND, 0, *uids((FACE_KIND, 2)), MODIFIED],
        [16, *label('D'), *uids((EDGE_KIND, 1), (EDGE_KIND, 0), (EDGE_KIND, 1))],
        [16, *label('Dnone'), *uids((EDGE_KIND, 0))],
        [17, *label('I'), 0, EDGE_KIND, 2, *uids((0, EDGE_KIND, 3), (1, FACE_REF, 1), (0, EDGE_KIND, 3)), GENERATED],
        [17, *label('Iref'), 1, WIRE_REF, 2, *uids((1, WIRE_REF, 5)), REPLACED],
        [17, *label('Iempty'), 0, VERTEX_KIND, 7, *uids(), MODIFIED],
        [18, *label('ID'), *uids((0, FACE_KIND, 1), (1, CHILD_REF, 1), (0, FACE_KIND, 0))],
        [15, *label('Sol'), SOLID_KIND, 1, *uids((SOLID_KIND, 1)), GENERATED],
    ]
    requests.append(request(s, actions))


def event_requests(requests: list[str]) -> None:
    """Logging layers through registration, graph mutations, dispatch and unregistration."""
    s = solid_scene()
    ids = s.ids
    actions = [
        [29, 0, ALL_KINDS, ALL_REF_KINDS],
        [29, 1, 0, 0],
        [29, 2, 1 << FACE_KIND, 1 << FACE_REF],
        [ADD, ids['compound'], 0],
        [ADD_UNDER, ids['high'], SHELL_KIND, 0, 0],
        [ADD_UNDER, ids['low'], COMPOUND_KIND, 0, TRACK],
        [21, FACE_KIND, 0], [22, FACE_KIND, 0, FACE_KIND, 1], [23, WIRE_REF, 1],
        [24, FACE_KIND, 1], [24, EDGE_KIND, 1], [25, *nodes((EDGE_KIND, 0)), 1 << EDGE_KIND],
        [25, *nodes((FACE_KIND, 0), (EDGE_KIND, 0)), 0], [26, FACE_REF, 0], [26, VERTEX_REF, 0],
        [27], [28], [29, 1, 0, 0], [30, 2], [30, 9], [60],
    ]
    requests.append(request(s, actions))
    requests.append(request(s, [[29, 3, ALL_KINDS, 0], [PERFORM, ids['compound'], 0], [30, 0], [30, 0], [24, FACE_KIND, 0]]))


def supplement_requests(requests: list[str]) -> None:
    """Population with a registered supplement layer, the supplement route of Add, and
    the layer API with its callbacks."""
    s = solid_scene()
    ids = s.ids
    requests.append(request(s, [[34], [PERFORM, ids['compound'], 0]]))
    f = free_scene()
    requests.append(request(f, [[34], [PERFORM, f.ids['compound'], 0], [ADD, f.ids['inner_face'], AUTO]]))
    requests.append(request(f, [[ADD, f.ids['compound'], AUTO | TRACK], [ADD, f.ids['inner_face'], 0]]))
    vertex = ids['low_vertices'][0]
    requests.append(request(s, [
        [ADD, ids['compound'], 0],
        [ADD_UNDER, vertex, FACE_KIND, 0, 0], [ADD_UNDER, vertex, EDGE_KIND, 1, 0],
        [ADD_UNDER, ids['low_edges'][0], SHELL_KIND, 0, 0], [ADD_UNDER, ids['high'], SOLID_KIND, 0, 0],
        [ADD_UNDER, vertex, WIRE_KIND, 0, 0], [ADD_UNDER, vertex, FACE_KIND, 50, 0],
    ]))
    generic, face_vertex, compound_aux = 7, 2, 6
    requests.append(request(s, [
        [PERFORM, ids['compound'], 0],
        [31, FACE_KIND, 0, face_vertex, vertex], [31, FACE_KIND, 0, compound_aux, vertex],
        [31, EDGE_KIND, 0, generic, ids['high']], [31, WIRE_KIND, 0, generic, vertex],
        [31, FACE_KIND, 1, generic, -1], [32, EDGE_KIND, 1, 9, generic, vertex],
        [32, EDGE_KIND, 1, 9, generic, vertex], [32, EDGE_KIND, 1, 0, generic, vertex],
        [31, EDGE_KIND, 1, 0, vertex], [31, FACE_KIND, 1, face_vertex, vertex],
        [33, 1], [33, 1], [33, 10],
        [22, FACE_KIND, 1, FACE_KIND, 0], [22, EDGE_KIND, 1, FACE_KIND, 2], [21, EDGE_KIND, 0],
        [22, FACE_KIND, 0, FACE_KIND, 0], [34],
    ]))
    requests.append(request(s, [[34], [PERFORM, ids['solid'], 0], [31, SOLID_KIND, 0, 3, vertex], [28],
                                [31, SOLID_KIND, 0, 3, vertex], [60], [34]]))


def import_requests(requests: list[str]) -> None:
    """Shapes().AddWithHistory and the cross-graph Absorb over BRepTools histories.

    Absorb walks its input map, whose TopTools_ShapeMapHasher order follows the TShape
    addresses (natively as in Rust), so each history gives Modified or Generated images to
    one input only (the removed inputs are recorded together at the end)."""
    s = solid_scene()
    ids = s.ids
    low, high, moved, shell = ids['low'], ids['high'], ids['moved'], ids['shell']
    edge = ids['low_edges'][0]
    ops = [4, 0, low, moved, 1, low, moved, 2, ids['high_edges'][1], 0, 0, shell, moved]
    requests.append(request(s, [[ADD, ids['compound'], AUTO], [35, moved, 1, COMPOUND_KIND, 0, *ops, -1]]))
    requests.append(request(s, [[ADD, ids['compound'], 0],
                                [35, moved, 1, COMPOUND_KIND, 0, 4, 0, shell, moved, 0, edge, edge, 1, low, low,
                                 2, low, 0, AUTO | TRACK]]))
    requests.append(request(s, [[ADD, ids['compound'], 0], [19, 0], [35, moved, 1, COMPOUND_KIND, 0, *ops, 0]]))
    requests.append(request(s, [[ADD, ids['compound'], 0], [35, moved, 0, 0, 0], [35, -1, 1, COMPOUND_KIND, 0, *ops, 0]]))
    requests.append(request(s, [[ADD, ids['compound'], 0], [ADD, moved, 0],
                                [36, 1, COMPOUND_KIND, 0, 2, FACE_KIND, 2, FACE_KIND, 3, *ops],
                                [36, 0, 1, COMPOUND_KIND, 0, *ops]]))


def cache_requests(requests: list[str]) -> None:
    """Probe cache services: registration, replacement, entries across mutations."""
    s = solid_scene()
    ids = s.ids
    binds = [[43, 0, SHELL_KIND, 0], [43, 1, SOLID_KIND, 0], [43, 0, FACE_KIND, 1], [43, 2, FACE_REF, 0],
             [43, 2, VERTEX_REF, 0], [43, 2, CHILD_REF, 0], [43, 3, 1, WIRE_REF, 1], [43, 3, 0, EDGE_KIND, 2],
             [43, 0, FACE_KIND, 99], [43, 3, 1, 9, 0]]
    requests.append(request(s, [[41], [43, 0, FACE_KIND, 0], [PERFORM, ids['compound'], 0], *binds,
                                [ADD_UNDER, ids['high'], SHELL_KIND, 0, 0], [44], [40], *binds[:3]]))
    requests.append(request(s, [[ADD, ids['compound'], AUTO], [40], [40], *binds[:4], [42], [41], [41],
                                *binds[4:6], [45], [44], [41]]))
    # After Clear only node entries are checked: a reference past the end is read
    # unchecked natively (`BRepGraph_RefId::IsRemoved`).
    requests.append(request(s, [[40], [ADD, ids['compound'], 0], *binds[:3], [60], *binds[:2]]))


def deferred_requests(requests: list[str]) -> None:
    """A standalone deferred layer: registration, merging, limits, bulk mode and moves."""
    s = BRepScript()
    mesh, geometry, topology = 2, 1, 3

    def rep(kind: int, role: int, name: str, index: float) -> list[float]:
        return [kind, role, *label(name), index]

    face1, face2, ref = [0, FACE_KIND, 1], [0, FACE_KIND, 2], [1, FACE_REF, 4]
    nine = [9, *[v for i in range(9) for v in rep(mesh, i, 'm', i)]]
    actions = [
        [50, *face1, *label('STEP'), *label('#1'), 2, *rep(mesh, 0, 'a', 4294967295.0), *rep(mesh, 0, 'a', 4294967295.0)],
        [50, *face1, *label('STEP'), *label('#1'), 1, *rep(geometry, 1, 'g', 3)],
        [50, *face1, *label('STEP'), *label('#1'), 1, *rep(geometry, 1, 'g', 3)],
        [50, *face1, *label('IGES'), *label('#2'), 0],
        [50, *ref, *label('P'), *label('k'), 1, *rep(topology, 2, 't', 0)],
        [50, 0, 9, 0, *label('P'), *label('k'), 1, *rep(topology, 2, 't', 0)],
        [51, *face2, *label('D'), *label('d'), 2, *rep(mesh, 0, 'x', 1), *rep(mesh, 0, 'x', 1)],
        [51, *face2, *label('D'), *label('d'), 1, *rep(mesh, 0, 'y', 1)],
        [50, 0, EDGE_KIND, 5, *label('Big'), *label('b'), *nine],
        [51, 0, EDGE_KIND, 6, *label('Big'), *label('b'), *nine],
        [53], [53], [50, 0, VERTEX_KIND, 1, *label('V'), *label('v'), 1, *rep(mesh, 0, '', 0)], [54],
        [58, 10], [54], [54],
        [55, FACE_KIND, 1, FACE_KIND, 2], [55, FACE_KIND, 2, FACE_KIND, 3], [55, FACE_KIND, 7, FACE_KIND, 1],
        [56, FACE_REF, 4], [52, 0, VERTEX_KIND, 1], [52, 0, VERTEX_KIND, 1],
    ]
    requests.append(request(s, actions))
    requests.append(request(s, [*actions[:7], [57], *actions[4:6]]))


def append_tkbrep_brepgraph_layer_requests(requests: list[str]) -> None:
    """Append the BRepGraph layer and cache requests."""
    requests.append(request(BRepScript(), []))
    history_requests(requests)
    uid_requests(requests)
    event_requests(requests)
    supplement_requests(requests)
    import_requests(requests)
    cache_requests(requests)
    deferred_requests(requests)
