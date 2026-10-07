"""Deterministic requests for the ChFi3d_Builder_0 probes (tools/oracle/tkfillet_probe_builder0.cxx).

Periodic parameters and boxes, the bounds of surfaces and faces around boxes, the boxes of
curves and edges, the faces and states around the edges and vertices of solids, the corner
guide lines, the cut of curves by planes, the straight, projected and reparametrized
pcurves with their same-parameter checks, and the cubic pcurves and boundaries of corners.
"""
from __future__ import annotations
import math

from tkfillet_corpus_blendfunc import numbers
from tkfillet_corpus_chfi3d import SOLIDS, guide

PI = math.pi


def surface(kind, p=(0, 0, 0), n=(0, 0, 1), x=(1, 0, 0), a=0.0, b=0.0):
    """`fl_surface`: 0 plane, 1 cylinder (a), 2 sphere (a), 3 cone (angle a, radius b),
    4 torus (a, b), in the frame (p, n, x)."""
    return [kind, *p, *n, *x, a, b]


SURFACES = [
    surface(0),
    surface(0, (1, 2, 3), (0, 1, 1), (1, 0, 0)),
    surface(1, a=2.0),
    surface(2, a=3.0),
    surface(3, a=0.3, b=2.0),
    surface(4, a=5.0, b=1.0),
]
LINE_X = guide(0, (0, 0, 0), (1, 0, 0), (0, 1, 0), 0)
LINE_Y1 = guide(0, (1, 0, 0), (0, 1, 0), (1, 0, 0), 0)
CIRCLE2 = guide(1, (0, 0, 1), (0, 0, 1), (1, 0, 0), 2.0)
SLANT = guide(0, (0, 0, -2), (1, 1, 1), (1, -1, 0), 0)


def scalars(requests: list[str]) -> None:
    pts = [0, 0, 1, 1, 2, 2, 3, 3]
    for u, f, l, eps in ((7, 0, 5, 1e-9), (-3, 0, 5, 1e-9), (5, 0, 5, 1e-9), (0, 0, 2 * PI, 1e-7),
                         (2 * PI + 1e-8, 0, 2 * PI, 1e-7), (-1e-10, 0, 1, 1e-9), (13.5, -1, 1, 0.0)):
        requests.append('fl_b0_scalars ' + numbers([u, f, l, eps, *pts]))
    requests.append('fl_b0_scalars ' + numbers([1, 0, 1, 1e-9, 3, -1, -2, 5, 0.5, 0.5, 4, -3]))


def bounds(requests: list[str]) -> None:
    for s in SURFACES:
        for box in ((0.2, 0.4, -0.5, 0.5), (1.0, 2.0, 0.1, 0.3), (-1.0, 1.0, -3.0, -1.0)):
            for check in (0, 1):
                requests.append('fl_bound_srf ' + numbers([*s, 0, 0, 0, 0, 0, *box, check]))
        requests.append('fl_bound_srf ' + numbers([*s, 1, 0.0, 1.5, -0.5, 0.75, 0.2, 0.4, -0.2, 0.3, 1]))
    for solid in SOLIDS:
        for t, check in ((0.25, 1), (0.25, 0), (0.45, 1)):
            requests.append('fl_bound_fac ' + numbers([*solid, t, check]))


def boxes(requests: list[str]) -> None:
    for g, s in ((LINE_X, SURFACES[0]), (CIRCLE2, SURFACES[2]), (SLANT, SURFACES[3]), (LINE_Y1, SURFACES[5])):
        for wd, wf in ((0.0, 1.0), (-2.0, 3.5)):
            requests.append('fl_enlarge_box ' + numbers([*g, wd, wf, *s, 0.1, 0.2, 0.6, 0.8, 0.0, 1.5]))
    for solid in SOLIDS:
        requests.append('fl_b0_faces ' + numbers([*solid, 0.3, 1.0]))


def spines(requests: list[str]) -> None:
    for pd, vd, pf, vf, r in (((0, 0, 0), (1, 0, 0), (2, 2, 0), (0, 1, 0), 1.0),
                              ((0, 0, 0), (1, 0, 0), (2, 2, 0), (0, -1, 0), 1.0),
                              ((0, 0, 0), (1, 0, 0), (0, 3, 1), (-1, 1, 0), 2.0),
                              ((0, 0, 0), (1, 0, 0), (5, 0, 0), (1, 0, 0), 1.0),
                              ((1, 2, 3), (0, 0, 2), (4, 2, 6), (3, 0, 0), 0.5)):
        requests.append('fl_b0_spines ' + numbers([*pd, *vd, *pf, *vf, r]))


def plane_cuts(requests: list[str]) -> None:
    for s in (SURFACES[0], SURFACES[1], surface(0, (0, 0, 0), (1, 0, 0), (0, 1, 0))):
        for g, f, l in ((SLANT, -5, 5), (SLANT, 3, 5), (CIRCLE2, 0, 2 * PI), (guide(1, (0, 0, 0), (0, 1, 0), (1, 0, 0), 2.0), 0, 2 * PI),
                        (guide(1, (0, 0, 0), (0, 1, 0), (1, 0, 0), 2.0), 1.0, 1.0 + 2 * PI)):
            for sens in (0, 1):
                requests.append('fl_inter_plane_edge ' + numbers([*s, *g, f, l, sens, 1e-7]))


def pcurves(requests: list[str]) -> None:
    plane, cyl = SURFACES[0], SURFACES[2]
    cases = [
        ((1, 0), (1, 5), 0, 5, LINE_Y1, plane),
        ((1, 5), (1, 0), 0, 5, LINE_Y1, plane),
        ((1, 7), (1, 2), 2, 7, LINE_Y1, plane),
        ((0, 1), (5, 1), 0, 5, guide(0, (0, 1, 0), (1, 0, 0), (0, 1, 0), 0), plane),
        ((5, 1), (0, 1), 0, 5, guide(0, (0, 1, 0), (1, 0, 0), (0, 1, 0), 0), plane),
        ((0, 1), (PI, 1), 0, PI, CIRCLE2, cyl),
        ((0, 1), (PI, 1.2), 0, PI, CIRCLE2, cyl),
        ((0, 0), (4, 3), 0, 5, guide(0, (0, 0, 0), (0.8, 0.6, 0), (0, 0, 1), 0), plane),
        ((0, 0), (4, 3.5), 0, 5, guide(0, (0, 0, 0), (0.8, 0.6, 0), (0, 0, 1), 0), plane),
    ]
    for uv1, uv2, pd, pf, g, s in cases:
        for rev in (0, 1):
            requests.append('fl_b0_pcurves ' + numbers([*uv1, *uv2, pd, pf, rev, *g, *s, 1e-6]))
    for trim, t1, t2, uf, ul in ((0, 0, 0, 0, 1), (0, 0, 0, 2, 5), (1, 0.2, 0.7, 0.2, 0.7), (1, 0.2, 0.7, 0, 1), (1, 0.1, 0.9, -1, 3)):
        requests.append('fl_reparam_pcurv ' + numbers([0, 0, 1, 2, 3, 2, 4, 0, 0, 1, trim, t1, t2, uf, ul]))
    for g, f, l, s in ((LINE_Y1, 0, 3, plane), (SLANT, -1, 1, plane), (CIRCLE2, 0, PI, cyl), (CIRCLE2, 0, PI, SURFACES[3]),
                       (guide(0, (2, 0, 0), (0, 0, 1), (1, 0, 0), 0), -1, 2, cyl), (CIRCLE2, 0, 1, SURFACES[5])):
        requests.append('fl_project_pcurv ' + numbers([*g, f, l, *s, 1e-6]))


def corner_bounds(requests: list[str]) -> None:
    for s in (SURFACES[0], SURFACES[2], SURFACES[3]):
        for red in (0, 1):
            for s1, s2 in ((0, 0), (1, 0), (0, 1)):
                for free in (0, 1):
                    requests.append('fl_b0_bounds ' + numbers([*s, 0.1, 0.2, 1, 0, 0.9, 0.7, 0, 1, red,
                                                               1, 0, 0.2, 0, 1, 0.3, s1, s2, 1e-4, 1e-2, free]))


def aretes(requests: list[str]) -> None:
    guide_line = guide(0, (0, 0, 0), (0.6, 0.8, 0), (0, 0, 1), 0)
    for s in (SURFACES[0], SURFACES[2], SURFACES[3]):
        for uv1, uv2 in (((0, 0), (0, 3)), ((0, 3), (0, 0)), ((0.5, 1), (2, 1)), ((2, 1), (0.5, 1)), ((0, 0), (2, 3)), ((0.2, 0.1), (1.0, 0.7))):
            for m1, m2 in ((0, 0), (1, 1), (2, 1)):
                for iflag in (0, 1, 2):
                    requests.append('fl_compute_arete ' + numbers([*s, *uv1, *uv2, m1, 0.5, m2, 0.25, iflag, 1e-4, 1e-5,
                                                                   *guide_line, 0.0, 2.0]))


def common_points(requests: list[str]) -> None:
    for p in ((0, 0, 0), (0, 0, 5), (0.5, 0.3, 10)):
        for ei in (1, 3):
            for isv, isa in ((0, 0), (1, 0), (0, 1), (1, 1)):
                for trans, start in ((0, 1), (1, 0), (2, 1)):
                    requests.append('fl_fil_common_point ' + numbers([0, 10, 20, 30, 0, 1, ei, *p, 0.1, 0.2, 7.5, 1e-3, isv, isa,
                                                                      4.0, 1, 0, 0, 1, trans, start, 1e-5, 1e-4]))
    for rev, nsd in ((0, -1), (0, 0), (0, 3), (5, 2), (2, 4)):
        requests.append('fl_b0_stripes ' + numbers([4, 0, 0, 0, 1, 0, 0, 1, 1, 0, 2, 1, 1, rev, nsd]))


def tolerances(requests: list[str]) -> None:
    pivots = (([1, 1, 1], [0.0] * 9), ([1, -1, 1], [0, 1, 2, 3, 0, -1, 0.5, 0.5, 0]), ([-1, -1, -1], [0, 2, -1, 1, 0, 3, -2, 1, 0]))
    for s in (SURFACES[0], SURFACES[2], SURFACES[3], SURFACES[5]):
        for (sens, u), off in zip(pivots, (0.0, 0.5, 1.0)):
            requests.append('fl_b0_tols ' + numbers([*s, 1e-9, 0, 0, 1, 0, 0, 0.1, 1, 0, *LINE_X, 0, 1, *sens, *u, 1e-9,
                                                     1, 2, 0.5, off, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 2, 1, 1]))
            requests.append('fl_b0_tols ' + numbers([*s, 1e-7, 0.5, 0, 0, 1, 0.5, 0.1, 0.6, 0.8, *CIRCLE2, 0.2, 2.0, *sens, *u, 1e-3,
                                                     2, 0, 1, off, 2, 0, 1, 0, 2, 1, 1, 0, 0, 0, 1, 1, 1, 2, 0, 1]))
    for s, bounds in ((SURFACES[0], (-5, 5, -5, 5)), (SURFACES[2], (0, 2 * PI, -3, 3)), (SURFACES[3], (0, 2 * PI, -1.5, 1.5)), (SURFACES[2], (0, PI, -3, 3))):
        for g, f, l in ((SLANT, -5, 5), (guide(0, (-5, 0, 0.5), (1, 0, 0), (0, 0, 1), 0), 0, 10), (CIRCLE2, 0, 2 * PI)):
            for wc in (-2e100, 2e100, 1.0, 6.0):
                requests.append('fl_int_cs ' + numbers([*s, *bounds, *g, f, l, wc]))


def meetings(requests: list[str]) -> None:
    other = surface(0, (0, 0, 0), (1, 0, 0), (0, 1, 0))
    for s1, s2 in ((SURFACES[0], SURFACES[0]), (SURFACES[0], other), (SURFACES[2], SURFACES[0])):
        for (u1, u2) in ((0.5, 0.5), (1.4, 0.2), (0.0, 0.0)):
            requests.append('fl_computes_int_pc ' + numbers([*s1, 0, 0, 1, 0, -1, 3, *s2, 0.5, -1, 0, 1, -1, 3, u1, u2]))
    curves = (([0, 0, 0, 1, 0], 0, 4), ([0, 4, 0, -1, 0], 0, 4), ([0, 2, -1, 0, 1], 0, 2), ([1, 2, 0, 2, 0], 0, PI),
              ([0, 0, 0, 1, 0], 4, 6), ([0, 1, 0, 1, 0], 0, 4), ([1, 0, 0, 1, 0], 0, 2 * PI))
    for i, (c1, f1, l1) in enumerate(curves):
        for c2, f2, l2 in curves[i + 1:]:
            requests.append('fl_b0_inter2d ' + numbers([*c1, f1, l1, *c2, f2, l2, 1e-7]))


def topology(requests: list[str]) -> None:
    for solid in [*SOLIDS, (7, 10, 8, 0, 2)]:
        requests.append('fl_b0_topology ' + numbers(list(solid)))
    bezier = [4, 0, 0, 0, 1, 2, 0, 3, -1, 1, 4, 1, 0]
    kinked = [3, 0, 0, 0, 1, 0, 0, 1, 0.01, 0]
    for kind, g, poles, f, l in ((0, LINE_X, [0], 0, 0), (0, LINE_X, [0], -2, 3), (0, CIRCLE2, [0], 0, 0), (0, CIRCLE2, [0], 0.5, 4.0),
                                 (1, LINE_X, bezier, 0, 0), (1, LINE_X, bezier, 0.2, 0.9), (1, LINE_X, kinked, 0, 0),
                                 (0, guide(1, (0, 0, 0), (0, 0, 1), (1, 0, 0), 1e-3), [0], 0.1, 6.0)):
        requests.append('fl_b0_smooth ' + numbers([kind, *g, *poles, f, l]))


def append_builder0_requests(requests: list[str]) -> None:
    """Append the ChFi3d_Builder_0 request families."""
    scalars(requests)
    bounds(requests)
    boxes(requests)
    spines(requests)
    plane_cuts(requests)
    pcurves(requests)
    corner_bounds(requests)
    aretes(requests)
    common_points(requests)
    tolerances(requests)
    meetings(requests)
    topology(requests)
