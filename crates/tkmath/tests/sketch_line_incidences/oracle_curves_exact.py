# SPDX-License-Identifier: GPL-2.0-only
"""Exact contacts of conic and spline sources for the curve-source oracle.

Every curve is either parametric (a line or a B-spline span rebuilt from exact
de Boor points) or implicit (a line or a conic). A contact is a root of one
exact Fraction polynomial, isolated by a Sturm sequence; signs at a root are
exact (oracle_exact.sign_at). Conic pairs eliminate y between the two Cartesian
conic equations, a different elimination from the kernel's sweep chart.
Spline pairs implicitize the target span by a symbolic Sylvester resultant.
Only the order of distinct contacts along a conic and the coordinate brackets
use 2^-300 refinements; the generator rejects any order decided within 1e-200.
"""
from decimal import Decimal
from fractions import Fraction as F
from oracle_exact import (bracket, derivative, evaluate, gcd, multiplicity, roots_in, sign,
                          trim)
from oracle_exact import refine as refine_simple
from oracle_angles import frame_angle, in_bounds
from oracle_spline import span_polynomials

WIDTH = F(1, 2 ** 300)


# ---------- univariate Fraction polynomials (ascending) ----------
def add(first, second):
    size = max(len(first), len(second))
    return trim([(first[i] if i < len(first) else F(0)) + (second[i] if i < len(second) else F(0))
                 for i in range(size)])


def scale(poly, factor):
    return trim([factor * c for c in poly])


def mul(first, second):
    if not first or not second:
        return []
    product = [F(0)] * (len(first) + len(second) - 1)
    for i, a in enumerate(first):
        for j, b in enumerate(second):
            product[i + j] += a * b
    return trim(product)


def const(value):
    return trim([F(value)])


def quotient(dividend, divisor):
    dividend, result = trim(dividend), [F(0)] * max(1, len(dividend) - len(divisor) + 1)
    while dividend and len(dividend) >= len(divisor):
        factor = dividend[-1] / divisor[-1]
        shift = len(dividend) - len(divisor)
        result[shift] = factor
        for i, c in enumerate(divisor):
            dividend[i + shift] -= factor * c
        dividend = trim(dividend)
    return trim(result)


def refine(poly, interval, width):
    """Bisect on the square-free part, whose simple roots change sign."""
    common = gcd(poly, derivative(poly))
    simple = quotient(poly, common) if len(common) > 1 else poly
    return refine_simple(simple, interval, width)


def sign_at(poly, interval, query):
    """Exact sign of `query` at the unique root of `poly` in the closed interval."""
    common = gcd(poly, query)
    if len(common) >= 2 and roots_in(common, *interval):
        return 0
    low, high = interval
    while True:
        if low == high:
            return sign(evaluate(query, low))
        if not roots_in(query, low, high):
            return sign(evaluate(query, (low + high) / 2))
        low, high = refine(poly, (low, high), (high - low) / 2)


# ---------- curves ----------
class Conic:
    """Q(p - c) = k with Q = [[xx, xy], [xy, yy]] positive definite."""

    def __init__(self, center, form, constant, sector):
        self.center, self.form, self.constant, self.sector = center, form, constant, sector

    @staticmethod
    def circle(center, radius, sector=None):
        return Conic(tuple(map(F, center)), (F(1), F(0), F(1)), F(radius) ** 2, sector)

    @staticmethod
    def ellipse(center, axis, radii, sector=None):
        ax, ay = F(axis[0]), F(axis[1])
        major, minor = F(radii[0]) ** 2, F(radii[1]) ** 2
        form = (minor * ax * ax + major * ay * ay, (minor - major) * ax * ay,
                minor * ay * ay + major * ax * ax)
        return Conic(tuple(map(F, center)), form, major * minor * (ax * ax + ay * ay), sector)

    def compose(self, x, y, w):
        """w^2 F(x/w, y/w) for polynomial x, y, w."""
        qx = add(x, scale(w, -self.center[0]))
        qy = add(y, scale(w, -self.center[1]))
        xx, xy, yy = self.form
        value = add(add(scale(mul(qx, qx), xx), scale(mul(qx, qy), 2 * xy)), scale(mul(qy, qy), yy))
        return add(value, scale(mul(w, w), -self.constant))

    def cartesian(self):
        """F(x, y) as {(i, j): coefficient} of x^i y^j."""
        cx, cy = self.center
        xx, xy, yy = self.form
        terms = {(2, 0): xx, (1, 1): 2 * xy, (0, 2): yy,
                 (1, 0): -2 * xx * cx - 2 * xy * cy, (0, 1): -2 * yy * cy - 2 * xy * cx,
                 (0, 0): xx * cx * cx + 2 * xy * cx * cy + yy * cy * cy - self.constant}
        return terms


def line_pieces(points, bounded):
    (x0, y0), (x1, y1) = [tuple(map(F, p)) for p in points]
    piece = {'x': trim([x0, x1 - x0]), 'y': trim([y0, y1 - y0]), 'w': [F(1)],
             'range': (F(0), F(1)) if bounded else None, 'index': 0}
    return [piece]


def spline_pieces(degree, poles, knots, bounds):
    pieces = []
    for span in range(degree, len(poles)):
        lower = max(F(knots[span]), F(bounds[0]))
        upper = min(F(knots[span + 1]), F(bounds[1]))
        if lower >= upper:
            continue
        x, y = span_polynomials(degree, poles, knots, span, lower, upper)
        pieces.append({'x': trim(x), 'y': trim(y), 'w': [F(1)], 'range': (lower, upper),
                       'index': len(pieces)})
    return pieces


def line_implicit(points):
    (x0, y0), (x1, y1) = [tuple(map(F, p)) for p in points]
    return {'origin': (x0, y0), 'direction': (x1 - x0, y1 - y0)}


def line_compose(line, piece):
    (ox, oy), (dx, dy) = line['origin'], line['direction']
    qx = add(piece['x'], scale(piece['w'], -ox))
    qy = add(piece['y'], scale(piece['w'], -oy))
    return add(scale(qy, dx), scale(qx, -dy)), add(scale(qx, dx), scale(qy, dy))


# ---------- roots on pieces ----------
def piece_roots(equation, piece):
    if not equation:
        raise ValueError('generated piece lies on the other curve')
    lower, upper = piece['range'] if piece['range'] else cauchy(equation)
    roots = []
    for interval in roots_in(equation, lower, upper):
        count = multiplicity(equation, interval)
        exact = interval[0] == interval[1]
        sides = [None if piece['range'] and exact and interval[0] == piece['range'][0] else count,
                 None if piece['range'] and exact and interval[0] == piece['range'][1] else count]
        roots.append({'equation': equation, 'interval': interval, 'sides': sides,
                      'piece': piece})
    return roots


def cauchy(poly):
    bound = 1 + max(abs(c / poly[-1]) for c in poly[:-1]) if len(poly) > 1 else F(1)
    return -bound, bound


def merge(roots):
    merged = []
    for root in roots:
        previous = merged[-1] if merged else None
        if (previous and root['sides'][0] is None and previous['sides'][1] is None
                and previous['interval'] == root['interval']):
            previous['sides'][1] = root['sides'][1]
            continue
        merged.append(root)
    return merged


def side_signs(root):
    """Exact equation signs just left and right of the root (None beyond a piece end)."""
    equation, (low, high) = root['equation'], root['interval']
    signs = []
    for index, side in enumerate(root['sides']):
        if side is None:
            signs.append(None)
            continue
        signs.append(one_side(equation, low, high, index))
    return signs


def one_side(equation, low, high, index):
    if low != high:
        return sign(evaluate(equation, high if index else low))
    current, order = equation, 0
    while current:
        value = sign(evaluate(current, low))
        if value:
            return value if index or order % 2 == 0 else -value
        current, order = derivative(current), order + 1
    return 0


def transition(signs, missing):
    if signs[0] is None or signs[1] is None:
        return missing
    return 'T' if signs[0] == signs[1] else 'C'


def point_at(piece, root):
    low, high = refine(root['equation'], root['interval'], WIDTH)
    value = low if low == high else (low + high) / 2
    w = evaluate(piece['w'], value)
    return [evaluate(piece['x'], value) / w, evaluate(piece['y'], value) / w], low == high


def coordinates(point):
    return [bracket(value) for value in point]


def position(root, lower, upper):
    """Exact position of the isolated root relative to two rationals."""
    equation, interval = root['equation'], root['interval']
    first = sign_at(equation, interval, [-F(lower), F(1)])
    if first < 0:
        return 'B'
    if first == 0:
        return 'S'
    second = sign_at(equation, interval, [-F(upper), F(1)])
    return 'I' if second < 0 else ('E' if second == 0 else 'A')


# ---------- exact conic sector membership ----------
class AngleSector:
    """Parameter-angle bounds of a conic frame (axis, sense, radii)."""

    def __init__(self, axis, ccw, bounds, radii):
        self.axis, self.ccw, self.bounds, self.radii = axis, ccw, bounds, radii


def direction_at(conic, root, x, y, w):
    """Decimal contact direction from the center at a 2^-300 refinement."""
    low, high = refine(root['equation'], root['interval'], WIDTH)
    value = low if low == high else (low + high) / 2
    weight = evaluate(w, value)
    return tuple(decimal(evaluate(poly, value) / weight - conic.center[axis])
                 for axis, poly in enumerate((x, y)))


def on_axis_ray(conic, root, x, y, w, axis):
    qx = add(x, scale(w, -conic.center[0]))
    qy = add(y, scale(w, -conic.center[1]))
    across = add(scale(qy, F(axis[0])), scale(qx, -F(axis[1])))
    along = add(scale(qx, F(axis[0])), scale(qy, F(axis[1])))
    return (sign_at(root['equation'], root['interval'], across) == 0
            and sign_at(root['equation'], root['interval'], along) > 0)


def angle_position(conic, root, x, y, w):
    """S/E only at an exact zero bound on the axis ray; otherwise decided numerically."""
    sector = conic.sector
    if on_axis_ray(conic, root, x, y, w, sector.axis):
        for index, label in ((0, 'S'), (1, 'E')):
            if sector.bounds[index] == 0:
                return label
    direction = direction_at(conic, root, x, y, w)
    angle = frame_angle(direction, sector.axis, sector.ccw, sector.radii)
    return in_bounds(angle, sector.bounds)


def sector_position(conic, root, x, y, w):
    if conic.sector is None:
        return 'I'
    if isinstance(conic.sector, AngleSector):
        return angle_position(conic, root, x, y, w)
    start, end, ccw = conic.sector
    qx = add(x, scale(w, -conic.center[0]))
    qy = add(y, scale(w, -conic.center[1]))
    cross = lambda v: sign_at(root['equation'], root['interval'],
                              add(scale(qy, F(v[0])), scale(qx, -F(v[1]))))
    dot = lambda v: sign_at(root['equation'], root['interval'],
                            add(scale(qx, F(v[0])), scale(qy, F(v[1]))))
    first, second, labels = (start, end, 'SE') if ccw else (end, start, 'ES')
    after_first, before_second = cross(first), -cross(second)
    if after_first == 0 and dot(first) > 0:
        return labels[0]
    if before_second == 0 and dot(second) > 0:
        return labels[1]
    turn = sign(F(first[0]) * F(second[1]) - F(first[1]) * F(second[0]))
    inside = {1: after_first > 0 and before_second > 0,
              -1: after_first > 0 or before_second > 0,
              0: after_first > 0}[turn]
    return 'I' if inside else None


def decimal(value):
    return Decimal(value.numerator) / Decimal(value.denominator)
