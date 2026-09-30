"""Deterministic requests for the GeomPlate package (tools/oracle/tkgeomalgo_probe_geomplate*.cxx).

`gplate_point` and `gplate_curve` query every member of the point and curve constraints
(3d curves, curves on surfaces, criterion laws, projected curves, setters, exceptions);
`gplate_surface` evaluates a GeomPlate_Surface on several initial surfaces and plates;
`gplate_average`, `gplate_normals` and `gplate_halfspace` drive GeomPlate_BuildAveragePlane;
`gplate_build` performs GeomPlate_BuildPlateSurface on curve and point constraints of every
order (G0/G1/G2, with and without initial surface, open and closed contours, the non-linear
first plate, the three constructors) and reads every result; `gplate_approx` runs
GeomPlate_MakeApprox with its own and explicit G0/G1 criteria.
"""
from __future__ import annotations
import math


def _line(op: str, values: list[float]) -> str:
    return op + ' ' + ' '.join(repr(float(v)) for v in values)


# Surface specs: kind then data (tkgeomalgo_probe_geomplate.cxx, ga_gp_surface).
NONE = [-1]


def plane(p=(0., 0., 0.), n=(0., 0., 1.)) -> list[float]:
    return [0, *p, *n, 0.]


def cylinder(r: float, p=(0., 0., 0.), n=(0., 0., 1.)) -> list[float]:
    return [1, *p, *n, r]


def sphere(r: float, p=(0., 0., 0.), n=(0., 0., 1.)) -> list[float]:
    return [2, *p, *n, r]


def bezier_surface(heights: list[float]) -> list[float]:
    poles = []
    for i in range(3):
        for j in range(3):
            poles += [float(i), float(j), heights[3 * i + j]]
    return [3, *poles]


# Curve constraint specs: kind order npts tolDist tolAng tolCurv crit laws geometry.
def curve(kind: int, geometry: list[float], order: int = 0, npts: int = 10, tols=(1e-4, 0.01, 0.1),
          laws: list[float] | None = None) -> list[float]:
    laws = laws or []
    crit = {0: 0, 2: 1, 6: 2}[len(laws)]
    return [kind, order, npts, *tols, crit, *laws, *geometry]


def bezier(*poles) -> list[float]:
    return [len(poles), *[c for p in poles for c in p]]


def seg(p, d, t0, t1) -> list[float]:
    return [*p, *d, t0, t1]


def arc(c, n, r, t0, t1) -> list[float]:
    return [*c, *n, r, t0, t1]


def on_line(surface, p2, d2, t0, t1) -> list[float]:
    return [*surface, *p2, *d2, t0, t1]


def on_bezier(surface, *poles) -> list[float]:
    return [*surface, len(poles), *[c for p in poles for c in p]]


def point(p, order: int = 0, tol: float = 1e-4, uv=None) -> list[float]:
    return [0, order, *p, tol, *([1, *uv] if uv else [0])]


def surface_point(u, v, surface, order: int, tols=(1e-4, 0.01, 0.1), uv=None) -> list[float]:
    return [1, order, u, v, *surface, *tols, *([1, *uv] if uv else [0])]


def builder(ctor: int, curves: list[list[float]], points: list[list[float]], init=NONE, degree=3, nbpts=10,
            nbiter=3, tols=(1e-5, 1e-4, 0.01), aniso=0, nbbounds=0) -> list[float]:
    fields = [ctor, degree, nbpts, nbiter, *tols, aniso, nbbounds, *init, len(curves)]
    for c in curves:
        fields += c
    fields.append(len(points))
    for p in points:
        fields += p
    return fields


SAMPLES = [5, 0.5, 0.5, 0.25, 0.75, 1.0, 1.0, 0.1, 0.9, 1.5, 0.3]


def _build(requests: list[str], spec: list[float], samples=SAMPLES, gaps: int = 1) -> None:
    requests.append(_line('gplate_build', spec + samples + [gaps]))


# The A1/A2 contour of GeomPlate_BuildPlateSurface_Test.cxx: four quadratic Bezier curves.
def a_contour(h: float = 1.0, order: int = 0, npts: int = 10) -> list[list[float]]:
    return [curve(0, bezier((0, 0, 0), (1, 0, h), (2, 0, 0)), order, npts),
            curve(0, bezier((0, 2, 0), (1, 2, h), (2, 2, 0)), order, npts),
            curve(0, bezier((0, 0, 0), (0, 1, 2 * h), (0, 2, 0)), order, npts),
            curve(0, bezier((2, 0, 0), (2, 1, 2 * h), (2, 2, 0)), order, npts)]


def square_on(surface, order: int, u1: float = 1.0, v1: float = 1.0, npts: int = 8, reverse_top=True) -> list[list[float]]:
    """The boundary of [0, u1] x [0, v1] as 2d lines on `surface`, chained (top reversed)."""
    top = on_line(surface, (u1, v1), (-1, 0), 0, u1) if reverse_top else on_line(surface, (0, v1), (1, 0), 0, u1)
    return [curve(3, on_line(surface, (0, 0), (1, 0), 0, u1), order, npts),
            curve(3, on_line(surface, (u1, 0), (0, 1), 0, v1), order, npts),
            curve(3, top, order, npts),
            curve(3, on_line(surface, (0, v1), (0, -1), 0, v1), order, npts)]


def _point_requests(requests: list[str]) -> None:
    for spec, setters in [
        (point((1, 2, 3)), [1, 1e-3, 0.02, 0.2, 0.25, 0.75]),
        (point((0, -1, 0.5), -1, 1e-5), [0, 1e-4, 0., 0., 1, 2]),
        (point((0, 0, 0), 1, 1e-3, uv=(0.5, 0.5)), [2, 1e-2, 0.1, 1.0, -1, -2]),
        (point((0, 0, 0), 2), [0, 0, 0, 0, 0, 0]),
        (point((0, 0, 0), -2), [0, 0, 0, 0, 0, 0]),
        (surface_point(0.3, 0.4, sphere(2.0), 2), [1, 1e-3, 0.05, 0.3, 0.1, 0.2]),
        (surface_point(0.5, 0.25, cylinder(1.5), 1, (1e-3, 0.02, 0.2), uv=(1, 2)), [0, 1e-3, 0.05, 0.3, 0.1, 0.2]),
        (surface_point(0.2, 0.7, bezier_surface([0, .5, 0, .2, 1, .2, 0, .5, 0]), 0), [2, 1e-5, 0.05, 0.3, 0.4, 0.2]),
        (surface_point(1.0, -2.0, plane((1, 1, 1), (0, 1, 1)), 3), [-1, 1e-3, 0.05, 0.3, 0.1, 0.2]),
    ]:
        requests.append(_line('gplate_point', spec + setters))


def _curve_requests(requests: list[str]) -> None:
    geo_line = seg((0, 0, 0), (1, 1, 0), 0, 2)
    cases = [
        (curve(1, geo_line), [0., 1., 2.], [0], [1, 12]),
        (curve(1, geo_line, 0, 5, laws=[1e-3, 2e-3]), [0., 0.5, 2.], [1, *plane(), 1e-7, 1e-7], [2, 3]),
        (curve(2, arc((0, 0, 1), (0, 0, 1), 2, 0, math.pi)), [0., 1., math.pi], [1, *plane(), 1e-7, 1e-7], [0, 4]),
        (curve(0, bezier((0, 0, 0), (1, 0, 1), (2, 0, 0)), -1, 7), [0., .5, 1.], [1, *plane((0, 0, -1)), 1e-6, 1e-6], [-1, 7]),
        (curve(3, on_line(plane(), (0, 0), (1, 0), 0, 1), 1), [0., .5, 1.], [0], [2, 9]),
        (curve(3, on_line(cylinder(1.5), (0, 0), (1, 1), 0, 1), 2, 6, laws=[1e-3, 1e-2, 0.01, 0.02, 0.1, 0.2]),
         [0., .3, 1.], [1, *cylinder(1.5), 1e-7, 1e-7], [1, 10]),
        (curve(4, on_bezier(sphere(3.0), (0, 0), (0.3, 0.5), (0.6, 0.1)), 2), [0., .5, 1.], [1, *sphere(3.0), 1e-7, 1e-7], [0, 3]),
        (curve(3, on_line(bezier_surface([0, .5, 0, .2, 1, .2, 0, .5, 0]), (0, 0.2), (1, 0), 0, 1), 1),
         [0., .5, 1.], [0], [1, 20]),
        (curve(1, geo_line, 3), [0., 1., 2.], [0], [1, 1]),
        (curve(1, geo_line, -2), [0., 1., 2.], [0], [1, 1]),
        (curve(1, geo_line, 1, laws=[1e-3, 1e-2, 0.01, 0.02, 0.1, 0.2]), [0., 1., 2.], [0], [1, 1]),
    ]
    for spec, us, projected, setters in cases:
        requests.append(_line('gplate_curve', spec + us + projected + setters))
    requests.append('gplate_curve_empty')


def _plate(ppcs: list[tuple], order: int) -> list[float]:
    fields = [len(ppcs)]
    for u, v, x, y, z, iu, iv in ppcs:
        fields += [u, v, x, y, z, iu, iv]
    return fields + [order]


BUMP = [(0, 0, 0, 0, 0, 0, 0), (1, 0, 0, 0, 0, 0, 0), (0, 1, 0, 0, 0, 0, 0), (1, 1, 0, 0, 0, 0, 0),
        (0.5, 0.5, 0, 0, 0.3, 0, 0), (0.25, 0.75, 0.05, 0, 0.1, 0, 0)]
SLOPE = BUMP + [(0.5, 0.5, 0.1, 0, 0, 1, 0), (0.5, 0.5, 0, 0.1, 0, 0, 1)]


def _surface_requests(requests: list[str]) -> None:
    uv = [4, 0.5, 0.5, 0.2, 0.8, 0.0, 1.0, 1.3, -0.2]
    for init, plate, bounds in [
        (plane(), _plate(BUMP, 3), [0, 1, 0, 1]),
        (plane((0, 0, 1), (0, 1, 1)), _plate(SLOPE, 4), [-1, 2, -1, 2]),
        (cylinder(2.0), _plate(BUMP, 2), [0, 1, 1, 0]),
        (sphere(1.5), _plate(SLOPE, 3), [0, 1, 0.5, 0.5]),
        (bezier_surface([0, .5, 0, .2, 1, .2, 0, .5, 0]), _plate(BUMP, 5), [0, 1, 0, 1]),
        (plane(), _plate([], 3), [0, 1, 0, 1]),
        (plane(), _plate(BUMP[:1], 3), [0, 1, 0, 1]),
    ]:
        requests.append(_line('gplate_surface', init + plate + bounds + uv))


def _cloud(n: int, f, t0: float = 0.0) -> list[float]:
    pts = []
    for i in range(n):
        pts += f(t0 + i / n)
    return [n, *pts]


def _average_requests(requests: list[str]) -> None:
    def ring(t):
        a = 2 * math.pi * t
        return [math.cos(a), math.sin(a), 0.2 * math.sin(3 * a)]

    def tilted(t):
        a = 2 * math.pi * t
        return [2 * math.cos(a), math.sin(a), 0.5 * math.cos(a) + 0.1 * math.sin(2 * a)]

    def line(t):
        return [t, 2 * t, -t]

    for cloud in (_cloud(20, ring), _cloud(24, tilted), _cloud(9, line)):
        n = int(cloud[0])
        for nb, tol, popt, nopt in [(0, 1e-7, 1, 1), (0, 1e-7, 2, 1), (n, 1e-7, 2, 2), (n // 2, 1e-7, 1, 1),
                                    (n, 1e-7, 1, 2), (0, 1e-7, 2, 2), (0, 1e-7, 1, 3), (n // 2, 1e3, 2, 1)]:
            requests.append(_line('gplate_average', cloud + [nb, tol, popt, nopt]))
    square = [4, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0]
    requests.append(_line('gplate_average', square + [4, 1e-7, 2, 2]))
    requests.append(_line('gplate_average', square + [0, 1e-7, 2, 1]))
    requests.append(_line('gplate_average', [1, 1, 2, 3, 0, 1e-7, 1, 1]))


def _normals_requests(requests: list[str]) -> None:
    pts = _cloud(12, lambda t: [math.cos(2 * math.pi * t), math.sin(2 * math.pi * t), 0.1 * t])
    s = math.sqrt(0.5)
    for normals in ([(0, 0, 1)], [(0, 0, 1), (0, s, s)], [(0, 0, 1), (0, 0, -1)], [(0, 0, 2), (0, 0, -1)],
                    [(0, 0, 1), (0, s, s), (s, 0, s)], [(0, 0, 1), (0.1, 0, 1), (0, 0.2, 1), (-0.1, -0.1, 1)],
                    [(1, 0, 0), (0, 1, 0), (0, 0, 1)], [(0, 0, 1), (0, 0, 1), (0, 0, 1)], [],
                    [(1, 0, 0), (-1, 0, 0), (0, 1, 0)], [(1, 0, 0)]):
        flat = [c for n in normals for c in n]
        requests.append(_line('gplate_normals', [len(normals), *flat, *pts]))
    requests.append(_line('gplate_normals', [1, 0, 0, 0, *pts]))


def _halfspace_requests(requests: list[str]) -> None:
    s = math.sqrt(0.5)
    steps_list = [
        [[(0, 0, 1)], [(0, s, s)], [(s, 0, s)], [(-s, 0, s)], [(0, -s, s)]],
        [[(0, 0, 1), (0, s, s)], [(s, 0, s), (0.5, 0.5, 0.7071)], [(0, 0, -1)]],
        [[(0, 0, 1), (0, 0, 1)]],
        [[(1, 0, 0), (0, 1, 0)], [(-1, 0, 0)]],
        [[(1, 0, 0), (0, 1, 0), (0, 0, 1)], [(0.577, 0.577, 0.577)], [(-1, -1, -1)]],
        [[(0, 0, 1)], [(0, 0.1, 1)], [(0.1, 0, 1)], [(-0.1, -0.1, 1)], [(0.05, 0.05, 1)]],
        [[]],
        [[(0, 0, 1), (0, 1, 0), (1, 0, 0), (-1, -1, 0.1)]],
    ]
    for lin, ang in [(1e-3, 1e-3), (0.2, 0.1)]:
        for steps in steps_list:
            fields = [lin, ang, len(steps)]
            for step in steps:
                fields += [len(step), *[c for n in step for c in n]]
            requests.append(_line('gplate_halfspace', fields))


def _build_requests(requests: list[str]) -> None:
    # Construction errors and the empty builder (OCC525).
    _build(requests, builder(0, [], [], nbiter=0))
    _build(requests, builder(0, [], [], degree=1))
    _build(requests, builder(0, [], []))
    _build(requests, builder(2, [], []))
    _build(requests, builder(2, [curve(1, seg((0, 0, 0), (1, 0, 0), 0, 1), 0, 0)], []))
    # Point constraints alone (A3/A4 style), degrees and initial surfaces.
    ring = [point((math.sin(2 * math.pi * i / 12), math.cos(2 * math.pi * i / 12), abs(1 - 2 * i / 12))) for i in range(12)]
    for degree in (2, 3, 4, 6):
        _build(requests, builder(0, [], ring, degree=degree, nbpts=15, nbiter=2))
    _build(requests, builder(1, [], ring, init=plane((0, 0, 0.5))))
    _build(requests, builder(0, [], ring, init=cylinder(3.0, (0, 0, 0), (1, 0, 0))))
    _build(requests, builder(0, [], [point((0, 0, 0)), point((1, 0, 0)), point((0, 1, 0)), point((1, 1, 0.1))]))
    _build(requests, builder(0, [], [point((0, 0, 0)), point((1, 0, 0), -1), point((0, 1, 0)), point((1, 1, 0.1), uv=(1.1, 0.9))]))
    bump = sphere(2.0, (0, 0, -1.5))
    tangent_points = [surface_point(u, v, bump, order) for (u, v, order) in
                      [(0.1, 0.2, 1), (1.0, 0.3, 1), (2.0, 0.25, 2), (3.0, 0.4, 0), (4.5, 0.3, 1), (0.5, 0.9, 2)]]
    _build(requests, builder(0, [], tangent_points, nbiter=1))
    _build(requests, builder(0, [], tangent_points, init=plane((0, 0, 0.5)), degree=4))
    # The four Bezier curves of the A1/A2 cases, the three constructors.
    # `gaps` 0 where G0Error(Index) would read values EcartContraintesMil never writes (a
    # curve with fewer intervals than NbPtsOnCur, or of order -1): uninitialized natively.
    for h, order, init, ctor, nbpts, nbiter, gaps in [(1.0, 0, NONE, 0, 8, 2, 1), (1.0, 0, plane(), 0, 8, 2, 0),
                                                      (1.0, 0, plane(), 1, 6, 1, 0), (0.5, 0, NONE, 2, 8, 2, 1),
                                                      (2.0, 0, NONE, 0, 6, 3, 1), (1.0, -1, NONE, 0, 6, 1, 0)]:
        _build(requests, builder(ctor, a_contour(h, order, 6), [], init=init, nbpts=nbpts, nbiter=nbiter), gaps=gaps)
    _build(requests, builder(0, a_contour(1.0), [point((1, 1, 1.2))], nbpts=6, nbiter=2))
    _build(requests, builder(0, a_contour(1.0), [point((1, 1, 1.2))], nbpts=0, nbiter=2))
    _build(requests, builder(0, a_contour(1.0)[:3], [], nbpts=6, nbiter=1), gaps=0)
    _build(requests, builder(0, a_contour(1.0) + [curve(1, seg((0.5, 1, 0.8), (1, 0, 0), 0, 1))], [], nbpts=6, nbiter=1, nbbounds=4),
           gaps=0)
    # G1 contours of curves on surfaces (the half space of the normals), then G2.
    for surface, u1, order, extra, gaps in [(plane(), 1.0, 1, [point((0.5, 0.5, 0.2))], 1), (plane(), 1.0, 1, [], 0),
                                            (cylinder(1.0), math.pi / 2, 1, [], 0), (sphere(2.0), 0.5, 2, [], 1),
                                            (cylinder(1.0), math.pi, 1, [], 1), (plane((0, 0, 0), (0, 0.2, 1)), 1.0, 2, [], 0)]:
        _build(requests, builder(0, square_on(surface, order, u1, 1.0, 6), extra, nbpts=6, nbiter=2), gaps=gaps)
    _build(requests, builder(1, square_on(plane(), 1, 1.0, 1.0, 6), [], init=plane(), nbpts=6, nbiter=2), gaps=0)
    _build(requests, builder(1, square_on(cylinder(1.0), 1, 1.0, 1.0, 6, False), [], init=cylinder(1.0), nbpts=6, nbiter=2),
           gaps=0)
    _build(requests, builder(2, square_on(sphere(2.0), 2, 0.5, 0.5, 6), [], nbiter=2))
    # Mixed orders: G0 Bezier sides against G1 sides on a plane, and G1 sides on two planes.
    mixed = [curve(3, on_line(plane(), (0, 0), (1, 0), 0, 1), 1, 6), curve(0, bezier((1, 0, 0), (1.2, 0.5, 0.3), (1, 1, 0)), 0, 6),
             curve(3, on_line(plane(), (1, 1), (-1, 0), 0, 1), 1, 6), curve(0, bezier((0, 1, 0), (-0.2, 0.5, 0.3), (0, 0, 0)), 0, 6)]
    _build(requests, builder(0, mixed, [], nbpts=6, nbiter=2))
    tilted = plane((0, 0, 0), (0, 0.3, 1))
    two_planes = [curve(3, on_line(plane(), (0, 0), (1, 0), 0, 1), 1, 6), curve(3, on_line(tilted, (1, 0), (0, 1), 0, 1), 1, 6),
                  curve(0, bezier((1, 1 / math.sqrt(1.09), -0.3 / math.sqrt(1.09)), (0.5, 1.1, 0), (0, 1, 0)), 0, 6),
                  curve(3, on_line(plane(), (0, 1), (0, -1), 0, 1), 1, 6)]
    _build(requests, builder(0, two_planes, [], nbpts=6, nbiter=2))
    # Criterion laws and tolerances.
    lawed = [curve(0, bezier((0, 0, 0), (1, 0, 1), (2, 0, 0)), 0, 6, laws=[1e-5, 1e-3]),
             curve(0, bezier((2, 0, 0), (2, 1, 1), (2, 2, 0)), 0, 6), curve(0, bezier((2, 2, 0), (1, 2, 1), (0, 2, 0)), 0, 6),
             curve(0, bezier((0, 2, 0), (0, 1, 1), (0, 0, 0)), 0, 6, tols=(1e-6, 0.01, 0.1))]
    _build(requests, builder(0, lawed, [], nbpts=6, nbiter=3, tols=(1e-5, 1e-5, 0.01)))
    _variation_requests(requests, mixed)


def _variation_requests(requests: list[str], mixed: list[list[float]]) -> None:
    """Degrees, iterations, tolerances and the anisotropy flag on a few contours, and point
    constraints of order G1 on a non-linear (free) plate."""
    square = square_on(plane(), 1, 1.0, 1.0, 6)
    bump = [point((0.5, 0.5, 0.2)), point((0.25, 0.25, 0.05), -1)]
    # The `gaps` flags of the square and mixed contours (see _build_requests).
    for degree, nbiter, tols, aniso, gaps in [(2, 1, (1e-5, 1e-4, 0.01), 0, (0, 0)), (4, 3, (1e-5, 1e-3, 0.1), 1, (0, 1)),
                                              (5, 4, (1e-4, 1e-5, 0.01), 0, (0, 1)), (3, 2, (1e-3, 1e-4, 0.001), 1, (1, 1))]:
        _build(requests, builder(0, square, bump, degree=degree, nbpts=6, nbiter=nbiter, tols=tols, aniso=aniso), gaps=gaps[0])
        _build(requests, builder(0, mixed, [], degree=degree, nbpts=6, nbiter=nbiter, tols=tols, aniso=aniso), gaps=gaps[1])
    half_cylinder = square_on(cylinder(1.0), 1, math.pi, 1.0, 5)
    free_points = [surface_point(1.5, 0.5, cylinder(1.2), 1), surface_point(0.8, 0.3, cylinder(1.1), 2)]
    _build(requests, builder(0, half_cylinder, free_points, nbpts=5, nbiter=2))
    _build(requests, builder(0, half_cylinder, [point((0, 0.5, 1.3))], nbpts=8, nbiter=1, degree=4), gaps=0)


def _approx_requests(requests: list[str]) -> None:
    plates = [(plane(), _plate(BUMP, 3)), (cylinder(2.0), _plate(SLOPE, 3)),
              (bezier_surface([0, .5, 0, .2, 1, .2, 0, .5, 0]), _plate(BUMP, 4))]
    for init, plate in plates:
        # Continuity codes: 0 C0, 1 G1 (refused by AdvApp2Var), 2 C1, 4 C2.
        for crit, cont, nbmax, dgmax in [(-1, 2, 1, 3), (0, 2, 4, 6), (1, 4, 3, 5), (0, 0, 1, 3), (2, 2, 1, 3),
                                         (-1, 4, 2, 8), (0, 1, 1, 3)]:
            requests.append(_line('gplate_approx', init + plate + [0, 1e-4, nbmax, dgmax, cont, 1.1, 1e-3, crit]))
        for mode, maximum, kind, rep, offset in [(1, 1e-3, 0, 0, (0, 0, 0)), (1, 1e-6, 1, 1, (0, 0, 1e-3)),
                                                 (2, 1e-2, 0, 1, (0, 0, 0)), (2, 1e-4, 1, 0, (0.1, 0, 0))]:
            requests.append(_line('gplate_approx', init + plate + [mode, 1e-3, 3, 5, 2, 1.3, maximum, kind, rep, *offset]))


def append_geomplate_requests(requests: list[str]) -> None:
    """Append every GeomPlate request family in a fixed order."""
    _point_requests(requests)
    _curve_requests(requests)
    _surface_requests(requests)
    _average_requests(requests)
    _normals_requests(requests)
    _halfspace_requests(requests)
    _build_requests(requests)
    _approx_requests(requests)
