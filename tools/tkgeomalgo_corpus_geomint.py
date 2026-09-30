"""Deterministic requests for GeomInt, ApproxInt and GeomAPI_IntSS
(tools/oracle/tkgeomalgo_probe_geomint*.cxx).

`gi_intss` runs GeomInt_IntSS on every pair of surface kinds (planes, quadrics, tori,
Bezier, B-spline, offset, revolution and extrusion surfaces, and trimmed ones), with and
without the approximation and the 2d curves, through each Perform overload (the
constructor, from a start point, on adaptors with trimmed domains, twice, and on one
surface); `gapi_intss` runs GeomAPI_IntSS. `gi_wlapprox` approximates the walking lines of
intersections with every option of SetParameters (tolerances, degrees, iterations, points
per interval, tangency, parametrization), on and without the surfaces, over the whole
lines, sub-ranges and the default range. `gi_mline` and `gi_sv` drive the multi-line and
the PrmPrm and ImpPrm SvSurfaces (values, tangents, point insertions, the solver switch,
the cache of the last two results). `gi_knots` and `gi_partype` run ApproxInt_KnotTools,
`gi_lconst`, `gi_lines` and `gi_ltool` the line constructor and the line tool (with the
decomposition at the seams), and the static members follow (`gi_pcurves`, `gi_trim`,
`gi_mkbsp`, `gi_rline`, `gi_adjust`, `gi_porient`).
"""
from __future__ import annotations
import math
from tkgeomalgo_corpus_intcurvesurface import (
    bez_s, bezier, bsp_s, circle, cone, cyl, ellipse, extr, hyperbola, line, line_of, offs, parabola, plane,
    revol, rtrim, sad, sphere, torus)

TWO_PI = 2.0 * math.pi
O, X, Y, Z = [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]
GRID = [[[-2.0 + i, -2.0 + j, 0.4 * math.sin(i + 2 * j)] for j in range(4)] for i in range(4)]
GRID2 = [[[-2.0 + 1.3 * i, -2.0 + j, 0.3 + 0.5 * math.cos(i - j)] for j in range(4)] for i in range(4)]

# Geom surfaces (ga_ics_geom_surface) of every kind.
GEOM = {
    'plane': plane([0.0, 0.0, 0.3], Z),
    'plane_tilt': plane([0.1, 0.0, 0.0], [1.0, 0.2, 0.4]),
    'plane_x': plane([0.2, 0.0, 0.0], X),
    'cyl': cyl(O, Z, 1.0),
    'cyl_x': cyl([0.0, 0.0, 0.2], X, 0.7),
    'cyl_par': cyl([0.5, 0.0, 0.0], Z, 0.8),
    'cone': cone([0.0, 0.0, -1.0], Z, 0.4, 0.8),
    'cone_y': cone([0.0, -1.0, 0.0], Y, 0.5, 0.6),
    'sphere': sphere(O, Z, 1.5),
    'sphere2': sphere([1.0, 0.2, 0.1], Z, 1.2),
    'torus': torus([0.0, 0.0, 0.5], Z, 1.5, 0.6),
    'bezier': bez_s(GRID),
    'bezier2': bez_s(GRID2),
    'bspline': bsp_s(2, 2, GRID, [0.0, 1.0, 2.0], [3, 1, 3], [0.0, 0.5, 1.0], [3, 1, 3]),
    'bspline2': bsp_s(3, 2, GRID2, [0.0, 1.0], [4, 4], [0.0, 0.4, 1.0], [3, 1, 3]),
    'offset': offs(0.2, bez_s([[[-2.0 + i, -2.0 + j, 0.3 * math.sin(i + j)] for j in range(4)] for i in range(4)])),
    'rev': revol(O, Z, bezier([[1.0, 0.0, -1.0], [2.0, 0.0, 0.0], [0.8, 0.0, 1.5]])),
    'rev_line': revol(O, Z, line([1.0, 0.0, 0.0], [0.3, 0.0, 1.0])),
    'ext': extr([0.0, 0.2, 1.0], bezier([[-2.0, 0.0, 0.0], [0.0, 2.0, 0.0], [2.0, -1.0, 0.0]])),
    'ext_circle': extr(Z, circle([0.3, 0.0, 0.0], Z, 0.9)),
    'trim_cyl': rtrim(0.5, 3.0, -1.0, 1.0, cyl(O, Z, 1.0)),
    'trim_plane': rtrim(-1.0, 1.5, -0.5, 2.0, plane([0.0, 0.0, 0.3], Z)),
    'trim_sphere': rtrim(0.0, 4.0, -1.0, 1.2, sphere(O, Z, 1.5)),
    'trim_cone': rtrim(0.0, TWO_PI, -1.0, 2.0, cone([0.0, 0.0, -1.0], Z, 0.4, 0.8)),
    'trim_torus': rtrim(0.0, 3.0, 0.0, 5.0, torus([0.0, 0.0, 0.5], Z, 1.5, 0.6)),
}

# The pairs of `gi_intss`, one of each kind combination that intersects.
PAIRS = [
    ('plane', 'plane_tilt'), ('plane', 'cyl'), ('plane', 'cyl_x'), ('plane', 'cone'), ('plane', 'sphere'),
    ('plane', 'torus'), ('plane', 'bezier'), ('plane', 'bspline'), ('plane', 'offset'), ('plane', 'rev'),
    ('plane', 'ext'), ('plane_x', 'ext_circle'), ('plane_tilt', 'rev_line'), ('cyl', 'cyl_x'), ('cyl', 'cyl_par'),
    ('cyl', 'cone_y'), ('cyl', 'sphere2'), ('cyl', 'torus'), ('cyl', 'bezier'), ('cyl', 'rev'), ('cone', 'cone_y'),
    ('cone', 'sphere'), ('sphere', 'sphere2'), ('sphere', 'torus'), ('sphere', 'bspline'), ('torus', 'bezier'),
    ('bezier', 'bezier2'), ('bezier', 'bspline2'), ('bspline', 'bspline2'), ('bezier', 'offset'), ('bspline', 'rev'),
    ('offset', 'rev'), ('ext', 'bezier2'), ('ext', 'cyl'), ('rev_line', 'bezier2'), ('trim_cyl', 'trim_plane'),
    ('trim_sphere', 'plane_tilt'), ('trim_cone', 'plane_tilt'), ('trim_torus', 'plane'), ('trim_cyl', 'bezier'),
]
FLAGS = [(1, 0, 0), (1, 1, 1), (0, 1, 1), (1, 1, 0), (0, 0, 1)]


def intss(requests, a, b, tol, flags, tail):
    requests.append(line_of('gi_intss', [*GEOM[a], *GEOM[b], tol, *flags, *tail]))


def _intss_requests(requests: list[str]) -> None:
    for a, b in PAIRS:
        for flags in FLAGS:
            intss(requests, a, b, 1.0e-7, flags, [1])
        intss(requests, a, b, 1.0e-7, (1, 1, 1), [0])
        intss(requests, b, a, 1.0e-6, (1, 1, 1), [1])
    for a, b in (('bezier', 'bezier2'), ('plane', 'bezier'), ('cyl', 'bezier'), ('sphere', 'bspline')):
        intss(requests, a, b, 1.0e-7, (1, 1, 1), [2, 0.5, 0.5, 0.5, 0.5])
        intss(requests, a, b, 1.0e-7, (1, 1, 1), [2, 50.0, 50.0, 0.5, 0.5])
        intss(requests, a, b, 1.0e-7, (1, 1, 1), [5])
    bounded = {'plane': (-1.5, 1.5, -1.5, 1.5), 'cyl': (0.0, 4.0, -1.0, 1.0), 'bezier': (0.1, 0.9, 0.2, 0.8),
               'sphere': (0.0, 5.0, -1.0, 1.0), 'bspline': (0.2, 1.8, 0.1, 0.9)}
    for a, b in (('plane', 'cyl'), ('plane', 'bezier'), ('cyl', 'bezier'), ('sphere', 'bspline'), ('bezier', 'bspline')):
        ad = [1, *bounded[a], 1, *bounded[b]]
        intss(requests, a, b, 1.0e-7, (1, 1, 1), [3, *ad])
        intss(requests, a, b, 1.0e-7, (0, 1, 1), [3, 0, 0])
        intss(requests, a, b, 1.0e-7, (1, 1, 1), [4, *ad, 0.5, 0.5, 0.5, 0.5])
    for a in ('plane', 'bezier', 'offset', 'ext', 'cyl'):
        intss(requests, a, a, 1.0e-7, (1, 1, 1), [6])
    for a, b in PAIRS[:12]:
        requests.append(line_of('gapi_intss', [*GEOM[a], *GEOM[b], 1.0e-7, 0]))
    for mode in (1, 2):
        requests.append(line_of('gapi_intss', [*GEOM['bezier'], *GEOM['bspline2'], 1.0e-7, mode]))


# Adaptor pairs (ga_ics_surface) whose intersections have walking lines.
WALK = {
    'bez_bsp': (sad(GEOM['bezier']), sad(GEOM['bspline2'])),
    'bez_bez': (sad(GEOM['bezier']), sad(GEOM['bezier2'])),
    'bez_off': (sad(GEOM['bezier']), sad(GEOM['offset'])),
    'bsp_rev': (sad(GEOM['bspline']), sad(GEOM['rev'])),
    'plane_bez': (sad(GEOM['plane'], (-3.0, 3.0, -3.0, 3.0)), sad(GEOM['bezier'])),
    'bez_plane': (sad(GEOM['bezier']), sad(GEOM['plane_tilt'], (-4.0, 4.0, -4.0, 4.0))),
    'cyl_bez': (sad(GEOM['cyl'], (0.0, TWO_PI, -2.0, 2.0)), sad(GEOM['bezier'])),
    'bez_sphere': (sad(GEOM['bezier']), sad(GEOM['sphere'])),
    'cone_bsp': (sad(GEOM['cone'], (0.0, TWO_PI, -2.0, 3.0)), sad(GEOM['bspline'])),
    'rev_cyl': (sad(GEOM['rev']), sad(GEOM['cyl_x'], (0.0, TWO_PI, -3.0, 3.0))),
    'torus_bez': (sad(GEOM['torus']), sad(GEOM['bezier2'])),
    # A Bezier surface with a degenerated edge (v = 0) through a plane, and a cone whose
    # apex is on a Bezier surface: the singular processing of both SvSurfaces.
    'plane_degbez': (sad(plane([0.0, 0.0, 0.1], [0.0, 0.2, 1.0]), (-3.0, 3.0, -3.0, 3.0)),
                     sad(bez_s([[[0.0, 0.0, -0.5], [-1.0 + 0.7 * i, -1.0, 0.0], [-1.0 + 0.7 * i, 0.0, 0.5],
                                 [-1.0 + 0.7 * i, 1.0, 1.0]] for i in range(4)]))),
    'apex_bez': (sad(cone(O, Z, 0.5, 0.0), (0.0, TWO_PI, -1.0, 2.0)),
                 sad(bez_s([[[-1.0 + 0.7 * i, -1.0 + 0.7 * j, 0.1 * (i - j)] for j in range(4)] for i in range(4)]))),
}


def walk(name, tol=1.0e-7):
    s1, s2 = WALK[name]
    return [*s1, *s2, tol]


SETTINGS = [
    [0],
    [1, 1.0e-7, 1.0e-7, 4, 8, 0, 30, 1, 0],
    [1, 1.0e-5, 1.0e-5, 2, 6, 5, 10, 1, 1],
    [1, 1.0e-3, 1.0e-4, 3, 5, 2, 5, 0, 2],
    [1, 1.0e-7, 1.0e-7, 4, 8, 0, 30, 0, 1],
    [1, 1.0e-6, 1.0e-6, 1, 3, 3, 4, 1, 0],
]
SPACES = [(1, 1, 1), (1, 0, 0), (0, 1, 0), (0, 0, 1), (1, 1, 0), (0, 1, 1)]


def _wlapprox_requests(requests: list[str]) -> None:
    for name in WALK:
        for settings in SETTINGS:
            for spaces in SPACES[:3]:
                requests.append(line_of('gi_wlapprox', [*walk(name), *settings, *spaces, 0, 2, 0, 0]))
        requests.append(line_of('gi_wlapprox', [*walk(name), *SETTINGS[1], 1, 1, 1, 1, 2, 0, 0]))
        requests.append(line_of('gi_wlapprox', [*walk(name), *SETTINGS[1], 0, 1, 1, 0, 1, 1, 2, 9]))
        requests.append(line_of('gi_wlapprox', [*walk(name), *SETTINGS[2], 1, 1, 1, 0, 1, 2]))
        for spaces in SPACES[3:]:
            requests.append(line_of('gi_wlapprox', [*walk(name), *SETTINGS[1], *spaces, 0, 1, 0]))


# The SvSurfaces kinds of the pairs: 1 PrmPrm, 2 the quadric of S1 first, 3 the quadric of S2.
QUADRIC_KINDS = {'plane_bez': (0, 1, 2), 'cyl_bez': (0, 1, 2), 'bez_sphere': (0, 1, 3), 'bez_plane': (0, 1, 3),
                 'plane_degbez': (0, 1, 2), 'apex_bez': (0, 1, 2), 'cone_bsp': (0, 1, 2), 'torus_bez': (0, 1, 2)}


def _mline_requests(requests: list[str]) -> None:
    offs0 = [0.0] * 7
    offs1 = [2.0, 1.0, -0.5, 0.25, 0.5, -1.0, 0.75]
    # nops, then MakeMLBetween (1 low high nb), MakeMLOneMorePoint (2 low high bad), Dump (3).
    ops = [5, 1, 1, 6, 6, 1, 2, 8, 10, 2, 3, 6, 5, 2, 1, 4, 2, 3]
    for name in ('bez_bsp', 'bez_bez', 'plane_bez', 'cyl_bez', 'bez_sphere', 'bez_off'):
        # The ImpPrm SvSurfaces take the quadric of a quadric surface only: an undefined quadric
        # leaves the parameters of SeekPoint uninitialized natively.
        kinds = QUADRIC_KINDS.get(name, (0, 1))
        for kind in kinds:
            for layout in ([1, 2, 1, 1, *offs0, 1], [1, 1, 1, 0, *offs1, 1], [0, 1, 0, 1, *offs0, 0],
                           [1, 0, 0, 0, *offs1, 1], [0, 2, 1, 1, *offs1, 0]):
                requests.append(line_of('gi_mline', [*walk(name), kind, 1, *layout, 0, 3, 1, 2, 3, *ops]))
            requests.append(line_of('gi_mline', [*walk(name), kind, 1, 1, 2, 1, 1, *offs0, 1, 1, 2, 9, 0, 2, 1, 2, 9, 4, 3]))
            requests.append(line_of('gi_mline', [*walk(name), kind, 2, 1, 2, 1, 1, *offs0, 1, 2, 1, 0, 1, 1, 0, 0, 1]))


def _sv_requests(requests: list[str]) -> None:
    def seq(u):
        a, b, c, d = u
        return [0, a, b, c, d, 0, a, b, c, d, 1, a, b, c, d, 3, a + 0.01, b, c, d, 0, a, b, c, d, 4, a, b, c, d,
                5, a + 0.02, b, c, d, 2, a, b, c, d, 0, a, b, c, d, 6, 0, 0, a, b, c, d, 2, a, b, c, d, 6, 1,
                0, a + 0.3, b - 0.2, c, d]
    starts = {'bez_bsp': (0.5, 0.5, 0.5, 0.5), 'plane_bez': (0.1, 0.2, 0.5, 0.5), 'cyl_bez': (0.5, 0.3, 0.5, 0.5),
              'bez_sphere': (0.5, 0.5, 1.0, 0.2), 'bez_bez': (0.3, 0.6, 0.4, 0.5),
              'plane_degbez': (0.0, 0.0, 0.4, 0.0), 'apex_bez': (0.0, 0.0, 0.47, 0.47),
              'cone_bsp': (1.0, 0.5, 0.5, 0.5), 'torus_bez': (0.3, 0.2, 0.6, 0.6)}
    extra = {'bez_sphere': (0.5, 0.5, 0.0, 1.5), 'plane_bez': (0.1, 0.2, 1.0, 1.0), 'apex_bez': (1.0, 0.0, 0.0, 0.0),
             'plane_degbez': (1.0, 1.0, 0.0, 1.0)}
    for name, u in starts.items():
        for kind in QUADRIC_KINDS.get(name, (1,)):
            if kind == 0:
                continue
            ops = seq(u) + ([7, *u, 7, u[0] + 7.0, u[1], u[2] - 7.0, u[3], 7, -20.0, u[1], 30.0, u[3]] if kind != 1 else [])
            count = _count_ops(ops)
            requests.append(line_of('gi_sv', [*walk(name), kind, count, *ops]))
            if name in extra:
                ops = seq(extra[name])
                requests.append(line_of('gi_sv', [*walk(name), kind, _count_ops(ops), *ops]))


def _count_ops(ops: list[float]) -> int:
    i, n = 0, 0
    while i < len(ops):
        i += 2 if ops[i] == 6 else 5
        n += 1
    return n


def _knots_requests(requests: list[str]) -> None:
    helix = [[math.cos(0.3 * t), math.sin(0.3 * t), 0.1 * t, 0.05 * t, 0.2 * math.sin(t), 0.1 * t * t, t]
             for t in range(40)]
    kinked = [[t if t < 20 else 20.0, 0.0 if t < 20 else t - 20.0, 0.0, t * 0.1, 0.0, 0.0, 1.0] for t in range(45)]
    line_pts = [[0.5 * t, 0.25 * t, 0.0, t, 0.0, 0.0, t] for t in range(12)]
    for pts in (helix, kinked, line_pts):
        n = len(pts)
        pars = [i / (n - 1) for i in range(n)]
        for dim, cols in ((3, slice(0, 3)), (2, slice(3, 5)), (7, slice(0, 7))):
            coords = [c for p in pts for c in p[cols]]
            requests.append(line_of('gi_knots', [0, dim, n, *coords, 1, *pars]))
        for flags in ((1, 0, 0), (0, 1, 0), (0, 0, 1), (1, 1, 1), (0, 0, 0)):
            for mn in (2, 5, 30):
                flat = [c for p in pts for c in p]
                requests.append(line_of('gi_knots', [1, n, 1, *flat, *pars, *flags, mn]))
    requests.append(line_of('gi_knots', [0, 3, 2, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1, 0.0, 1.0]))
    requests.append(line_of('gi_knots', [1, 1, 3, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.5, 1, 0, 0, 3]))
    for name in ('bez_bsp', 'plane_bez', 'cyl_bez', 'bez_sphere', 'torus_bez'):
        for flags in ((1, 1, 1), (0, 1, 0), (1, 0, 1)):
            requests.append(line_of('gi_partype', [*walk(name), *flags, 2, 0, 1, 2, 3]))


def _lines_requests(requests: list[str]) -> None:
    for name in WALK:
        requests.append(line_of('gi_lines', walk(name)))
    for a, b in (('plane', 'cyl'), ('cyl', 'sphere2'), ('cone', 'cone_y'), ('plane', 'torus'), ('cyl', 'cyl_par'),
                 ('sphere', 'sphere2'), ('plane', 'plane_tilt')):
        requests.append(line_of('gi_lines', [*sad(GEOM[a], None), *sad(GEOM[b], None), 1.0e-7]))
    periodic = {'cyl_bez_seam': (sad(cyl(O, Z, 1.0), (0.0, TWO_PI, -2.0, 2.0)),
                                 sad(bez_s([[[-2.0 + i * 1.3, -0.2 + 0.1 * j, -1.5 + j] for j in range(4)] for i in range(4)]))),
                'sphere_bsp': (sad(sphere(O, Z, 1.2)), sad(GEOM['bspline'])),
                'torus_plane': (sad(torus(O, Z, 1.5, 0.6)), sad(plane([0.0, 0.0, 0.2], [0.1, 0.0, 1.0]), (-3.0, 3.0, -3.0, 3.0)))}
    for s1, s2 in periodic.values():
        requests.append(line_of('gi_lines', [*s1, *s2, 1.0e-7]))


def _lconst_requests(requests: list[str]) -> None:
    from tkgeomalgo_corpus_intpatch import free_point, p_arc, p_para, p_params, p_value, point
    from tkgeomalgo_corpus_intpatch_wltool import (
        OWN, S_C2, S_CYL, S_XY, S_XZ, TOUCH, TYPES, UNDECIDED, along_u, at, domain, gline, pts, straight, vtx,
        wavy, wline)
    def lc(line, s1=S_XY, s2=S_XZ, d1=domain(1, 2, 3), d2=domain()):
        requests.append(line_of('gi_lconst', [*s1, *s2, *d1, *d2, *line]))
    def ltool(line):
        requests.append(line_of('gi_ltool', line))
    rows = straight(12, step=0.3, x0=0.2)
    mid = [at(0.2, 1.0), at(1.4, 5.0), at(rows[-1][0], 12.0)]
    walking = [wline(rows, vertices=mid), wline(rows, vertices=mid, trans=TYPES),
               wline(rows, vertices=[at(0.2, 1.0), at(0.2, 1.0), at(3.5, 12.0)]),
               wline(rows, vertices=[at(0.2, 1.0), at(0.5, 2.0), at(0.8, 3.0)]),
               wline(rows, vertices=[at(0.2, 1.0), at(0.5, 2.0), at(0.8, 3.0)], way=1),
               wline(rows, vertices=[at(0.2, 1.0), at(0.5, 2.0), at(0.8, 3.0)], way=2),
               wline([[x, 0.0, 0.0, x + 5.0, 5.0, x, 0.0] for x in [0.2 + 0.3 * i for i in range(12)]],
                     vertices=[at(0.2, 1.0, u1=5.2), at(3.5, 12.0, u1=8.5)]),
               wline(rows), wline(rows, vertices=[at(0.2, 1.5), at(3.5, 11.5)])]
    for w in walking:
        lc(w)
        lc(w, s1=S_CYL)
        ltool(w)
    plane_ext = sad(extr(Z, bezier([[-2.0, 0.0, 0.0], [0.0, 2.0, 0.0], [2.0, -1.0, 0.0]])))
    parts = [at(0.2, 1.0), at(0.5, 2.0), at(0.8, 3.0), at(1.1, 4.0), at(1.4, 5.0), at(1.7, 6.0)]
    lc(wline(rows, vertices=parts), s2=plane_ext)
    lc(wline(rows, vertices=parts), s1=plane_ext, s2=S_XY)
    def rline(vertices, arcs, points=None, trans=UNDECIDED):
        return [1, *trans, *([1, *pts(points)] if points else [0]), *vtx(*vertices), *arcs]
    vs = [free_point(1.0, 0.5 * k, 0.0, 1.0, 0.5 * k, 0.5 * k, 0.1, 0.5 * k) for k in (1, 4, 9, 12)]
    both = point(p_value(1.0, 0.5, 0.0), p_params(1.0, 0.5, 0.5, 0.1), p_para(2.0), p_arc(1, 1, 0.5, 1, 2), p_arc(0, 2, 0.5, 2, 1))
    touch = point(p_value(1.0, 2.0, 0.0), p_params(1.0, 2.0, 2.0, 0.1), p_para(4.0), p_arc(1, 1, 2.0, 5, 5), p_arc(0, 2, 2.0, 0, 0))
    for arcs in ((1, 1), (1, 0), (0, 1)):
        for vertices in (vs, [vs[0], both, vs[2]], [both, touch, vs[3]], [touch, vs[1]], []):
            line = rline(vertices, arcs, trans=TYPES)
            lc(line)
            lc(line, d1=domain(tol=1.0e-7), d2=domain(3, tol=1.0e-6))
            ltool(line)
    lc(rline(vs, (1, 0), points=along_u(4, 1.0, 0.5, 6.0)))
    lin = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]
    lv = [free_point(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, p) for p in (-1.0, 1.5, 3.0, 7.0)]
    p_inf1, p_inf2 = sad(plane(O, Z)), sad(plane(O, Y))
    circ = [0.0, 0.0, 0.5, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0]
    cv = [free_point(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, p) for p in (0.5, 2.0, 4.0, 5.5)]
    sph = sad(sphere(O, Z, 1.2))
    con = sad(cone(O, Z, 0.5, 1.0), [0.0, TWO_PI, -3.0, 3.0])
    for line, s1, s2 in ((gline(0, lin, (0.0, 0.0), lv), p_inf1, p_inf2), (gline(0, lin, (0.0, 0.0), []), p_inf1, p_inf2),
                         (gline(0, lin, (0.0, 0.0), [lv[0], lv[0]]), p_inf1, p_inf2),
                         (gline(1, circ, (1.0, 0.0), cv), S_CYL, sph), (gline(2, circ, (1.5, 0.8), cv), S_CYL, sph),
                         (gline(1, circ, (1.0, 0.0), []), S_CYL, sph), (gline(1, circ, (1.0e-9, 0.0), cv), S_CYL, sph),
                         (gline(1, circ, (1.0, 0.0), [cv[0], cv[0], cv[3]]), S_CYL, sph),
                         (gline(1, circ, (1.0, 0.0), [free_point(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, p) for p in (-0.5, 6.0, 6.28, 12.0)]), S_CYL, sph),
                         (gline(3, circ, (0.5, 0.0), cv), con, p_inf2), (gline(4, circ, (2.0, 1.0), cv), con, p_inf2),
                         (gline(4, circ, (2.0, 1.0), []), con, p_inf2), (gline(0, lin, (0.0, 0.0), lv), S_C2, p_inf2)):
        lc(line, s1=s1, s2=s2, d1=domain(), d2=domain())
        lc(line, s1=s1, s2=s2, d1=domain(1, 2, tol=1.0e-7), d2=domain(tol=1.0e-7))
        ltool(line)
    cylinder = sad(cyl(O, Z, 1.0), [0.0, TWO_PI, -3.0, 3.0])
    sph2 = sad(sphere([0.5, 0.0, 0.0], Z, 1.2))
    curve = [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.5, 0.0, 0.0, 1.2, 1.0e-8]
    av = [free_point(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, p) for p in (4.7, 5.5, 6.5, 7.9)]
    for vertices in (av, [av[0], av[0]], [], av[:1]):
        line = [3, *curve, *UNDECIDED, *vtx(*vertices)]
        lc(line, s1=cylinder, s2=sph2, d1=domain(), d2=domain())
        lc(line, s1=S_C2, s2=sph2, d1=domain(), d2=domain())
        ltool(line)


def _statics_requests(requests: list[str]) -> None:
    from tkgeomalgo_corpus_geom2d import c_bspline, c_circle, c_line, c_trimmed
    from tkgeomalgo_corpus_intpatch_wltool import S_CYL, S_XY, at, straight, wavy, wline
    from tkgeomalgo_corpus_intcurvesurface import circle as c3_circle, ellipse as c3_ellipse
    curves = {'line': line([0.0, 0.0, 0.3], [1.0, 0.2, 0.0]), 'circle': c3_circle([0.0, 0.0, 0.3], Z, 1.0),
              'circle_s': c3_circle(O, X, 1.5), 'ellipse': c3_ellipse([0.0, 0.0, 0.3], Z, 1.2, 0.8),
              'bez': bezier([[-1.0, -1.0, 0.3], [0.0, 1.0, 0.3], [1.0, -0.5, 0.3]])}
    surfaces = {'plane': GEOM['plane'], 'cyl': cyl(O, Z, 1.0), 'sphere': sphere(O, Z, 1.5), 'bezier': GEOM['bezier'],
                'torus': torus(O, Z, 1.0, 0.3), 'trim_cyl': GEOM['trim_cyl'], 'cone': cone(O, Z, 0.5, 1.0)}
    cases = [('line', 'plane', 0.0, 2.0), ('circle', 'plane', 0.0, TWO_PI), ('circle', 'cyl', 0.0, TWO_PI),
             ('circle', 'cyl', 1.0, 4.0), ('circle_s', 'sphere', 0.0, TWO_PI), ('circle_s', 'sphere', 0.5, 2.5),
             ('ellipse', 'plane', 0.3, 5.0), ('bez', 'plane', 0.0, 1.0), ('line', 'plane', 0.5, 0.5 + 1.0e-9),
             ('line', 'plane', 0.5, 0.5 + 1.0e-14), ('line', 'plane', 0.5, 0.5), ('circle', 'cyl', 1.0, 1.0 + 1.5e-9),
             ('circle', 'torus', 0.0, TWO_PI), ('circle', 'trim_cyl', 0.6, 2.9), ('circle', 'cone', 0.0, 3.0)]
    # A curve whose projections both fail (a line off a Bezier surface) crashes natively: the
    # null 2d curve is dereferenced (docs/tkgeomalgo-progress.md); it is left out.
    for c, s, f, l in cases:
        base = [*curves[c], *surfaces[s], f, l, 1.0e-7]
        requests.append(line_of('gi_pcurves', [*base, 0, 0]))
        requests.append(line_of('gi_pcurves', [*base, 1, -1.0, 8.0, -1.0, 2.0, 0]))
        requests.append(line_of('gi_pcurves', [*base, 1, 7.0, 13.0, -1.0, 2.0, 0]))
    requests.append(line_of('gi_pcurves', [*curves['line'], *surfaces['plane'], 0.0, 1.0, 1.0e-7, 0, 1, *c_line(0.0, 0.0, 1.0, 0.0)]))
    requests.append(line_of('gi_pcurves', [*curves['circle'], *surfaces['cyl'], 0.0, TWO_PI, 1.0e-7, 1, -7.0, -1.0, -5.0, 5.0, 0]))
    seg = c_line(-1.0, 0.5, 1.0, 0.2)
    arc = c_circle(0.5, 0.5, 0.8)
    bsp = c_bspline(2, [0.0, 1.0, 2.0], [3, 1, 3], [(-1.0, 0.0), (0.0, 2.0), (1.0, -1.0), (2.0, 1.5)])
    trimmed = c_trimmed(-2.0, 3.0, seg)
    for c1, c2 in ((seg, arc), (arc, None), (None, bsp), (trimmed, trimmed), (bsp, arc), (None, None)):
        body = [*([1, *c1] if c1 else [0]), *([1, *c2] if c2 else [0])]
        for boxes in ([0.0, 0.0, 1.0, 1.0, -0.5, -0.5, 1.5, 1.5], [0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0],
                      [-1.0e101, 0.0, 1.0, 1.0, 0.2, 0.2, 0.2, 0.2]):
            requests.append(line_of('gi_trim', [*body, *boxes, 2, 0.0, 1.0]))
            requests.append(line_of('gi_trim', [*body, *boxes, 3, 1.0, 0.5, 0.5]))
    rows = wavy(10)
    for a, b in ((1, 10), (3, 7), (1, 1), (0, 5), (5, 4), (8, 12), (1, 2)):
        requests.append(line_of('gi_mkbsp', [*wline(rows), a, b]))
    from tkgeomalgo_corpus_intpatch import free_point
    from tkgeomalgo_corpus_intpatch_wltool import UNDECIDED, TYPES, domain, vtx
    vs = [free_point(1.0, 0.5 * k, 0.0, 1.0, 0.5 * k, 0.5 * k, 0.1, 0.5 * k) for k in (1, 4, 9, 12)]
    for arcs in ((1, 0), (0, 2), (3, 3), (0, 0)):
        for fl in ((0, 0), (1, 0), (0, 1), (1, 1)):
            rl = [1, *TYPES, 0, *vtx(*vs), *arcs]
            requests.append(line_of('gi_rline', [*S_XY, *sad(plane(O, [0.0, 0.3, 1.0]), (-5.0, 5.0, -5.0, 5.0)), *rl, *fl]))
            requests.append(line_of('gi_rline', [*S_CYL, *S_XY, *rl, *fl]))
    degenerate = [1, *UNDECIDED, 0, *vtx(*vs[:2]), 1, 0]
    requests.append(line_of('gi_rline', [*sad(sphere(O, Z, 1.0), (0.0, 4.0, 0.9, 1.6)), *S_XY, *degenerate, 0, 0]))
    adj = [(-7.0, 0.0, 6.0, 6.0, 0.0), (13.0, 0.0, 6.0, 6.0, 0.0), (3.0, 0.0, 6.0, 6.0, 0.0), (-0.1, 0.0, 6.0, 6.0, 0.5),
           (6.05, 0.0, 6.0, 6.0, 0.1), (-1.0e-12, 0.0, TWO_PI, TWO_PI, 1.0e-9), (1.0e10, 0.0, TWO_PI, TWO_PI, 0.0),
           (-3.0, 0.0, 1.0, 0.0, 0.0), (2.0, 0.0, 1.0, -1.0, 0.0), (1.0e308, 0.0, 1.0, 1.0e-300, 0.0),
           (-6.0, -1.0, 1.0, 2.0, 0.0)]
    requests.append(line_of('gi_adjust', [len(adj), *[v for t in adj for v in t]]))
    for p, o in ((0.5, (0, 1, 2, 3)), (-1.0, (3, 2, 1, 0)), (1.0e300, (2, 2, 0, 1))):
        requests.append(line_of('gi_porient', [p, *o]))


def append_geomint_requests(requests: list[str]) -> None:
    """The GeomInt, ApproxInt and GeomAPI_IntSS requests."""
    _intss_requests(requests)
    _wlapprox_requests(requests)
    _mline_requests(requests)
    _sv_requests(requests)
    _knots_requests(requests)
    _lines_requests(requests)
    _lconst_requests(requests)
    _statics_requests(requests)
