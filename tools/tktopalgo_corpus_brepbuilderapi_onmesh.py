"""Deterministic requests for BRepBuilderAPI_MakeShapeOnMesh (tools/oracle/tktopalgo_probe_brepbuilderapi_onmesh.cxx).

A `bba_onmesh` request carries its triangulation (nodes, triangles, deflection) and the
number of explicit Build calls (0: Shape() builds), then prints IsDone, Check, the digest
of the compound and its history. The meshes: a null mesh, meshes without nodes or
triangles, one triangle in every index order, two triangles with consistent and
opposite orientations, a tetrahedron, an octahedron, height-field grids, triangles with
a repeated index, with coincident nodes (skipped), with collinear nodes (no face), with
nodes 1e-4 apart (built) and 1e-8 apart (the edge maker fails: StdFail_NotDone).
"""
from __future__ import annotations
import math
from tktopalgo_corpus_brepbndlib import numbers

Mesh = tuple[list[list[float]], list[tuple[int, int, int]]]


def mesh_fields(mesh: Mesh | None, deflection: float = 0.01) -> list[float]:
    """The mesh layout of the probe; None is the null mesh."""
    if mesh is None:
        return [-1]
    nodes, triangles = mesh
    fields: list[float] = [len(nodes)]
    for node in nodes:
        fields.extend(node)
    fields.append(len(triangles))
    for triangle in triangles:
        fields.extend(triangle)
    return [*fields, deflection]


def grid(n: int, height) -> Mesh:
    """An n x n grid of squares on z = height(x, y), two triangles per square."""
    nodes = [[float(i), float(j), height(i, j)] for j in range(n + 1) for i in range(n + 1)]
    triangles = []
    for j in range(n):
        for i in range(n):
            a = j * (n + 1) + i + 1
            b, c, d = a + 1, a + n + 2, a + n + 1
            triangles += [(a, b, c), (a, c, d)]
    return nodes, triangles


TRIANGLE = [[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 1.0, 0.5]]
SQUARE = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]]
TETRA = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
OCTA = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [-1.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, 1.0], [0.0, 0.0, -1.0]]


def meshes() -> list[Mesh | None]:
    """Every mesh of the corpus."""
    orders = [(1, 2, 3), (1, 3, 2), (2, 1, 3), (2, 3, 1), (3, 1, 2), (3, 2, 1)]
    return [
        None,
        ([], []),
        (TRIANGLE, []),
        *[(TRIANGLE, [order]) for order in orders],
        (SQUARE, [(1, 2, 3), (1, 3, 4)]),
        (SQUARE, [(1, 2, 3), (1, 4, 3)]),
        (TETRA, [(1, 3, 2), (1, 2, 4), (2, 3, 4), (3, 1, 4)]),
        (OCTA, [(1, 2, 5), (2, 3, 5), (3, 4, 5), (4, 1, 5), (2, 1, 6), (3, 2, 6), (4, 3, 6), (1, 4, 6)]),
        grid(3, lambda i, j: 0.0),
        grid(4, lambda i, j: 0.25 * math.sin(i) * math.cos(j)),
        (SQUARE, [(1, 1, 2), (1, 2, 3), (3, 3, 3)]),
        ([*SQUARE, [1.0, 0.0, 0.0]], [(1, 2, 5), (1, 2, 3), (1, 3, 4)]),
        ([*SQUARE, [2.0, 0.0, 0.0]], [(1, 2, 5), (1, 3, 4)]),
        ([*TRIANGLE, [1e-4, 0.0, 0.0]], [(1, 4, 3), (1, 2, 3)]),
        ([*TRIANGLE, [1e-8, 0.0, 0.0]], [(1, 2, 3), (1, 4, 3)]),
    ]


def append_brepbuilderapi_onmesh_requests(requests: list[str]) -> None:
    """Append every `bba_onmesh` request: each mesh built by Shape(), once and twice."""
    for mesh in meshes():
        for builds in (0, 1, 2):
            requests.append('bba_onmesh ' + numbers([*mesh_fields(mesh), builds]))
