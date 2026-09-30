"""Deterministic requests for the BRepLib vertex and edge makers
(tools/oracle/tktopalgo_probe_breplib_edge.cxx).

`blm_vertex` covers BRepLib_MakeVertex with the default and a changed precision.
`blm_edge` covers every BRepLib_MakeEdge constructor and Init member: segments between
points and vertices (identical, near, far), each gp curve whole, between parameters,
points and vertices, the Geom curves (Bezier, B-spline, periodic B-spline, trimmed and
offset curves), curves on planes, cylinders, spheres, cones, tori and spline surfaces,
and each BRepLib_EdgeError value; later Init calls on a done or failed maker.
`blm_edge2d` covers BRepLib_MakeEdge2d on the default plane and on a tilted plane.
"""
from __future__ import annotations
import random
from tktopalgo_corpus_breplib_records import (
    INF, geom2d_bezier, geom2d_bspline, geom2d_gp, geom2d_trimmed, geom_bezier, geom_bspline, geom_gp,
    geom_offset, geom_trimmed, bspline_surface, bezier_surface, cone, cylinder, numbers, plane, sphere,
    torus, trimmed_plane, vertices, GP_BUILDERS, GP2D_BUILDERS)

# A vertex table shared by the requests: indices 0..7.
TABLE = [(0., 0., 0., 1e-7), (1., 0., 0., 1e-7), (0., 1., 0., 1e-7), (1., 0., 0., 0.5),
         (1., 1e-9, 0., 1e-7), (3., 4., 0., 1e-3), (0., 0., 1e-8, 1e-7), (2., 0., 0., 1e-7)]


def edge_request(calls: list[list[float]], prec: float = 0., table=TABLE) -> str:
    """`blm_edge prec vertices ncalls (call)...`."""
    fields = [prec] + vertices(table) + [len(calls)] + [c for call in calls for c in call]
    return 'blm_edge ' + numbers(fields)


def edge2d_request(calls: list[list[float]], prec: float = 0., frame=None, table=TABLE) -> str:
    """`blm_edge2d prec plane vertices ncalls (call)...`; `frame` is (P, N) or None."""
    plane_fields = [0] if frame is None else [1, *frame[0], *frame[1]]
    fields = [prec] + plane_fields + vertices(table) + [len(calls)] + [c for call in calls for c in call]
    return 'blm_edge2d ' + numbers(fields)


def tails_3d() -> list[tuple[int, list[float]]]:
    """(j, fields) of the curve tails: nothing, parameters, points, vertices."""
    return [(0, []), (1, [0.25, 1.5]), (1, [1.5, 0.25]), (1, [-INF, 2.]), (1, [-1., INF]),
            (2, [0., 0., 0., 1., 0., 0.]), (2, [5., 5., 5., 1., 0., 0.]), (3, [0, 1]), (3, [1, 0]),
            (3, [-1, 7]), (3, [3, -1]), (3, [0, 5]), (3, [0, 0])]


def gp_constructor_requests() -> list[str]:
    """Every gp curve kind with every tail of its four constructors."""
    curves = [GP_BUILDERS[0](), GP_BUILDERS[0](p=(0., 0., 0.), d=(0.6, 0.8, 0.)), GP_BUILDERS[1](),
              GP_BUILDERS[1](radius=1e-8), GP_BUILDERS[1](p=(1., 2., 3.), n=(0., 1., 0.), radius=2.5),
              GP_BUILDERS[2](), GP_BUILDERS[3](), GP_BUILDERS[4](), GP_BUILDERS[4](focal=0.25)]
    kinds = [0, 0, 1, 1, 1, 2, 3, 4, 4]
    requests = []
    for kind, curve in zip(kinds, curves):
        for j, tail in tails_3d():
            if j > 3:
                continue
            requests.append(edge_request([[10 + 4 * kind + j] + curve + tail]))
    return requests


def geom_curves() -> list[list[float]]:
    """Geom curve records: conics, Bezier, B-splines, trimmed and offset curves."""
    line = geom_gp(0)
    circle = geom_gp(1)
    return [line, circle, geom_gp(2), geom_gp(3), geom_gp(4),
            geom_bezier([(0., 0., 0.), (1., 1., 0.), (2., 0., 0.)]),
            geom_bezier([(0., 0., 0.), (1., 0., 0.), (0., 0., 0.)]),
            geom_bspline(2, False, [(0., 0., 0.), (1., 1., 0.), (2., 0., 0.), (3., 1., 0.)],
                         [(0., 3), (0.5, 1), (1., 3)]),
            geom_bspline(2, True, [(1., 0., 0.), (0., 1., 0.), (-1., 0., 0.), (0., -1., 0.)],
                         [(0., 1), (1., 1), (2., 1), (3., 1), (4., 1)]),
            geom_trimmed(0., 1., line), geom_trimmed(0.5, 2., circle), geom_trimmed(0.2, 0.8, geom_trimmed(0., 1., line)),
            geom_offset(0.5, (0., 0., 1.), circle), geom_offset(0.1, (0., 0., 1.), line)]


def geom_constructor_requests() -> list[str]:
    """Every Geom curve with the six tails (ids 30..35), and the curve ends as vertices."""
    tails = tails_3d() + [(4, [0., 0., 0., 1., 0., 0., 0., 1.]), (4, [0., 0., 0., 0., 0., 1e-9, 0., 1e-9]),
                          (5, [0, 1, 0., 1.]), (5, [0, 1, 1., 0.]), (5, [0, 0, 0., 6.283185307179586]),
                          (5, [-1, 1, -INF, 1.]), (5, [0, -1, 0., INF]), (5, [5, 1, 0., 1.]),
                          (5, [0, 1, -INF, 1.]), (5, [0, 1, 0., INF])]
    return [edge_request([[30 + j] + curve + tail]) for curve in geom_curves() for j, tail in tails]


def pcurve_cases() -> list[tuple[list[float], list[float]]]:
    """(2d curve, surface) pairs for the curves on surface."""
    seg = geom2d_gp(0, p=(0., 0.), d=(1., 0.))
    vline = geom2d_gp(0, p=(0.5, 0.), d=(0., 1.))
    circ = geom2d_gp(1, p=(0., 0.), x=(1., 0.), radius=0.5)
    bez = geom2d_bezier([(0., 0.), (0.5, 1.), (1., 0.)])
    spl = geom2d_bspline(2, False, [(0., 0.), (0.3, 0.6), (0.6, 0.2), (1., 1.)], [(0., 3), (0.5, 1), (1., 3)])
    patch = bezier_surface([[(0., 0., 0.), (0., 1., 0.)], [(1., 0., 0.5), (1., 1., 0.)]])
    spline = bspline_surface(1, 1, [[(0., 0., 0.), (0., 1., 0.)], [(1., 0., 0.), (1., 1., 1.)]],
                             [(0., 2), (1., 2)], [(0., 2), (1., 2)])
    return [(seg, plane()), (circ, plane()), (bez, plane()), (seg, cylinder()), (vline, cylinder()),
            (vline, sphere()), (seg, sphere()), (seg, cone()), (seg, torus()), (vline, torus()),
            (bez, patch), (spl, spline), (geom2d_trimmed(0., 1., seg), trimmed_plane(-1., 1., -1., 1.)),
            (spl, plane(p=(0., 0., 1.), n=(0., 1., 1.)))]


def pcurve_constructor_requests() -> list[str]:
    """Every curve on surface with the six tails (ids 40..45)."""
    tails = [(0, []), (1, [0.1, 0.9]), (1, [0.9, 0.1]), (1, [-INF, 0.5]), (2, [0., 0., 0., 1., 0., 0.]),
             (2, [1., 0., 0., 0., 1., 0.]), (3, [0, 1]), (3, [-1, 1]), (3, [5, 1]), (3, [0, 0]),
             (4, [0., 0., 0., 1., 0., 0., 0., 1.]), (5, [0, 1, 0., 1.]), (5, [1, 0, 1., 0.]),
             (5, [0, -1, 0., INF]), (5, [0, 0, 0., 6.283185307179586]), (5, [0, 1, -INF, 0.5])]
    return [edge_request([[40 + j] + curve + surf + tail]) for curve, surf in pcurve_cases() for j, tail in tails]


def segment_requests() -> list[str]:
    """Constructors 0, 1 and 2, with the default and changed precisions."""
    requests = [edge_request([[0]])]
    pairs = [(0, 1), (0, 0), (1, 4), (0, 5), (0, 6), (3, 1), (1, 3), (5, 7)]
    points = [((0., 0., 0.), (1., 0., 0.)), ((0., 0., 0.), (0., 0., 0.)), ((0., 0., 0.), (1e-8, 0., 0.)),
              ((0., 0., 0.), (1e-16, 0., 0.)), ((1., 2., 3.), (-4., 5., 0.5)), ((1e20, 0., 0.), (-1e20, 1., 0.))]
    for prec in (0., 1e-3):
        requests += [edge_request([[1, a, b]], prec) for a, b in pairs]
        requests += [edge_request([[2, *a, *b]], prec) for a, b in points]
    return requests


def reinit_requests() -> list[str]:
    """A constructor, then Init members that succeed or fail (the maker keeps its edge)."""
    line, circle = geom_gp(0), geom_gp(1)
    bez = geom_bezier([(0., 0., 0.), (1., 1., 0.), (2., 0., 0.)])
    return [edge_request([[1, 0, 1], [31] + bez + [1.5, 2.]]),
            edge_request([[31] + bez + [1.5, 2.], [31] + line + [0., 2.]]),
            edge_request([[30] + circle, [33] + line + [0, 5]]),
            edge_request([[0], [30] + line, [35] + circle + [0, 1, 0., 1.]]),
            edge_request([[2, 0., 0., 0., 1., 0., 0.], [41] + geom2d_gp(0) + cylinder() + [0., 1.]])]


def random_requests(rng: random.Random) -> list[str]:
    """Seeded segments between random points and random arcs of random circles."""
    requests = []
    for _ in range(12):
        a = [rng.uniform(-10., 10.) for _ in range(3)]
        b = [rng.uniform(-10., 10.) for _ in range(3)]
        requests.append(edge_request([[2, *a, *b]]))
        n = [rng.uniform(-1., 1.) for _ in range(3)]
        radius = rng.uniform(0.1, 5.)
        p1, p2 = rng.uniform(-7., 7.), rng.uniform(-7., 7.)
        requests.append(edge_request([[15] + GP_BUILDERS[1](p=tuple(a), n=tuple(n), radius=radius) + [p1, p2]]))
    return requests


def edge2d_requests() -> list[str]:
    """BRepLib_MakeEdge2d on the default plane and on a tilted plane."""
    frames = [None, ((1., 2., 3.), (0., 0.6, 0.8))]
    table = [(0., 0., 0., 1e-7), (1., 0., 0., 1e-7), (0., 1., 0., 1e-7), (0., 0., 0.5, 1e-7), (1., 2., 3., 1e-7)]
    tails = [(0, []), (1, [0.25, 1.5]), (1, [1.5, 0.25]), (1, [-INF, 2.]), (2, [0., 0., 1., 0.]),
             (2, [0., 0., 0., 1e-9]), (3, [0, 1]), (3, [1, 0]), (3, [-1, 2]), (3, [0, 0]), (3, [4, 1])]
    curves = [(k, GP2D_BUILDERS[k]()) for k in range(5)] + [(1, GP2D_BUILDERS[1](radius=1e-8)),
                                                          (1, GP2D_BUILDERS[1](sense=False))]
    requests = []
    for frame in frames:
        requests += [edge2d_request([[1, 0, 1]], frame=frame, table=table),
                     edge2d_request([[1, 0, 3]], frame=frame, table=table),
                     edge2d_request([[2, 0., 0., 1., 1.]], frame=frame, table=table),
                     edge2d_request([[2, 1., 1., 1., 1.]], frame=frame, table=table)]
        for kind, curve in curves:
            requests += [edge2d_request([[10 + 4 * kind + j] + curve + tail], frame=frame, table=table)
                         for j, tail in tails if j <= 3]
        spl = geom2d_bspline(2, False, [(0., 0.), (0.3, 0.6), (0.6, 0.2), (1., 1.)], [(0., 3), (0.5, 1), (1., 3)])
        for curve in [geom2d_gp(0), geom2d_gp(1), geom2d_bezier([(0., 0.), (0.5, 1.), (1., 0.)]), spl,
                      geom2d_trimmed(0.5, 2., geom2d_gp(1))]:
            more = [(4, [0., 0., 1., 0., 0., 1.]), (5, [0, 1, 0., 1.]), (5, [-1, 1, -INF, 1.]), (5, [0, 0, 0., 1.])]
            # The vertex (1, 2, 3) projects beyond the end of the B-spline: Extrema_ExtPC2d is
            # done without extremum and the native parameter stays uninitialized (garbage).
            skip = [(3, [4, 1])] if curve is spl else []
            requests += [edge2d_request([[30 + j] + curve + tail], frame=frame, table=table)
                         for j, tail in tails + more if (j, tail) not in skip]
        requests.append(edge2d_request([[31] + geom2d_bezier([(0., 0.), (1., 1.)]) + [2., 3.],
                                        [31] + geom2d_gp(0) + [0., 1.]], frame=frame, table=table))
    return requests


def vertex_requests() -> list[str]:
    points = [(0., 0., 0.), (1., 2., 3.), (-0., 1e-300, 1e300)]
    return ['blm_vertex ' + numbers([prec, *p]) for prec in (0., 1e-3, 0.5) for p in points]


def append_breplib_edge_requests(requests: list[str]) -> None:
    """Append the BRepLib vertex and edge requests in a fixed order."""
    requests += vertex_requests()
    requests += segment_requests()
    requests += gp_constructor_requests()
    requests += geom_constructor_requests()
    requests += pcurve_constructor_requests()
    requests += reinit_requests()
    requests += random_requests(random.Random(20261001))
    requests += edge2d_requests()
