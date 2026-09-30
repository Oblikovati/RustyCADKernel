"""Deterministic requests for IntPatch_PrmPrmIntersection
(tools/oracle/tkgeomalgo_probe_intpatch_prmprm.cxx).

`ip_ppgrid` runs the grid members (the cell codes, the rejection codes and the filling
of the cells of segments and triangles, with degenerate and out-of-grid cells) on one
bit map. `ip_ppdepart` computes the start points of PointDepart on crossing, tangent,
disjoint and far apart surface pairs (planes, a sphere, a Bezier and a B-spline).
"""
from __future__ import annotations
import math
from tkgeomalgo_corpus_intcurvesurface import bez_s, bsp_s, cyl, line_of, plane, sad, sphere


def script(*cmds):
    return [len(cmds), *[v for c in cmds for v in c]]


def _grid_requests(requests: list[str]) -> None:
    codes = [[1, 0, 0, 0], [1, 127, 127, 127], [1, 5, 3, 1], [1, -1, 2, 3], [1, 128, 0, 0],
             [2, 0], [2, 16384 * 3 + 128 * 2 + 1], [2, -1], [2, 2097151], [2, 2097152 + 5],
             [3, -1], [3, 0], [3, 127], [3, 128], [4]]
    rejects = [[5, 0, 0, 0, 1, 1, 1, x, y, z] for x, y, z in
               [(0.5, 0.5, 0.5), (-1.0, 0.5, 2.0), (2.0, -1.0, -1.0), (0.0, 1.0, 0.5), (1.0, 1.0, 1.0)]]
    requests.append(line_of('ip_ppgrid', script(*codes, *rejects)))
    lines = [[6, 0, 0, 0, 5, 3, 1], [6, 10, 10, 10, 10, 10, 10], [6, -3, -2, 0, 4, 5, 6],
             [6, 120, 120, 120, 130, 127, 0], [6, 0, 0, 0, 1, 1, 1]]
    requests.append(line_of('ip_ppgrid', script(*lines)))
    tris = [[7, 1, 1, 1, 1, 1, 1, 1, 1, 1], [7, 0, 0, 0, 9, 0, 0, 0, 9, 0], [7, 5, 5, 5, 6, 5, 5, 5, 6, 5],
            [7, 0, 0, 0, 1, 0, 0, 2, 0, 0], [7, 2, 0, 0, 0, 0, 0, 1, 0, 0], [7, 0, 0, 0, 2, 0, 0, 1, 0, 0],
            [7, 100, 20, 3, 60, 90, 40, 10, 10, 120], [7, -5, -5, -5, 130, 0, 0, 0, 130, 0]]
    requests.append(line_of('ip_ppgrid', script(*tris)))
    code = lambda x, y, z: x | (y << 7) | (z << 14)
    remplit = [[8, -1, -1, -1], [8, code(1, 2, 3), -1, code(4, 5, 6)],
               [8, code(1, 1, 1), code(8, 2, 1), code(3, 9, 7)], [8, code(0, 0, 0), code(0, 0, 0), code(0, 0, 0)]]
    requests.append(line_of('ip_ppgrid', script(*remplit)))


GRID = [[[-1.0 + 0.7 * i, -1.0 + 0.7 * j, 0.3 * math.sin(i + j)] for j in range(4)] for i in range(4)]


def depart(requests, s1, su1, sv1, s2, su2, sv2):
    requests.append(line_of('ip_ppdepart', [*s1, su1, sv1, *s2, su2, sv2]))


def _depart_requests(requests: list[str]) -> None:
    xy = sad(plane([0.0, 0.0, 0.0], [0.0, 0.0, 1.0]), [-1.0, 1.0, -1.0, 1.0])
    xz = sad(plane([0.0, 0.1, 0.0], [0.0, 1.0, 0.0]), [-1.0, 1.0, -1.0, 1.0])
    tilted = sad(plane([0.0, 0.0, 0.2], [0.3, 0.2, 1.0]), [-1.0, 1.0, -1.0, 1.0])
    ball = sad(sphere([0.2, 0.1, 0.0], [0.0, 0.0, 1.0], 0.6))
    touch = sad(sphere([0.0, 0.0, 0.5], [0.0, 0.0, 1.0], 0.5))
    far = sad(sphere([10.0, 0.0, 0.0], [0.0, 0.0, 1.0], 0.5))
    bez = sad(bez_s(GRID))
    bsp = sad(bsp_s(2, 2, GRID, [0.0, 1.0, 2.0], [3, 1, 3], [0.0, 1.0, 2.0], [3, 1, 3]))
    column = sad(cyl([0.1, 0.0, 0.0], [1.0, 0.0, 0.0], 0.4), [0.0, 2.0 * math.pi, -1.0, 1.0])
    depart(requests, xy, 1, 1, xz, 1, 1)
    depart(requests, xy, 2, 2, xz, 1, 3)
    depart(requests, xy, 2, 2, tilted, 2, 2)
    depart(requests, xy, 2, 2, ball, 2, 2)
    depart(requests, xy, 2, 2, touch, 2, 2)
    depart(requests, xy, 2, 2, far, 2, 2)
    depart(requests, bez, 2, 2, xy, 2, 2)
    depart(requests, bez, 1, 2, bsp, 2, 1)
    depart(requests, bsp, 2, 2, column, 2, 2)
    depart(requests, ball, 3, 3, column, 1, 1)
    depart(requests, xy, 20, 20, xz, 1, 1)
    depart(requests, xy, 0, 2, xz, 2, 2)


def append_intpatch_prmprm_requests(requests: list[str]) -> None:
    """The PrmPrm requests."""
    _grid_requests(requests)
    _depart_requests(requests)
