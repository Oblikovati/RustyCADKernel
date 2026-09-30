"""Deterministic requests for GeomFill_SweepSectionGenerator, GeomFill_AppSweep and GeomFill_Pipe (tools/oracle/tkgeomalgo_probe_geomfill_pipe.cxx).

`gfs_sweepsec` builds the section generator of every kind (a tube, one section,
two sections, along a curve or a circle, and the circles between two rails), then
prints its sections (with the end derivatives), parameters, transformations (along a
curve they stay the identity, a native quirk) and the AppSweep approximation.
"""
from __future__ import annotations
from tkgeomalgo_corpus_intcurvesurface import ad, bezier, bspline, circle, ellipse, line, line_of, trimmed

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


def append_pipe_requests(requests: list[str]) -> None:
    """Append every section generator and pipe request in a fixed order."""
    _sweepsec(requests)
