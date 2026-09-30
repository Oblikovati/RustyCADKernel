"""Deterministic requests for the GeomFill trihedron laws (tools/oracle/tkgeomalgo_probe_geomfill_trihedron.cxx).

`gfs_trihedron` builds one law (Fixed, Frenet, CorrectedFrenet in its three modes,
ConstantBiNormal, Darboux, DiscreteTrihedron, DraftTrihedron, GuideTrihedronAC,
GuideTrihedronPlan) on lines, conics, Bezier and B-spline curves, including
inflections (the Frenet singular points), a cusp where C' vanishes, a straight
B-spline (C' ^ C'' null everywhere), periodic and ranged curves, and curves on
surfaces for the Darboux law; it prints SetCurve, the queries, D0/D1/D2 at several
parameters, Copy, SetInterval with the evaluators again and the guide members.
Parameters below the first sampled parameter of a CorrectedFrenet law are left out:
its GetAngleAT bisection loops forever there natively.
"""
from __future__ import annotations
from tkgeomalgo_corpus_geom2d import c_bspline, c_line
from tkgeomalgo_corpus_intcurvesurface import (ad, bezier, bspline, bsp_s, circle, cyl, ellipse, hyperbola,
                                               line, line_of, parabola, plane, sad, sphere)

X, Y, Z = [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]
O = [0.0, 0.0, 0.0]

# name -> (curve source, parameters, interval)
CURVES = {
    'line': (ad(line(O, [1.0, 2.0, 0.5])), [0.0, 0.5, 2.0], (0.2, 1.5)),
    'circle': (ad(circle([1.0, 0.0, 0.0], Z, 2.0), (0.0, 6.0)), [0.0, 1.0, 3.0], (0.5, 2.5)),
    'ellipse': (ad(ellipse(O, [0.0, 1.0, 1.0], 3.0, 1.0), (0.0, 6.2)), [0.3, 1.2, 4.0], (1.0, 5.0)),
    'parabola': (ad(parabola(O, Z, 0.5), (-2.0, 2.0)), [-1.0, 0.0, 1.5], (-1.0, 1.0)),
    'hyperbola': (ad(hyperbola(O, Y, 2.0, 1.0), (-1.0, 1.0)), [-0.5, 0.25, 0.9], (-0.5, 0.5)),
    'bezier3d': (ad(bezier([[0, 0, 0], [1, 2, 0], [2, -1, 1], [3, 1, 2]])), [0.0, 0.3, 0.7, 1.0], (0.1, 0.9)),
    'inflection': (ad(bspline(3, [[0, 0, 0], [1, 1, 0], [2, -1, 0], [3, 0, 0]], [0.0, 1.0], [4, 4])),
                   [0.0, 0.25, 0.5, 0.75, 1.0], (0.2, 0.8)),
    'wave': (ad(bspline(3, [[0, 0, 0], [1, 2, 0.5], [2, -2, 1], [3, 2, 1.5], [4, -2, 2], [5, 0, 2.5]],
                        [0.0, 1.0, 2.0, 3.0], [4, 1, 1, 4])), [0.0, 0.5, 1.5, 2.2, 3.0], (0.5, 2.5)),
    'cusp': (ad(bspline(3, [[0, 0, 0], [0, 0, 0], [1, 1, 0], [2, 0, 1]], [0.0, 1.0], [4, 4])),
             [0.0, 0.001, 0.5, 1.0], (0.0, 0.6)),
    'straight': (ad(bspline(3, [[0, 0, 0], [1, 1, 1], [2, 2, 2], [4, 4, 4]], [0.0, 1.0], [4, 4])),
                 [0.0, 0.4, 1.0], (0.1, 0.9)),
    'periodic': (ad(bspline(3, [[1, 0, 0], [0, 1, 0.2], [-1, 0, 0.4], [0, -1, 0.2]], [0.0, 1.0, 2.0, 3.0, 4.0],
                            [1, 1, 1, 1, 1], periodic=True)), [0.0, 1.3, 3.9], (0.5, 3.5)),
    'ranged': (ad(bspline(2, [[0, 0, 0], [1, 1, 0], [2, 0, 1], [3, 1, 1]], [0.0, 0.5, 1.0], [3, 1, 3]), (0.2, 0.9)),
               [0.2, 0.5, 0.9], (0.3, 0.8)),
}

LAWS = {
    'fixed': [0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.2],
    'fixed_parallel': [0, 1.0, 0.0, 0.0, 2.0, 0.0, 0.001],
    'frenet': [1],
    'corrected': [2, 0],
    'corrected_eval': [2, 1],
    'corrected_default': [2, 2],
    'binormal': [3, 0.0, 0.0, 1.0],
    'binormal_x': [3, 1.0, 0.0, 0.0],
    'discrete': [5],
    'draft': [6, 0.0, 0.0, 1.0, 0.3],
    'draft_x': [6, 1.0, 1.0, 0.0, -0.2],
}


def _src(curve: list[float]) -> list[float]:
    return [0, *curve]


def _request(law: list[float], source: list[float], params: list[float], interval: tuple, origins=(0.0, 1.0)) -> str:
    return line_of('gfs_trihedron', [*law, *source, len(params), *params, *interval, *origins])


def _free_laws(requests: list[str]) -> None:
    for law in LAWS.values():
        for name, (curve, params, interval) in CURVES.items():
            if law[0] == 2 and name == 'cusp':
                params = [p for p in params if p >= 0.0]
            requests.append(_request(law, _src(curve), params, interval))


def _darboux(requests: list[str]) -> None:
    surfaces = [
        (sad(plane(O, Z)), c_line(0.0, 0.0, 1.0, 0.5), [0.0, 1.0, 2.0], (0.5, 1.5)),
        (sad(cyl(O, Z, 2.0)), c_line(0.0, 0.0, 1.0, 0.3), [0.0, 1.0, 2.5], (0.5, 2.0)),
        (sad(sphere(O, Z, 1.5)), c_line(0.2, -1.0, 0.1, 1.0), [0.0, 0.5, 1.0], (0.1, 0.9)),
        (sad(sphere(O, Z, 1.5)), c_line(0.0, -1.5707963267948966, 0.0, 1.0), [0.0, 0.001, 1.0], (0.0, 1.0)),
        (sad(bsp_s(2, 2, [[[0, 0, 0], [0, 1, 0.5], [0, 2, 0]], [[1, 0, 0.5], [1, 1, 1], [1, 2, 0.5]],
                          [[2, 0, 0], [2, 1, 0.5], [2, 2, 0]]], [0.0, 1.0], [3, 3], [0.0, 1.0], [3, 3])),
         c_bspline(2, [0.0, 1.0], [3, 3], [(0.1, 0.1), (0.5, 0.8), (0.9, 0.3)]), [0.0, 0.5, 1.0], (0.2, 0.8)),
    ]
    for surface, c2d, params, interval in surfaces:
        requests.append(_request([4], [1, *c2d, *surface], params, interval))


def _guides(requests: list[str]) -> None:
    pairs = [
        (ad(line(O, X), (0.0, 4.0)), ad(line([0.0, 1.0, 0.0], X), (0.0, 4.0)), [0.0, 1.0, 3.5], (0.5, 3.0)),
        (ad(circle(O, Z, 2.0), (0.0, 6.0)), ad(circle([0.0, 0.0, 1.0], Z, 3.0), (0.0, 6.0)), [0.0, 1.5, 5.0], (1.0, 4.0)),
        (ad(bspline(3, [[0, 0, 0], [1, 0, 0.5], [2, 0, 0.5], [3, 0, 0]], [0.0, 1.0], [4, 4])),
         ad(bspline(3, [[0, 1, 0], [1, 1.5, 0.5], [2, 1.5, 1], [3, 1, 1]], [0.0, 1.0], [4, 4])),
         [0.0, 0.3, 0.8, 1.0], (0.2, 0.9)),
        (ad(line(O, Z), (0.0, 3.0)), ad(bspline(3, [[1, 0, 0], [1, 1, 1], [-1, 1, 2], [-1, 0, 3]], [0.0, 1.0], [4, 4])),
         [0.0, 1.5, 3.0], (0.5, 2.5)),
    ]
    for kind in (7, 8):
        for path, guide, params, interval in pairs:
            law = [kind, 0, *guide]
            requests.append(_request(law, _src(path), params, interval, (0.1, 0.9)))
    far = ad(line([0.0, 5.0, 0.0], Z), (0.0, 1.0))
    requests.append(_request([8, 0, *far], _src(ad(line(O, X), (0.0, 2.0))), [0.0, 1.0], (0.0, 1.0)))


def append_trihedron_requests(requests: list[str]) -> None:
    """Append every trihedron request family in a fixed order."""
    _free_laws(requests)
    _darboux(requests)
    _guides(requests)
