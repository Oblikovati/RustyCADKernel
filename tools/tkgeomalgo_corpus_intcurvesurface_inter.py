"""Deterministic requests for the IntCurveSurface algorithm
(tools/oracle/tkgeomalgo_probe_intcurvesurface_inter.cxx), over the geometry catalog
of tkgeomalgo_corpus_intcurvesurface.py.

`ics_hinter` runs IntCurveSurface_HInter::Perform on every catalog curve against every
catalog surface (conics against quadrics analytically, lines against tori, cones and
sampled infinite swept and offset surfaces, other curves through the polygon and
polyhedron search), plus tangent, grazing, parallel, in-surface and missing lines, and
the polygon and polyhedron overloads. `ics_hinter_protected` drives the protected steps
directly; `ics_interference`, `ics_exact` and `ics_quadcurv` the instance classes;
`ics_estlim`, `ics_project` and `ics_interutils` the header utilities.
"""
from __future__ import annotations
from tkgeomalgo_corpus_intcurvesurface import (CURVES, HALF_PI, O, SURFACES, TWO_PI, X, Y, Z, ad, circle, cyl,
                                               extr, hyperbola, line, line_of, offs, parabola, plane, revol, sad,
                                               sphere, torus)

CURVE_LIST = list(CURVES.values())
SURFACE_LIST = list(SURFACES.values())


def _hinter(requests: list[str]) -> None:
    for c in CURVES.values():
        for s in SURFACES.values():
            requests.append(line_of('ics_hinter', c + s + [0]))
    special = [
        (ad(line([-3.0, 0.0, 1.5], X), (0.0, 6.0)), sad(sphere(O, Z, 1.5))),                  # tangent to a sphere
        (ad(line([-3.0, 1.0, 0.0], X), (0.0, 6.0)), sad(cyl(O, Z, 1.0), (0.0, TWO_PI, -2.0, 2.0))),  # grazing
        (ad(line([-3.0, 0.0, 0.0], X), (0.0, 6.0)), sad(torus(O, Z, 3.0, 1.0))),               # four roots
        (ad(line([0.0, 0.0, -5.0], Z), (0.0, 10.0)), sad(torus(O, Z, 3.0, 1.0))),              # through the hole
        (ad(line([2.0, 0.0, -5.0], Z), (0.0, 10.0)), sad(torus(O, Z, 3.0, 1.0))),              # tangent inside
        (ad(line([0.0, 0.0, 5.0], [0.0, 0.0, -1.0]), (0.0, 10.0)), SURFACES['cone']),          # along the axis
        (ad(line([-3.0, 0.0, 0.3], X), (0.0, 6.0)), SURFACES['plane']),                        # in the plane
        (ad(line([-3.0, 0.0, 0.8], X), (0.0, 6.0)), SURFACES['plane']),                        # parallel
        (ad(line([-3.0, 5.0, 0.0], X), (0.0, 6.0)), SURFACES['sphere']),                       # miss
        (ad(circle([0.0, 0.0, 0.3], Z, 1.0)), SURFACES['plane']),                              # circle in plane
        (ad(circle(O, Z, 1.0)), SURFACES['cyl']),                                              # circle on cylinder
        (ad(circle([0.0, 0.0, 1.5], Z, 0.5)), SURFACES['sphere']),                             # miss above
        (ad(line([0.2, 0.1, -3.0], [0.0, 0.0, 1.0]), (0.0, 6.0)), SURFACES['ext_parab']),      # along rulings
        (ad(line([-5.0, 0.1, 0.2], [1.0, 0.05, 0.0])), SURFACES['ext_line']),                  # parallel extrusion
        (ad(line([0.0, 0.0, -3.0], [0.1, 0.0, 1.0])), SURFACES['rev_line']),                   # near the axis
        (ad(line([0.0, 0.0, -3.0], Z)), SURFACES['rev_parab']),                                # on the axis
        (ad(line([-5.0, 0.0, 0.0], X)), SURFACES['off_rev']),
        (ad(line([-5.0, 0.3, 0.1], X)), SURFACES['off_cyl']),
        (ad(line([0.0, 0.0, 0.0], Z)), SURFACES['off_cyl']),                                   # parallel offset cylinder
        (ad(line([0.1, 0.1, 5.0], [0.0, 0.0, -1.0])), sad(offs(-1.5, cyl(O, Z, 1.0)))),       # flipped offset
        (ad(line([0.1, 0.1, 5.0], [0.0, 0.1, -1.0])), sad(offs(-1.0, cyl(O, Z, 1.0)))),       # null offset radius
        (ad(line([0.1, 0.1, 5.0], [0.3, 0.1, -1.0])), sad(offs(-2.0, plane(O, Z)))),
        (ad(line([0.1, 0.1, 5.0], [0.3, 0.1, -1.0])), sad(offs(0.5, extr(Z, line(O, X))))),
        (ad(line([0.1, 0.1, 5.0], [0.3, 0.1, -1.0])), sad(offs(0.5, extr(Z, hyperbola(O, Z, 1.0, 0.6))))),
        (ad(line([0.1, 0.1, 5.0], [0.3, 0.1, -1.0])), sad(offs(0.5, revol(O, Z, line([1.0, 0.0, 0.0], [0.3, 0.0, 1.0]))))),
        (ad(line([0.1, 0.1, 5.0], [0.3, 0.1, -1.0])), sad(offs(0.5, revol(O, Z, parabola([1.0, 0.0, 0.0], Y, 0.5))))),
        (ad(line([-3.0, 0.0, 0.1], [1.0, 0.0, 0.3])), sad(revol(O, Z, line([1.0, 0.0, 0.0], Z)))),
        (ad(line([0.0, 5.0, 0.2], [0.0, -1.0, 0.0])), sad(extr([1.0, 0.0, 0.0], parabola(O, X, 0.3)))),
    ]
    for c, s in special:
        requests.append(line_of('ics_hinter', c + s + [0]))
    _hinter_overloads(requests)


def _hinter_overloads(requests: list[str]) -> None:
    pairs = [('bspline', 'bezier_s'), ('bezier', 'torus'), ('circle', 'bspline_s'), ('line', 'cyl'),
             ('parabola', 'rev_bezier'), ('bspline_c1', 'plane'), ('ellipse', 'sphere')]
    polygons = [[0, 12], [1, 0.1, 0.9, 7], [2, 4, 0.0, 0.3, 0.6, 1.0]]
    polyhedra = [[0, 6, 6, 0.0, 0.0, 1.0, 1.0], [1, 3, 0.0, 0.5, 1.0, 3, 0.0, 0.5, 1.0]]
    for cn, sn in pairs:
        c, s = CURVES[cn], SURFACES[sn]
        for k, pg in enumerate(polygons):
            ph = polyhedra[k % 2]
            requests.append(line_of('ics_hinter', c + s + [1] + pg))
            requests.append(line_of('ics_hinter', c + s + [2] + pg + ph))
            requests.append(line_of('ics_hinter', c + s + [3] + pg + ph))
            requests.append(line_of('ics_hinter', c + s + [4] + ph))


def _protected(requests: list[str]) -> None:
    box = [0.0, 0.0, 1.0, 1.0]
    for cn, sn in [('bspline', 'bspline_s'), ('line', 'cyl'), ('bezier', 'sphere'), ('circle', 'torus'),
                   ('line_nat', 'ext_hypr'), ('parabola', 'bezier_s'), ('hyperbola', 'rev_hypr')]:
        c, s = CURVES[cn], SURFACES[sn]
        requests.append(line_of('ics_hinter_protected', c + s + [0, -1e60, -2.0, 1e60, 2.0]))
        requests.append(line_of('ics_hinter_protected', c + s + [0, 0.0, 0.0, 1.0, 1.0]))
        requests.append(line_of('ics_hinter_protected', c + s + [1]))
        requests.append(line_of('ics_hinter_protected', c + s + [2, 0, 9, 0, 5, 5, *box, *box]))
        requests.append(line_of('ics_hinter_protected', c + s + [3, 1, 0.0, 1.0, 9, 0, 4, 4, *box, *box]))
        requests.append(line_of('ics_hinter_protected', c + s + [4, 0, 12, *box]))
        for k in range(5):
            conic = [k, 0.1, 0.2, -2.0, 0.1, 0.2, 1.0, 1.2, 0.7]
            requests.append(line_of('ics_hinter_protected', c + s + [5, *conic, -1.0, -1.0, 2.0, 2.0]))
        # AppendIntAna reads uninitialized (u, v) natively on a surface that is not a
        # quadric (docs/tkgeomalgo-progress.md), so it runs on quadrics only.
        if sn in ('cyl', 'sphere'):
            requests.append(line_of('ics_hinter_protected', c + s + [6, -2.0, 0.0, 0.3, 1.0, 0.0, 0.0, 0, *O, *Z, 1e-12]))
            requests.append(line_of('ics_hinter_protected', c + s + [6, -2.0, 0.0, 0.3, 1.0, 0.0, 0.0, 1, *O, 1.0]))
            requests.append(line_of('ics_hinter_protected', c + s + [6, -2.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0, *O, *Z, 1e-12]))
        for w, u, v in ((0.5, 0.2, 0.3), (-7.0, 0.2, 0.3), (0.5, 20.0, 0.3), (0.5, 0.2, -30.0), (0.5, -0.3, 0.3)):
            requests.append(line_of('ics_hinter_protected', c + s + [7, w, u, v]))
        requests.append(line_of('ics_hinter_protected', c + s + [8, 0.0, 1.0]))


def _interference(requests: list[str]) -> None:
    ops = [0, 3, 6, 7, 8, 11, 1, -1.0, 0.0, 0.2, 1.0, 0.1, 0.05, 4, 0.0, 0.0, -2.0, 0.0, 0.0, 1.0,
           2, 2, 0.0, 0.0, -2.0, 0.0, 0.1, 1.0, -1.0, 0.2, 0.3, 1.0, 0.0, 0.0,
           9, -1.0, 0.0, 0.2, 1.0, 0.1, 0.05, 10, 2, 0.0, 0.0, -2.0, 0.0, 0.1, 1.0, -1.0, 0.2, 0.3, 1.0, 0.0, 0.0,
           12, 0.0, 0.0, -2.0, 0.0, 0.0, 1.0, 13, 1, 0.3, 0.3, -2.0, 0.0, 0.0, 1.0]
    count = 13
    for cn, sn in [('bspline', 'bezier_s'), ('line', 'plane'), ('circle', 'sphere'), ('bezier', 'bspline_s'),
                   ('polyline', 'plane_tilt'), ('ellipse', 'cyl'), ('line_z', 'torus')]:
        for pg, ph in ([[0, 10], [0, 5, 5, 0.0, 0.0, 1.0, 1.0]], [[1, 0.2, 0.8, 20], [0, 8, 3, 0.0, 0.0, 1.0, 1.0]]):
            requests.append(line_of('ics_interference', CURVES[cn] + pg + SURFACES[sn] + ph + [count, *ops]))


def _exact(requests: list[str]) -> None:
    bounds = [0.0, 1.0, 0.0, 1.0, 0.0, 6.0]
    cases = [('line_z', 'plane', 0.3, 0.2, 3.3), ('line', 'sphere', 0.1, 0.1, 1.6), ('bspline', 'bezier_s', 0.5, 0.5, 1.5),
             ('circle', 'cyl', 1.0, 0.5, 1.0), ('line', 'torus', 3.1, 0.1, 0.1), ('line_tangent', 'sphere', 0.0, 1.5, 3.0),
             ('bezier', 'bspline_s', 1.0, 0.5, 0.5), ('line_diag', 'plane_tilt', 0.0, 0.0, 3.0)]
    for cn, sn, u, v, w in cases:
        for tol, margin in ((1e-7, 0.0), (1e-3, 0.1), (0.0, 0.5)):
            requests.append(line_of('ics_exact', SURFACES[sn] + CURVES[cn] + [u, v, w, tol, margin, *bounds]))


def _quadric_spec(kind: int) -> list[float]:
    return [kind, 0.0, 0.1, 0.0, *Z, *X, 1.0, 0.4]


def _quadcurv(requests: list[str]) -> None:
    pars = [5, -0.5, 0.0, 0.3, 1.0, 2.5]
    for sn in ('plane', 'plane_nat', 'cyl', 'cyl_nat', 'cone', 'sphere', 'sphere_part', 'torus', 'bezier_s'):
        for k, (cn, c) in enumerate(CURVES.items()):
            requests.append(line_of('ics_quadcurv', SURFACES[sn] + c + _quadric_spec(k % 6) + pars))


def _estlim(requests: list[str]) -> None:
    flag_sets = [[1, 1, 1, 1], [0, 0, 1, 1], [1, 0, 0, 0], [0, 1, 0, 1], [1, 1, 0, 0], [0, 0, 1, 0]]
    lines = [[0.2, 0.1, -3.0, 0.3, 0.1, 1.0], [-5.0, 0.1, 0.2, 1.0, 0.1, 0.05], [0.0, 0.0, -3.0, 0.0, 0.0, 1.0],
             [-5.0, 0.0, 0.0, 1.0, 0.0, 0.0], [1.0, 2.0, 3.0, 0.0, 1.0, 0.0]]
    bounds = [-2e100, 2e100, -2e100, 2e100]
    for sn, op, off in (('ext_line', 0, 0), ('ext_parab', 0, 0), ('ext_hypr', 0, 0), ('ext_bezier', 0, 0),
                        ('rev_line', 1, 0), ('rev_parab', 1, 0), ('rev_hypr', 1, 0), ('rev_bezier', 1, 0),
                        ('off_plane', 2, 0), ('off_cyl', 2, 0), ('off_cone', 2, 0), ('off_ext', 2, 0),
                        ('off_rev', 2, 0), ('off_sphere', 2, 0), ('off_ext', 0, 1), ('plane_nat', 3, 0)):
        for k, ln in enumerate(lines):
            for j, flags in enumerate(flag_sets):
                b = [bounds[i] if flags[i] else [-1.0, 1.0, -1.5, 1.5][i] for i in range(4)]
                requests.append(line_of('ics_estlim', SURFACES[sn] + ln + [op, *flags, *b, 20 - 7 * (j % 2), off]))


def _project(requests: list[str]) -> None:
    lines = [[0.0, 0.0, -1.0, 0.3, 0.2, 1.0], [1.0, 2.0, 0.0, 0.0, 0.0, 1.0], [0.5, 0.0, 0.0, 1.0, 0.0, 0.0]]
    planes = [[0.0, 0.0, 0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0, 0.0, 0.0]]
    for ln in lines:
        for pl in planes:
            for k in (0, 1, 3, 4):
                conic = [k, 1.0, 0.0, 0.0, 0.0, 1.0, 0.2, 0.8, 0.5]
                requests.append(line_of('ics_project', ln + pl + [0.0, 0.0, 0.0, 0.0, 1.0, 0.0] + conic + [1e308, -1e308]))
                requests.append(line_of('ics_project', ln + pl + [0.0, 0.0, 0.0, 1.0, 0.0, 0.0] + conic + [0.0, 1.0]))


def _interutils(requests: list[str]) -> None:
    for cn in ('line', 'line_tangent', 'circle', 'bspline', 'offset_circle', 'line_z', 'line_diag'):
        for sn in ('plane', 'cyl', 'cone', 'sphere', 'torus', 'bezier_s', 'trim_cyl', 'off_sphere'):
            c, s = CURVES[cn], SURFACES[sn]
            for w, u, v in ((0.5, 0.2, 0.3), (3.0, 1.0, 0.0), (-0.5, 7.0, -3.0), (6.0 + 1e-9, 0.0, 0.0)):
                requests.append(line_of('ics_interutils', c + s + [0, w, u, v]))
                requests.append(line_of('ics_interutils', c + s + [2, w, u, v]))
            requests.append(line_of('ics_interutils', c + s + [1, 0.3, -0.2, 0.9, -5.0, 5.0]))
            requests.append(line_of('ics_interutils', c + s + [3]))
            requests.append(line_of('ics_interutils', c + s + [4]))
            requests.append(line_of('ics_interutils', c + s + [5]))
            requests.append(line_of('ics_interutils', c + s + [8]))
    c, s = CURVES['bspline'], SURFACES['bezier_s']
    points = [4, 0.5, 0.5, 1.0, 0.2, 0.1, 1.0 + 1e-10, 0.1, 0.9, 0.5, 0.3, 0.1, 1.0]
    requests.append(line_of('ics_interutils', c + s + [6, *points]))
    requests.append(line_of('ics_interutils', c + s + [6, 0]))
    requests.append(line_of('ics_interutils', c + s + [6, 5, 1.0, 1.0, 1.0, 1.0, 0.5, 1.0, 0.5, 1.0, 1.0,
                                                   0.5, 0.5, 1.0, 2.0, 0.0, 0.0]))
    for b in ([-1e60, 1e60, -1e51, 1e49], [0.0, 1.0, 0.0, 1.0], [-2e100, 2e100, -2e100, 2e100]):
        requests.append(line_of('ics_interutils', c + s + [7, *b]))
    for sn in ('bezier_s', 'cyl', 'sphere', 'bspline_s', 'torus', 'plane'):
        for xyz in ([-1.0, 1.0, -1.0, 1.0, -0.5, 0.5], [0.0, 0.2, 0.0, 0.2, -5.0, 5.0], [9.0, 9.5, 9.0, 9.5, 9.0, 9.5]):
            requests.append(line_of('ics_interutils', c + SURFACES[sn] + [9, 0.0, 1.0, 0.0, 1.0, *xyz]))
    spoints = [3,
               0, 0.5, 0.5, 0.2, 2, 0, 2, 0.4, 1, 3, 0, 0.0, 0.0,
               0, 0.1, 0.2, 0.0, 2, 0, 3, 0.5, 2, 5, 6, 0.25, 0.0,
               0, 0.0, 0.0, 0.0, 3, 0, 1, 0.0, 3, 7, 0, 0.0, 0.0]
    for sn in ('bezier_s', 'plane', 'sphere'):
        requests.append(line_of('ics_interutils', c + SURFACES[sn] + [10, 0, 8, 0, 4, 4, 0.0, 0.0, 1.0, 1.0,
                                                                     *spoints]))
    for cn, sn in (('bspline', 'bezier_s'), ('circle', 'bspline_s'), ('bezier', 'torus'), ('line', 'rev_bezier')):
        requests.append(line_of('ics_interutils', CURVES[cn] + SURFACES[sn] + [11, 0, 20, 0, 10, 10, 0.0, 0.0, 1.0, 1.0,
                                                                              0.0, 1.0, 0.0, 1.0]))


def append_inter_requests(requests: list[str]) -> None:
    """Append the algorithm request families in a fixed order."""
    _hinter(requests)
    _protected(requests)
    _interference(requests)
    _exact(requests)
    _quadcurv(requests)
    _estlim(requests)
    _project(requests)
    _interutils(requests)
