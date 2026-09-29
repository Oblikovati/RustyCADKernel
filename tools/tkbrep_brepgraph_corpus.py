"""Deterministic BRepGraph request corpus for the TKBRep oracle (`br_bgraph`).

A request is `br_bgraph script nactions (action)...`: the script is a `BRepScript` of
tools/tkbrep_brep_corpus.py, the actions ingest scene shapes into one BRepGraph
(tools/oracle/tkbrep_probe_brepgraph.cxx and crates/tkbrep/examples/br_probe/brep_graph.rs),
and the probes then print the storage through the views, the ids, UIDs and stamps, and
every original, cached and reconstructed shape. The scenes cover solids, shells,
compounds and compsolids, a cylinder seam, a degenerated edge, natural-restriction faces
(bounded, periodic and infinite), triangulation-only faces, free wires out of chain
order, shared and located sub-shapes, INTERNAL and EXTERNAL children (the orientation
errors), products and occurrences. No request routes a shape to the topology
supplement layer: the layers area of the port attaches that layer (docs/tkbrep-progress.md).
"""
from __future__ import annotations
import math
from tkbrep_brep_corpus import (BRepScript, FORWARD, REVERSED, INTERNAL, EXTERNAL, edge_scenario,
                                face_scenario, plane_scenario, text)
from tkbrep_breptools_corpus import (COMPOUND, COMPSOLID, SOLID, SHELL, WIRE, bezier_scene,
                                     cylinder_scene, mesh_scene, solid_scene, square_edge,
                                     square_face)

PERFORM, APPEND, FLATTEN, ADD, ADD_UNDER, CLEAR_CACHED = range(6)
AUTO, FLAT, PARALLEL, TRACK = 1, 2, 4, 8
SOLID_KIND, SHELL_KIND, FACE_KIND, COMPOUND_KIND, COMPSOLID_KIND, PRODUCT_KIND = 0, 1, 2, 6, 7, 10
TWO_PI = 2.0 * math.pi


def natural_scene() -> BRepScript:
    """Faces with the natural-restriction flag and no wire: a trimmed plane (four
    boundary edges), a full cylinder band (a seam and two circles), a half cylinder and
    an infinite plane (an unsupported natural boundary), in one shell."""
    s = BRepScript()
    plane = s.plane((0.0, 0.0, 0.0), (0.0, 0.0, 1.0))
    cylinder = s.cylinder((0.0, 0.0, 0.0), (0.0, 0.0, 1.0), 2.0)
    shell = s.make(SHELL)
    for surface in (s.rtrim(plane, -1.0, 2.0, 0.0, 1.5), s.rtrim(cylinder, 0.0, TWO_PI, 0.0, 3.0),
                    s.rtrim(cylinder, 0.0, math.pi, -1.0, 1.0), plane):
        face = s.face(surface, 0, 1e-6)
        s.do(42, face, 1)
        s.add(shell, face, FORWARD)
    s.ids = dict(root=shell)
    return s


def degenerate_scene() -> BRepScript:
    """A planar face whose wire holds three line edges and a degenerated edge (no 3D
    curve, one vertex at both ends, a p-curve), and a compsolid of two solids sharing it."""
    s = BRepScript()
    face = s.face(s.plane((0.0, 0.0, 0.0), (0.0, 0.0, 1.0)), 0, 1e-7)
    corners = [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)]
    vertices = [s.vertex((x, y, 0.0)) for x, y in corners]
    wire = s.make(WIRE)
    for i in range(3):
        j = (i + 1) % 3
        edge = square_edge(s, face, vertices[i], vertices[j], (*corners[i], 0.0), (*corners[j], 0.0),
                           corners[i], corners[j])
        s.add(wire, edge, FORWARD)
    degenerate = s.new(3)
    s.do(27, degenerate, 1)
    s.add(degenerate, vertices[0], FORWARD)
    s.add(degenerate, vertices[0], REVERSED)
    s.do(28, degenerate, 0.0, 1.0, 0)
    s.do(13, degenerate, s.line2d((0.0, 0.0), (0.0, 1.0)), face, 1e-7)
    s.add(wire, degenerate, FORWARD)
    s.add(face, wire, FORWARD)
    shell = s.make(SHELL)
    s.add(shell, face, FORWARD)
    solids = []
    for orientation in (FORWARD, REVERSED):
        solid = s.make(SOLID)
        s.add(solid, shell, orientation)
        solids.append(solid)
    compsolid = s.make(COMPSOLID)
    for solid in solids:
        s.add(compsolid, solid, FORWARD)
    s.ids = dict(face=face, degenerate=degenerate, shell=shell, compsolid=compsolid, wire=wire,
                 solid=solids[1])
    return s


def free_scene() -> BRepScript:
    """A free wire listing its edges out of chain order, a reversed free edge, a free
    vertex, a wire with an EXTERNAL edge and a face with an INTERNAL wire, in a compound
    with located and repeated children."""
    s = BRepScript()
    t = s.translation(0.0, 0.0, 5.0)
    r = s.rotation(0.0, 1.0, 0.0, 0.3)
    face, edges, wire, vertices = square_face(s, 2.0, 0.0)
    shuffled = s.make(WIRE)
    for edge, orientation in ((edges[2], FORWARD), (edges[0], FORWARD), (edges[3], FORWARD),
                              (edges[1], FORWARD)):
        s.add(shuffled, edge, orientation)
    external = s.make(WIRE)
    s.add(external, edges[0], EXTERNAL)
    inner_face, _, inner_wire, _ = square_face(s, 0.5, 3.0)
    s.add(inner_face, inner_wire, INTERNAL)
    compound = s.make(COMPOUND)
    s.add(compound, shuffled, FORWARD)
    s.add(compound, edges[1], REVERSED, t)
    s.add(compound, vertices[2], FORWARD, r)
    s.add(compound, face, FORWARD, t)
    s.add(compound, face, REVERSED, r)
    s.add(compound, vertices[2], FORWARD, r)
    s.ids = dict(face=face, shuffled=shuffled, external=external, inner_face=inner_face,
                 compound=compound, edge=edges[1], vertex=vertices[2])
    return s


def request(script: BRepScript, actions: list[list[float]]) -> str:
    numbers = [*script.numbers(), len(actions), *[v for action in actions for v in action]]
    return f'br_bgraph {text(numbers)}'


def solid_requests(requests: list[str]) -> None:
    """The solid scene through every ingestion path and a parent of every kind."""
    s = solid_scene()
    ids = s.ids
    compound, solid, low, moved = ids['compound'], ids['solid'], ids['low'], ids['moved']
    shell = ids['shell']
    for actions in ([[PERFORM, compound, 0]], [[PERFORM, compound, 1]],
                    [[ADD, compound, AUTO | TRACK]], [[ADD, compound, AUTO | FLAT | TRACK]],
                    [[APPEND, solid, 0], [APPEND, compound, 0], [APPEND, compound, 0]],
                    [[FLATTEN, compound, 0], [FLATTEN, solid, 1]],
                    [[ADD, compound, AUTO], [ADD_UNDER, solid, PRODUCT_KIND, 0, TRACK],
                     [ADD_UNDER, moved, COMPOUND_KIND, 0, 0], [CLEAR_CACHED, SOLID_KIND, 0]],
                    [[ADD, solid, 0], [ADD_UNDER, moved, SHELL_KIND, 0, TRACK],
                     [ADD_UNDER, shell, SOLID_KIND, 0, 0], [ADD_UNDER, low, FACE_KIND, 0, 0]],
                    [[PERFORM, compound, 0], [PERFORM, low, 0]]):
        requests.append(request(s, actions))


def scene_requests(requests: list[str]) -> None:
    """The other scenes: the root through Perform and Add, then every shape appended."""
    for s in (cylinder_scene(), mesh_scene(), bezier_scene(), natural_scene()):
        root = getattr(s, 'ids', {}).get('root', s.shapes - 1)
        requests.append(request(s, [[PERFORM, root, 0]]))
        requests.append(request(s, [[ADD, root, AUTO | TRACK]]))
        requests.append(request(s, [[APPEND, i, 0] for i in range(s.shapes)]))
    d = degenerate_scene()
    requests.append(request(d, [[PERFORM, d.ids['compsolid'], 0]]))
    requests.append(request(d, [[PERFORM, d.ids['compsolid'], 0], [ADD_UNDER, d.ids['solid'], COMPSOLID_KIND, 0, 0],
                                [ADD, d.ids['wire'], AUTO | TRACK]]))
    f = free_scene()
    requests.append(request(f, [[PERFORM, f.ids['compound'], 0]]))
    requests.append(request(f, [[ADD, f.ids['compound'], AUTO | TRACK], [ADD, f.ids['external'], 0],
                                [ADD, f.ids['inner_face'], AUTO]]))
    requests.append(request(f, [[FLATTEN, f.ids['compound'], 0], [PERFORM, f.ids['shuffled'], 1]]))


def edge_case_requests(requests: list[str]) -> None:
    """The BRep corpus scenes (flags, polygons, seams, projections, empty shapes): every
    shape appended, one at a time."""
    for s in (edge_scenario(), face_scenario(), plane_scenario()):
        requests.append(request(s, [[APPEND, i, 0] for i in range(s.shapes)]))
        requests.append(request(s, [[ADD, i, AUTO] for i in range(s.shapes)]))


def append_tkbrep_brepgraph_requests(requests: list[str]) -> None:
    """Append the BRepGraph requests."""
    solid_requests(requests)
    scene_requests(requests)
    edge_case_requests(requests)
