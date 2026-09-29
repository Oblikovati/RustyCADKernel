"""Deterministic BRepGraph navigation corpus for the TKBRep oracle (`br_bgraph_walk`).

A request is `br_bgraph_walk script nactions (action)... nedits (edit)...`: the script
and actions are those of `br_bgraph` (tools/tkbrep_brepgraph_corpus.py), the edits
build assemblies through `Editor().Products()` (0 Add(root kind, index, location),
1 Add(), 2 AppendDocumentRoot(product), 3 Append(parent, child, location, parent
occurrence)). The probes (tools/oracle/tkbrep_probe_brepgraph_walk.cxx, _explore.cxx
and _tool.cxx) then print the dense, defs, refs, reverse and related iterators, the
child and parent explorers under 19 and 14 configurations from every node, and every
BRepGraph_Tool member. The scenes cover solids, shells, compounds and compsolids with
located and repeated children, a cylinder seam, a degenerated edge, natural
restrictions, triangulation-only faces, free wires, and part, nested and repeated
product occurrences. No request removes a node: removal is the editor's `RemoveNode`,
a later wave of the port.
"""
from __future__ import annotations
from tkbrep_brep_corpus import edge_scenario, face_scenario, text
from tkbrep_brepgraph_corpus import (ADD, APPEND, AUTO, PERFORM, degenerate_scene, free_scene,
                                     natural_scene)
from tkbrep_breptools_corpus import bezier_scene, cylinder_scene, mesh_scene, solid_scene

ADD_PRODUCT, ADD_EMPTY, DOCUMENT_ROOT, APPEND_PRODUCT = range(4)
SOLID_KIND, COMPOUND_KIND = 0, 6


def request(script, actions: list[list[float]], edits: list[list[float]]) -> str:
    numbers = [*script.numbers(), len(actions), *[v for a in actions for v in a],
               len(edits), *[v for e in edits for v in e]]
    return f'br_bgraph_walk {text(numbers)}'


def assembly_requests(requests: list[str]) -> None:
    """The solid scene under products: a part placed twice in an assembly, a nested
    sub-assembly, a compound root, and the invalid edits (self append, bad roots)."""
    s = solid_scene()
    compound = s.ids['compound']
    part_twice = [[ADD_PRODUCT, SOLID_KIND, 0, 0], [DOCUMENT_ROOT, 0], [ADD_EMPTY], [DOCUMENT_ROOT, 1],
                  [APPEND_PRODUCT, 1, 0, 1, -1], [APPEND_PRODUCT, 1, 0, 2, -1]]
    requests.append(request(s, [[PERFORM, compound, 0]], part_twice))
    nested = [[ADD_PRODUCT, SOLID_KIND, 0, 1], [ADD_EMPTY], [ADD_EMPTY], [DOCUMENT_ROOT, 2],
              [APPEND_PRODUCT, 1, 0, 0, -1], [APPEND_PRODUCT, 1, 0, 2, -1], [APPEND_PRODUCT, 2, 1, 1, -1],
              [APPEND_PRODUCT, 2, 0, 3, -1], [APPEND_PRODUCT, 2, 2, 0, -1], [ADD_PRODUCT, 3, 9, 0]]
    requests.append(request(s, [[PERFORM, compound, 0]], nested))
    compound_part = [[ADD_PRODUCT, COMPOUND_KIND, 0, 2], [ADD_EMPTY], [APPEND_PRODUCT, 1, 0, 1, -1],
                     [APPEND_PRODUCT, 1, 0, 0, 0], [APPEND_PRODUCT, 1, 0, 0, -1]]
    requests.append(request(s, [[PERFORM, compound, 0]], compound_part))
    requests.append(request(s, [[ADD, compound, AUTO]], [[ADD_EMPTY], [APPEND_PRODUCT, 1, 0, 1, -1]]))


def scene_requests(requests: list[str]) -> None:
    """Every scene root through Perform, and the scenes of every shape appended."""
    for s in (cylinder_scene(), mesh_scene(), bezier_scene(), natural_scene()):
        root = getattr(s, 'ids', {}).get('root', s.shapes - 1)
        requests.append(request(s, [[PERFORM, root, 0]], []))
    d = degenerate_scene()
    requests.append(request(d, [[PERFORM, d.ids['compsolid'], 0]], []))
    f = free_scene()
    requests.append(request(f, [[PERFORM, f.ids['compound'], 0]], []))
    requests.append(request(f, [[ADD, f.ids['shuffled'], AUTO], [ADD, f.ids['face'], 0]], []))
    for s in (edge_scenario(), face_scenario()):
        requests.append(request(s, [[APPEND, i, 0] for i in range(s.shapes)], []))


def append_tkbrep_brepgraph_walk_requests(requests: list[str]) -> None:
    """Append the BRepGraph navigation requests."""
    assembly_requests(requests)
    scene_requests(requests)
