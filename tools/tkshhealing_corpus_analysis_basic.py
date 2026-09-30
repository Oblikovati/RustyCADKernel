"""Deterministic requests for ShapeAnalysis_Geom, ShapeAnalysis_ShapeTolerance,
ShapeAnalysis_Shell, ShapeAnalysis_WireOrder and ShapeAnalysis_FreeBoundData
(tools/oracle/tkshhealing_probe_analysis_basic.cxx).

`sh_geom` fits planes to point clouds (planar, noisy, linear, a point, a volume, tiny
and empty clouds) and reads transformations from 3 x 4 matrices (identity, rotations,
left-handed frames, scaled, not orthogonal, unequal columns, translations with units,
too small matrices, a null handle, zero precision).

`sh_shapetol`, `sh_shell` and `sh_fbdata` work on one scene built with a TKTopAlgo shape
program (tools/tktopalgo_shape_program.py): a strip of three planar faces with shared
edges and vertices of different tolerances, shells whose faces use the shared edges
with opposite, equal and INTERNAL orientations, a degenerated edge, a solid, compounds,
free faces and wires.

`sh_wireorder` orders edges given by their ends in the 3D, 2D and miscible modes: in
order, shuffled, reversed, shifted, with gaps, loops (seams), degenerated edges, one
edge, none, with the keep-loops mode, then chains and couples.
"""
from __future__ import annotations
import math
import random
from dataclasses import dataclass
from tktopalgo_shape_program import (COMPOUND, FORWARD, INTERNAL, REVERSED, SHELL, SOLID, WIRE,
                                     ShapeProgram, num)

# TopAbs_ShapeEnum values.
T_COMPOUND, T_COMPSOLID, T_SOLID, T_SHELL, T_FACE, T_WIRE, T_EDGE, T_VERTEX, T_SHAPE = range(9)


def request(op: str, *fields: float | int | str) -> str:
    """One request line."""
    return ' '.join([op, *(f if isinstance(f, str) else num(f) for f in fields)])


def geom_requests() -> list[str]:
    """NearestPlane on point clouds and PositionTrsf on matrices."""
    rng = random.Random(2024)
    clouds = [
        [(0., 0., 0.), (1., 0., 0.), (1., 1., 0.), (0., 1., 0.)],
        [(0., 0., 0.), (2., 0., 0.1), (2., 1., -0.1), (0., 1., 0.05), (1., .5, 0.)],
        [(0., 0., 0.), (1., 1., 1.), (2., 2., 2.)],
        [(1., 2., 3.)] * 3,
        [(0., 0., 0.), (1., 0., 0.), (0., 1., 0.), (0., 0., 1.)],
        [(0., 0., 0.), (1e-9, 0., 0.), (0., 1e-9, 0.)],
        [(0., 0., 5.), (3., 0., 5.), (3., 4., 5.), (0., 4., 5.), (1.5, 2., 5.)],
        [(1., 0., 0.), (0., 1., 0.), (0., 0., 1.), (1., 1., -1.)],
        [(0., 0., 0.), (1., 0., 0.)],
        [],
        [(0., 0., 0.), (4., 0., 0.), (4., 3., 1.), (0., 3., 1.)],
    ]
    clouds.append([(rng.uniform(-5, 5), rng.uniform(-5, 5), rng.uniform(-0.01, 0.01)) for _ in range(20)])
    clouds.append([(rng.uniform(-1, 1), rng.uniform(-1, 1), rng.uniform(-1, 1)) for _ in range(12)])
    out = [request('sh_geom', 0, len(c), *[x for p in c for x in p]) for c in clouds]
    return out + position_requests()


def matrix(rows: list[list[float]]) -> list[float]:
    return [x for row in rows for x in row]


def rotation(axis: tuple[float, float, float], angle: float, scale: float = 1.) -> list[list[float]]:
    """The 3 x 3 rotation about a unit axis, times a scale."""
    x, y, z = axis
    c, s = math.cos(angle), math.sin(angle)
    t = 1 - c
    m = [[t*x*x+c, t*x*y-s*z, t*x*z+s*y], [t*x*y+s*z, t*y*y+c, t*y*z-s*x], [t*x*z-s*y, t*y*z+s*x, t*z*z+c]]
    return [[v*scale for v in row] for row in m]


def with_translation(m: list[list[float]], t: tuple[float, float, float]) -> list[list[float]]:
    return [row+[t[i]] for i, row in enumerate(m)]


def position_requests() -> list[str]:
    """PositionTrsf on 3 x 4 matrices."""
    ident = [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]]
    k = 1/math.sqrt(3.)
    cases = [
        with_translation(ident, (0., 0., 0.)),
        with_translation(ident, (1., -2., 3.)),
        with_translation(rotation((0., 0., 1.), 0.5), (0., 0., 0.)),
        with_translation(rotation((k, k, k), 1.2), (5., 6., 7.)),
        with_translation(rotation((1., 0., 0.), math.pi/2, 2.), (1., 1., 1.)),
        with_translation([[1., 0., 0.], [0., -1., 0.], [0., 0., 1.]], (0., 0., 0.)),
        with_translation([[2., 0., 0.], [0., 2., 0.], [0., 0., 2.]], (0., 0., 0.)),
        with_translation([[1., 0.5, 0.], [0., 1., 0.], [0., 0., 1.]], (0., 0., 0.)),
        with_translation([[1., 0., 0.], [0., 2., 0.], [0., 0., 1.]], (0., 0., 0.)),
        with_translation([[0., 0., 0.], [0., 1., 0.], [0., 0., 1.]], (0., 0., 0.)),
        with_translation([[1.+1e-9, 0., 0.], [0., 1., 0.], [0., 0., 1.-1e-9]], (0., 0., 0.)),
        with_translation([[0., 1., 0.], [1., 0., 0.], [0., 0., 1.]], (2., 0., 0.)),
    ]
    out = []
    for m in cases:
        for unit, prec in ((1., 1e-6), (1000., 1e-6), (0.001, 1e-3), (1., 0.)):
            out.append(request('sh_geom', 1, 0, 1, 3, 1, 4, *matrix(m), unit, prec))
    out.append(request('sh_geom', 1, 1, 1, 0, 1, 0, 1., 1e-6))
    out.append(request('sh_geom', 1, 0, 1, 2, 1, 4, *matrix(cases[1][:2]), 1., 1e-6))
    out.append(request('sh_geom', 1, 0, 1, 3, 1, 3, *matrix([row[:3] for row in cases[1]]), 1., 1e-6))
    out.append(request('sh_geom', 1, 0, 0, 3, 0, 4, *([0.]*20), 1., 1e-6))
    return out


@dataclass
class TopoScene:
    """The program and the named indices of its shapes."""
    program: ShapeProgram
    n: dict[str, int]


def container(p: ShapeProgram, kind: int, children: list[tuple[int, int]]) -> int:
    """A new container of (child, orientation) pairs."""
    shape = p.container(kind)
    for child, orientation in children:
        p.add(shape, child, orientation)
    return shape


def strip_edges(p: ShapeProgram, n: dict[str, int]) -> None:
    """The vertices and edges of a strip of three unit squares along X."""
    tols = [1e-7, 2e-5, 1e-3, 5e-4, 1e-7, 3e-6, 1e-2, 1e-7]
    for i in range(4):
        n[f'a{i}'] = p.vertex((float(i), 0., 0.), tols[i])
        n[f'b{i}'] = p.vertex((float(i), 1., 0.), tols[4+i])

    def edge(name: str, v1: str, v2: str, start, end, tol: float) -> None:
        e = p.edge()
        length = math.dist(start, end)
        direction = tuple((b-a)/length for a, b in zip(start, end))
        p.edge_curve(e, p.line(start, direction), 0, tol)
        p.edge_range(e, 0., length)
        p.add(e, n[v1], FORWARD)
        p.add(e, n[v2], REVERSED)
        n[name] = e
    for i in range(3):
        edge(f'h{i}', f'a{i}', f'a{i+1}', (float(i), 0., 0.), (float(i+1), 0., 0.), 1e-7 * (i+1))
        edge(f't{i}', f'b{i}', f'b{i+1}', (float(i), 1., 0.), (float(i+1), 1., 0.), 4e-6)
    for i in range(4):
        edge(f'v{i}', f'a{i}', f'b{i}', (float(i), 0., 0.), (float(i), 1., 0.), [1e-7, 2e-4, 3e-7, 1e-7][i])
    dg = p.edge()
    p.add(dg, n['a0'], FORWARD)
    p.add(dg, n['a0'], REVERSED)
    p.edge_flags(dg, True, True, True)
    n['deg'] = dg
    ie = p.edge()
    p.edge_curve(ie, p.line((0.5, 0.2, 0.), (0., 1., 0.)), 0, 2e-3)
    p.edge_range(ie, 0., 0.5)
    n['inner'] = ie


def strip_faces(p: ShapeProgram, n: dict[str, int]) -> None:
    """Faces over the strip, shells and containers."""
    plane = p.surface(0, 0., 0., 0., 0., 0., 1.)

    def face(name: str, edges: list[tuple[str, int]], tol: float) -> None:
        w = container(p, WIRE, [(n[e], o) for e, o in edges])
        f = p.face(plane, 0, tol)
        p.add(f, w)
        n[name] = f
        n['w_'+name] = w
    for i in range(3):
        face(f'f{i}', [(f'h{i}', FORWARD), (f'v{i+1}', FORWARD), (f't{i}', REVERSED), (f'v{i}', REVERSED)],
             [1e-7, 6e-5, 2e-6][i])
    # f1 again with its left edge in the same orientation as in f0: a bad edge.
    face('f1bad', [('h1', FORWARD), ('v2', FORWARD), ('t1', REVERSED), ('v1', FORWARD)], 1e-7)
    face('fint', [('h0', FORWARD), ('v1', FORWARD), ('t0', REVERSED), ('v0', REVERSED), ('inner', INTERNAL)], 1e-6)
    face('fdeg', [('h2', FORWARD), ('deg', FORWARD), ('v3', FORWARD), ('t2', REVERSED), ('v2', REVERSED)], 1e-7)
    face('fv1int', [('h1', FORWARD), ('v2', FORWARD), ('t1', REVERSED), ('v1', INTERNAL)], 1e-7)
    n['sh_good'] = container(p, SHELL, [(n['f0'], FORWARD), (n['f1'], FORWARD), (n['f2'], FORWARD)])
    n['sh_bad'] = container(p, SHELL, [(n['f0'], FORWARD), (n['f1bad'], FORWARD)])
    n['sh_twice'] = container(p, SHELL, [(n['f0'], FORWARD), (n['f0'], FORWARD)])
    n['sh_int'] = container(p, SHELL, [(n['f0'], FORWARD), (n['fv1int'], FORWARD), (n['fdeg'], FORWARD)])
    n['sh_one'] = container(p, SHELL, [(n['fint'], FORWARD)])
    n['sh_empty'] = container(p, SHELL, [])
    n['so'] = container(p, SOLID, [(n['sh_good'], FORWARD)])
    n['c_shells'] = container(p, COMPOUND, [(n['sh_bad'], FORWARD), (n['sh_int'], FORWARD), (n['sh_good'], FORWARD)])
    n['c_mixed'] = container(p, COMPOUND, [(n['so'], FORWARD), (n['f2'], FORWARD), (n['h0'], FORWARD),
                                           (n['sh_one'], FORWARD)])
    n['c_empty'] = container(p, COMPOUND, [])
    n['sh_bad_rev'] = p.oriented(n['sh_bad'], REVERSED)


def oriented_copies(p: ShapeProgram, n: dict[str, int]) -> None:
    """REVERSED and INTERNAL copies of every edge, so the probe can name them."""
    for name in [k for k in n if k[0] in 'htv' and k[1:].isdigit()] + ['inner', 'deg']:
        n[name+'_r'] = p.oriented(n[name], REVERSED)
        n[name+'_i'] = p.oriented(n[name], INTERNAL)
    for name in [k for k in n if k[0] == 'f']:
        n[name+'_r'] = p.oriented(n[name], REVERSED)


def topo_scene() -> TopoScene:
    """The shapes every tolerance, shell and free-bound request works on."""
    p = ShapeProgram()
    n: dict[str, int] = {}
    strip_edges(p, n)
    strip_faces(p, n)
    oriented_copies(p, n)
    return TopoScene(p, n)


SHAPES_TOL = ['f0', 'h1', 'a2', 'w_f0', 'sh_good', 'sh_bad', 'so', 'c_shells', 'c_mixed', 'c_empty', 'fint',
              'sh_int', 'deg', 'inner', 'sh_bad_rev']
TYPES = [T_SHAPE, T_VERTEX, T_EDGE, T_FACE, T_SHELL, T_WIRE, T_SOLID]


def shapetol_requests(s: TopoScene) -> list[str]:
    """Tolerance, OverTolerance, InTolerance and the cumulation members."""
    n, prog = s.n, s.program.tokens()
    out = []
    for name in SHAPES_TOL:
        ops = []
        for ty in TYPES:
            ops += [[0, n[name], mode, ty] for mode in (-1, 0, 1)]
        out.append(request('sh_shapetol', prog, len(ops), *[x for op in ops for x in op]))
        ops = []
        for ty in TYPES:
            ops += [[1, n[name], v, ty] for v in (0., 1e-6, 1e-4, -1e-4, 1e-2)]
            ops += [[2, n[name], a, b, ty] for a, b in ((0., 1.), (1e-6, 1e-3), (1e-3, 1e-6), (2e-5, 2e-5), (0., 0.))]
        out.append(request('sh_shapetol', prog, len(ops), *[x for op in ops for x in op]))
    scripts = [
        [[3], [4, n['f0'], T_SHAPE], [5], [4, n['f2'], T_VERTEX], [5], [4, n['c_empty'], T_SHAPE], [5]],
        [[3], [5], [4, n['h0'], T_EDGE], [5], [4, n['v1'], T_EDGE], [5], [3], [5]],
        [[0, n['so'], 0, T_SHAPE], [4, n['fint'], T_FACE], [5], [4, -1, T_SHAPE]],
        [[3], [4, n['a0'], T_VERTEX], [4, n['a0'], T_VERTEX], [5], [4, n['a2'], T_VERTEX], [5]],
        [[0, -1, 1, T_SHAPE], [5], [1, -1, 1e-3, T_SHAPE], [2, -1, 0., 1., T_SHELL]],
    ]
    for ops in scripts:
        out.append(request('sh_shapetol', prog, len(ops), *[x for op in ops for x in op]))
    return out


def shell_requests(s: TopoScene) -> list[str]:
    """LoadShells, CheckOrientedShells and the queries."""
    n, prog = s.n, s.program.tokens()
    out = []
    shapes = ['sh_good', 'sh_bad', 'sh_twice', 'sh_int', 'sh_one', 'sh_empty', 'so', 'c_shells', 'c_mixed',
              'c_empty', 'f0', 'sh_bad_rev']
    for name in shapes:
        for also_free, internal in ((0, 0), (1, 0), (1, 1), (0, 1)):
            ops = [[1, n[name], also_free, internal], [3, n[name]], [4, 1], [4, 0]]
            out.append(request('sh_shell', prog, len(ops), *[x for op in ops for x in op]))
        ops = [[0, n[name]], [3, n[name]], [3, n['sh_good']], [4, 1], [4, 2], [4, -1]]
        out.append(request('sh_shell', prog, len(ops), *[x for op in ops for x in op]))
    scripts = [
        [[0, n['c_shells']], [1, n['c_shells'], 1, 1], [1, n['sh_bad'], 1, 0], [2], [1, n['sh_good'], 1, 0]],
        [[0, -1], [1, -1, 1, 1], [3, -1], [0, n['sh_good']], [0, n['sh_good']], [1, n['sh_bad'], 0, 0]],
        [[1, n['sh_bad'], 1, 0], [1, n['sh_bad'], 1, 0], [1, n['sh_twice'], 0, 0]],
    ]
    for ops in scripts:
        out.append(request('sh_shell', prog, len(ops), *[x for op in ops for x in op]))
    return out


def fbdata_requests(s: TopoScene) -> list[str]:
    """The FreeBoundData setters, notches and queries."""
    n, prog = s.n, s.program.tokens()
    w0, w1, w2 = n['w_f0'], n['w_f1'], n['w_f2']
    scripts = [
        (-2, [], w0),
        (w0, [[0, 2.5], [1, 6.], [2, 1.5], [3, 0.25]], w1),
        (w0, [[4, w1, 0.1], [4, w2, 0.2], [4, w1, 0.3]], w2),
        (w0, [[4, w1, 0.1], [5], [4, w2, 0.2], [0, 1.]], w1),
        (-2, [[6, w2], [4, w0, -1.], [3, 7.]], w0),
        (w1, [[4, n['w_fint'], 0.5], [6, w0], [1, 0.], [5], [5]], n['w_fint']),
    ]
    return [request('sh_fbdata', prog, w, len(ops), *[x for op in ops for x in op], probe)
            for w, ops, probe in scripts]


def wo_request(mode3d: int, tol: float, both: int, ops: list[list[float]]) -> str:
    return request('sh_wireorder', mode3d, tol, both, len(ops), *[x for op in ops for x in op])


def polygon(points: list[tuple[float, ...]]) -> list[tuple[tuple[float, ...], tuple[float, ...]]]:
    """The closed chain of edges through the points."""
    return [(points[i], points[(i+1) % len(points)]) for i in range(len(points))]


def arrangements(edges, rng: random.Random) -> list[list]:
    """The edges in order, reversed, shifted, shuffled, with some reversed, with a gap."""
    out = [list(edges), list(reversed(edges)), edges[2:]+edges[:2]]
    shuffled = list(edges)
    rng.shuffle(shuffled)
    out.append(shuffled)
    out.append([(b, a) if i % 2 else (a, b) for i, (a, b) in enumerate(edges)])
    out.append([(b, a) for a, b in reversed(edges)])
    gap = [(a, tuple(x+0.01 for x in b)) if i == 1 else (a, b) for i, (a, b) in enumerate(edges)]
    out.append(gap)
    return out


FINAL = [[6, 1e-7], [6, 0.05], [7, 0.1]]


def wireorder_requests() -> list[str]:
    """Orders of 3D, 2D and miscible edge lists."""
    rng = random.Random(77)
    square = polygon([(0., 0., 0.), (1., 0., 0.), (1., 1., 0.), (0., 1., 0.)])
    hexagon = polygon([(math.cos(k*math.pi/3), math.sin(k*math.pi/3), 0.) for k in range(6)])
    two_loops = polygon([(0., 0., 0.), (1., 0., 0.), (1., 1., 0.)]) + \
        polygon([(5., 5., 0.), (6., 5., 0.), (6., 6., 0.), (5., 6., 0.)])
    open_chain = [((0., 0., 0.), (1., 0., 0.)), ((1., 0., 0.), (2., 1., 0.)), ((2., 1., 0.), (3., 1., 1.))]
    degenerate = [((0., 0., 0.), (1., 0., 0.)), ((1., 0., 0.), (1., 0., 0.)), ((1., 0., 0.), (0., 0., 0.))]
    out = []
    for edges in (square, hexagon, two_loops, open_chain, degenerate):
        for arranged in arrangements(edges, rng):
            for keep in (0, 1):
                ops = [[8, keep]] + [[0, *a, *b] for a, b in arranged] + [[3, 1]] + FINAL
                out.append(wo_request(1, 1e-7, 0, ops))
            ops = [[1, a[0], a[1], b[0], b[1]] for a, b in arranged] + [[3, 0]] + FINAL
            out.append(wo_request(0, 1e-3, 0, ops))
    out += miscible_requests() + mode_requests()
    return out


def miscible_requests() -> list[str]:
    """The miscible mode: a cylinder band whose seam is used twice (same 3D, other 2D)."""
    pi2 = 2*math.pi
    band = [((1., 0., 0.), (1., 0., 0.), (0., 0.), (pi2, 0.)),
            ((1., 0., 0.), (1., 0., 1.), (pi2, 0.), (pi2, 1.)),
            ((1., 0., 1.), (1., 0., 1.), (pi2, 1.), (0., 1.)),
            ((1., 0., 1.), (1., 0., 0.), (0., 1.), (0., 0.))]
    out = []
    for order in ([0, 1, 2, 3], [0, 3, 2, 1], [2, 0, 3, 1], [1, 3, 0, 2]):
        for flip in (False, True):
            ops = []
            for k, i in enumerate(order):
                a, b, c, d = band[i]
                if flip and k % 2:
                    a, b, c, d = b, a, d, c
                ops.append([2, *a, *b, *c, *d])
            for keep in (0, 1):
                out.append(wo_request(1, 1e-7, 1, [[8, keep]] + ops + [[3, 1]] + FINAL))
    return out


def mode_requests() -> list[str]:
    """Adds in the wrong mode, SetMode, Clear, one edge, no edge, a second Perform."""
    e = [[0, 0., 0., 0., 1., 0., 0.], [0, 1., 0., 0., 1., 1., 0.]]
    return [
        wo_request(1, 1e-7, 0, [[3, 1]] + FINAL),
        wo_request(1, 1e-7, 0, [e[0], [3, 1]] + FINAL),
        wo_request(1, 1e-7, 0, [e[1], e[0], [1, 0., 0., 1., 1.], [2, *([0.]*10)], [3, 1]] + FINAL),
        wo_request(1, 1e-7, 0, [e[1], e[0], [3, 1], [5], [3, 1]] + FINAL),
        wo_request(1, 1e-7, 0, [e[1], e[0], [3, 1], [4, 1, -1., 0]] + FINAL),
        wo_request(1, 1e-7, 0, [e[1], e[0], [3, 1], [4, 0, 0.5, 0], [1, 0., 0., 1., 0.], [3, 1]] + FINAL),
        wo_request(0, 0., 1, [e[1], [2, 0., 0., 0., 1., 0., 0., 0., 0., 1., 0.], [3, 1]] + FINAL),
        wo_request(1, 1e-7, 0, [e[1], e[0], [6, 1e-7], [5], [6, 1e-7]] + FINAL),
    ]


def append_analysis_basic_requests(requests: list[str]) -> None:
    """Append the Geom, ShapeTolerance, Shell, WireOrder and FreeBoundData requests."""
    requests += geom_requests()
    s = topo_scene()
    requests += shapetol_requests(s)
    requests += shell_requests(s)
    requests += fbdata_requests(s)
    requests += wireorder_requests()
