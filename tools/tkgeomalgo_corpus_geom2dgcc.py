"""Deterministic requests for Geom2dGcc (tools/oracle/tkgeomalgo_probe_geom2dgcc.cxx).

An adapted curve is `curve u0 u1` (bounded to [u0, u1] when u0 < u1, see ga_hatch_adaptor),
a qualified curve `position curve u0 u1`, a GccEnt argument `position x y dx dy [radius]`
and a point `x y`. The requests run every constructor overload of every Geom2dGcc class on
lines, circles (whole and trimmed: the adaptor reports both as circles), ellipses,
parabolas, Bezier, B-spline (C0 knots, periodic, rational), offset and trimmed curves, with
each qualifier (the rejected GccEnt_noqualifier included), tangent, secant, nested and far
arguments, start parameters on and off the solutions, and the consultations from index 0
to NbSolutions + 1. This module holds the qualified curve, the curve tool, the tangency
functions and the line solvers; tkgeomalgo_corpus_geom2dgcc_circles.py holds the circle
solvers and says which consultations their `detail` masks clear.
"""
from __future__ import annotations
import math
from tkgeomalgo_corpus_intcurve import fmt
from tkgeomalgo_corpus_geom2d import (BEZIER, KINKED, PERIODIC, RATIONAL, WAVE, c_bezier,
                                      c_bspline, c_circle, c_ellipse, c_line, c_offset,
                                      c_parabola, c_trimmed)

TWO_PI = 2.0 * math.pi
TOL = 1e-7

# (curve, u0, u1, lo, hi): u0 < u1 bounds the adaptor, (0, 0) keeps the curve's own range;
# [lo, hi] is where the start parameters are spread.
LINE_X = (c_line(-5.0, -1.0, 1.0, 0.0), -10.0, 10.0, -10.0, 10.0)
LINE_INF = (c_line(0.0, 2.0, 1.0, 0.2), 0.0, 0.0, -5.0, 5.0)
CIRCLE = (c_circle(0.0, 0.0, 2.0), 0.0, 0.0, 0.0, TWO_PI)
CIRCLE_FAR = (c_circle(8.0, 1.0, 1.5), 0.0, 0.0, 0.0, TWO_PI)
ARC = (c_trimmed(0.5, 2.5, c_circle(0.0, 0.0, 2.0)), 0.0, 0.0, 0.5, 2.5)
ELLIPSE = (c_ellipse(0.0, 0.0, 3.0, 1.5), 0.0, 0.0, 0.0, TWO_PI)
PARABOLA = (c_parabola(0.0, -3.0, 0.5), -4.0, 4.0, -4.0, 4.0)
BEZ = (BEZIER, 0.0, 0.0, 0.0, 1.0)
BUMP = (c_bezier([(-4.0, 3.0), (0.0, 1.0), (4.0, 3.0)]), 0.0, 0.0, 0.0, 1.0)
WAVE_C = (WAVE, 0.0, 0.0, 0.0, 3.0)
KINK = (KINKED, 0.0, 0.0, 0.0, 3.0)
PER = (PERIODIC, 0.0, 0.0, 0.0, 4.0)
RAT = (RATIONAL, 0.0, 0.0, 0.0, 1.0)
OFFSET = (c_offset(0.3, WAVE), 0.0, 0.0, 0.0, 3.0)
TRIM = (c_trimmed(0.2, 2.8, WAVE), 0.0, 0.0, 0.2, 2.8)
SEG = (c_bspline(1, [0.0, 1.0], [2, 2], [(-10.0, -1.0), (10.0, -1.0)]), 0.0, 0.0, 0.0, 1.0)
SEG_UP = (c_bspline(1, [0.0, 1.0], [2, 2], [(-10.0, 3.0), (10.0, 3.0)]), 0.0, 0.0, 0.0, 1.0)
PART = (WAVE, 0.5, 2.5, 0.5, 2.5)
CURVES = [BEZ, BUMP, WAVE_C, KINK, PER, RAT, OFFSET, TRIM, ELLIPSE, PARABOLA, SEG, PART]
ANALYTIC = [LINE_X, CIRCLE, ARC]


def ad(c: tuple) -> list:
    """An adapted curve: the curve then its bounds."""
    return [*c[0], c[1], c[2]]


def q(pos: int, c: tuple) -> list:
    """A qualified curve."""
    return [pos, *ad(c)]


def circ(x: float, y: float, r: float) -> list:
    """`x y dx dy r`: a circle as ga_read_circ2d reads it."""
    return [x, y, 1.0, 0.0, r]


def lin(x: float, y: float, dx: float, dy: float) -> list:
    """`x y dx dy`: a line as ga_read_lin2d reads it."""
    return [x, y, dx, dy]


def qcirc(pos: int, x: float, y: float, r: float) -> list:
    """A qualified circle as ga_read_qcirc reads it."""
    return [pos, *circ(x, y, r)]


def qlin(pos: int, x: float, y: float, dx: float, dy: float) -> list:
    """A qualified line as ga_read_qlin reads it."""
    return [pos, *lin(x, y, dx, dy)]


def req(op: str, *values) -> str:
    """One request line; list arguments are flattened."""
    flat: list = []
    for v in values:
        flat.extend(v if isinstance(v, list) else [v])
    return op + ' ' + fmt(flat)


def params(c: tuple, n: int = 3) -> list[float]:
    """n parameters spread over [lo, hi] of a curve."""
    lo, hi = c[3], c[4]
    return [lo + (k + 0.5) * (hi - lo) / n for k in range(n)]


def append_basic_requests(requests: list[str]) -> None:
    """The qualified curve, the curve tool and the scalar tangency functions."""
    for c in CURVES + ANALYTIC + [LINE_INF, CIRCLE_FAR]:
        for pos in range(5):
            requests.append(req('g2gcc_qualified', pos, ad(c)))
        us = params(c, 3) + [c[3], c[4]]
        requests.append(req('g2gcc_tool', ad(c), 1e-7, len(us), us))
    for c in CURVES + [LINE_X, CIRCLE]:
        us = params(c, 4)
        requests.append(req('g2gcc_func1', 0, ad(c), 0.6, 0.8, len(us), us))
        requests.append(req('g2gcc_func1', 1, ad(c), 5.0, 4.0, len(us), us))
        requests.append(req('g2gcc_func1', 1, ad(c), 0.0, 0.0, len(us), us))
        requests.append(req('g2gcc_func1', 2, ad(c), circ(0.5, 4.0, 1.0), len(us), us))


def append_set_function_requests(requests: list[str]) -> None:
    """FunctionTanCuCu, FunctionTanCuCuCu and FunctionTanCuCuOnCu at a few points."""
    for a, b in [(BEZ, WAVE_C), (CIRCLE, BUMP), (OFFSET, RAT), (SEG, SEG_UP), (KINK, PER)]:
        xs = [v for u, w in zip(params(a, 2), params(b, 2)) for v in (u, w)]
        requests.append(req('g2gcc_func2', 0, ad(a), ad(b), 2, 2, xs))
        ub = params(b, 2)
        requests.append(req('g2gcc_func2', 1, circ(0.0, 5.0, 1.0), ad(b), 2, 2, [0.3, ub[0], 2.0, ub[1]]))
    firsts = {0: ad(BEZ), 1: circ(0.0, 0.0, 1.0), 2: circ(0.0, 0.0, 1.0), 3: circ(0.0, 0.0, 1.0),
              4: lin(0.0, -4.0, 1.0, 0.0), 5: lin(0.0, -4.0, 1.0, 0.0)}
    seconds = {0: ad(RAT), 1: ad(RAT), 2: circ(5.0, 0.0, 2.0), 3: lin(-6.0, 0.0, 0.0, 1.0),
               4: lin(-6.0, 0.0, 0.0, 1.0), 5: ad(RAT)}
    for c3 in (WAVE_C, BUMP, OFFSET):
        u3 = params(c3, 2)
        for kind in range(6):
            xs = [0.2, 0.4, u3[0], 1.1, 0.7, u3[1]]
            requests.append(req('g2gcc_func3', kind, firsts[kind], seconds[kind], ad(c3), 2, 3, xs))
    for kind in range(12):
        if kind == 1:
            continue  # the circle-curve-on-circle constructor dereferences a null curve natively
        first = kind % 4
        a1 = {0: ad(BEZ), 1: circ(0.0, 0.0, 1.0), 2: lin(0.0, -4.0, 1.0, 0.0), 3: ad(BEZ)}[first]
        a2 = [3.0, 3.0] if first == 3 else ad(WAVE_C)
        locus = {0: circ(1.0, 1.0, 4.0), 1: lin(0.0, 0.0, 0.0, 1.0), 2: ad(BUMP)}[kind // 4]
        xs = [0.3, 1.2, 0.5, 2.0, 0.8, 2.5, 1.0, 1.5]
        for rad in (1.0, 0.5):
            requests.append(req('g2gcc_func4', kind, a1, a2, locus, rad, 2, 4, xs))


def append_lin2tan_requests(requests: list[str]) -> None:
    """Geom2dGcc_Lin2d2Tan: the qualifiers of two circles only when both are unqualified."""
    op = 'g2gcc_lin2tan'
    for a, b in [(CIRCLE, CIRCLE_FAR), (CIRCLE_FAR, ARC)]:
        for p1 in range(4):
            for p2 in range(4):
                detail = 7 if (p1, p2) == (0, 0) else 6
                requests.append(req(op, 0, detail, q(p1, a), q(p2, b), TOL))
                requests.append(req(op, 2, detail, q(p1, a), q(p2, b), TOL, 0.5, 0.5))
    pairs = [(BEZ, CIRCLE_FAR), (BUMP, WAVE_C), (ELLIPSE, CIRCLE_FAR), (RAT, OFFSET), (PARABOLA, BUMP),
             (BUMP, LINE_X), (CIRCLE_FAR, PER), (KINK, TRIM), (PART, CIRCLE_FAR)]
    for a, b in pairs:
        for p1, p2 in ((0, 0), (3, 3), (1, 0), (2, 3), (0, 4)):
            requests.append(req(op, 0, 7, q(p1, a), q(p2, b), 1e-4))
        for u1, u2 in zip(params(a, 3), params(b, 3)):
            requests.append(req(op, 2, 7, q(0, a), q(0, b), 1e-4, u1, u2))
    for c in CURVES + ANALYTIC:
        for pos in range(5):
            for pnt in ((5.0, 4.0), (0.0, 0.0), (-6.0, 1.0)):
                requests.append(req(op, 1, 7, q(pos, c), *pnt, 1e-4))
        for u in params(c, 3):
            requests.append(req(op, 3, 7, q(0, c), 5.0, 4.0, 1e-4, u))


def append_lin2tan_iter_requests(requests: list[str]) -> None:
    """Geom2dGcc_Lin2d2TanIter on every curve (the curve-curve form of two lines raises)."""
    op = 'g2gcc_lin2tan_iter'
    for c in CURVES:
        for pos in range(4):
            for u in params(c, 2):
                requests.append(req(op, 0, q(pos, c), 5.0, 4.0, u, 1e-4))
                requests.append(req(op, 1, qcirc(pos, 0.0, 6.0, 1.0), q(0, c), u, 1e-4))
                requests.append(req(op, 2, q(pos, c), q(0, CIRCLE_FAR), u, 0.3, 1e-4))
    requests.append(req(op, 2, q(0, SEG), q(0, SEG_UP), 0.5, 0.5, 1e-4))
    requests.append(req(op, 2, q(0, LINE_X), q(0, SEG_UP), 0.0, 0.5, 1e-4))
    requests.append(req(op, 0, q(0, BUMP), 0.0, 1.0, 0.5, 1e-4))


def append_lintanobl_requests(requests: list[str]) -> None:
    """Geom2dGcc_Lin2dTanObl (both overloads) and Geom2dGcc_Lin2dTanOblIter."""
    op = 'g2gcc_lintanobl'
    for c in CURVES + ANALYTIC:
        for pos in range(5):
            for angle in (0.7, math.pi / 2, -math.pi / 2, 2.0):
                requests.append(req(op, 0, 7, q(pos, c), lin(0.0, 0.0, 1.0, 0.0), 1e-6, angle))
        for angle in (0.0, math.pi):
            requests.append(req(op, 0, 7, q(0, c), lin(0.0, 0.0, 1.0, 0.0), 1e-6, angle))
        for u in params(c, 2):
            requests.append(req(op, 1, 7, q(0, c), lin(0.0, 0.0, 1.0, 0.0), 1e-6, u, 0.7))
            requests.append(req(op, 1, 7, q(0, c), lin(0.0, 0.0, 1.0, 0.0), u, 1e-6, 0.7))
    op = 'g2gcc_lintanobl_iter'
    for c in CURVES:
        for pos in range(5):
            for u in params(c, 2):
                for angle in (0.7, 0.0, math.pi / 2):
                    requests.append(req(op, q(pos, c), lin(0.0, 0.0, 1.0, 0.0), u, 1e-6, angle))


def append_geom2dgcc_requests(requests: list[str]) -> None:
    """Every Geom2dGcc request family in a fixed order."""
    append_basic_requests(requests)
    append_set_function_requests(requests)
    append_lin2tan_requests(requests)
    append_lin2tan_iter_requests(requests)
    append_lintanobl_requests(requests)
    __import__('tkgeomalgo_corpus_geom2dgcc_circles').append_circle_requests(requests)
