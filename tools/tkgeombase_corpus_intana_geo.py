"""Deterministic requests for IntAna_QuadQuadGeo, IntAna_IntQuadQuad and IntAna_Curve
(tools/oracle/tkgeombase_probe_intana.cxx), appended by tkgeombase_corpus_intana.py.
"""
from __future__ import annotations
import math
from tkg3d_corpus import frame
from tkgeombase_corpus_gce import add
from tkgeombase_corpus_intana import Z, at, cone, cyl, pln, random_surface, sph, tor

XN = (1.0, 0.0, 0.0)
YN = (0.0, 1.0, 0.0)
ZN = (0.0, 0.0, 1.0)
XZ = at(0, 0, 0, YN, ZN)   # the plane y = 0
YZ = at(0, 0, 0, XN, YN)   # the plane x = 0


def tilted(angle, x=0.0, y=0.0, z=0.0):
    """A frame at (x, y, z) whose normal leans `angle` from Z towards X."""
    return at(x, y, z, (math.sin(angle), 0.0, math.cos(angle)), (math.cos(angle), 0.0, -math.sin(angle)))


PAIRS = [(0, 0), (0, 1), (0, 2), (0, 3), (1, 1), (1, 2), (1, 3), (2, 2), (2, 3), (3, 3),
         (0, 4), (1, 4), (3, 4), (2, 4), (4, 4)]


def tolerances(ka, kb):
    """The tolerances the overload of the pair (ka, kb) reads."""
    if (ka, kb) == (0, 0):
        return [1e-12, 1e-7]
    if (ka, kb) == (0, 1):
        return [1e-12, 1e-7, 0.0]
    if (ka, kb) == (0, 2):
        return []
    if (ka, kb) == (0, 3):
        return [1e-12, 1e-7]
    return [1e-7]


def qqgeo(requests, a, b, *tol):
    add(requests, 'ia_qqgeo', *a, *b, *(tol if tol else tolerances(a[0], b[0])))


def random_pairs(rng, requests):
    for _ in range(4):
        for ka, kb in PAIRS:
            qqgeo(requests, random_surface(rng, ka), random_surface(rng, kb))


def plane_requests(requests):
    for angle in [0.0, 1e-13, 1e-9, 1e-7, 2e-6, 1e-3, 0.5, math.pi / 2]:
        qqgeo(requests, pln(Z), pln(tilted(angle, 0.0, 0.0, 1.0)))
        qqgeo(requests, pln(Z), pln(tilted(angle)))
    qqgeo(requests, pln(Z), pln(at(0, 0, 1e-8)))
    qqgeo(requests, pln(Z), pln(at(0, 0, 0, (0, 0, -1), XN)))
    for d in [0.0, 1.0, 1.0 + 1e-8, 0.5, 2.0]:
        qqgeo(requests, pln(at(0, d, 0, YN, ZN)), cyl(Z, 1.0))
        qqgeo(requests, pln(at(0, 0, d)), cyl(Z, 1.0))
        qqgeo(requests, pln(at(0, 0, d)), sph(Z, 1.0))
        qqgeo(requests, pln(at(0, 0, d, (0, 0, -1), XN)), sph(Z, 1.0))
    for angle in [1e-9, 1e-6, 0.3, math.pi / 2 - 1e-6, math.pi / 2 - 1e-9]:
        qqgeo(requests, pln(tilted(angle, 0.0, 0.0, 0.3)), cyl(Z, 1.0))
        qqgeo(requests, pln(tilted(angle, 0.0, 0.0, 0.3)), cyl(Z, 1.0), 1e-12, 1e-7, 1.0)
    for d in [0.0, 1e-8, 2.0, -2.0]:
        for angle in [0.0, 0.2, 0.5, math.pi / 2 - 0.5, 1.2, math.pi / 2, 1.0]:
            qqgeo(requests, pln(tilted(angle, 0.0, 0.0, d)), cone(Z, 0.5, 1.0))
        qqgeo(requests, pln(at(0, 0, d, (0, 0, -1), XN)), cone(Z, 0.5, 1.0))
    # Sections beyond the ellipse (1e9) and hyperbola (2e6) limits are not done.
    qqgeo(requests, pln(tilted(math.pi / 2 - 0.5, 0.0, 0.0, 1e9)), cone(Z, 0.5, 1.0))
    qqgeo(requests, pln(tilted(0.2, 0.0, 0.0, 1e10)), cone(Z, 0.5, 1.0))
    qqgeo(requests, pln(tilted(1.2, 0.0, 0.0, 1e7)), cone(Z, 0.5, 1.0))
    qqgeo(requests, pln(tilted(math.pi / 2, 0.0, 0.0, 1e7)), cone(Z, 0.5, 1.0))


def cylinder_requests(requests):
    for d in [0.0, 1e-8, 0.5, 1.8, 2.0 - 1e-9, 2.0, 2.0 + 1e-8, 3.0]:
        qqgeo(requests, cyl(Z, 1.0), cyl(at(d, 0, 0), 1.0), 1e-4)
        qqgeo(requests, cyl(Z, 1.0), cyl(at(d, 0, 0), 1.0), 1e-7)
        qqgeo(requests, cyl(Z, 2.0), cyl(at(d, 0, 0), 1.0), 1e-7)
    qqgeo(requests, cyl(Z, 1.0), cyl(at(0.3, 0.2, 5.0, (0, 0, -1), XN), 1.0), 1e-7)
    for angle in [math.pi / 2, 0.7, 1e-6]:
        qqgeo(requests, cyl(Z, 1.0), cyl(tilted(angle), 1.0), 1e-7)
        qqgeo(requests, cyl(Z, 1.0), cyl(tilted(angle), 1.0 + 1e-14), 1e-7)
    qqgeo(requests, cyl(Z, 3.0), cyl(at(0.0, 5.0, 0.0, XN, YN), 2.0), 1e-7)
    qqgeo(requests, cyl(Z, 3.0), cyl(at(0.0, 6.0, 0.0, XN, YN), 2.0), 1e-7)
    for r in [0.5, 1.0, 2.0]:
        qqgeo(requests, cyl(Z, 1.0), sph(Z, r), 1e-7)
        qqgeo(requests, cyl(Z, 1.0), sph(at(0, 0, 3), r), 1e-7)
        qqgeo(requests, cyl(Z, 1.0), sph(at(0.5, 0, 0), r), 1e-7)
        qqgeo(requests, cyl(Z, r), cone(Z, 0.5, 0.0), 1e-7)
        qqgeo(requests, cyl(Z, r), cone(at(0, 0, 0, (0, 0, -1), XN), 0.5, 1.0), 1e-7)
        qqgeo(requests, cyl(Z, r), cone(at(1, 0, 0), 0.5, 1.0), 1e-7)


def sphere_requests(requests):
    for d, r2 in [(0.0, 2.0), (0.0, 1.0), (1e-8, 2.0), (3.0, 2.0), (1.0, 2.0), (1.0 + 1e-9, 2.0), (2.0, 2.0),
                  (4.0, 2.0), (5.0, 2.0), (0.5, 2.0), (1e-3, 1.0)]:
        qqgeo(requests, sph(Z, 1.0 if d < 1e-7 else 2.0), sph(at(d, 0, 0), r2), 1e-7)
        qqgeo(requests, sph(at(d, 0, 0), r2), sph(Z, 1.0), 1e-7)
    qqgeo(requests, sph(Z, 1.0), sph(at(2.0, 0, 0), 1.0), 1e-7)
    qqgeo(requests, sph(Z, 1.0), sph(at(2.0 - 1e-10, 0, 0), 1.0), 1e-7)
    for z, r in [(0.0, 1.0), (2.0, 1.0), (2.0, 3.0), (0.0, 0.0), (-2.0, 1.0), (1e-12, 1.0), (2.0, 2.0 * math.sin(0.5))]:
        qqgeo(requests, sph(at(0, 0, z), max(r, 1e-3)), cone(Z, 0.5, 0.0), 1e-7)
        qqgeo(requests, sph(at(0, 0, z), max(r, 1e-3)), cone(Z, 0.5, 1.0), 1e-7)
    qqgeo(requests, sph(at(1, 0, 0), 1.0), cone(Z, 0.5, 1.0), 1e-7)
    # A sphere through the apex: a point and a circle.
    qqgeo(requests, sph(at(0, 0, 2), 2.0), cone(Z, 0.5, 0.0), 1e-7)
    qqgeo(requests, sph(at(0, 0, -2), 2.0), cone(Z, 0.5, 0.0), 1e-7)


def cone_requests(requests):
    down = (0.0, 0.0, -1.0)
    cases = [
        (cone(Z, 0.5, 1.0), cone(Z, 0.5, 1.0)), (cone(Z, 0.5, 1.0), cone(Z, 0.3, 1.0)),
        (cone(Z, 0.5, 0.0), cone(Z, 0.3, 0.0)), (cone(Z, 0.5, 1.0), cone(at(0, 0, 3), 0.5, 1.0)),
        (cone(Z, 0.5, 1.0), cone(at(0, 0, 3, down, XN), 0.5, 1.0)), (cone(Z, 0.5, 1.0), cone(at(0, 0, 3, down, XN), 0.3, 1.0)),
        (cone(Z, 0.5, 0.0), cone(at(3, 0, 0), 0.5, 0.0)), (cone(Z, 0.5, 0.0), cone(at(300, 0, 0), 0.5, 0.0)),
        (cone(Z, 0.5, 0.0), cone(at(3, 0, 1), 0.5, 0.0)), (cone(Z, 0.5, 0.0), cone(at(0, 0, 0, XN, YN), 0.5, 0.0)),
        (cone(Z, 0.5, 0.0), cone(at(0, 0, 0, XN, YN), 0.3, 0.0)), (cone(Z, 0.5, 0.0), cone(tilted(1.0), 0.5, 0.0)),
        (cone(Z, 0.5, 0.0), cone(tilted(0.2), 0.1, 0.0)), (cone(Z, 0.5, 0.0), cone(tilted(0.8), 0.3, 0.0)),
        (cone(Z, 0.5, 0.0), cone(tilted(0.5, 0.0, 0.0, 1e-9), 0.5, 0.0)),
        (cone(Z, 0.5, 0.0), cone(at(0.0, 0.0, 2.0 / math.cos(0.5) * math.cos(0.5), (math.sin(1.0), 0.0, -math.cos(1.0)), (math.cos(1.0), 0.0, math.sin(1.0))), 0.5, 0.0)),
        (cone(Z, 0.5, 0.0), cone(at(1.0, 0.0, 0.0), 0.5, 0.0)), (cone(Z, 0.5, 0.0), cone(at(1.0, 1.0, 5.0, XN, YN), 0.5, 0.0)),
    ]
    for a, b in cases:
        qqgeo(requests, a, b, 1e-7)
    # Common generatrix: the second apex on the first cone, the axes in one plane.
    t = math.tan(0.5)
    for h in [1.0, 2.0]:
        apex = [h * t, 0.0, h]
        for lean in [0.3, 0.7, 1.0]:
            n = (math.sin(lean), 0.0, math.cos(lean))
            qqgeo(requests, cone(Z, 0.5, 0.0), cone(at(*apex, n, (n[2], 0.0, -n[0])), 0.5, 0.0), 1e-7)


def torus_requests(requests):
    for d in [0.0, 0.5, 1.0, 1.0 + 1e-14, 1.5, 1e-15]:
        qqgeo(requests, pln(at(0, 0, d)), tor(Z, 4.0, 1.0))
    for n in [YN, XN, (0.6, 0.8, 0.0), (0.6, 0.0, 0.8)]:
        qqgeo(requests, pln(at(0, 0, 0, n, ZN if n[2] == 0.0 else YN)), tor(Z, 4.0, 1.0))
    qqgeo(requests, pln(at(0, 1e-9, 0, YN, ZN)), tor(Z, 4.0, 1.0))
    qqgeo(requests, pln(Z), tor(Z, 1.0, 1.0))
    for r in [2.0, 3.0, 3.5, 4.0, 5.0, 5.5, 3.0 - 1e-8]:
        qqgeo(requests, cyl(Z, r), tor(Z, 4.0, 1.0))
        qqgeo(requests, sph(Z, r), tor(Z, 4.0, 1.0))
        qqgeo(requests, sph(at(0, 0, 1.0), r), tor(Z, 4.0, 1.0))
    qqgeo(requests, cyl(at(1, 0, 0), 3.0), tor(Z, 4.0, 1.0))
    qqgeo(requests, cyl(tilted(0.1), 3.0), tor(Z, 4.0, 1.0))
    qqgeo(requests, sph(at(1, 0, 0), 3.0), tor(Z, 4.0, 1.0))
    for angle, z in [(0.5, 0.0), (0.5, -5.0), (0.2, -8.0), (1.0, -4.0), (0.3, 20.0), (0.7, -5.5)]:
        qqgeo(requests, cone(at(0, 0, z), angle, 0.0), tor(Z, 4.0, 1.0))
    qqgeo(requests, cone(at(1, 0, 0), 0.5, 0.0), tor(Z, 4.0, 1.0))
    for d, r1, r2, big2 in [(0.0, 1.0, 1.0, 4.0), (0.0, 1.0, 0.5, 4.0), (0.5, 1.0, 1.0, 4.0), (0.0, 1.0, 1.0, 5.5),
                            (0.0, 1.0, 1.0, 6.0), (0.0, 1.0, 1.0, 7.0), (0.0, 1.0, 0.5, 4.2), (0.0, 1.0, 1.0, 5.0)]:
        qqgeo(requests, tor(Z, 4.0, r1), tor(at(0, 0, d), big2, r2))
    qqgeo(requests, tor(Z, 4.0, 1.0), tor(at(1, 0, 0), 4.0, 1.0))
    qqgeo(requests, tor(Z, 1.0, 2.0), tor(Z, 4.0, 1.0))


def qq_requests(rng, requests):
    quadrics = [sph(Z, 5.0), sph(at(3, 0, 0), 2.0), sph(at(2, 0, 0), 1.0), sph(at(1, 0, 0), 3.0), sph(at(0, 0, 0), 3.0),
                sph(at(4, 0, 0), 1.0), sph(at(1.5, 0.5, 0), 2.0),
                cyl(at(0, 0, 0, XN, YN), 1.0), cyl(at(0, 0, 0, XN, YN), 2.0), cyl(at(0.5, 0, 0, XN, YN), 1.0),
                cyl(tilted(0.7, 0.2), 1.3), cone(at(0, 0, 0, XN, YN), 0.4, 0.5), cone(tilted(0.4, 0.5), 0.3, 1.0),
                pln(tilted(0.7)), pln(Z)]
    for quad in quadrics:
        add(requests, 'ia_qq', 0, *Z, 2.0, *quad, 1e-7)
        add(requests, 'ia_qq', 1, *Z, 0.4, 1.0, *quad, 1e-7)
    add(requests, 'ia_qq', 1, *Z, 0.4, 0.0, *sph(at(0, 0, 2), 1.0), 1e-7)
    add(requests, 'ia_qq', 1, *Z, 0.4, 0.0, *cone(at(0, 0, 3, (0, 0, -1), XN), 0.4, 0.0), 1e-7)
    add(requests, 'ia_qq', 1, *Z, 0.5, 0.0, *pln(at(0, 0, 0, (math.cos(0.5), 0.0, -math.sin(0.5)), (math.sin(0.5), 0.0, math.cos(0.5)))), 1e-7)
    add(requests, 'ia_qq', 2)
    for _ in range(6):
        for kind in (1, 2, 3):
            add(requests, 'ia_qq', 0, *frame(rng), 1.5, *random_surface(rng, kind), 1e-7)
            add(requests, 'ia_qq', 1, *frame(rng), 0.5, 1.0, *random_surface(rng, kind), 1e-7)


def curve_requests(rng, requests):
    q_sphere = [1.0, 1.0, 1.0, 0.0, 0.0, 0.0, -1.5, 0.0, 0.0, -1.75]
    for two in (0, 1):
        for pos in (0, 1):
            add(requests, 'ia_curve', 0, *Z, 1.0, *q_sphere, 1e-8, 0.3, 2.8, two, pos, 0.5, 2.5, 1, 0)
            add(requests, 'ia_curve', 1, *Z, 0.5, 1.0, *q_sphere, 1e-8, 0.0, 2 * math.pi, two, pos, 1.0, 0.5, 0, 1)
    for _ in range(4):
        q = [rng.uniform(-2, 2) for _ in range(10)]
        add(requests, 'ia_curve', 0, *frame(rng), 1.2, *q, 1e-8, 0.0, 1.0, 1, 1, 0.2, 1.7, 0, 0)


def append_intana_geo_requests(rng, requests: list[str]) -> None:
    """Append the geometric, analytic and curve request families."""
    random_pairs(rng, requests)
    plane_requests(requests)
    cylinder_requests(requests)
    sphere_requests(requests)
    cone_requests(requests)
    torus_requests(requests)
    qq_requests(rng, requests)
    curve_requests(rng, requests)
