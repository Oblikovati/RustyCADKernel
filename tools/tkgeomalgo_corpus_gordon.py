"""Deterministic requests for GeomFill_Gordon and GeomFill_NetworkSurface
(tools/oracle/tkgeomalgo_probe_gordon.cxx).

The Gordon networks follow GeomFill_Gordon_Test: trimmed-line grids (square, non-uniform,
asymmetric, dense, reversed, shuffled, scaled, offset), quadratic bumps and saddles,
tilted planes, rational profiles or guides, degree-9 rational lines (the exact rational
degree overflow and its approximate fallback), rational quadratic lines that need exact
rational reparametrization, periodic and closed B-spline profiles and guides, meandering
profiles with several contacts per guide, circular arcs, Bezier and cubic B-spline
iso-curve networks of analytic surfaces (also translated and scaled), a disjoint network
and networks with too few curves. Each network runs in exact mode, in approximate-fallback
mode with the parallel flag and a second Perform, and with other tolerances.

The NetworkSurface requests give prepared networks: U- and V-closed seams (also with a
seam gap within and beyond the tolerance and non-unit parameters), rational families
with compatible and incompatible weights, the degree overflow, zero and negative weights,
too few curves, a grid of the wrong size, iso-curve networks of cubic surfaces and a
second Perform (which starts from the curves the first one made compatible).
"""
from __future__ import annotations
import math
from tkgeomalgo_corpus_intcurvesurface import bezier, bspline, circle, line, line_of, trimmed

ORIGIN = (0.0, 0.0, 0.0)


def lin(p1, p2):
    """`makeLinearBSpline`: a degree-1 B-spline on [0, 1]."""
    return bspline(1, [p1, p2], [0.0, 1.0], [2, 2])


def quad(p1, p2, p3, a=0.0, b=1.0, weights=None):
    """`makeQuadraticBSpline` (optionally rational) on [a, b]."""
    return bspline(2, [p1, p2, p3], [a, b], [3, 3], weights)


def tline(p, d, u1, u2):
    """`Geom_TrimmedCurve(Geom_Line(p, d), u1, u2)`."""
    return trimmed(u1, u2, line(list(p), list(d)))


def rational_line(p1, p2, degree):
    """`makeRationalLineBSpline`: a straight rational B-spline of the given degree."""
    poles = [[p1[k] * (1 - i / degree) + p2[k] * (i / degree) for k in range(3)] for i in range(degree + 1)]
    weights = [0.5 if i + 1 == (degree + 2) // 2 else 1.0 for i in range(degree + 1)]
    return bspline(degree, poles, [0.0, 1.0], [degree + 1, degree + 1], weights)


def rational_quad_line(p1, p2, w):
    mid = [0.5 * (p1[k] + p2[k]) for k in range(3)]
    return quad(p1, mid, p2, weights=list(w))


def periodic_cubic(poles):
    return bspline(3, poles, [0.2 * i for i in range(6)], [1] * 6, periodic=True)


RING = [(1.0, 0.0), (0.309, 0.951), (-0.809, 0.588), (-0.809, -0.588), (0.309, -0.951)]


def closed_profile(y, rx, rz):
    """`makeClosedBSplineProfile`: a clamped closed cubic."""
    poles = [(rx, y, 0.0), (rx, y, 0.7 * rz), (0.3 * rx, y, rz), (-0.7 * rx, y, 0.8 * rz), (-rx, y, 0.0),
             (-0.7 * rx, y, -0.8 * rz), (0.3 * rx, y, -rz), (rx, y, -0.7 * rz), (rx, y, 0.0)]
    return bspline(3, poles, [i / 6.0 for i in range(7)], [4, 1, 1, 1, 1, 1, 4])


# B-spline evaluation (Cox-de Boor) for iso-curves and intersection grids.
def flat_knots(knots, mults):
    return [k for k, m in zip(knots, mults) for _ in range(m)]


def basis(deg, flat, t):
    n = len(flat) - deg - 1
    span = deg
    while span < n - 1 and t >= flat[span + 1]:
        span += 1
    out = [0.0] * n
    local = [1.0] + [0.0] * deg
    for d in range(1, deg + 1):
        nxt = [0.0] * (deg + 1)
        for r in range(d + 1):
            i = span - d + r
            left = 0.0 if r == 0 else local[r - 1] * (t - flat[i]) / (flat[i + d] - flat[i])
            right = 0.0 if r == d else local[r] * (flat[i + d + 1] - t) / (flat[i + d + 1] - flat[i + 1])
            nxt[r] = left + right
        local = nxt
    for r in range(deg + 1):
        out[span - deg + r] = local[r]
    return out


def evaluate(curve, t):
    """The point of an encoded (possibly rational) B-spline curve at `t`."""
    deg, n = int(curve[1]), int(curve[2])
    poles = [curve[3 + 3 * i:6 + 3 * i] for i in range(n)]
    at = 3 + 3 * n
    weights = [1.0] * n
    if curve[at] == 1:
        weights = curve[at + 1:at + 1 + n]
        at += n
    nk = int(curve[at + 1])
    knots, mults = curve[at + 2:at + 2 + nk], [int(m) for m in curve[at + 2 + nk:at + 2 + 2 * nk]]
    b = basis(deg, flat_knots(knots, mults), t)
    den = sum(b[i] * weights[i] for i in range(n))
    return [sum(b[i] * weights[i] * poles[i][k] for i in range(n)) / den for k in range(3)]


CLAMPED = ([0.0, 0.5, 1.0], [4, 1, 4])


def iso_network(grid, profiles_v, guides_u):
    """Iso-curves of a bicubic B-spline surface (knots 0, 0.5, 1): profiles at v, guides at u."""
    flat = flat_knots(*CLAMPED)
    nu, nv = len(grid), len(grid[0])

    def iso(poles):
        return bspline(3, poles, *CLAMPED)
    profiles = [iso([[sum(basis(3, flat, v)[j] * grid[i][j][k] for j in range(nv)) for k in range(3)]
                     for i in range(nu)]) for v in profiles_v]
    guides = [iso([[sum(basis(3, flat, u)[i] * grid[i][j][k] for i in range(nu)) for k in range(3)]
                   for j in range(nv)]) for u in guides_u]
    return profiles, guides


def height_grid(f, origin=ORIGIN, scale=1.0):
    return [[[origin[0] + scale * i / 4.0, origin[1] + scale * j / 4.0, origin[2] + scale * f(i / 4.0, j / 4.0)]
             for j in range(5)] for i in range(5)]


def wavy(u, v):
    return 0.35 * math.sin(math.pi * u) * (0.4 + v) + 0.12 * math.sin(2 * math.pi * v) * (1 - u)


def saddle(u, v):
    return 0.6 * (u - 0.5) * (v - 0.5) + 0.15 * math.sin(math.pi * u) * math.sin(math.pi * v)


def uniform(n):
    return [i / (n - 1) for i in range(n)]


X, Y, NX, NY = (1.0, 0.0, 0.0), (0.0, 1.0, 0.0), (-1.0, 0.0, 0.0), (0.0, -1.0, 0.0)


def line_grid(ys, xs, length=1.0, z=0.0):
    """Trimmed-line profiles at `ys` and guides at `xs`."""
    return ([tline((0.0, y, z), X, 0.0, length) for y in ys], [tline((x, 0.0, z), Y, 0.0, length) for x in xs])


def unit_guides(xs=(0.0, 1.0)):
    return [lin((x, 0.0, 0.0), (x, 1.0, 0.0)) for x in xs]


def bump_profiles(h1, h2):
    return [quad((0, 0, 0), (0.5, 0, h1), (1, 0, 0)), quad((0, 0.5, 0), (0.5, 0.5, h2), (1, 0.5, 0)),
            quad((0, 1, 0), (0.5, 1, h1), (1, 1, 0))]


def meander(y):
    return bspline(3, [(0, y, 0), (0.9, y + 0.04, 0.01), (0.1, y - 0.04, -0.01), (1, y, 0)], [0.0, 1.0], [4, 4])


def arc(z, r):
    return trimmed(0.0, math.pi, circle([0.0, 0.0, z], [0.0, 0.0, 1.0], r))


def bernstein(t):
    return [(1 - t) ** 3, 3 * t * (1 - t) ** 2, 3 * t * t * (1 - t), t ** 3]


def bezier_network(profiles_v, guides_u):
    """Iso-curves of the bicubic Bezier surface on a 4 x 4 saddle grid."""
    grid = [[[i / 3.0, j / 3.0, saddle(i / 3.0, j / 3.0) + 0.2 * (i % 2)] for j in range(4)] for i in range(4)]
    profiles = [bezier([[sum(bernstein(v)[j] * grid[i][j][k] for j in range(4)) for k in range(3)]
                        for i in range(4)]) for v in profiles_v]
    guides = [bezier([[sum(bernstein(u)[i] * grid[i][j][k] for i in range(4)) for k in range(3)]
                      for j in range(4)]) for u in guides_u]
    return profiles, guides


def rational_network():
    return ([rational_line((0, y, 0), (1, y, 0), 9) for y in (0.0, 0.5, 1.0)],
            [rational_line((x, 0, 0), (x, 1, 0), 9) for x in (0.0, 0.5, 1.0)])


def rational_reparam_network():
    return ([rational_quad_line((0, 0, 0), (1, 0, 0), (1.0, 0.45, 1.0)),
             rational_quad_line((0, 0.4, 0), (1, 0.4, 0), (1.0, 1.8, 0.7)),
             rational_quad_line((0, 1, 0), (1, 1, 0), (0.8, 0.65, 1.5))], unit_guides((0.0, 0.4, 1.0)))


def periodic_point(poles, t):
    """The point at `t` of the periodic cubic on `poles` (knots 0, 0.2, ..., 1)."""
    wrapped = [*poles, *poles[:3]]
    b = basis(3, [0.2 * i - 0.6 for i in range(12)], t)
    return [sum(b[i] * wrapped[i][k] for i in range(len(wrapped))) for k in range(3)]


def periodic_network(transpose):
    """`makePeriodicProfile` rings and linear guides at 0, 0.5 and 1 (or the transposed
    periodic guides and linear profiles)."""
    rings = []
    for c in (0.0, 0.5, 1.0):
        rings.append([(c, x, z) if transpose else (x, c, z) for x, z in RING])
    ends = [lin(periodic_point(rings[0], t), periodic_point(rings[2], t)) for t in (0.0, 0.5, 1.0)]
    periodic = [periodic_cubic(r) for r in rings]
    return (ends, periodic) if transpose else (periodic, ends)


def closed_network():
    profiles = [closed_profile(0.0, 1.0, 1.0), closed_profile(1.0, 1.15, 0.85)]
    guides = [lin(evaluate(profiles[0], t), evaluate(profiles[1], t)) for t in (0.12, 0.34, 0.59, 0.83)]
    return profiles, guides


def gordon_networks():
    """(profiles, guides, tolerance) of every Gordon network."""
    nets = [(*line_grid((0.0, 1.0), (0.0, 1.0)), 1e-7), (*line_grid((0.0, 0.5, 1.0), (0.0, 0.5, 1.0)), 1e-7),
            (*line_grid((0.0, 0.2, 0.7, 1.0), (0.0, 0.3, 1.0)), 1e-7),
            (*line_grid([0.25 * i for i in range(5)], (0.0, 1.0)), 1e-7),
            (*line_grid((0.0, 1.0), [0.25 * i for i in range(5)]), 1e-7),
            (*line_grid([i / 5 for i in range(6)], [i / 5 for i in range(6)]), 1e-7),
            (*line_grid((0.0, 1000.0), (0.0, 1000.0), 1000.0), 0.1),
            (*line_grid((0.0, 1e-3), (0.0, 1e-3), 1e-3), 1e-7), (*line_grid((0.0, 1.0), (0.0, 1.0), z=5.0), 1e-7),
            ([tline((0, 0, 0), X, 0.0, 2.0), tline((0, 3, 0), X, 0.0, 2.0)],
             [tline((0, 0, 0), Y, 0.0, 3.0), tline((2, 0, 0), Y, 0.0, 3.0)], 1e-7),
            ([tline((1, 0, 0), NX, 0.0, 1.0), tline((0, 1, 0), X, 0.0, 1.0)],
             [tline((0, 0, 0), Y, 0.0, 1.0), tline((1, 1, 0), NY, 0.0, 1.0)], 1e-7),
            ([tline((1, 0, 0), NX, 0.0, 1.0), tline((1, 1, 0), NX, 0.0, 1.0)],
             [tline((0, 1, 0), NY, 0.0, 1.0), tline((1, 1, 0), NY, 0.0, 1.0)], 1e-7),
            ([tline((0, y, 0), X, 0.0, 1.0) for y in (1.0, 0.5, 0.0)],
             [tline((x, 0, 0), Y, 0.0, 1.0) for x in (1.0, 0.0)], 1e-7),
            (*line_grid((0.0, 1.0), (2.0, 3.0)), 1e-7),
            (*line_grid((0.0,), (0.0,)), 1e-7), (*line_grid((0.0,), (0.0, 1.0)), 1e-7),
            (bump_profiles(0.4, 0.6), unit_guides(), 1e-7), (bump_profiles(0.3, 0.5), unit_guides(), 0.01),
            ([quad((0, 0, 0), (0.5, 0, 0.5), (1, 0, 0)), quad((0, 1, 0), (0.5, 1, 0.5), (1, 1, 0))],
             [quad((0, 0, 0), (0, 0.5, -0.5), (0, 1, 0)), quad((1, 0, 0), (1, 0.5, -0.5), (1, 1, 0))], 0.01),
            ([lin((0, 0, 0), (1, 0, 1)), lin((0, 1, 1), (1, 1, 2))],
             [lin((0, 0, 0), (0, 1, 1)), lin((1, 0, 1), (1, 1, 2))], 1e-7),
            ([quad((0, y, 0), (0.5, y, 0), (1, y, 0), weights=[1.0, 0.5, 1.0]) for y in (0.0, 0.5, 1.0)],
             unit_guides((0.0, 0.5, 1.0)), 1e-7),
            ([lin((0, y, 0), (1, y, 0)) for y in (0.0, 0.5, 1.0)],
             [quad((x, 0, 0), (x, 0.5, 0), (x, 1, 0), weights=[1.0, 0.5, 1.0]) for x in (0.0, 0.5, 1.0)], 1e-7),
            (*rational_network(), 1e-7), (*rational_reparam_network(), 1e-7), (*closed_network(), 1e-7),
            ([meander(y) for y in (0.0, 0.5, 1.0)], unit_guides((0.0, 0.5, 1.0)), 1e-5),
            ([arc(0.0, 1.0), arc(1.0, 1.0)], [lin((1, 0, 0), (1, 0, 1)), lin((-1, 0, 0), (-1, 0, 1))], 1e-7),
            (*periodic_network(False), 1e-7), (*periodic_network(True), 1e-7),
            (*bezier_network((0.0, 0.5, 1.0), (0.0, 0.5, 1.0)), 1e-7),
            (*bezier_network((0.0, 1.0), (0.0, 0.3, 0.7, 1.0)), 1e-7),
            ([quad((0, 0, 0), (0.5, 0, 0.4), (1, 0, 0), 2.0, 5.0), quad((0, 1, 0), (0.5, 1, 0.4), (1, 1, 0), -1.0, 3.0)],
             unit_guides(), 1e-7)]
    for f, origin, scale, pv, gu in ((wavy, ORIGIN, 1.0, uniform(3), uniform(3)),
                                     (wavy, ORIGIN, 1.0, uniform(5), uniform(4)),
                                     (saddle, ORIGIN, 1.0, uniform(4), uniform(6)),
                                     (saddle, (1e5, -2e5, 3e5), 250.0, uniform(4), uniform(3)),
                                     (wavy, ORIGIN, 0.01, uniform(3), uniform(4))):
        tol = 1e-5 if scale > 1.0 else 1e-7
        nets.append((*iso_network(height_grid(f, origin, scale), pv, gu), tol))
    nets.append((*iso_network(height_grid(wavy), (0.0, 0.3, 1.0), (0.0, 0.6, 1.0)), 1e-7))
    return nets


def gordon(mode, parallel, tol, profiles, guides, performs) -> str:
    values = [mode, parallel, tol, len(profiles), *[v for c in profiles for v in c], len(guides),
              *[v for c in guides for v in c], performs]
    return line_of('gfill_gordon', values)


def append_gordon_requests(requests: list[str]) -> None:
    """GeomFill_Gordon: every network in the three modes."""
    for profiles, guides, tol in gordon_networks():
        requests.append(gordon(0, 0, tol, profiles, guides, 1))
        requests.append(gordon(1, 1, tol, profiles, guides, 2))
        requests.append(gordon(0, 0, 1e-3 if tol < 1e-3 else 1e-7, profiles, guides, 1))
    profiles, guides, tol = gordon_networks()[0]
    requests.append(gordon(0, 1, tol, profiles, guides, 0))
    requests.append(gordon(1, 0, tol, guides, profiles, 1))


def network(profiles, guides, vparams, uparams, weights=None, tol=1e-7, closed=(0, 0), performs=1, grid=None):
    """A GeomFill_NetworkSurface request; the points are profile(u) per (guide, profile)."""
    if grid is None:
        grid = [[evaluate(p, u) for p in profiles] for u in uparams]
    rows, cols = len(grid), len(grid[0])
    if weights is None:
        weights = [[1.0] * cols for _ in range(rows)]
    values = [len(profiles), *[v for c in profiles for v in c], len(guides), *[v for c in guides for v in c],
              len(vparams), *vparams, len(uparams), *uparams, 1, rows, 1, cols,
              *[c for row in grid for p in row for c in p], *[w for row in weights for w in row],
              tol, *closed, performs]
    return line_of('gfill_network', values)


def closed_quads(gap):
    profiles = [quad((0, y, 0), (1, y, h), (gap, y, 0)) for y, h in ((0.0, 0.0), (1.0, 1.0), (2.0, 0.0))]
    guides = [lin((0, 0, 0), (0, 2, 0)), lin((1, 0, 0), (1, 2, 0)), lin((gap, 0, 0), (gap, 2, 0))]
    return profiles, guides


def append_network_requests(requests: list[str]) -> None:
    """GeomFill_NetworkSurface: prepared networks of every status."""
    half = [0.0, 0.5, 1.0]
    for gap, tol in ((0.0, 1e-7), (1e-5, 1e-4), (1e-5, 1e-7), (1e-2, 1e-4)):
        profiles, guides = closed_quads(gap)
        requests.append(network(profiles, guides, half, half, tol=tol, closed=(1, 0)))
        requests.append(network(profiles, guides, half, half, tol=tol, closed=(0, 0), performs=2))
    vprofiles = [lin((0, 0, 0), (2, 0, 0)), lin((0, 1, 0), (2, 1, 1)), lin((0, 0, 0), (2, 0, 0))]
    vguides = [quad((x, 0, 0), (x, 1, 0), (x, 0, 0)) for x in (0.0, 1.0, 2.0)]
    requests.append(network(vprofiles, vguides, half, half, closed=(0, 1)))
    requests.append(network(vprofiles, vguides, half, half, closed=(1, 1)))
    lines = [lin((0, 0, 0), (1, 0, 0)), lin((0, 1, 0), (1, 1, 0))]
    guides2 = unit_guides()
    requests.append(network(lines[:1], guides2, [0.0], [0.0, 1.0]))
    for w in (0.0, -1.0, 1e-300):
        requests.append(network(lines, guides2, [0.0, 1.0], [0.0, 1.0], weights=[[1.0, 1.0], [1.0, w]]))
    requests.append(network(lines, guides2, [0.0, 1.0], [0.0, 1.0], grid=[[[0.0, 0.0, 0.0], [0.0, 1.0, 0.0]]]))
    requests.append(network(lines, guides2, [0.0, 1.0], [1.0, 0.0]))
    rprofiles = [quad((0, y, 0), (0.5, y, 0), (1, y, 0), weights=[1.0, 0.5, 1.0]) for y in half]
    rguides = [bspline(1, [(x, 0, 0), (x, 1, 0)], [0.0, 1.0], [2, 2], [w, w]) for x, w in ((0, 1.0), (0.5, 0.75), (1, 1.0))]
    rweights = [[w] * 3 for w in (1.0, 0.75, 1.0)]
    requests.append(network(rprofiles, rguides, half, half, weights=rweights))
    requests.append(network(rprofiles, rguides, half, half, weights=rweights, performs=2))
    requests.append(network(rprofiles, unit_guides(half), half, half))
    requests.append(network([rprofiles[0], bspline(1, [(0, 1, 0), (1, 1, 0)], [0.0, 1.0], [2, 2], [1.0, 1.0])],
                            guides2, [0.0, 1.0], [0.0, 1.0]))
    requests.append(network(*rational_network(), half, half))
    nprofiles = [quad((0, 0, 0), (1, 0, 0), (0, 0, 0), 2.0, 5.0), quad((0, 1, 0), (1, 1, 1), (0, 1, 0), 2.0, 5.0)]
    nguides = [lin(evaluate(nprofiles[0], t), evaluate(nprofiles[1], t)) for t in (2.0, 3.5)]
    requests.append(network(nprofiles, nguides, [0.0, 1.0], [2.0, 3.5], closed=(1, 0)))
    requests.append(network(nprofiles, nguides, [0.0, 1.0], [2.0, 3.5], closed=(0, 0)))
    for f, origin, scale in ((wavy, ORIGIN, 1.0), (saddle, (10.0, -5.0, 2.0), 3.0)):
        grid = height_grid(f, origin, scale)
        for pv, gu in ((uniform(3), uniform(3)), (uniform(5), uniform(4)), ((0.0, 0.3, 1.0), (0.0, 0.6, 1.0))):
            profiles, guides = iso_network(grid, pv, gu)
            requests.append(network(profiles, guides, list(pv), list(gu)))


def append_gordon_package_requests(requests: list[str]) -> None:
    """The GeomFill Gordon requests in a fixed order."""
    append_gordon_requests(requests)
    append_network_requests(requests)
