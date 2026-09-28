"""TKG3d corpus: the GeomHash hashers (tools/oracle/tkg3d_probe_hash.cxx).

Handle items are `0 <object> | 1 (null) | 2 index` (an alias of an earlier item); a
negative CompTolerance selects the default hasher constructor. Pairs that the native
comparisons would read past the second object's poles (equal knots but a different pole
count) terminate the native process, so no request compares such a pair.
"""
from __future__ import annotations
import random
from tkg3d_corpus import (bezier, bspline, circle, ellipse, fmt, frame, hyperbola, line, offset, parabola,
                          point, trimmed, unit)
from tkg3d_corpus_bezier_surface import bezier_surface
from tkg3d_corpus_bspl import bspline_random
from tkg3d_corpus_bspline_surface import bspline as bspline_surface
from tkg3d_corpus_offset_surface import offset as offset_surface, trim
from tkg3d_corpus_surfaces import cone, cylinder, extrusion, plane, revolution, sphere, torus

TOLERANCES = [(-1.0, 0.0), (1e-12, 1e-7), (0.1, 0.1), (1e-12, 0.0), (1e-12, 1e-300), (0.0, 1e-7), (1e-6, 1e-3),
              (1e-7, 2.5), (2.220446049250313e-16, 2.220446049250313e-16)]
Z_FRAME = [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0]


def with_aliases(rng: random.Random, objects: list[list], aliases: list[int], nulls: int) -> list[list]:
    """Items for `objects`, `nulls` null handles and aliases of the given earlier indices, shuffled
    before the aliases so every alias names an earlier item."""
    items = [[0, *spec] for spec in objects] + [[1] for _ in range(nulls)]
    rng.shuffle(items)
    return items + [[2, min(index, len(items) - 1)] for index in aliases]


def request(op: str, tolerances, items: list[list]) -> str:
    return op + ' ' + fmt([*tolerances, len(items), *[c for item in items for c in item]])


def curve_objects(rng: random.Random) -> list[list]:
    """Copies, near copies and every curve class GeomHash dispatches on."""
    near = 5.0 + 5e-13
    tilted = [0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0]
    bez = [[0, 0, 0, 1], [1, 2, 0, 1], [3, 2, 0, 1], [4, 0, 0, 1]]
    spline = bspline_random(rng, 3, 2, True)
    base = line([0, 0, 0], [1, 0, 0])
    return [line([1, 2, 3], [1, 0, 0]), line([1, 2, 3], [1, 0, 0]), line([1, 2, 3 + 1e-8], [1, 0, 0]),
            line([1e10, 1e10, 1e10], [0, 1, 0]), line(point(rng), unit(rng)),
            circle(Z_FRAME, 5.0), circle(Z_FRAME, 5.0), circle(Z_FRAME, near), circle(tilted, 5.0),
            circle(Z_FRAME, 1e-10), circle(Z_FRAME, 1e10), ellipse(Z_FRAME, 10.0, 5.0), ellipse(Z_FRAME, 5.0, 5.0),
            hyperbola(Z_FRAME, 5.0, 3.0), hyperbola(frame(rng), 5.0, 3.0), parabola(Z_FRAME, 2.0),
            parabola(Z_FRAME, 2.0 + 1e-11), bezier(bez), bezier(bez), bezier(bez[:3]),
            bezier([[0, 0, 0, 1], [1, 2, 0, 2], [3, 2, 0, 1], [4, 0, 0, 1]], True),
            spline, spline, bspline_random(rng, 3, 2, False), bspline_random(rng, 2, 0, False),
            bspline(1, [[0.0, 2], [1.0, 1], [2.0, 1], [3.0, 2]], [[0, 0, 0, 1], [1, 1, 0, 1], [2, 0, 0, 1], [3, 1, 0, 1]]),
            bspline(2, [[0.5, 1], [1.5, 1], [2.5, 1], [3.5, 1], [4.5, 1]],
                    [[0, 0, 0, 1], [1, 1, 0, 1], [2, 0, 0, 1], [1, -1, 0, 1]], periodic=True),
            trimmed(0.0, 10.0, base), trimmed(0.0, 10.0, base), trimmed(0.0, 20.0, base),
            trimmed(0.5, 2.0, circle(Z_FRAME, 5.0)), offset(5.0, [0, 0, 1], base), offset(5.0, [0, 0, 1], base),
            offset(5.0, [0, 1, 0], base), offset(0.5, [0, 0, 1], circle(Z_FRAME, 5.0))]


def surface_objects(rng: random.Random) -> list[list]:
    """Copies, near copies and every surface class GeomHash dispatches on."""
    z_axis = [0.0, 0.0, 0.0, 0.0, 0.0, 1.0]
    left = [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0]
    net = [[0, 0, 0, 1], [1, 0, 0, 1], [0, 1, 0, 1], [1, 1, 1, 1]]
    weighted = [[0, 0, 0, 1], [1, 0, 0, 1], [0, 1, 0, 1], [1, 1, 1, 2]]
    bilinear = bspline_surface(rng, (1, 1), [[0.0, 2], [1.0, 2]], [[0.0, 2], [1.0, 2]]).values
    cubic = bspline_surface(rng, (3, 2), [[0.0, 4], [0.3, 1], [1.0, 4]], [[0.0, 3], [1.0, 3]],
                            weights=lambda i, j: 1.0 + 0.1 * i).values
    flat = plane(Z_FRAME)
    return [flat, flat, plane(left), plane([0.0, 0.0, 1.0, *Z_FRAME[3:]]), plane([1e10, 1e10, 1e10, *Z_FRAME[3:]]),
            cylinder(Z_FRAME, 5.0), cylinder(Z_FRAME, 5.0), cylinder(frame(rng), 5.0), cone(Z_FRAME, 0.5, 5.0),
            cone(Z_FRAME, 0.5 + 1e-13, 5.0), sphere(Z_FRAME, 5.0), sphere(Z_FRAME, 1e-10), sphere(Z_FRAME, 1e10),
            torus(Z_FRAME, 10.0, 3.0), torus(Z_FRAME, 10.0, 5.0),
            extrusion([0, 0, 1], line([0, 0, 0], [1, 0, 0])), extrusion([0, 1, 0], line([0, 0, 0], [1, 0, 0])),
            revolution(z_axis, line([1, 0, 0], [0, 0, 1])), revolution(z_axis, line([2, 0, 0], [0, 0, 1])),
            revolution(z_axis, circle([5.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0], 1.0)),
            bezier_surface(2, 2, net), bezier_surface(2, 2, net), bezier_surface(2, 2, weighted, True),
            bilinear, bilinear, cubic, trim(0.0, 10.0, 0.0, 10.0, flat), trim(0.0, 20.0, 0.0, 10.0, flat),
            trim(0.1, 0.9, 0.2, 0.8, bezier_surface(2, 2, net)), offset_surface(5.0, flat),
            offset_surface(10.0, flat), offset_surface(5.0, plane([1.0, 0.0, 0.0, *Z_FRAME[3:]]))]


EDGES = [[0.0, -0.0, 0.0], [1.5e-7, -1.5e-7, 2.5e-7], [1e300, -1e300, 0.0], [9.223372036854775808e11, 0.0, -1e-300],
         [-9.223372036854775808e11, 9.2233720368547748e11, 3.0]]


def primitive_request(rng: random.Random, tolerances, index: int) -> str:
    p1 = point(rng) if index < 3 else rng.choice(EDGES)
    p2 = p1 if index % 3 == 0 else (rng.choice(EDGES) if index % 3 == 1 else point(rng))
    d1 = unit(rng)
    d2 = d1 if index % 2 == 0 else unit(rng)
    v1 = rng.choice(EDGES) if index % 2 else point(rng)
    v2 = v1 if index % 3 == 0 else point(rng)
    f1 = frame(rng)
    f2 = f1 if index % 3 == 0 else frame(rng)
    return 'g3_hash_primitives ' + fmt([*tolerances, *p1, *p2, *d1, *d2, *v1, *v2, *f1, *f2])


def poly2d(rng: random.Random, deflection: float, n: int, shift: float = 0.0) -> list:
    return [deflection, n, *[c for i in range(n) for c in (float(i) + shift, rng.uniform(-1, 1))]]


def poly3d(rng: random.Random, deflection: float, n: int, parameters: bool) -> list:
    nodes = [c for i in range(n) for c in (float(i), rng.uniform(-1, 1), rng.uniform(-1, 1))]
    return [deflection, int(parameters), n, *nodes, *([0.1 * i for i in range(n)] if parameters else [])]


def ontri(deflection: float, nodes: list[int], parameters: bool) -> list:
    return [deflection, int(parameters), len(nodes), *nodes, *([0.25 * i for i in range(len(nodes))] if parameters else [])]


def triangulation(deflection: float, uv: bool, far: float = 1.0) -> list:
    nodes = [0, 0, 0, 1, 0, 0, 0, far, 0, 1, 1, 0.5]
    uvs = [0, 0, 1, 0, 0, 1, 1, 1] if uv else []
    return [deflection, int(uv), 4, 2, *nodes, *uvs, 1, 2, 3, 2, 4, 3]


def append_mesh_requests(rng: random.Random, requests: list[str]) -> None:
    for tolerances in TOLERANCES + [(0.1, 0.01)]:
        polygons = [poly2d(rng, 0.004, 2), poly2d(rng, 0.0044, 2), poly2d(rng, 0.06, 3), poly2d(rng, 0.004, 0),
                    poly2d(rng, 1e300, 1, 1e20)]
        requests.append(request('g3_hash_poly2d', tolerances, with_aliases(rng, polygons, [0, 2], 2)))
        polygons = [poly3d(rng, 0.004, 2, False), poly3d(rng, 0.0044, 2, False), poly3d(rng, 0.004, 3, True),
                    poly3d(rng, 0.004, 3, True), poly3d(rng, 0.004, 0, False), poly3d(rng, 0.004, 0, True)]
        requests.append(request('g3_hash_poly3d', tolerances, with_aliases(rng, polygons, [1, 3], 2)))
        meshes = [triangulation(0.004, False), triangulation(0.0044, False), triangulation(0.004, True),
                  triangulation(0.004, False, 2.0), triangulation(0.004, True, 1e-9)]
        requests.append(request('g3_hash_triangulation', tolerances, with_aliases(rng, meshes, [0], 2)))
        keys = [[7, 0, *ontri(0.004, [1, 2], False)], [7, 0, *ontri(0.0044, [1, 2], False)],
                [7, 0, *ontri(0.004, [1, 3], False)], [8, 0, *ontri(0.004, [1, 2], True)],
                [4294967295, 0, *ontri(0.004, [3, 1, 2], True)], [7, 0, *ontri(0.004, [], False)], [7, 1], [8, 1],
                [7, 2, 0], [9, 2, 3]]
        requests.append('g3_hash_ontri ' + fmt([*tolerances, len(keys), *[c for key in keys for c in key]]))


def append_hash_requests(requests: list[str]) -> None:
    """Append every GeomHash request; append_hash_requests(requests)."""
    rng = random.Random(0x6E0A5)
    for tolerances in TOLERANCES:
        requests.append(request('g3_hash_curves', tolerances, with_aliases(rng, curve_objects(rng), [0, 5, 20], 2)))
        requests.append(request('g3_hash_surfaces', tolerances,
                                with_aliases(rng, surface_objects(rng), [0, 7, 25], 2)))
        requests += [primitive_request(rng, tolerances, index) for index in range(6)]
    # A failing construction ends the request with the exception type.
    requests.append(request('g3_hash_curves', (1e-12, 1e-7), [[0, *circle(Z_FRAME, -1.0)]]))
    requests.append(request('g3_hash_surfaces', (1e-12, 1e-7), [[0, *sphere(Z_FRAME, -1.0)]]))
    append_mesh_requests(rng, requests)
