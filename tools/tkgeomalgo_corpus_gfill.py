"""Deterministic requests for the skinning classes of the GeomFill filling part
(tools/oracle/tkgeomalgo_probe_gfill.cxx).

`gfill_line` reads `GeomFill_Line` points. `gfill_profiler` adds section curves of every
kind to a `GeomFill_Profiler` (B-splines of several degrees, rational, periodic and
mixed; Bezier curves; circles and ellipses, trimmed or not, which take the C1
approximation branch; trimmed lines), reads it before and after `Perform` (the
common degree, knots, poles and weights, with wrong array sizes and indices out of
range), including knot vectors that the knot insertion cannot unify (the mean-knot
branch). `gfill_secgen` runs the `GeomFill_SectionGenerator` queries and `SetParam` on
arrays of several lower bounds (the in-place copy of the native code). `gfill_appsurf`
skins the sections through `GeomFill_AppSurf` in every Perform mode, with imposed
parameters, and on lines shorter and longer than the section list.
"""
from __future__ import annotations

Point = tuple[float, float, float]


def _line(op: str, values: list[float]) -> str:
    return op + ' ' + ' '.join(repr(float(v)) for v in values)


def bspline(deg: int, knots: list[float], mults: list[int], poles: list[Point],
            weights: list[float] | None = None, periodic: bool = False) -> list[float]:
    """Kind 0: `deg periodic rational nk knots mults np poles [weights]`."""
    values: list[float] = [0, deg, int(periodic), int(weights is not None), len(knots), *knots, *mults, len(poles)]
    for pole in poles:
        values += list(pole)
    return values + (weights or [])


def bezier(poles: list[Point], weights: list[float] | None = None) -> list[float]:
    """Kind 1: `np poles rational [weights]`."""
    values: list[float] = [1, len(poles)]
    for pole in poles:
        values += list(pole)
    return values + [int(weights is not None)] + (weights or [])


def circle(center: Point, normal: Point, radius: float) -> list[float]:
    return [2, *center, *normal, radius]


def ellipse(center: Point, normal: Point, major: float, minor: float) -> list[float]:
    return [3, *center, *normal, major, minor]


def trimmed(u1: float, u2: float, basis: list[float]) -> list[float]:
    return [4, u1, u2, *basis]


def line(point: Point, direction: Point) -> list[float]:
    return [5, *point, *direction]


def curves(items: list[list[float]]) -> list[float]:
    values: list[float] = [len(items)]
    for item in items:
        values += item
    return values


def _row(z: float, xs: list[float], y: float = 0.0, bump: float = 0.0) -> list[Point]:
    return [(x, y + bump * x * (len(xs) - 1 - i) / max(1, len(xs) - 1), z) for i, x in enumerate(xs)]


SECTIONS: dict[str, list[list[float]]] = {
    'bspl_mixed': [
        bspline(2, [0, 0.5, 1], [3, 1, 3], _row(0.0, [0, 1, 2, 3], bump=0.4)),
        bspline(3, [0, 0.3, 0.7, 1], [4, 1, 1, 4], _row(2.0, [0, 0.6, 1.2, 2.2, 2.8, 3.1], 0.5, 0.3)),
    ],
    'circle_ellipse': [
        circle((0, 100, 100), (0, 1, 1), 50),
        ellipse((0, 0, 0), (0, 1, 1), 100, 50),
    ],
    'rational3': [
        bspline(2, [0, 1], [3, 3], [(0, 0, 0), (1, 1, 0), (2, 0, 0)], [1, 0.7, 1]),
        bspline(2, [0, 2], [3, 3], [(0, 0, 1), (1, 1.5, 1), (2, 0, 1)], [2, 1.2, 2]),
        bspline(3, [0, 0.5, 1], [4, 1, 4], _row(3.0, [0, 0.5, 1, 1.5, 2], 0.2, 0.1), [1, 1.1, 0.9, 1.3, 1]),
    ],
    'bezier_mix': [
        bezier([(0, 0, 0), (1, 2, 0), (3, 1, 0)]),
        bspline(2, [0, 1, 2], [3, 1, 3], _row(1.0, [0, 1, 2, 3])),
        trimmed(0.2, 2.5, circle((1.5, 0, 2), (0, 0, 1), 1.5)),
    ],
    'periodic': [
        bspline(2, [0, 1, 2, 3, 4], [1, 1, 1, 1, 1], [(1, 0, 0), (0, 1, 0), (-1, 0, 0), (0, -1, 0)], periodic=True),
        bspline(3, [0, 1, 2, 3, 4, 5], [1, 1, 1, 1, 1, 1],
                [(2, 0, 1), (1, 2, 1), (-1, 2, 1), (-2, 0, 1), (0, -2, 1)], periodic=True),
    ],
    'periodic_mixed': [
        bspline(2, [0, 1, 2, 3, 4], [1, 1, 1, 1, 1], [(1, 0, 0), (0, 1, 0), (-1, 0, 0), (0, -1, 0)], periodic=True),
        bspline(2, [0, 2, 4], [3, 1, 3], _row(1.0, [-1, 0, 1, 2])),
    ],
    'single': [bspline(3, [0, 1], [4, 4], _row(0.0, [0, 1, 2, 3], bump=1.0))],
    'lines': [
        trimmed(0, 2, line((0, 0, 0), (1, 0, 0))),
        trimmed(-1, 3, line((0, 1, 1), (1, 0.2, 0))),
    ],
    'middle_knots': [
        bspline(2, [0, 0.3, 0.6, 1], [3, 1, 1, 3], _row(0.0, [0, 1, 2, 3, 4])),
        bspline(2, [0, 0.3000004, 0.3000008, 1], [3, 1, 1, 3], _row(1.0, [0, 1, 2, 3, 4], bump=0.2)),
    ],
    'merge_fail': [
        bspline(2, [0, 0.5, 0.5000005, 1], [3, 1, 1, 3], _row(0.0, [0, 1, 2, 3, 4])),
        bspline(2, [0, 0.5000002, 1], [3, 1, 3], _row(1.0, [0, 1, 2, 3])),
    ],
    'three_ranges': [
        bspline(1, [2, 5], [2, 2], [(0, 0, 0), (3, 0, 0)]),
        bspline(3, [0, 1, 3], [4, 2, 4], _row(1.0, [0, 0.5, 1, 1.5, 2, 3], 0.0, 0.5)),
        bezier([(0, 0, 2), (1, 1, 2), (2, -1, 2), (3, 0, 2)], [1, 2, 2, 1]),
    ],
    'empty': [],
}


def _profiler_requests(requests: list[str]) -> None:
    for name, items in SECTIONS.items():
        for ptol, perform in ((1e-9, 1), (1e-6, 1), (0.05, 1), (1e-9, 0)):
            if name in ('middle_knots', 'merge_fail') and ptol == 0.05:
                continue
            requests.append(_line('gfill_profiler', [ptol, perform, *curves(items)]))


def _secgen_requests(requests: list[str]) -> None:
    for name, items in SECTIONS.items():
        n = len(items)
        requests.append(_line('gfill_secgen', [1e-9, 1, 0, *curves(items)]))
        params = [0.5 * i * i for i in range(n)]
        requests.append(_line('gfill_secgen', [1e-9, 1, n, *params, *curves(items)]))
    items = SECTIONS['rational3']
    for lower in (0, 2, -3):
        requests.append(_line('gfill_secgen', [1e-9, lower, 3, 1.5, 2.5, 4.0, *curves(items)]))
    requests.append(_line('gfill_secgen', [1e-9, 1, 2, 0.0, 1.0, *curves(items)]))


def _appsurf(items: list[list[float]], mode: int, *, params: list[float] | None = None, degrees=(3, 8),
             tols=(1e-7, 1e-9), nbit: int = 0, knownp: int = 0, nbmaxp: int = 10, nlin: int | None = None) -> str:
    prm = params or []
    nlin = len(items) if nlin is None else nlin
    values = [1e-9, len(prm), *prm, mode, *degrees, *tols, nbit, knownp, nbmaxp, nlin, *curves(items)]
    return _line('gfill_appsurf', values)


def _appsurf_requests(requests: list[str]) -> None:
    for name in ('bspl_mixed', 'circle_ellipse', 'rational3', 'bezier_mix', 'three_ranges', 'lines', 'periodic_mixed'):
        items = SECTIONS[name]
        for mode in range(4):
            requests.append(_appsurf(items, mode))
            requests.append(_appsurf(items, mode, degrees=(2, 5), tols=(1e-4, 1e-5), nbit=3))
        params = [0.0, 1.5, 2.25][:len(items)]
        for mode in (0, 1, 3):
            requests.append(_appsurf(items, mode, params=params, knownp=1))
    items = SECTIONS['rational3']
    requests.append(_appsurf(items, 0, nlin=2))
    requests.append(_appsurf(items, 1, nlin=4))
    requests.append(_appsurf(items, 3, nbmaxp=2))
    requests.append(_appsurf(SECTIONS['single'], 0, nlin=1))
    requests.append(_appsurf(SECTIONS['middle_knots'], 1))
    requests.append(_appsurf(SECTIONS['circle_ellipse'], 0, tols=(1e-7, 1e-9), degrees=(3, 8)))


def append_gfill_requests(requests: list[str]) -> None:
    """Append the GeomFill skinning requests: line, profiler, generator, AppSurf."""
    for n in (0, 1, 2, 7):
        requests.append(_line('gfill_line', [n, 4, -1, 0, 1, 99]))
    _profiler_requests(requests)
    _secgen_requests(requests)
    _appsurf_requests(requests)
