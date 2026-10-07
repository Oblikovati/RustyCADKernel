"""Deterministic requests for Graphic3d_ArrayOfPrimitives (tools/oracle/tkservice_probe_array.cxx).

`sv_array` creates an array of each primitive type with each attribute flag (normals, colors,
texels, bound colors, mutable and deinterleaved attributes, mutable indices), runs a scripted
sequence of vertex, color, edge and bound operations (including indices, counts and types that
raise) and compares the vertex bytes, index values, bounds and the vertices read back.
Operation codes: 1 xyz, 2 xyz+normal, 3 xyz+texel, 4 xyz+normal+texel, 5 rgb color, 6 packed
color, 7 edge, 8 bound, 9 colored bound, 10 strip edges, 11 fan edges, 12 polyline edges,
13 bound color, 14 quad as triangles, 15 IsValid, 16 normal, 17 texel, 18 read a bound color
(only after it was set: the native allocation leaves unset colors uninitialised).
"""
from __future__ import annotations
import random

ARITY = {1: 3, 2: 6, 3: 5, 4: 8, 5: 4, 6: 2, 7: 1, 8: 1, 9: 4, 10: 2, 11: 3, 12: 3, 13: 4, 14: 4, 15: 0, 16: 4, 17: 3}
FLAG_SETS = [0, 1, 2, 4, 7, 16, 18, 0x20, 0x40, 0x41, 0x80, 0xE7]


def request(kind: int, sizes: tuple[int, int, int], flags: int, ops: list[list[float]]) -> str:
    """One `sv_array` line."""
    fields: list[float] = [kind, *sizes, flags, len(ops)]
    for op in ops:
        fields.extend(op)
    return 'sv_array ' + ' '.join(repr(float(v)) for v in fields)


def random_op(rng: random.Random, max_vertexs: int) -> list[float]:
    """A random operation with arguments around the valid ranges."""
    code = rng.choice(list(ARITY))
    index = rng.randint(0, max_vertexs + 1)
    args: list[float]
    if code in (1, 2, 3, 4):
        args = [round(rng.uniform(-10.0, 10.0), 3) for _ in range(ARITY[code])]
    elif code in (5, 13):
        args = [index] + [round(rng.random(), 3) for _ in range(3)]
    elif code == 9:
        args = [rng.randint(1, 4)] + [round(rng.random(), 3) for _ in range(3)]
    elif code in (16, 17):
        args = [index] + [round(rng.uniform(-1.0, 1.0), 3) for _ in range(ARITY[code] - 1)]
    elif code == 6:
        args = [index, rng.randint(-2**31, 2**31 - 1)]
    else:
        args = [rng.randint(0, max_vertexs + 1) for _ in range(ARITY[code])]
    return [code, *args]


def fixed_requests() -> list[str]:
    """Hand-written sequences: a quad as triangles, a strip, a fan, polylines and polygons."""
    quad = [[1, 0, 0, 0], [1, 1, 0, 0], [1, 1, 1, 0], [1, 0, 1, 0], [14, 1, 2, 3, 4], [15]]
    strip = [[1, i, i % 2, 0] for i in range(5)] + [[10, 1, 5], [15]]
    fan = [[2, 0, 0, 0, 0, 0, 1]] + [[2, i, 1, 0, 0, 0, 1] for i in range(4)] + [[11, 1, 5, 1], [15]]
    polyline = [[1, i, 0, 0] for i in range(4)] + [[12, 1, 4, 1], [15]]
    bounds = [[1, i, i, i] for i in range(6)] + [[9, 3, 1, 0, 0], [9, 3, 0, 1, 0], [9, 1, 0, 0, 1], [13, 1, 0.5, 0.5, 0.5], [18, 1], [18, 2], [18, 3], [15]]
    return [request(4, (4, 0, 6), 0, quad), request(4, (5, 0, 9), 0, strip), request(4, (5, 0, 12), 0, fan),
            request(2, (4, 0, 8), 2, polyline), request(13, (6, 2, 0), 16, bounds), request(3, (6, 2, 0), 18, bounds),
            request(5, (6, 2, 0), 0, bounds), request(0, (4, 0, 0), 0, []), request(4, (0, 0, 0), 0, [[1, 0, 0, 0], [15]])]


def append_array_requests(requests: list[str]) -> None:
    """Append the fixed sequences, then seeded random ones over every type and flag set."""
    requests.extend(fixed_requests())
    rng = random.Random(20261014)
    for kind in range(1, 14):
        for flags in FLAG_SETS:
            max_vertexs = rng.randint(1, 8)
            sizes = (max_vertexs, rng.choice([0, 0, 1, 3]), rng.choice([0, 0, 2, 6]))
            ops = [random_op(rng, max_vertexs) for _ in range(rng.randint(0, 14))] + [[15]]
            requests.append(request(kind, sizes, flags, ops))
