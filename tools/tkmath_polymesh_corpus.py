"""Deterministic Poly differential requests for tools/verify_tkmath_numeric.py.

Usage: append_polymesh_requests(requests) after the Convert requests.
Meshes are open grids on curved sheets, closed polyhedra, fans, strips with duplicated
nodes, zero-area and repeated-node triangles, isolated nodes, non-manifold edges and
reversed triangles, in double and single precision and at 1e-8 .. 1e8 scales.
Operations: Poly_Triangulation queries (normals, cached ranges, ray intersection,
Catenate, DumpJson), PointOnTriangle, IntersectTriLine, PolygonProperties, the text
Write/Dump/Read round trips, Poly_Connect, Poly_CoherentTriangulation with editing,
Poly_MergeNodesTool and Poly_MakeLoops2D/3D.
Requests stay inside the native defined domain: every mesh has a node (a zero-length
Poly_ArrayOfNodes throws inside the reader), coherent and merge meshes index valid nodes
(Poly_CoherentTriangulation does not range-check), and no SetNode replaces a used node.
"""
from __future__ import annotations
import math
import random

Mesh = tuple[list[tuple[float, float, float]], list[tuple[int, int, int]]]


def line(op: str, values: list) -> str:
    """Format one request with round-trip float text; line('poly_pot', [0, 1])."""
    return op+' '+' '.join(str(value) for value in values)


def grid(rng: random.Random, nx: int, ny: int, scale: float) -> Mesh:
    """A curved sheet with coherently oriented triangles; grid(rng, 3, 2, 1.0)."""
    a, b = rng.uniform(-1, 1), rng.uniform(-1, 1)
    nodes = [(i*scale, j*scale, (a*math.sin(i)+b*math.cos(j))*scale) for j in range(ny) for i in range(nx)]
    triangles = []
    for j in range(ny-1):
        for i in range(nx-1):
            p = j*nx+i+1
            triangles += [(p, p+1, p+nx+1), (p, p+nx+1, p+nx)]
    return nodes, triangles


def polyhedron(rng: random.Random, kind: int) -> Mesh:
    """Closed tetrahedron, octahedron or cube, randomly offset; polyhedron(rng, 1)."""
    o = [rng.uniform(-5, 5) for _ in range(3)]
    if kind == 0:
        nodes = [(0, 0, 0), (1, 0, 0), (0, 1, 0), (0, 0, 1)]
        tris = [(1, 3, 2), (1, 2, 4), (2, 3, 4), (1, 4, 3)]
    elif kind == 1:
        nodes = [(1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1)]
        tris = [(1, 3, 5), (3, 2, 5), (2, 4, 5), (4, 1, 5), (3, 1, 6), (2, 3, 6), (4, 2, 6), (1, 4, 6)]
    else:
        nodes = [(x, y, z) for z in (0, 1) for y in (0, 1) for x in (0, 1)]
        quads = [(1, 3, 4, 2), (5, 6, 8, 7), (1, 2, 6, 5), (3, 7, 8, 4), (1, 5, 7, 3), (2, 4, 8, 6)]
        tris = [t for a, b, c, d in quads for t in ((a, b, c), (a, c, d))]
    return [tuple(float(p[k]+o[k]) for k in range(3)) for p in nodes], tris


def fan(rng: random.Random, count: int) -> Mesh:
    """A disk fan around node 1, open or closed; fan(rng, 5)."""
    nodes = [(0.0, 0.0, rng.uniform(-1, 1))]
    nodes += [(math.cos(2*math.pi*k/count), math.sin(2*math.pi*k/count), 0.0) for k in range(count)]
    closed = rng.random() < 0.5
    tris = [(1, k+2, (k+1) % count+2) for k in range(count if closed else count-1)]
    return nodes, tris


def damaged(rng: random.Random, mesh: Mesh, kind: int) -> Mesh:
    """Duplicated nodes, degenerate, repeated-node, reversed or non-manifold triangles."""
    nodes, tris = list(mesh[0]), list(mesh[1])
    n = len(nodes)
    if kind == 0:
        for k in range(min(3, n)):
            nodes.append(nodes[k])
        tris.append((n+1, n+2, n+3 if n >= 3 else n+1))
    elif kind == 1:
        tris.append((1, 1, min(2, n)))
    elif kind == 2 and tris:
        a, b, c = tris[0]
        tris[0] = (a, c, b)
    elif kind == 3 and tris:
        a, b, _ = tris[0]
        nodes.append((rng.uniform(-1, 1), rng.uniform(-1, 1), 2.0))
        tris.append((a, b, len(nodes)))
    elif kind == 4:
        nodes.append((9.0, 9.0, 9.0))
    elif kind == 5 and n >= 2:
        nodes.append(tuple(v+1e-9 for v in nodes[0]))
        tris.append((len(nodes), 2, 1))
    return nodes, tris


def random_mesh(rng: random.Random, case: int) -> Mesh:
    """One mesh of the corpus families, then optionally damaged; random_mesh(rng, 7)."""
    family = case % 4
    if family == 0:
        scale = rng.choice([1.0, 1.0, 1e-8, 1e8, 0.37])
        mesh = grid(rng, rng.randint(2, 5), rng.randint(2, 4), scale)
    elif family == 1:
        mesh = polyhedron(rng, case % 3)
    elif family == 2:
        mesh = fan(rng, rng.randint(3, 7))
    else:
        mesh = grid(rng, 2, 2, rng.uniform(0.5, 2))
    if case % 3 == 0:
        mesh = damaged(rng, mesh, rng.randint(0, 5))
    return mesh


def encode(mesh: Mesh, single: bool) -> list:
    """Precision flag, counts, coordinates and triangle indices; encode(mesh, False)."""
    nodes, tris = mesh
    return [int(single), len(nodes), len(tris)] + [v for p in nodes for v in p] + [v for t in tris for v in t]


def broken(rng: random.Random, mesh: Mesh) -> Mesh:
    """Adds a triangle indexing node 0 or a node past the end; broken(rng, mesh)."""
    nodes, tris = mesh
    return nodes, tris + [(1, rng.choice([0, len(nodes)+1]), 2)]


def transform(rng: random.Random) -> list:
    """Offset, angle or scale, and a form selector 0..4 (identity .. point mirror)."""
    form = rng.randint(0, 4)
    angle = rng.uniform(0.25, 3) * rng.choice([1, -1]) if form == 3 else rng.uniform(-4, 4)
    return [rng.uniform(-3, 3) for _ in range(3)] + [angle, form]


def append_polymesh_requests(requests: list[str]) -> None:
    """Append every Poly request with its own seed; append_polymesh_requests(requests)."""
    rng = random.Random(19950306)
    append_mesh_requests(rng, requests)
    append_geometry_requests(rng, requests)
    append_io_requests(rng, requests)
    append_connect_requests(rng, requests)
    append_coherent_requests(rng, requests)
    append_merge_requests(rng, requests)
    append_loop_requests(rng, requests)


def append_mesh_requests(rng: random.Random, requests: list[str]) -> None:
    """Triangulation queries, including out-of-range node indices."""
    for case in range(240):
        mesh = random_mesh(rng, case)
        if case % 11 == 0:
            mesh = broken(rng, mesh)
        ray = [rng.uniform(-1, 1) for _ in range(3)] + [rng.uniform(-1, 1) for _ in range(3)]
        if case % 17 == 0:
            ray[3:] = [0.0, 0.0, 0.0]
        values = encode(mesh, case % 5 == 4) + transform(rng) + [rng.randint(0, 1)] + ray
        requests.append(line('poly_mesh', values))


def append_geometry_requests(rng: random.Random, requests: list[str]) -> None:
    """PointOnTriangle, IntersectTriLine and PolygonProperties, regular and degenerate."""
    for case in range(300):
        points = [rng.uniform(-2, 2) for _ in range(8)]
        kind = case % 6
        if kind == 1:
            points[2:4] = points[0:2]
        elif kind == 2:
            points[2:6] = points[0:2]*2
        elif kind == 3:
            points[4:6] = [2*points[2]-points[0], 2*points[3]-points[1]]
        elif kind == 4:
            points[6:8] = [(points[0]+points[2]+points[4])/3, (points[1]+points[3]+points[5])/3]
        requests.append(line('poly_pot', points))
    for case in range(300):
        values = [rng.uniform(-2, 2) for _ in range(15)]
        if case % 7 == 0:
            values[3:6] = [0.0, 0.0, 0.0]
        elif case % 7 == 1:
            values[12:15] = values[9:12]
        elif case % 7 == 2:
            values[3:6] = [values[9]-values[6], values[10]-values[7], values[11]-values[8]]
        requests.append(line('poly_triline', values))
    for case in range(120):
        count = case % 9
        points = [rng.uniform(-5, 5) for _ in range(2*count)]
        requests.append(line('poly_props', [count] + points))


def append_io_requests(rng: random.Random, requests: list[str]) -> None:
    """Write/Dump/Read round trips with and without UV nodes and parameters."""
    for case in range(120):
        mesh = random_mesh(rng, case)
        deflection = rng.choice([0.0, 0.001, 1e-7, 123456.789, rng.uniform(0, 1)])
        scale = rng.choice([1.0, 3.0, 1e-5, rng.uniform(0.1, 10)])
        requests.append(line('poly_io', encode(mesh, case % 7 == 3) + [case % 4, deflection, scale]))


def append_connect_requests(rng: random.Random, requests: list[str]) -> None:
    """Adjacency and node fans, including out-of-range node indices."""
    for case in range(160):
        mesh = random_mesh(rng, case)
        if case % 13 == 0:
            mesh = broken(rng, mesh)
        requests.append(line('poly_connect', encode(mesh, False)))


def edits(rng: random.Random, nodes: int, triangles: int, count: int) -> list:
    """Valid edit operations 1..8 for the coherent probe; edits(rng, 4, 2, 3)."""
    values = []
    for _ in range(count):
        op = rng.randint(1, 8)
        corners = rng.sample(range(nodes), 3)
        if op == 1 and triangles:
            values += [1, rng.randint(0, triangles-1), 0, 0, 0]
        elif op == 2:
            values += [2] + corners + [0]
            triangles += 1
        elif op == 3 and triangles:
            values += [3, rng.randint(0, triangles-1)] + corners
        elif op == 4:
            values += [4, rng.randint(0, 6), 0, 0, 0]
        elif op == 7:
            values += [7, rng.randint(-3, 3), rng.randint(-3, 3), rng.randint(-3, 3), 0]
            nodes += 1
        elif op == 8 and triangles:
            values += [8, rng.randint(0, triangles-1), rng.randint(0, 2), 0, 0]
        else:
            values += [rng.choice([5, 6]), 0, 0, 0, 0]
    return values


def append_coherent_requests(rng: random.Random, requests: list[str]) -> None:
    """Coherent triangulations with links, edits and degenerate-side removal."""
    for case in range(220):
        nodes, tris = random_mesh(rng, case)
        if case % 5 == 1 and len(nodes) >= 3:
            nodes = nodes + [tuple(v+rng.choice([0.0, 1e-4]) for v in nodes[2])]
            tris = tris + [(3, len(nodes), 1)]
        kept = sum(1 for a, b, c in tris if a != b and b != c and c != a)
        count = rng.randint(0, 4)
        tolerance = rng.choice([1e-6, 1e-3, 0.05, 0.3, 2.0])
        values = encode((nodes, tris), case % 9 == 5) + [case % 8, tolerance, count]
        requests.append(line('poly_coherent', values + edits(rng, len(nodes), kept, count)))


def append_merge_requests(rng: random.Random, requests: list[str]) -> None:
    """Node merging by distance and angle, with element filters and outputs."""
    for case in range(220):
        nodes, tris = random_mesh(rng, case)
        if case % 2 == 0:
            nodes, tris = damaged(rng, (nodes, tris), 0)
        if len(nodes) < 3:
            continue
        angle = rng.choice([0.0, 0.0, 0.1, 0.5, 1.2, 1.5707, 1.6, 3.2])
        tolerance = rng.choice([0.0, 0.0, 1e-10, 1e-3, 0.05, 0.6])
        unit = rng.choice([1.0, 1.0, 0.001, 25.4])
        facets = rng.choice([-1, 0, 1, 7, len(tris)])
        values = encode((nodes, tris), case % 6 == 5) + transform(rng)
        values += [rng.randint(0, 1), angle, tolerance, unit, rng.randint(0, 15), facets]
        requests.append(line('poly_merge', values))


def loop_graph(rng: random.Random, case: int) -> tuple[list[float], list[int]]:
    """Points and flagged links: polygons, grids, chains and bifurcations."""
    kind = case % 4
    if kind == 0:
        count = rng.randint(3, 7)
        points = [c for k in range(count) for c in (math.cos(2*math.pi*k/count), math.sin(2*math.pi*k/count))]
        links = [(k+1, (k+1) % count+1) for k in range(count)]
    else:
        w = rng.randint(2, 4)
        points = [float(c) for j in range(w) for i in range(w) for c in (i, j)]
        links = []
        for j in range(w):
            for i in range(w):
                p = j*w+i+1
                if i+1 < w and rng.random() < 0.8:
                    links.append((p, p+1))
                if j+1 < w and rng.random() < 0.8:
                    links.append((p, p+w))
        if kind == 3:
            points += [points[0]+0.5, points[1]-1.0]
            links.append((1, w*w+1))
        count = len(points)//2
    flags = [rng.choice([1, 1, 1, 2, 3, 3, 0]) for _ in links]
    if rng.random() < 0.3:
        rng.shuffle(links)
    values = [v for link, flag in zip(links, flags) for v in (link[0], link[1], flag)]
    return [count] + points, [len(links)] + values


def append_loop_requests(rng: random.Random, requests: list[str]) -> None:
    """Loop building in 2D (left and right) and 3D, with orientation edits."""
    for case in range(240):
        points, links = loop_graph(rng, case)
        link_count = links[0]
        if link_count == 0:
            continue
        k = rng.choice([0, rng.randint(0, link_count-1), link_count+5])
        r = rng.choice([link_count+5, rng.randint(0, link_count-1)])
        requests.append(line('poly_loops', [case % 3] + points + links + [k, rng.randint(0, 3), r]))
