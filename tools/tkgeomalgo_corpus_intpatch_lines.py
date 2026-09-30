"""Deterministic requests for the IntPatch lines (tools/oracle/tkgeomalgo_probe_intpatch_lines.cxx).

`ip_wline` builds walking lines (straight and periodic polylines, confused points, empty
lines), adds vertices on no domain, on the arcs of either surface and on the vertices of
the domains, and runs ComputeVertexParameters: vertices before, on, between and after the
points, the seam duplication of close vertices on different arcs, the filters of the
restrictions and the ImpImp creation way. Every vertex edit runs on valid and
out-of-range indices, and the lines without arcs are dumped in every mode. `ip_rline`,
`ip_gline` and `ip_aline` do the same for the restriction, conic and analytic lines.
"""
from __future__ import annotations
import math
from tkgeomalgo_corpus_intcurvesurface import line_of
from tkgeomalgo_corpus_intpatch import (
    free_point, p_arc, p_para, p_params, p_tol, p_value, p_vertex, point)

TWO_PI = 2.0 * math.pi
UNDECIDED = [0, 0]
TYPES = [1, 1, 0, 1]
TOUCH = [2, 0, 0, 2]


def polyline(n, step=0.1, period_shift=0.0):
    """`n` points on the x axis at `x = i step`, with the parameters `(x, 0, x + shift, 1)`."""
    return [n, *[v for i in range(n) for v in (i * step, 0.0, 0.0, i * step, 0.0, i * step + period_shift, 1.0)]]


def vertices(*points):
    return [len(points), *[v for p in points for v in p]]


def commands(*cmds):
    return [len(cmds), *[v for c in cmds for v in c]]


def on_arc(x, para, first=1, rank=1, prm=0.5, tl=1, ta=2, tol=1.0e-7):
    """A vertex at `(x, 0, 0)` on the arc `rank` of one domain."""
    return point(p_value(x, 0.0, 0.0, tol), p_params(x, 0.0, x, 1.0), p_para(para), p_arc(first, rank, prm, tl, ta))


def on_both(x, para, r1, r2, prm1=0.5, prm2=0.5, tol=1.0e-7):
    return point(p_value(x, 0.0, 0.0, tol), p_params(x, 0.0, x, 1.0), p_para(para),
                 p_arc(1, r1, prm1), p_arc(0, r2, prm2))


def free(x, para, tol=1.0e-7):
    return free_point(x, 0.0, 0.0, x, 0.0, x, 1.0, para, tol)


def wline(requests, pts, vtx, cmds, trans=UNDECIDED, way=0, periods=(0.0, 0.0, 0.0, 0.0), dump=-1):
    requests.append(line_of('ip_wline', [dump, *trans, way, *periods, *pts, *vtx, *cmds]))


def _wline_computes(requests: list[str]) -> None:
    pts = polyline(6)
    compute = commands([6, 1.0e-7])
    cases = [
        vertices(),
        vertices(free(0.0, 1.0), free(0.5, 6.0)),
        vertices(free(-0.2, 0.5), free(0.25, 3.5), free(0.8, 8.0)),
        vertices(on_arc(0.2, 3.0), on_arc(0.2, 3.0, rank=1), on_arc(0.3, 4.0, first=0)),
        vertices(on_arc(0.1, 2.0), on_arc(0.1 + 1.0e-9, 2.5, rank=2)),
        vertices(on_arc(0.1, 2.0, first=0), on_arc(0.1, 3.0, first=0, rank=2), free(0.1, 2.0)),
        vertices(on_both(0.2, 3.0, 1, 2), on_arc(0.2, 3.0, rank=1), on_arc(0.2, 3.0, first=0, rank=2)),
        vertices(on_arc(0.1, 2.0, tl=1), on_arc(0.2, 3.0, tl=1), on_arc(0.3, 4.0, tl=1)),
        vertices(on_arc(0.1, 2.0, first=0, tl=2), on_arc(0.2, 3.0, first=0, tl=2), free(0.5, 6.0)),
        vertices(free(0.3, 4.0, 0.5), free(0.31, 4.2)),
        vertices(point(p_value(0.2, 0.0, 0.0), p_params(0.2, 0.0, 0.2, 1.0), p_para(3.0), p_vertex(1, 1),
                       p_arc(1, 1, 0.1)), free(0.2, 3.0)),
    ]
    for vtx in cases:
        wline(requests, pts, vtx, compute)
        wline(requests, pts, vtx, compute, way=1)
    dup = [4, *[v for i in range(4) for v in (0.1 * (i // 2), 0.0, 0.0, 0.1 * i, 0.0, 0.1 * i, 1.0)]]
    wline(requests, dup, vertices(free(0.1, 3.0), free(0.0, 1.0)), compute)
    wline(requests, polyline(0), vertices(free(0.0, 1.0)), compute)
    wline(requests, polyline(2), vertices(), compute)
    wline(requests, polyline(1), vertices(free(0.0, 1.0)), compute)
    # Periodic parameters brought back near their reference.
    periodic = polyline(5, 0.5, TWO_PI)
    far = point(p_value(1.0, 0.0, 0.0), p_params(1.0 + 2 * TWO_PI, 0.0, 1.0, 1.0 - TWO_PI), p_para(2.5))
    wline(requests, periodic, vertices(far), compute, periods=(TWO_PI, TWO_PI, TWO_PI, TWO_PI))
    wline(requests, periodic, vertices(far), compute, periods=(TWO_PI, 0.0, 0.0, TWO_PI), way=1)


def _wline_edits(requests: list[str]) -> None:
    pts = polyline(4)
    vtx = vertices(free(0.0, 1.0), free(0.3, 4.0))
    edits = [
        [[1, 0, *free(0.1, 2.0)], [1, 1, *free(0.2, 3.0)]],
        [[2, 2, *free(9.0, 1.0)], [2, 0, *free(9.0, 1.0)], [2, 5, *free(9.0, 1.0)]],
        [[3, 1, *free(5.0, 7.0)], [3, 0, *free(5.0, 7.0)], [3, 3, *free(5.0, 7.0)]],
        [[4, 2], [5, 1]], [[4, 5], [5, 0]],
        [[7, 1], [7, 0], [7, 2]],
        [[8, 1, *free(1.0, 0.5)], [8, 0, *free(1.0, 0.5)], [8, 2, *free(1.0, 0.5)], [8, 9, *free(1.0, 0.5)]],
        [[9], [8, -1, *free(1.0, 0.5)], [8, 0, *free(1.0, 0.5)]],
        [[10, 1], [11, 0], [10, 3], [13, 0]],
        [[12, 1, 0, 1, 0], [14, 1, 8.0], [14, 3, 8.0]],
        [[6, 1.0e-7], [7, 1], [6, 1.0e-7]],
    ]
    for cmds in edits:
        wline(requests, pts, vtx, commands(*cmds))
    for trans in (UNDECIDED, TYPES, TOUCH, [1, 1, 3, 2], [2, 0, 1, 0]):
        wline(requests, pts, vtx, commands(), trans=trans, way=3)
    for mode in (0, 1, 2, 3, 7):
        wline(requests, pts, vtx, commands([4, 1], [5, 2]), dump=mode)
    wline(requests, polyline(0), vertices(), commands(), dump=0)


def rline(requests, trans, pts, vtx, cmds, dump=-1):
    body = [dump, *trans, *([1, *pts] if pts is not None else [0]), *vtx, *cmds]
    requests.append(line_of('ip_rline', body))


def _rline_requests(requests: list[str]) -> None:
    pts = polyline(4)
    compute = commands([6, 1.0e-7])
    cases = [
        vertices(),
        vertices(free(0.3, 4.0), free(0.0, 1.0), free(0.1, 2.0)),
        vertices(on_arc(0.1, 2.0, prm=0.5), on_arc(0.1, 2.0, prm=0.5), on_arc(0.1, 2.0, prm=0.6)),
        vertices(on_arc(0.1, 2.0, first=0), on_arc(0.1, 2.0, first=0), free(0.1, 2.0)),
        vertices(on_both(0.2, 3.0, 1, 2), on_arc(0.2, 3.0, rank=1), on_arc(0.2, 3.0, rank=2),
                 on_both(0.2, 3.0, 1, 2, prm2=0.7)),
        vertices(on_both(0.2, 3.0, 1, 2), on_arc(0.2, 3.0, first=0, rank=2), free(0.2, 3.0)),
    ]
    for vtx in cases:
        rline(requests, UNDECIDED, pts, vtx, compute)
        rline(requests, TOUCH, None, vtx, compute)
    rline(requests, TYPES, None, vertices(free(0.0, 1.0)), commands([10, 2], [11, 3], [7, 1], [7, 1]))
    rline(requests, TYPES, pts, vertices(free(0.0, 1.0), free(0.1, 5.0)),
          commands([10, 1], [2, 1, *free(7.0, 1.0)], [2, 9, *free(7.0, 1.0)], [15], [4, 1]))
    rline(requests, UNDECIDED, None, vertices(free(0.0, 1.0)),
          commands([13, *polyline(3)], [3, 1, *free(4.0, 2.0)], [3, 2, *free(4.0, 2.0)], [11, 1]))
    rline(requests, UNDECIDED, pts, vertices(free(0.0, 1.0)),
          commands([1, 1, *free(0.1, -1.0)], [9], [1, 0, *free(0.1, 3.0)], [14, 1, 2.5], [14, 2, 1.0],
                   [12, 0, 1, 0, 1]))
    for mode in (0, 1, 2, 3):
        rline(requests, UNDECIDED, pts, vertices(free(0.0, 1.0)), commands(), dump=mode)
    rline(requests, UNDECIDED, None, vertices(), commands(), dump=1)


def gline(requests, kind, frame, radii, vtx, cmds, trans=UNDECIDED):
    requests.append(line_of('ip_gline', [kind, *trans, *frame, *radii, *vtx, *cmds]))


FRAME = [0.1, 0.2, 0.3, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0]
TILTED = [0.0, 0.0, 0.0, 0.0, 0.6, 0.8, 1.0, 0.0, 0.0]


def seam(para, u1, u2, arc=None):
    """A point at parameter `para` whose parameters differ on one surface only by `u1`/`u2`."""
    cmds = [p_value(1.0, 0.0, 0.0), p_params(u1, 0.5, u2, 0.25), p_para(para)]
    if arc:
        cmds.append(p_arc(*arc))
    return point(*cmds)


def _gline_requests(requests: list[str]) -> None:
    for kind, radii in ((0, (0.0, 0.0)), (1, (1.5, 0.0)), (2, (2.0, 1.0)), (3, (0.5, 0.0)), (4, (2.0, 1.0))):
        for trans in (UNDECIDED, TYPES, TOUCH):
            gline(requests, kind, FRAME, radii, vertices(), commands(), trans)
        gline(requests, kind, TILTED, radii, vertices(free(0.0, 0.5), free(0.0, 2.0)),
              commands([4, 1], [5, 2], [1, *free(0.0, 1.0)], [1, *free(0.0, 7.0)], [1, *free(0.0, -9.0)],
                       [1, *free(0.0, 2.0 - 1.0e-7)], [3, 9, *free(0.0, 4.0)]))
    gline(requests, 1, FRAME, (-1.0, 0.0), vertices(), commands())
    gline(requests, 2, FRAME, (1.0, 2.0), vertices(), commands())
    circle = (1.0, 0.0)
    filters = [
        vertices(free(0.0, 1.0), on_arc(0.0, 1.0 + 1.0e-7), free(0.0, 3.0)),
        vertices(on_arc(0.0, 1.0, prm=0.5), on_arc(0.0, 2.0, prm=0.5 + 1.0e-7), on_arc(0.0, 3.0, prm=0.9)),
        vertices(point(p_value(0.0, 0.0, 0.0), p_para(1.0), p_vertex(1, 1), p_arc(1, 1, 0.5)), on_arc(0.0, 2.0)),
        vertices(on_arc(0.0, 1.0, first=0), on_arc(0.0, 2.0, first=0), point(p_para(3.0), p_vertex(1, 2),
                                                                            p_arc(0, 1, 0.5))),
        vertices(on_both(0.0, 1.0, 1, 2), on_arc(0.0, 1.0, rank=1)),
        vertices(on_both(0.0, 1.0, 1, 2), on_both(0.0, 1.0, 1, 2)),
        vertices(on_both(0.0, 1.0, 1, 2), point(p_para(1.0), p_vertex(0, 1), p_arc(1, 1, 0.5), p_arc(0, 2, 0.5))),
        vertices(on_arc(0.0, 1.0, first=0), on_both(0.0, 1.0, 3, 2)),
        vertices(on_arc(0.0, 1.0), free(0.0, 1.0)),
        vertices(on_arc(0.0, 1.0, first=0), free(0.0, 1.0), free(0.0, 2.0)),
        vertices(seam(TWO_PI, 0.0, 0.0, (1, 1, 0.5)), seam(0.0, TWO_PI, 0.0, (1, 2, 0.5))),
        vertices(seam(0.0, 0.0, 0.5), seam(0.0, TWO_PI, 0.5)),
        vertices(seam(0.0, 0.5, 0.0), seam(0.0, 0.5, TWO_PI)),
        vertices(seam(1.0, 0.0, 0.5), seam(1.0, TWO_PI, 0.5)),
        vertices(seam(TWO_PI, 0.0, 0.5), seam(TWO_PI, TWO_PI, 0.5), seam(3.0, 1.0, 1.0)),
        vertices(seam(0.0, 0.0, 0.5), seam(0.0, TWO_PI, 0.5), seam(TWO_PI, 1.0, 0.5), seam(TWO_PI, 1.5, 0.5)),
    ]
    for vtx in filters:
        for kind, radii in ((1, circle), (0, (0.0, 0.0))):
            gline(requests, kind, FRAME, radii, vtx, commands([6, 1.0e-7]))
            gline(requests, kind, FRAME, radii, vtx, commands([4, 1], [5, 2], [6, 1.0e-7]))
    gline(requests, 1, FRAME, circle, vertices(free(0.0, 0.0), free(0.0, TWO_PI)),
          commands([4, 2], [5, 1], [6, 1.0e-7]))
    gline(requests, 2, FRAME, (2.0, 1.0), vertices(free(0.0, 5.0)), commands([4, 3], [6, 1.0e-7]))


def aline(requests, cyl, sph, tol, vtx, cmds, params, trans=UNDECIDED):
    requests.append(line_of('ip_aline', [*cyl, *sph, tol, *trans, *vtx, *cmds, len(params), *params]))


CYL = [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0]


def _aline_requests(requests: list[str]) -> None:
    samples = [0.0, 0.5, 1.0, 2.0, 3.0, TWO_PI, 7.0, 13.0, -1.0]
    spheres = ([0.5, 0.0, 0.0, 1.2], [0.0, 0.0, 0.0, 1.5], [0.3, 0.2, 0.1, 1.3], [0.0, 0.0, 0.0, 0.5],
               [1.0, 0.0, 0.0, 2.0])
    for sph in spheres:
        aline(requests, CYL, sph, 1.0e-8, vertices(), commands(), samples)
    sph = spheres[0]
    cases = [
        vertices(free(0.0, 0.5), free(0.0, 1.0)),
        vertices(on_arc(0.0, 0.5), on_arc(0.0, 0.5 + 1.0e-7), on_arc(0.0, 2.0, first=0)),
        vertices(on_arc(0.0, 0.5, first=0, prm=0.2), on_arc(0.0, 0.5, first=0, prm=0.2)),
        vertices(free(0.0, 0.5), on_arc(0.0, 0.5), on_both(0.0, 1.0, 1, 2), on_arc(0.0, 1.0, rank=1)),
        vertices(on_arc(0.0, 1.0, first=0), free(0.0, 1.0), free(0.0, 2.0)),
        vertices(point(p_value(1.0, 0.0, 0.0), p_para(1.0), p_vertex(1, 1), p_arc(1, 1, 0.5)),
                 point(p_value(2.0, 0.0, 0.0), p_para(1.0 + 1.0e-6), p_arc(0, 2, 0.5)), free(0.0, 3.0)),
        vertices(point(p_value(0.0, 1.0, 0.0), p_para(1.0), p_vertex(0, 1), p_arc(0, 1, 0.5)),
                 point(p_value(0.0, 1.0, 0.0), p_para(1.0), p_arc(1, 2, 0.5), p_tol(1.0e-3))),
    ]
    for vtx in cases:
        aline(requests, CYL, sph, 1.0e-8, vtx, commands([6, 1.0e-7]), samples[:3])
        aline(requests, CYL, sph, 1.0e-8, vtx, commands([4, 1], [5, 2], [6, 1.0e-3]), samples[:1])
        aline(requests, CYL, spheres[1], 1.0e-8, vtx, commands([6, 1.0e-7]), [])
    aline(requests, CYL, sph, 1.0e-8, vertices(free(0.0, 0.5)),
          commands([1, *free(0.0, 2.0)], [3, 1, *free(0.0, 3.0)], [3, 5, *free(0.0, 3.0)], [4, 2], [5, 9],
                   [14, 1, 0.25], [14, 7, 0.25]), [0.1])
    aline(requests, [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5], [3.0, 0.0, 0.0, 0.5], 1.0e-8, vertices(free(0.0, 0.5)),
          commands([6, 1.0e-7]), [0.0])
    aline(requests, CYL, [0.0, 0.0, 0.0, -1.0], 1.0e-8, vertices(), commands(), [])


def append_intpatch_line_requests(requests: list[str]) -> None:
    """The line requests: walking, restriction, conic and analytic lines."""
    _wline_computes(requests)
    _wline_edits(requests)
    _rline_requests(requests)
    _gline_requests(requests)
    _aline_requests(requests)
