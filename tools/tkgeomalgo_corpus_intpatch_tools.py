"""Deterministic requests for the IntPatch tools (tools/oracle/tkgeomalgo_probe_intpatch_tools.cxx).

`ip_hinter` samples every catalog surface (tkgeomalgo_corpus_intcurvesurface.py) and runs
the arc members of IntPatch_HInterTool on the 2d curve catalog (tkgeomalgo_corpus_geom2d.py),
bounded and natural. `ip_hc2d` runs every IntPatch_HCurve2dTool member on each curve.
`ip_arcf` evaluates IntPatch_ArcFunction along arcs of the surfaces against every quadric
kind (the unset quadric included); `ip_csf` evaluates IntPatch_CSFunction and runs
IntPatch_CurvIntSurf from converging and diverging starts; `ip_curvature` computes the
curvature radius of the intersection line of surface pairs, including tangent and
parallel ones.
"""
from __future__ import annotations
from tkgeomalgo_corpus_geom2d import CURVES as CURVES2D, RANGES, c_circle, c_line
from tkgeomalgo_corpus_intcurvesurface import O, SURFACES, TWO_PI, Z, cyl, line_of, plane, sad, sphere
from tkgeomalgo_corpus_intstart import QUADRICS


def arc(curve, rng=None):
    """`curve2d bounded [u0 u1]`."""
    return [*curve, 0] if rng is None else [*curve, 1, *rng]


ARCS = [arc(c, r) for c, r in zip(CURVES2D, RANGES)] + [arc(c) for c in CURVES2D[:6]]
UNSET_QUADRIC = [5, *O, *Z, 1.0, 0.0, 0.0, 0.0, 0.0]


def _hinter_requests(requests: list[str]) -> None:
    arcs = [ARCS[0], ARCS[1], ARCS[6], ARCS[8], ARCS[18]]
    for s in SURFACES.values():
        requests.append(line_of('ip_hinter', [*s, len(arcs), *[v for a in arcs for v in a], 0.3, -0.7]))
    requests.append(line_of('ip_hinter', [*SURFACES['plane'], len(ARCS), *[v for a in ARCS for v in a], 1.5, 2.5]))


def _hc2d_requests(requests: list[str]) -> None:
    for a, (u0, u1) in zip(ARCS, RANGES + [(0.0, 1.0)] * 6):
        for u in (u0, 0.5 * (u0 + u1), u1):
            requests.append(line_of('ip_hc2d', [*a, u, u0, 0.5 * (u0 + u1)]))


SURFACE_ARCS = [
    (SURFACES['plane'], arc(c_line(-2.0, 0.5, 1.0, 0.0), (0.0, 4.0))),
    (SURFACES['plane'], arc(c_circle(0.0, 0.0, 1.5), (0.0, TWO_PI))),
    (SURFACES['cyl'], arc(c_line(0.0, -2.0, 0.0, 1.0), (0.0, 4.0))),
    (SURFACES['cyl'], arc(c_line(0.0, 0.5, 1.0, 0.0), (0.0, TWO_PI))),
    (SURFACES['sphere'], arc(c_line(0.3, -1.5, 0.0, 1.0), (0.0, 3.0))),
    (SURFACES['cone'], arc(c_line(0.0, 1.0, 1.0, 0.0), (0.0, TWO_PI))),
    (SURFACES['torus'], arc(c_line(0.0, 0.0, 1.0, 0.0), (0.0, TWO_PI))),
    (SURFACES['bspline_s'], arc(c_line(0.0, 0.5, 1.0, 0.0), (0.0, 2.0))),
    (SURFACES['bezier_s'], arc(CURVES2D[7], (0.0, 1.0))),
    (SURFACES['rev_bezier'], arc(c_line(0.0, 0.5, 1.0, 0.0), (0.0, TWO_PI))),
]


def _arcf_requests(requests: list[str]) -> None:
    xs = [0.0, 0.3, 1.0, 1.7, 2.5, 3.0]
    for s, a in SURFACE_ARCS:
        for q in [*QUADRICS.values(), UNSET_QUADRIC]:
            requests.append(line_of('ip_arcf', [*s, *q, *a, len(xs), *xs]))


def _csf_requests(requests: list[str]) -> None:
    starts = [(0.0, 0.0, 0.5), (0.3, 0.2, 1.0), (1.0, -1.0, 3.0), (5.0, 5.0, -2.0)]
    pairs = [
        (SURFACES['plane'], arc(c_line(0.0, -1.0, 0.0, 1.0), (0.0, 4.0)), SURFACES['cyl']),
        (SURFACES['sphere'], arc(c_line(0.0, 0.1, 1.0, 0.0), (0.0, TWO_PI)), SURFACES['cyl']),
        (SURFACES['cyl'], arc(c_line(-1.0, 0.0, 1.0, 1.0), (0.0, 2.0)), SURFACES['plane']),
        (SURFACES['bspline_s'], arc(c_circle(1.0, 1.0, 0.5), (0.0, TWO_PI)), SURFACES['plane_tilt']),
        (SURFACES['torus'], arc(c_line(0.0, 0.0, 0.0, 1.0), (0.0, TWO_PI)), SURFACES['sphere']),
    ]
    for s1, a, s2 in pairs:
        pts = [v for p in starts for v in p]
        for u, v, w in starts:
            for tol, margin in ((1.0e-7, 0.0), (1.0e-5, 0.1)):
                requests.append(line_of('ip_csf', [*s1, *a, *s2, len(starts), *pts, u, v, w, tol, margin]))


def _curvature_requests(requests: list[str]) -> None:
    pairs = [
        (sad(plane(O, Z)), sad(sphere(O, Z, 1.0)), [0.0, 0.5, 1.0]),
        (sad(sphere(O, Z, 1.0)), sad(cyl([0.5, 0.0, 0.0], Z, 0.5)), [0.3, 1.0, 2.0]),
        (sad(cyl(O, Z, 1.0)), sad(cyl(O, [1.0, 0.0, 0.0], 1.0)), [0.0, 0.7, 1.5]),
        (sad(plane(O, Z)), sad(plane([0.0, 0.0, 1.0], Z)), [0.0, 1.0]),
        (sad(plane(O, Z)), sad(plane(O, [1.0, 0.0, 0.0])), [0.0, 1.0]),
        (SURFACES['torus'], SURFACES['cone'], [0.2, 1.1, 2.4]),
        (SURFACES['bspline_s'], SURFACES['plane_tilt'], [0.3, 0.9, 1.5]),
    ]
    for s1, s2, ts in pairs:
        pts = []
        for t in ts:
            pts += [0.0, 0.0, 0.0, t, 0.5 * t, 0.3 + t, 0.2 * t]
        requests.append(line_of('ip_curvature', [*s1, *s2, len(ts), *pts]))


def append_intpatch_tool_requests(requests: list[str]) -> None:
    """The tool requests."""
    _hinter_requests(requests)
    _hc2d_requests(requests)
    _arcf_requests(requests)
    _csf_requests(requests)
    _curvature_requests(requests)
