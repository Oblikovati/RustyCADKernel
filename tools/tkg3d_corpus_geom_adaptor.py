"""TKG3d corpus: the Adaptor3d base classes, GeomAdaptor_Curve, GeomAdaptor_Surface, the swept
and transformed surface adaptors and GeomAdaptor::MakeCurve/MakeSurface
(tools/oracle/tkg3d_probe_geom_adaptor.cxx).

Curve and surface adaptors are `0 <spec>` (whole) or `1 u1 u2 <spec>` / `1 u1 u2 v1 v2 tolu tolv
<spec>` (ranged). Parameters include the adaptor bounds exactly (the boundary spans of B-splines
and the snapped bounds of surfaces), interior knots, and values outside the ranges.
"""
from __future__ import annotations
import math
import random
from tkg3d_corpus import (axis, bezier, bspline, circle, ellipse, fmt, frame, hyperbola, line, offset, params,
                          parabola, point, trimmed, trsf, unit)
from tkg3d_corpus_bezier_surface import bezier_surface, net
from tkg3d_corpus_bspline_surface import bspline as bspline_surface
from tkg3d_corpus_offset_surface import collapsed_bezier, collapsed_bspline, trim
from tkg3d_corpus_offset_surface import offset as offset_surface
from tkg3d_corpus_surfaces import cone, cylinder, descriptor, extrusion, plane, revolution, sphere, torus

Z = [0.0, 0.0, 1.0]
X = [1.0, 0.0, 0.0]
ORIGIN_Z = [0.0, 0.0, 0.0, *Z, *X]
HUGE = 3e100  # beyond Precision::Infinite(): an infinite bound


def poles(rng: random.Random, count, rational=False):
    return [[*point(rng, 4.0), rng.uniform(0.5, 2.0) if rational else 1.0] for _ in range(count)]


def curve_specs(rng: random.Random) -> list[tuple[list, list, tuple]]:
    """Curve specs with evaluation parameters and a trim range, every kind of the adaptor."""
    cubic = bspline(3, [[0.0, 4], [0.3, 1], [0.6, 2], [1.0, 4]], poles(rng, 7))
    c0 = bspline(2, [[0.0, 3], [0.5, 2], [1.0, 3]], poles(rng, 5, True), rational=True)
    periodic = bspline(3, [[float(k), 1] for k in range(6)], poles(rng, 5), periodic=True)
    knots_u = [0.0, 0.3, 0.45, 0.6, 1.0, 1e-9, 0.999999999, -0.5, 1.5]
    return [
        (line(point(rng), unit(rng)), [0.0, 1.5, -3.0], (-1.0, 2.0)),
        (circle(frame(rng), 2.0), [0.0, 1.0, 2 * math.pi, 7.0], (0.5, 5.0)),
        (circle(frame(rng), 1e-9), [0.0, 1.0], (0.0, 1.0)),
        (ellipse(frame(rng), 3.0, 1.0), [0.0, 2.0, 4.5], (1.0, 3.0)),
        (hyperbola(frame(rng), 2.0, 1.0), [0.0, -1.0, 1.5], (-1.0, 1.0)),
        (parabola(frame(rng), 0.5), [0.0, -2.0, 3.0], (-1.0, 4.0)),
        (bezier(poles(rng, 4)), [0.0, 0.3, 1.0, 1.2], (0.2, 0.8)),
        (bezier(poles(rng, 5, True), rational=True), [0.0, 0.5, 1.0], (0.0, 0.5)),
        (cubic, knots_u, (0.3, 0.6)),
        (c0, [0.0, 0.25, 0.5, 0.75, 1.0], (0.5, 1.0)),
        (periodic, [0.0, 1.0, 2.5, 5.0, 6.5, -1.0], (1.0, 3.0)),
        (trimmed(0.5, 2.5, circle(frame(rng), 1.5)), [0.5, 1.0, 2.5], (1.0, 2.0)),
        (trimmed(0.2, 0.8, cubic), [0.2, 0.3, 0.8], (0.3, 0.7)),
        (offset(0.5, unit(rng), line(point(rng), unit(rng))), [0.0, 1.0], (0.0, 1.0)),
        (offset(-0.3, Z, circle(ORIGIN_Z, 2.0)), [0.0, 1.0, 3.0], (0.5, 1.5)),
        (offset(0.2, Z, cubic), knots_u, (0.25, 0.75)),
        (offset(0.2, Z, c0), [0.0, 0.5, 1.0], (0.2, 0.9)),
        (offset(0.1, Z, trimmed(0.1, 0.9, c0)), [0.1, 0.5, 0.9], (0.2, 0.6)),
        (offset(0.4, Z, periodic), [0.0, 2.0, 5.0], (1.0, 4.0)),
    ]


def curve_adaptors(spec, rng: random.Random):
    """Whole and ranged adaptors on one spec, plus an inverted and a half-infinite range."""
    return [[0, *spec], [1, 0.3, 0.6, *spec], [1, 0.3, 0.6 - 1e-8, *spec], [1, 0.6, 0.3, *spec],
            [1, -HUGE, 0.5, *spec] if spec[0] == 0 else [1, 0.0, 1.0, *spec]]


def append_curve_requests(rng: random.Random, requests: list[str]) -> None:
    specs = curve_specs(rng)
    for spec, u, (t1, t2) in specs:
        for adaptor in curve_adaptors(spec, rng):
            requests.append('a3_curve ' + fmt([*adaptor, *params(u + [0.3, 0.6]), t1, t2]))
    # The second curve is read unguarded: only kinds that always construct.
    seconds = [specs[i][0] for i in (0, 1, 8, 10, 11)]
    for index, (c1, u, (u1, u2)) in enumerate(specs):
        requests.append('a3_curve_load ' + fmt([*c1, *seconds[index % len(seconds)], *params(u), u1, u2]))
    requests.append('a3_curve_load ' + fmt([*specs[8][0], *specs[8][0], *params(specs[8][1]), 0.3, 0.6]))


def curve_descriptors(rng: random.Random):
    """g3_desc specs: full, bounded, mapped with and without a domain."""
    return [[0, 0, 0, 0.0, 1.0, 1.0, 0.0], [1, 2, 0, 0.0, 1.0, 1.0, 0.0], [2, 3, 1, 0.0, 1.0, 1.0, 0.0],
            [2, 1, 0, 0.0, 1.0, 2.0, -0.5]]


def append_curve_evalrep_requests(rng: random.Random, requests: list[str]) -> None:
    cubic = bspline(3, [[0.0, 4], [0.5, 1], [1.0, 4]], poles(rng, 5))
    curves = [bezier(poles(rng, 4)), cubic, offset(0.3, Z, cubic)]
    rep = bezier(poles(rng, 4))
    for curve in curves:
        for desc in curve_descriptors(rng):
            requests.append('a3_curve_evalrep ' + fmt([*curve, *rep, *desc, *params([0.0, 0.25, 0.5, 1.0]), 0.0, 1.0]))


def surface_specs(rng: random.Random) -> list[tuple[list, tuple]]:
    """Surface specs with a finite parameter box (for the ranged adaptors), every kind."""
    cubic = bspline(3, [[0.0, 4], [0.3, 1], [0.6, 2], [1.0, 4]], poles(rng, 7))
    smooth = bspline_surface(rng, (3, 2), [[0.0, 4], [0.3, 1], [0.7, 2], [1.0, 4]],
                             [[0.0, 3], [0.5, 1], [1.0, 3]]).values
    rational = bspline_surface(rng, (2, 2), [[0.0, 3], [0.5, 2], [1.0, 3]], [[0.0, 3], [0.4, 1], [1.0, 3]],
                               weights=lambda i, j: 1.0 + 0.5 * j).values
    per = [[float(k), 1] for k in range(6)]
    periodic = bspline_surface(rng, (3, 2), per, [[0.0, 3], [1.0, 3]], periodic=(True, False)).values
    return [
        (plane(frame(rng)), (-1.0, 2.0, -3.0, 1.0)),
        (cylinder(frame(rng), 2.0), (0.0, math.pi, -1.0, 2.0)),
        (cylinder(frame(rng), 1e-9), (0.0, 1.0, 0.0, 1.0)),
        (cone(frame(rng), 0.4, 1.0), (0.0, 2 * math.pi, -1.0, 2.0)),
        (cone(frame(rng), 0.4, 1.0), (0.0, 1.0, -10.0, 2.0)),
        (sphere(frame(rng), 3.0), (0.0, 2 * math.pi, -math.pi / 2, math.pi / 2)),
        (sphere(frame(rng), 1e-9), (0.0, 1.0, 0.0, 1.0)),
        (torus(frame(rng), 4.0, 1.0), (0.0, 2 * math.pi, 0.0, 2 * math.pi)),
        (extrusion(unit(rng), line(point(rng), unit(rng))), (-1.0, 1.0, -2.0, 2.0)),
        (extrusion(unit(rng), circle(frame(rng), 2.0)), (0.0, 3.0, -1.0, 1.0)),
        (extrusion(Z, cubic), (0.0, 1.0, -1.0, 1.0)),
        (revolution([0.0, 0.0, 0.0, *Z], line([2.0, 0.0, 0.0], [0.0, 0.5, 1.0])), (0.0, 3.0, -1.0, 2.0)),
        (revolution(axis(rng), circle(frame(rng), 1.0)), (0.0, 2.0, 0.0, 3.0)),
        (revolution([0.0, 0.0, 0.0, *Z], trimmed(0.0, 1.0, cubic)), (0.0, 6.0, 0.0, 1.0)),
        (bezier_surface(3, 4, net(rng, 3, 4)), (0.0, 1.0, 0.0, 1.0)),
        (bezier_surface(3, 3, net(rng, 3, 3, lambda i, j, r: 1.0 + 0.5 * i), rational=True), (0.2, 0.8, 0.0, 1.0)),
        (smooth, (0.0, 1.0, 0.0, 1.0)),
        (smooth, (0.3, 0.7, 0.5, 1.0)),
        (rational, (0.0, 1.0, 0.4, 1.0)),
        (periodic, (1.0, 4.0, 0.0, 1.0)),
        (trim(0.2, 1.4, -1.0, 2.0, cylinder(frame(rng), 1.0)), (0.2, 1.4, -1.0, 2.0)),
        (trim(0.1, 0.9, 0.2, 0.8, smooth), (0.1, 0.9, 0.2, 0.8)),
        (offset_surface(0.5, plane(frame(rng))), (-1.0, 1.0, -1.0, 1.0)),
        (offset_surface(-0.5, cylinder(frame(rng), 2.0)), (0.0, 3.0, -1.0, 1.0)),
        (offset_surface(0.2, smooth), (0.0, 1.0, 0.0, 1.0)),
        (offset_surface(0.3, collapsed_bspline(rng)), (0.0, 1.0, 0.0, 1.0)),
        (offset_surface(0.2, collapsed_bezier()), (0.0, 1.0, 0.0, 1.0)),
        (offset_surface(0.4, extrusion(Z, cubic)), (0.0, 1.0, -1.0, 1.0)),
    ]


def surface_uv(box) -> list[float]:
    """The bounds, a point inside, points within 1e-6 of the bounds and one outside."""
    u1, u2, v1, v2 = box
    um, vm = 0.5 * (u1 + u2), 0.5 * (v1 + v2)
    pairs = [(u1, v1), (u2, v2), (um, vm), (u1, vm), (um, v2), (u1 + 1e-7, v1), (u2, v2 - 5e-7),
             (u1 - 0.25, vm + 0.1)]
    return [c for pair in pairs for c in pair]


def append_surface_requests(rng: random.Random, requests: list[str]) -> None:
    specs = surface_specs(rng)
    for spec, box in specs:
        u1, u2, v1, v2 = box
        uv = surface_uv(box)
        um, vm = 0.5 * (u1 + u2), 0.5 * (v1 + v2)
        trims = [u1, um, vm, v2, 1e-6]
        for tol in (0.0, 1e-6):
            requests.append('a3_surface ' + fmt([1, u1, u2, v1, v2, tol, tol, *spec, *params(uv), *trims]))
        requests.append('a3_surface ' + fmt([0, *spec, *params(uv), *trims]))
        requests.append('a3_surface ' + fmt([1, u2, u1, v1, v2, 0.0, 0.0, *spec, *params(uv), *trims]))
    for (s1, box), (s2, _) in zip(specs, specs[1:] + specs[:1]):
        requests.append('a3_surface_load ' + fmt([*s1, *s2, *params(surface_uv(box)), *box, 1e-6]))


def append_surface_evalrep_requests(rng: random.Random, requests: list[str]) -> None:
    cubic = bspline(3, [[0.0, 4], [0.5, 1], [1.0, 4]], poles(rng, 5))
    smooth = bspline_surface(rng, (3, 2), [[0.0, 4], [0.3, 1], [1.0, 4]], [[0.0, 3], [1.0, 3]]).values
    surfaces = [smooth, bezier_surface(3, 3, net(rng, 3, 3)), offset_surface(0.2, smooth),
                extrusion(Z, cubic), revolution([0.0, 0.0, 0.0, *Z], trimmed(0.0, 1.0, cubic))]
    rep = plane(frame(rng))
    box = (0.0, 1.0, 0.0, 1.0)
    for surface in surfaces:
        for kind, order in ((0, 0), (1, 2), (2, 3)):
            requests.append('a3_surface_evalrep ' + fmt([*surface, *rep, *descriptor(rng, kind, order),
                                                         *params(surface_uv(box)), *box, 1e-6]))


def extrusion_curves(rng: random.Random) -> list[list]:
    """Curve adaptors whose extrusion is a plane, a cylinder or a general surface."""
    cubic = bspline(3, [[0.0, 4], [0.3, 1], [1.0, 4]], poles(rng, 5))
    circle_z = circle(ORIGIN_Z, 2.0)
    return [
        [0, *line(point(rng), X)], [1, -HUGE, HUGE, *line(point(rng), X)], [1, -HUGE, 2.0, *line(point(rng), X)],
        [1, -1.0, HUGE, *line(point(rng), X)], [1, -1.0, 1.0, *line([0.0, 0.0, 0.0], Z)],
        [0, *circle_z], [1, 0.5, 2.0, *circle_z], [0, *ellipse(ORIGIN_Z, 3.0, 1.0)],
        [0, *parabola(ORIGIN_Z, 0.5)], [1, -1.0, 1.0, *hyperbola(ORIGIN_Z, 2.0, 1.0)],
        [0, *cubic], [1, 0.2, 0.8, *cubic], [0, *bezier(poles(rng, 4))], [0, *offset(0.3, Z, cubic)],
    ]


def append_extrusion_requests(rng: random.Random, requests: list[str]) -> None:
    directions = [Z, X, [0.0, 0.6, 0.8], unit(rng)]
    uv = [0.0, 0.0, 0.5, 1.0, 1.0, -2.0, 0.3, 0.7]
    for curve in extrusion_curves(rng):
        for direction in directions:
            for mode in (0, 1):
                requests.append('a3_extrusion ' + fmt([*curve, *params(uv), *direction, mode, -1.0, 1.0, 0.2, 0.6]))


def revolution_curves(rng: random.Random) -> list[list]:
    """Meridians whose revolution about Z is a cylinder, plane, cone, sphere, torus or other."""
    cubic = bspline(3, [[0.0, 4], [0.3, 1], [1.0, 4]], [[2.0 + 0.3 * k, 0.1 * k, k - 1.0, 1.0] for k in range(5)])
    xz = [0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0]  # the XZ plane: circles coplanar with Z
    return [
        [0, *line([2.0, 0.0, 0.0], Z)], [1, -1.0, 2.0, *line([2.0, 0.0, 0.0], Z)],
        [1, -1.0, 2.0, *line([2.0, 0.0, 0.0], [0.0, 0.0, -1.0])], [0, *line([0.0, 0.0, 1.0], X)],
        [1, 0.5, 3.0, *line([0.0, 0.0, 1.0], X)], [0, *line([0.0, 0.0, 0.0], [0.6, 0.0, 0.8])],
        [1, 0.0, 2.0, *line([0.0, 0.0, 0.0], [0.6, 0.0, 0.8])], [1, -1.0, 2.0, *line([1.0, 0.0, 0.0], [0.6, 0.0, 0.8])],
        [1, 0.0, 2.0, *line([1.0, 1.0, 0.0], [0.6, 0.0, 0.8])], [1, -1.0, 1.0, *line([2.0, 0.0, 1.0], [1.0, 0.0, 1e-9])],
        [1, -1.0, 1.0, *line([0.0, 0.0, 0.0], Z)],
        [0, *circle(xz, 2.0)], [1, 0.0, 3.0, *circle(xz, 2.0)], [0, *circle([3.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0], 1.0)],
        [0, *circle([1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0], 2.0)],
        [0, *circle([3.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0], 1.0)], [0, *circle(frame(rng), 1.0)],
        [0, *cubic], [1, 0.2, 0.8, *cubic], [0, *ellipse([3.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0], 1.0, 0.5)],
    ]


def append_revolution_requests(rng: random.Random, requests: list[str]) -> None:
    axes = [[0.0, 0.0, 0.0, *Z], [0.0, 0.0, 0.0, 0.0, 0.0, -1.0], [0.5, 0.0, 0.0, *Z]]
    uv = [0.0, 0.0, 1.0, 0.5, 2 * math.pi, 1.0, 3.0, -0.5]
    for curve in revolution_curves(rng):
        for ax in axes:
            for mode in (0, 1):
                requests.append('a3_revolution ' + fmt([*curve, *params(uv), *ax, mode, 0.2, 0.9, 0.0, 1.0]))


def append_transformed_requests(rng: random.Random, requests: list[str]) -> None:
    for spec, box in surface_specs(rng)[::2]:
        for kind in (1, 3, 4, 5, 7):
            u1, u2, v1, v2 = box
            requests.append('a3_transformed ' + fmt([1, u1, u2, v1, v2, 0.0, 0.0, *spec, *params(surface_uv(box)),
                                                     *trsf(rng, kind), *trsf(rng, 3), 1e-6]))
    requests.append('a3_transformed ' + fmt([0, *plane(frame(rng)), *params([0.0, 0.0, 1.0, 2.0]),
                                             *trsf(rng, 5), *trsf(rng, 0), 0.0]))


def append_geom_adaptor_requests(requests: list[str]) -> None:
    """Append every GeomAdaptor request family in a fixed order."""
    rng = random.Random(0x3AD4)
    requests.append('a3_base ' + fmt(params([0.0, 0.5])))
    append_curve_requests(rng, requests)
    append_curve_evalrep_requests(rng, requests)
    append_surface_requests(rng, requests)
    append_surface_evalrep_requests(rng, requests)
    append_extrusion_requests(rng, requests)
    append_revolution_requests(rng, requests)
    append_transformed_requests(rng, requests)
