"""Deterministic BRepTools, BRepAdaptor and BRepLProp request corpus for the TKBRep oracle.

A request is `op script params...`: the script is a `BRepScript` of
tools/tkbrep_brep_corpus.py (read by `br_scene` in tools/oracle/tkbrep_probe_brep.cxx
and crates/tkbrep/examples/br_probe/brep.rs), the parameters are those of the
operation. The scenes below are closed topology: planar faces bounded by line edges
with p-curves, a cylinder with its seam, shells, solids and compounds, located and
reversed copies, meshes and INTERNAL sub-shapes, so every tool has real input; the BRep
scenes of the BRep corpus add the degenerate and error states.
"""
from __future__ import annotations
import math
from tkbrep_brep_corpus import (BRepScript, FORWARD, REVERSED, INTERNAL, edge_scenario,
                                face_scenario, plane_scenario, text)

COMPOUND, COMPSOLID, SOLID, SHELL, WIRE = 0, 1, 2, 3, 5
TWO_PI = 2.0 * math.pi


def square_edge(s: BRepScript, face: int, v1: int, v2: int, a, b, uv_a, uv_b) -> int:
    """A line edge from vertex v1 at a to v2 at b, with its p-curve on `face`."""
    d = [b[i] - a[i] for i in range(3)]
    length = sum(c * c for c in d) ** 0.5
    e = s.edge(s.line(a, d))
    s.add(e, v1, FORWARD)
    s.add(e, v2, REVERSED)
    s.do(28, e, 0.0, length, 0)
    duv = [(uv_b[i] - uv_a[i]) / length for i in range(2)]
    s.do(13, e, s.line2d(uv_a, duv), face, 1e-7)
    s.do(34, v1, 0.0, e, 1e-7)
    s.do(34, v2, length, e, 1e-7)
    return e


def square_face(s: BRepScript, size: float, z: float,
                internal: bool = False) -> tuple[int, list[int], int]:
    """A planar face on z with one wire of four line edges and shared vertices; with
    `internal`, the third edge holds an INTERNAL vertex at its middle."""
    plane = s.plane((0.0, 0.0, z), (0.0, 0.0, 1.0))
    face = s.face(plane, 0, 1e-7)
    corners = [(0.0, 0.0), (size, 0.0), (size, size), (0.0, size)]
    vertices = [s.vertex((x, y, z)) for x, y in corners]
    wire = s.make(WIRE)
    edges = []
    for i in range(4):
        j = (i + 1) % 4
        a, b = (*corners[i], z), (*corners[j], z)
        edges.append(square_edge(s, face, vertices[i], vertices[j], a, b, corners[i], corners[j]))
        if internal and i == 2:
            middle = s.vertex(tuple((a[k] + b[k]) / 2 for k in range(3)))
            s.add(edges[-1], middle, INTERNAL)
            s.do(34, middle, size / 2, edges[-1], 1e-7)
        s.add(wire, edges[-1], FORWARD)
    s.add(face, wire, FORWARD)
    return face, edges, wire


def cylinder_face(s: BRepScript) -> tuple[int, int, int]:
    """A cylinder face bounded by two circles and its seam; returns face, seam, wire."""
    cylinder = s.cylinder((0.0, 0.0, 0.0), (0.0, 0.0, 1.0), 1.0)
    face = s.face(cylinder, 0, 1e-7)
    low, high = s.vertex((1.0, 0.0, 0.0)), s.vertex((1.0, 0.0, 1.0))
    circles = []
    for z, v in ((0.0, low), (1.0, high)):
        c = s.edge(s.circle((0.0, 0.0, z), (0.0, 0.0, 1.0), 1.0))
        s.add(c, v, FORWARD)
        s.add(c, v, REVERSED)
        s.do(28, c, 0.0, TWO_PI, 0)
        s.do(13, c, s.line2d((0.0, z), (1.0, 0.0)), face, 1e-7)
        circles.append(c)
    seam = s.edge(s.line((1.0, 0.0, 0.0), (0.0, 0.0, 1.0)))
    s.add(seam, low, FORWARD)
    s.add(seam, high, REVERSED)
    s.do(28, seam, 0.0, 1.0, 0)
    s.do(14, seam, s.line2d((TWO_PI, 0.0), (0.0, 1.0)), s.line2d((0.0, 0.0), (0.0, 1.0)), face, 1e-7)
    wire = s.make(WIRE)
    for edge, orientation in ((circles[0], FORWARD), (seam, FORWARD), (circles[1], REVERSED),
                              (seam, REVERSED)):
        s.add(wire, edge, orientation)
    s.add(face, wire, FORWARD)
    return face, seam, wire


def solid_scene() -> BRepScript:
    """Two squares in a shell, a solid, a compound, located copies, a mesh and internals."""
    s = BRepScript()
    t = s.translation(0.0, 0.0, 2.0)
    r = s.rotation(0.0, 0.0, 1.0, 0.5)
    low, low_edges, _ = square_face(s, 1.0, 0.0, internal=True)
    high, _, _ = square_face(s, 2.0, 1.0)
    # INTERNAL sub-shapes, added before the faces are frozen by their parents.
    s.add(low, s.vertex((0.5, 0.5, 0.0)), INTERNAL)
    shell = s.make(SHELL)
    s.add(shell, low, REVERSED)
    s.add(shell, high, FORWARD)
    solid = s.make(SOLID)
    s.add(solid, shell, FORWARD)
    compound = s.make(COMPOUND)
    s.add(compound, solid, FORWARD)
    s.add(compound, low, FORWARD, t)
    s.moved(low, REVERSED, r)
    tri = s.triangulation([(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (1.0, 1.0, 0.0), (0.0, 1.0, 0.0)],
                          [(1, 2, 3), (1, 3, 4)], 0)
    s.do(40, low, tri, 1)
    s.do(15, low_edges[0], s.polygon3d([(0.0, 0.0, 0.0), (0.5, 0.0, 0.0), (1.0, 0.0, 0.0)]), 0)
    s.do(16, low_edges[1], s.nodes([2, 3]), tri, 0)
    return s


def cylinder_scene() -> BRepScript:
    """A closed cylinder face with its seam, a located copy and a shell around it."""
    s = BRepScript()
    t = s.translation(1.0, 2.0, 3.0)
    face, seam, _ = cylinder_face(s)
    s.moved(face, REVERSED, t)
    shell = s.make(SHELL)
    s.add(shell, face, FORWARD)
    s.reversed(seam)
    return s


def mesh_scene() -> BRepScript:
    """Triangulation-only faces with several meshes, polygons and a face with normals."""
    s = BRepScript()
    tri = s.triangulation([(0.0, 0.0, 0.0), (2.0, 0.0, 0.0), (0.0, 2.0, 1.0)], [(1, 2, 3)], 1)
    tri2 = s.triangulation([(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (0.0, 1.0, 0.5)], [(1, 2, 3)], 2)
    s.new(7, tri)
    s.new(47, 2, tri, tri2, tri2)
    poly = s.polygon3d([(0.0, 0.0, 0.0), (1.0, 0.0, 0.0)])
    s.new(8, poly)
    s.new(9, s.nodes([1, 2]), tri, 0)
    return s


def scenes() -> list[BRepScript]:
    """Every scene of the BRepTools requests."""
    return [solid_scene(), cylinder_scene(), mesh_scene(), edge_scenario(), face_scenario(),
            plane_scenario()]


def append_tkbrep_breptools_requests(requests: list[str]) -> None:
    """Append the BRepTools requests: every scene through every text, query and tool."""
    for script in scenes():
        numbers = text(script.numbers())
        requests.append(f'br_tools_text {numbers}')
        requests.append(f'br_tools_query {numbers}')
        for code in range(10):
            requests.append(f'br_tools_mutate {numbers} {code}')
