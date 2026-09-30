"""Deterministic requests for the GeomAPI plane conversions, curve-surface intersection,
interpolation and approximation (tools/oracle/tkgeomalgo_probe_geomapi_fit.cxx).

3D curves and surfaces come from the catalog of tkgeomalgo_corpus_geomapi.py, 2D curves
from tkgeomalgo_corpus_geom2d.py. GeomAPI::To2d runs on curves lying in the plane (each
conic and polynomial kind), on curves out of the plane (projected) and on tilted planes;
To3d on every 2D curve kind and several planes. GeomAPI_IntCS runs on curve-surface pairs
with points, tangencies, curves lying on the surface (segments) and misses, with every
index 0..N+1. GeomAPI_Interpolate and GeomAPI_PointsToBSpline run every constructor and
Init on smooth, noisy and degenerate point sets.
"""
from __future__ import annotations
import math
import random
from tkgeomalgo_corpus_geom2d import CURVES as CURVES_2D
from tkgeomalgo_corpus_geomapi import GEOM_CURVES, GEOM_SURFACES
from tkgeomalgo_corpus_intcurvesurface import (O, X, Z, bezier, bspline, circle, ellipse,
                                               hyperbola, line, line_of, offset_curve, parabola,
                                               plane, trimmed)

PLANES = [[0.0, 0.0, 0.0, 0.0, 0.0, 1.0], [0.0, 0.0, 0.5, 0.0, 0.0, 1.0],
          [0.3, -0.2, 0.1, 0.2, 0.3, 1.0], [1.0, 1.0, 1.0, 1.0, 0.0, 0.0]]
IN_PLANE = [line([-1.0, 0.5, 0.0], [1.0, 0.3, 0.0]), circle([0.2, 0.1, 0.0], Z, 1.3),
            ellipse([0.0, 0.0, 0.0], Z, 2.0, 0.7), parabola([0.1, 0.0, 0.0], Z, 0.6),
            hyperbola([0.0, 0.0, 0.0], Z, 1.0, 0.4),
            bezier([[-1.0, 0.0, 0.0], [0.0, 2.0, 0.0], [1.0, -1.0, 0.0], [2.0, 0.5, 0.0]]),
            bezier([[-1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]], [1.0, 0.6, 1.0]),
            bspline(3, [[-2.0, 0.0, 0.0], [-1.0, 1.0, 0.0], [0.0, -1.0, 0.0], [1.0, 1.0, 0.0],
                        [2.0, 0.0, 0.0]], [0.0, 1.0, 2.0], [4, 1, 4]),
            trimmed(0.2, 2.0, circle(O, Z, 1.0)), offset_curve(0.2, Z, circle(O, Z, 1.0))]


def append_to2d_requests(requests: list[str]) -> None:
    """GeomAPI::To2d on curves in and out of the planes."""
    for curve in IN_PLANE + list(GEOM_CURVES.values()):
        for pln in PLANES:
            requests.append(line_of('gapi_to2d', [*curve, *pln]))


def append_to3d_requests(requests: list[str]) -> None:
    """GeomAPI::To3d on every 2D curve kind and every plane."""
    for curve in CURVES_2D:
        for pln in PLANES:
            requests.append(line_of('gapi_to3d', [*curve, *pln]))


INTCS_PAIRS = [('line', 'plane'), ('line_z', 'plane'), ('line', 'cyl'), ('line_z', 'sphere'),
               ('line_diag', 'torus'), ('circle', 'plane'), ('circle', 'cyl'), ('ellipse', 'sphere'),
               ('parabola', 'plane_tilt'), ('hyperbola', 'cone'), ('bezier', 'bspline_s'),
               ('bspline', 'bezier_s'), ('bspline_rat', 'rev_bezier'), ('bspline_per', 'ext_parab'),
               ('polyline', 'plane'), ('offset_circle', 'cyl'), ('trimmed_parab', 'off_sphere'),
               ('segment', 'trim_cyl'), ('circle_tilt', 'trim_sphere'), ('bspline_c1', 'off_bspline')]


def append_intcs_requests(requests: list[str]) -> None:
    """GeomAPI_IntCS: points, curves lying on the surface (segments), misses."""
    for c, s in INTCS_PAIRS:
        for mode in (0, 1):
            requests.append(line_of('gapi_intcs', [*GEOM_CURVES[c], *GEOM_SURFACES[s], mode]))
    on_plane = line([-1.0, 0.0, 0.3], X)
    on_cyl = line([1.0, 0.0, -1.0], Z)
    requests.append(line_of('gapi_intcs', [*on_plane, *GEOM_SURFACES['plane'], 0]))
    requests.append(line_of('gapi_intcs', [*trimmed(0.0, 2.0, on_plane), *GEOM_SURFACES['plane'], 0]))
    requests.append(line_of('gapi_intcs', [*trimmed(-1.0, 1.0, on_cyl), *GEOM_SURFACES['cyl'], 0]))
    requests.append(line_of('gapi_intcs', [*circle([0.0, 0.0, 0.3], Z, 1.0), *GEOM_SURFACES['plane'], 0]))
    requests.append(line_of('gapi_intcs', [*GEOM_CURVES['line'], *GEOM_SURFACES['plane'], 2]))


def interpolate(points, periodic=0, tol=1e-6, params=None, mode=0, extra=()) -> str:
    head = [periodic, tol, len(points), *[v for p in points for v in p]]
    head += [1, len(params), *params] if params is not None else [0]
    return line_of('gapi_interpolate', [*head, mode, *extra])


def tangent_requests(pts, periodic) -> list[str]:
    """Load(t0, t1) with and without scaling, and flagged tangents at the ends and the middle."""
    out = [interpolate(pts, periodic, 1e-6, None, 1, (0.3, 0.4, 0.2, 1.0, 0.0, -0.5, s)) for s in (1, 0)]
    ends = [(1.0, 0.5, 0.2, 1)] + [(0.0, 0.0, 0.0, 0)] * (len(pts) - 2) + [(0.5, -1.0, 0.3, 1)]
    out.append(interpolate(pts, periodic, 1e-6, None, 2, (*[v for t in ends for v in t], 1)))
    mid = [(0.0, 0.0, 0.0, 0)] * len(pts)
    mid[len(pts) // 2] = (1.0, 1.0, 0.5, 1)
    out.append(interpolate(pts, periodic, 1e-6, None, 2, (*[v for t in mid for v in t], 0)))
    return out


def append_interpolate_requests(requests: list[str]) -> None:
    """GeomAPI_Interpolate: 2, 3 and more points, periodic or not, parameters, tangents,
    and the construction errors."""
    helix = [(math.cos(0.7 * i), math.sin(0.7 * i), 0.3 * i) for i in range(7)]
    square = [(100.0, 0.0, 0.0), (100.0, 100.0, 10.0), (0.0, 100.0, 20.0), (0.0, 0.0, 10.0), (50.0, -50.0, 0.0)]
    for pts in ([(0.0, 0.0, 0.0), (1.0, 1.0, 1.0)], [(0.0, 0.0, 0.0), (1.0, 1.0, 0.5), (2.0, 0.0, 1.0)], square, helix):
        for periodic in (0, 1):
            requests.append(interpolate(pts, periodic))
            requests.extend(tangent_requests(pts, periodic))
            params = [float(i) ** 1.1 for i in range(len(pts) + periodic)]
            requests.append(interpolate(pts, periodic, 1e-6, params))
    requests.append(interpolate([(0.0, 0.0, 0.0), (1e-9, 0.0, 0.0), (1.0, 1.0, 1.0)]))
    requests.append(interpolate(square, 0, 1e-6, [0.0, 1.0, 1.0, 2.0, 3.0]))
    requests.append(interpolate(square, 1, 1e-6, [0.0, 1.0, 2.0, 3.0]))
    requests.append(interpolate(square, 0, 1e-6, None, 1, (0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1)))
    requests.append(interpolate([(0.0, 0.0, 0.0)], 0))
    requests.append(interpolate([(0.0, 0.0, 0.0), (1.0, 0.0, 0.0)], 0, 1e-6, None, 1, (1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1)))


def fit(mode, points, args, params=None) -> str:
    extra = [len(params), *params] if params is not None else []
    return line_of('gapi_points_to_bspline', [mode, len(points), *[v for p in points for v in p], *extra, *args])


def fit_family(pts) -> list[str]:
    """Every Init and constructor of GeomAPI_PointsToBSpline on one point set."""
    out = []
    for build in (0, 5):
        for s in (0, 2, 4, 6):
            out.append(fit(build, pts, (3, 8, s, 1e-3)))
            out.append(fit(build, pts, (2, 5, s, 1e-1)))
        for pt in (0, 1, 2):
            out.append(fit(build + 1, pts, (pt, 3, 8, 4, 1e-3)))
        params = [float(i) ** 1.2 for i in range(len(pts))]
        out.append(fit(build + 2, pts, (3, 8, 4, 1e-3), params))
        out.append(fit(build + 2, pts, (3, 8, 4, 1e-3), [5.0] * len(pts)))
        out.append(fit(build + 2, pts, (3, 8, 4, 1e-3), params[:-1]))
        out.append(fit(4 if build == 0 else 8, pts, (1.0, 0.0, 0.0, 8, 4, 1e-3)))
        out.append(fit(4 if build == 0 else 8, pts, (0.5, 0.3, 0.2, 6, 2, 1e-2)))
    return out


def append_points_to_bspline_requests(requests: list[str]) -> None:
    """GeomAPI_PointsToBSpline on smooth, noisy and small point sets."""
    smooth = [(0.1 * i, math.sin(0.1 * i), 0.05 * (0.1 * i) ** 2) for i in range(15)]
    rng = random.Random(311)
    noisy = [(0.2 * i, rng.uniform(-0.5, 0.5), rng.uniform(-0.2, 0.2)) for i in range(12)]
    small = [(0.0, 0.0, 0.0), (1.0, 0.5, 0.0), (2.0, 0.0, 0.0)]
    for pts in (smooth, noisy, small):
        requests.extend(fit_family(pts))


def append_geomapi_fit_requests(requests: list[str]) -> None:
    """Append the GeomAPI plane, intersection and fitting requests in a fixed order."""
    append_to2d_requests(requests)
    append_to3d_requests(requests)
    append_intcs_requests(requests)
    append_interpolate_requests(requests)
    append_points_to_bspline_requests(requests)
    __import__('tkgeomalgo_corpus_geomapi_surface').append_p2bss_requests(requests)
