"""Deterministic requests for the GC package (tools/oracle/tkgeombase_probe_gc*.cxx).

Every constructor overload of every GC_Make* class gets random valid inputs plus the
degenerate ones that reach each status and each raising Geom, Geom2d or gp constructor:
equal trim parameters, points on and off the conics, tangents null, along or almost
along the chord, confused and collinear points, null and negative radii and focal
lengths, angles at the resolution bounds, null scale factors and parallel axes.

Inputs that make a nested stack gce algorithm read its unassigned status (three points
within gp::Resolution() for the 3D circle, collinear points for the 2D circle) are
excluded: the native result depends on stack contents there.
"""
from __future__ import annotations
import math
import random
from tkg3d_corpus import axis, frame, point
from tkgeombase_corpus_gce import AXES, RES, TINY, add, axis_frames, frame2d, near, normalized, point2d, unit2d


def circ_spec(rng, radius=None):
    return [*frame(rng), rng.uniform(0.5, 5.0) if radius is None else radius]


def conic_specs(rng):
    """(op suffix, 3D spec) pairs for circle, ellipse, hyperbola and parabola."""
    f = frame(rng)
    return [('circle', [*f, 2.5]), ('ellipse', [*f, 3.0, 1.5]), ('hyperbola', [*f, 2.0, 1.0]), ('parabola', [*f, 0.75])]


def conic2d_specs(rng):
    f = frame2d(rng)
    return [('circle2d', [*f, 2.5]), ('ellipse2d', [*f, 3.0, 1.5]), ('hyperbola2d', [*f, 2.0, 1.0]),
            ('parabola2d', [*f, 0.75])]


def arc_requests(rng, requests):
    """The three shared overloads of every arc class, 3D and 2D."""
    for _ in range(3):
        for suffix, spec in conic_specs(rng) + conic2d_specs(rng):
            op = 'gc_arc_' + suffix
            two_d = suffix.endswith('2d')
            pt = (lambda: point2d(rng, 4.0)) if two_d else (lambda: point(rng, 4.0))
            for a, b in [(0.3, 2.1), (2.1, 0.3), (-1.0, 1.0), (0.5, 0.5), (0.0, 2 * math.pi), (-7.0, 9.5)]:
                for sense in (0, 1):
                    add(requests, op, 0, *spec, a, b, sense)
            for sense in (0, 1):
                add(requests, op, 1, *spec, *pt(), 1.25, sense)
                add(requests, op, 2, *spec, *pt(), *pt(), sense)
                p = pt()
                add(requests, op, 2, *spec, *p, *p, sense)
                add(requests, op, 1, *spec, *spec[0:(2 if two_d else 3)], 0.0, sense)


def arc_through_requests(rng, requests):
    """GC_MakeArcOfCircle(2d) through three points and from a tangent."""
    for _ in range(10):
        add(requests, 'gc_arc_circle', 3, *point(rng), *point(rng), *point(rng))
        add(requests, 'gc_arc_circle2d', 3, *point2d(rng), *point2d(rng), *point2d(rng))
    a, b = point(rng), point(rng)
    add(requests, 'gc_arc_circle', 3, *a, *b, *[2 * y - x for x, y in zip(a, b)])
    add(requests, 'gc_arc_circle', 3, *a, *a, *b)
    add(requests, 'gc_arc_circle', 3, *a, *b, *b)
    add(requests, 'gc_arc_circle', 3, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, -1.0, 0.0, 0.0)
    # Two confused 2D points make the bisectors parallel (the unassigned gce status).
    add(requests, 'gc_arc_circle2d', 3, 1.0, 0.0, 0.0, 1.0, -1.0, 0.0)
    add(requests, 'gc_arc_circle2d', 3, 1.0, 0.0, 0.0, -1.0, -1.0, 0.0)
    for _ in range(8):
        add(requests, 'gc_arc_circle', 4, *point(rng), *point(rng), *point(rng))
        add(requests, 'gc_arc_circle2d', 4, *point2d(rng), *point2d(rng), *point2d(rng))
    tangent_requests(rng, requests)


def tangent_requests(rng, requests):
    """Tangents null, along, almost along and across the chord, and confused points."""
    p1, p2 = [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]
    chord = [p2[i] - p1[i] for i in range(3)]
    for v in [[0.0, 1.0, 0.0], [0.0, 0.0, 0.0], chord, [-c for c in chord], [chord[0], chord[1] + 1e-13, 0.0],
              [chord[0], chord[1] + 1e-9, 0.0], [0.0, 0.0, 1.0], [1e-160, 1.0, 0.0], [2e-5, 0.0, 0.0]]:
        add(requests, 'gc_arc_circle', 4, *p1, *v, *p2)
        add(requests, 'gc_arc_circle2d', 4, *p1[0:2], *v[0:2], *p2[0:2])
    for delta in TINY:
        add(requests, 'gc_arc_circle', 4, *p1, 0.0, 1.0, 0.0, *near(p1, delta))
        add(requests, 'gc_arc_circle2d', 4, *p1[0:2], 0.0, 1.0, *near(p1[0:2], delta))
    scale = 2e-7
    add(requests, 'gc_arc_circle', 4, scale, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, scale, 0.0)


def circle_requests(rng, requests):
    for f in [frame(rng) for _ in range(3)] + axis_frames(rng)[:4]:
        for radius in [2.0, 0.0, -1.0]:
            add(requests, 'gc_circle', 1, *f, radius)
            add(requests, 'gc_circle', 2, *f, 3.0, radius - 3.0)
            add(requests, 'gc_circle', 5, *f[0:6], radius)
            add(requests, 'gc_circle', 7, *f[0:6], radius)
            add(requests, 'gc_circle', 6, *f[0:3], *[a + 2 * b for a, b in zip(f[0:3], f[3:6])], radius)
        add(requests, 'gc_circle', 0, *f, 1.5)
        add(requests, 'gc_circle', 3, *f, 1.5, *point(rng))
        add(requests, 'gc_circle', 6, *f[0:3], *f[0:3], 1.0)
    for _ in range(8):
        add(requests, 'gc_circle', 4, *point(rng), *point(rng), *point(rng))
    a, b = point(rng), point(rng)
    add(requests, 'gc_circle', 4, *a, *b, *[(x + y) / 2 for x, y in zip(a, b)])
    add(requests, 'gc_circle', 4, *a, *a, *b)
    circle2d_requests(rng, requests)


def circle2d_requests(rng, requests):
    for _ in range(4):
        ax, f2 = [*point2d(rng), *unit2d(rng)], frame2d(rng)
        for radius in [2.0, 0.0, -1.0]:
            for sense in (0, 1):
                add(requests, 'gc_circle2d', 1, *ax, radius, sense)
                add(requests, 'gc_circle2d', 6, *ax[0:2], radius, sense)
            add(requests, 'gc_circle2d', 2, *f2, radius)
            add(requests, 'gc_circle2d', 3, *f2, 1.5, radius)
        add(requests, 'gc_circle2d', 0, *f2, 1.5)
        add(requests, 'gc_circle2d', 4, *f2, 1.5, *point2d(rng))
        add(requests, 'gc_circle2d', 4, *f2, 1.5, *f2[0:2])
        for sense in (0, 1):
            add(requests, 'gc_circle2d', 7, *ax[0:2], *point2d(rng), sense)
    for _ in range(8):
        add(requests, 'gc_circle2d', 5, *point2d(rng), *point2d(rng), *point2d(rng))
    add(requests, 'gc_circle2d', 5, 0.0, 0.0, 1e-150, 0.0, 0.0, 1.0)


def conic_requests(rng, requests):
    radii = [(10.0, 5.0), (5.0, 5.0), (3.0, 7.0), (0.0, 0.0), (5.0, -1.0), (-1.0, 5.0), (-1.0, -3.0)]
    for f in [frame(rng) for _ in range(2)] + axis_frames(rng)[:2]:
        for major, minor in radii:
            add(requests, 'gc_ellipse', 1, *f, major, minor)
            add(requests, 'gc_hyperbola', 1, *f, major, minor)
        add(requests, 'gc_ellipse', 0, *f, 4.0, 2.0)
        add(requests, 'gc_hyperbola', 0, *f, 4.0, 2.0)
    for _ in range(2):
        ax, f2 = [*point2d(rng), *unit2d(rng)], frame2d(rng)
        for major, minor in radii:
            for sense in (0, 1):
                add(requests, 'gc_ellipse2d', 2, *ax, major, minor, sense)
                add(requests, 'gc_hyperbola2d', 1, *ax, major, minor, sense)
            add(requests, 'gc_ellipse2d', 1, *f2, major, minor)
            add(requests, 'gc_hyperbola2d', 2, *f2, major, minor)
        add(requests, 'gc_ellipse2d', 0, *f2, 4.0, 2.0)
        add(requests, 'gc_hyperbola2d', 0, *f2, 4.0, 2.0)
    for s1, s2 in [([10, 0, 0], [0, 5, 0]), ([10, 0, 0], [5, 0, 0]), ([3, 0, 0], [0, 7, 0]), ([0, 0, 0], [1, 1, 0]),
                   ([1, 2, 0], [4, 0, 0]), ([2 * RES, 0, 0], [0, 1.1 * RES, 0])]:
        s1, s2 = [float(v) for v in s1], [float(v) for v in s2]
        add(requests, 'gc_ellipse', 2, *s1, *s2, 0.0, 0.0, 0.0)
        add(requests, 'gc_hyperbola', 2, *s1, *s2, 0.0, 0.0, 0.0)
        add(requests, 'gc_ellipse2d', 3, *s1[0:2], *s2[0:2], 0.0, 0.0)
        add(requests, 'gc_hyperbola2d', 3, *s1[0:2], *s2[0:2], 0.0, 0.0)
    for _ in range(4):
        add(requests, 'gc_ellipse', 2, *point(rng), *point(rng), *point(rng))
        add(requests, 'gc_hyperbola', 2, *point(rng), *point(rng), *point(rng))
        add(requests, 'gc_ellipse2d', 3, *point2d(rng), *point2d(rng), *point2d(rng))
        add(requests, 'gc_hyperbola2d', 3, *point2d(rng), *point2d(rng), *point2d(rng))


def parabola2d_requests(rng, requests):
    for _ in range(3):
        ax, f2 = [*point2d(rng), *unit2d(rng)], frame2d(rng)
        for focal in [1.5, 0.0, -1.0]:
            for sense in (0, 1):
                add(requests, 'gc_parabola2d', 1, *ax, focal, sense)
            add(requests, 'gc_parabola2d', 2, *f2, focal)
        add(requests, 'gc_parabola2d', 0, *f2, 0.5)
        for sense in (0, 1):
            add(requests, 'gc_parabola2d', 3, *ax, *point2d(rng), sense)
            add(requests, 'gc_parabola2d', 3, *ax, *ax[0:2], sense)
        add(requests, 'gc_parabola2d', 4, *point2d(rng), *ax[0:2])
        add(requests, 'gc_parabola2d', 4, *ax[0:2], *ax[0:2])
    for focus, sense in [([1.0, 3.0], 1), ([-1.0, 3.0], 0), ([0.0, 3.0], 0), ([0.0, 3.0], 1)]:
        add(requests, 'gc_parabola2d', 3, 0.0, 3.0, 0.0, 1.0, *focus, sense)


def line_requests(rng, requests):
    for _ in range(3):
        a = axis(rng)
        for kind in (0, 1, 2):
            add(requests, 'gc_line', kind, *a)
        add(requests, 'gc_line', 4, *a, *point(rng))
        a2 = [*point2d(rng), *unit2d(rng)]
        for kind in (0, 1, 2):
            add(requests, 'gc_line2d', kind, *a2)
        add(requests, 'gc_line2d', 4, *a2, *point2d(rng))
        for dist in [1.5, 0.0, -2.0]:
            add(requests, 'gc_line2d', 5, *a2, dist)
    for delta in TINY + [1.0]:
        p = point(rng)
        add(requests, 'gc_line', 3, *p, *near(p, delta))
        add(requests, 'gc_line2d', 3, *p[0:2], *near(p[0:2], delta))
    segment_requests(rng, requests)


def segment_requests(rng, requests):
    for delta in TINY + [1.0, -2.5]:
        p = point(rng)
        add(requests, 'gc_segment', 0, *p, *near(p, delta))
        add(requests, 'gc_segment2d', 1, *p[0:2], *near(p[0:2], delta))
        add(requests, 'gc_segment2d', 0, *p[0:2], 1.0, 0.0, *near(p[0:2], delta))
    for _ in range(3):
        a, a2 = axis(rng), [*point2d(rng), *unit2d(rng)]
        add(requests, 'gc_segment', 1, *a, *point(rng), 2.5)
        add(requests, 'gc_segment', 2, *a, *point(rng), *point(rng))
        p = point(rng)
        add(requests, 'gc_segment', 2, *a, *p, *p)
        add(requests, 'gc_segment2d', 0, *a2[0:2], *a2[2:4], *point2d(rng))
        add(requests, 'gc_segment2d', 2, *a2, *point2d(rng), -1.5)
        add(requests, 'gc_segment2d', 3, *a2, *point2d(rng), *point2d(rng))
        for u, v in [(0.0, 3.0), (3.0, -1.0), (1.0, 1.0)]:
            add(requests, 'gc_segment', 3, *a, u, v)
            add(requests, 'gc_segment2d', 4, *a2, u, v)


def append_gc_curve_requests(requests: list[str]) -> None:
    """Append the GC curve requests in a fixed order."""
    rng = random.Random(0x6C0)
    arc_requests(rng, requests)
    arc_through_requests(rng, requests)
    circle_requests(rng, requests)
    conic_requests(rng, requests)
    parabola2d_requests(rng, requests)
    line_requests(rng, requests)
    __import__('tkgeombase_corpus_gc_surfaces').append_gc_surface_requests(requests)


__all__ = ['append_gc_curve_requests', 'AXES', 'normalized', 'circ_spec']
