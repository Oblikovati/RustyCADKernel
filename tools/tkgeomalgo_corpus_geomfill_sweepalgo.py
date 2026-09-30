"""Deterministic requests for GeomFill_SweepFunction and GeomFill_Sweep (tools/oracle/tkgeomalgo_probe_geomfill_sweepalgo.cxx).

`gfs_sweepfunc` places uniform and evolved sections by curve-and-trihedron, draft and
guided location laws; it prints the queries (the intervals fuse the section and path
intervals), D0/D1/D2 with the 2d poles of the laws, SetInterval and the evaluators
again. `gfs_sweep` builds sweeps twice (with two continuities): the elementary cases
of a translation (plane, cylinder, extrusion, cone) and of a rotation (sphere, torus,
cylinder, cone, surface of revolution), and the approximations with their
restrictions, traces and the optional C1 re-approximation. The second build keeps the
2d curves of the first one, as natively. `gfs_blendfunc` evaluates the circular
blend sections between two rails around a path, in the three arc parameterisations.
"""
from __future__ import annotations
from tkgeomalgo_corpus_intcurvesurface import ad, bezier, bspline, circle, line, line_of, plane, trimmed

X, Y, Z = [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]
O = [0.0, 0.0, 0.0]

FIXED_Z = [0, 0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0]
FRENET = [0, 1]
CORRECTED = [0, 2, 2]
BINORMAL_Z = [0, 3, 0.0, 0.0, 1.0]

STRAIGHT = [0, *ad(line(O, Z), (0.0, 4.0))]
ROUND = [0, *ad(circle(O, Z, 3.0), (0.0, 2.0))]
FULL_ROUND = [0, *ad(circle(O, Z, 3.0))]
WAVE = [0, *ad(bspline(3, [[0, 0, 0], [1, 2, 0.5], [2, -2, 1], [3, 2, 1.5], [4, -2, 2], [5, 0, 2.5]],
                       [0.0, 1.0, 2.0, 3.0], [4, 1, 1, 4]))]
ARC = [0, *ad(bezier([[0, 0, 0], [2, 0, 1], [3, 2, 2]]))]
HELIX = bspline(3, [[1.2, 0, 0], [1.2, 1.2, 1], [-1.2, 1.2, 2], [-1.2, 0, 3]], [0.0, 1.0], [4, 4])


def uniform(curve: list[float], f: float, l: float) -> list[float]:
    return [0, *curve, f, l]


def evolved(curve: list[float], law: list[float]) -> list[float]:
    return [1, *curve, *law]


CIRCLE_SEC = uniform(circle(O, Z, 1.0), 0.0, 1.0)
TILTED_SEC = uniform(circle(O, [0.0, 0.6, 0.8], 1.0), 0.0, 1.0)
LINE_SEC = uniform(trimmed(-1.0, 1.0, line(O, X)), 0.0, 1.0)
LINE_AXIAL_SEC = uniform(trimmed(0.0, 2.0, line([1.0, 0.0, 0.0], Y)), 0.0, 1.0)
LINE_OBLIQUE_SEC = uniform(trimmed(0.0, 2.0, line([1.0, 0.0, 0.0], [0.6, 0.8, 0.0])), 0.0, 1.0)
SPLINE_SEC = uniform(bspline(3, [[1, 0, 0], [1, 1, 0], [0, 2, 0], [-1, 1, 0], [-1, 0, 0]], [0.0, 0.5, 1.0],
                             [4, 1, 4]), 0.0, 1.0)
ARC_SEC = uniform(trimmed(0.3, 2.5, circle(O, Z, 0.8)), 0.0, 1.0)
# Rotation sections, in the plane of the path frame (the section X and Y map to the
# frame normal and binormal).
TORUS_SEC = uniform(circle([1.0, 0.0, 0.0], Z, 0.5), 0.0, 1.0)
SPHERE_SEC = uniform(trimmed(-1.2, 1.2, circle([3.0, 0.0, 0.0], Z, 1.0)), 0.0, 1.0)
CONE_EVOLVED = evolved(circle(O, Z, 1.0), [1, 0.0, 1.0, 4.0, 2.0])
CONE_EVOLVED_SHRINK = evolved(circle(O, Z, 1.0), [1, 0.0, 2.0, 4.0, 0.5])
PERIODIC_SEC = uniform(bspline(2, [[1, 0, 0], [0, 1, 0], [-1, 0, 0], [0, -1, 0]], [0.0, 1.0, 2.0, 3.0, 4.0],
                               [1, 1, 1, 1, 1], periodic=True), 0.0, 1.0)

DEFAULT_TOL = [1e-4, 1.0, 1e-5, 1.0]


def _sweep(loc: list[float], path: list[float], section: list[float], kpart: int = 1,
           domain: list[float] | None = None, tol: list[float] | None = None, forcec1: int = 0,
           methode: int = 1, cont: int = 4, degmax: int = 10, segmax: int = 30, cont2: int = 2) -> str:
    dom = [1, *domain] if domain else [0]
    return line_of('gfs_sweep', [*loc, *path, *section, kpart, *dom, *(tol or DEFAULT_TOL), forcec1, methode,
                                 cont, degmax, segmax, cont2])


def _sweeps(requests: list[str]) -> None:
    for loc in (FIXED_Z, FRENET, BINORMAL_Z):
        for section in (CIRCLE_SEC, TILTED_SEC, LINE_SEC, SPLINE_SEC, ARC_SEC, CONE_EVOLVED,
                        CONE_EVOLVED_SHRINK, PERIODIC_SEC):
            requests.append(_sweep(loc, STRAIGHT, section))
    for loc in (FRENET, CORRECTED, BINORMAL_Z):
        for section in (TORUS_SEC, SPHERE_SEC, LINE_AXIAL_SEC, LINE_OBLIQUE_SEC, SPLINE_SEC, CIRCLE_SEC):
            requests.append(_sweep(loc, ROUND, section))
            requests.append(_sweep(loc, FULL_ROUND, section))
    for loc in (FRENET, CORRECTED):
        for section in (CIRCLE_SEC, SPLINE_SEC, CONE_EVOLVED):
            requests.append(_sweep(loc, WAVE, section, cont=2, cont2=1))
            requests.append(_sweep(loc, ARC, section, kpart=0, cont=4, cont2=0))
    requests.append(_sweep(FIXED_Z, STRAIGHT, CIRCLE_SEC, kpart=0))
    requests.append(_sweep(FRENET, ROUND, TORUS_SEC, kpart=0, forcec1=1))
    requests.append(_sweep(FRENET, WAVE, CIRCLE_SEC, forcec1=1, cont=0, cont2=0, segmax=4, degmax=3))
    requests.append(_sweep(FRENET, WAVE, CIRCLE_SEC, domain=[0.5, 2.5, 0.0, 1.0]))
    requests.append(_sweep(FIXED_Z, STRAIGHT, CIRCLE_SEC, domain=[1.0, 3.0, 0.2, 0.8]))
    requests.append(_sweep(FRENET, ARC, SPLINE_SEC, methode=0, tol=[1e-2, 1e-3, 1e-4, 0.1]))
    requests.append(_sweep(FRENET, WAVE, SPLINE_SEC, tol=[1e-7, 1e-7, 1e-7, 1e-3], degmax=6, segmax=5))
    _nsection_sweeps(requests)
    _guided_sweeps(requests)


def _nsection_sweeps(requests: list[str]) -> None:
    """N-section laws: two equal circles (a constant law), coaxial circles (a cone
    along a translation) and B-spline sections (the approximation)."""
    two_circles = [2, 1, 2, *circle(O, Z, 1.0), *circle(O, Z, 1.0), 2, 0.0, 1.0]
    cone = [2, 3, 2, *circle(O, Z, 1.0), *circle(O, Z, 2.0), 2, 0.0, 4.0, 0.0, 6.283185307179586, 0.0, 4.0]
    shrink = [2, 3, 2, *circle(O, Z, 2.0), *circle(O, Z, 0.5), 2, 0.0, 4.0, 0.0, 6.283185307179586, 0.0, 4.0]
    splines = [2, 1, 2, *SPLINE_SEC[1:-2], *bspline(3, [[1, 0, 0], [1.5, 1, 0], [0, 2.5, 0], [-1.5, 1, 0], [-1, 0, 0]],
                                                     [0.0, 0.5, 1.0], [4, 1, 4]), 2, 0.0, 1.0]
    for loc in (FIXED_Z, FRENET):
        for section in (two_circles, cone, shrink, splines):
            requests.append(_sweep(loc, STRAIGHT, section, cont=2, cont2=1))
    requests.append(_sweep(FRENET, WAVE, splines, cont=2, cont2=1))
    requests.append(_sweep(FRENET, ROUND, cone, cont=2, cont2=1))


def _guided_sweeps(requests: list[str]) -> None:
    section = [0, *circle(O, Z, 1.2), 0.0, 1.0]
    axis = [0, *ad(line(O, Z), (0.0, 3.0))]
    for kind in (7, 8):
        guide = [kind, 0, *HELIX, 0]
        loc = [2, *guide, 1, *section, 0, 0.0, 1.0, 0.0]
        requests.append(_sweep(loc, axis, section, cont=2, cont2=1))
    draft = [1, 0.0, 0.0, 1.0, 0.2, 1, *plane([0.0, 0.0, -3.0], Z), 0]
    requests.append(_sweep(draft, [0, *ad(line(O, X), (0.0, 4.0))], LINE_SEC, cont=2, cont2=1))


def _functions(requests: list[str]) -> None:
    params = [0.0, 0.7, 1.9]
    for loc, path in ((FRENET, WAVE), (FIXED_Z, STRAIGHT), (CORRECTED, ROUND)):
        for section in (CIRCLE_SEC, SPLINE_SEC, CONE_EVOLVED, PERIODIC_SEC):
            requests.append(line_of('gfs_sweepfunc', [*loc, *path, *section, 0.0, 0.0, 0.25, len(params), *params,
                                                      0.5, 1.5]))
    requests.append(line_of('gfs_sweepfunc', [*FRENET, *WAVE, *SPLINE_SEC, 1.0, 0.2, -0.5, 2, 1.0, 2.0, 1.2, 2.8]))
    section = [0, *circle(O, Z, 1.2), 0.0, 1.0]
    axis = [0, *ad(line(O, Z), (0.0, 3.0))]
    for kind in (7, 8):
        loc = [2, kind, 0, *HELIX, 0, 1, *section, 0, 0.0, 1.0, 0.0]
        requests.append(line_of('gfs_sweepfunc', [*loc, *axis, *section, 0.0, 0.0, 1.0 / 3.0, 2, 0.5, 2.5, 0.5, 2.5]))
    draft = [1, 0.0, 0.0, 1.0, 0.2, 1, *plane([0.0, 0.0, -3.0], Z), 0]
    requests.append(line_of('gfs_sweepfunc', [*draft, 0, *ad(line(O, X), (0.0, 4.0)), *LINE_SEC, 0.0, 0.0, 0.25,
                                              2, 1.0, 3.0, 1.0, 3.0]))


def _blend(requests: list[str]) -> None:
    """`gfs_blendfunc`: straight and curved rails at right, obtuse and flat angles."""
    def ranged(c: list[float]) -> list[float]:
        return [0, *ad(c, (0.0, 4.0))]
    axis = ranged(line(O, Z))
    right = (ranged(line(X, Z)), ranged(line(Y, Z)))
    obtuse = (ranged(line(X, Z)), ranged(line([-1.0, 0.1, 0.0], Z)))
    flat = (ranged(line(X, Z)), ranged(line([-1.0, 0.0, 0.0], Z)))
    wavy = bspline(3, [[0, 0, 0], [0.2, 0.1, 1], [-0.1, 0.2, 2], [0.1, -0.1, 3], [0, 0, 4]], [0.0, 2.0, 4.0],
                   [4, 1, 4])
    rail1 = bspline(3, [[1, 0, 0], [1.2, 0, 1], [0.9, 0.1, 2], [1.1, 0, 3], [1, 0, 4]], [0.0, 1.0, 4.0], [4, 1, 4])
    rail2 = bspline(2, [[0, 1, 0], [0.1, 1.1, 1.3], [0, 0.9, 2.2], [0.05, 1.05, 3.2], [0, 1, 4]],
                    [0.0, 1.5, 3.0, 4.0], [3, 1, 1, 3])
    curved = ([0, *ad(rail1)], [0, *ad(rail2)])
    params = [0.0, 1.3, 4.0]
    for path, (c1, c2) in ((axis, right), (axis, obtuse), (axis, flat), ([0, *ad(wavy)], curved)):
        for radius, poly in ((1.0, 0), (1.3, 0), (1.0, 1)):
            requests.append(line_of('gfs_blendfunc', [*path, *c1, *c2, radius, poly, len(params), *params, 0.5, 3.5]))
    requests.append(line_of('gfs_blendfunc', [*axis, *right[0], *right[0], 1.0, 0, 1, 1.0, 0.5, 3.5]))


def append_sweepalgo_requests(requests: list[str]) -> None:
    """Append every sweep function and sweep request in a fixed order."""
    _functions(requests)
    _sweeps(requests)
    _blend(requests)
