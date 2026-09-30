"""Deterministic requests for GeomFill_SweepSectionGenerator, GeomFill_AppSweep and GeomFill_Pipe (tools/oracle/tkgeomalgo_probe_geomfill_pipe.cxx).

`gfs_sweepsec` builds the section generator of every kind (a tube, one section,
two sections, along a curve or a circle, and the circles between two rails), then
prints its sections (with the end derivatives), parameters, transformations (along a
curve they stay the identity, a native quirk) and the AppSweep approximation.
`gfs_pipe` builds pipes of every constructor (tube, section with each trihedron
option, path on a surface, constant binormal, two and N sections, blends between
rails, guided sections) and performs them. The empty pipe is left out: its Perform
reads null adaptors (a crash natively).
"""
from __future__ import annotations
from tkgeomalgo_corpus_geom2d import c_line, c_trimmed
from tkgeomalgo_corpus_intcurvesurface import ad, bezier, bspline, circle, cyl, ellipse, line, line_of, plane, trimmed

X, Y, Z = [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]
O = [0.0, 0.0, 0.0]

STRAIGHT = trimmed(0.0, 4.0, line(O, Z))
ROUND = trimmed(0.0, 2.0, circle(O, Z, 3.0))
WAVE = bspline(3, [[0, 0, 0], [1, 2, 0.5], [2, -2, 1], [3, 2, 1.5], [4, -2, 2], [5, 0, 2.5]], [0.0, 1.0, 2.0, 3.0],
               [4, 1, 1, 4])
ARC = bezier([[0, 0, 0], [2, 0, 1], [3, 2, 2]])
SEC_CIRCLE = circle(O, Z, 0.5)
SEC_ELLIPSE = ellipse(O, Z, 0.8, 0.4)
SEC_SPLINE = bspline(2, [[0.5, 0, 0], [0.5, 0.5, 0], [-0.5, 0.5, 0], [-0.5, 0, 0]], [0.0, 1.0, 2.0], [3, 1, 3])
SEC_SPLINE2 = bspline(3, [[0.4, 0, 0], [0.6, 0.4, 0.1], [-0.2, 0.7, 0], [-0.6, 0.1, 0.1], [-0.4, 0, 0]],
                      [0.0, 0.5, 1.0], [4, 1, 4])


def _sweepsec(requests: list[str]) -> None:
    for path in (STRAIGHT, ROUND, WAVE, ARC):
        for poly in (0, 1):
            requests.append(line_of('gfs_sweepsec', [1, *path, 0.5, poly, 0]))
        requests.append(line_of('gfs_sweepsec', [2, *path, *SEC_ELLIPSE, 0, 1]))
        requests.append(line_of('gfs_sweepsec', [2, *path, *SEC_SPLINE, 0, 0]))
        requests.append(line_of('gfs_sweepsec', [3, *path, *SEC_CIRCLE, *SEC_ELLIPSE, 0, 0]))
        requests.append(line_of('gfs_sweepsec', [3, *path, *SEC_SPLINE, *SEC_SPLINE2, 0, 1]))

    def ranged(c: list[float]) -> list[float]:
        return [0, *ad(c, (0.0, 4.0))]
    axis = ranged(line(O, Z))
    rail1 = bspline(3, [[1, 0, 0], [1.2, 0, 1], [0.9, 0.1, 2], [1.1, 0, 3], [1, 0, 4]], [0.0, 1.0, 4.0], [4, 1, 4])
    for c1, c2 in ((ranged(line(X, Z)), ranged(line(Y, Z))), (ranged(line(X, Z)), ranged(line([-1.0, 0.2, 0.0], Z))),
                   ([0, *ad(rail1)], ranged(line(Y, Z))), (ranged(line(X, Z)), ranged(line(X, Z)))):
        for poly, withp in ((0, 0), (1, 1)):
            requests.append(line_of('gfs_sweepsec', [0, *axis, *c1, *c2, 1.0, poly, withp]))
    requests.append(line_of('gfs_sweepsec', [0, *axis, *[0, *line(X, Z), 0], *ranged(line(Y, Z)), 1.0, 0, 0]))


HELIX = bspline(3, [[1.2, 0, 0], [1.2, 1.2, 1], [-1.2, 1.2, 2], [-1.2, 0, 3]], [0.0, 1.0], [4, 4])
# kpart mode tol poly conti degmax nbseg withparams
LAW_PERFORM = [1, 0, 1e-4, 0, 2, 11, 30, 0]
PERFORMS = [LAW_PERFORM, [0, 0, 1e-3, 0, 4, 8, 20, 0], [1, 1, 0.0, 0, 0, 0, 0, 0]]


def _pipe(requests: list[str], ctor: list[float], performs: list[list[float]] | None = None) -> None:
    for perform in performs or PERFORMS:
        requests.append(line_of('gfs_pipe', [*ctor, *perform]))


def _law_pipes(requests: list[str]) -> None:
    for path in (STRAIGHT, ROUND, WAVE, ARC):
        _pipe(requests, [1, *path, 0.5])
        for option in range(5):
            for sect in (SEC_CIRCLE, SEC_SPLINE):
                _pipe(requests, [2, *path, *sect, option], [LAW_PERFORM])
        _pipe(requests, [6, *path, *SEC_ELLIPSE, 0.0, 0.0, 1.0], [LAW_PERFORM])
    _pipe(requests, [2, *WAVE, *SEC_CIRCLE, 5], [LAW_PERFORM])
    _pipe(requests, [2, *WAVE, *SEC_CIRCLE, 9], [LAW_PERFORM])
    _pipe(requests, [3, *c_trimmed(0.0, 2.0, c_line(0.0, 0.0, 1.0, 0.5)), *plane(O, Z), *SEC_CIRCLE])
    _pipe(requests, [3, *c_trimmed(0.0, 2.0, c_line(0.0, 0.0, 1.0, 0.3)), *cyl(O, Z, 2.0), *SEC_ELLIPSE])


def _section_pipes(requests: list[str]) -> None:
    for path in (STRAIGHT, WAVE, ARC):
        _pipe(requests, [4, *path, *SEC_CIRCLE, *SEC_ELLIPSE])
        _pipe(requests, [4, *path, *SEC_SPLINE, *SEC_SPLINE2], [LAW_PERFORM])
    _pipe(requests, [4, *STRAIGHT, *SEC_CIRCLE, *circle(O, Z, 1.0)])
    moved = [circle([0.0, 0.0, z], Z, 0.5 + 0.2 * z) for z in (0.0, 1.5, 3.0)]
    _pipe(requests, [5, *STRAIGHT, 3, *moved[2], *moved[0], *moved[1]])
    _pipe(requests, [5, *WAVE, 2, *SEC_CIRCLE, *SEC_ELLIPSE], [LAW_PERFORM])
    _pipe(requests, [5, *STRAIGHT, 2, *moved[0], *moved[0]], [LAW_PERFORM])


def _blend_pipes(requests: list[str]) -> None:
    def ranged(c: list[float]) -> list[float]:
        return [0, *ad(c, (0.0, 4.0))]
    geo = (trimmed(0.0, 4.0, line(O, Z)), trimmed(0.0, 4.0, line(X, Z)), trimmed(0.0, 4.0, line(Y, Z)))
    _pipe(requests, [7, *geo[0], *geo[1], *geo[2], 1.0])
    _pipe(requests, [7, *geo[0], *geo[1], *trimmed(0.0, 4.0, line([-1.0, 0.2, 0.0], Z)), 1.0])
    rings = (trimmed(0.0, 2.0, circle(O, Z, 3.0)), trimmed(0.0, 2.0, circle(O, Z, 4.0)),
             trimmed(0.0, 2.0, circle([0.0, 0.0, 1.0], Z, 3.0)))
    _pipe(requests, [7, *rings[0], *rings[1], *rings[2], 1.0])
    _pipe(requests, [7, *rings[0], *rings[2], *rings[1], 1.0])
    rail1 = bspline(3, [[1, 0, 0], [1.2, 0, 1], [0.9, 0.1, 2], [1.1, 0, 3], [1, 0, 4]], [0.0, 1.0, 4.0], [4, 1, 4])
    _pipe(requests, [8, *ranged(line(O, Z)), 0, *ad(rail1), *ranged(line(Y, Z)), 1.0])
    _pipe(requests, [8, *ranged(line(O, Z)), *ranged(line(X, Z)), *ranged(line(Y, Z)), 1.0],
          [[1, 1, 0.0, 1, 0, 0, 0, 1], [0, 0, 1e-4, 1, 2, 11, 30, 0]])


def _guide_pipes(requests: list[str]) -> None:
    axis = trimmed(0.0, 3.0, line(O, Z))
    for acr in (1, 0):
        for rot in (0, 1):
            _pipe(requests, [9, *axis, 0, *ad(HELIX), *circle(O, Z, 1.2), acr, rot], [LAW_PERFORM])


def append_pipe_requests(requests: list[str]) -> None:
    """Append every section generator and pipe request in a fixed order."""
    _sweepsec(requests)
    _law_pipes(requests)
    _section_pipes(requests)
    _blend_pipes(requests)
    _guide_pipes(requests)
