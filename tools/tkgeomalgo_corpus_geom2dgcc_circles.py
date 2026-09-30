"""Deterministic Geom2dGcc circle solver requests (tools/oracle/tkgeomalgo_probe_geom2dgcc_circles.cxx).

The argument forms are those of tkgeomalgo_corpus_geom2dgcc.py. The `detail` masks (bit 1
qualifiers, 2 first tangency, 4 second, 8 third tangency or centre) and the skipped
configurations keep every request off the slots native leaves uninitialized
(docs/tkgeomalgo-progress.md): the second qualifier of the curve-curve
Circ2d2TanRadGeo; the GccAna Circ2d2TanRad slots of an enclosing and an unqualified circle
and of an enclosing or enclosed circle with a line, and the qualifiers of a line with a
point; the three-line Circ2d3Tan side triples tkgeomalgo_corpus_gccana.py skips; the
tangencies of Circ2d2TanOn(circle, point) on a line and its centre parameter after an
iteration; and the two-circle Circ2d2TanOnGeo, whose radius slots native fills only for an
enclosing, enclosed or outside first circle with an enclosing or unqualified second one
(its qualifiers only for an enclosing second one). Geom2dGcc_Circ2d2TanOnIter of a circle
and a curve on a circle dereferences a null curve natively, so it is left out.
"""
from __future__ import annotations
from tkgeomalgo_corpus_geom2dgcc import (ANALYTIC, ARC, BEZ, BUMP, CIRCLE, CIRCLE_FAR, CURVES, KINK,
                                         LINE_X, OFFSET, PARABOLA, PART, PER, RAT, SEG, TOL, TRIM,
                                         WAVE_C, ad, c_circle, c_line, circ, lin, params, q, qcirc,
                                         qlin, req)

LINE_Y = (c_line(4.0, 0.0, 0.0, 1.0), -10.0, 10.0, -10.0, 10.0)
LINE_TOP = (c_line(0.0, 4.0, 1.0, 0.0), -10.0, 10.0, -10.0, 10.0)
CIRCLE_SMALL = (c_circle(-3.0, 3.0, 1.0), 0.0, 0.0, 0.0, 6.283185307179586)
LINES = (LINE_X, LINE_Y, LINE_TOP)


def is_line(c: tuple) -> bool:
    return c[0][0] == 0


def is_circle(c: tuple) -> bool:
    return c[0][0] == 1 or (c[0][0] == 7 and c[0][5] == 1)


def analytic(c: tuple) -> bool:
    return is_line(c) or is_circle(c)


def append_tancen_requests(requests: list[str]) -> None:
    """Geom2dGcc_Circ2dTanCen and Circ2dTanCenGeo, the centre off, on and inside the curve."""
    for c in CURVES + ANALYTIC:
        for pos in range(5):
            for pnt in ((0.5, 4.0), (0.0, 0.0), (-6.0, -1.0)):
                requests.append(req('g2gcc_tancen', 3, q(pos, c), *pnt, TOL))
    for c in CURVES:
        for pos in range(5):
            for pnt in ((0.5, 4.0), (0.0, 0.0)):
                requests.append(req('g2gcc_tancen_geo', 3, q(pos, c), *pnt, TOL))
    requests.append(req('g2gcc_tancen', 3, q(0, CIRCLE), 0.0, 0.0, TOL))


def tanrad_detail(a: tuple, b: tuple, p1: int, p2: int) -> int | None:
    """The mask of a Circ2d2TanRad pair, None for a pair native reads unset slots of."""
    if is_circle(a) and is_circle(b) and {p1, p2} == {0, 1}:
        return None
    if (is_circle(a) and is_line(b) and p1 in (1, 2)) or (is_line(a) and is_circle(b) and p2 in (1, 2)):
        return None
    if analytic(a) or analytic(b):
        return 7
    return 6


def append_tanrad_requests(requests: list[str]) -> None:
    """Geom2dGcc_Circ2d2TanRad: two arguments, an argument and a point, two points, Results."""
    op = 'g2gcc_2tanrad'
    pairs = [(CIRCLE, CIRCLE_FAR), (CIRCLE, LINE_TOP), (LINE_X, LINE_Y), (LINE_X, CIRCLE), (BUMP, LINE_X),
             (CIRCLE, BUMP), (BUMP, WAVE_C), (PARABOLA, SEG), (RAT, OFFSET), (ARC, KINK), (LINE_TOP, PER)]
    for a, b in pairs:
        for p1 in range(4):
            for p2 in range(4):
                detail = tanrad_detail(a, b, p1, p2)
                if detail is None:
                    continue
                for radius in (0.5, 2.0):
                    requests.append(req(op, 0, detail, q(p1, a), q(p2, b), radius, 1e-6))
    for c in CURVES + ANALYTIC:
        for pos in range(5):
            for pnt in ((0.5, 4.0), (0.0, 0.0)):
                requests.append(req(op, 1, 6 if is_line(c) else 7, q(pos, c), *pnt, 1.5, 1e-6))
    for a, b, radius in (((0.0, 0.0), (2.0, 0.0), 1.0), ((0.0, 0.0), (2.0, 0.0), 3.0),
                         ((0.0, 0.0), (0.0, 0.0), 1.0), ((0.0, 0.0), (5.0, 0.0), 1.0)):
        requests.append(req(op, 2, 7, *a, *b, radius, 1e-6))
    requests.append(req(op, 0, 7, q(0, CIRCLE), q(0, BUMP), -1.0, 1e-6))
    base = [q(0, CIRCLE), q(0, CIRCLE_FAR), 2.0, 1e-6]
    for p1, p2 in ((0, 0), (3, 3), (2, 3)):
        requests.append(req('g2gcc_2tanrad', 3, 7, *base, qcirc(p1, 0.0, 0.0, 2.0), qcirc(p2, 5.0, 0.0, 1.0), 1.0, 1e-6))
        requests.append(req('g2gcc_2tanrad', 4, 7, *base, qcirc(p1, 0.0, 0.0, 1.0), q(p2, BUMP), 1.0, 1e-6))


def append_tanrad_geo_requests(requests: list[str]) -> None:
    """Geom2dGcc_Circ2d2TanRadGeo: a line, a circle, a curve or a point with a curve."""
    op = 'g2gcc_2tanrad_geo'
    for c in CURVES:
        for p1 in range(5):
            for p2 in range(5):
                requests.append(req(op, 0, 7, qlin(p1, 0.0, -4.0, 1.0, 0.0), q(p2, c), 1.0, 1e-6))
                requests.append(req(op, 1, 7, qcirc(p1, 0.0, 5.0, 1.5), q(p2, c), 1.0, 1e-6))
        for p1 in range(4):
            requests.append(req(op, 2, 6, q(p1, c), q(3 - p1, BUMP), 1.0, 1e-6))
            requests.append(req(op, 3, 7, q(p1, c), 0.5, 4.0, 1.0, 1e-6))


def append_3tan_requests(requests: list[str]) -> None:
    """Geom2dGcc_Circ2d3Tan: every overload on analytic and non-analytic arguments."""
    op = 'g2gcc_3tan'
    triples = [(CIRCLE, CIRCLE_FAR, CIRCLE_SMALL), (LINE_X, LINE_Y, LINE_TOP), (CIRCLE, LINE_Y, LINE_TOP),
               (LINE_X, CIRCLE_FAR, CIRCLE_SMALL), (CIRCLE, CIRCLE_FAR, LINE_TOP), (BUMP, CIRCLE, LINE_Y),
               (BUMP, WAVE_C, LINE_Y), (CIRCLE, BUMP, SEG), (PARABOLA, LINE_TOP, CIRCLE_SMALL),
               (OFFSET, RAT, PER)]
    for a, b, c in triples:
        us = [params(a, 2)[0], params(b, 2)[1], params(c, 2)[0]]
        for ps in ((0, 0, 0), (3, 3, 3), (0, 2, 3), (2, 3, 0), (3, 0, 2), (1, 0, 0), (0, 0, 4)):
            if all(is_line(x) for x in (a, b, c)) and (1 in ps or ps[:3] in ((0, 2, 0), (2, 0, 0))):
                continue
            requests.append(req(op, 0, 15, q(ps[0], a), q(ps[1], b), q(ps[2], c), TOL, *us))
    for a, b in ((CIRCLE, CIRCLE_FAR), (LINE_X, CIRCLE), (LINE_X, LINE_Y), (BUMP, CIRCLE), (BUMP, WAVE_C), (KINK, TRIM)):
        for ps in ((0, 0), (3, 3), (2, 0)):
            requests.append(req(op, 1, 15, q(ps[0], a), q(ps[1], b), 0.5, 4.0, TOL, params(a, 2)[0], params(b, 2)[1]))
    for c in CURVES + ANALYTIC:
        for pos in (0, 2, 3):
            requests.append(req(op, 2, 15, q(pos, c), 0.5, 4.0, -3.0, 5.0, TOL, params(c, 2)[0]))
    for pts in (((0.0, 0.0), (4.0, 3.0), (-3.0, 1.0)), ((0.0, 0.0), (1.0, 1.0), (2.0, 2.0)), ((0.0, 0.0), (0.0, 0.0), (4.0, 3.0))):
        requests.append(req(op, 3, 15, *pts[0], *pts[1], *pts[2], TOL))
    base = [q(0, CIRCLE), q(0, CIRCLE_FAR), q(0, CIRCLE_SMALL), TOL, 0.0, 0.0, 0.0]
    gccana = [qcirc(0, 0.0, 0.0, 50.0), qcirc(0, 20.0, 0.0, 10.0), qcirc(0, 0.0, 20.0, 10.0), TOL]
    for ranks in ((1, 2, 3), (3, 1, 2), (2, 3, 1)):
        requests.append(req(op, 4, 15, *base, *gccana, *ranks))


def append_3tan_iter_requests(requests: list[str]) -> None:
    """Geom2dGcc_Circ2d3TanIter: the ten overloads from start parameters near a solution."""
    op = 'g2gcc_3tan_iter'
    for curve in (BUMP, WAVE_C, OFFSET, SEG):
        u = params(curve, 2)
        for pos in (0, 3):
            requests.append(req(op, 0, q(pos, BEZ), q(0, curve), q(0, RAT), 0.3, u[0], 0.5, 1e-6))
            requests.append(req(op, 1, qcirc(pos, 0.0, 6.0, 1.0), q(0, curve), q(0, RAT), 1.0, u[0], 0.5, 1e-6))
            requests.append(req(op, 2, qcirc(pos, 0.0, 6.0, 1.0), qcirc(0, 5.0, 3.0, 1.0), q(0, curve), 4.0, 3.0, u[1], 1e-6))
            requests.append(req(op, 3, qcirc(pos, 0.0, 6.0, 1.0), qlin(0, 6.0, 0.0, 0.0, 1.0), q(0, curve), 4.0, 1.0, u[1], 1e-6))
            requests.append(req(op, 4, qcirc(pos, 0.0, 6.0, 1.0), q(0, curve), 4.0, 5.0, 4.0, u[0], 1e-6))
            requests.append(req(op, 5, qlin(pos, 0.0, 6.0, 1.0, 0.0), qlin(0, 6.0, 0.0, 0.0, 1.0), q(0, curve), 2.0, 2.0, u[1], 1e-6))
            requests.append(req(op, 6, qlin(pos, 0.0, 6.0, 1.0, 0.0), q(0, curve), q(0, RAT), 1.0, u[0], 0.5, 1e-6))
            requests.append(req(op, 7, qlin(pos, 0.0, 6.0, 1.0, 0.0), q(0, curve), 3.0, 5.0, 1.0, u[0], 1e-6))
            requests.append(req(op, 8, q(pos, curve), 3.0, 5.0, -3.0, 4.0, u[0], 1e-6))
            requests.append(req(op, 9, q(pos, curve), q(0, BEZ), 3.0, 5.0, u[1], 0.5, 1e-6))


def append_tanon_requests(requests: list[str]) -> None:
    """Geom2dGcc_Circ2d2TanOn on a line, a circle and a curve locus, and its Results."""
    op = 'g2gcc_2tanon'
    loci = (LINE_Y, CIRCLE_FAR, BUMP, PART)
    pairs = [(CIRCLE, CIRCLE_SMALL), (CIRCLE, LINE_TOP), (LINE_X, LINE_TOP), (BUMP, CIRCLE), (BEZ, WAVE_C),
             (LINE_X, KINK), (RAT, OFFSET)]
    for a, b in pairs:
        for p1, p2 in ((0, 0), (3, 3), (2, 0), (0, 1)):
            for on in loci:
                both = analytic(a) and analytic(b)
                if not both and not analytic(on) and p2 == 1:
                    continue
                detail = 15 if both else 7
                if both and not analytic(on) and is_circle(a) and is_circle(b) and not (p1 in (1, 2, 3) and p2 in (0, 1)):
                    continue
                if both and not analytic(on) and is_circle(a) and is_circle(b) and p2 == 0:
                    detail = 14
                requests.append(req(op, 0, detail, q(p1, a), q(p2, b), ad(on), TOL, params(a, 2)[0], params(b, 2)[1], params(on, 2)[0]))
    for c in CURVES + ANALYTIC:
        for pos in (0, 2, 3):
            for on in loci:
                detail = 15 if analytic(c) else 7
                if is_circle(c) and is_line(on):
                    detail = 9
                requests.append(req(op, 1, detail, q(pos, c), 0.5, 4.0, ad(on), TOL, params(c, 2)[0], params(on, 2)[1]))
    for on in loci + (LINE_X, CIRCLE):
        requests.append(req(op, 2, 15, 0.0, 0.0, 4.0, 3.0, ad(on), TOL))
    base = [q(0, CIRCLE), q(0, CIRCLE_SMALL), ad(LINE_Y), TOL, 0.0, 0.0, 0.0]
    requests.append(req(op, 3, 15, *base, qcirc(0, 0.0, 0.0, 2.0), qcirc(0, 5.0, 0.0, 1.0), lin(3.5, 0.0, 0.0, 1.0), TOL))
    requests.append(req(op, 3, 15, *base, qcirc(3, 0.0, 0.0, 2.0), qcirc(2, 5.0, 0.0, 1.0), lin(0.0, 3.0, 1.0, 0.2), TOL))
    requests.append(req(op, 4, 15, *base, qlin(0, 0.0, -3.0, 1.0, 0.0), qlin(0, -4.0, 0.0, 0.0, 1.0), ad(BUMP), TOL))
    requests.append(req(op, 4, 15, *base, qlin(2, 0.0, -3.0, 1.0, 0.0), qlin(3, -4.0, 0.0, 0.0, 1.0), ad(PART), TOL))


def append_tanon_geo_requests(requests: list[str]) -> None:
    """Geom2dGcc_Circ2d2TanOnGeo: the six argument pairs on curve loci."""
    op = 'g2gcc_2tanon_geo'
    for on in (BUMP, PART, PARABOLA, OFFSET, RAT):
        for p1 in (1, 2, 3):
            for p2 in (0, 1):
                args = [qcirc(p1, 0.0, 0.0, 1.0), qcirc(p2, 3.0, 1.0, 0.5)]
                requests.append(req(op, 0, 14 if p2 == 0 else 15, *args, ad(on), TOL))
        for p1 in range(4):
            for p2 in (0, 2, 3):
                requests.append(req(op, 1, 15, qcirc(p1, 0.0, 0.0, 1.0), qlin(p2, 0.0, -3.0, 1.0, 0.0), ad(on), TOL))
                requests.append(req(op, 3, 15, qlin(p2, 0.0, -3.0, 1.0, 0.0), qlin((0, 2, 3, 0)[p1], -4.0, 0.0, 0.0, 1.0), ad(on), TOL))
            requests.append(req(op, 2, 15, qcirc(p1, 0.0, 0.0, 1.0), 3.0, 2.0, ad(on), TOL))
            requests.append(req(op, 4, 15, qlin(p1, 0.0, -3.0, 1.0, 0.0), 3.0, 2.0, ad(on), TOL))
        requests.append(req(op, 5, 15, 0.0, 0.0, 3.0, 2.0, ad(on), TOL))
        requests.append(req(op, 5, 15, 1.0, 1.0, 1.0, 1.0, ad(on), TOL))


def append_tanon_iter_requests(requests: list[str]) -> None:
    """Geom2dGcc_Circ2d2TanOnIter on a line, a circle and a curve (no circle-curve on a circle)."""
    op = 'g2gcc_2tanon_iter'
    loci = {0: lin(4.0, 0.0, 0.0, 1.0), 1: circ(1.0, 1.0, 4.0), 2: ad(BUMP)}
    for curve in (WAVE_C, OFFSET, RAT, SEG):
        u = params(curve, 2)
        for kind in range(12):
            if kind == 4:
                continue
            first, locus = kind % 4, loci[kind // 4]
            for pos in (0, 3):
                if first == 0:
                    args = [qcirc(pos, 0.0, 6.0, 1.0), q(0, curve), 1.0, u[0], 0.5]
                elif first == 1:
                    args = [qlin(pos, 0.0, 6.0, 1.0, 0.0), q(0, curve), 1.0, u[0], 0.5]
                elif first == 2:
                    args = [q(pos, BEZ), q(0, curve), 0.3, u[1], 0.5]
                else:
                    args = [q(pos, curve), 3.0, 5.0, u[0], 0.5]
                requests.append(req(op, kind, locus, *args, 1e-6))


def append_circle_requests(requests: list[str]) -> None:
    """Every Geom2dGcc circle solver family in a fixed order."""
    append_tancen_requests(requests)
    append_tanrad_requests(requests)
    append_tanrad_geo_requests(requests)
    append_3tan_requests(requests)
    append_3tan_iter_requests(requests)
    append_tanon_requests(requests)
    append_tanon_geo_requests(requests)
    append_tanon_iter_requests(requests)
