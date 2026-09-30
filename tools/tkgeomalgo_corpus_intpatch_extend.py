"""Deterministic requests for IntPatch_WLineTool::ExtendTwoWLines
(tools/oracle/tkgeomalgo_probe_intpatch_extend.cxx).

`ip_extend` extends walking lines on two quadrics given by their 3d points: pieces of
the line of two planes with gaps in the four end orders and with shared ends, arcs of
the circle of a sphere and a plane (with a gap across the seam and at a pole), of the
ellipse of a cylinder and a tilted plane, of the circles of a cone near its apex, lines
whose tangents turn too much, critical points at a gap, and three pieces in a row.
"""
from __future__ import annotations
import math
from tkgeomalgo_corpus_intcurvesurface import cone, cyl, line_of, plane, sad, sphere

TWO_PI = 2.0 * math.pi
XY = sad(plane([0.0, 0.0, 0.0], [0.0, 0.0, 1.0]))
TILT = sad(plane([0.0, 0.0, 0.0], [0.0, 0.6, 0.8]))
HALF = sad(plane([0.0, 0.0, 0.5], [0.0, 0.0, 1.0]))
BALL = sad(sphere([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1.0))
TUBE = sad(cyl([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1.0), [0.0, TWO_PI, -5.0, 5.0])
SLANT = sad(plane([0.0, 0.0, 0.3], [0.2, 0.1, 1.0]))
CONE = sad(cone([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 0.5, 1.0), [0.0, TWO_PI, -3.0, 3.0])
LEVEL = sad(plane([0.0, 0.0, 0.4], [0.0, 0.0, 1.0]))


def seg(x0, x1, n=6):
    return [[x0 + (x1 - x0) * i / (n - 1), 0.0, 0.0] for i in range(n)]


def wl(points, shift1=0.0, shift2=0.0):
    return [len(points), *[c for p in points for c in p], shift1, shift2]


def extend(requests, s1, s2, lines, crit=(), tol=1.0e-7):
    body = [*s1, *s2, tol, len(crit), *[c for p in crit for c in p], len(lines), *[v for l in lines for v in l]]
    requests.append(line_of('ip_extend', body))


def circle(a0, a1, n=8, rc=math.sqrt(0.75), z=0.5):
    return [[rc * math.cos(a0 + (a1 - a0) * i / (n - 1)), rc * math.sin(a0 + (a1 - a0) * i / (n - 1)), z]
            for i in range(n)]


def ellipse(a0, a1, n=8):
    pts = []
    for i in range(n):
        a = a0 + (a1 - a0) * i / (n - 1)
        x, y = math.cos(a), math.sin(a)
        pts.append([x, y, 0.3 - 0.2 * x - 0.1 * y])
    return pts


def _plane_requests(requests: list[str]) -> None:
    a, b = seg(0.0, 1.0), seg(1.1, 2.0)
    extend(requests, XY, TILT, [wl(a), wl(b)])
    extend(requests, XY, TILT, [wl(b), wl(a)])
    extend(requests, XY, TILT, [wl(a), wl(list(reversed(b)))])
    extend(requests, XY, TILT, [wl(list(reversed(a))), wl(b)])
    extend(requests, XY, TILT, [wl(list(reversed(a))), wl(list(reversed(b)))])
    extend(requests, XY, TILT, [wl(a), wl(seg(1.0, 2.0))])
    extend(requests, XY, TILT, [wl(a), wl(seg(1.0 + 5.0e-8, 2.0))])
    extend(requests, XY, TILT, [wl(a), wl(b), wl(seg(2.2, 3.0))])
    extend(requests, XY, TILT, [wl(a), wl(b)], crit=[[1.05, 0.0, 0.0]])
    extend(requests, XY, TILT, [wl(a), wl(b)], crit=[[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]])
    bent = [[1.1 + 0.2 * i, 0.3 * i * i, 0.0] for i in range(6)]
    extend(requests, XY, TILT, [wl(a), wl(bent)])
    extend(requests, XY, TILT, [wl(a), wl(b)], tol=0.2)
    extend(requests, XY, TILT, [wl(a)])
    extend(requests, XY, TILT, [wl(a), [1, 1.5, 0.0, 0.0, 0.0, 0.0]])


def _curved_requests(requests: list[str]) -> None:
    extend(requests, BALL, HALF, [wl(circle(0.3, 2.0)), wl(circle(2.2, 6.0))])
    extend(requests, HALF, BALL, [wl(circle(0.3, 2.0)), wl(circle(2.2, 6.0))])
    extend(requests, BALL, HALF, [wl(circle(5.0, TWO_PI - 0.1)), wl(circle(0.1, 1.0))])
    extend(requests, BALL, HALF, [wl(circle(5.0, TWO_PI - 0.1)), wl(circle(0.1, 1.0), shift1=TWO_PI)])
    extend(requests, BALL, HALF, [wl(circle(0.3, 3.0)), wl(list(reversed(circle(3.1, 6.0))))])
    extend(requests, TUBE, SLANT, [wl(ellipse(0.2, 2.5)), wl(ellipse(2.6, 5.9))])
    extend(requests, SLANT, TUBE, [wl(list(reversed(ellipse(0.2, 2.5)))), wl(ellipse(2.6, 5.9))])
    extend(requests, TUBE, SLANT, [wl(ellipse(5.5, TWO_PI - 0.05)), wl(ellipse(0.05, 1.0))])
    r = 1.0 + 0.4 * math.tan(0.5)
    extend(requests, CONE, LEVEL, [wl(circle(0.4, 2.8, rc=r, z=0.4)), wl(circle(2.9, 5.0, rc=r, z=0.4))])
    extend(requests, CONE, LEVEL, [wl(circle(0.4, 2.8, rc=r, z=0.4)), wl(circle(2.9, 5.0, rc=r, z=0.4))],
           crit=[[r * math.cos(2.85), r * math.sin(2.85), 0.4]])


def append_intpatch_extend_requests(requests: list[str]) -> None:
    """The ExtendTwoWLines requests."""
    _plane_requests(requests)
    _curved_requests(requests)
