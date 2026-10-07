"""Deterministic requests for the ChFi2d probes (tools/oracle/tkfillet_probe_chfi2d.cxx).

`fl_chfi2d_fillet` fillets two edges of the plane z = 0 (segments and arcs of circle,
both orders of their ends, every angle, short and intersecting edges) by the fillet API,
the analytical algorithm and the iterative one. `fl_chfi2d_chamfer` chamfers two
segments. `fl_chfi2d_builder` edits polygon faces with fillets and chamfers, modified and
removed, including the refused edits.
"""
from __future__ import annotations
import math
import random

from tkfillet_corpus_blendfunc import numbers


def seg(p1, p2):
    """A segment edge spec (10 numbers)."""
    return [0, *p1, *p1, *p2]


def arc(p1, pm, p2):
    """An arc of circle through three points (10 numbers)."""
    return [1, *p1, *pm, *p2]


def pt(x, y):
    return (float(x), float(y), 0.0)


def fillet(mode, e1, e2, radius, p=(0.0, 0.0, 0.0), isol=-1) -> str:
    return 'fl_chfi2d_fillet ' + numbers([mode, *e1, *e2, radius, *p, isol])


def corner(angle, length=10.0, reverse1=False, reverse2=False):
    """Two segments from the origin: along X and at `angle`, the common point at the
    origin (the end of the first one and the start of the second one unless reversed)."""
    a = pt(length, 0)
    b = pt(length * math.cos(angle), length * math.sin(angle))
    o = pt(0, 0)
    e1 = seg(o, a) if reverse1 else seg(a, o)
    e2 = seg(b, o) if reverse2 else seg(o, b)
    return e1, e2


def segment_requests(requests: list[str]) -> None:
    """Two segments: every angle, both orientations, every mode, several radii."""
    for angle in (math.pi / 2, math.pi / 3, 2.5, 0.4, 4.0, 5.5):
        for rev in ((False, False), (True, False), (False, True), (True, True)):
            e1, e2 = corner(angle, reverse1=rev[0], reverse2=rev[1])
            for mode in (0, 1, 2, 3):
                requests.append(fillet(mode, e1, e2, 1.0, (0.0, 0.0, 0.0)))
    e1, e2 = corner(math.pi / 2)
    for radius in (0.0, 1e-8, 0.5, 9.99999, 9.999999999, 10.0, 12.0):
        for mode in (0, 2, 3):
            requests.append(fillet(mode, e1, e2, radius))
    for length in (0.5, 1.0, 1.00001, 2.0):
        e1, e2 = corner(math.pi / 2, length)
        requests.append(fillet(0, e1, e2, 1.0))
        requests.append(fillet(3, e1, e2, 1.0))
    # parallel and opposite segments
    requests.append(fillet(0, seg(pt(-10, 0), pt(0, 0)), seg(pt(0, 0), pt(10, 0)), 1.0))
    requests.append(fillet(2, seg(pt(-10, 0), pt(0, 0)), seg(pt(0, 0), pt(-10, 1e-9)), 1.0))
    # no common point
    requests.append(fillet(1, seg(pt(0, 0), pt(10, 0)), seg(pt(0, 1), pt(0, 10)), 1.0))
    requests.append(fillet(2, seg(pt(0, 0), pt(10, 0)), seg(pt(0, 1), pt(0, 10)), 1.0))


def arc_requests(requests: list[str]) -> None:
    """A segment and an arc in both orders and both directions, two arcs, intersecting
    arcs (cut first)."""
    s = seg(pt(0, 0), pt(10, 0))
    for pm, p2 in ((pt(15, 5), pt(10, 10)), (pt(5, -5), pt(10, -10)), (pt(14, 4), pt(10, 8)), (pt(12, 3), pt(14, 8))):
        a = arc(pt(10, 0), pm, p2)
        ra = arc(p2, pm, pt(10, 0))
        for mode in (0, 1, 2, 3):
            requests.append(fillet(mode, s, a, 1.5, pt(10, 0)))
            requests.append(fillet(mode, ra, seg(pt(10, 0), pt(0, 0)), 1.5, pt(10, 0)))
    a1 = arc(pt(0, 0), pt(5, 2), pt(10, 0))
    for pm, p2 in ((pt(15, 2), pt(20, 0)), (pt(14, -2), pt(18, -6)), (pt(12, 5), pt(10, 10))):
        for mode in (0, 2, 3):
            requests.append(fillet(mode, a1, arc(pt(10, 0), pm, p2), 1.0, pt(10, 0)))
    # intersecting arcs: the second one crosses the first one
    a2 = arc(pt(10, 0), pt(6, 3), pt(2, 0))
    for mode in (0, 2):
        requests.append(fillet(mode, a1, a2, 0.5, pt(10, 0)))
    for isol in (1, 2, 3):
        requests.append(fillet(3, s, arc(pt(10, 0), pt(15, 5), pt(10, 10)), 1.5, pt(10, 0), isol))


def chamfer_requests(requests: list[str]) -> None:
    """Two segments: every orientation, lengths up to the edge lengths."""
    for angle in (math.pi / 2, 2.0, 0.5):
        for rev in ((False, False), (True, False), (False, True), (True, True)):
            e1, e2 = corner(angle, reverse1=rev[0], reverse2=rev[1])
            for mode in (0, 1):
                requests.append('fl_chfi2d_chamfer ' + numbers([mode, *e1, *e2, 1.0, 2.0]))
    e1, e2 = corner(math.pi / 2)
    for l1, l2 in ((10.0, 10.0), (10.5, 1.0), (1.0, 11.0), (0.0, 0.0)):
        requests.append('fl_chfi2d_chamfer ' + numbers([0, *e1, *e2, l1, l2]))
    requests.append('fl_chfi2d_chamfer ' + numbers([0, *seg(pt(0, 0), pt(10, 0)), *seg(pt(3, 3), pt(5, 9)), 1.0, 1.0]))


def builder(points, ops) -> str:
    fields = [len(points)]
    for p in points:
        fields += list(p)
    fields.append(len(ops))
    for op in ops:
        fields += list(op)
    return 'fl_chfi2d_builder ' + numbers(fields)


SQUARE = [pt(0, 0), pt(10, 0), pt(10, 10), pt(0, 10)]
TRIANGLE = [pt(0, 0), pt(10, 0), pt(3, 8)]
L_SHAPE = [pt(0, 0), pt(10, 0), pt(10, 4), pt(4, 4), pt(4, 10), pt(0, 10)]


def builder_requests(requests: list[str]) -> None:
    """Polygon faces edited by sequences of fillets and chamfers."""
    for points in (SQUARE, TRIANGLE, L_SHAPE):
        for v in range(1, len(points) + 1):
            requests.append(builder(points, [(1, v, 0, 1.0, 0.0)]))
            requests.append(builder(points, [(1, v, 0, 1.0, 0.0), (4, 1, 0, 2.0, 0.0), (5, 1, 0, 0.0, 0.0)]))
        n = len(points)
        for e in range(1, n + 1):
            f = e % n + 1
            requests.append(builder(points, [(2, e, f, 1.0, 2.0), (8, 1, f, 1.5, 0.5), (7, 1, 0, 0.0, 0.0)]))
            requests.append(builder(points, [(3, e, 1, 1.0, 0.6), (6, 1, e, 2.0, 0.3), (7, 1, 0, 0.0, 0.0)]))
    requests.append(builder(SQUARE, [(1, 1, 0, 1.0, 0.0), (1, 2, 0, 1.0, 0.0), (1, 3, 0, 4.0, 0.0), (1, 4, 0, 5.0, 0.0)]))
    requests.append(builder(SQUARE, [(1, 1, 0, 5.0, 0.0), (1, 2, 0, 5.0, 0.0), (5, 1, 0, 0.0, 0.0), (5, 1, 0, 0.0, 0.0)]))
    requests.append(builder(SQUARE, [(1, 1, 0, 1.0, 0.0), (1, 1, 0, 1.0, 0.0), (2, 1, 2, 1.0, 1.0)]))
    requests.append(builder(SQUARE, [(2, 1, 2, -1.0, 1.0), (3, 1, 1, 1.0, 0.0), (1, 1, 0, 20.0, 0.0)]))
    requests.append(builder(SQUARE, [(2, 1, 3, 1.0, 1.0), (1, 1, 0, 10.0, 0.0), (2, 2, 3, 10.0, 10.0)]))
    requests.append(builder(L_SHAPE, [(1, 4, 0, 2.0, 0.0), (2, 1, 2, 1.0, 1.0), (1, 2, 0, 1.0, 0.0), (5, 1, 0, 0.0, 0.0)]))
    rng = random.Random(20261011)
    for _ in range(16):
        ops = []
        for _ in range(rng.randint(1, 4)):
            op = rng.choice((1, 1, 2, 3, 4, 5, 7))
            ops.append((op, rng.randint(1, 4), rng.randint(1, 4), rng.uniform(0.2, 3.0), rng.uniform(0.2, 2.0)))
        requests.append(builder(SQUARE if rng.random() < 0.5 else L_SHAPE, ops))


def append_chfi2d_requests(requests: list[str]) -> None:
    """Append the ChFi2d request families."""
    segment_requests(requests)
    arc_requests(requests)
    chamfer_requests(requests)
    builder_requests(requests)
