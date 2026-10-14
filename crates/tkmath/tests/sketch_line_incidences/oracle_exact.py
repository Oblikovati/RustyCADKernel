# SPDX-License-Identifier: GPL-2.0-only
"""Exact rational algebra and conic contacts for the incidence oracle.

Conic contacts use closed-form quadratic roots with exact Fraction
discriminants; a root is compared with a rational by exact sign reasoning on
(-b +/- sqrt(D)) / (2a). Polynomial helpers implement an exact Fraction Sturm
sequence. No floating arithmetic decides a count, multiplicity or position.
"""
from decimal import Decimal, getcontext
from fractions import Fraction as F
import math
import struct

getcontext().prec = 320


def hexbits(value):
    return format(struct.unpack('<Q', struct.pack('<d', value))[0], '016x')


def ulp_step(value, steps):
    for _ in range(abs(steps)):
        value = math.nextafter(value, math.inf if steps > 0 else -math.inf)
    return value


def sign(value):
    return (value > 0) - (value < 0)


# ---------- exact quadratic roots a t^2 + b t + c, a > 0 ----------
class QuadraticRoot:
    """(-b + s sqrt(D)) / (2a) with s in {-1, 0, +1}; exact comparisons."""

    def __init__(self, a, b, discriminant, branch):
        self.a, self.b, self.d, self.branch = a, b, discriminant, branch

    def compare(self, value):
        # sign(root - value) = sign(s sqrt(D) - (2 a value + b)), since 2a > 0.
        rest = 2 * self.a * value + self.b
        if self.branch == 0 or self.d == 0:
            return sign(-rest)
        if sign(rest) != self.branch:
            return self.branch
        # Same sign: compare squares; a negative branch reverses the order.
        return self.branch * sign(self.d - rest * rest)

    def decimal(self):
        root = Decimal(self.d.numerator).sqrt() / Decimal(self.d.denominator).sqrt()
        return (Decimal(-self.b.numerator) / Decimal(self.b.denominator)
                + self.branch * root) / (2 * Decimal(self.a.numerator) / Decimal(self.a.denominator))

    def exact(self):
        num, den = self.d.numerator, self.d.denominator
        rn, rd = math.isqrt(num), math.isqrt(den)
        if rn * rn == num and rd * rd == den:
            return (-self.b + self.branch * F(rn, rd)) / (2 * self.a)
        return None


def quadratic_roots(a, b, c):
    discriminant = b * b - 4 * a * c
    if discriminant < 0:
        return []
    if discriminant == 0:
        return [(QuadraticRoot(a, b, discriminant, 0), 2)]
    return [(QuadraticRoot(a, b, discriminant, -1), 1), (QuadraticRoot(a, b, discriminant, 1), 1)]


def position(compare_start, compare_end):
    if compare_start < 0:
        return 'B'
    if compare_start == 0:
        return 'S'
    if compare_end < 0:
        return 'I'
    return 'E' if compare_end == 0 else 'A'


def bracket(value):
    """Binary64 values lo <= value <= hi, adjacent or equal, as hex."""
    if isinstance(value, F):
        exact = value
    else:
        exact = F(value)
    nearest = float(exact)
    if F(nearest) == exact:
        return hexbits(nearest) + ',' + hexbits(nearest)
    if F(nearest) < exact:
        return hexbits(nearest) + ',' + hexbits(math.nextafter(nearest, math.inf))
    return hexbits(math.nextafter(nearest, -math.inf)) + ',' + hexbits(nearest)


def conic_contacts(source, coefficients, sector, center):
    a_point, b_point = [tuple(map(F, p)) for p in source]
    direction = (b_point[0] - a_point[0], b_point[1] - a_point[1])
    offset = (a_point[0] - F(center[0]), a_point[1] - F(center[1]))
    result = []
    for root, multiplicity in quadratic_roots(*coefficients):
        target = sector_position(root, sector, offset, direction)
        if target is None:
            continue
        line = position(root.compare(F(0)), root.compare(F(1)))
        exact = root.exact()
        coordinate = []
        for axis in range(2):
            if exact is not None:
                coordinate.append(bracket(a_point[axis] + exact * direction[axis]))
            else:
                value = Decimal(a_point[axis].numerator) / Decimal(a_point[axis].denominator)
                step = Decimal(direction[axis].numerator) / Decimal(direction[axis].denominator)
                coordinate.append(bracket(F(value + root.decimal() * step)))
        result.append((root, multiplicity, line, target, coordinate))
    return result


def linear_sign_at(root, constant, slope):
    if slope == 0:
        return sign(constant)
    return sign(slope) * root.compare(-constant / slope)


def sector_position(root, sector, offset, direction):
    if sector is None:
        return 'I'
    start, end, ccw = sector
    first, second, labels = (start, end, 'SE') if ccw else (end, start, 'ES')
    cross = lambda v: linear_sign_at(root, F(v[0]) * offset[1] - F(v[1]) * offset[0],
                                     F(v[0]) * direction[1] - F(v[1]) * direction[0])
    dot = lambda v: linear_sign_at(root, F(v[0]) * offset[0] + F(v[1]) * offset[1],
                                   F(v[0]) * direction[0] + F(v[1]) * direction[1])
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


def circle_coefficients(source, center, radius):
    a_point, b_point = [tuple(map(F, p)) for p in source]
    d = (b_point[0] - a_point[0], b_point[1] - a_point[1])
    q = (a_point[0] - F(center[0]), a_point[1] - F(center[1]))
    return (d[0] ** 2 + d[1] ** 2, 2 * (q[0] * d[0] + q[1] * d[1]),
            q[0] ** 2 + q[1] ** 2 - F(radius) ** 2)


def ellipse_coefficients(source, center, axis, radii):
    a_point, b_point = [tuple(map(F, p)) for p in source]
    d = (b_point[0] - a_point[0], b_point[1] - a_point[1])
    q = (a_point[0] - F(center[0]), a_point[1] - F(center[1]))
    ax, ay = F(axis[0]), F(axis[1])
    norm = ax * ax + ay * ay
    major, minor = F(radii[0]) ** 2, F(radii[1]) ** 2
    # In the normalized frame: (q.a)^2 / (|a|^2 ra^2) + (a x q)^2 / (|a|^2 rb^2) = 1.
    along = (q[0] * ax + q[1] * ay, d[0] * ax + d[1] * ay)
    across = (ax * q[1] - ay * q[0], ax * d[1] - ay * d[0])
    a = along[1] ** 2 / major + across[1] ** 2 / minor
    b = 2 * (along[0] * along[1] / major + across[0] * across[1] / minor)
    c = along[0] ** 2 / major + across[0] ** 2 / minor - norm
    return (a, b, c)


# ---------- exact polynomial helpers (ascending Fraction coefficients) ----------
def trim(poly):
    poly = list(poly)
    while poly and poly[-1] == 0:
        poly.pop()
    return poly


def evaluate(poly, x):
    total = F(0)
    for coefficient in reversed(poly):
        total = total * x + coefficient
    return total


def derivative(poly):
    return trim([i * c for i, c in enumerate(poly)][1:])


def remainder(dividend, divisor):
    dividend = trim(dividend)
    while dividend and len(dividend) >= len(divisor):
        factor = dividend[-1] / divisor[-1]
        shift = len(dividend) - len(divisor)
        for i, c in enumerate(divisor):
            dividend[i + shift] -= factor * c
        dividend = trim(dividend)
    return dividend


def gcd(first, second):
    first, second = trim(first), trim(second)
    while second:
        first, second = second, remainder(first, second)
    return first


def sturm(poly):
    chain = [trim(poly), derivative(poly)]
    while chain[-1]:
        chain.append([-c for c in remainder(chain[-2], chain[-1])])
    return [member for member in chain if member]


def variations(chain, x):
    signs = [sign(evaluate(member, x)) for member in chain]
    signs = [value for value in signs if value != 0]
    return sum(1 for left, right in zip(signs, signs[1:]) if left != right)


def roots_in(poly, lower, upper):
    """Distinct roots in [lower, upper], as exact points or isolating open intervals."""
    chain = sturm(poly)
    found = []
    if evaluate(poly, lower) == 0:
        found.append((lower, lower))
    pending = [(lower, upper)]
    while pending:
        low, high = pending.pop()
        count = variations(chain, low) - variations(chain, high)
        if count == 0:
            continue
        if evaluate(poly, high) == 0:
            if count == 1:
                found.append((high, high))
                continue
        elif count == 1 and evaluate(poly, low) != 0:
            found.append((low, high))
            continue
        middle = (low + high) / 2
        pending.extend([(middle, high), (low, middle)])
    return sorted(found, key=lambda pair: pair[0])


def refine(poly, interval, width):
    low, high = interval
    while low != high and high - low > width:
        middle = (low + high) / 2
        value = evaluate(poly, middle)
        if value == 0:
            return middle, middle
        if sign(value) == sign(evaluate(poly, low)):
            low = middle
        else:
            high = middle
    return low, high


def multiplicity(poly, interval):
    count, current = 1, trim(poly)
    while True:
        # Roots of gcd(G, G') are exactly the roots of G of multiplicity >= 2.
        current = gcd(current, derivative(current))
        if len(current) < 2 or not any(True for _ in roots_in(current, *interval)):
            return count
        count += 1


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
