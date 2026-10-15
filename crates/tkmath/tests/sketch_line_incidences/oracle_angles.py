# SPDX-License-Identifier: GPL-2.0-only
"""High-precision angle arithmetic for angle-bounded arcs in the curve oracle.

A binary64 bound `theta` is an exact rational angle. Its frame direction
`ra cos(theta) a + o rb sin(theta) a_perp` is transcendental for `theta != 0`, so a
contact (an algebraic direction) is never on it: membership and sweep order are
decided with 320-digit Decimal cosines, sines and arctangents, and the generator
refuses any decision closer than 1e-200. A zero bound is the exact axis ray.
"""
from decimal import ROUND_FLOOR, Decimal, getcontext
from fractions import Fraction as F

getcontext().prec = 320
TINY = Decimal(10) ** -200


def decimal(value):
    value = F(value)
    return Decimal(value.numerator) / Decimal(value.denominator)


def cos_sin(theta):
    x = decimal(theta)
    term, cosine, sine, order = Decimal(1), Decimal(0), Decimal(0), 0
    while True:
        if order % 2 == 0:
            cosine += term if (order // 2) % 2 == 0 else -term
        else:
            sine += term if (order // 2) % 2 == 0 else -term
        order += 1
        term = term * x / order
        if order > 4 and abs(term) < Decimal(10) ** -330:
            return cosine, sine


def pi():
    """Machin: 16 atan(1/5) - 4 atan(1/239)."""
    return 16 * arctan_inverse(5) - 4 * arctan_inverse(239)


def arctan_inverse(n):
    total, power, index = Decimal(0), Decimal(n), 0
    while True:
        term = 1 / ((2 * index + 1) * power)
        if term < Decimal(10) ** -330:
            return total
        total += term if index % 2 == 0 else -term
        power *= n * n
        index += 1


def arctan(x):
    """Arctangent by halving the argument, then the Taylor series."""
    halvings = 0
    while abs(x) > Decimal('0.1'):
        x = x / (1 + (1 + x * x).sqrt())
        halvings += 1
    total, power, index = Decimal(0), x, 0
    while True:
        term = power / (2 * index + 1)
        if abs(term) < Decimal(10) ** -330:
            return total * (2 ** halvings)
        total += term if index % 2 == 0 else -term
        power *= x * x
        index += 1


def frame_angle(direction, axis, ccw, radii):
    """Frame angle in (-pi, pi] of a Decimal direction: atan2 of normalized coordinates."""
    ax, ay = decimal(axis[0]), decimal(axis[1])
    along = (direction[0] * ax + direction[1] * ay) / decimal(radii[0])
    across = (ax * direction[1] - ay * direction[0]) / decimal(radii[1])
    if not ccw:
        across = -across
    if abs(along) < TINY and abs(across) < TINY:
        raise ValueError('zero direction')
    half = pi()
    if abs(along) >= abs(across):
        angle = arctan(across / along)
        if along < 0:
            angle += half if across >= 0 else -half
        return angle
    angle = half / 2 - arctan(along / across)
    return angle if across > 0 else angle - half


def in_bounds(angle, bounds):
    """Position of a frame angle in [start, end] modulo full turns: S/I/E or None."""
    start, end = decimal(bounds[0]), decimal(bounds[1])
    turn = 2 * pi()
    offset = modulo(angle - start, turn)
    span = end - start
    for value in (offset, offset - span, turn - offset):
        if abs(value) < TINY:
            raise ValueError('unresolved angle bound')
    return 'I' if offset < span else None


def sweep_offset(angle, start):
    """Sweep coordinate from the start angle, in [0, 2 pi)."""
    return modulo(angle - decimal(start), 2 * pi())


def modulo(value, turn):
    """Floor remainder in [0, turn); Decimal % keeps the dividend's sign."""
    return value - turn * (value / turn).to_integral_value(rounding=ROUND_FLOOR)
