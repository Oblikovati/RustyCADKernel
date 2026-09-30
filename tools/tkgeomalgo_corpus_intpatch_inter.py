"""Deterministic requests for IntPatch_Intersection and IntPatch_PrmPrmIntersection
(tools/oracle/tkgeomalgo_probe_intpatch_inter.cxx).

`ip_isingular` runs the static members of IntPatch_Intersection on surfaces with poles
(spheres, a cone to its apex) near and far from the other surface, and on flat cones
split at their apex. `ip_inter` runs the front end on every pair of surface kinds of the
IntCurveSurface catalog (bounded and infinite), on the nearly degenerated cones and tori,
with the three flags, with start points and from a start point, and on self-intersections.
`ip_prmprm` runs every Perform overload of the parametric intersection on Bezier,
B-spline, offset and revolution surfaces (crossing, tangent, closed, disjoint), with the
deflection options, and NewLine on the lines found.
"""
from __future__ import annotations
import math
from tkgeomalgo_corpus_intcurvesurface import (
    SURFACES, bez_s, bsp_s, bezier, cone, cyl, line, line_of, offs, plane, revol, sad, sphere, torus)

O, Z, X = [0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]
TWO_PI = 2.0 * math.pi
OWN = [-1, -1.0]
NONE = [0, -1.0]


def _singular_requests(requests: list[str]) -> None:
    ball = sad(sphere(O, Z, 1.0))
    near_pole = sad(plane([0.0, 0.0, 1.0 + 5.0e-6], Z), (-2.0, 2.0, -2.0, 2.0))
    on_pole = sad(plane([0.0, 0.0, 1.0], Z), (-2.0, 2.0, -2.0, 2.0))
    far = sad(plane([0.0, 0.0, 3.0], Z), (-2.0, 2.0, -2.0, 2.0))
    apex = sad(cone(O, Z, 0.5, 1.0), (0.0, TWO_PI, -1.0 / math.sin(0.5), 1.0))
    flat = sad(cone(O, Z, math.pi / 2.0 - 1.0e-9, 1.0), (0.0, TWO_PI, -2.0, 2.0))
    flat_out = sad(cone(O, Z, math.pi / 2.0 - 1.0e-9, 1.0), (0.0, TWO_PI, 1.0, 2.0))
    cases = [(ball, near_pole), (near_pole, ball), (ball, on_pole), (ball, far), (apex, near_pole),
             (apex, sad(plane([0.0, 0.0, -1.0 / math.tan(0.5) + 2.0e-6], Z), (-2.0, 2.0, -2.0, 2.0))),
             (flat, ball), (ball, flat), (flat_out, ball), (SURFACES['torus'], ball), (SURFACES['bezier_s'], ball)]
    for s1, s2 in cases:
        for tol in (1.0e-7, 1.0e-6):
            requests.append(line_of('ip_isingular', [*s1, *OWN, *s2, *OWN, tol]))
    requests.append(line_of('ip_isingular', [*ball, *NONE, *near_pole, *OWN, 1.0e-6]))


# The surfaces of the pairs: one of each kind, placed to intersect the others.
KINDS = {
    'plane': SURFACES['plane'],
    'cyl': SURFACES['cyl'],
    'cone': SURFACES['cone'],
    'sphere': SURFACES['sphere'],
    'torus': sad(torus([0.0, 0.0, 0.5], Z, 1.5, 0.6)),
    'bezier': SURFACES['bezier_s'],
    'bspline': SURFACES['bspline_s'],
    'rev': sad(revol(O, Z, bezier([[1.0, 0.0, -1.0], [2.0, 0.0, 0.0], [0.8, 0.0, 1.5]]))),
    'ext': sad(SURFACES['ext_bezier'][:-1], (0.0, 1.0, -2.0, 2.0)),
    'offset': sad(offs(0.2, bez_s([[[-2.0 + i, -2.0 + j, 0.3 * math.sin(i + j)] for j in range(4)] for i in range(4)]))),
}


def inter(requests, s1, s2, tail, d1=OWN, d2=OWN, tol=(1.0e-7, 1.0e-7), settol=None):
    st = [1, *settol] if settol else [0]
    requests.append(line_of('ip_inter', [*s1, *d1, *s2, *d2, *tol, *st, *tail]))


PLAIN = [0, 1, 0, 1]


def _pair_requests(requests: list[str]) -> None:
    names = list(KINDS)
    for i, a in enumerate(names):
        for b in names[i:]:
            inter(requests, KINDS[a], KINDS[b], PLAIN, settol=(1.0e-7, 1.0e-7, 0.05, 0.1))
    for a, b in (('plane', 'cyl'), ('cyl', 'sphere'), ('sphere', 'bezier'), ('bspline', 'bezier')):
        inter(requests, KINDS[a], KINDS[b], [0, 0, 1, 1], settol=(1.0e-7, 1.0e-7, 0.05, 0.1))
        inter(requests, KINDS[a], KINDS[b], [0, 1, 1, 0], settol=(1.0e-7, 1.0e-7, 0.05, 0.1))
        inter(requests, KINDS[a], KINDS[b], [4])
    # Infinite domains: two planes, a plane and an extrusion, two extrusions of lines.
    inter(requests, SURFACES['plane_nat'], sad(plane(O, [0.0, 0.6, 0.8])), PLAIN)
    inter(requests, SURFACES['ext_line'], sad(plane(O, [0.0, 0.0, 1.0])), PLAIN)
    inter(requests, SURFACES['ext_line'], SURFACES['ext_parab'], PLAIN)
    inter(requests, SURFACES['ext_parab'], KINDS['bezier'], PLAIN)
    inter(requests, SURFACES['cyl_nat'], SURFACES['plane_nat'], PLAIN)


def _special_requests(requests: list[str]) -> None:
    thin = sad(cone(O, Z, 0.01, 1.0), (0.0, TWO_PI, -1.0, 1.0))
    wide = sad(cone(O, Z, 1.56, 1.0), (0.0, TWO_PI, -0.5, 0.5))
    wide2 = sad(cone([0.0, 0.0, 0.0], Z, 1.565, 1.0), (0.0, TWO_PI, -0.5, 0.5))
    for s1, s2 in ((thin, KINDS['plane']), (KINDS['plane'], thin), (wide, KINDS['plane']), (wide, wide2),
                   (thin, SURFACES['sphere']), (thin, KINDS['cyl']), (KINDS['torus'], KINDS['plane']),
                   (KINDS['torus'], sad(torus([0.0, 0.0, 0.5], Z, 1.5, 0.6))), (KINDS['torus'], SURFACES['sphere']),
                   (sad(torus([0.0, 0.0, 0.5], X, 1.5, 0.6)), KINDS['cyl'])):
        inter(requests, s1, s2, PLAIN)
        inter(requests, s1, s2, [1, 0, 1, 0, 1])
    pts = [2, 0.5, 0.5, 0.5, 0.5, 1.0, 1.0, 1.0, 1.0]
    for a, b in (('bezier', 'bspline'), ('cyl', 'plane'), ('bezier', 'sphere')):
        inter(requests, KINDS[a], KINDS[b], [1, *pts, 1, 0, 1])
        inter(requests, KINDS[a], KINDS[b], [2, 0.5, 0.5, 0.5, 0.5])
    for name in ('plane', 'bezier', 'offset', 'ext'):
        inter(requests, KINDS[name], KINDS[name], [3])
        inter(requests, KINDS[name], KINDS[name], [5])
    inter(requests, KINDS['cyl'], KINDS['plane'], PLAIN, tol=(1.0e-12, 2.0))


def prmprm(requests, s1, s2, tail, d1=OWN, d2=OWN, tols=(1.0e-7, 1.0e-7, 0.05, 0.1), new_lines=()):
    body = [*s1, *d1, *s2, *d2, *tols, *tail, len(new_lines), *[v for nl in new_lines for v in nl]]
    requests.append(line_of('ip_prmprm', body))


def _prmprm_requests(requests: list[str]) -> None:
    bz, bs, off, rev = KINDS['bezier'], KINDS['bspline'], KINDS['offset'], KINDS['rev']
    tangent = sad(bez_s([[[-2.0 + i, -2.0 + j, 0.4 * math.sin(i + 2 * j) + 0.0] for j in range(4)] for i in range(4)]))
    far = sad(bez_s([[[-2.0 + i, -2.0 + j, 5.0] for j in range(4)] for i in range(4)]))
    closed = sad(revol(O, Z, line([1.0, 0.0, -1.0], Z)), (0.0, TWO_PI, 0.0, 2.0))
    pairs = [(bz, bs), (bs, off), (off, rev), (bz, rev), (bz, tangent), (bz, far), (closed, bz), (rev, closed)]
    for s1, s2 in pairs:
        prmprm(requests, s1, s2, [0, 1], new_lines=[(1, 1, 4, 10), (1, 3, 2, 5), (9, 1, 2, 5)])
        prmprm(requests, s1, s2, [0, 0], tols=(1.0e-6, 1.0e-6, 0.2, 0.3))
        prmprm(requests, s1, s2, [4, 0, 0, 0, 0])
        prmprm(requests, s1, s2, [4, 6, 6, 0, 0])
        prmprm(requests, s1, s2, [4, 0, 0, 5, 7])
        prmprm(requests, s1, s2, [4, 5, 5, 5, 5])
        prmprm(requests, s1, s2, [1, 3, 0.2, 0.3, 0.4, 0.5, 0.5, 0.5, 0.5, 0.5, 0.0, 0.0, 1.0, 1.0])
        prmprm(requests, s1, s2, [2, 0.5, 0.5, 0.5, 0.5])
    for s in (bz, off, rev, closed):
        prmprm(requests, s, s, [3])
        prmprm(requests, s, s, [5, 6, 6])
    prmprm(requests, bz, bs, [1, 0])
    prmprm(requests, bz, bs, [0, 1], tols=(1.0e-7, 1.0e-7, 0.001, 0.001))


def append_intpatch_inter_requests(requests: list[str]) -> None:
    """The front-end and PrmPrm requests."""
    _singular_requests(requests)
    _pair_requests(requests)
    _special_requests(requests)
    _prmprm_requests(requests)
