"""Deterministic requests for the Geom2dHatch package and its TopClass instances
(tools/oracle/tkgeomalgo_probe_geom2dhatch.cxx and tkgeomalgo_probe_geom2dhatcher.cxx).

Curves are written as ga_curve2d reads them, followed by `u0 u1` (the adaptor range when
u0 < u1, the whole curve otherwise). The boundaries are polygons (both orientations), a disc,
an annulus, a half disc, a region closed by B-spline and Bezier arcs, open boundaries,
INTERNAL and EXTERNAL edges and infinite lines. The requests cover LocalGeometry on every
curve kind (with a degenerate B-spline), the segment Perform (bounded and to RealLast),
Intersect with its tolerances, the explorer and its probing segments, the classifier and
FClass2d on points inside, outside, on edges, on vertices and near them, and the hatcher on
hatchings crossing, touching, tangent to and along the boundaries, with every flag and
every operation.
"""
from __future__ import annotations
import math
import random
from tkgeomalgo_corpus_intcurve import fmt
from tkgeomalgo_corpus_geom2d import (BEZIER, CURVES, KINKED, RANGES, WAVE, c_bezier, c_bspline, c_circle,
                                      c_ellipse, c_line, c_offset, c_trimmed)

TWO_PI = 2.0 * math.pi
REAL_LAST = 1.7976931348623157e308
FORWARD, REVERSED, INTERNAL, EXTERNAL = 0, 1, 2, 3
DEGENERATE = c_bspline(3, [0.0, 1.0], [4, 4], [(1.0, 1.0)] * 4)


def segment(x0, y0, x1, y1):
    """A line segment as (curve, u0, u1)."""
    length = math.hypot(x1 - x0, y1 - y0)
    return (c_line(x0, y0, (x1 - x0) / length, (y1 - y0) / length), 0.0, length)


def polygon(points):
    """The segments of the closed polygon `points`."""
    return [segment(*points[k], *points[(k + 1) % len(points)]) for k in range(len(points))]


def whole(curve, u0=0.0, u1=0.0):
    return (curve, u0, u1)


SQUARE = polygon([(0.0, 0.0), (2.0, 0.0), (2.0, 2.0), (0.0, 2.0)])
TRIANGLE = polygon([(-1.0, -1.0), (3.0, -1.0), (1.0, 2.0)])
CONCAVE = polygon([(0.0, 0.0), (4.0, 0.0), (4.0, 3.0), (2.0, 1.0), (0.0, 3.0)])
DISC = [whole(c_circle(0.0, 0.0, 1.0), 0.0, TWO_PI)]
ANNULUS = [whole(c_circle(0.0, 0.0, 2.0), 0.0, TWO_PI), whole(c_circle(0.0, 0.0, 1.0, 0), 0.0, TWO_PI)]
HALF_DISC = [segment(-1.0, 0.0, 1.0, 0.0), whole(c_circle(0.0, 0.0, 1.0), 0.0, math.pi)]
ARCH = c_bezier([(3.0, 0.0), (3.0, 3.0), (-3.0, 3.0), (-3.0, 0.0)])
CURVED = [segment(-3.0, 0.0, 3.0, 0.0), whole(ARCH, 0.0, 1.0)]
SPLINE_ARCH = c_bspline(3, [0.0, 1.0, 2.0], [4, 1, 4], [(3.0, 0.0), (3.0, 2.0), (0.0, 3.0), (-3.0, 2.0), (-3.0, 0.0)])
SPLINE_REGION = [segment(-3.0, 0.0, 3.0, 0.0), whole(SPLINE_ARCH, 0.0, 2.0)]
ELLIPSE = [whole(c_ellipse(0.0, 0.0, 2.0, 1.0), 0.0, TWO_PI)]
# Collinear edges along y = 0 (a hatching there ends one segment where the next begins) and a
# notch of 1e-4 on y = 1 (both ends confused under a 1e-3 confusion).
SPLIT = polygon([(0.0, 0.0), (1.0, 0.0), (2.0, 0.0), (2.0, 2.0), (1.0, 2.0), (1.0, 1.0), (1.0001, 1.0),
                 (1.0001, 1.5), (0.0, 1.5)])
# (name, boundary, orientation of every edge, extra (orientation, edge) pairs).
SHAPES = [
    ('square', SQUARE, FORWARD, []), ('square_cw', SQUARE, REVERSED, []), ('triangle', TRIANGLE, FORWARD, []),
    ('concave', CONCAVE, FORWARD, []), ('disc', DISC, FORWARD, []), ('annulus', ANNULUS, FORWARD, []),
    ('half_disc', HALF_DISC, FORWARD, []), ('curved', CURVED, FORWARD, []), ('spline', SPLINE_REGION, FORWARD, []),
    ('ellipse', ELLIPSE, FORWARD, []), ('open', SQUARE[:3], FORWARD, []),
    ('internal', SQUARE, FORWARD, [(INTERNAL, segment(0.5, 1.0, 1.5, 1.0))]),
    ('external', SQUARE, FORWARD, [(EXTERNAL, segment(1.0, 0.5, 1.0, 1.5))]),
    ('line', [whole(c_line(0.0, 0.0, 1.0, 0.0))], FORWARD, []),
    ('split', SPLIT, FORWARD, []), ('split_cw', SPLIT, REVERSED, []),
]


def shape_edges(shape):
    _, boundary, orientation, extra = shape
    return [(orientation, edge) for edge in boundary] + extra


def keyed_elements(shape, keys=None):
    """`n (key orientation curve u0 u1)*n` of the elements probes."""
    edges = shape_edges(shape)
    keys = keys or list(range(1, len(edges) + 1))
    values = [len(edges)]
    for key, (orientation, (curve, u0, u1)) in zip(keys, edges):
        values += [key, orientation, *curve, u0, u1]
    return values


def append_local_requests(requests: list[str]) -> None:
    """LocalGeometry on every curve kind at its ends, inside and outside its range."""
    curves = list(zip(CURVES, RANGES)) + [(DEGENERATE, (0.0, 1.0)), (ARCH, (0.0, 1.0)), (SPLINE_ARCH, (0.0, 2.0)),
                                          (c_offset(-0.5, KINKED), (0.0, 3.0))]
    for curve, (u0, u1) in curves:
        for u in (u0, u1, 0.5 * (u0 + u1), u0 + 0.137 * (u1 - u0), u1 + 0.25):
            requests.append('g2hatch_local ' + fmt([*curve, u0, u1, u]))
    for u in (1.0, 2.0, 0.0, 3.0):
        requests.append('g2hatch_local ' + fmt([*KINKED, 0.0, 0.0, u]))


def append_perform_requests(requests: list[str]) -> None:
    """The segment Perform against every curve, bounded and to RealLast."""
    lines = [(0.0, 0.3, 1.0, 0.0), (-4.0, -0.2, 0.8, 0.6), (0.0, 0.0, 0.0, 1.0), (5.0, 5.0, -0.6, -0.8),
             (0.0, 1.5, 1.0, 0.0), (-3.0, 0.5, 1.0, 0.0)]
    for index, (curve, (u0, u1)) in enumerate(zip(CURVES, RANGES)):
        for k, line in enumerate(lines):
            p = (REAL_LAST, 2.0, 10.0)[(index + k) % 3]
            tol = (1e-7, 1e-3)[(index * k) % 2]
            requests.append('g2hatch_perform ' + fmt([*line, p, tol, *curve, u0, u1]))
    for curve, u0, u1 in SQUARE + DISC + HALF_DISC:
        requests.append('g2hatch_perform ' + fmt([1.0, 1.0, 1.0, 0.0, REAL_LAST, 1e-7, *curve, u0, u1]))
        requests.append('g2hatch_perform ' + fmt([0.0, 0.0, 1.0, 0.0, 1.0, 1e-9, *curve, u0, u1]))


def append_intersect_requests(requests: list[str]) -> None:
    """Intersect(C1, C2) on pairs of curves, both parameter orders."""
    rng = random.Random(5521)
    pairs = [(0, 1), (1, 2), (0, 3), (8, 0), (9, 12), (13, 1), (6, 7), (14, 16), (10, 17), (5, 4)]
    pairs += [(rng.randrange(len(CURVES)), rng.randrange(len(CURVES))) for _ in range(20)]
    for k, (a, b) in enumerate(pairs):
        (ca, (a0, a1)), (cb, (b0, b1)) = (CURVES[a], RANGES[a]), (CURVES[b], RANGES[b])
        conf, tang = ((1e-9, 1e-9), (1e-7, 1e-6), (1e-5, 1e-7))[k % 3]
        requests.append('g2hatch_intersect ' + fmt([*ca, a0, a1, *cb, b0, b1, conf, tang, k % 2]))


def append_elements_requests(requests: list[str]) -> None:
    """The explorer of every shape from points inside, outside and on the boundary."""
    points = [(1.0, 1.0), (0.5, 0.25), (5.0, -3.0), (0.0, 0.0), (1.0, 0.0), (-0.5, 0.5)]
    for k, shape in enumerate(SHAPES):
        keys = [3 * j + (k % 3) for j in range(1, len(shape_edges(shape)) + 1)]
        for j, (x, y) in enumerate(points):
            nseg = (3, 12, 25)[(k + j) % 3]
            requests.append('g2hatch_elements ' + fmt([*keyed_elements(shape, keys if j % 2 else None), x, y, nseg]))
    requests.append('g2hatch_elements ' + fmt([0, 1.0, 1.0, 5]))


def classification_points(shape_name):
    base = [(1.0, 1.0), (0.5, 0.25), (5.0, -3.0), (0.0, 0.0), (1.0, 0.0), (2.0, 1.0), (1.0, 2.0), (0.0, 1.0),
            (1.0, 1e-8), (1.0, -1e-8), (0.3, 0.9), (-0.5, 0.5), (1.0, 0.5), (0.0, 1.0 + 1e-9), (0.25, 0.25)]
    if shape_name in ('disc', 'annulus', 'half_disc', 'ellipse'):
        base += [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (0.6, 0.8), (1.5, 0.0), (0.0, -0.5), (2.0, 0.0)]
    if shape_name in ('curved', 'spline'):
        base += [(0.0, 1.0), (0.0, 2.25), (3.0, 0.0), (-3.0, 0.0), (2.9, 0.1), (0.0, 3.5)]
    return base


def append_classify_requests(requests: list[str]) -> None:
    """The classifier of every shape on its characteristic points."""
    for shape in SHAPES:
        for k, (x, y) in enumerate(classification_points(shape[0])):
            tol = (1e-7, 1e-3, 0.0)[k % 3]
            requests.append('g2hatch_classify ' + fmt([*keyed_elements(shape), x, y, tol]))
    rng = random.Random(7731)
    for _ in range(60):
        shape = SHAPES[rng.randrange(len(SHAPES))]
        requests.append('g2hatch_classify ' + fmt([*keyed_elements(shape), rng.uniform(-1.5, 3.5),
                                                   rng.uniform(-1.5, 3.5), 1e-7]))
    requests.append('g2hatch_classify ' + fmt([0, 0.5, 0.5, 1e-7]))


def append_fclass_requests(requests: list[str]) -> None:
    """FClass2d from lines crossing, touching and missing the edges of every shape."""
    lines = [((1.0, 1.0, 1.0, 0.0), 10.0), ((1.0, 1.0, 0.0, 1.0), 0.5), ((0.0, 0.0, 1.0, 0.0), 5.0),
             ((-1.0, 0.0, 1.0, 0.0), REAL_LAST), ((1.0, 1.0, -0.6, 0.8), 3.0), ((0.5, -2.0, 0.0, 1.0), 10.0),
             ((0.0, 1.0, 1.0, 0.0), 10.0), ((1.0, 0.0, 0.0, 1.0), 10.0)]
    for shape in SHAPES:
        edges = shape_edges(shape)
        for k, (line, par) in enumerate(lines):
            values = [*line, par, (1e-7, 1e-3)[k % 2], len(edges)]
            for orientation, (curve, u0, u1) in edges:
                values += [orientation, *curve, u0, u1]
            requests.append('g2hatch_fclass ' + fmt(values))


def hatcher_elements(shape, handle=False):
    values = []
    for orientation, (curve, u0, u1) in shape_edges(shape):
        values += [orientation, *curve, 1.0, 0.0] if handle and not u0 < u1 else [orientation, *curve, u0, u1]
    return [len(shape_edges(shape)), *values]


def hatchings(ys, bounded=None, vertical=False):
    values = [len(ys)]
    for y in ys:
        curve = c_line(y, 0.0, 0.0, 1.0) if vertical else c_line(0.0, y, 1.0, 0.0)
        u0, u1 = bounded or (0.0, 0.0)
        values += [*curve, u0, u1]
    return values


def hatcher_request(shape, flags, lines, ops, confusions=(1e-9, 1e-7)):
    values = [*confusions, *flags, *hatcher_elements(shape, shape[0] == 'line'), *lines, len(ops)]
    for op, value in ops:
        values += [op, value]
    return 'g2hatch_hatcher ' + fmt(values)


# Hatching ordinates for each shape: across, through vertices, along edges, tangent, outside.
SHAPE_YS = {'square': [1.0, 0.0, 2.0, 0.5, 3.0], 'square_cw': [1.0, 0.0, 1.5], 'triangle': [0.0, -1.0, 2.0, 1.0],
            'concave': [0.5, 1.0, 2.0, 3.0], 'disc': [0.0, 1.0, 0.5, -0.99], 'annulus': [0.0, 1.0, 1.5, 2.0],
            'half_disc': [0.0, 0.5, 1.0], 'curved': [0.0, 1.0, 2.25, 2.5], 'spline': [0.0, 1.0, 2.0],
            'ellipse': [0.0, 1.0, 0.5], 'open': [1.0, 0.5], 'internal': [1.0, 0.5], 'external': [1.0, 0.5],
            'line': [0.0, 1.0], 'split': [0.0, 1.0, 2.0, 1.5, 0.5], 'split_cw': [0.0, 1.0, 1.5]}


def append_hatcher_requests(requests: list[str]) -> None:
    """The hatcher of every shape with every flag pair, then the operation sequences."""
    for shape in SHAPES:
        ys = SHAPE_YS[shape[0]]
        for flags in ((0, 0), (1, 0), (0, 1), (1, 1)):
            requests.append(hatcher_request(shape, flags, hatchings(ys), [(1, 0.0)]))
        requests.append(hatcher_request(shape, (1, 1), hatchings(ys, vertical=True), [(0, 0.0), (1, 0.0)]))
        requests.append(hatcher_request(shape, (0, 0), hatchings(ys, bounded=(0.25, 0.75)), [(3, 1.0), (3, 2.0)]))
        requests.append(hatcher_request(shape, (0, 1), hatchings([ys[0]]), [(1, 0.0)], confusions=(1e-3, 1e-2)))
        requests.append(hatcher_request(shape, (1, 1), hatchings(ys), [(1, 0.0)], confusions=(1e-3, 0.5)))
    append_hatcher_script_requests(requests)


def append_hatcher_script_requests(requests: list[str]) -> None:
    """Operation sequences: removals, flag and tolerance changes, clears, added hatchings."""
    scripts = [
        [(0, 0.0), (4, 1.0), (1, 0.0)], [(1, 0.0), (4, 2.0), (4, 9.0), (3, 1.0)], [(1, 0.0), (5, 1.0), (5, 7.0), (1, 0.0)],
        [(1, 0.0), (6, 1e-3), (1, 0.0)], [(1, 0.0), (7, 0.5), (8, 1.0), (9, 1.0)], [(0, 0.0), (10, 1e-5), (1, 0.0)],
        [(11, 0.0), (1, 0.0)], [(12, 0.0), (1, 0.0)], [(13, 0.0), (14, 1.0), (1, 0.0)], [(15, 0.5), (15, 2.0), (1, 0.0)],
        [(2, 1.0), (2, 9.0), (3, 2.0)], [(1, 0.0), (4, 4.0), (14, 1.2), (1, 0.0)], [(5, 2.0), (14, 1.3), (15, 1.4)],
        [(1, 0.0), (16, 1.0), (17, 1.0), (1, 0.0)], [(16, 9.0), (17, 9.0), (17, 0.0), (1, 0.0)],
    ]
    for shape_index in (0, 2, 4, 5, 11, 14):
        shape = SHAPES[shape_index]
        for k, ops in enumerate(scripts):
            flags = ((0, 0), (1, 1), (1, 0), (0, 1))[k % 4]
            requests.append(hatcher_request(shape, flags, hatchings(SHAPE_YS[shape[0]]), ops))
    rng = random.Random(3319)
    for _ in range(30):
        shape = SHAPES[rng.randrange(len(SHAPES))]
        ys = [round(rng.uniform(-1.5, 2.5), 3) for _ in range(rng.randrange(1, 5))]
        requests.append(hatcher_request(shape, (rng.randrange(2), rng.randrange(2)), hatchings(ys, vertical=rng.random() < 0.3),
                                        [(0, 0.0), (1, 0.0)]))


def hatching_point(param, position, before, after, begin, end, elements):
    values = [param, position, before, after, begin, end, len(elements)]
    for index, value in elements:
        values += [index, value]
    return values


def append_hatching_requests(requests: list[str]) -> None:
    """The Hatching members on points added in and out of order, equal within the confusion."""
    p1 = hatching_point(0.5, INTERNAL, 1, 0, 0, 0, [(1, 0.2)])
    p2 = hatching_point(0.25, FORWARD, 0, 1, 1, 0, [(2, 0.7), (2, 0.7)])
    p3 = hatching_point(0.5 + 1e-10, INTERNAL, 1, 0, 0, 0, [(3, 0.1)])
    p4 = hatching_point(1.5, REVERSED, 3, 3, 0, 1, [])
    curves = [(c_line(0.0, 1.0, 1.0, 0.0), 0.0, 0.0), (c_line(0.0, 1.0, 1.0, 0.0), -1.0, 2.0),
              (c_circle(0.0, 0.0, 1.0), 0.0, TWO_PI), (WAVE, 0.0, 3.0), (BEZIER, 0.0, 1.0),
              (c_trimmed(0.0, 5.0, c_line(1.0, 1.0, 0.0, 1.0)), 0.0, 0.0)]
    point_sets = [[], [p1], [p1, p2], [p2, p1, p3], [p4, p3, p2, p1], [p1, p1, p1]]
    for k, (curve, u0, u1) in enumerate(curves):
        for j, points in enumerate(point_sets):
            conf = (1e-9, 0.0, 1e-3)[(k + j) % 3]
            values = [*curve, u0, u1, conf, len(points)]
            for point in points:
                values += point
            requests.append('g2hatch_hatching ' + fmt([*values, (j % 4)]))


def append_geom2dhatch_requests(requests: list[str]) -> None:
    """Every Geom2dHatch request family in a fixed order."""
    append_local_requests(requests)
    append_perform_requests(requests)
    append_intersect_requests(requests)
    append_elements_requests(requests)
    append_classify_requests(requests)
    append_fclass_requests(requests)
    append_hatcher_requests(requests)
    append_hatching_requests(requests)
