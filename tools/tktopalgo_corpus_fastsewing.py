"""Deterministic requests for BRepBuilderAPI_FastSewing (tools/oracle/tktopalgo_probe_fastsewing.cxx).

The soups are grids of Bezier patches of degree 1 on a bumpy height field: quads (two
by two poles at the grid corners) and triangles (a quad split in two, the third corner
doubled, so that one side of each patch is degenerated), exact or with the corners
moved by noise below or above the tolerance. Other requests add rejected surfaces
(infinite, trimmed, null), closed surfaces (a sphere and a torus, whose corners
coincide), faces of a shape record, a patch whose side is a diagonal of an earlier one
(an exception inside Perform) and an empty input.
"""
from __future__ import annotations
import math
from tktopalgo_corpus_breplib_records import numbers
from tktopalgo_corpus_shapes import FaceRecord, ShapeSpec, FACE

Z = [0., 0., 0., 0., 0., 1.]  # the frame fields (P N) that the Bezier surfaces ignore


def height(x: float, y: float) -> float:
    return 0.3 * math.sin(1.3 * x) * math.cos(0.7 * y)


def noise(i: int, j: int, amplitude: float) -> tuple[float, float, float]:
    """A deterministic displacement of the grid point (i, j)."""
    return (amplitude * math.sin(12.9898 * i + 78.233 * j), amplitude * math.cos(39.346 * i + 11.135 * j),
            amplitude * math.sin(73.156 * i - 52.235 * j))


def grid_point(i: int, j: int, amplitude: float = 0.0, shift: int = 0) -> list[float]:
    """The grid corner (i, j) on the height field, moved by the noise of its copy."""
    d = noise(i + 31 * shift, j + 17 * shift, amplitude)
    return [i + d[0], j + d[1], height(i, j) + d[2]]


def bezier(poles: list[list[list[float]]]) -> list[float]:
    """A surface record of the face reader: a Bezier surface (U rows of V poles)."""
    return [6, *Z, len(poles), len(poles[0]), *[c for row in poles for p in row for c in p]]


def quads(n: int, amplitude: float) -> list[list[float]]:
    """The patches of an n x n quad grid; each patch moves its own copy of a corner."""
    patches = []
    for i in range(n):
        for j in range(n):
            s = i * n + j
            p = [[grid_point(i, j, amplitude, s), grid_point(i, j + 1, amplitude, s)],
                 [grid_point(i + 1, j, amplitude, s), grid_point(i + 1, j + 1, amplitude, s)]]
            patches.append(bezier(p))
    return patches


def triangles(n: int, amplitude: float) -> list[list[float]]:
    """The patches of an n x n grid of quads split into two triangles each."""
    patches = []
    for i in range(n):
        for j in range(n):
            s = i * n + j
            a, b = grid_point(i, j, amplitude, s), grid_point(i + 1, j, amplitude, s)
            c, d = grid_point(i + 1, j + 1, amplitude, s), grid_point(i, j + 1, amplitude, s)
            patches.append(bezier([[a, b], [c, c]]))
            patches.append(bezier([[a, c], [d, d]]))
    return patches


def item_surfaces(surfaces: list[list[float]]) -> list[float]:
    return [c for s in surfaces for c in [0, *s]]


def request(tol: float, items: list[float], count: int, settol: float = -1.0, perform: int = 1) -> str:
    return 'bba_fastsew ' + numbers([tol, settol, perform, count, *items])


def shape_item() -> list[float]:
    """A shape record of two Bezier faces (natural restriction, no wire) sharing a side."""
    spec = ShapeSpec()
    for i in range(2):
        poles = [[[i, 0., 0.], [i, 1., 0.]], [[i + 1., 0., 0.2 * i], [i + 1., 1., 0.2 * i]]]
        spec.face(FaceRecord([5, 2, 2, *[c for row in poles for p in row for c in p]], natural=True))
    spec.top = [(FACE, 0), (FACE, 1)]
    return [1, *spec.numbers()]


def special_items() -> list[tuple[list[float], int]]:
    """Rejected and closed surfaces, a shape, null inputs."""
    plane = [0, *Z]
    trimmed = [3, *Z, 0., 1., 0., 1.]
    cylinder = [1, *Z, 2.0]
    sphere = [2, *Z, 2.0]
    torus = [5, *Z, 3.0, 1.0]
    return [(item_surfaces([plane]), 1), (item_surfaces([trimmed]), 1), (item_surfaces([cylinder]), 1),
            (item_surfaces([sphere]), 1), (item_surfaces([torus]), 1), (shape_item(), 1), ([2], 1), ([3], 1),
            (item_surfaces([plane, trimmed, sphere]) + shape_item() + [2, 3], 6)]


def diagonal_items() -> list[float]:
    """A unit quad and a quad whose first side is the quad's diagonal."""
    a, b, c, d = [0., 0., 0.], [1., 0., 0.], [1., 1., 0.], [0., 1., 0.]
    first = bezier([[a, d], [b, c]])
    second = bezier([[a, [0.5, 1.5, 0.]], [c, [1.5, 1.5, 0.]]])
    return item_surfaces([first, second])


def append_fastsewing_requests(requests: list[str]) -> None:
    """Append the FastSewing requests in a fixed order."""
    for n in (1, 2, 4):
        for amplitude, tol in ((0.0, 1e-6), (1e-5, 1e-6), (1e-5, 1e-4), (1e-3, 1e-4)):
            q, t = quads(n, amplitude), triangles(n, amplitude)
            requests.append(request(tol, item_surfaces(q), len(q)))
            requests.append(request(tol, item_surfaces(t), len(t)))
    q = quads(2, 1e-5)
    requests += [request(1e-6, item_surfaces(q), len(q), settol=1e-4), request(1e-4, item_surfaces(q), len(q), perform=0),
                 request(1e-6, [], 0), request(1e-6, diagonal_items(), 2)]
    for items, count in special_items():
        requests.append(request(1e-6, items, count))
        requests.append(request(1e-6, items, count, perform=0))
