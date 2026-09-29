"""Deterministic requests for the GccAna package (tools/oracle/tkgeomalgo_probe_gccana.cxx).

Every solver family runs each constructor overload on hand-built configurations
(tangent, concentric, nested, secant, coincident, parallel, degenerate and no-solution
arguments, every qualifier including the rejected ones) and on seeded random arguments
with random qualifiers. Each request prints the solution count and order and, per
solution (from index 0 to NbSolutions + 1, so both out-of-range indices raise), the
circle or line and the consultations selected by its `detail` mask.

The mask is cleared for the consultations whose native slots are left uninitialized on
some path (see docs/tkgeomalgo-progress.md): the first-argument tangency of
Circ2d2TanRad(circle, line) with an enclosed or enclosing circle, the second-argument
tangency of Circ2d2TanRad with an enclosing and an unqualified circle, the tangencies
of Circ2d2TanOn(circle, point, line), the details of Circ2dTanOnRad(point, circle), and
the qualifiers of Lin2d2Tan(circle, circle) unless both circles are unqualified. The
three-line Circ2d3Tan never uses the two qualifier triples that read an unset side.
"""
from __future__ import annotations
import math
import random

CIRC_POSITIONS = (0, 1, 2, 3)
LINE_POSITIONS = (0, 2, 3)


def req(op: str, *values: float) -> str:
    """One request line."""
    return op + ' ' + ' '.join(repr(float(v)) for v in values)


def circ(x: float, y: float, r: float, angle: float = 0.0) -> tuple[float, ...]:
    """`x y dx dy r`: a circle whose X axis makes `angle` with the global X axis."""
    return (x, y, math.cos(angle), math.sin(angle), r)


def lin(x: float, y: float, dx: float, dy: float) -> tuple[float, ...]:
    """`x y dx dy`: a line (the direction need not be unit)."""
    return (x, y, dx, dy)


def qc(pos: int, c: tuple[float, ...]) -> tuple[float, ...]:
    """A qualified circle."""
    return (pos, *c)


def ql(pos: int, line: tuple[float, ...]) -> tuple[float, ...]:
    """A qualified line."""
    return (pos, *line)


TOL = 1e-7


def rand_circ(rng: random.Random) -> tuple[float, ...]:
    """A random circle of radius in [0.5, 4] near the origin."""
    return circ(rng.uniform(-5, 5), rng.uniform(-5, 5), rng.uniform(0.5, 4.0), rng.uniform(-3, 3))


def rand_lin(rng: random.Random) -> tuple[float, ...]:
    """A random line near the origin."""
    angle = rng.uniform(0.0, 2 * math.pi)
    return lin(rng.uniform(-5, 5), rng.uniform(-5, 5), math.cos(angle), math.sin(angle))


def rand_pnt(rng: random.Random) -> tuple[float, float]:
    """A random point near the origin."""
    return (rng.uniform(-6, 6), rng.uniform(-6, 6))


C0 = circ(0.0, 0.0, 2.0)
C1 = circ(5.0, 0.0, 1.0)
C2 = circ(1.0, 0.5, 0.5)
C3 = circ(0.0, 6.0, 1.5, 0.3)
CT = circ(3.0, 0.0, 1.0)       # tangent outside to C0
CN = circ(0.5, 0.0, 1.5)       # tangent inside C0
CC = circ(0.0, 0.0, 1.0)       # concentric with C0
LX = lin(0.0, -3.0, 1.0, 0.0)
LY = lin(-4.0, 0.0, 0.0, 1.0)
LD = lin(0.0, 0.0, 1.0, 1.0)
LT = lin(0.0, 2.0, 1.0, 0.0)   # tangent to C0
LP = lin(0.0, 3.0, 1.0, 0.0)   # parallel to LX
P0 = (0.0, 0.0)
P1 = (4.0, 3.0)
P2 = (-3.0, 1.0)
PC = (2.0, 0.0)                # on C0


def circ3tan(requests: list[str], rng: random.Random) -> None:
    """Circ2d3Tan: every overload, fixed then random (the tests' configurations included)."""
    op = 'gccana_circ3tan'
    full = 15
    for p in CIRC_POSITIONS + (4,):
        requests.append(req(op, 0, full, *qc(p, circ(0, 0, 50)), *qc(0, circ(20, 0, 10)), *qc(0, circ(0, 20, 10)), TOL))
    requests.append(req(op, 0, full, *qc(0, C0), *qc(0, C0), *qc(0, C1), TOL))
    requests.append(req(op, 0, full, *qc(0, C0), *qc(0, CT), *qc(0, circ(6, 0, 2)), TOL))
    for p1 in CIRC_POSITIONS:
        requests.append(req(op, 1, full, *qc(p1, C0), *qc(0, C1), *ql(0, LX), TOL))
        requests.append(req(op, 2, full, *qc(p1, circ(0, 120, 20)), *ql(0, lin(0, 0, 10, 20)), *ql(0, lin(0, 0, 10, -40)), TOL))
        requests.append(req(op, 4, full, *qc(p1, circ(0, 0, 10)), *qc(0, circ(30, 0, 20)), 10.0, 10.0, TOL))
        requests.append(req(op, 5, full, *qc(p1, C0), *ql(0, LX), *P1, TOL))
        requests.append(req(op, 7, full, *qc(p1, circ(0, 0, 10)), 20.0, 0.0, 15.0, 5.0, TOL))
    for p in LINE_POSITIONS + (1,):
        requests.append(req(op, 1, full, *qc(0, C0), *qc(0, C1), *ql(p, LX), TOL))
        requests.append(req(op, 6, full, *ql(p, lin(0, 0, 10, 20)), *ql(0, lin(0, 0, 10, -40)), 10.0, 80.0, TOL))
        requests.append(req(op, 8, full, *ql(p, lin(0, 0, 10, 20)), 10.0, 10.0, 100.0, 10.0, TOL))
    for p1 in LINE_POSITIONS:
        for p2 in LINE_POSITIONS:
            for p3 in LINE_POSITIONS:
                if (p1, p2, p3) in ((0, 2, 0), (2, 0, 0)):
                    continue  # native reads an unset side slot
                requests.append(req(op, 3, full, *ql(p1, lin(0, 0, 10, 20)), *ql(p2, lin(0, 0, 10, -40)), *ql(p3, lin(160, 0, -40, 10)), TOL))
    requests.append(req(op, 3, full, *ql(0, LX), *ql(0, LP), *ql(0, LY), TOL))
    for pts in ((P0, P1, P2), ((0, 50), (30, 20), (150, 150)), (P0, P0, P0), (P0, P0, P1), (P0, (1, 1), (2, 2)), (P1, P2, P1)):
        requests.append(req(op, 9, full, *pts[0], *pts[1], *pts[2], TOL))
    for pp in (((20, 0), (10, 0)), ((5, 0), (0, 0)), ((5, 0), (0, 10)), ((10, 0), (0, 10)), ((20, 0), (-20, 0)), ((3, 3), (3, 3))):
        requests.append(req(op, 7, full, *qc(0, circ(0, 0, 10)), *pp[0], *pp[1], TOL))
    requests.append(req(op, 4, full, *qc(0, circ(0, 0, 10)), *qc(0, circ(30, 0, 20)), 30.0, -20.0, TOL))
    requests.append(req(op, 4, full, *qc(0, circ(0, 0, 10)), *qc(0, circ(30, 0, 20)), 7.0, 30.0, TOL))
    requests.append(req(op, 8, full, *ql(0, LX), *P1, *P1, TOL))
    for _ in range(12):
        requests.append(req(op, 0, full, *qc(rng.choice(CIRC_POSITIONS), rand_circ(rng)), *qc(rng.choice(CIRC_POSITIONS), rand_circ(rng)), *qc(rng.choice(CIRC_POSITIONS), rand_circ(rng)), 1e-6))
        requests.append(req(op, 1, full, *qc(rng.choice(CIRC_POSITIONS), rand_circ(rng)), *qc(rng.choice(CIRC_POSITIONS), rand_circ(rng)), *ql(rng.choice(LINE_POSITIONS), rand_lin(rng)), 1e-6))
        requests.append(req(op, 2, full, *qc(rng.choice(CIRC_POSITIONS), rand_circ(rng)), *ql(rng.choice(LINE_POSITIONS), rand_lin(rng)), *ql(rng.choice(LINE_POSITIONS), rand_lin(rng)), 1e-6))
        requests.append(req(op, 4, full, *qc(rng.choice(CIRC_POSITIONS), rand_circ(rng)), *qc(rng.choice(CIRC_POSITIONS), rand_circ(rng)), *rand_pnt(rng), 1e-6))
        requests.append(req(op, 5, full, *qc(rng.choice(CIRC_POSITIONS), rand_circ(rng)), *ql(rng.choice(LINE_POSITIONS), rand_lin(rng)), *rand_pnt(rng), 1e-6))
        requests.append(req(op, 6, full, *ql(rng.choice(LINE_POSITIONS), rand_lin(rng)), *ql(rng.choice(LINE_POSITIONS), rand_lin(rng)), *rand_pnt(rng), 1e-6))
        requests.append(req(op, 7, full, *qc(rng.choice(CIRC_POSITIONS), rand_circ(rng)), *rand_pnt(rng), *rand_pnt(rng), 1e-6))
        requests.append(req(op, 8, full, *ql(rng.choice(LINE_POSITIONS), rand_lin(rng)), *rand_pnt(rng), *rand_pnt(rng), 1e-6))
        requests.append(req(op, 9, full, *rand_pnt(rng), *rand_pnt(rng), *rand_pnt(rng), 1e-6))


def tanrad(requests: list[str], rng: random.Random) -> None:
    """Circ2d2TanRad: every overload with tangent, nested, far and random arguments."""
    op = 'gccana_2tanrad'
    circle_pairs = ((C0, C1), (C0, CT), (C0, CN), (C0, CC), (C0, C2), (C1, C0), (C0, C0), (circ(0, 0, 3), circ(4, 0, 1)))
    for a, b in circle_pairs:
        for p1 in CIRC_POSITIONS:
            for p2 in CIRC_POSITIONS:
                detail = 3 if 1 in (p1, p2) and 0 in (p1, p2) else 7
                for radius in (0.5, 1.0, 2.0, 3.5):
                    requests.append(req(op, 0, detail, *qc(p1, a), *qc(p2, b), radius, TOL))
    for c in (C0, C2, circ(0, 5, 2)):
        for line in (LX, LT, LD, lin(0, 1, 1, 0)):
            for p1 in CIRC_POSITIONS:
                for p2 in LINE_POSITIONS + (1,):
                    detail = 5 if p1 in (1, 2) else 7
                    for radius in (0.5, 1.0, 3.0):
                        if p1 in (1, 2) and 2 * radius == c[4] and line == LT:
                            # Its tangency point is the centre of C1, whose parameter
                            # native skips or not on the unset TheSame1 slot.
                            continue
                        requests.append(req(op, 1, detail, *qc(p1, c), *ql(p2, line), radius, TOL))
    for point in (P0, P1, PC, (1.0, 0.0), (6.0, 0.0)):
        for p1 in CIRC_POSITIONS + (4,):
            for radius in (0.5, 1.0, 2.0, 3.0):
                requests.append(req(op, 2, 7, *qc(p1, C0), *point, radius, TOL))
    for point in (P1, (0.0, -3.0), (2.0, -1.0), (0.0, 5.0)):
        for p1 in LINE_POSITIONS + (1,):
            for radius in (0.5, 1.0, 4.0):
                requests.append(req(op, 3, 7, *ql(p1, LX), *point, radius, TOL))
    for a, b in ((LX, LY), (LX, LD), (LX, LP), (LD, LY)):
        for p1 in LINE_POSITIONS:
            for p2 in LINE_POSITIONS:
                requests.append(req(op, 4, 7, *ql(p1, a), *ql(p2, b), 1.0, TOL))
    for a, b, radius in ((P0, P1, 1.0), (P0, P1, 2.5), (P0, P1, 3.0), (P0, P0, 1.0), (P0, (2.0, 0.0), 1.0)):
        requests.append(req(op, 5, 7, *a, *b, radius, TOL))
    requests.append(req(op, 0, 7, *qc(0, C0), *qc(0, C1), -1.0, TOL))
    for _ in range(12):
        p1, p2 = rng.choice(CIRC_POSITIONS), rng.choice(CIRC_POSITIONS)
        detail = 3 if 1 in (p1, p2) and 0 in (p1, p2) else 7
        requests.append(req(op, 0, detail, *qc(p1, rand_circ(rng)), *qc(p2, rand_circ(rng)), rng.uniform(0.2, 4), 1e-6))
        p1 = rng.choice(CIRC_POSITIONS)
        requests.append(req(op, 1, 5 if p1 in (1, 2) else 7, *qc(p1, rand_circ(rng)), *ql(rng.choice(LINE_POSITIONS), rand_lin(rng)), rng.uniform(0.2, 4), 1e-6))
        requests.append(req(op, 2, 7, *qc(rng.choice(CIRC_POSITIONS), rand_circ(rng)), *rand_pnt(rng), rng.uniform(0.2, 4), 1e-6))
        requests.append(req(op, 3, 7, *ql(rng.choice(LINE_POSITIONS), rand_lin(rng)), *rand_pnt(rng), rng.uniform(0.2, 4), 1e-6))
        requests.append(req(op, 4, 7, *ql(rng.choice(LINE_POSITIONS), rand_lin(rng)), *ql(rng.choice(LINE_POSITIONS), rand_lin(rng)), rng.uniform(0.2, 4), 1e-6))
        requests.append(req(op, 5, 7, *rand_pnt(rng), *rand_pnt(rng), rng.uniform(0.2, 6), 1e-6))


def tanon(requests: list[str], rng: random.Random) -> None:
    """Circ2d2TanOn: the twelve overloads (six argument pairs on a line or a circle)."""
    op = 'gccana_2tanon'
    lines = (lin(0, -1, 1, 0), lin(0, 0, 0, 1), LD, lin(2.5, 0, 0, 1))
    circles = (circ(0, 0, 4), circ(2, 2, 3), circ(2.5, 0, 1))

    def detail(args: int) -> int:
        return 9 if args == 3 else 15

    def pair(args: int, p1: int, p2: int, rnd: bool) -> tuple[float, ...]:
        c_a = rand_circ(rng) if rnd else C0
        c_b = rand_circ(rng) if rnd else C1
        l_a = rand_lin(rng) if rnd else LX
        l_b = rand_lin(rng) if rnd else LY
        p_a = rand_pnt(rng) if rnd else P1
        p_b = rand_pnt(rng) if rnd else P2
        return {0: (*qc(p1, c_a), *qc(p2, c_b)), 1: (*qc(p1, c_a), *ql(p2, l_a)), 2: (*ql(p1, l_a), *ql(p2, l_b)),
                3: (*qc(p1, c_a), *p_b), 4: (*ql(p1, l_a), *p_b), 5: (*p_a, *p_b)}[args]

    for args in range(6):
        p1s = CIRC_POSITIONS if args in (0, 1, 3) else LINE_POSITIONS
        p2s = CIRC_POSITIONS if args == 0 else LINE_POSITIONS
        for p1 in p1s:
            for p2 in (p2s if args in (0, 1, 2) else (0,)):
                for locus in lines:
                    requests.append(req(op, args, detail(args), *pair(args, p1, p2, False), *locus, TOL))
                for locus in circles:
                    requests.append(req(op, 6 + args, 15, *pair(args, p1, p2, False), *locus, TOL))
    for p1 in CIRC_POSITIONS:
        for p2 in CIRC_POSITIONS:
            for locus in (lin(3.5, 0, 0, 1), lin(3.0, -2, 0.3, 1), lin(0, 3, 1, 0.2)):
                requests.append(req(op, 0, 15, *qc(p1, C0), *qc(p2, C1), *locus, TOL))
            requests.append(req(op, 6, 15, *qc(p1, C0), *qc(p2, C1), *circ(3.5, 0, 2), TOL))
    requests.append(req(op, 0, 15, *qc(0, C0), *qc(0, CC), *lin(0, 0, 1, 0), TOL))
    requests.append(req(op, 6, 15, *qc(0, C0), *qc(0, C1), *circ(0, 0, 1), TOL))
    requests.append(req(op, 5, 15, *P0, *P0, *LX, TOL))
    requests.append(req(op, 11, 15, *P0, *P0, *circles[0], TOL))
    requests.append(req(op, 4, 15, *ql(0, LX), *P1, *LX, TOL))
    for _ in range(10):
        for kind in range(12):
            args = kind % 6
            p1 = rng.choice(CIRC_POSITIONS if args in (0, 1, 3) else LINE_POSITIONS)
            p2 = rng.choice(CIRC_POSITIONS if args == 0 else LINE_POSITIONS)
            locus = rand_lin(rng) if kind < 6 else rand_circ(rng)
            requests.append(req(op, kind, detail(args) if kind < 6 else 15, *pair(args, p1, p2, True), *locus, 1e-6))


def tanonrad(requests: list[str], rng: random.Random) -> None:
    """Circ2dTanOnRad: tangent to a circle, a line or a point, centred on a line or a circle."""
    op = 'gccana_tanonrad'
    for radius in (0.5, 1.0, 2.0, 3.0, 5.0):
        for p1 in CIRC_POSITIONS:
            for locus in (LX, lin(0, 0, 1, 0), LD, lin(0, 4, 1, 0)):
                requests.append(req(op, 0, 11, *qc(p1, C0), *locus, radius, TOL))
            for locus in (circ(0, 0, 4), circ(3, 0, 2), circ(0, 0, 2), circ(7, 0, 1)):
                requests.append(req(op, 3, 11, *qc(p1, C0), *locus, radius, TOL))
        for p1 in LINE_POSITIONS:
            requests.append(req(op, 1, 11, *ql(p1, LX), *LY, radius, TOL))
            requests.append(req(op, 1, 11, *ql(p1, LX), *LP, radius, TOL))
            requests.append(req(op, 4, 11, *ql(p1, LX), *circ(0, 0, 4), radius, TOL))
        for point in (P0, P1, (0.0, -3.0), (0.0, 1.0)):
            requests.append(req(op, 2, 11, *point, *LX, radius, TOL))
            requests.append(req(op, 5, 0, *point, *circ(0, 0, 3), radius, TOL))
    requests.append(req(op, 0, 11, *qc(0, C0), *LX, -1.0, TOL))
    for _ in range(12):
        for kind in range(6):
            first = {0: qc(rng.choice(CIRC_POSITIONS), rand_circ(rng)), 1: ql(rng.choice(LINE_POSITIONS), rand_lin(rng)), 2: rand_pnt(rng)}[kind % 3]
            locus = rand_lin(rng) if kind < 3 else rand_circ(rng)
            requests.append(req(op, kind, 0 if kind == 5 else 11, *first, *locus, rng.uniform(0.2, 5), 1e-6))


def small_solvers(requests: list[str], rng: random.Random) -> None:
    """Circ2dTanCen, Lin2d2Tan, Lin2dTanObl, Lin2dBisec and the bisector classes."""
    for p1 in CIRC_POSITIONS + (4,):
        for centre in (P0, P1, PC, (1.0, 0.0)):
            requests.append(req('gccana_tancen', 0, 3, *qc(p1, C0), *centre, TOL))
    for centre in (P1, (0.0, 2.0)):
        requests.append(req('gccana_tancen', 1, 3, *LX, *centre))
        requests.append(req('gccana_tancen', 2, 3, *P2, *centre))
    for a, b in ((P0, P1), (P0, P0), (P1, P2)):
        requests.append(req('gccana_lin2tan', 0, 7, *a, *b, TOL))
    for p1 in CIRC_POSITIONS + (4,):
        for point in (P0, P1, PC, (2.0, 2.0)):
            requests.append(req('gccana_lin2tan', 1, 7, *qc(p1, C0), *point, TOL))
        for other in (C1, CT, CN, CC, C0, C2):
            for p2 in CIRC_POSITIONS:
                requests.append(req('gccana_lin2tan', 2, 7 if (p1, p2) == (0, 0) else 6, *qc(p1, C0), *qc(p2, other), TOL))
    for angle in (0.0, 0.5, math.pi / 2, math.pi, -1.0):
        requests.append(req('gccana_lintanobl', 0, 7, *P1, *LX, angle))
        for p1 in CIRC_POSITIONS + (4,):
            requests.append(req('gccana_lintanobl', 1, 7, *qc(p1, C0), *LX, angle))
    for a, b in ((LX, LY), (LX, LP), (LX, LX), (LD, LY), (LY, LD)):
        requests.append(req('gccana_linbisec', *a, *b))
    for a, b in ((C0, C1), (C0, CT), (C0, CN), (C0, CC), (C0, C2), (C0, circ(1, 0, 2)), (C0, circ(3, 0, 2)), (C0, circ(4, 0, 2)), (C0, circ(10, 0, 2)), (C1, C0)):
        requests.append(req('gccana_bisec', 0, *a, *b))
    for line in (LX, LT, LD, lin(0, -2, 1, 0)):
        requests.append(req('gccana_bisec', 1, *C0, *line))
    for point in (P0, P1, PC, (1.0, 0.0)):
        requests.append(req('gccana_bisec', 2, *C0, *point))
        requests.append(req('gccana_bisec', 3, *C0, *point, 1e-3))
        requests.append(req('gccana_bisec', 4, *LX, *point))
        requests.append(req('gccana_bisec', 5, *P1, *point))
    for _ in range(10):
        requests.append(req('gccana_tancen', 0, 3, *qc(rng.choice(CIRC_POSITIONS), rand_circ(rng)), *rand_pnt(rng), 1e-6))
        requests.append(req('gccana_lin2tan', 1, 7, *qc(rng.choice(CIRC_POSITIONS), rand_circ(rng)), *rand_pnt(rng), 1e-6))
        requests.append(req('gccana_lin2tan', 2, 7, *qc(0, rand_circ(rng)), *qc(0, rand_circ(rng)), 1e-6))
        requests.append(req('gccana_lintanobl', 1, 7, *qc(rng.choice(CIRC_POSITIONS), rand_circ(rng)), *rand_lin(rng), rng.uniform(-3, 3)))
        requests.append(req('gccana_linbisec', *rand_lin(rng), *rand_lin(rng)))
        requests.append(req('gccana_bisec', 0, *rand_circ(rng), *rand_circ(rng)))
        requests.append(req('gccana_bisec', 1, *rand_circ(rng), *rand_lin(rng)))
        requests.append(req('gccana_bisec', 2, *rand_circ(rng), *rand_pnt(rng)))


def append_gccana_requests(requests: list[str]) -> None:
    """Append the GccAna requests in a fixed order."""
    rng = random.Random(20260930)
    circ3tan(requests, rng)
    tanrad(requests, rng)
    tanon(requests, rng)
    tanonrad(requests, rng)
    small_solvers(requests, rng)
