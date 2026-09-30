"""Deterministic requests for the GeomFill section and location laws (tools/oracle/tkgeomalgo_probe_geomfill_laws.cxx).

`gfs_section` builds GeomFill_UniformSection and GeomFill_EvolvedSection laws on
circles (converted to periodic quasi-angular B-splines that lose a knot), trimmed
circles, lines, Bezier curves and rational or periodic B-splines, evolved by constant,
linear and B-spline laws; it prints every query, the evaluators on arrays of the
right size, of one pole less and of other bounds (the native assignment resizes
them), SetInterval and the evaluators again.
"""
from __future__ import annotations
from tkgeomalgo_corpus_intcurvesurface import bezier, bspline, circle, line, line_of, plane, trimmed

Z = [0.0, 0.0, 1.0]

SECTIONS = [
    circle([0.0, 0.0, 0.0], Z, 1.5),
    trimmed(0.2, 2.5, circle([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 2.0)),
    trimmed(-1.0, 2.0, line([0.0, 0.0, 0.0], [1.0, 1.0, 0.0])),
    bezier([[0, 0, 0], [1, 2, 0], [3, 1, 0]], [1.0, 2.0, 1.0]),
    bspline(3, [[0, 0, 0], [1, 2, 0], [2, -1, 0], [3, 1, 0], [4, 0, 0]], [0.0, 0.5, 1.0], [4, 1, 4]),
    bspline(2, [[1, 0, 0], [0, 1, 0], [-1, 0, 0], [0, -1, 0]], [0.0, 1.0, 2.0, 3.0, 4.0], [1, 1, 1, 1, 1],
            weights=[1.0, 2.0, 1.0, 2.0], periodic=True),
]
LAWS = [
    [0, 2.0, 0.0, 1.0],
    [1, 0.0, 1.0, 1.0, 3.0],
    [2, 3, 0, 0, 5, 1.0, 2.0, 0.5, 1.5, 1.0, 3, 0.0, 0.5, 1.0, 3, 4, 1, 4, 0.0, 1.0],
]


ROT = [0.0, -1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]
WAVE = bspline(3, [[0, 0, 0], [1, 2, 0.5], [2, -2, 1], [3, 2, 1.5], [4, -2, 2], [5, 0, 2.5]],
               [0.0, 1.0, 2.0, 3.0], [4, 1, 1, 4])
HELIX = bspline(3, [[1.2, 0, 0], [1.2, 1.2, 1], [-1.2, 1.2, 2], [-1.2, 0, 3]], [0.0, 1.0], [4, 4])
AXIS = [0, *line([0.0, 0.0, 0.0], Z), 1, 0.0, 3.0]


def _loc(law: list[float], path: list[float], params: list[float], interval: tuple, trsf: list[float] | None,
         tol: float = 1e-4) -> str:
    tail = [1, *trsf] if trsf else [0]
    return line_of('gfs_location', [*law, 0, *path, len(params), *params, *interval, *tail, tol])


def _locations(requests: list[str]) -> None:
    wave = [*WAVE, 0]
    circ = [*circle([0.0, 0.0, 0.0], Z, 2.0), 1, 0.0, 6.0]
    straight = [*line([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]), 1, 0.0, 4.0]
    bez = [*bezier([[0, 0, 0], [1, 2, 0], [2, -1, 1], [3, 1, 2]]), 0]
    for law in ([0, 1], [0, 2, 0], [0, 5], [0, 0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0], [0, 6, 0.0, 0.0, 1.0, 0.3]):
        for path, params, interval in ((wave, [0.0, 1.2, 3.0], (0.5, 2.5)), (circ, [0.0, 2.0, 6.0], (1.0, 5.0)),
                                       (straight, [0.0, 1.0, 4.0], (0.5, 3.5)), (bez, [0.0, 0.5, 1.0], (0.2, 0.9))):
            requests.append(_loc(law, path, params, interval, ROT))
    for angle in (0.1, -0.3):
        requests.append(_loc([1, 0.0, 0.0, 1.0, angle, 0], straight, [0.0, 2.0, 4.0], (1.0, 3.0), ROT))
        requests.append(_loc([1, 0.0, 0.0, 1.0, angle, 1, *plane([0.0, 0.0, -3.0], Z), 0], straight,
                             [0.0, 1.3, 4.0], (1.0, 3.0), None))
        requests.append(_loc([1, 0.3, 0.0, 1.0, angle, 1, *plane([0.0, 0.0, -3.0], Z), 0], bez,
                             [0.0, 0.4, 1.0], (0.2, 0.8), ROT))
    circle_section = [0, *circle([0.0, 0.0, 0.0], Z, 1.2), 0.0, 1.0]
    evolved = [1, *circle([0.0, 0.0, 0.0], Z, 1.0), 1, 0.0, 1.0, 1.0, 1.5]
    for kind in (7, 8):
        guide = [kind, 0, *HELIX, 0]
        requests.append(_loc([2, *guide, 0], AXIS[1:], [0.0, 1.5, 3.0], (0.5, 2.5), None))
        for section, rotat in ((circle_section, 1), (circle_section, 0), (evolved, 0), (evolved, 1)):
            requests.append(_loc([2, *guide, 1, *section, rotat, 0.0, 1.0, 0.0], AXIS[1:], [0.0, 1.5, 3.0],
                                 (0.5, 2.5), ROT if rotat == 0 else None))


def append_laws_requests(requests: list[str]) -> None:
    """Append every section and location law request family in a fixed order."""
    params = [0, 0.25, 0.8]
    for section in SECTIONS:
        requests.append(line_of('gfs_section', [0, *section, 0.0, 1.0, len(params), *params, 1, 0.1, 0.9]))
        requests.append(line_of('gfs_section', [0, *section, -1.0, 2.0, len(params), *params, 3, 0.0, 1.0]))
        for law in LAWS:
            requests.append(line_of('gfs_section', [1, *section, *law, len(params), *params, 1, 0.2, 0.7]))
    _locations(requests)
