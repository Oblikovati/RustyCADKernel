"""Deterministic requests for the GeomAPI projections and extrema
(tools/oracle/tkgeomalgo_probe_geomapi.cxx).

Curves and surfaces are written as ga_ics_geom_curve and ga_ics_geom_surface read them
(the builders of tkgeomalgo_corpus_intcurvesurface.py). The catalog holds every curve
kind (lines, conics, polynomial and rational Bezier and B-spline curves, periodic and C1
B-splines, a polyline, offset and trimmed curves) and every surface kind (elementary,
Bezier and B-spline surfaces, surfaces of revolution and extrusion, offset and trimmed
surfaces). The requests run every constructor and Init overload, every accessor with the
out-of-range indices 0 and N+1, default objects, reversed and infinite ranges, points on
the geometry and at degenerate places (circle and sphere centres, axes), repeated
Performs that switch the Extrema algorithm, and repeated Inits of one
GeomAPI_ExtremaCurveCurve (its Extrema member keeps solutions across Inits).
tkgeomalgo_corpus_geomapi_fit.py adds the plane conversions, the curve-surface
intersection, the interpolation and the approximation.
"""
from __future__ import annotations
import random
from tkgeomalgo_corpus_intcurvesurface import (
    BEZ_POLES, BSP_KNOTS, BSP_MULTS, BSP_POLES, HALF_PI, O, SURFACE_GRID, TWO_PI, X, Y, Z,
    bez_s, bezier, bsp_s, bspline, circle, cone, cyl, ellipse, extr, hyperbola, line,
    line_of, offs, offset_curve, parabola, plane, revol, rtrim, sphere, torus, trimmed)

INF = 2.0e100
GEOM_CURVES = {
    'line': line([-3.0, 0.2, 0.3], X),
    'line_diag': line([-2.0, -2.0, -2.0], [1.0, 1.0, 1.0]),
    'line_z': line([0.0, 0.0, -1.0], Z),
    'circle': circle([0.0, 0.0, 0.5], Z, 1.2),
    'circle_tilt': circle([0.2, -0.1, 0.0], [0.3, 0.1, 1.0], 1.5),
    'ellipse': ellipse([0.1, 0.0, -0.2], [0.2, 1.0, 0.4], 2.0, 0.9),
    'parabola': parabola([0.0, -0.5, 0.0], Y, 0.4),
    'hyperbola': hyperbola([0.0, 0.0, 0.1], [0.0, 1.0, 1.0], 0.8, 0.5),
    'bezier': bezier(BEZ_POLES),
    'bezier_rat': bezier(BEZ_POLES, [1.0, 2.0, 0.5, 1.0]),
    'bspline': bspline(3, BSP_POLES, BSP_KNOTS, BSP_MULTS),
    'bspline_c1': bspline(3, BSP_POLES + [[4.0, 1.0, 0.0]], [0.0, 1.0, 2.0], [4, 3, 4]),
    'bspline_rat': bspline(2, BSP_POLES[:5], [0.0, 0.5, 1.0, 2.0], [3, 1, 1, 3], [1.0, 1.5, 0.7, 1.2, 1.0]),
    'bspline_per': bspline(2, [[1.5, 0.0, 0.2], [0.0, 1.5, -0.3], [-1.5, 0.0, 0.4], [0.0, -1.5, -0.1]],
                           [0.0, 1.0, 2.0, 3.0, 4.0], [1, 1, 1, 1, 1], periodic=True),
    'polyline': bspline(1, BSP_POLES[:4], [0.0, 1.0, 2.0, 3.0], [2, 1, 1, 2]),
    'offset_circle': offset_curve(0.3, Z, circle([0.0, 0.0, 0.2], Z, 1.1)),
    'trimmed_parab': trimmed(-1.0, 2.0, parabola([0.0, 0.0, -0.6], [1.0, 0.0, 1.0], 0.5)),
    'trimmed_circle': trimmed(0.5, 2.5, circle(O, Z, 2.0)),
    'segment': trimmed(0.0, 4.0, line([-2.0, 0.5, 0.0], X)),
}
# (first, last) of each curve, for the ranged overloads (infinite curves get a window).
CURVE_RANGES = {
    'line': (0.0, 6.0), 'line_diag': (-1.0, 5.0), 'line_z': (-2.0, 3.0), 'circle': (0.0, TWO_PI),
    'circle_tilt': (0.5, 4.0), 'ellipse': (0.0, TWO_PI), 'parabola': (-2.5, 2.5),
    'hyperbola': (-1.5, 1.5), 'bezier': (0.0, 1.0), 'bezier_rat': (0.1, 0.9), 'bspline': (0.0, 3.0),
    'bspline_c1': (0.0, 2.0), 'bspline_rat': (0.0, 2.0), 'bspline_per': (0.0, 4.0),
    'polyline': (0.0, 3.0), 'offset_circle': (0.0, TWO_PI), 'trimmed_parab': (-1.0, 2.0),
    'trimmed_circle': (0.5, 2.5), 'segment': (0.0, 4.0),
}
GEOM_SURFACES = {
    'plane': plane([0.0, 0.0, 0.3], Z),
    'plane_tilt': plane([0.1, 0.0, 0.0], [1.0, 0.2, 0.4]),
    'cyl': cyl(O, Z, 1.0),
    'cone': cone([0.0, 0.0, -1.0], Z, 0.4, 0.8),
    'sphere': sphere(O, Z, 1.5),
    'torus': torus(O, Z, 3.0, 1.0),
    'bezier_s': bez_s(SURFACE_GRID),
    'bspline_s': bsp_s(2, 2, SURFACE_GRID, [0.0, 1.0, 2.0], [3, 1, 3], [0.0, 0.5, 1.0], [3, 1, 3]),
    'rev_line': revol(O, Z, line([1.0, 0.0, 0.0], [0.3, 0.0, 1.0])),
    'rev_bezier': revol(O, Z, bezier([[1.0, 0.0, -1.0], [2.0, 0.0, 0.0], [0.8, 0.0, 1.5]])),
    'ext_parab': extr(Z, parabola(O, Z, 0.5)),
    'ext_bezier': extr([0.0, 0.2, 1.0], bezier([[-2.0, 0.0, 0.0], [0.0, 2.0, 0.0], [2.0, -1.0, 0.0]])),
    'off_sphere': offs(0.5, sphere(O, Z, 1.0)),
    'off_bspline': offs(0.2, bsp_s(2, 2, SURFACE_GRID, [0.0, 1.0, 2.0], [3, 1, 3], [0.0, 0.3, 1.0], [3, 1, 3])),
    'trim_cyl': rtrim(0.5, 3.0, -1.0, 1.0, cyl(O, Z, 1.0)),
    'trim_sphere': rtrim(0.2, 3.0, -1.0, 1.2, sphere([0.5, 0.0, 0.0], Z, 2.0)),
}
# A finite parameter box of each surface.
SURFACE_BOUNDS = {
    'plane': (-2.0, 2.0, -2.0, 2.0), 'plane_tilt': (-4.0, 4.0, -3.0, 3.0), 'cyl': (0.0, TWO_PI, -2.0, 2.0),
    'cone': (0.0, TWO_PI, -2.0, 3.0), 'sphere': (0.0, TWO_PI, -HALF_PI, HALF_PI), 'torus': (0.0, TWO_PI, 0.0, TWO_PI),
    'bezier_s': (0.0, 1.0, 0.0, 1.0), 'bspline_s': (0.0, 2.0, 0.0, 1.0), 'rev_line': (0.0, TWO_PI, -2.0, 2.0),
    'rev_bezier': (0.0, 4.0, 0.0, 1.0), 'ext_parab': (-2.0, 2.0, -1.0, 1.0), 'ext_bezier': (0.0, 1.0, -2.0, 2.0),
    'off_sphere': (0.0, TWO_PI, -1.0, 1.0), 'off_bspline': (0.0, 2.0, 0.0, 1.0), 'trim_cyl': (0.5, 3.0, -1.0, 1.0),
    'trim_sphere': (0.2, 3.0, -1.0, 1.2),
}
POINTS = [[0.3, -0.4, 0.8], [2.5, 1.0, -0.5], [0.0, 0.0, 0.5], [0.0, 0.0, 0.0], [-1.2, 3.0, 2.0],
          [10.0, -7.0, 4.0], [1.2, 0.0, 0.5]]


def append_ppc_requests(requests: list[str]) -> None:
    """GeomAPI_ProjectPointOnCurve: every mode on every curve, several points."""
    for name, curve in GEOM_CURVES.items():
        u0, u1 = CURVE_RANGES[name]
        for k, p in enumerate(POINTS):
            q = POINTS[(k + 3) % len(POINTS)]
            for mode in range(6):
                requests.append(line_of('gapi_ppc', [*curve, mode, *p, u0, u1, *q]))
        requests.append(line_of('gapi_ppc', [*curve, 1, 0.1, 0.2, 0.3, u1, u0, *O]))
        requests.append(line_of('gapi_ppc', [*curve, 2, 0.1, 0.2, 0.3, u0 + 0.3 * (u1 - u0), u0 + 0.6 * (u1 - u0), *O]))
        requests.append(line_of('gapi_ppc', [*curve, 1, 0.1, 0.2, 0.3, -INF, INF, *O]))


def pps(surface, ctor, algo, tol, bounds, flag, steps) -> str:
    return line_of('gapi_pps', [*surface, ctor, algo, tol, *bounds, flag, len(steps),
                                *[v for p, a in steps for v in (*p, a)]])


def append_pps_requests(requests: list[str]) -> None:
    """GeomAPI_ProjectPointOnSurf: every constructor and Init, algorithms, flags, and
    Performs that switch the algorithm (the grid and the tree persist)."""
    rng = random.Random(4417)
    for name, surface in GEOM_SURFACES.items():
        bounds = SURFACE_BOUNDS[name]
        for ctor in range(11):
            p = POINTS[ctor % len(POINTS)]
            steps = [(p, -1), (POINTS[(ctor + 2) % len(POINTS)], -1)]
            requests.append(pps(surface, ctor, ctor % 2, 1e-7 * (1 + ctor % 3), bounds, -1, steps))
        for flag in (0, 1, 2):
            steps = [(POINTS[flag], -1), (POINTS[flag + 3], -1)]
            requests.append(pps(surface, 5, 0, 1e-9, bounds, flag, steps))
        switches = [(POINTS[0], -1), (POINTS[1], 1), (POINTS[4], 0), (POINTS[1], 1), (POINTS[5], -1)]
        requests.append(pps(surface, 4, 0, 1e-9, bounds, -1, switches))
        requests.append(pps(surface, 9, 1, 1e-9, bounds, 0, switches))
        cloud = [([round(rng.uniform(-3, 3), 3) for _ in range(3)], rng.choice([-1, -1, 0, 1])) for _ in range(4)]
        requests.append(pps(surface, 2, rng.randint(0, 1), 1e-9, bounds, -1, cloud))
    requests.append(pps(GEOM_SURFACES['plane'], 4, 0, 1e-9, (1.0, -1.0, 0.0, 1.0), -1, [(POINTS[0], -1)]))
    requests.append(pps(GEOM_SURFACES['bspline_s'], 8, 0, 1e-9, (0.5, 1.5, 0.2, 0.8), -1, [(POINTS[1], -1)]))


CC_PAIRS = [('line', 'line_diag'), ('line', 'line_z'), ('segment', 'line_z'), ('circle', 'line_z'),
            ('circle', 'circle_tilt'), ('circle', 'trimmed_circle'), ('ellipse', 'line'),
            ('parabola', 'line'), ('hyperbola', 'line_z'), ('bezier', 'bspline'), ('bezier_rat', 'segment'),
            ('bspline_c1', 'polyline'), ('bspline_rat', 'circle'), ('bspline_per', 'ellipse'),
            ('offset_circle', 'line_diag'), ('trimmed_parab', 'bspline'), ('polyline', 'segment')]
PARALLEL_LINES = (line([0.0, 0.0, 0.0], X), line([1.0, 2.0, 0.5], X))
CONCENTRIC = (circle(O, Z, 3.0), circle(O, Z, 1.0))


def ecc_init(c1: str, c2: str, ranged: int) -> list[float]:
    return [*GEOM_CURVES[c1], *GEOM_CURVES[c2], ranged, *CURVE_RANGES[c1], *CURVE_RANGES[c2]]


def append_ecc_requests(requests: list[str]) -> None:
    """GeomAPI_ExtremaCurveCurve: both orders, whole and ranged, repeated Inits, the
    Total members (infinite parallel lines, concentric circles), the default object."""
    for a, b in CC_PAIRS:
        for c1, c2 in ((a, b), (b, a)):
            for ranged in (0, 1):
                requests.append(line_of('gapi_ecc', [1, *ecc_init(c1, c2, ranged), ranged]))
        requests.append(line_of('gapi_ecc', [2, *ecc_init(a, b, 0), *ecc_init(b, a, 1), 0]))
        requests.append(line_of('gapi_ecc', [3, *ecc_init(a, b, 1), *ecc_init(a, b, 1), *ecc_init(a, b, 0), 1]))
    for c1, c2 in (PARALLEL_LINES, CONCENTRIC):
        requests.append(line_of('gapi_ecc', [1, *c1, *c2, 0, 0.0, 1.0, 0.0, 1.0, 0]))
        requests.append(line_of('gapi_ecc', [1, *c1, *c2, 1, -1.0, 1.0, 0.5, 2.0, 1]))
    requests.append(line_of('gapi_ecc', [1, *GEOM_CURVES['segment'], *GEOM_CURVES['line'], 1, 3.0, 1.0, 0.0, 1.0, 0]))
    requests.append(line_of('gapi_ecc', [0, 0]))
    requests.append(line_of('gapi_ecc', [0, 1]))


CS_PAIRS = [('line', 'plane'), ('line_z', 'plane'), ('line', 'cyl'), ('line_z', 'cyl'), ('line_diag', 'sphere'),
            ('circle', 'plane'), ('circle', 'cyl'), ('circle', 'sphere'), ('hyperbola', 'plane'),
            ('ellipse', 'torus'), ('bezier', 'bspline_s'), ('bspline', 'plane_tilt'), ('segment', 'cone'),
            ('circle_tilt', 'rev_bezier'), ('parabola', 'ext_parab'), ('bspline_rat', 'off_sphere'),
            ('trimmed_circle', 'trim_cyl'), ('polyline', 'bezier_s'), ('offset_circle', 'trim_sphere')]


def append_ecs_requests(requests: list[str]) -> None:
    """GeomAPI_ExtremaCurveSurface: every constructor and Init on curve-surface pairs
    (parallel, intersecting and separated), with ranges."""
    for c, s in CS_PAIRS:
        w, b = CURVE_RANGES[c], SURFACE_BOUNDS[s]
        for mode in range(5):
            requests.append(line_of('gapi_ecs', [*GEOM_CURVES[c], *GEOM_SURFACES[s], mode, *w, *b]))
    concentric = circle(O, Z, 0.5)
    requests.append(line_of('gapi_ecs', [*concentric, *GEOM_SURFACES['sphere'], 0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0]))
    requests.append(line_of('gapi_ecs', [*GEOM_CURVES['line'], *GEOM_SURFACES['plane'], 1, 5.0, 1.0,
                                         -2.0, 2.0, -2.0, 2.0]))


SS_PAIRS = [('plane', 'plane'), ('plane', 'plane_tilt'), ('plane', 'sphere'), ('sphere', 'torus'),
            ('cyl', 'plane'), ('bspline_s', 'plane'), ('bezier_s', 'sphere'), ('cone', 'trim_sphere'),
            ('rev_line', 'ext_parab'), ('off_sphere', 'bspline_s'), ('trim_cyl', 'rev_bezier')]


def append_ess_requests(requests: list[str]) -> None:
    """GeomAPI_ExtremaSurfaceSurface: every constructor and Init on surface pairs."""
    for a, b in SS_PAIRS:
        for s1, s2 in ((a, b), (b, a)):
            for mode in range(5):
                requests.append(line_of('gapi_ess', [*GEOM_SURFACES[s1], *GEOM_SURFACES[s2], mode,
                                                     *SURFACE_BOUNDS[s1], *SURFACE_BOUNDS[s2]]))
    shifted = plane([0.0, 0.0, 10.0], Z)
    requests.append(line_of('gapi_ess', [*GEOM_SURFACES['plane'], *shifted, 0, *[0.0] * 8]))
    requests.append(line_of('gapi_ess', [*GEOM_SURFACES['sphere'], *sphere([5.0, 0.0, 0.0], Z, 1.0), 0, *[0.0] * 8]))


def append_geomapi_requests(requests: list[str]) -> None:
    """Append the GeomAPI requests in a fixed order."""
    append_ppc_requests(requests)
    append_pps_requests(requests)
    append_ecc_requests(requests)
    append_ecs_requests(requests)
    append_ess_requests(requests)
    __import__('tkgeomalgo_corpus_geomapi_fit').append_geomapi_fit_requests(requests)


if __name__ == '__main__':
    out: list[str] = []
    append_geomapi_requests(out)
    print('\n'.join(out))
