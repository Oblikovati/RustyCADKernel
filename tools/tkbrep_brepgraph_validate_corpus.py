"""Deterministic BRepGraph algorithm requests for the TKBRep oracle (`br_bedit`).

The requests reuse the editor request format of tools/tkbrep_brepgraph_edit_corpus.py
with the action codes of tools/oracle/tkbrep_probe_brepgraph_validate.cxx and
crates/tkbrep/examples/br_probe/brep_graph_validate_transform.rs: 260-265 write raw definition and
reference fields through a mutation guard (so the graph can be broken on purpose), 266
runs BRepGraph_Validate through each `Perform` overload and prints every issue in
report order, and 267/268 are the two BRepGraph_Transform::MoveRef overloads. A field
written out of range is restored before the request ends, since the final reports read
the graph without range checks natively.
"""
from __future__ import annotations
from tkbrep_brep_corpus import BRepScript, FORWARD
from tkbrep_brepgraph_corpus import ADD, PERFORM
from tkbrep_brepgraph_edit_corpus import (
    CHILD_REF, COMPOUND, EDGE, FACE, LOG, OCCURRENCE, PRODUCT, SOLID, VERTEX, ids, nodes, request,
    triangle_actions, triangle_geometry)
from tkbrep_brepgraph_algo_corpus import AUTO_PRODUCT, request as balgo_request
from tkbrep_brepgraph_cache_corpus import meshed_solid_scene
from tkbrep_breptools_corpus import solid_scene

AUDIT, LIGHT = [266, 4], [266, 0]
ALL_MODES = [[266, how] for how in range(5)]
FAR = 999
# `ba_trsf` specs: translation, rotation, scale, point, axis and plane mirrors, values.
TRANSLATION = [0, 1.0, 2.0, 3.0]
ROTATION = [1, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.5]
SCALE = [2, 0.0, 0.0, 0.0, 2.0]
POINT_MIRROR = [3, 1.0, 1.0, 1.0]
AXIS_MIRROR = [4, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0]
PLANE_MIRROR = [5, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0]
GENERAL = [6, 0.0, -1.0, 0.0, 1.0, 1.0, 0.0, 0.0, 2.0, 0.0, 0.0, 1.0, 3.0]
SINGULAR = [6, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 3.0]


def triangle_graph() -> tuple[BRepScript, list[list[float]]]:
    """A triangle face built from scratch (vertices 0-2, edges 0-2, coedges 0-2 with
    p-curves, wire 0, face 0) inside a shell and a solid."""
    s = BRepScript()
    g = triangle_geometry(s)
    actions = [LOG, *triangle_actions(g), [126], [136, 0, 0, FORWARD], [127], [138, 0, 0, FORWARD]]
    return s, actions


def definition_corruptions() -> list[list[float]]:
    """Edge, coedge and face fields broken then restored, each checked by an audit."""
    broken_restored = [
        ([260, 0, 2, -1], [260, 0, 2, 0]),      # edge without a 3D curve record
        ([260, 1, 2, 2], [260, 1, 2, 1]),       # two edges on one curve record
        ([260, 2, 3, FAR], [260, 2, 3, -1]),    # 3D polygon out of range
        ([261, 0, 1, FAR], [261, 0, 1, 0]),     # coedge face out of range
        ([261, 1, 0, FAR], [261, 1, 0, 1]),     # coedge edge out of range
        ([261, 2, 2, FAR], [261, 2, 2, 2]),     # p-curve out of range
        ([261, 0, 2, -1], [261, 0, 2, 0]),      # coedge without its p-curve
        ([261, 0, 5, -1], [261, 0, 5, 0]),      # coedge without a live wire
        ([262, 0, 0, FAR], [262, 0, 0, 0]),     # surface out of range
        ([262, 0, 1, FAR], [262, 0, 1, -1]),    # triangulation out of range
        ([261, 1, 1, -1], [261, 1, 1, 0]),      # coedge off the face
    ]
    actions: list[list[float]] = []
    for broken, restored in broken_restored:
        actions += [broken, AUDIT, restored]
    return actions + [AUDIT]


def triangle_requests(requests: list[str]) -> None:
    """Every Perform overload on a valid graph, then broken definitions, vertex
    references and seam pairs, and the lightweight versus audit difference."""
    s, base = triangle_graph()
    requests.append(request(s, [*base, *ALL_MODES]))
    requests.append(request(s, [*base, *definition_corruptions()]))
    # A vertex reference moved onto another vertex breaks the wire chain (a warning)
    # and the edge-vertex relations; one moved onto a removed vertex breaks isolation.
    vertex_refs = [[264, 0, 2], AUDIT, LIGHT, [264, 0, 0],
                   [120, 5.0, 5.0, 5.0, 1e-7], [208, VERTEX, 3], [264, 1, 3], LIGHT, AUDIT, [264, 1, 1], AUDIT]
    requests.append(request(s, [*base, *vertex_refs]))
    # A coedge moved onto the edge of another coedge of the same face: a seam pair of
    # equal orientations, then three coedges on one (edge, face) pair.
    seams = [[261, 1, 0, 0], AUDIT, [261, 2, 0, 0], AUDIT, [261, 1, 0, 1], [261, 2, 0, 2], AUDIT]
    requests.append(request(s, [*base, *seams]))
    # A surfaced face that lost its only wire: the warning of an unbounded face.
    requests.append(request(s, [*base, [144, 0, 0], AUDIT, LIGHT]))


def assembly_requests(requests: list[str]) -> None:
    """Products and occurrences: document roots, broken occurrence children and an
    assembly cycle."""
    s, base = triangle_graph()
    products = [[130, SOLID, 0, 0], [131], [132, 1, 0, 0, -1], [134, 1], AUDIT, [134, 0], AUDIT, LIGHT]
    children = [[263, 0, OCCURRENCE, 0], AUDIT, [263, 0, 9, 0], AUDIT, [263, 0, PRODUCT, FAR], AUDIT,
                [263, 0, SOLID, 0], AUDIT]
    cycle = [[131], [132, 2, 1, 0, -1], [263, 2, PRODUCT, 2], AUDIT, [263, 1, PRODUCT, 2], AUDIT,
             [263, 1, PRODUCT, 0], [263, 2, PRODUCT, 1], AUDIT]
    requests.append(request(s, [*base, *products, *children, *cycle]))
    # A removed product with a surviving occurrence and a removed occurrence.
    requests.append(request(s, [*base, [130, SOLID, 0, 0], [131], [132, 1, 0, 0, -1], [208, PRODUCT, 0], AUDIT,
                                [208, OCCURRENCE, 1], AUDIT, LIGHT]))


def container_requests(requests: list[str]) -> None:
    """Compound cycles, shared child references, cross-container solids and the
    ownership of references."""
    s, base = triangle_graph()
    compounds = [[128, *nodes((FACE, 0))], [128, *nodes((COMPOUND, 0))], AUDIT,
                 [265, 0, COMPOUND, 1], AUDIT, [265, 0, COMPOUND, 0], AUDIT, [265, 0, FACE, 0],
                 [128, *nodes((SOLID, 0))], [129, *ids(0)], AUDIT, [216], AUDIT]
    requests.append(request(s, [*base, *compounds]))


def solid_requests(requests: list[str]) -> None:
    """The ingested solid scene: every overload, then removals and cleanup."""
    s = solid_scene()
    compound = s.ids['compound']
    ingest = [LOG, [PERFORM, compound, 0]]
    requests.append(request(s, [*ingest, *ALL_MODES]))
    requests.append(request(s, [*ingest, [208, FACE, 0], LIGHT, AUDIT, [216], AUDIT]))
    requests.append(request(s, [*ingest, [210, SOLID, 0], LIGHT, AUDIT, [208, EDGE, 0], AUDIT]))


def move_ref_requests(requests: list[str]) -> None:
    """Both MoveRef overloads with every transform kind, refused scales, removed and
    out-of-range references, a singular transform and a lock owner."""
    s = solid_scene()
    compound = s.ids['compound']
    ingest = [LOG, [PERFORM, compound, 0]]
    child = [[267, 0, *spec] for spec in (TRANSLATION, ROTATION, SCALE, POINT_MIRROR, AXIS_MIRROR,
                                          PLANE_MIRROR, GENERAL)]
    occurrence = [[268, 0, *TRANSLATION], [268, 0, *ROTATION], [268, 0, *SCALE], [268, 0, *PLANE_MIRROR],
                  [268, FAR, *TRANSLATION], [268, -1, *TRANSLATION]]
    refused = [[267, FAR, *TRANSLATION], [267, 1, *SINGULAR], [211, CHILD_REF, 1], [267, 1, *TRANSLATION]]
    requests.append(request(s, [*ingest, *child, *occurrence, *refused, AUDIT]))
    locked = [[62, 1, CHILD_REF, 0, 1], [267, 0, *TRANSLATION], [63, 1, CHILD_REF, 0], [267, 0, *TRANSLATION]]
    requests.append(request(s, [*ingest, *locked, AUDIT]))


COPY, SHARE, DROP = range(3)
PERFORM_OTHER, PERFORM_SELF, NODE_OTHER, NODE_SELF = range(270, 274)
TRANSFORMS = (TRANSLATION, ROTATION, SCALE, POINT_MIRROR, PLANE_MIRROR, GENERAL)


def meshed_source() -> tuple[BRepScript, list[list[float]]]:
    """The meshed solid scene ingested with a cache mesh on the first face."""
    s, geometry = meshed_solid_scene()
    return s, [LOG, [PERFORM, s.ids['compound'], 0], [98, 0, geometry['tri']]]


def transform_requests(requests: list[str]) -> None:
    """Transform::Perform and TransformNode (`br_balgo`) for every transform kind: in
    place under each geometry and mesh policy pair (the refused pairs included), and
    into the other graph (empty, then not), for the whole graph, a face and a refused
    product, and in place for an edge and a face; each request ends with both graphs'
    reports."""
    for spec in TRANSFORMS:
        s, source = meshed_source()
        pairs = [(COPY, DROP), (COPY, COPY), (SHARE, COPY), (SHARE, DROP), (SHARE, SHARE), (DROP, DROP)]
        requests.append(balgo_request(s, [*source, *[[PERFORM_SELF, *spec, g, m] for g, m in pairs]]))
        other = [[PERFORM_OTHER, *spec, COPY, COPY], [PERFORM_OTHER, *spec, SHARE, COPY],
                 [PERFORM_OTHER, *spec, SHARE, SHARE], [NODE_OTHER, FACE, 0, *spec, COPY, COPY],
                 [NODE_OTHER, PRODUCT, 0, *spec, COPY, DROP], [NODE_SELF, EDGE, 0, *spec, COPY, COPY],
                 [NODE_SELF, FACE, 1, *spec, SHARE, COPY]]
        # These scenes are ingested without an assembly (no product, no occurrence), so
        # a location-only TransformNode of a product or occurrence would name an id past
        # the store: the native CopyNode reads it unchecked and crashes. The requests of
        # placement_transform_requests add the shape with an auto product; the refused
        # geometry-level case above returns before the copy.
        requests.append(balgo_request(s, [*source, *other]))
    empty = [[PERFORM_OTHER, *TRANSLATION, COPY, DROP], [PERFORM_SELF, *TRANSLATION, COPY, DROP],
             [NODE_OTHER, FACE, 0, *TRANSLATION, COPY, DROP], [NODE_SELF, FACE, 0, *TRANSLATION, COPY, DROP]]
    requests.append(balgo_request(BRepScript(), empty))


def placement_transform_requests(requests: list[str]) -> None:
    """Location-only TransformNode (`br_balgo`) of the product and the occurrence that
    Shapes().Add creates with an auto product, into the other graph, for every transform
    kind: the rigid ones copy the node with CopyNode and compose the placement, a scale
    or a mirror is refused before the copy; both graphs are then printed."""
    for spec in TRANSFORMS:
        s, geometry = meshed_solid_scene()
        source = [LOG, [ADD, s.ids['compound'], AUTO_PRODUCT], [98, 0, geometry['tri']]]
        placements = [[NODE_OTHER, PRODUCT, 0, *spec, SHARE, DROP],
                      [NODE_OTHER, OCCURRENCE, 0, *spec, SHARE, DROP]]
        requests.append(balgo_request(s, [*source, *placements]))


def append_tkbrep_brepgraph_validate_requests(requests: list[str]) -> None:
    """Append the BRepGraph validation and transform requests."""
    requests.append(request(BRepScript(), [*ALL_MODES, [267, 0, *TRANSLATION], [268, 0, *TRANSLATION]]))
    triangle_requests(requests)
    assembly_requests(requests)
    container_requests(requests)
    solid_requests(requests)
    move_ref_requests(requests)
    transform_requests(requests)
    placement_transform_requests(requests)
