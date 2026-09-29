"""Deterministic BRepGraph editor request corpus for the TKBRep oracle (`br_bedit`).

A request is `br_bedit script nactions (action)...`: the script is a `BRepScript` of
tools/tkbrep_brep_corpus.py and the actions (tools/oracle/tkbrep_probe_brepgraph_edit*.cxx
and crates/tkbrep/examples/br_probe/brep_graph_edit*.rs) ingest scene shapes (the
`br_bgraph` codes 0-5), run the editor operation groups (creation, containers, edges,
vertices, coedges, wires, faces, references, generic removal and replacement), the
mutation guards, ApplyModification, deferred invalidation and DeferredScope, the
supplement editor, RemoveShape and PromoteToPersistent, mixed with the `br_blayer`
logging layers, history and caches, the `br_bcache` lock owners and mesh-cache writes.
The probes then print the graph, the editor state, every layer and cache, the mesh
cache and view and every node's shapes.
"""
from __future__ import annotations
from tkbrep_brep_corpus import BRepScript, FORWARD, REVERSED, INTERNAL, text
from tkbrep_brepgraph_corpus import ADD, PERFORM
from tkbrep_brepgraph_cache_corpus import meshed_solid_scene
from tkbrep_breptools_corpus import solid_scene

SOLID, SHELL, FACE, WIRE, EDGE, VERTEX, COMPOUND, COMPSOLID, COEDGE = range(9)
PRODUCT, OCCURRENCE = 10, 11
SHELL_REF, FACE_REF, WIRE_REF, VERTEX_REF, SOLID_REF, CHILD_REF, OCCURRENCE_REF = range(7)
ALL_KINDS, ALL_REF_KINDS = 0xFFF, 0x7F
LOG, HISTORY_ON = [29, 0, ALL_KINDS, ALL_REF_KINDS], [19, 1]


def ids(*values: int) -> list[int]:
    """A typed id list as `count index...`."""
    return [len(values), *values]


def nodes(*pairs) -> list[int]:
    """A node list as `count (kind index)...`."""
    return [len(pairs), *[v for pair in pairs for v in pair]]


def label(word: str) -> list[int]:
    return [len(word), *word.encode()]


def request(script: BRepScript, actions: list[list[float]]) -> str:
    numbers = [*script.numbers(), len(actions), *[v for action in actions for v in action]]
    return f'br_bedit {text(numbers)}'


def triangle_geometry(s: BRepScript) -> dict[str, int]:
    """Three lines of a unit triangle in z=0, its plane, the pcurves and a spare curve."""
    return dict(
        lines=[s.line((0.0, 0.0, 0.0), (1.0, 0.0, 0.0)), s.line((1.0, 0.0, 0.0), (-1.0, 1.0, 0.0)),
               s.line((0.0, 1.0, 0.0), (0.0, -1.0, 0.0))],
        plane=s.plane((0.0, 0.0, 0.0), (0.0, 0.0, 1.0)),
        pcurves=[s.line2d((0.0, 0.0), (1.0, 0.0)), s.line2d((1.0, 0.0), (-1.0, 1.0)),
                 s.line2d((0.0, 1.0), (0.0, -1.0))],
        spare=s.circle((0.0, 0.0, 0.0), (0.0, 0.0, 1.0), 2.0),
        polygon=s.polygon3d([(0.0, 0.0, 0.0), (1.0, 0.0, 0.0)]),
        polygon2d=s.polygon2d([(0.0, 0.0), (1.0, 0.0)]))


def triangle_actions(g: dict[str, int]) -> list[list[float]]:
    """Vertices, edges, free coedges, a wire and a face built from scratch, then the
    coedges bound to the face with their pcurves (a coedge with a pcurve needs its face)."""
    corners = [(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (0.0, 1.0, 0.0)]
    actions: list[list[float]] = [[120, *p, 1e-7] for p in corners]
    ranges = [(0.0, 1.0), (0.0, 1.0), (0.0, 1.0)]
    for i in range(3):
        actions.append([121, i, (i + 1) % 3, g['lines'][i], *ranges[i], 1e-6])
    actions += [[122, i, FORWARD] for i in range(3)]
    actions += [[182, *ids(0, 1, 2)], [182, *ids(0, 2, 1)], [182, *ids()], [182, *ids(0, 0)],
                [124, *ids(0, 1, 2)], [125, g['plane'], 0, *ids(), 1e-7]]
    for i in range(3):
        actions += [[181, i, 0], [175, i, g['pcurves'][i], *ranges[i]]]
    return actions


def creation_requests(requests: list[str]) -> None:
    """A triangle face, its shell, solid, compound, compsolid, products and occurrences."""
    s = BRepScript()
    g = triangle_geometry(s)
    t = s.translation(0.0, 0.0, 3.0)
    actions = [LOG, HISTORY_ON, *triangle_actions(g),
               [126], [136, 0, 0, FORWARD], [137, 0, *ids(0, 0), *ids(REVERSED, FORWARD)],
               [127], [138, 0, 0, FORWARD], [139, 0, *ids(0), *ids(FORWARD, FORWARD)],
               [128, *nodes((SOLID, 0), (FACE, 0))], [140, 0, EDGE, 1, REVERSED],
               [141, 0, *nodes((VERTEX, 2), (WIRE, 0)), *ids(FORWARD, REVERSED)],
               [127], [129, *ids(1)], [142, 0, 1, REVERSED], [143, 0, *ids(1), *ids(FORWARD)],
               [201, 0, REVERSED], [202, 1, 0], [204, 0, 1], [204, 2, 0],
               [130, COMPOUND, 0, 0], [131], [132, 1, 0, t, -1], [133, 1, *ids(0, 0), *ids(0, t)],
               [134, 1], [134, 1], [132, 1, 1, 0, -1], [122, 1, REVERSED], [124, *ids(3)],
               [183, 0, *ids(2, 0, 1)], [184, 0, 3], [185, 0, 0, 1, 0], [185, 0, 0, 0, 1],
               [123, 0, 0, g['pcurves'][0], 0.0, 1.0, FORWARD], [123, 0, 5, g['pcurves'][0], 0.0, 1.0, FORWARD],
               [121, 0, 9, -1, 0.0, 1.0, 1e-7],
               [122, 1, INTERNAL], [140, 0, 12, 0, FORWARD], [223]]
    requests.append(request(s, actions))


def setter_requests(requests: list[str]) -> None:
    """Every setter on the ingested solid scene, notified to a logging layer."""
    s = solid_scene()
    g = triangle_geometry(s)
    t = s.translation(1.0, 0.0, 0.0)
    compound = s.ids['compound']
    setters = [
        [168, 0, 0.25, 0.0, 0.0], [169, 1, 1e-4], [170, 0, REVERSED], [171, 1, 3],
        [160, 0, 1e-3], [161, 1, 0.1, 0.9], [162, 2, g['spare'], 0.0, 1.5], [163, 3],
        [164, 4, g['polygon']], [165, 4], [166, 5, 2], [167, 5, 3],
        [172, 0, 0.0, 0.5], [173, 1, REVERSED], [174, 2, g['pcurves'][1]],
        [175, 3, g['pcurves'][2], 0.25, 0.75], [176, 4], [177, 5, g['polygon2d']],
        [178, 6, -1], [179, 7], [180, 7, 1], [181, 6, 1],
        [190, 0, REVERSED], [191, 1, 0], [192, 0, 1e-5], [193, 1, g['plane']],
        # A face left without a surface crashes the native reports (the release build
        # dereferences the null surface), so the cleared surface is set again.
        [194, 0], [193, 0, g["plane"]],
        [196, 1], [197, 0, REVERSED], [198, 1, 0],
        [199, 0, REVERSED], [200, 0, 0],
        [213, 0, REVERSED], [214, 1, FACE, 1], [215, 0, t], [203, 1, SOLID, 0],
        [223],
    ]
    requests.append(request(s, [LOG, HISTORY_ON, [PERFORM, compound, 0], *setters]))
    # Errors: INTERNAL parity, an absent occurrence, removed targets, unchanged rebinds.
    # A setter whose target id is out of range writes out of bounds natively (the release
    # build has no range check; the Rust storage panics), and a child rebound out of range
    # makes the reports read out of bounds, so no request names either.
    errors = [[173, 0, INTERNAL], [206, 0, FACE, 0], [208, FACE, 1], [192, 1, 1e-5], [198, 0, 1],
              [171, 0, 0], [180, 0, 0]]
    requests.append(request(s, [LOG, [PERFORM, compound, 0], *errors, [223]]))


def edge_wire_requests(requests: list[str]) -> None:
    """Split, vertex replacement and removal, reversal and wire rewrites with history."""
    s = solid_scene()
    compound = s.ids['compound']
    extra = [120, 0.5, 0.0, 0.0, 1e-7]
    actions = [LOG, HISTORY_ON, [PERFORM, compound, 0], extra, [156, 0, 8, 0.5], [156, 99, 8, 0.5],
               [156, 1, 99, 0.5], [158, 2, 4, 8], [158, 2, 99, 8], [121, 0, 8, -1, 0.0, 1.0, 1e-7],
               [157, 3, 6], [157, 3, 99],
               [159, 1], [188, 0], [186, 1, 4, 10, 0], [186, 1, 99, 10, 0], [187, 1, 5], [187, 1, 5],
               [189, 0, *ids(3, 2, 1, 0)], [189, 0, *ids(0)], [185, 0, 5, 6, 1], [184, 1, 0],
               [183, 1, *ids(4, 5)], [223]]
    requests.append(request(s, actions))


def removal_requests(requests: list[str]) -> None:
    """Node, subgraph and reference removal, replacement, cleanup and RemoveShape."""
    s = solid_scene()
    ids_ = s.ids
    compound = ids_['compound']
    requests.append(request(s, [LOG, HISTORY_ON, [PERFORM, compound, 0], [208, FACE, 0], [208, FACE, 0],
                                [216], [209, FACE, 1, FACE, 0], [209, EDGE, 0, EDGE, 1], [209, EDGE, 2, FACE, 1],
                                [211, FACE_REF, 0], [211, FACE_REF, 0], [212, SHELL, 0, FACE_REF, 1, 1],
                                [216], [223]]))
    requests.append(request(s, [LOG, HISTORY_ON, [PERFORM, compound, 0], [210, SOLID, 0], [210, SOLID, 0],
                                [210, 9, 0], [216], [223]]))
    requests.append(request(s, [LOG, [PERFORM, compound, 0], [144, 0, 0], [144, 0, 0], [145, 0, 1],
                                [146, 0, *ids(0, 1)], [147, 0, 0], [148, 0, *ids(0)], [149, 0, 1],
                                [150, 0, *ids(0, 1)], [129, *ids(0)], [151, 0, 0], [152, 0, *ids(0)],
                                [131], [131], [132, 0, 1, 0, -1], [153, 0, 0], [154, 0, *ids(0)],
                                [130, SOLID, 0, 0], [155, 2], [155, 2], [223]]))
    requests.append(request(s, [LOG, [ADD, compound, 0], [237, ids_['low']], [237, ids_['low']],
                                [237, compound], [237, ids_['high']], [223]]))


def batch_requests(requests: list[str]) -> None:
    """Deferred invalidation, DeferredScope, guards and ApplyModification."""
    s = solid_scene()
    compound = s.ids['compound']
    batch = [[218], [220], [169, 0, 1e-3], [160, 0, 1e-3], [192, 0, 1e-4], [218], [219], [220], [219], [220]]
    scope = [221, 4, [169, 1, 1e-3], [173, 0, INTERNAL], [221, 1, [192, 1, 1e-4]], [220]]
    guards = [[224, 0, 1e-3, 0.0, 0.0, 0.5], [225, 0, 1e-3, 0.0, 2.0], [226, 0, REVERSED, 0.0, 0.5],
              [227, 0, 1e-4], [228, 1], [229, 2], [230, 0, REVERSED], [231, 0, REVERSED], [233, 0, REVERSED],
              [224, 99, 1e-3, 0.0, 0.0, 0.0], [226, 0, INTERNAL, 0.0, 1.0], [232, 0, 0]]
    modify = [[217, EDGE, 0, *label('Trim'), *nodes((EDGE, 1)), 160, 0, 1e-2],
              [217, FACE, 0, *label('Fail'), *nodes(), 173, 0, INTERNAL],
              [217, FACE, 1, *label('Swap'), *nodes((FACE, 0), (FACE, 1)), 220]]
    requests.append(request(s, [LOG, HISTORY_ON, [PERFORM, compound, 0], *batch, _flatten(scope), *guards, *modify,
                                [222], [223]]))
    requests.append(request(s, [LOG, [PERFORM, compound, 0], [218], [169, 0, 1e-3], [208, EDGE, 0], [223]]))
    # Removal, replacement and a split inside a DeferredScope: the flush sees them all.
    inner = [[208, FACE, 1], [209, EDGE, 0, EDGE, 1], [156, 2, 5, 0.25], [216], [220], [223]]
    requests.append(request(s, [LOG, HISTORY_ON, [PERFORM, compound, 0],
                                _flatten([221, len(inner), *inner]), [223]]))


def lock_requests(requests: list[str]) -> None:
    """Lock owners on nodes and references refuse each operation group's writes."""
    s = solid_scene()
    compound = s.ids['compound']
    owners = [[62, 0, VERTEX, 0, 1], [62, 0, EDGE, 1, 1], [62, 0, COEDGE, 2, 1], [62, 0, WIRE, 0, 1],
              [62, 0, FACE, 0, 1], [62, 0, SHELL, 0, 1], [62, 1, FACE_REF, 1, 1], [62, 1, CHILD_REF, 0, 1]]
    writes = [[168, 0, 1.0, 0.0, 0.0], [224, 0, 1e-3, 0.0, 0.0, 0.0], [160, 1, 1e-3], [159, 1],
              [156, 1, 3, 0.5], [173, 2, REVERSED], [226, 2, REVERSED, 0.0, 1.0], [188, 0],
              [192, 0, 1e-4], [135, 0, 1, FORWARD], [136, 0, 1, FORWARD], [145, 0, 0], [197, 1, REVERSED],
              [230, 1, REVERSED], [213, 0, REVERSED], [231, 0, REVERSED], [208, FACE, 0], [210, SHELL, 0],
              [63, 0, VERTEX, 0], [168, 0, 1.0, 0.0, 0.0], [223]]
    requests.append(request(s, [LOG, [PERFORM, compound, 0], *owners, *writes]))


def _flatten(action) -> list[float]:
    """A nested action list (a DeferredScope with its inner actions) as numbers."""
    out: list[float] = []
    for item in action:
        out.extend(_flatten(item) if isinstance(item, list) else [item])
    return out


def mesh_requests(requests: list[str]) -> None:
    """Cache meshes, persistent setters and PromoteToPersistent with stale entries."""
    s, geometry = meshed_solid_scene()
    compound = s.ids['compound']
    tri, poly, poly2d, nodes_ = geometry['tri'], geometry['poly'], geometry['poly2d'], geometry['nodes']
    cache = [[98, 0, tri], [98, 1, geometry['tri2']], [100, 0, poly], [100, 1, geometry['poly2']],
             [103, 0, poly2d], [102, 0, nodes_], [102, 1, geometry['nodes2']], [103, 4, poly2d]]
    requests.append(request(s, [LOG, [PERFORM, compound, 0], *cache, [238], [238]]))
    requests.append(request(s, [LOG, [PERFORM, compound, 0], *cache, [169, 0, 1e-3], [238]]))
    requests.append(request(s, [LOG, [PERFORM, compound, 0], [195, 0, tri], [196, 0], [196, 0],
                                [164, 0, poly], [165, 0], [177, 0, poly2d], [178, 0, nodes_], [178, 0, -1], [223]]))
    # A lock owner refuses every write of the promotion after the first.
    requests.append(request(s, [LOG, [PERFORM, compound, 0], *cache, [62, 0, EDGE, 0, 1], [238], [169, 0, 1e-3],
                                [62, 0, VERTEX, 0, 1], [169, 0, 1e-3], [224, 0, 1e-3, 0.0, 0.0, 0.0]]))


def supplement_requests(requests: list[str]) -> None:
    """The supplement editor attaching to every owner kind, with default and explicit kinds."""
    s = solid_scene()
    ids_ = s.ids
    vertex, edge = ids_['low_vertices'][0], ids_['low_edges'][0]
    requests.append(request(s, [LOG, [PERFORM, ids_['compound'], 0],
                                [234, FACE, 0, 2, vertex], [234, FACE, 99, 2, vertex],
                                *[[235, owner, 0, vertex, -1] for owner in range(7)],
                                [235, 0, 0, edge, 7], [235, 2, 0, vertex, 0], [235, 1, 99, vertex, -1],
                                [236, 1], [236, 1], [236, 99], [223]]))


def append_tkbrep_brepgraph_edit_requests(requests: list[str]) -> None:
    """Append the BRepGraph editor requests."""
    requests.append(request(BRepScript(), [[223], [220], [216]]))
    creation_requests(requests)
    lock_requests(requests)
    setter_requests(requests)
    edge_wire_requests(requests)
    removal_requests(requests)
    batch_requests(requests)
    mesh_requests(requests)
    supplement_requests(requests)
