"""TKG3d corpus: the GeomGridEval grid evaluators (tools/oracle/tkg3d_probe_grid.cxx).

A parameter grid is written `lower n u*n` (the bounds of the NCollection_Array1 the
evaluators receive). Curve requests cover every curve kind, the dispatchers on handles and
adaptors, the specialized evaluators and the adaptor fallback; surface requests do the
same for surfaces, plus every adaptor resolution path of GeomGridEval_Surface
(transformed, revolution and extrusion adaptors, plain and foreign adaptors). Grids are
sorted and dense (span-cache runs), sparse, shuffled (the sorted-parameter fast path of
the B-spline loops), single, outside the range and empty. The corpus has its own seed.
"""
from __future__ import annotations
import math
import random
from tkg3d_corpus import (bezier, bspline, circle, ellipse, fmt, frame, hyperbola, line, offset, parabola, point,
                          trimmed, trsf, unit)
from tkg3d_corpus_bezier_surface import bezier_surface, net
from tkg3d_corpus_bspl import bspline_periodic, bspline_random, poles
from tkg3d_corpus_bspline_surface import bspline as bspline_surface
from tkg3d_corpus_eval_curves import helix, sine
from tkg3d_corpus_eval_surfaces import ellipsoid, helicoid
from tkg3d_corpus_offset_surface import c0_bspline, collapsed_bezier, collapsed_bspline, trim
from tkg3d_corpus_offset_surface import offset as offset_surface
from tkg3d_corpus_surfaces import cone, cylinder, extrusion, plane, revolution, sphere, torus

Z = [0.0, 0.0, 1.0]
X = [1.0, 0.0, 0.0]
ORIGIN_Z = [0.0, 0.0, 0.0, *Z, *X]
IDENTITY = [8] + [0.0] * 9  # a g3_trsf kind that sets nothing: the identity


def grid(values, lower=1):
    """A parameter grid with its lower bound; grid([0.0, 1.0], -3)."""
    return [lower, len(values), *values]


def linspace(first, last, count):
    step = (last - first) / (count - 1)
    return [first + i * step for i in range(count)]


def spline_under(spec):
    """The B-spline curve spec under trimmed (7) and offset (8) wrappers, or None."""
    while spec and spec[0] in (7, 8):
        spec = spec[5:] if spec[0] == 7 else spec[6:]
    return spec if spec and spec[0] == 6 else None


def flat_span(flat, degree, count, u):
    """The 1-based flat-knot span that BSplCLib::LocateParameter gives `u`."""
    spans = [s for s in range(degree + 1, count + 1) if flat[s - 1] < flat[s]]
    below = [s for s in spans if flat[s - 1] <= u]
    return below[-1] if below else spans[0]


def dn_safe(spec, values):
    """Drops the parameters whose flat span index equals the last knot index of a
    non-periodic B-spline curve: GeomGridEval_BSplineCurve::EvaluateGridDN passes that
    index to BSplCLib::DN with the multiplicities, and BuildKnots then reads past both
    arrays natively (undefined), where the port raises Standard_OutOfRange."""
    spline = spline_under(spec)
    if spline is None or spline[2]:
        return values
    degree, nk = spline[3], spline[4]
    knots = spline[5:5 + 2 * nk]
    flat = [k for k, m in zip(knots[::2], knots[1::2]) for _ in range(int(m))]
    count = len(flat) - degree - 1
    return [u for u in values if flat_span(flat, degree, count, u) != nk]


def curve_grids(rng: random.Random, first, last, spec):
    """Sorted dense grids, sparse and shuffled ones, a single value, values beyond the range
    and the empty grid, with varied lower bounds."""
    shuffled = linspace(first, last, 9)
    rng.shuffle(shuffled)
    outside = [first - 0.5 * (last - first), first, 0.5 * (first + last), last, last + 0.75 * (last - first)]
    sets = [(linspace(first, last, 21), 1), (linspace(first, last, 3), 0), (shuffled, -3),
            ([rng.uniform(first, last)], 7), (outside, 2)]
    return [grid(dn_safe(spec, values), lower) for values, lower in sets] + [grid([])]


def surface_grids(rng: random.Random, box, u_spec=None, v_spec=None):
    """(U, V) grid pairs over a parameter box: dense sorted, Nx1 and 1xN isolines, shuffled,
    outside the box and empty; `u_spec`/`v_spec` filter DN-unsafe curve parameters."""
    u1, u2, v1, v2 = box
    u_safe = (lambda values: dn_safe(u_spec, values)) if u_spec else (lambda values: values)
    v_safe = (lambda values: dn_safe(v_spec, values)) if v_spec else (lambda values: values)
    shuffled = linspace(u1, u2, 5)
    rng.shuffle(shuffled)
    pairs = [(linspace(u1, u2, 5), linspace(v1, v2, 4), 1, 1),
             (linspace(u1, u2, 9), [0.5 * (v1 + v2)], 0, 3),
             ([u1 + 0.3 * (u2 - u1)], linspace(v1, v2, 7), -2, 1),
             (shuffled, [v2, v1, 0.5 * (v1 + v2)], 1, -1),
             ([u1 - 0.25 * (u2 - u1), u2 + 0.25 * (u2 - u1)], [v1 - 0.5, v2 + 0.5], 4, 1)]
    out = [grid(u_safe(u), lu) + grid(v_safe(v), lv) for u, v, lu, lv in pairs]
    return out + [grid([]) + grid(linspace(v1, v2, 3)), grid(linspace(u1, u2, 3)) + grid([])]


def curve_specs(rng: random.Random):
    """(spec, first, last): lines and conics, Bezier and B-spline curves (multi-span,
    periodic, rational, C0, repeated knots), trimmed and offset curves of every basis (with a
    cusp for AdjustDerivative and a degenerate normal), and eval curves."""
    cubic = bspline(3, [[0.0, 4], [0.3, 1], [0.6, 2], [1.0, 4]], poles(rng, 7, False))
    c0 = bspline(2, [[0.0, 3], [0.5, 2], [1.0, 3]], poles(rng, 5, True), rational=True)
    cusp = bezier([[0, 0, 0, 1], [1, 1, 0, 1], [0, 1, 0, 1], [1, 0, 0, 1]])
    specs = [(line(point(rng), unit(rng)), -5.0, 5.0), (circle(frame(rng), 2.0), 0.0, 2 * math.pi),
             (circle(ORIGIN_Z, 0.0), 0.0, 1.0), (ellipse(frame(rng), 3.0, 1.0), -1.0, 7.0),
             (hyperbola(frame(rng), 2.0, 1.0), -2.0, 2.0), (parabola(frame(rng), 0.5), -3.0, 3.0),
             (parabola(ORIGIN_Z, 0.0), -1.0, 1.0), (hyperbola(ORIGIN_Z, 2.0, 1.0), -40.0, 700.0)]
    for degree in (1, 2, 3, 6):
        specs.append((bezier(poles(rng, degree + 1, False)), 0.0, 1.0))
        specs.append((bezier(poles(rng, degree + 1, True), True), 0.0, 1.0))
    for degree in (1, 2, 3, 5):
        specs.append((bspline_random(rng, degree, 3, False), 0.0, 1.0))
        specs.append((bspline_random(rng, degree, 2, True), 0.0, 1.0))
        specs.append((bspline_periodic(rng, degree, degree + 4, degree % 2 == 1), 0.0, float(degree + 4)))
    uniform = bspline(1, [[0.0, 2]] + [[float(i), 1] for i in range(1, 9)] + [[9.0, 2]], poles(rng, 10, False))
    specs += [(cubic, 0.0, 1.0), (c0, 0.0, 1.0), (uniform, 0.0, 9.0), (bspline_random(rng, 3, 2, False, 3), 0.0, 1.0)]
    wrapped = [(trimmed(0.5, 2.5, circle(ORIGIN_Z, 2.0)), 0.0, 3.0), (trimmed(0.1, 0.9, trimmed(0.0, 1.0, cubic)), 0.0, 1.0),
               (offset(0.5, Z, circle(ORIGIN_Z, 2.0)), 0.0, 2 * math.pi), (offset(-3.0, Z, circle(ORIGIN_Z, 2.0)), 0.0, 6.0),
               (offset(1.5, unit(rng), line(point(rng), unit(rng))), -2.0, 2.0), (offset(0.3, Z, ellipse(ORIGIN_Z, 3.0, 1.0)), 0.0, 6.0),
               (offset(0.2, Z, hyperbola(ORIGIN_Z, 1.0, 0.5)), -1.0, 1.0), (offset(0.5, Z, parabola(ORIGIN_Z, 1.0)), -1.0, 1.0),
               (offset(0.25, Z, cusp), 0.0, 1.0), (offset(-0.25, Z, cusp), 0.0, 1.0), (offset(0.2, Z, cubic), 0.0, 1.0),
               (offset(0.1, X, line([0, 0, 0], X)), 0.0, 1.0), (offset(0.2, Z, offset(0.3, Z, circle(ORIGIN_Z, 1.0))), 0.0, 6.0),
               (offset(0.1, Z, c0, True), 0.0, 1.0), (offset(0.4, Z, bspline_periodic(rng, 3, 7, True)), 0.0, 7.0),
               (offset(0.2, Z, trimmed(0.5, 2.0, circle(ORIGIN_Z, 1.0))), 0.0, 3.0),
               (sine(frame(rng), 2.0, 3.0, 0.5), -1.0, 4.0), (trimmed(0.5, 2.0, helix(frame(rng), 1.0, 2.0)), 0.0, 3.0),
               (offset(0.3, Z, sine(ORIGIN_Z, 1.0, 1.0, 0.0)), 0.0, 2.0)]
    return specs + wrapped


def append_grid_curve_requests(rng: random.Random, requests: list[str]) -> None:
    for spec, first, last in curve_specs(rng):
        for parameters in curve_grids(rng, first, last, spec):
            requests.append('g3_grid ' + fmt(spec + parameters))


def a3_surface(spec):
    """A GeomAdaptor_Surface of a3_surface (kind 0)."""
    return [0, *spec]


def curve_adaptor_specs(rng: random.Random):
    """(grid curve adaptor, first, last, curve spec for the DN filter)."""
    cubic = bspline(3, [[0.0, 4], [0.4, 1], [1.0, 4]], poles(rng, 5, False))
    sph = sphere(ORIGIN_Z, 2.0)
    return [([0, *cubic], 0.0, 1.0, cubic), ([1, 0.2, 0.7, *cubic], 0.0, 1.0, cubic),
            ([1, 0.5, 2.0, *circle(frame(rng), 1.0)], 0.0, 3.0, None), ([2], 0.0, 1.0, None),
            ([3, *trsf(rng, 3), *circle(frame(rng), 2.0)], 0.0, 6.0, None), ([3, *IDENTITY, *cubic], 0.0, 1.0, None),
            ([3, *trsf(rng, 7), *line(point(rng), unit(rng))], -1.0, 1.0, None),
            ([4, 0, 0.7, *a3_surface(sph)], -1.5, 1.5, None), ([4, 1, 0.3, *a3_surface(torus(ORIGIN_Z, 3.0, 1.0))], 0.0, 6.0, None),
            ([5, 0, 0, 0.2, 0.3, 0.4, 0.8, *a3_surface(cylinder(ORIGIN_Z, 1.5))], -1.0, 1.0, None),
            ([0, *sine(ORIGIN_Z, 1.0, 2.0, 0.0)], 0.0, 3.0, None)]


def append_grid_adaptor_requests(rng: random.Random, requests: list[str]) -> None:
    for spec, first, last, curve in curve_adaptor_specs(rng):
        for parameters in curve_grids(rng, first, last, curve or [])[:4]:
            requests.append('g3_grid_adaptor ' + fmt(spec + parameters))


def surface_specs(rng: random.Random):
    """(spec, box, U curve spec, V curve spec): every surface kind and its degenerate forms."""
    cubic = bspline(3, [[0.0, 4], [0.3, 1], [0.6, 2], [1.0, 4]], poles(rng, 7, False))
    periodic = bspline_periodic(rng, 3, 6, False)
    smooth = bspline_surface(rng, (3, 2), [[0.0, 4], [0.3, 1], [0.7, 2], [1.0, 4]], [[0.0, 3], [0.5, 1], [1.0, 3]]).values
    rational = bspline_surface(rng, (2, 2), [[0.0, 3], [0.5, 2], [1.0, 3]], [[0.0, 3], [0.4, 1], [1.0, 3]],
                               weights=lambda i, j: 1.0 + 0.5 * j).values
    per = bspline_surface(rng, (3, 2), [[float(k), 1] for k in range(6)], [[0.0, 3], [1.0, 3]], periodic=(True, False)).values
    linear = bspline_surface(rng, (1, 1), [[0.0, 2], [0.5, 1], [1.0, 2]], [[0.0, 2], [1.0, 2]]).values
    return [
        (plane(frame(rng)), (-1.0, 2.0, -3.0, 1.0), None, None), (cylinder(frame(rng), 2.0), (0.0, 7.0, -1.0, 2.0), None, None),
        (cylinder(ORIGIN_Z, 0.0), (0.0, 1.0, 0.0, 1.0), None, None), (cone(frame(rng), 0.4, 1.0), (0.0, 6.0, -3.0, 2.0), None, None),
        (cone(ORIGIN_Z, -0.3, 0.2), (0.0, 6.0, -1.0, 1.0), None, None), (sphere(frame(rng), 3.0), (0.0, 7.0, -1.6, 1.6), None, None),
        (torus(frame(rng), 4.0, 1.0), (0.0, 7.0, 0.0, 7.0), None, None), (torus(ORIGIN_Z, 1.0, 2.0), (0.0, 6.0, 0.0, 6.0), None, None),
        (extrusion(unit(rng), line(point(rng), unit(rng))), (-1.0, 1.0, -2.0, 2.0), None, None),
        (extrusion(unit(rng), circle(frame(rng), 2.0)), (0.0, 6.0, -1.0, 1.0), None, None),
        (extrusion(Z, cubic), (0.0, 1.0, -1.0, 1.0), cubic, None), (extrusion(Z, periodic), (0.0, 6.0, -1.0, 1.0), None, None),
        (extrusion(Z, offset(0.2, Z, circle(ORIGIN_Z, 1.0))), (0.0, 6.0, 0.0, 1.0), None, None),
        (extrusion(X, sine(ORIGIN_Z, 1.0, 2.0, 0.0)), (0.0, 3.0, 0.0, 1.0), None, None),
        (revolution([0.0, 0.0, 0.0, *Z], line([2.0, 0.0, 0.0], [0.0, 0.5, 1.0])), (0.0, 6.0, -1.0, 2.0), None, None),
        (revolution([0.0, 0.0, 0.0, *Z], circle([3.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0], 1.0)), (0.0, 6.0, 0.0, 6.0), None, None),
        (revolution([0.0, 0.0, 0.0, *Z], trimmed(0.0, 1.0, cubic)), (0.0, 6.0, 0.0, 1.0), None, trimmed(0.0, 1.0, cubic)),
        (revolution([0.0, 0.0, 0.0, *X], helix(ORIGIN_Z, 1.0, 1.0)), (0.0, 3.0, 0.0, 2.0), None, None),
        (bezier_surface(3, 4, net(rng, 3, 4)), (0.0, 1.0, 0.0, 1.0), None, None),
        (bezier_surface(3, 3, net(rng, 3, 3, lambda i, j, r: 1.0 + 0.5 * i), rational=True), (0.0, 1.0, 0.0, 1.0), None, None),
        (bezier_surface(2, 2, net(rng, 2, 2)), (0.0, 1.0, 0.0, 1.0), None, None),
        (smooth, (0.0, 1.0, 0.0, 1.0), None, None), (rational, (0.0, 1.0, 0.0, 1.0), None, None),
        (per, (0.0, 7.0, 0.0, 1.0), None, None), (linear, (0.0, 1.0, 0.0, 1.0), None, None),
        (trim(0.2, 1.4, -1.0, 2.0, cylinder(frame(rng), 1.0)), (0.2, 1.4, -1.0, 2.0), None, None),
        (trim(0.1, 0.9, 0.2, 0.8, trim(0.0, 1.0, 0.0, 1.0, smooth)), (0.1, 0.9, 0.2, 0.8), None, None),
        (offset_surface(0.5, plane(frame(rng))), (-1.0, 1.0, -1.0, 1.0), None, None),
        (offset_surface(-0.5, cylinder(frame(rng), 2.0)), (0.0, 6.0, -1.0, 1.0), None, None),
        (offset_surface(0.5, sphere(ORIGIN_Z, 0.5)), (0.0, 6.0, -1.5, 1.5), None, None),
        (offset_surface(0.2, smooth), (0.0, 1.0, 0.0, 1.0), None, None),
        (offset_surface(0.3, collapsed_bspline(rng)), (0.0, 1.0, 0.0, 1.0), None, None),
        (offset_surface(0.2, collapsed_bezier()), (0.0, 1.0, 0.0, 1.0), None, None),
        (offset_surface(-0.2, collapsed_bezier()), (0.0, 1.0, 0.0, 1.0), None, None),
        (offset_surface(0.3, c0_bspline(rng), True), (0.0, 1.0, 0.0, 1.0), None, None),
        (offset_surface(0.4, extrusion(Z, cubic)), (0.0, 1.0, -1.0, 1.0), None, None),
        (offset_surface(0.5, offset_surface(0.25, torus(frame(rng), 3.0, 1.0))), (0.0, 6.0, 0.0, 6.0), None, None),
        (offset_surface(0.2, revolution([0.0, 0.0, 0.0, *Z], line([2.0, 0.0, 0.0], [0.0, 0.5, 1.0]))), (0.0, 6.0, -1.0, 1.0), None, None),
        (offset_surface(0.1, ellipsoid(ORIGIN_Z, 3.0, 2.0, 1.0)), (0.0, 6.0, -1.5, 1.5), None, None),
        (ellipsoid(frame(rng), 3.0, 2.0, 1.0), (0.0, 6.0, -1.5, 1.5), None, None),
        (helicoid(ORIGIN_Z, 2.0), (0.0, 6.0, 0.0, 2.0), None, None),
    ]


def append_grid_surface_requests(rng: random.Random, requests: list[str]) -> None:
    for spec, box, u_spec, v_spec in surface_specs(rng):
        for parameters in surface_grids(rng, box, u_spec, v_spec):
            requests.append('g3_grid_surface ' + fmt(spec + parameters))


def surface_adaptor_specs(rng: random.Random):
    """(grid surface adaptor, box): every resolution path of GeomGridEval_Surface."""
    cubic = bspline(3, [[0.0, 4], [0.5, 1], [1.0, 4]], poles(rng, 5, False))
    smooth = bspline_surface(rng, (2, 2), [[0.0, 3], [0.5, 1], [1.0, 3]], [[0.0, 3], [1.0, 3]]).values
    tr_circle = [3, *trsf(rng, 3), *circle(frame(rng), 1.0)]
    return [
        ([0, 0, *cylinder(frame(rng), 1.0)], (0.0, 6.0, -1.0, 1.0)),
        ([0, 1, 0.2, 0.8, 0.1, 0.9, 1e-7, 1e-7, *smooth], (0.2, 0.8, 0.1, 0.9)),
        ([0, 2], (0.0, 1.0, 0.0, 1.0)),
        ([0, 3, *unit(rng), *cubic], (0.0, 1.0, -1.0, 1.0)),
        ([0, 0, *ellipsoid(ORIGIN_Z, 3.0, 2.0, 1.0)], (0.0, 6.0, -1.5, 1.5)),
        ([1, *Z, 0, *line([0, 0, 0], X)], (-1.0, 1.0, -1.0, 1.0)),
        ([1, *Z, 0, *circle(ORIGIN_Z, 2.0)], (0.0, 6.0, -1.0, 1.0)),
        ([1, *unit(rng), 1, 0.2, 0.8, *cubic], (0.2, 0.8, -1.0, 1.0)),
        ([1, *Z, *tr_circle], (0.0, 6.0, -1.0, 1.0)),
        ([2, 0.0, 0.0, 0.0, *Z, 0, *line([2.0, 0.0, 0.0], Z)], (0.0, 6.0, -1.0, 1.0)),
        ([2, 0.0, 0.0, 0.0, *Z, 0, *line([2.0, 0.0, 0.0], [-0.5, 0.0, 1.0])], (0.0, 6.0, -1.0, 1.0)),
        ([2, 0.0, 0.0, 0.0, *Z, 0, *circle([3.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0], 1.0)], (0.0, 6.0, 0.0, 6.0)),
        ([2, 0.0, 0.0, 0.0, *Z, 0, *circle([0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0], 2.0)], (0.0, 6.0, -1.5, 1.5)),
        ([2, 0.0, 0.0, 0.0, *Z, 0, *cubic], (0.0, 6.0, 0.0, 1.0)),
        ([2, 0.0, 0.0, 0.0, *X, *tr_circle], (0.0, 6.0, 0.0, 6.0)),
        ([3, *IDENTITY, *sphere(ORIGIN_Z, 2.0)], (0.0, 6.0, -1.5, 1.5)),
        ([3, *trsf(rng, 5), *plane(frame(rng))], (-1.0, 1.0, -1.0, 1.0)),
        ([3, *trsf(rng, 3), *smooth], (0.0, 1.0, 0.0, 1.0)),
        ([3, *trsf(rng, 7), *torus(frame(rng), 3.0, 1.0)], (0.0, 6.0, 0.0, 6.0)),
        ([3, *trsf(rng, 2), *offset_surface(0.3, collapsed_bezier())], (0.0, 1.0, 0.0, 1.0)),
        ([3, *trsf(rng, 4), *revolution([0.0, 0.0, 0.0, *Z], cubic)], (0.0, 6.0, 0.0, 1.0)),
        ([4, 0.2, 0.8, 0.1, 0.9, 0.1, 0.2, *trsf(rng, 6), *smooth], (0.2, 0.8, 0.1, 0.9)),
    ]


def append_grid_surface_adaptor_requests(rng: random.Random, requests: list[str]) -> None:
    for spec, box in surface_adaptor_specs(rng):
        for parameters in surface_grids(rng, box)[:3]:
            requests.append('g3_grid_surface_adaptor ' + fmt(spec + parameters))


def append_tkg3d_grid_requests(requests: list[str]) -> None:
    """Append the GeomGridEval requests; append_tkg3d_grid_requests(requests)."""
    rng = random.Random(20260928)
    append_grid_curve_requests(rng, requests)
    append_grid_adaptor_requests(rng, requests)
    append_grid_surface_requests(rng, requests)
    append_grid_surface_adaptor_requests(rng, requests)
    for u, v in ((grid([0.0, 0.5, 1.0]), grid([0.25, 0.75], -1)), (grid([]), grid([1.0])), (grid([2.0], -4), grid([]))):
        requests.append('g3_grid_null ' + fmt(u + v))
        requests.append('g3_grid_helpers ' + fmt(u + v))
