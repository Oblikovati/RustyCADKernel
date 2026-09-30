"""Deterministic requests for GeomFill_SectionPlacement (tools/oracle/tkgeomalgo_probe_geomfill_placement.cxx).

`gfs_placement` places point sections (on and off the path) and curve sections
(conics, whose axis is the plane normal; planar and non-planar Bezier and B-spline
curves, whose axis comes from the inertia of sample points; lines, which have no
plane; periodic and trimmed curves) on lines, circles and B-spline paths under
several trihedron laws. Each request prints the state after the constructor and
after Perform(Tol), Perform(Param, Tol) and Perform(Path, Tol), with the four
transformations, the placed and corrected sections. The correction
of a non-planar section raises Standard_Failure.
"""
from __future__ import annotations
from tkgeomalgo_corpus_intcurvesurface import ad, bezier, bspline, circle, ellipse, line, line_of, trimmed

X, Y, Z = [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]
O = [0.0, 0.0, 0.0]

# name -> (path source, parameter for Perform(Param, Tol))
PATHS = {
    'line': ([0, *ad(line(O, Z), (0.0, 5.0))], 2.0),
    'circle': ([0, *ad(circle([0.0, 0.0, 0.0], Y, 4.0), (0.0, 3.0))], 1.0),
    'wave': ([0, *ad(bspline(3, [[0, 0, 0], [1, 2, 0.5], [2, -2, 1], [3, 2, 1.5], [4, -2, 2], [5, 0, 2.5]],
                             [0.0, 1.0, 2.0, 3.0], [4, 1, 1, 4]))], 1.5),
    'helix': ([0, *ad(bspline(3, [[1.2, 0, 0], [1.2, 1.2, 1], [-1.2, 1.2, 2], [-1.2, 0, 3]], [0.0, 1.0], [4, 4]))],
              0.4),
    'bezier': ([0, *ad(bezier([[0, 0, 0], [0, 1, 2], [1, 0, 4], [2, 1, 5]]))], 0.5),
}

LAWS = [[1], [2, 2], [0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0], [3, 0.0, 1.0, 0.0], [5]]

SECTIONS = {
    'point_on': [1, 0.0, 0.0, 0.0],
    'point_off': [1, 1.0, 2.0, 0.5],
    'circle_start': [0, *circle(O, Z, 1.0)],
    'circle_mid': [0, *circle([0.0, 0.0, 2.5], Z, 0.8)],
    'circle_side': [0, *circle([3.0, 0.0, 1.0], X, 0.5)],
    'ellipse': [0, *ellipse([0.2, 0.1, 0.0], [0.0, 0.3, 1.0], 1.5, 0.5)],
    'trimmed_circle': [0, *trimmed(0.3, 2.0, circle([0.0, 0.0, 1.0], Z, 1.0))],
    'line': [0, *trimmed(-1.0, 1.0, line([0.5, 0.0, 0.0], X))],
    'planar_bspline': [0, *bspline(3, [[1, 0, 0], [1, 1, 0], [0, 2, 0], [-1, 1, 0], [-1, 0, 0]],
                                   [0.0, 0.5, 1.0], [4, 1, 4])],
    'skew_bspline': [0, *bspline(2, [[1, 0, 0], [0, 1, 1], [-1, 0, 0], [0, -1, 2], [1, 0, 3]],
                                 [0.0, 1.0, 2.0, 3.0], [3, 1, 1, 3])],
    'periodic_bspline': [0, *bspline(2, [[1, 0, 0.5], [0, 1, 0.5], [-1, 0, 0.5], [0, -1, 0.5]],
                                     [0.0, 1.0, 2.0, 3.0, 4.0], [1, 1, 1, 1, 1], periodic=True)],
    'planar_bezier': [0, *bezier([[0, 0, 1], [1, 1, 1], [2, 0, 1], [3, 1, 1]])],
    'skew_bezier': [0, *bezier([[0, 0, 0], [1, 1, 1], [2, 0, 3], [3, 1, 1]])],
}


def append_placement_requests(requests: list[str]) -> None:
    """Append every section placement request in a fixed order."""
    for law in LAWS:
        for path, param in PATHS.values():
            for section in SECTIONS.values():
                requests.append(line_of('gfs_placement', [*law, *path, *section, 1e-4, param]))
    for tol in (1e-7, 0.5, 5.0):
        path, param = PATHS['wave']
        for section in (SECTIONS['skew_bspline'], SECTIONS['circle_side'], SECTIONS['point_off']):
            requests.append(line_of('gfs_placement', [1, *path, *section, tol, param]))
