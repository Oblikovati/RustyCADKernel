"""Deterministic requests for IntPatch_WLineTool and IntPatch_LineConstructor
(tools/oracle/tkgeomalgo_probe_intpatch_wltool.cxx).

`ip_purge` purges walking lines: two equal points, equal points among a few, long
straight and curved lines (the tube criteria), lines leaving the domains at either end
(the outer points), periodic and C0 surfaces, uneven steps, the bad distributions, and
vertices on and out of the line. `ip_join` joins the walking lines of two cylinders in
the four end orders, across the seam, with several candidates, with lines of another
kind and with points at the ends. `ip_lc` splits every line kind (walking, restriction,
conic, analytic) at its vertices against domains made of the fixture arcs, with and
without a 3d tolerance, and codes walking lines on the arcs as restrictions.
"""
from __future__ import annotations
import math
from tkgeomalgo_corpus_intcurvesurface import bsp_s, cone, cyl, line_of, plane, sad, sphere
from tkgeomalgo_corpus_intpatch import free_point, p_arc, p_para, p_params, p_value, p_vertex, point

UNDECIDED = [0, 0]
TYPES = [1, 1, 0, 1]
TOUCH = [2, 1, 0, 1]
OWN = [-1, -1.0]
TWO_PI = 2.0 * math.pi

# A plane z = 0 over [0, 4] x [0, 10] (the fixture arcs u = 1, 2, 3 cross it) and a plane
# y = 0 over [-5, 5] x [-5, 5].
S_XY = sad(plane([0.0, 0.0, 0.0], [0.0, 0.0, 1.0]), [0.0, 4.0, 0.0, 10.0])
S_XZ = sad(plane([0.0, 0.0, 0.0], [0.0, 1.0, 0.0]), [-5.0, 5.0, -5.0, 5.0])
S_CYL = sad(cyl([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1.0), [0.0, TWO_PI, -5.0, 5.0])
S_CYLX = sad(cyl([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 1.0), [0.0, TWO_PI, -5.0, 5.0])
GRID = [[[i, j, 0.1 * i * j] for j in range(4)] for i in range(4)]
S_C0 = sad(bsp_s(1, 1, GRID, [0.0, 1.0, 2.0, 3.0], [2, 1, 1, 2], [0.0, 1.0, 2.0, 3.0], [2, 1, 1, 2]))
S_C2 = sad(bsp_s(3, 3, GRID, [0.0, 1.0], [4, 4], [0.0, 1.0], [4, 4]))


def domain(*ranks, tol=-1.0):
    return [len(ranks), *ranks, tol]


def pts(rows):
    """`n (x y z u1 v1 u2 v2)*`."""
    return [len(rows), *[v for row in rows for v in row]]


def vtx(*scripts):
    return [len(scripts), *[v for s in scripts for v in s]]


def wline(rows, vertices=(), way=3, trans=UNDECIDED, periods=(0.0, 0.0, 0.0, 0.0), arcs=(0, 0)):
    return [0, *trans, way, *periods, *pts(rows), *vtx(*vertices), *arcs]


def at(x, para, u1=None, v1=0.0, u2=None, v2=0.0, tol=1.0e-7):
    """A vertex at `(x, 0, 0)` of parameter `para` on the line."""
    return free_point(x, 0.0, 0.0, x if u1 is None else u1, v1, x if u2 is None else u2, v2, para, tol)


def straight(n, step=0.1, x0=0.5, v1=5.0):
    """`n` evenly spaced points on the x axis, on the first plane at `(x, v1)`."""
    return [[x0 + i * step, 0.0, 0.0, x0 + i * step, v1, x0 + i * step, 0.0] for i in range(n)]


def wavy(n, step=0.08, x0=0.3, amp=0.3):
    rows = []
    for i in range(n):
        x = x0 + i * step
        y = amp * math.sin(3.0 * x)
        rows.append([x, y, 0.1 * x * x, x, 5.0 + y, x, 0.1 * x * x])
    return rows


def purge(requests, s1, s2, line, d1=OWN, d2=OWN):
    requests.append(line_of('ip_purge', [*s1, *s2, *d1, *d2, *line]))


def _purge_requests(requests: list[str]) -> None:
    same = [0.5, 0.0, 0.0, 0.5, 5.0, 0.5, 0.0]
    purge(requests, S_XY, S_XZ, wline([same, same]))
    purge(requests, S_XY, S_XZ, wline([same, [0.6, 0.0, 0.0, 0.6, 5.0, 0.6, 0.0]]))
    dup = straight(5)
    purge(requests, S_XY, S_XZ, wline([dup[0], dup[1], dup[1], dup[2], [9.0, 9.0, 9.0, *dup[2][3:]], dup[3]],
                                      vertices=[at(0.5, 1.0), at(0.8, 6.0)]))
    purge(requests, S_XY, S_XZ, wline([dup[0], dup[0], dup[0]], vertices=[at(0.5, 3.0)]))
    long_line = straight(40)
    ends = [at(0.5, 1.0), at(long_line[-1][0], 40.0)]
    purge(requests, S_XY, S_XZ, wline(long_line))
    purge(requests, S_XY, S_XZ, wline(long_line, vertices=ends))
    purge(requests, S_XY, S_XZ, wline(long_line, vertices=[*ends, at(2.0, 16.0)]))
    curve = wavy(45)
    for way in (0, 1, 3):
        purge(requests, S_XY, S_XZ, wline(curve, vertices=[at(0.3, 1.0), at(curve[-1][0], 45.0)], way=way))
    purge(requests, S_XY, S_XZ, wline(curve))
    # The line leaves the first domain at both ends (u1 < 0 and u1 > 4).
    outer = wavy(60, step=0.08, x0=-0.4)
    purge(requests, S_XY, S_XZ, wline(outer, vertices=[at(-0.4, 1.0), at(outer[-1][0], 60.0)]))
    purge(requests, S_XY, S_XZ, wline(outer))
    purge(requests, S_XY, S_XZ, wline(outer, vertices=[at(0.0, 6.0), at(outer[-1][0], 60.0)]))
    far = [[x, y, z, u1 + 10.0, v1, u2, v2] for x, y, z, u1, v1, u2, v2 in wavy(35)]
    purge(requests, S_XY, S_XZ, wline(far))
    purge(requests, S_CYL, S_XZ, wline(outer))
    purge(requests, S_C0, S_XZ, wline(curve))
    purge(requests, S_C2, S_XZ, wline(curve))
    # Uneven steps: the evening of the kept points.
    uneven = [[x, 0.1 * x * x, 0.0, x, 5.0 + 0.1 * x * x, x, 0.0]
              for x in [0.2 + 0.001 * i * i * (1 + i % 3) for i in range(40)]]
    purge(requests, S_XY, S_XZ, wline(uneven))
    # About 20 kept points: the bad distribution of a single Bezier.
    few = wavy(18, step=0.2) + straight(14, step=0.01, x0=3.8)
    purge(requests, S_XY, S_XZ, wline(few))
    purge(requests, S_XY, S_XZ, wline(long_line, vertices=[at(0.5, 50.0)]))
    purge(requests, S_XY, S_XZ, wline(long_line, vertices=[at(0.5, 0.0)]))
    purge(requests, S_XY, S_XZ, wline([]))
    purge(requests, S_XY, S_XZ, wline(curve), d1=domain(1, 2, 3, tol=1.0e-7))


def join(requests, lines, points=(), s1=S_CYL, s2=S_CYLX, tol=1.0e-7):
    body = [*s1, *s2, tol, len(lines), *[v for l in lines for v in l], len(points), *[v for p in points for v in p]]
    requests.append(line_of('ip_join', body))


def arc_rows(n, a0, a1, v=0.5):
    """Points on the first cylinder at the angles `a0..a1`, height `v`."""
    rows = []
    for i in range(n):
        a = a0 + (a1 - a0) * i / (n - 1)
        rows.append([math.cos(a), math.sin(a), v, a, v, 0.5 * a, 0.25])
    return rows


def _join_requests(requests: list[str]) -> None:
    first = arc_rows(6, 0.3, 1.0)
    second = arc_rows(6, 1.0, 1.8)
    rev = list(reversed(second))
    join(requests, [])
    join(requests, [wline(first), wline(second)])
    join(requests, [wline(second), wline(first)])
    join(requests, [wline(first), wline(rev)])
    join(requests, [wline(list(reversed(first))), wline(second)])
    join(requests, [wline(list(reversed(first))), wline(rev)])
    join(requests, [wline(first), wline(second), wline(arc_rows(5, 1.0, 1.5, v=0.9))])
    ends = [at(first[0][0], 1.0), at(first[-1][0], 6.0)]
    join(requests, [wline(first, vertices=ends), wline(second, vertices=ends)],
         points=[point(p_value(*first[-1][:3]), p_params(*first[-1][3:])),
                 point(p_value(*first[0][:3]), p_params(*first[0][3:])),
                 point(p_value(5.0, 5.0, 5.0), p_params(0.1, 0.2, 0.3, 0.4))])
    seam1, seam2 = arc_rows(6, TWO_PI - 0.8, TWO_PI), arc_rows(6, 0.0, 0.8)
    join(requests, [wline(seam1), wline(seam2)])
    join(requests, [wline(first), [1, *UNDECIDED, 0, 0, 0, 0, 0], wline(second)])
    sharp = arc_rows(6, 1.0, 1.1)
    sharp = [[r[0], r[1], r[2] + 3.0 * i, *r[3:]] for i, r in enumerate(sharp)]
    join(requests, [wline(first), wline(sharp)])
    join(requests, [wline(first), wline(second)], s2=S_CYL)
    join(requests, [wline(first), wline(second)], s1=S_XY)
    join(requests, [wline(first[:2]), wline(second)])


def lc(requests, line, refs=(), s1=S_XY, s2=S_XZ, d1=domain(1, 2, 3), d2=domain(), tol=1.0e-7):
    body = [*s1, *s2, *d1, *d2, tol, len(refs), *[v for l in refs for v in l], *line]
    requests.append(line_of('ip_lc', body))


def on_arc_at(x, para, rank, prm, first=1, v1=5.0):
    return point(p_value(x, 0.0, 0.0), p_params(x, v1, x, 0.0), p_para(para), p_arc(first, rank, prm))


def along_u(n, u, v0, v1):
    """Points of the first plane on the line `u` (the fixture arc of rank `u`)."""
    return [[u, v0 + (v1 - v0) * i / (n - 1), 0.0, u, v0 + (v1 - v0) * i / (n - 1), u, 0.0] for i in range(n)]


def _lc_walking(requests: list[str]) -> None:
    rows = straight(12, step=0.3, x0=0.2)
    mid = [at(0.2, 1.0), at(1.4, 5.0), at(rows[-1][0], 12.0)]
    lc(requests, wline(rows, vertices=mid))
    lc(requests, wline(rows, vertices=mid, trans=TYPES))
    lc(requests, wline(rows, vertices=mid, trans=TOUCH))
    lc(requests, wline(rows, vertices=[at(0.2, 1.0), at(0.2, 1.0), at(3.5, 12.0)]))
    lc(requests, wline(rows, vertices=[at(0.2, 1.0), at(0.5, 2.0), at(0.8, 3.0)]))
    lc(requests, wline(rows, vertices=[at(0.2, 1.0, tol=5.0), at(3.5, 12.0, tol=5.0)]))
    outside = [[x, 0.0, 0.0, x + 5.0, 5.0, x, 0.0] for x in [0.2 + 0.3 * i for i in range(12)]]
    lc(requests, wline(outside, vertices=[at(0.2, 1.0, u1=5.2), at(3.5, 12.0, u1=8.5)]))
    both = point(p_value(0.2, 0.0, 0.0), p_params(0.2, 5.0, 0.2, 0.0), p_para(3.0), p_arc(1, 1, 0.5), p_arc(0, 2, 0.5))
    lc(requests, wline(rows, vertices=[at(0.2, 1.0), both]))
    # A walking line on the arc u = 1: coded as a restriction line.
    on1 = along_u(10, 1.0, 1.0, 6.0)
    ends1 = [on_arc_at(1.0, 1.0, 1, 1.0), on_arc_at(1.0, 10.0, 1, 6.0)]
    for arcs in ((1, 0), (0, 0), (1, 1)):
        lc(requests, wline(on1, vertices=ends1, arcs=arcs))
    lc(requests, wline(on1, vertices=list(reversed(ends1)), arcs=(1, 0)))
    lc(requests, wline(on1, vertices=[on_arc_at(1.0, 1.0, 1, 6.0), on_arc_at(1.0, 10.0, 1, 1.0)], arcs=(1, 0)))
    free_ends = [free_point(1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0), free_point(1.0, 6.0, 0.0, 1.0, 6.0, 1.0, 0.0, 10.0)]
    lc(requests, wline(on1, vertices=free_ends, arcs=(1, 0)))
    lc(requests, wline(on1, vertices=free_ends, arcs=(1, 0)), d1=domain(2, 1, tol=1.0e-3))
    two = along_u(2, 1.0, 1.0, 6.0)
    lc(requests, wline(two, vertices=[free_point(1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0),
                                      free_point(1.0, 6.0, 0.0, 1.0, 6.0, 1.0, 0.0, 2.0)], arcs=(1, 0)))
    ref = [1, *UNDECIDED, 0, *vtx(*free_ends), 1, 0]
    lc(requests, wline(on1, vertices=free_ends), refs=[ref])
    lc(requests, wline(on1, vertices=free_ends), refs=[ref], d1=domain(1, tol=1.0e-7))
    lc(requests, wline(rows, vertices=mid), s1=S_CYL, s2=S_XZ)


def _lc_restriction(requests: list[str]) -> None:
    def rline(vertices, arcs, points=None, trans=UNDECIDED):
        body = [1, *trans, *([1, *pts(points)] if points else [0]), *vtx(*vertices), *arcs]
        return body
    vs = [free_point(1.0, 0.5 * k, 0.0, 1.0, 0.5 * k, 0.5 * k, 0.1, 0.5 * k) for k in (1, 4, 9, 12)]
    for arcs in ((1, 1), (1, 0), (0, 1)):
        lc(requests, rline(vs, arcs))
        lc(requests, rline(vs, arcs, trans=TYPES), d2=domain(3, tol=1.0e-6))
    lc(requests, rline(vs, (1, 0), points=along_u(4, 1.0, 0.5, 6.0)))
    lc(requests, rline([vs[0], vs[0]], (1, 0)))
    far = [free_point(1.0, 0.5 * k, 0.0, 1.0, 0.5 * k, 9.0, 9.0, 0.5 * k) for k in (1, 4)]
    lc(requests, rline(far, (1, 0)))
    lc(requests, rline(far, (0, 1)))
    lc(requests, rline(vs, (0, 1), trans=TOUCH), s1=S_CYL)


def gline(ck, frame, radii, vertices, trans=UNDECIDED):
    return [2, ck, *trans, *frame, *radii, *vtx(*vertices)]


def _lc_conics(requests: list[str]) -> None:
    lin = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]
    lv = [free_point(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, p) for p in (-1.0, 1.5, 3.0, 7.0)]
    p_inf1 = sad(plane([0.0, 0.0, 0.0], [0.0, 0.0, 1.0]))
    p_inf2 = sad(plane([0.0, 0.0, 0.0], [0.0, 1.0, 0.0]))
    lc(requests, gline(0, lin, (0.0, 0.0), lv), s1=p_inf1, s2=p_inf2, d1=domain(), d2=domain())
    lc(requests, gline(0, lin, (0.0, 0.0), lv, trans=TYPES), d1=domain(1, 2, tol=1.0e-6), d2=domain(tol=1.0e-6))
    lc(requests, gline(0, lin, (0.0, 0.0), lv))
    lc(requests, gline(0, lin, (0.0, 0.0), lv), d1=OWN, d2=OWN)
    circ = [0.0, 0.0, 0.5, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0]
    cv = [free_point(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, p) for p in (0.5, 2.0, 4.0, 5.5)]
    sph = sad(sphere([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1.2))
    for ck, radii in ((1, (1.0, 0.0)), (2, (1.5, 0.8))):
        lc(requests, gline(ck, circ, radii, cv), s1=S_CYL, s2=sph, d1=domain(1, tol=1.0e-7), d2=domain(tol=1.0e-7))
        lc(requests, gline(ck, circ, radii, cv, trans=TOUCH), s1=S_CYL, s2=sph, d1=domain(tol=1.0e-7), d2=domain())
        lc(requests, gline(ck, circ, radii, cv[:1]), s1=S_CYL, s2=sph, d1=domain(tol=1.0e-7), d2=domain())
        lc(requests, gline(ck, circ, radii, [cv[3], cv[0]]), s1=S_CYL, s2=sph, d1=domain(tol=0.1), d2=domain(tol=0.1))
    both = point(p_value(0.0, 0.0, 0.0), p_params(0.0, 0.0, 0.0, 0.0), p_para(1.0), p_arc(1, 1, 0.5), p_arc(0, 2, 0.5))
    lc(requests, gline(1, circ, (1.0, 0.0), [both]), s1=S_CYL, s2=sph, d1=domain(tol=1.0e-7), d2=domain())
    con = sad(cone([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 0.5, 1.0), [0.0, TWO_PI, -3.0, 3.0])
    for ck, radii in ((3, (0.5, 0.0)), (4, (2.0, 1.0))):
        lc(requests, gline(ck, circ, radii, cv), s1=con, s2=p_inf2, d1=domain(tol=1.0e-7), d2=domain())
    lc(requests, gline(0, lin, (0.0, 0.0), lv), s1=S_C2, s2=p_inf2, d1=domain(), d2=domain())


def _lc_analytic(requests: list[str]) -> None:
    cylinder, sph = sad(cyl([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1.0), [0.0, TWO_PI, -3.0, 3.0]), sad(sphere([0.5, 0.0, 0.0], [0.0, 0.0, 1.0], 1.2))
    curve = [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.5, 0.0, 0.0, 1.2, 1.0e-8]
    av = [free_point(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, p) for p in (4.7, 5.5, 6.5, 7.9)]
    aline = lambda vertices, trans=UNDECIDED: [3, *curve, *trans, *vtx(*vertices)]
    lc(requests, aline(av), s1=cylinder, s2=sph, d1=domain(), d2=domain())
    lc(requests, aline(av, trans=TYPES), s1=cylinder, s2=sph, d1=domain(1, 2), d2=domain())
    both = point(p_value(0.0, 0.0, 0.0), p_para(7.0), p_arc(1, 1, 0.5), p_arc(0, 2, 0.5))
    lc(requests, aline([*av[:2], both]), s1=cylinder, s2=sph, d1=domain(), d2=domain())
    lc(requests, aline([av[0], av[0]]), s1=cylinder, s2=sph, d1=domain(), d2=domain())
    lc(requests, aline(av), s1=S_C2, s2=sph, d1=domain(), d2=domain())


def append_intpatch_wltool_requests(requests: list[str]) -> None:
    """The WLineTool and LineConstructor requests."""
    _purge_requests(requests)
    _join_requests(requests)
    _lc_walking(requests)
    _lc_restriction(requests)
    _lc_conics(requests)
    _lc_analytic(requests)
