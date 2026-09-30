"""Deterministic requests for the IntPolyh package (tools/oracle/tkgeomalgo_probe_intpolyh.cxx),
over the surface catalog of tkgeomalgo_corpus_intcurvesurface.py.

`ph_inter` intersects pairs of planes, quadrics, tori, Bezier, B-spline, swept, offset
and trimmed surfaces with the default sampling, given sample counts and given parameters,
plus tangent, coincident, parallel and disjoint pairs. `ph_maillage` drives
IntPolyh_MaillageAffinage step by step (standard, shifted and parameter fills, the
enlarged zone, each refinement, the comparison and the chaining) and prints its whole
state. `ph_tricontact` and `ph_edgecontact` test triangle pairs and triangle-edge pairs
(coplanar, crossing, touching, parallel, degenerate). `ph_triangle` refines grid
triangles once and repeatedly. The record operations cover every member of the points,
start points, section lines, arrays, couples and edges, with out-of-range indices where
the native code checks them.
"""
from __future__ import annotations
import random
from tkgeomalgo_corpus_intcurvesurface import (HALF_PI, O, SURFACES, TWO_PI, X, Y, Z, bez_s, bsp_s, cone, cyl,
                                               line_of, offs, plane, rtrim, sad, sphere, torus)

GRID = [[[-1.5 + i, -1.5 + j, 0.3 * ((i * j) % 3) - 0.3] for j in range(4)] for i in range(4)]
INTER_SURFACES = {
    'plane': SURFACES['plane'],
    'plane_tilt': sad(plane([0.1, 0.0, 0.0], [1.0, 0.2, 0.4]), (-2.0, 2.0, -2.0, 2.0)),
    'cyl': SURFACES['cyl'],
    'cyl_x': sad(cyl([0.0, 0.3, 0.2], X, 0.9), (0.0, TWO_PI, -2.0, 2.0)),
    'cone': SURFACES['cone'],
    'sphere': SURFACES['sphere'],
    'sphere_part': SURFACES['sphere_part'],
    'torus': sad(torus(O, Z, 1.5, 0.5)),
    'bezier_s': sad(bez_s(GRID)),
    'bspline_s': SURFACES['bspline_s'],
    'rev_bezier': SURFACES['rev_bezier'],
    'off_sphere': SURFACES['off_sphere'],
    'off_bspline': SURFACES['off_bspline'],
    'trim_cyl': SURFACES['trim_cyl'],
    'trim_plane': SURFACES['trim_plane'],
}


def _inter(requests: list[str], s1, s2, mode=0, extra=()) -> None:
    requests.append(line_of('ph_inter', [mode, *s1, *s2, *extra]))


def _params(values) -> list[float]:
    return [len(values), *values]


def _inter_family(requests: list[str]) -> None:
    names = list(INTER_SURFACES)
    for i, a in enumerate(names):
        for b in names[i:]:
            _inter(requests, INTER_SURFACES[a], INTER_SURFACES[b])
    pairs = [('plane', 'sphere'), ('cyl', 'sphere'), ('bezier_s', 'plane_tilt'), ('torus', 'cyl_x'), ('bspline_s', 'sphere')]
    for a, b in pairs:
        s1, s2 = INTER_SURFACES[a], INTER_SURFACES[b]
        for counts in ([5, 5, 5, 5], [20, 3, 3, 20], [2, 2, 12, 7], [30, 30, 10, 10]):
            _inter(requests, s1, s2, 1, counts)
        u = [0.0, 0.1, 0.35, 0.7, 1.0]
        _inter(requests, s1, s2, 2, _params([-2.0, -0.5, 0.2, 2.0]) + _params([-2.0, 0.0, 2.0]) +
               _params([0.0, 1.0, 2.5, 4.0, TWO_PI]) + _params([-HALF_PI, 0.0, 0.7, HALF_PI]))
        _inter(requests, s1, s2, 2, _params(u) + _params(u[:2]) + _params(u) + _params([0.0]))
    special = [
        (sad(plane([0.0, 0.0, 1.5], Z), (-2.0, 2.0, -2.0, 2.0)), INTER_SURFACES['sphere']),       # tangent
        (sad(plane(O, Z), (-1.0, 1.0, -1.0, 1.0)), sad(plane(O, [0.0, 0.01, 1.0]), (-1.0, 1.0, -1.0, 1.0))),  # grazing
        (INTER_SURFACES['plane'], INTER_SURFACES['plane']),                                        # coincident
        (INTER_SURFACES['plane'], sad(plane([0.0, 0.0, 0.3 + 1e-9], Z), (-2.0, 2.0, -2.0, 2.0))),  # parallel
        (INTER_SURFACES['sphere'], sad(sphere([10.0, 0.0, 0.0], Z, 1.0))),                         # disjoint
        (sad(cyl(O, Z, 1.0), (0.0, TWO_PI, -1.0, 1.0)), sad(cyl(O, Z, 1.0), (0.0, TWO_PI, -1.0, 1.0))),
        (sad(cyl(O, Z, 1.0), (0.0, TWO_PI, -1.0, 1.0)), sad(cyl([2.0, 0.0, 0.0], Z, 1.0), (0.0, TWO_PI, -1.0, 1.0))),
        (sad(cone(O, Z, 0.5, 1.0), (0.0, TWO_PI, -1.0, 1.0)), sad(plane(O, Z), (-3.0, 3.0, -3.0, 3.0))),
        (sad(bez_s(GRID)), sad(offs(0.0001, bez_s(GRID)))),
        (sad(rtrim(0.0, 1.0, 0.0, 1.0, plane(O, Z))), sad(plane([0.5, 0.5, 0.0], X), (-5.0, 5.0, -5.0, 5.0))),
        (sad(plane(O, Z), (-1e3, 1e3, -1e3, 1e3)), sad(sphere(O, Z, 0.01))),                    # small in large
    ]
    for s1, s2 in special:
        _inter(requests, s1, s2)
        _inter(requests, s2, s1)


def _maillage(requests: list[str], s1, c1, s2, c2, steps) -> None:
    flat_steps = [x for step in steps for x in step]
    requests.append(line_of('ph_maillage', [*s1, *c1, *s2, *c2, len(steps), *flat_steps]))


def _linspace(a: float, b: float, n: int) -> list[float]:
    return [a + (b - a) * i / (n - 1) for i in range(n)] if n > 1 else [a]


def _maillage_family(requests: list[str]) -> None:
    std = [[0, 1], [0, 2], [15], [12]]
    shifted = [[14, 1, 0], [15], [12]]
    steps_list = [
        std, shifted,
        [[0, 1], [0, 2], [4], [5, 1], [5, 2], [6, 1], [6, 2], [8], [10, 1], [10, 2], [11], [12]],
        [[0, 1], [0, 2], [4], [5, 1], [5, 2], [6, 1], [6, 2], [9, 1], [9, 2], [9, 3], [11], [12]],
        [[13, 1], [0, 1], [0, 2], [15], [13, 0], [12]],
        [[0, 1], [0, 2], [4], [5, 1], [5, 2], [6, 1], [6, 2], [7], [7], [11], [11], [12], [12]],
        [[0, 1], [1, 2, 0], [15], [12]],
        [[4]],
        [[0, 1], [4]],
    ]
    pairs = [(INTER_SURFACES['plane'], [5, 5], INTER_SURFACES['sphere'], [6, 4]),
             (INTER_SURFACES['cyl'], [6, 6], INTER_SURFACES['cyl_x'], [5, 5]),
             (INTER_SURFACES['bezier_s'], [4, 7], INTER_SURFACES['plane_tilt'], [5, 5]),
             (INTER_SURFACES['torus'], [8, 5], INTER_SURFACES['plane'], [4, 4])]
    for s1, c1, s2, c2 in pairs:
        u1, v1 = _params(_linspace(-1.0, 1.0, c1[0])), _params(_linspace(-0.5, 1.0, c1[1]))
        u2, v2 = _params(_linspace(0.2, 1.5, c2[0])), _params(_linspace(-1.0, 0.8, c2[1]))
        params = [[2, 1, *u1, *v1, 0, 0.0], [2, 2, *u2, *v2, 1, 0.05], [15], [12]]
        shift_params = [[3, 2, 1, *u1, *v1, 0, 0.0], [3, 1, 0, *u1, *v1, 1, 0.2], [0, 2], [15], [12]]
        # Too few parameters: the fill raises Standard_OutOfRange part way.
        short = [[2, 1, *_params([-1.0, 1.0]), *v1, 0, 0.0], [0, 2], [15], [12]]
        for steps in steps_list + [params, shift_params, short]:
            _maillage(requests, s1, c1, s2, c2, steps)
    # The enlarged zone of B-spline surfaces that do not meet.
    near = sad(bsp_s(2, 2, GRID, [0.0, 1.0, 2.0], [3, 1, 3], [0.0, 0.5, 1.0], [3, 1, 3]))
    _maillage(requests, near, [5, 5], sad(plane([0.0, 0.0, 2.0], Z), (-1.0, 1.0, -1.0, 1.0)), [4, 4], std)


def _tricontact_family(requests: list[str]) -> None:
    rng = random.Random(4417)
    base = [[0, 0, 0], [1, 0, 0], [0, 1, 0]]
    cases = [
        [[0.2, 0.2, -1], [0.2, 0.2, 1], [0.3, 0.3, 1]],        # crossing
        [[2, 2, 2], [3, 2, 2], [2, 3, 2]],                     # apart
        [[0.1, 0.1, 0], [0.5, 0.1, 0], [0.1, 0.5, 0]],         # coplanar inside
        [[0.5, 0.5, 0], [2, 0.5, 0], [0.5, 2, 0]],             # coplanar overlapping
        [[1, 0, 0], [2, 0, 1], [2, 1, 0]],                     # touching at a vertex
        [[0, 0, 0], [0, 0, 0], [0, 0, 0]],                     # null triangle
        [[0.3, 0.3, 0.01], [1, 1, 0.01], [0.3, 1, 0.01]],      # parallel above
        [[0.2, -1, 0], [0.2, 2, 0], [0.2, 0.5, 1]],            # through an edge
    ]
    for c in cases:
        for angle in (-2.0, 0.25):
            requests.append(line_of('ph_tricontact', [angle, *_pts(base), *_pts(c)]))
    for _ in range(30):
        t1 = [[round(rng.uniform(-1, 1), 3) for _ in range(3)] for _ in range(3)]
        t2 = [[round(rng.uniform(-1, 1), 3) for _ in range(3)] for _ in range(3)]
        requests.append(line_of('ph_tricontact', [-2.0, *_pts(t1), *_pts(t2)]))


def _pts(points) -> list[float]:
    return [c for i, p in enumerate(points) for c in (*p, 0.1 * i, 0.2 * i)]


SP_DEFAULT = [0, 0, 0, 0, 0, 0, 0, -1, -2, -1.0, -1, -2, -1.0, -1]


def _edgecontact(requests: list[str], sid, ei, tri_pts, edge_pts, tris=None) -> None:
    tris = tris or [[3, 4, 5, 1, -1, 1], [6, 7, 8, -1, 1, 1]]
    requests.append(line_of('ph_edgecontact', [sid, ei, *tris[0], *tris[1], *_pts(tri_pts + edge_pts),
                                               *SP_DEFAULT, *SP_DEFAULT]))


def _edgecontact_family(requests: list[str]) -> None:
    tri = [[0, 0, 0], [2, 0, 0], [0, 2, 0]]
    edges = [
        [[0.5, 0.5, -1], [0.5, 0.5, 1]],       # crossing inside
        [[0, 0, -1], [0, 0, 1]],               # through the first vertex
        [[2, 0, -1], [2, 0, 1]],               # through the second vertex
        [[1, 0, -1], [1, 0, 1]],               # through the first side
        [[1, 1, -1], [1, 1, 1]],               # through the second side
        [[0, 1, -1], [0, 1, 1]],               # through the third side
        [[0.5, 0.5, 0], [0.5, 0.5, 1]],        # ending on the triangle
        [[3, 3, -1], [3, 3, 1]],               # outside
        [[0.5, 0.5, 1], [0.6, 0.6, 2]],        # above
        [[-1, 0.5, 0], [3, 0.5, 0]],           # coplanar across two sides
        [[-1, 0, 0], [3, 0, 0]],               # coplanar along the first side
        [[0.5, 0, 0], [1.5, 0, 0]],            # coplanar inside the first side
        [[-1, -1, 0], [1, 1, 0]],              # coplanar through a vertex
        [[0.5, 0.2, 0], [0.7, 0.3, 0]],        # coplanar inside
        [[3, 3, 0], [4, 4, 0]],                # coplanar outside
        [[0.5, 0.5, 0], [0.5, 0.5, 0]],        # null edge
    ]
    tilted = [[0, 0, 0], [2, 0, 1], [0, 2, 1]]
    for sid in (1, 2):
        for ei in (1, 2, 3):
            for e in edges:
                _edgecontact(requests, sid, ei, tri, e)
        _edgecontact(requests, sid, 2, tilted, [[0.5, 0.5, -1], [0.5, 0.5, 2]])
        _edgecontact(requests, sid, 1, [[0, 0, 0], [0, 2, 0], [0, 0, 2]], [[-1, 0.5, 0.5], [1, 0.5, 0.5]])
        _edgecontact(requests, sid, 1, [[0, 0, 0], [0, 0, 2], [2, 0, 0]], [[0.5, -1, 0.5], [0.5, 1, 0.5]])
        _edgecontact(requests, sid, 3, tri, [[0.5, 0.5, -1], [0.5, 0.5, 1]], [[3, 4, 5, -1, 1, -1], [6, 7, 8, 1, -1, -1]])
        _edgecontact(requests, sid, 0, tri, [[0.5, 0.5, -1], [0.5, 0.5, 1]])
        _edgecontact(requests, 3, 1, tri, [[0.5, 0.5, -1], [0.5, 0.5, 1]])
        # Sliver triangles in the plane x = 0 whose PT2PT3 side has almost no Y (then no
        # Z): the last two coordinate combinations of the crossing point.
        for pt3 in ([0.0, 1e-13, 1.0], [0.0, 1e-13, 2.0]):
            _edgecontact(requests, sid, 1, [[0, 0, 0], [0, 0, 1], pt3], [[-1, 0, 0.5], [1, 0, 0.5]])


def _triangle_family(requests: list[str]) -> None:
    box_in = [-3, -3, -3, 3, 3, 3]
    box_small = [0.0, 0.0, -0.1, 0.2, 0.2, 0.1]
    surfaces = [INTER_SURFACES['plane'], INTER_SURFACES['sphere'], INTER_SURFACES['cyl'], INTER_SURFACES['bezier_s'],
                INTER_SURFACES['torus']]
    for s in surfaces:
        for nsu, nsv, num in ((3, 3, 0), (4, 5, 7), (5, 4, 12), (3, 6, 9)):
            requests.append(line_of('ph_triangle', [*s, nsu, nsv, num, 0, 0.1, *box_in]))
            requests.append(line_of('ph_triangle', [*s, nsu, nsv, num, 1, 0.05, *box_in]))
            requests.append(line_of('ph_triangle', [*s, nsu, nsv, num, 1, 0.01, *box_small]))


def _sp(values) -> list[float]:
    return list(values)


def _startpoint_family(requests: list[str]) -> None:
    a = [1, 2, 3, 0.1, 0.2, 0.3, 0.4, 5, 1, 0.25, 6, 2, 0.5, 3]
    variants = [
        ([0.1, 0.2, 0.3, 0.4], (1, 0.25), (2, 0.5)), ([0.1, 0.2, 0.9, 0.9], (1, 0.25 + 1e-13), (7, 0.9)),
        ([0.9, 0.9, 0.3, 0.4], (4, 0.3), (2, 0.5)), ([0.1, 0.2, 0.3, 0.4], (-1, -1.0), (2, 0.5)),
        ([0.1, 0.2 + 1e-9, 0.3, 0.4], (-1, -1.0), (-1, -1.0)), ([0.5, 0.5, 0.5, 0.5], (-2, -1.0), (3, 0.5)),
    ]
    for uv, (e1, l1), (e2, l2) in variants:
        b = [4, 5, 6, *uv, 5, e1, l1, 6, e2, l2, 3]
        for ea in (1, 2, 3, -1, 5):
            a2 = list(a)
            a2[8] = ea
            requests.append(line_of('ph_startpoint', [*a2, *b, 10, 11, 12]))
        a3 = list(a)
        a3[8], a3[11] = -1, 2
        requests.append(line_of('ph_startpoint', [*a3, *b, 10, 11, 12]))
        a4 = list(a)
        a4[8], a4[11] = -1, 3
        requests.append(line_of('ph_startpoint', [*a4, *b, 10, 11, 12]))


def _sectionline_family(requests: list[str]) -> None:
    sps = [[i, i + 1, i + 2, 0.1 * i, 0.2 * i, 0.3 * i, 0.4 * i, i, 1, 0.5, i + 1, 2, 0.25, 0] for i in range(3)]
    flat_sps = [x for s in sps for x in s]
    ops = [
        [[0, 10], [3, 0, 0], [1], [3, 1, 1], [1], [2, 2], [5, 0], [5, 2], [5, 3], [5, 4], [5, -1], [6], [4], [6]],
        [[5, 0], [3, 0, 0], [1], [1], [3, 5, 1], [0, 3], [6]],
        [[7, 5], [3, 0, 2], [1], [7, 2], [6], [2, 0], [2, 1], [6]],
    ]
    for seq in ops:
        body = [len(sps), *flat_sps, len(seq), *[x for op in seq for x in op]]
        requests.append(line_of('ph_sectionline', body))


def _array_family(requests: list[str]) -> None:
    seqs = [
        (3, 2, [[1], [1], [1], [3, 1, 1, 2, 3, 4, 5], [4], [1], [1], [5]]),
        (0, 256, [[0, 4], [3, 2, 9, 8, 7, 6, 5], [2, 3], [5], [0, 2], [1], [4], [5]]),
        (5, 1, [[2, 5], [1], [1], [0, 3], [2, 2], [5]]),
        (2, 3, [[3, 0, 1, 1, 1, 1, 1], [3, 1, -1, 2, 3, 4, 5], [2, 2], [4], [1], [1], [5]]),
    ]
    for n, inc, ops in seqs:
        requests.append(line_of('ph_array', [n, inc, len(ops), *[x for op in ops for x in op]]))


def _records_family(requests: list[str]) -> None:
    rng = random.Random(7710)
    surfaces = [INTER_SURFACES['sphere'], INTER_SURFACES['bezier_s'], sad(offs(0.2, torus(O, Z, 2.0, 0.5)))]
    for k in range(12):
        p1 = [round(rng.uniform(-5, 5), 4) for _ in range(5)]
        p2 = [round(rng.uniform(-5, 5), 4) for _ in range(5)]
        rr = [2.5, 0.0, 1e-20, -3.0, 1e-300, 7e22][k % 6]
        requests.append(line_of('ph_point', [*p1, *p2, rr, *surfaces[k % 3], k * 37 - 50]))
    for a, b, angle, i in ((3, 5, 0.25, 4), (7, 7, -2.0, 0), (-1, 2, 1e-7, 123), (1000, 3, -0.996, -7)):
        requests.append(line_of('ph_couple', [a, b, angle, i]))
    tool_surfaces = list(INTER_SURFACES.values()) + [SURFACES['plane_nat'], SURFACES['cyl_nat'], SURFACES['off_plane']]
    for k, s in enumerate(tool_surfaces):
        counts = [(10, 10), (2, 2), (5, 7), (1, 3), (0, 4)][k % 5]
        requests.append(line_of('ph_tools', [*s, *counts]))


def append_intpolyh_requests(requests: list[str]) -> None:
    """Append the IntPolyh request families in a fixed order."""
    _inter_family(requests)
    _maillage_family(requests)
    _tricontact_family(requests)
    _edgecontact_family(requests)
    _triangle_family(requests)
    _startpoint_family(requests)
    _sectionline_family(requests)
    _array_family(requests)
    _records_family(requests)
