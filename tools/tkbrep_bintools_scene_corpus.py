"""BRep scenes of the BinTools oracle (`br_bin_shape`, `br_bin_truncate`).

A scene is the `BRepScript` of tools/tkbrep_brep_corpus.py with mesh tweaks between
its geometries and its commands (see tools/oracle/tkbrep_probe_bintools_shape.cxx),
and a root shape: a compound of every shape the script made. Each scene is written in
every format version, with and without triangles and normals.
"""
from __future__ import annotations
from tkbrep_brep_corpus import (FORWARD, INTERNAL, REVERSED, BRepScript, edge_scenario,
                                face_scenario, plane_scenario, error_scenario, text)

O, X, Y, Z = (0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (0.0, 1.0, 0.0), (0.0, 0.0, 1.0)
NORMALS, UV_NODES, DEFLECTION, POLYGON_PARAMETERS, NODE_PARAMETERS = range(1, 6)


def power(s: BRepScript, i: int, p: int) -> int:
    """The graph location kind 3: location `i` to the power `p`."""
    s.locations.append([3, i, p])
    return len(s.locations)


def compound_of(s: BRepScript) -> int:
    """A compound of every shape made so far; its index."""
    count = s.shapes
    c = s.make(0)
    for k in range(count):
        s.add(c, k)
    return c


def numbers(s: BRepScript, tweaks: list[list[float]], root: int) -> list[float]:
    out: list[float] = [len(s.locations), *[v for spec in s.locations for v in spec]]
    out += [len(s.geometries), *[v for spec in s.geometries for v in spec]]
    out += [len(tweaks), *[v for spec in tweaks for v in spec]]
    out += [len(s.commands), *[v for spec in s.commands for v in spec]]
    return out + [root]


def square_scene() -> tuple[BRepScript, list[list[float]]]:
    """A planar face bounded by four shared edges with p-curves, in a shell, a solid and a
    compound with located and reversed instances."""
    s = BRepScript()
    t = s.translation(5.0, 0.0, 0.0)
    r = s.rotation(0.0, 0.0, 1.0, 0.5)
    tr = s.product(t, r)
    squared = power(s, tr, 2)
    plane = s.plane(O, Z)
    f = s.face(plane, 0, 1e-6)
    corners = [(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (1.0, 1.0, 0.0), (0.0, 1.0, 0.0)]
    vertices = [s.vertex(c, 1e-6) for c in corners]
    wire = s.make(5)
    for k in range(4):
        a, b = corners[k], corners[(k + 1) % 4]
        d = (b[0] - a[0], b[1] - a[1], 0.0)
        e = s.edge(s.line(a, d), 0, 1e-6)
        s.add(e, vertices[k], FORWARD)
        s.add(e, vertices[(k + 1) % 4], REVERSED)
        s.do(28, e, 0.0, 1.0, 0)
        s.do(13, e, s.line2d(a[:2], d[:2]), f, 1e-6)
        s.add(wire, e)
    s.add(f, wire)
    shell = s.make(3)
    s.add(shell, f)
    solid = s.make(2)
    s.add(solid, shell)
    c = s.make(0)
    s.add(c, solid)
    s.add(c, solid, REVERSED, tr)
    s.add(c, f, FORWARD, squared)
    return s, []


def mesh_scene() -> tuple[BRepScript, list[list[float]]]:
    """Triangulations with normals and UV nodes, a triangulation-only face, polygons on
    triangulations (one and two node arrays, with parameters) and 3D polygons."""
    s = BRepScript()
    t = s.translation(0.0, 0.0, 2.0)
    plane = s.plane(O, Z)
    nodes = [(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (1.0, 1.0, 0.0), (0.0, 1.0, 0.5)]
    tri = s.triangulation(nodes, [(1, 2, 3), (1, 3, 4)], 0)
    tri2 = s.triangulation(nodes[:3], [(1, 2, 3)], 1)
    n1, n2, n3 = s.nodes([1, 2]), s.nodes([2, 3]), s.nodes([1, 3])
    p3 = s.polygon3d([(0.0, 0.0, 0.0), (0.5, 0.0, 0.0), (1.0, 0.0, 0.0)])
    tweaks = [[NORMALS, tri], [UV_NODES, tri], [DEFLECTION, tri, 0.01], [NORMALS, tri2],
              [POLYGON_PARAMETERS, p3, 0.5], [NODE_PARAMETERS, n1, 0.25]]
    s.new(7, tri)
    f = s.face(plane)
    s.do(40, f, tri2, 1)
    e = s.new(9, n1, tri, 0)
    s.do(17, e, n2, n3, tri, 0)
    s.new(8, p3)
    s.new(9, n3, tri2, t)
    e4 = s.new(9, n2, tri, 0)
    s.do(15, e4, p3, t)
    return s, tweaks


def points_scene() -> tuple[BRepScript, list[list[float]]]:
    """Point representations on a curve, on a p-curve and on a surface; one parameter is
    the tiny double that versions 1, 2 and 4 misread (its last byte is 1)."""
    s = BRepScript()
    line = s.line(O, X)
    e = s.edge(line)
    s.do(28, e, 0.0, 10.0, 0)
    plane = s.plane(O, Z)
    f = s.face(plane)
    s.do(13, e, s.line2d((0.0, 0.0), (1.0, 0.0)), f, 1e-7)
    inner = s.vertex((2.0, 0.0, 0.0))
    s.add(e, inner, INTERNAL)
    s.do(34, inner, 2.0, e, 1e-7)
    s.do(35, inner, 2.0, e, f, 1e-7)
    s.do(37, inner, 0.5, 0.25, f, 1e-7)
    tiny = s.vertex((3.0, 0.0, 0.0))
    s.add(e, tiny, INTERNAL)
    s.do(34, tiny, 2.0 ** -1000, e, 1e-7)
    return s, []


def locations_scene() -> tuple[BRepScript, list[list[float]]]:
    """One vertex under products, powers, inverses and a scaling, in both orientations,
    with located geometry."""
    s = BRepScript()
    t = s.translation(1.0, 2.0, 3.0)
    r1 = s.rotation(0.0, 0.0, 1.0, 0.3)
    r2 = s.rotation(1.0, 0.0, 0.0, 0.7)
    p = s.product(t, r1)
    pw = power(s, p, 3)
    inv = s.inverse(pw)
    p2 = s.product(pw, r2)
    sc = s.scale(2.0)
    v = s.vertex((1.0, 1.0, 1.0))
    c = s.make(0)
    for loc in (0, t, r1, p, pw, inv, p2, sc):
        s.add(c, v, FORWARD, loc)
        s.add(c, v, REVERSED, loc)
    s.edge(s.line(O, Y), p2)
    s.face(s.plane(O, X), pw)
    return s, []


def scenes() -> list[tuple[str, BRepScript, list[list[float]], int]]:
    """Every scene with its root (a compound of all its shapes)."""
    out = []
    for name, build in (('square', square_scene), ('mesh', mesh_scene), ('points', points_scene),
                        ('locations', locations_scene)):
        s, tweaks = build()
        out.append((name, s, tweaks, compound_of(s)))
    for name, build in (('edge', edge_scenario), ('face', face_scenario), ('plane', plane_scenario),
                        ('error', error_scenario)):
        s = build()
        out.append((name, s, [], compound_of(s)))
    return out


def append_tkbrep_bintools_scene_requests(requests: list[str]) -> None:
    """Every scene in every version with triangles, plus the triangle and normal
    options in versions 3 and 4; a null root; truncated shape sets."""
    configs = [(1, 0, v) for v in (1, 2, 3, 4)] + [(0, 0, 3), (0, 0, 4), (1, 1, 3), (1, 1, 4)]
    all_scenes = scenes()
    for _name, s, tweaks, root in all_scenes:
        body = text(numbers(s, tweaks, root))
        for wt, wn, version in configs:
            requests.append(f'br_bin_shape {wt} {wn} {version} {body}')
    square = all_scenes[0]
    requests.append(f'br_bin_shape 1 0 4 {text(numbers(square[1], square[2], -1))}')
    requests.append(f'br_bin_shape 1 0 9 {text(numbers(square[1], square[2], square[3]))}')
    truncations = [('square', 1, 0, 2, 1), ('square', 1, 0, 4, 1), ('mesh', 1, 1, 4, 1),
                   ('points', 1, 0, 3, 1), ('points', 1, 0, 1, 1), ('face', 1, 0, 2, 5),
                   ('edge', 1, 0, 4, 3), ('locations', 1, 0, 4, 1)]
    by_name = {name: (s, tweaks, root) for name, s, tweaks, root in all_scenes}
    for name, wt, wn, version, step in truncations:
        s, tweaks, root = by_name[name]
        requests.append(f'br_bin_truncate {wt} {wn} {version} {step} {text(numbers(s, tweaks, root))}')
