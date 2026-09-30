"""Deterministic requests for the IntStart templates (tools/oracle/tkgeomalgo_probe_intstart.cxx),
over the surface catalog of tkgeomalgo_corpus_intcurvesurface.py.

`is_sob` runs IntStart_SearchOnBoundaries.gxx on the restrictions of every catalog surface
against every quadric kind, with the IntPatch tools, with and without the regularity
recheck, with an edge behind the domain (degenerated, straight and tangent to the
cylinder, regular) and with the probe tool whose arcs have been seen (known points and
segments, out-of-range indices). `is_inside` runs IntStart_SearchInside.gxx: the sampled
search on every surface against every quadric, the search from one start point (inside,
outside and on the bounds) and the consultation without a Perform.
"""
from __future__ import annotations
from tkgeomalgo_corpus_intcurvesurface import O, SURFACES, TWO_PI, X, Y, Z, cyl, line_of, plane, sad, sphere

# Quadrics `kind loc dir xdir r1 r2`: plane, cylinder, sphere, cone (r1 radius, r2
# semi-angle) and torus.
QUADRICS = {
    'q_plane': [0, 0.0, 0.0, 0.1, *Z, *X, 0.0, 0.0],
    'q_plane_tilt': [0, 0.2, 0.1, 0.0, 0.3, 0.1, 1.0, 1.0, 0.0, -0.3, 0.0, 0.0],
    'q_cyl': [1, 0.0, 0.0, 0.0, *Z, *X, 1.2, 0.0],
    'q_cyl_x': [1, 0.0, 0.3, 0.0, *X, *Y, 0.7, 0.0],
    'q_sphere': [2, 0.0, 0.0, 0.0, *Z, *X, 1.3, 0.0],
    'q_sphere_off': [2, 0.5, -0.4, 0.2, *Z, *X, 0.9, 0.0],
    'q_cone': [3, 0.0, 0.0, 0.0, *Z, *X, 0.8, 0.5],
    'q_cone_flat': [3, 0.0, 0.0, 0.0, *Z, *X, 0.8, 0.01],
    'q_torus': [4, 0.0, 0.0, 0.0, *Z, *X, 1.5, 0.6],
}
# Surfaces with finite bounds, whose domain has four restrictions (and the natural
# periodic or infinite ones of the catalog).
SEARCH_SURFACES = {name: s for name, s in SURFACES.items()}
SEARCH_SURFACES.update({
    'plane_small': sad(plane(O, Z), (-1.0, 1.0, -1.0, 1.0)),
    'cyl_band': sad(cyl(O, Z, 1.0), (0.0, TWO_PI, -1.0, 1.0)),
    'sphere_cap': sad(sphere(O, Z, 1.0), (0.0, TWO_PI, 0.2, 1.4)),
})
NO_EDGE = [0]


def _edge(p, d, first, last, degenerated=0, regular=0) -> list[float]:
    return [1, *p, *d, first, last, degenerated, regular]


def _sob(requests: list[str], surface, quadric, tols=(1.0e-7, 1.0e-6), recheck=0, mode=0,
         edge=NO_EDGE, seen=None) -> None:
    body = [1 if seen is not None else 0, *surface, *quadric, *tols, recheck, mode, *edge]
    requests.append(line_of('is_sob', body + (seen or [])))


def _seen_table(points, segments, all_solution) -> list[float]:
    return [len(points), *[x for p in points for x in p], len(segments),
            *[x for s in segments for x in s], all_solution]


def _sob_family(requests: list[str]) -> None:
    for s in SEARCH_SURFACES.values():
        for q in QUADRICS.values():
            _sob(requests, s, q)
    small, band = SEARCH_SURFACES['plane_small'], SEARCH_SURFACES['cyl_band']
    for q in QUADRICS.values():
        _sob(requests, small, q, recheck=1)
        _sob(requests, band, q, tols=(1.0e-4, 1.0e-3), mode=1)
    _sob(requests, small, QUADRICS['q_plane'], mode=2)
    # An arc on the quadric: the plane itself, with and without a regular edge.
    on_plane = [0, 0.0, 0.0, 0.0, *Z, *X, 0.0, 0.0]
    for recheck in (0, 1):
        _sob(requests, small, on_plane, recheck=recheck)
        _sob(requests, small, on_plane, recheck=recheck, edge=_edge(O, X, -1.0, 1.0, 0, 1))
    # The same plane shifted by less than the tolerances: tangent zeros.
    near_plane = [0, 0.0, 0.0, 2.0e-7, *Z, *X, 0.0, 0.0]
    _sob(requests, small, near_plane, recheck=1, edge=_edge(O, X, -1.0, 1.0, 0, 1))
    # A cylinder tangent to the boundary lines x = +-1 of the square: TreatLC with a
    # straight edge tangent to it, a degenerated edge, and an edge off the cylinder.
    tangent_cyl = [1, 0.0, 0.0, 0.0, *Z, *X, 1.0, 0.0]
    for edge in (_edge([1.0, -1.0, 0.0], Y, 0.0, 2.0), _edge([1.0, -1.0, 0.0], Y, 0.0, 2.0, 1),
                 _edge([3.0, -1.0, 0.0], Y, 0.0, 2.0), _edge([1.0, -1.0, 0.0], [0.0, 1.0, 1.0], 0.0, 2.0),
                 _edge([1.0, -1.0, 0.0], Y, 0.0, 2.0, 0, 1)):
        _sob(requests, small, tangent_cyl, edge=edge)
        _sob(requests, small, tangent_cyl, recheck=1, edge=edge)
    tilted_cyl = [1, 0.0, 0.0, 0.0, 0.0, 0.3, 1.0, *X, 1.0, 0.0]
    _sob(requests, small, tilted_cyl, edge=_edge([1.0, -1.0, 0.0], Y, 0.0, 2.0))
    # Disjoint: a sphere far away, rejected on every arc.
    _sob(requests, small, [2, 5.0, 5.0, 5.0, *Z, *X, 1.0, 0.0])
    _near_family(requests, small)
    _seen_family(requests, small)


def _near_family(requests: list[str], small) -> None:
    """Tangent and almost tangent quadrics, larger tolerances and the regular recheck."""
    for radius in (1.0, 1.0 + 1.0e-9, 1.0 - 1.0e-12, 1.0 + 1.0e-6):
        sph = [2, 0.0, 0.0, 0.0, *Z, *X, radius, 0.0]
        _sob(requests, small, sph)
        _sob(requests, small, sph, tols=(1.0e-3, 1.0e-2))
        # The torus is not intersected exactly: math_FunctionAllRoots finds the zeros.
        tor = [4, 0.0, 0.0, 0.0, *Z, *X, radius - 0.25, 0.25]
        _sob(requests, small, tor)
        _sob(requests, small, tor, tols=(1.0e-3, 1.0e-2))
    for s in (SURFACES['bspline_s'], SURFACES['bezier_s'], SURFACES['sphere_part']):
        for q in (QUADRICS['q_torus'], QUADRICS['q_sphere'], QUADRICS['q_plane']):
            _sob(requests, s, q, tols=(1.0e-4, 1.0e-3))
            _sob(requests, s, q, tols=(1.0e-2, 1.0e-2))
    # A plane quadric at a small angle to two restrictions: one isolated zero, and the
    # whole regular edge within the recheck tolerance.
    for slope in (1.0e-4, 1.0e-3):
        tilt = [0, 0.0, 0.0, 0.0, 0.0, slope, 1.0, *X, 0.0, 0.0]
        for recheck in (0, 1):
            _sob(requests, small, tilt, recheck=recheck, edge=_edge(O, Y, -1.0, 1.0, 0, 1))


def _seen_family(requests: list[str], small) -> None:
    q = QUADRICS['q_sphere']
    points = [[0.1, 0.2, 0.3, 1.0e-6, 0.25, 0], [0.4, 0.5, 0.6, 1.0e-5, 0.75, 1], [0.7, 0.8, 0.9, 1.0e-4, -0.5, 1]]
    for segments, all_solution in (([], 1), ([[1, 2]], 1), ([[1, 0], [0, 3], [0, 0]], 0), ([[4, 1]], 1),
                                   ([[1, -1]], 0), ([[-4, 2]], 1)):
        _sob(requests, small, q, seen=_seen_table(points, segments, all_solution))
    _sob(requests, small, q, seen=_seen_table([], [], 0))
    _sob(requests, small, q, seen=_seen_table([], [[1, 1]], 1))
    _sob(requests, small, q, mode=1, seen=_seen_table(points[:1], [[1, 1]], 1))


def _inside(requests: list[str], surface, quadric, tol=1.0e-6, eps=1.0e-6, mode=0, start=()) -> None:
    requests.append(line_of('is_inside', [*surface, *quadric, tol, eps, mode, *start]))


def _inside_family(requests: list[str]) -> None:
    for s in SEARCH_SURFACES.values():
        for q in QUADRICS.values():
            _inside(requests, s, q)
    small = SEARCH_SURFACES['plane_small']
    for start in ((0.5, 0.5), (1.2, 0.0), (1.0 + 1.0e-9, -1.0), (0.0, 0.0), (-1.0, 1.0)):
        _inside(requests, small, QUADRICS['q_sphere'], mode=1, start=start)
        _inside(requests, SEARCH_SURFACES['cyl_band'], QUADRICS['q_plane_tilt'], mode=1, start=start)
    _inside(requests, small, QUADRICS['q_sphere'], mode=2)
    # Tangent: the sphere touches the plane at its pole; coincident: the plane itself.
    _inside(requests, small, [2, 0.0, 0.0, 1.0, *Z, *X, 1.0, 0.0])
    _inside(requests, small, [0, 0.0, 0.0, 0.0, *Z, *X, 0.0, 0.0])
    _inside(requests, small, QUADRICS['q_sphere'], tol=1.0e-2, eps=0.5)
    _inside(requests, small, [2, 5.0, 5.0, 5.0, *Z, *X, 1.0, 0.0])


def append_intstart_requests(requests: list[str]) -> None:
    """Append the IntStart request families in a fixed order."""
    _sob_family(requests)
    _inside_family(requests)
