"""Deterministic BRepGraph lock and cache request corpus for the TKBRep oracle (`br_bcache`).

A request is `br_bcache script nactions (action)...`: the script is a `BRepScript` of
tools/tkbrep_brep_corpus.py and the actions (tools/oracle/tkbrep_probe_brepgraph_cache*.cxx
and crates/tkbrep/examples/br_probe/brep_graph_cache*.rs) ingest scene shapes (the
`br_bgraph` codes 0-5), set, unset, dispatch and scope owners in the lock layer, register
deferred items in an attached deferred layer, query and seed the derived-state cache,
write, bump, clear and refill the mesh cache (with a logging driver), and copy the layers
and caches into a second graph or migrate the layers in place. The probes then print,
for both graphs, the registries, every node's and reference's owner, the deferred
entries, every derived-state answer, three mesh-cache slots and the mesh view, and the
driver logs.
"""
from __future__ import annotations
from tkbrep_brep_corpus import BRepScript, FORWARD, REVERSED, text
from tkbrep_brepgraph_corpus import ADD_UNDER, APPEND, COMPOUND_KIND, FACE_KIND, PERFORM, SHELL_KIND
from tkbrep_breptools_corpus import COMPOUND, cylinder_scene, mesh_scene, solid_scene, square_edge, square_face

WIRE_KIND, EDGE_KIND, VERTEX_KIND, COEDGE_KIND, SOLID_KIND = 3, 4, 5, 8, 0
SHELL_REF, FACE_REF, WIRE_REF, VERTEX_REF, SOLID_REF, CHILD_REF = 0, 1, 2, 3, 4, 5
COPY, COMPACT = 0, 1
TARGET = 116


def node(kind: int, index: int) -> list[int]:
    """A node item: `0 kind index`."""
    return [0, kind, index]


def nid(kind: int, index: int) -> list[int]:
    """A node id: `kind index`."""
    return [kind, index]


def ref(kind: int, index: int) -> list[int]:
    """A reference item: `1 kind index`."""
    return [1, kind, index]


def label(word: str) -> list[int]:
    return [len(word), *word.encode()]


def request(script: BRepScript, actions: list[list[float]]) -> str:
    numbers = [*script.numbers(), len(actions), *[v for action in actions for v in action]]
    return f'br_bcache {text(numbers)}'


def meshed_solid_scene() -> tuple[BRepScript, dict[str, int]]:
    """The solid scene (a persistent triangulation, 3D polygon and polygon on
    triangulation), a persistent 2D polygon on a face, and spare cache meshes."""
    s = solid_scene()
    ids = s.ids
    polygon2d = s.polygon2d([(0.0, 0.0), (1.0, 0.0)])
    s.do(20, ids['low_edges'][2], polygon2d, ids['low'])
    geometry = dict(
        tri=s.triangulation([(0.0, 0.0, 0.0), (2.0, 0.0, 0.0), (2.0, 2.0, 0.0)], [(1, 2, 3)], 0),
        tri2=s.triangulation([(0.0, 0.0, 1.0), (1.0, 0.0, 1.0), (0.0, 1.0, 1.0)], [(1, 2, 3)], 1),
        poly=s.polygon3d([(0.0, 0.0, 0.0), (0.25, 0.0, 0.0), (1.0, 0.0, 0.0)]),
        poly2=s.polygon3d([(0.0, 1.0, 0.0), (1.0, 1.0, 0.0)]),
        poly2d=s.polygon2d([(0.0, 0.0), (0.5, 0.5), (1.0, 1.0)]),
        nodes=s.nodes([1, 2]),
        nodes2=s.nodes([2, 3]),
        polygon2d=polygon2d)
    return s, geometry


def free_edge_scene() -> BRepScript:
    """A face and two free edges (one sharing the face's vertex) in a compound."""
    s = BRepScript()
    face, edges, _, vertices = square_face(s, 1.0, 0.0)
    far = s.vertex((3.0, 0.0, 0.0))
    free = square_edge(s, face, vertices[1], far, (1.0, 0.0, 0.0), (3.0, 0.0, 0.0), (1.0, 0.0), (3.0, 0.0), False)
    other = s.vertex((3.0, 3.0, 0.0))
    lonely = square_edge(s, face, far, other, (3.0, 0.0, 0.0), (3.0, 3.0, 0.0), (3.0, 0.0), (3.0, 3.0), False)
    compound = s.make(COMPOUND)
    s.add(compound, face, FORWARD)
    s.add(compound, free, FORWARD)
    s.add(compound, lonely, REVERSED)
    s.ids = dict(compound=compound, face=face, edge=edges[0], free=free)
    return s


def lock_requests(requests: list[str]) -> None:
    """Root owners, overlaps, descendant absorption and conflicts, removals, dispatch,
    scopes and a detached layer on the solid scene."""
    s = solid_scene()
    compound = s.ids['compound']
    actions = [
        [PERFORM, compound, 0],
        [61, *node(SHELL_KIND, 0), 1, 1], [61, *node(FACE_KIND, 0), 2, 1], [61, *node(FACE_KIND, 0), 1, 0],
        [61, *ref(FACE_REF, 0), 1, 1], [61, *node(SHELL_KIND, 0), 1, 1], [61, *node(SHELL_KIND, 0), 3, 1],
        [61, *node(COMPOUND_KIND, 0), 1, 1], [61, *ref(CHILD_REF, 0), 2, 1], [61, *node(EDGE_KIND, 7), 1, 0],
        [62, *node(FACE_KIND, 1), 4], [62, *ref(VERTEX_REF, 2), 5],
        [72, *node(FACE_KIND, 1), 1], [72, *node(FACE_KIND, 1), 2], [72, *ref(WIRE_REF, 0), 1],
        [65, 10], [66],
        [64, *node(COMPOUND_KIND, 0), 2], [63, *node(SOLID_KIND, 0)], [63, *node(COMPOUND_KIND, 0)],
        [61, *node(FACE_KIND, 1), 2, 1], [61, *node(SHELL_KIND, 0), 1, 1],
        [61, *node(EDGE_KIND, 0), 3, 1], [61, *ref(VERTEX_REF, 0), 3, 1], [61, *ref(VERTEX_REF, 1), 4, 1],
        [73, *node(FACE_KIND, 0), 6], [73, *ref(FACE_REF, 1), 7],
    ]
    requests.append(request(s, actions))
    requests.append(request(s, [
        *actions[:9],
        [67, SHELL_KIND, 0], [67, FACE_KIND, 0], [68, COMPOUND_KIND, 0, COMPOUND_KIND, 1],
        [69, CHILD_REF, 0], [69, VERTEX_REF, 2], [64, *ref(CHILD_REF, 1), 2],
        [61, *node(EDGE_KIND, 1), 5, 1], [70], [61, *node(WIRE_KIND, 1), 6, 1], [71],
        [61, *node(VERTEX_KIND, 0), 1, 1], [60],
    ]))


def deferred_requests(requests: list[str]) -> None:
    """An attached deferred layer locks, bulk-locks, unlocks and copies its items."""
    s = solid_scene()
    mesh_rep = [2, 0, *label('m'), 3]

    def register(code: int, item: list[int], reps: list[list[float]]) -> list[float]:
        return [code, *item, *label('STEP'), *label('#1'), len(reps), *[v for r in reps for v in r]]

    actions = [
        [PERFORM, s.ids['compound'], 0],
        register(74, node(FACE_KIND, 0), [mesh_rep]), register(74, node(FACE_KIND, 0), [mesh_rep]),
        register(74, ref(WIRE_REF, 1), [[1, 1, *label('g'), 0]]),
        [79, 4], [77], register(74, node(EDGE_KIND, 2), [mesh_rep]), register(75, node(EDGE_KIND, 3), [mesh_rep]),
        [78], [78],
        [61, *node(SHELL_KIND, 0), 1, 1], register(74, node(FACE_KIND, 1), [mesh_rep]),
        [76, *node(FACE_KIND, 0)], [76, *node(FACE_KIND, 0)],
        [TARGET, PERFORM, s.ids['compound'], 0], [113, COPY],
    ]
    requests.append(request(s, actions))
    requests.append(request(s, [*actions[:8], [67, EDGE_KIND, 2], [69, WIRE_REF, 1], [60]]))


def derived_requests(requests: list[str]) -> None:
    """Derived-state queries, seeded answers served from the cache, invalidation by a
    mutation or a Clear, Tool reads through the service and fresh entries copied."""
    s = solid_scene()
    ids = s.ids
    queries = [[80, e] for e in range(8)] + [[81, c] for c in range(8)] + [[82, c] for c in range(8)]
    actions = [
        [PERFORM, ids['compound'], 0], *queries,
        [83, 0], [83, 1], [85, 0], [84, 0, 0], [83, 0], [87, 0], [89, 0], [88, 0],
        [86], [83, 0], [84, 1, 0], [84, 7, 1], [83, 7],
        [ADD_UNDER, ids['high_edges'][0], WIRE_KIND, 0, 0], [83, 0], [85, 0],
        [ADD_UNDER, ids['low'], SHELL_KIND, 0, 0], [85, 0], [87, 1],
        [TARGET, PERFORM, ids['compound'], 0], [113, COPY],
    ]
    requests.append(request(s, actions))
    c = cylinder_scene()
    every = [[APPEND, i, 0] for i in range(c.shapes)]
    requests.append(request(c, [*every, *[[82, i] for i in range(4)], [80, 0], [85, 0],
                                *[[TARGET, *a] for a in every],
                                [114, COPY, 2, *node(EDGE_KIND, 0), *node(EDGE_KIND, 1),
                                 *node(COEDGE_KIND, 0), *node(COEDGE_KIND, 1)]]))


def mesh_requests(requests: list[str]) -> None:
    """Mesh-cache writes, freshness through bumps and topology edits, slots, drivers,
    Ensure and Needs, and the mesh view over persistent and cached meshes."""
    s, geo = meshed_solid_scene()
    compound = s.ids['compound']
    writes = [
        [98, 0, geo['tri']], [98, 1, geo['tri2']], [98, 9, geo['tri']], [100, 0, geo['poly']],
        [100, 2, -1], [102, 0, geo['nodes']], [102, 0, geo['nodes2']], [102, 1, -1],
        [103, 1, geo['poly2d']], [103, 2, geo['poly2d']],
    ]
    actions = [
        [PERFORM, compound, 0], *writes,
        [99, 0], [105, 1, 0], [104, 2], [101, 2],
        [107, 1, 1, geo['tri2']], [109, 1, 3, geo['poly2']], [110, 1, 4, geo['poly2d']],
        [107, 2, 0, geo['tri']], [93, 1],
        [ADD_UNDER, s.ids['high_edges'][0], WIRE_KIND, 1, 0],
        [108, 2, 0], [111, 1, 3], [112, 1, 4], [111, 5, 0],
        [TARGET, PERFORM, compound, 0], [113, COPY],
    ]
    requests.append(request(s, actions))
    requests.append(request(s, [
        [PERFORM, compound, 0], [94, 0], [97, *nid(FACE_KIND, 0), 0],
        [90, 0, 0, 7, geo['tri'], geo['poly'], 1], [97, *nid(COMPOUND_KIND, 0), 0], [94, 0], [94, 0],
        [98, 1, geo['tri2']], [95, *nid(FACE_KIND, 1), 0], [102, 0, geo['nodes']], [99, 0], [94, 0],
        [90, 0, 1, 8, geo['tri2'], -1, 0], [96, 2, *[FACE_KIND, 0], *[SHELL_KIND, 0], 0],
        [91, 0, 1], [91, 0, 0], [97, *nid(FACE_KIND, 0), 0], [90, 2, 2, 5, geo['tri'], geo['poly'], 1],
        [94, 2], [93, 2], [92, 0], [94, 0], [92, 3], [94, 3], [106],
    ]))
    f = free_edge_scene()
    requests.append(request(f, [
        [PERFORM, f.ids['compound'], 0],
        [90, 0, 0, 3, -1, -1, 1], [94, 0], [95, *nid(EDGE_KIND, 4), 0], [97, *nid(COMPOUND_KIND, 0), 0],
        [ADD_UNDER, f.ids['edge'], COMPOUND_KIND, 0, 0], [94, 0],
    ]))
    m = mesh_scene()
    requests.append(request(m, [[APPEND, i, 0] for i in range(m.shapes)]))


def copy_requests(requests: list[str]) -> None:
    """Every layer and cache copied through an explicit item map, and the layers of a
    graph migrated in place by a compaction."""
    s, geo = meshed_solid_scene()
    compound = s.ids['compound']
    seed = [
        [PERFORM, compound, 0], [61, *node(FACE_KIND, 1), 1, 1], [61, *ref(FACE_REF, 0), 2, 1],
        [74, *node(EDGE_KIND, 1), *label('P'), *label('k'), 1, 2, 0, *label('m'), 1],
        [80, 0], [84, 0, 0], [98, 0, geo['tri']], [102, 0, geo['nodes']], [103, 1, geo['poly2d']],
    ]
    swap = [4, *node(FACE_KIND, 0), *node(FACE_KIND, 1), *node(FACE_KIND, 1), *node(FACE_KIND, 0),
            *ref(FACE_REF, 0), *ref(FACE_REF, 1), *node(COEDGE_KIND, 0), *node(COEDGE_KIND, 4)]
    requests.append(request(s, [*seed, [TARGET, PERFORM, compound, 0], [114, COPY, *swap]]))
    requests.append(request(s, [*seed, [TARGET, PERFORM, compound, 0], [114, COMPACT, *swap]]))
    requests.append(request(s, [*seed, [115, *swap], [115, 0]]))


def append_tkbrep_brepgraph_cache_requests(requests: list[str]) -> None:
    """Append the BRepGraph lock, derived-state and mesh cache requests."""
    requests.append(request(BRepScript(), []))
    lock_requests(requests)
    deferred_requests(requests)
    derived_requests(requests)
    mesh_requests(requests)
    copy_requests(requests)
