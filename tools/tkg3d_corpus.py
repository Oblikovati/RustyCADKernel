"""Deterministic request corpus for the TKG3d (Geom) numeric oracle.

Each request is one `operation numbers...` line read by tools/oracle/tkg3d_probe.cxx
and crates/tkg3d/examples/numeric_probe.rs. Curve and surface specifications follow
the grammar documented in tools/oracle/tkg3d_probe_common.cxx (g3_curve, g3_surface).
The helpers below build those specifications; the request families live in
tkg3d_corpus_*.py modules, one per probe file.
"""
from __future__ import annotations
import math
import random
from tkg2d_corpus import fmt, params


def point(rng: random.Random, scale=10.0):
    """A random point (or vector) with coordinates in [-scale, scale]."""
    return [rng.uniform(-scale, scale) for _ in range(3)]


def unit(rng: random.Random):
    """A random unit direction, uniform on the sphere."""
    z = rng.uniform(-1.0, 1.0)
    angle = rng.uniform(-math.pi, math.pi)
    r = math.sqrt(1.0 - z * z)
    return [r * math.cos(angle), r * math.sin(angle), z]


def cross(a, b):
    return [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]


def frame(rng: random.Random, scale=5.0):
    """A frame P N X (gp_Ax2/gp_Ax3 input) with X well away from N."""
    n = unit(rng)
    while True:
        x = unit(rng)
        if sum(c * c for c in cross(n, x)) > 0.05:
            return [*point(rng, scale), *n, *x]


def axis(rng: random.Random, scale=5.0):
    """An axis P D (gp_Ax1 input)."""
    return [*point(rng, scale), *unit(rng)]


def trsf(rng: random.Random, kind=None):
    """A transformation spec of g3_trsf: kind then nine numbers."""
    kind = rng.randrange(8) if kind is None else kind
    values = [rng.uniform(-3, 3) for _ in range(9)]
    if kind in (1, 2, 3, 7):
        values[3:6] = unit(rng)
    if kind == 2:
        values[3:9] = frame(rng)[3:9]
    if kind == 4:
        values[3] = rng.choice([2.0, -1.5, 0.5, -1.0])
    if kind == 7:
        values[7] = rng.choice([2.0, -0.5, 3.0])
    return [kind, *values]


def moves(rng: random.Random):
    """The inputs of g3_moves: P, axis, frame, angle, factor, V, Q, transformation."""
    return [*point(rng), *axis(rng), *frame(rng), rng.uniform(-4, 4), rng.choice([2.0, -0.5, 1.0, 0.0]),
            *point(rng), *point(rng), *trsf(rng)]


def line(p, d):
    return [0, *p, *d]


def circle(f, radius):
    return [1, *f, radius]


def ellipse(f, major, minor):
    return [2, *f, major, minor]


def hyperbola(f, major, minor):
    return [3, *f, major, minor]


def parabola(f, focal):
    return [4, *f, focal]


def bezier(poles, rational=False):
    """Poles are [x, y, z, w] lists."""
    return [5, int(rational), len(poles), *[c for pole in poles for c in pole]]


def bspline(degree, knots, poles, periodic=False, rational=False):
    """Knots are [k, m] pairs, poles [x, y, z, w] lists."""
    flat_knots = [c for knot in knots for c in knot]
    flat_poles = [c for pole in poles for c in pole]
    return [6, int(rational), int(periodic), degree, len(knots), *flat_knots, len(poles), *flat_poles]


def trimmed(u1, u2, basis, sense=True, adjust=True):
    return [7, u1, u2, int(sense), int(adjust), *basis]


def offset(distance, direction, basis, no_check=False):
    return [8, distance, *direction, int(no_check), *basis]


def append_tkg3d_requests(requests: list[str]) -> None:
    """Append every TKG3d request family in a fixed order."""
    from tkg3d_corpus_simple import append_simple_requests
    from tkg3d_corpus_curves import append_curve_family_requests
    from tkg3d_corpus_advapprox import append_advapprox_requests
    rng = random.Random(0x7C63D)
    append_simple_requests(rng, requests)
    append_curve_family_requests(rng, requests)
    __import__('tkg3d_corpus_surfaces').append_surface_family_requests(random.Random(0x5F3A), requests)
    append_advapprox_requests(requests)
    __import__('tkg3d_corpus_bspl').append_bspl_requests(requests)
    __import__('tkg3d_corpus_offset').append_offset_requests(requests)
    __import__('tkg3d_corpus_bezier_surface').append_bezier_surface_requests(requests)
    __import__('tkg3d_corpus_bspline_surface').append_bspline_surface_requests(requests)
    __import__('tkg3d_corpus_offset_surface').append_offset_surface_requests(requests)
    # GeomEval curves (tkg3d_corpus_eval_curves.py)
    __import__('tkg3d_corpus_eval_curves').append_eval_curve_requests(requests)
    # GeomEval elementary surfaces (tkg3d_corpus_eval_surfaces.py)
    # GeomEval Bezier surfaces (tkg3d_corpus_eval_bezier_surfaces.py)
    __import__('tkg3d_corpus_eval_bezier_surfaces').append_eval_bezier_surface_requests(requests)
    # GeomHash (tkg3d_corpus_hash.py)
    # TopAbs (tkg3d_corpus_topabs.py)
    __import__('tkg3d_corpus_topabs').append_topabs_requests(requests)


__all__ = ['fmt', 'params', 'point', 'unit', 'frame', 'axis', 'trsf', 'moves', 'line', 'circle', 'ellipse',
           'hyperbola', 'parabola', 'bezier', 'bspline', 'trimmed', 'offset', 'append_tkg3d_requests']
