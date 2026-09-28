"""Deterministic requests for CPnts and GCPnts (tools/oracle/tkgeombase_probe_gcpnts.cxx).

Every curve kind of the LProp builders (tkgeombase_probe_lprop_build.cxx) is probed in
2D and 3D: lines, conics (full, arcs, reversed sense), two-pole segments (the
length-parametrized paths, rational or not), Bezier curves with a cusp, a zero-length
Bezier curve, B-splines that are C2, C1 (composite deflection), C0 and periodic,
trimmed and offset curves (a collapsing one, and offsets of a cusp, whose undefined
derivatives raise inside the Gauss integrand and the other callbacks), and circles of radius 1e-6 and 1e6. Each class runs every overload with
normal, reversed, partial and out-of-range parameter ranges, zero, negative and
beyond-the-end abscissas, tolerances from 0 to 1e3 (and -1, the native default),
too-small point counts and deflections that raise, and reuses one object across runs
to expose the state the native classes keep.
"""
from __future__ import annotations
import math
from tkgeombase_corpus_gce import add

PI = math.pi
TWO_PI = 2.0 * PI

# 2D specs of the LProp builders: a gp_Ax2d (location, direction), the sense, radii.
AX2D = [0.5, -1.0, 0.6, 0.8]
S_BEZIER = [5, 4, 0, 0, 1, 1, 3, 1, 1, -2, 2, 4, 0, 1]
CUSP = [5, 4, 0, 0, 1, 0, 0, 1, 2, 1, 1.5, 3, 0.5, 1]
C2 = [6, 3, 0, 5, 0, 0, 1, 1, 2, 1, 2, -1, 1, 3, 1, 2, 4, 0, 1, 3, 0, 4, 1, 1, 2, 4]
C1 = [6, 3, 0, 7, 0, 0, 1, 1, 2, 1, 2, -1, 1, 3, 0.5, 1, 4, 2, 2, 5, -1, 1, 6, 1, 1,
      4, 0, 4, 1, 2, 2, 1, 3, 4]
C0 = [6, 2, 0, 5, 0, 0, 1, 1, 1, 1, 2, 0, 1, 3, 1, 2, 4, 0, 1, 3, 0, 3, 1, 2, 2, 3]
HEXAGON = [coordinate for k in range(6)
           for coordinate in (2.0 * math.cos(k * PI / 3), 2.0 * math.sin(k * PI / 3), 1.0)]
PERIODIC = [6, 3, 1, 6, *HEXAGON, 7, *[v for k in range(7) for v in (float(k), 1)]]
ELLIPSE = [2, *AX2D, 1, 10.0, 3.0]
CIRCLE = [1, *AX2D, 1, 2.5]

# (name, dimension, spec, adaptor range or None, parameter domain, length scale).
CURVES = [
    ('line2', 2, [0, 1.0, 2.0, 0.6, 0.8], (-3.0, 7.0), (-3.0, 7.0), 10.0),
    ('circle2', 2, CIRCLE, None, (0.0, TWO_PI), 15.7),
    ('arc2_indirect', 2, [1, *AX2D, 0, 2.5], (1.0, 4.0), (1.0, 4.0), 7.5),
    ('ellipse2', 2, ELLIPSE, None, (0.0, TWO_PI), 44.0),
    ('hyperbola2', 2, [3, *AX2D, 1, 6.0, 3.0], (-1.5, 1.2), (-1.5, 1.2), 20.0),
    ('parabola2', 2, [4, *AX2D, 1, 2.0], (-3.0, 2.0), (-3.0, 2.0), 8.0),
    ('segment2', 2, [5, 2, 0, 0, 1, 3, 4, 1], None, (0.0, 1.0), 5.0),
    ('rational_segment2', 2, [5, 2, 0, 0, 1, 3, 4, 2], None, (0.0, 1.0), 5.0),
    ('bezier2', 2, S_BEZIER, None, (0.0, 1.0), 6.0),
    ('cusp2', 2, CUSP, None, (0.0, 1.0), 4.0),
    ('point2', 2, [5, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1], None, (0.0, 1.0), 1.0),
    ('bspline_c2', 2, C2, None, (0.0, 2.0), 10.0),
    ('bspline_c1', 2, C1, None, (0.0, 4.0), 14.0),
    ('bspline_c0', 2, C0, None, (0.0, 3.0), 10.0),
    ('bspline_linear2', 2, [6, 1, 0, 2, 0, 0, 1, 5, 1, 1, 2, 0, 2, 1, 2], None, (0.0, 1.0), 5.1),
    ('bspline_periodic2', 2, PERIODIC, None, (0.0, 6.0), 11.0),
    ('trimmed_ellipse2', 2, [7, 0.5, 4.0, *ELLIPSE], None, (0.5, 4.0), 25.0),
    ('offset_ellipse2', 2, [8, 0.5, *ELLIPSE], None, (0.0, TWO_PI), 47.0),
    ('offset_collapse2', 2, [8, -2.5, *CIRCLE], None, (0.0, TWO_PI), 1.0),
    ('offset_cusp2', 2, [8, 0.3, *CUSP], None, (0.0, 1.0), 4.0),
    ('tiny_circle2', 2, [1, *AX2D, 1, 1e-6], None, (0.0, TWO_PI), 6.3e-6),
    ('huge_circle2', 2, [1, *AX2D, 1, 1e6], (0.0, 3.0), (0.0, 3.0), 3e6),
]

FRAME = [1.0, -2.0, 0.5, 0.0, 0.6, 0.8, 1.0, 0.0, 0.0]


def to3d(spec2d):
    """The 3D twin of a Bezier or B-spline 2D spec, z = 0.3 x (as the LProp corpus)."""
    kind = spec2d[0]
    head = [kind] + (spec2d[1:3] if kind == 6 else [])
    body = spec2d[len(head):]
    n = body[0]
    poles = []
    for i in range(n):
        x, y, w = body[1 + 3 * i: 4 + 3 * i]
        poles += [x, y, 0.3 * x, w]
    return head + [n] + poles + body[1 + 3 * n:]


HELIX = [6, 3, 0, 5, 1, 0, 0, 1, 0, 1, 0.5, 1, -1, 0, 1, 1, 0, -1, 1.5, 1, 1, 0, 2, 1,
         3, 0, 4, 1, 1, 2, 4]
CURVES += [
    ('line3', 3, [0, 1.0, 2.0, 3.0, 0.0, 0.6, 0.8], (-2.0, 5.0), (-2.0, 5.0), 7.0),
    ('circle3', 3, [1, *FRAME, 2.5], None, (0.0, TWO_PI), 15.7),
    ('ellipse3', 3, [2, *FRAME, 4.0, 1.0], (0.3, 5.0), (0.3, 5.0), 16.0),
    ('helix3', 3, HELIX, None, (0.0, 2.0), 6.0),
    ('bezier3', 3, to3d(S_BEZIER), None, (0.0, 1.0), 6.5),
    ('bspline_c1_3', 3, to3d(C1), None, (0.0, 4.0), 15.0),
    ('segment3', 3, to3d([5, 2, 0, 0, 1, 3, 4, 1]), None, (0.0, 1.0), 5.2),
    ('offset_cusp3', 3, [8, 0.3, 0, 0, 1, *to3d(CUSP)], None, (0.0, 1.0), 4.2),
    ('trimmed_circle3', 3, [7, 0.5, 2.0, 1, *FRAME, 2.5], None, (0.5, 2.0), 3.8),
    ('offset_circle3', 3, [8, 0.75, 0, 0, 1, 1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 2.0], None,
     (0.0, TWO_PI), 17.0),
]


def curve_head(dim, spec, adaptor_range):
    """The request prefix read by gb_gcp_curve: dimension, range flag and bounds, spec."""
    if adaptor_range is None:
        return [dim, 0, *spec]
    return [dim, 1, *adaptor_range, *spec]


def inner(domain, t):
    a, b = domain
    return a + t * (b - a)


def length_requests(requests):
    for _, dim, spec, rng_, dom, _ in CURVES:
        head = curve_head(dim, spec, rng_)
        a, b = dom
        for u1, u2, tol in [(a, b, 1e-9), (b, a, 1e-3), (inner(dom, 0.2), inner(dom, 0.7), 0.0),
                            (a - 0.5, b + 0.5, 1e3), (inner(dom, 0.4), inner(dom, 0.4), 1e-12),
                            (inner(dom, 0.9), inner(dom, 0.1), -1.0)]:
            add(requests, 'gcp_length', *head, u1, u2, tol)


def abscissas(scale):
    return [0.3 * scale, -0.2 * scale, 0.0, 1.5 * scale, 1e-12, 0.95 * scale]


def abscissa_requests(requests):
    for _, dim, spec, rng_, dom, scale in CURVES:
        head = curve_head(dim, spec, rng_)
        a, b = dom
        for ab in abscissas(scale):
            for u0 in (a, inner(dom, 0.5), b):
                ui = u0 + 0.1 * (b - a)
                add(requests, 'cpnts_abscissa', *head, ab, u0, ui, 1e-9,
                    inner(dom, 0.1), inner(dom, 0.8), 1e-9)
                add(requests, 'gcp_abscissa', *head, ab, u0, ui, 1e-9)
        add(requests, 'cpnts_abscissa', *head, 0.4 * scale, a, b, 1e-3, b, a, 1e-2)
        add(requests, 'gcp_abscissa', *head, 0.4 * scale, a, b + 5.0, 1e-3)
        add(requests, 'gcp_abscissa', *head, -0.4 * scale, b, a - 5.0, 1e-14)


def cpnts_udefl_requests(requests):
    for _, dim, spec, rng_, dom, scale in CURVES:
        head = curve_head(dim, spec, rng_)
        a, b = dom
        for defl in (0.1 * scale, 0.01 * scale, 1e-3 * scale):
            for ctl in (1, 0):
                add(requests, 'cpnts_udefl', *head, defl, 1e-9, ctl, inner(dom, 0.8),
                    inner(dom, 0.1), 3000)
        add(requests, 'cpnts_udefl', *head, 0.05 * scale, 1e-3, 1, a, b, 5)
        add(requests, 'cpnts_udefl', *head, 10.0 * scale, 1e-9, 1, a, inner(dom, 0.01), 3000)


def uabs_runs(dom, scale):
    a, b = dom
    lo, hi = inner(dom, 0.25), inner(dom, 0.9)
    return [
        (0, scale / 7.0, 0, 0, -1.0, 0), (1, scale / 5.0, hi, lo, 1e-9, 0),
        (2, 11, 0, 0, -1.0, 0), (3, 5, lo, hi, 1e3, 0),
        (0, scale / 40.0, 0, 0, 1e-7, 0), (0, scale / 3.0, 0, 0, 1e-7, 1),
        (1, -scale / 6.0, lo, hi, -1.0, 0), (0, 0.0, 0, 0, -1.0, 0),
        (2, 1, 0, 0, -1.0, 0), (3, 2, a, a, -1.0, 0), (1, scale / 9.0, inner(dom, -0.2), inner(dom, 1.2), 0.0, 0),
        (4, scale / 6.0, 0, 0, -1.0, 0), (5, scale / 4.0, lo, hi, 1e-5, 0),
        (6, 7, 0, 0, -1.0, 0), (7, 3, hi, lo, -1.0, 0), (6, 0, 0, 0, -1.0, 0),
    ]


def uabs_requests(requests):
    for _, dim, spec, rng_, dom, scale in CURVES:
        runs = uabs_runs(dom, scale)
        add(requests, 'gcp_uabs', *curve_head(dim, spec, rng_), len(runs),
            *[v for run in runs for v in run])


def quabs_requests(requests):
    for _, dim, spec, rng_, dom, _ in CURVES:
        a, b = dom
        lo, hi = inner(dom, 0.3), inner(dom, 0.95)
        runs = [(0, 7, 0, 0), (1, 5, hi, lo), (2, 2, 0, 0), (3, 50, lo, hi), (0, 1, 0, 0),
                (1, 4, a, a), (3, 0, a, b), (2, 13, 0, 0)]
        add(requests, 'gcp_quabs', *curve_head(dim, spec, rng_), len(runs),
            *[v for run in runs for v in run])


def udefl_requests(requests):
    for _, dim, spec, rng_, dom, scale in CURVES:
        a, b = dom
        lo, hi = inner(dom, 0.15), inner(dom, 0.85)
        runs = [(0, 0.05 * scale, 1, 0, 0), (1, 0.01 * scale, 1, hi, lo), (0, 0.02 * scale, 0, 0, 0),
                (2, 0.2 * scale, 1, 0, 0), (3, 0.005 * scale, 0, lo, hi), (1, 1e3 * scale, 1, a, b),
                (1, 0.03 * scale, 1, a, a)]
        add(requests, 'gcp_udefl', *curve_head(dim, spec, rng_), len(runs),
            *[v for run in runs for v in run])


def qudefl_requests(requests):
    for _, dim, spec, rng_, dom, scale in CURVES:
        a, b = dom
        lo, hi = inner(dom, 0.15), inner(dom, 0.85)
        runs = [(0, 0.05 * scale, 2, 0, 0), (1, 0.01 * scale, 0, hi, lo), (0, 0.002 * scale, 4, 0, 0),
                (2, 0.2 * scale, 1, 0, 0), (3, 0.005 * scale, 3, lo, hi), (1, 1e3 * scale, 6, a, b),
                (1, 0.03 * scale, 2, a, a), (0, 1e-6 * scale, 0, 0, 0)]
        add(requests, 'gcp_qudefl', *curve_head(dim, spec, rng_), len(runs),
            *[v for run in runs for v in run])


def tdefl_requests(requests):
    for _, dim, spec, rng_, dom, scale in CURVES:
        a, b = dom
        lo, hi = inner(dom, 0.2), inner(dom, 0.7)
        runs = [(0, 0.5, 0.01 * scale, 2, 1e-9, 1e-7, 0, 0), (1, 0.1, 0.001 * scale, 5, 1e-9, 1e-7, hi, lo),
                (0, 0.2, 0.05 * scale, 20, 1e-3, 0.1 * scale, 0, 0), (2, 1.0, 0.1 * scale, 0, 1e-9, 1e-7, 0, 0),
                (3, 0.3, 0.002 * scale, 3, 0.1, 1e-9, lo, hi), (0, 1e-13, 0.01, 2, 1e-9, 1e-7, 0, 0),
                (1, 0.5, 1e-9, 2, 1e-9, 1e-7, a, b), (1, 0.4, 0.01 * scale, 2, 1e-9, 1e-7, a, a)]
        adds = [(0.0, 0.0, 0.0, inner(dom, 0.33), 1), (1.0, 1.0, 0.0, inner(dom, 0.33), 0),
                (2.0, 0.0, 1.0, inner(dom, 0.33) + 1e-10, 1), (3.0, 3.0, 3.0, b + 1.0, 0),
                (4.0, 4.0, 4.0, a - 1.0, 1)]
        add(requests, 'gcp_tdefl', *curve_head(dim, spec, rng_), len(runs),
            *[v for run in runs for v in run], len(adds), *[v for p in adds for v in p])


def dist_requests(requests):
    for _, dim, spec, rng_, dom, _ in CURVES:
        a, b = dom
        u1, u2 = inner(dom, 0.1), inner(dom, 0.6)
        xs = [u1, inner(dom, 0.25), inner(dom, 0.35), u2, a - 0.1, inner(dom, 0.9)]
        head = curve_head(dim, spec, rng_)
        add(requests, 'gcp_dist', *head, u1, u2, len(xs), *xs)
        add(requests, 'gcp_dist', *head, u1, u1, 2, u1, u2)
        add(requests, 'gcp_dist', *head, a, b, 1, inner(dom, 0.5))


def arc_step_requests(requests):
    for radius in (-1.0, 0.0, 1e-8, 1e-7, 1.0, 2.5, 1e6):
        for lin in (0.01, 0.5, 3.0):
            for ang in (0.1, 0.5, 4.0):
                for min_len in (0.0, 1e-7, 1e-3, 10.0):
                    add(requests, 'gcp_arc_step', radius, lin, ang, min_len)


def append_gcpnts_requests(requests: list[str]) -> None:
    """Append every CPnts and GCPnts request family in a fixed order."""
    length_requests(requests)
    abscissa_requests(requests)
    cpnts_udefl_requests(requests)
    uabs_requests(requests)
    quabs_requests(requests)
    udefl_requests(requests)
    qudefl_requests(requests)
    tdefl_requests(requests)
    dist_requests(requests)
    arc_step_requests(requests)
