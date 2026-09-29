"""Deterministic requests for the classic Extrema point-curve classes
(tools/oracle/tkgeombase_probe_extrema_pc_classic.cxx).

Every 3D and 2D curve kind of the ExtremaPC corpora (conics in random frames,
degenerate conics, planar and spatial Bezier and B-spline curves, rational, periodic,
trimmed and offset curves) runs through Extrema_ExtPC(2d) on the whole adaptor range
and on sub-ranges (inside, crossing a period, reversed, single-point), from the
special points of each curve (centres, foci, axis points, points on the curve) and
random points; Extrema_LocateExtPC(2d) from start parameters inside, at and outside
the range; the EPC, PCF and LocEPC instantiations of the templates directly
(math_FunctionRoots and math_FunctionRoot with several samplings and tolerances,
Value/Derivative/Values/GetStateNumber at chosen parameters, including cusps where the
first derivative vanishes); Extrema_GCurveLocator with 0 to 30 samples; and the tool
members that are not plain forwards.
"""
from __future__ import annotations
import math
import random
from tkg3d_corpus import point
from tkgeombase_corpus_gce import add
import tkgeombase_corpus_extrema_pc as pc3
import tkgeombase_corpus_extrema_pc2d as pc2

CUSP3 = [5, 0, 4, 0.0, 0.0, 0.0, 1, 1.0, 1.0, 0.0, 1, 0.0, 1.0, 0.0, 1, 1.0, 0.0, 0.0, 1]
CUSP2 = [5, 0, 4, 0.0, 0.0, 1, 1.0, 1.0, 1, 0.0, 1.0, 1, 1.0, 0.0, 1]


def sub_range(kind):
    """A sub-range of the curve parameters for an adaptor (kind = spec[0])."""
    if kind == 5:
        return [0.2, 0.7]
    if kind <= 4:
        return [0.3, 2.1]
    return None


def adaptor_specs(spec):
    """The adaptor encodings of one curve: the whole range and a sub-range."""
    out = [[*spec, 0]]
    rng_ = sub_range(spec[0])
    if rng_ is not None:
        out.append([*spec, 1, *rng_])
    return out


def reuse_ranges(kind):
    """(s1, s2) ranges for the reused Initialize."""
    if kind in (1, 2):
        return [(1.0, 1.0 + 2.0 * math.pi), (5.0, 5.5), (0.5, 0.5)]
    if kind == 5:
        return [(0.1, 0.9), (0.4, 0.4)]
    if kind == 6:
        return [(0.5, 1.5), (0.0, 3.0)]
    return [(-2.0, 2.0), (0.2, 0.1)]


def pc_family(requests, op, specs, pts_of, rng):
    """ExtPC requests over (spec, special points) pairs."""
    for spec, special in specs:
        pts = special + [pts_of(rng) for _ in range(2)]
        for adaptor in adaptor_specs(spec):
            for (s1, s2) in reuse_ranges(spec[0]):
                add(requests, op, *adaptor, 1.0e-10, s1, s2, len(pts), *[c for p in pts for c in p])


def loc_family(requests, op, specs, pts_of, rng):
    """LocateExtPC requests: start parameters inside, at and outside the range."""
    for spec, special in specs:
        pts = special[:2] + [pts_of(rng)]
        starts = [0.0, 0.35, 1.2, -3.0, 50.0]
        queries = [[*p, u0] for p in pts for u0 in starts]
        for adaptor in adaptor_specs(spec):
            s1, s2 = reuse_ranges(spec[0])[0]
            add(requests, op, *adaptor, 1.0e-10, s1, s2, len(queries), *[c for q in queries for c in q])


def gen_family(requests, op, specs, pts_of, rng):
    """The template instantiations with several samplings and tolerances."""
    for index, (spec, special) in enumerate(specs):
        pts = special[:2] + [pts_of(rng)]
        nb, tolu, tolf = [(12, 1.0e-9, 1.0e-10), (3, 1.0e-6, 1.0e-6), (40, 0.0, 0.0)][index % 3]
        s1, s2 = reuse_ranges(spec[0])[0]
        add(requests, op, *adaptor_specs(spec)[-1], nb, s1, s2, tolu, tolf, len(pts), *[c for p in pts for c in p])


def pcf_family(requests, op, specs, pts_of, rng):
    """The GFuncExtPC function at chosen parameters."""
    for spec, special in specs:
        p = (special + [pts_of(rng)])[0]
        us = [0.0, 0.25, 0.5, 0.999, 1.0, 1.7, -0.5]
        for adaptor in adaptor_specs(spec):
            add(requests, op, *adaptor, *p, len(us), *us)


def cloc_family(requests, op, specs, pts_of, rng):
    """Extrema_GCurveLocator with 0 to 30 samples."""
    for spec, special in specs:
        p = (special + [pts_of(rng)])[0]
        queries = [[*p, nb, a, b] for nb, a, b in ((0, 0.0, 1.0), (1, 0.0, 1.0), (2, 0.2, 0.8), (7, 0.9, 0.1),
                                                     (30, -5.0, 50.0), (11, 0.3, 0.3))]
        add(requests, op, *adaptor_specs(spec)[0], len(queries), *[c for q in queries for c in q])


def specs3d(rng):
    """The 3D curves of the ExtremaPC corpus plus a cusp Bezier."""
    return pc3.conic_curves(rng) + pc3.spline_curves(rng) + [(CUSP3, [[0.5, 0.75, 0.0], [0.5, 0.5, 1.0]])]


def specs2d(rng):
    """The 2D curves of the ExtremaPC2d corpus (with 2D special points) plus a cusp."""
    out = []
    for spec, _ in pc2.conic_curves(rng) + pc2.spline_curves(rng):
        out.append((spec, [[0.0, 0.0], [1.0, 0.5]]))
    return out + [(CUSP2, [[0.5, 0.75], [0.5, 0.0]])]


def point2(rng):
    """A random 2D point."""
    return point(rng, 5.0)[0:2]


def append_extrema_pc_classic_requests(requests: list[str]) -> None:
    """Append the classic point-curve requests (seed 20260930)."""
    rng = random.Random(20260930)
    s3, s2 = specs3d(rng), specs2d(rng)
    p3 = lambda r: point(r, 5.0)  # noqa: E731
    pc_family(requests, 'ex_pc', s3, p3, rng)
    pc_family(requests, 'ex_pc2d', s2, point2, rng)
    loc_family(requests, 'ex_locpc', s3, p3, rng)
    loc_family(requests, 'ex_locpc2d', s2, point2, rng)
    gen_family(requests, 'ex_genpc', s3, p3, rng)
    gen_family(requests, 'ex_genpc2d', s2, point2, rng)
    pcf_family(requests, 'ex_pcf', s3, p3, rng)
    pcf_family(requests, 'ex_pcf2d', s2, point2, rng)
    cloc_family(requests, 'ex_cloc', s3, p3, rng)
    cloc_family(requests, 'ex_cloc2d', s2, point2, rng)
    for spec, _ in s3:
        for adaptor in adaptor_specs(spec):
            add(requests, 'ex_ctool', *adaptor)
    for spec, _ in s2:
        for adaptor in adaptor_specs(spec):
            add(requests, 'ex_ctool2d', *adaptor)
