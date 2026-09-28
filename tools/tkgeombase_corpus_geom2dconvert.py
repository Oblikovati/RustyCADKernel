"""Deterministic requests for Geom2dConvert (tools/oracle/tkgeombase_probe_geom2dconvert.cxx).

Curves use the LProp builder specs (tkgeombase_probe_lprop_build.cxx). The requests
convert every Geom2d curve kind (conics on direct and indirect frames, full and trimmed
with each parameterisation, lines, Bezier and B-spline curves rational or not, periodic,
trimmed beyond their bounds, offset curves) and the degenerate inputs of each member:
equal and reversed split bounds, knot indices out of range, negative continuity
ranges, Bezier ranges that snap to a knot or are empty, curves that do not touch,
touch reversed or close a loop, concatenations of one curve, of curves with corners,
of G1 and C1 loops (with speed ratios that need the degree-doubling
reparametrization, and rational curves that need the Hermit law), too short
tolerance arrays and separated curves, C0 knots in open and closed curves, offset and
plain curves approximated at several orders, segment counts and degrees, and the
segment/biarc approximation of lines, circles, conics, S-curves, curves with straight
parts and closed curves at several tolerances.
"""
from __future__ import annotations
import math
from tkgeombase_corpus_gce import add

PI = math.pi
R2 = math.sqrt(0.5)

AX2D = [0.5, -1.0, 0.6, 0.8]
AX_X = [0.0, 0.0, 1.0, 0.0]


def circle(r, sense=1, ax=AX2D):
    return [1, *ax, sense, r]


def ellipse(a, b, sense=1, ax=AX2D):
    return [2, *ax, sense, a, b]


def hyperbola(a, b, sense=1):
    return [3, *AX2D, sense, a, b]


def parabola(f, sense=1):
    return [4, *AX2D, sense, f]


def line(p=(1.0, 2.0), d=(0.6, 0.8)):
    return [0, *p, *d]


def bezier(poles):
    """poles: (x, y, w) triples."""
    return [5, len(poles), *[c for p in poles for c in p]]


def bspline(degree, poles, knots, periodic=0):
    """poles: (x, y, w); knots: (u, mult)."""
    return [6, degree, periodic, len(poles), *[c for p in poles for c in p],
            len(knots), *[c for k in knots for c in k]]


def trimmed(u1, u2, basis):
    return [7, u1, u2, *basis]


def offset(d, basis):
    return [8, d, *basis]


def unit(points, w=1.0):
    return [(x, y, w) for x, y in points]


S_BEZIER = bezier(unit([(0, 0), (1, 3), (2, -2), (4, 0)]))
RAT_BEZIER = bezier([(0, 0, 1), (1, 2, 2), (3, 2, 0.5), (4, 0, 1)])
C2 = bspline(3, unit([(0, 0), (1, 2), (2, -1), (3, 1), (4, 0)]), [(0, 4), (1, 1), (2, 4)])
C0 = bspline(2, unit([(0, 0), (1, 1), (2, 0), (3, 1), (4, 0)]), [(0, 3), (1, 2), (3, 3)])
RAT_C1 = bspline(2, [(0, 0, 1), (1, 1, 2), (2, 0, 1), (3, -1, 0.5), (4, 0, 1)],
                 [(0, 3), (1, 1), (2, 1), (4, 3)])
HEXAGON = [(2.0 * math.cos(k * PI / 3), 2.0 * math.sin(k * PI / 3)) for k in range(6)]
PERIODIC = bspline(3, unit(HEXAGON), [(float(k), 1) for k in range(7)], 1)
LINEAR_PART = bspline(3, unit([(0, 0), (1, 0), (2, 0), (3, 0), (4, 1), (5, 3)]),
                      [(0, 4), (1, 1), (2, 1), (3, 4)])
SPLINES = [C2, C0, RAT_C1, PERIODIC, LINEAR_PART]


def to_bspline_requests(requests):
    conics = [circle(2.5), circle(2.5, 0), ellipse(4.0, 1.5), ellipse(4.0, 1.5, 0)]
    arcs = [(0.3, 1.3), (0.5, 3.5), (1.0, 7.1), (0.0, 2 * PI), (-1.0, 5.2), (2.0, 2.0 + 1e-9),
            (4.0, 1.0)]
    for p in range(8):
        for c in conics:
            add(requests, 'g2c_to_bspline', p, *c)
            for u1, u2 in arcs:
                add(requests, 'g2c_to_bspline', p, *trimmed(u1, u2, c))
    others = [trimmed(-2.0, 3.0, line()), trimmed(3.0, 3.0 + 1e-12, line()), line(),
              trimmed(-1.2, 0.9, hyperbola(3.0, 1.0)), trimmed(0.4, 1.9, hyperbola(3.0, 1.0, 0)),
              hyperbola(3.0, 1.0), trimmed(-3.0, 2.0, parabola(0.7)),
              trimmed(-3.0, 2.0, parabola(0.7, 0)), parabola(0.7),
              S_BEZIER, RAT_BEZIER, trimmed(0.2, 0.7, S_BEZIER), trimmed(-0.5, 1.5, RAT_BEZIER),
              *SPLINES, trimmed(0.5, 1.5, C2), trimmed(-1.0, 9.0, C0), trimmed(4.5, 7.5, PERIODIC),
              trimmed(0.1, 3.2, RAT_C1)]
    for p in (0, 6):
        for c in others:
            add(requests, 'g2c_to_bspline', p, *c)
    for c in [offset(0.5, ellipse(4.0, 1.5)), offset(-0.3, C2), offset(0.2, S_BEZIER),
              trimmed(0.2, 2.5, offset(0.5, ellipse(4.0, 1.5))), offset(0.1, trimmed(0.5, 1.5, C2)),
              offset(-2.5, circle(2.5))]:
        add(requests, 'g2c_to_bspline', 0, *c)


def split_requests(requests):
    for spec in SPLINES:
        for a, b in [(1, 3), (3, 1), (2, 2), (0, 2), (1, 9), (2, 3)]:
            for same in (1, 0):
                add(requests, 'g2c_split', 0, a, b, 1e-9, same, *spec)
        for a, b in [(0.2, 1.5), (1.5, 0.2), (-1.0, 5.0), (0.5, 0.5), (0.0, 2.0), (1.0, 3.5)]:
            for same in (1, 0):
                add(requests, 'g2c_split', 1, a, b, 1e-9, same, *spec)


def knot_split_requests(requests):
    extra = [bspline(3, unit([(0, 0), (1, 2), (2, 2), (3, 0), (4, 1), (5, 0), (6, 2)]),
                     [(0, 4), (1, 1), (2, 2), (3, 4)]),
             bspline(3, unit([(0, 0), (1, 2), (2, 2), (3, 0), (4, 1), (5, 0), (6, 2), (7, 1)]),
                     [(0, 4), (1, 3), (2, 1), (3, 4)])]
    for spec in SPLINES + extra:
        for rng in (-1, 0, 1, 2, 3, 5):
            add(requests, 'g2c_knot_split', rng, *spec)


def bezier_requests(requests):
    for spec in SPLINES:
        add(requests, 'g2c_bezier', 0, 0.0, 0.0, 0.0, *spec)
        for u1, u2, tol in [(0.2, 1.7, 1e-9), (0.0, 1.0, 1e-9), (1.0, 1.0, 1e-9),
                            (1.0 - 1e-12, 2.0 + 1e-12, 1e-9), (0.9, 2.1, 0.5), (1.5, 1.2, 0.0),
                            (0.3, 0.35, 0.1)]:
            add(requests, 'g2c_bezier', 1, u1, u2, tol, *spec)


def comp_item(tol, after, spec):
    return [tol, after, *spec]


def seg(p, q):
    return bezier(unit([p, q]))


def comp_requests(requests):
    arcs = [trimmed(k * 2 * PI / 5, (k + 1) * 2 * PI / 5, circle(3.0)) for k in range(5)]
    for p in (0, 6):
        for with_first in (1, 0):
            add(requests, 'g2c_comp', p, with_first, 5, *[v for a in arcs for v in comp_item(1e-7, 0, a)])
            rev = [arcs[0], arcs[2], arcs[1], arcs[4], arcs[3]]
            add(requests, 'g2c_comp', p, with_first, 5, *[v for a in rev for v in comp_item(1e-7, 1, a)])
    chains = [
        [seg((0, 0), (1, 0)), seg((1, 0), (1, 1)), seg((0, 1), (1, 1)), seg((0, 0), (0, 1))],
        [seg((0, 0), (1, 0)), seg((2, 0), (3, 0)), seg((1, 0), (2, 0))],
        [S_BEZIER, bezier(unit([(4, 0), (5, 1), (6, 0)])), seg((-1, 0), (0, 0)), RAT_BEZIER],
        [C2, seg((4, 0), (5, 5)), C0, bezier(unit([(4, 0), (4, 0 + 1e-9)]))],
        [seg((0, 0), (1, 0)), seg((1, 1e-5), (2, 0)), seg((2, 0), (0, 0))],
        [RAT_C1, trimmed(0.0, 1.0, C2), trimmed(1.0, 2.0, C2)],
    ]
    for chain in chains:
        for tol in (1e-7, 1e-4):
            for after in (0, 1):
                items = [v for c in chain for v in comp_item(tol, after, c)]
                add(requests, 'g2c_comp', 0, 1, len(chain), *items)
                add(requests, 'g2c_comp', 0, 0, len(chain), *items)


def quarter(k, r=1.0, w=1.0, c=(0.0, 0.0)):
    """The rational quadratic quarter circle from angle k PI/2, weights scaled by w."""
    a0, a1 = k * PI / 2, (k + 1) * PI / 2
    p0 = (c[0] + r * math.cos(a0), c[1] + r * math.sin(a0))
    p2 = (c[0] + r * math.cos(a1), c[1] + r * math.sin(a1))
    p1 = (c[0] + r * (math.cos(a0) - math.sin(a0)), c[1] + r * (math.sin(a0) + math.cos(a0)))
    return bspline(2, [(*p0, w), (*p1, w * R2), (*p2, w)], [(0.0, 3), (1.0, 3)])


def cubic(p0, p1, p2, p3, u0=0.0, u1=1.0):
    return bspline(3, unit([p0, p1, p2, p3]), [(u0, 4), (u1, 4)])


def concat_requests(requests):
    square = [bspline(1, unit([p, q]), [(0.0, 2), (1.0, 2)])
              for p, q in [((0, 0), (1, 0)), ((1, 0), (1, 1)), ((1, 1), (0, 1)), ((0, 1), (0, 0))]]
    smooth_open = [cubic((0, 0), (1, 1), (2, 1), (3, 0)), cubic((3, 0), (4, -1), (5, -1), (6, 0)),
                   cubic((6, 0), (8, 1), (9, 1), (10, 0), 0.0, 2.0)]
    corner = [cubic((0, 0), (1, 1), (2, 1), (3, 0)), cubic((3, 0), (4, 1), (5, 1), (6, 0)),
              cubic((6, 0), (7, -1), (8, -1), (9, 0))]
    circle4 = [quarter(k) for k in range(4)]
    circle4_scaled = [quarter(k, w=2.0 if k % 2 else 1.0) for k in range(4)]
    # The first quarter runs twice as slowly: the loop needs the degree-doubling law.
    circle4_speed = [bspline(2, [(1, 0, 1), (1, 1, R2), (0, 1, 1)], [(0.0, 3), (2.0, 3)])] + \
        [quarter(k) for k in range(1, 4)]
    loop = [cubic((0, 0), (1, 1), (2, 1), (3, 0)), cubic((3, 0), (4, -1), (1, -3), (0, -2), 0.0, 3.0),
            cubic((0, -2), (-1, -1), (-1, -1), (0, 0), 0.0, 0.5)]
    rational_open = [RAT_C1, bspline(2, [(4, 0, 2), (5, 1, 1), (6, 0, 2)], [(4.0, 3), (5.0, 3)])]
    apart = [cubic((0, 0), (1, 1), (2, 1), (3, 0)), cubic((3.1, 0), (4, -1), (5, -1), (6, 0))]
    arrays = [square, smooth_open, corner, circle4, circle4_scaled, circle4_speed, loop,
              rational_open, apart, [C2], [PERIODIC]]
    for curves in arrays:
        n = len(curves)
        for kind in (0, 1, 2):
            for closed in (0, 1):
                for tol in (1e-7, 1e-3):
                    tols = [tol] * (n - 1)
                    add(requests, 'g2c_concat', kind, closed, tol, 1e-9 if kind == 2 else 0.0,
                        len(tols), *tols, n, *[v for c in curves for v in c])
        add(requests, 'g2c_concat', 1, 1, 1e-7, 0.0, 0, n, *[v for c in curves for v in c])
    high = [bspline(13, unit([(math.cos(a), math.sin(a)) for a in
                              [PI / 2 * (k + i / 13.0) for i in range(14)]]),
                    [(0.0, 14), (1.0 + k, 14)]) for k in range(4)]
    for kind in (0, 1):
        add(requests, 'g2c_concat', kind, 1, 1e-2, 0.0, 3, 1e-2, 1e-2, 1e-2, 4,
            *[v for c in high for v in c])


def c0c1_requests(requests):
    smooth_c0 = bspline(3, unit([(0, 0), (1, 1), (2, 1), (3, 0), (4, -1), (5, -1), (6, 0)]),
                        [(0, 4), (1, 3), (2, 4)])
    closed_c0 = bspline(2, [(1, 0, 1), (1, 1, R2), (0, 1, 1), (-1, 1, R2), (-1, 0, 1),
                            (-1, -1, R2), (0, -1, 1), (1, -1, R2), (1, 0, 1)],
                        [(0, 3), (1, 2), (2, 2), (3, 2), (4, 3)])
    polygon = bspline(1, unit([(0, 0), (1, 0), (1, 1), (0, 1), (0, 0)]),
                      [(0, 2), (1, 1), (2, 1), (3, 1), (4, 2)])
    for spec in [C2, C0, smooth_c0, closed_c0, polygon, RAT_C1, PERIODIC]:
        for mode in (0, 1, 2):
            for tol, ang in [(1e-7, 1e-9), (1e-3, 1e-2), (1e-2, 1e-12)]:
                add(requests, 'g2c_c0c1', mode, tol, ang, *spec)


def approx_requests(requests):
    curves = [offset(0.5, ellipse(4.0, 1.5)), offset(-0.3, C2), offset(0.2, S_BEZIER),
              trimmed(0.2, 2.5, offset(0.5, ellipse(4.0, 1.5))), C2, trimmed(0.5, 2.0, circle(2.0)),
              offset(0.1, C0)]
    for c in curves:
        for order, seg_count, deg, tol in [(4, 16, 14, 1e-4), (2, 16, 14, 1e-7), (0, 1, 3, 1e-2),
                                          (5, 16, 14, 1e-4), (4, 0, 14, 1e-4), (2, 3, 1, 1e-3)]:
            add(requests, 'g2c_approx', 0, 0, 0.0, 0.0, tol, order, seg_count, deg, *c)
        add(requests, 'g2c_approx', 1, 0, 0.0, 0.0, 1e-4, 4, 16, 14, *c)
    add(requests, 'g2c_approx', 1, 1, 0.3, 1.7, 1e-5, 2, 10, 8, *offset(0.25, C2))
    add(requests, 'g2c_approx', 1, 1, 1.0, 5.0, 1e-5, 4, 10, 8, *ellipse(4.0, 1.5))


def arcs_requests(requests):
    cases = [
        (1, -2.0, 3.0, line()), (1, 1.0, 1.0 + 1e-9, line()),
        # No circle arc as short as 1e-9: GC_MakeArcOfCircle2d then reads the status gce_MakeCirc2d
        # leaves unassigned for parallel bisectors (indeterminate natively).
        (0, 0.0, 0.0, circle(2.5)), (1, 0.5, 2.0, circle(2.5, 0)), (1, 0.0, 1e-3, circle(2.5)),
        (0, 0.0, 0.0, ellipse(4.0, 1.5)), (1, 0.3, 2.9, ellipse(4.0, 1.5, 0)),
        (1, -1.0, 1.0, hyperbola(3.0, 1.0)), (1, -2.0, 2.0, parabola(0.7)),
        (0, 0.0, 0.0, S_BEZIER), (0, 0.0, 0.0, RAT_BEZIER), (0, 0.0, 0.0, C2), (0, 0.0, 0.0, C0),
        (0, 0.0, 0.0, LINEAR_PART), (0, 0.0, 0.0, PERIODIC), (0, 0.0, 0.0, RAT_C1),
        (0, 0.0, 0.0, offset(0.3, S_BEZIER)),
        (0, 0.0, 0.0, bspline(3, unit([(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]), [(0, 4), (1, 1), (2, 4)])),
        (0, 0.0, 0.0, bspline(3, unit([(0, 0), (1, 1), (2, 1), (3, 0), (4, 0), (5, 0), (6, 0),
                                       (7, 1), (8, -1)]), [(0, 4), (1, 1), (2, 1), (3, 1), (4, 1),
                                                           (5, 1), (6, 4)])),
    ]
    for ranged, u1, u2, spec in cases:
        for tol, ang in [(1e-2, 0.1), (1e-3, 0.01), (0.1, 0.5), (1e-4, 1e-3)]:
            add(requests, 'g2c_arcs', ranged, u1, u2, tol, ang, *spec)


def ppoint_requests(requests):
    for spec in [circle(2.5), S_BEZIER, C2]:
        for u1, u2 in [(0.1, 0.1 + 1e-10), (0.1, 0.2), (0.5, 0.5)]:
            add(requests, 'g2c_ppoint', u1, u2, 1.0, 2.0, 0.0, -1.0, *spec)


def append_geom2dconvert_requests(requests: list[str]) -> None:
    """Append every Geom2dConvert request family in a fixed order."""
    to_bspline_requests(requests)
    split_requests(requests)
    knot_split_requests(requests)
    bezier_requests(requests)
    comp_requests(requests)
    concat_requests(requests)
    c0c1_requests(requests)
    approx_requests(requests)
    arcs_requests(requests)
    ppoint_requests(requests)
