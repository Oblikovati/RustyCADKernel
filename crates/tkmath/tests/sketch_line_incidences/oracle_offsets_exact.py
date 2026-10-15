# SPDX-License-Identifier: GPL-2.0-only
"""Exact offset contacts for the offset oracle (ADR 0080), independent of the kernel.

An offset point is X = C + d N with N = (V_y, -V_x) / s, s = sqrt(G), G = |V|^2,
V = (x' w - x w', y' w - y w'). A curve equation composed with X is a + b s for
exact Fraction polynomials a, b. Its roots are roots of the norm a^2 - b^2 G,
isolated by exact Sturm sequences; a norm root is a contact exactly when a and b
vanish together or have opposite signs there (exact signs at the root). The
multiplicity splits off g = gcd(a, b): a + b s = g (a1 + b1 s) with coprime a1, b1
(the kernel uses derivative orders instead). Keys and coordinates are 320-digit
Decimal values at a 2^-300 refinement; the generator refuses orders within 1e-200.
"""
from decimal import Decimal
from fractions import Fraction as F
from oracle_curves_exact import WIDTH, add, cauchy, mul, quotient, refine, scale, sign_at
from oracle_exact import derivative, evaluate, gcd, multiplicity, roots_in, sign, trim


# ---------- surds a + b sqrt(g) with polynomial parts ----------
def surd_add(first, second):
    return (add(first[0], second[0]), add(first[1], second[1]))


def surd_scale(value, factor):
    return (scale(value[0], factor), scale(value[1], factor))


def surd_mul(first, second, g):
    rational = add(mul(first[0], second[0]), mul(mul(first[1], second[1]), g))
    return (rational, add(mul(first[0], second[1]), mul(first[1], second[0])))


def norm(value, g):
    return add(mul(value[0], value[0]), scale(mul(mul(value[1], value[1]), g), F(-1)))


def surd_sign_ratio(value, g, u):
    """Exact sign of a(u) + b(u) sqrt(g(u)) at a rational u with g(u) > 0."""
    a, b = evaluate(value[0], u), evaluate(value[1], u)
    if b == 0 or a == 0 or sign(a) == sign(b):
        return sign(a) if a != 0 else sign(b)
    return sign(a) * sign(a * a - b * b * evaluate(g, u))


def surd_sign_root(value, g, root):
    """Exact sign at the isolated root {equation, interval}."""
    equation, interval = root['equation'], root['interval']
    a = sign_at(equation, interval, value[0]) if value[0] else 0
    b = sign_at(equation, interval, value[1]) if value[1] else 0
    if b == 0 or a == 0 or a == b:
        return a if a != 0 else b
    return a * sign_at(equation, interval, norm(value, g))


def surd_decimal(value, g, u):
    root = Decimal(evaluate(g, u).numerator).sqrt() / Decimal(evaluate(g, u).denominator).sqrt()
    return to_decimal(evaluate(value[0], u)) + to_decimal(evaluate(value[1], u)) * root


def to_decimal(value):
    return Decimal(value.numerator) / Decimal(value.denominator)


# ---------- offset pieces ----------
class OffsetPiece:
    """The offset of one basis piece: surd coordinates (x, y) / w of X."""

    def __init__(self, piece, distance):
        x, y, w = piece['x'], piece['y'], piece['w']
        dw = derivative(w)
        vx = add(mul(derivative(x), w), scale(mul(x, dw), F(-1)))
        vy = add(mul(derivative(y), w), scale(mul(y, dw), F(-1)))
        self.g = add(mul(vx, vx), mul(vy, vy))
        shift = scale(w, F(distance))
        self.x = (mul(shift, vy), x)
        self.y = (scale(mul(shift, vx), F(-1)), y)
        self.w = ([], w)
        self.basis, self.range, self.index = piece, piece['range'], piece['index']

    def relative(self, center):
        return (surd_add(self.x, surd_scale(self.w, -F(center[0]))),
                surd_add(self.y, surd_scale(self.w, -F(center[1]))))

    def point(self, u):
        weight = surd_decimal(self.w, self.g, u)
        return [surd_decimal(c, self.g, u) / weight for c in (self.x, self.y)]


def compose_line(line, piece):
    """det(d, X - origin) and d.(X - origin), both scaled by W = w sqrt(G) > 0."""
    (ox, oy), (dx, dy) = line['origin'], line['direction']
    qx, qy = piece.relative((ox, oy))
    det = surd_add(surd_scale(qy, dx), surd_scale(qx, -dy))
    return det, surd_add(surd_scale(qx, dx), surd_scale(qy, dy))


def compose_conic(conic, piece):
    qx, qy = piece.relative(conic.center)
    xx, xy, yy = conic.form
    g = piece.g
    value = surd_add(surd_scale(surd_mul(qx, qx, g), xx), surd_scale(surd_mul(qx, qy, g), 2 * xy))
    value = surd_add(value, surd_scale(surd_mul(qy, qy, g), yy))
    return surd_add(value, surd_scale(surd_mul(piece.w, piece.w, g), -conic.constant))


def compose_offset_line(offset_line, piece):
    """The squared offset-line equation and its branch filter sign(det) = -sign(d)."""
    line, distance = offset_line
    det, _ = compose_line(line, piece)
    dx, dy = line['direction']
    constant = F(distance) ** 2 * (dx * dx + dy * dy)
    value = surd_add(surd_mul(det, det, piece.g), surd_scale(surd_mul(piece.w, piece.w, piece.g),
                                                             -constant))
    return value, [(det, -sign(distance))]


# ---------- roots ----------
def surd_roots(value, g, bounds, filters):
    """Contacts of a + b sqrt(g) on [lower, upper] (None: the whole line)."""
    a, b = value
    if not trim(list(b)):
        return rational_roots(a, g, bounds, filters)
    equation = norm(value, g)
    if not equation:
        raise ValueError('generated surd with a square radicand')
    lower, upper = bounds if bounds else cauchy(equation)
    found = roots_in(square_free(equation), F(lower), F(upper))
    roots = []
    for index, interval in enumerate(found):
        root = {'equation': equation, 'interval': interval}
        if surd_sign_root(value, g, root) != 0 or not passes(filters, g, root):
            continue
        root['order'] = order(value, g, root)
        root['signs'] = side_signs(value, g, found, index, (F(lower), F(upper)), bool(bounds))
        roots.append(root)
    return roots


def rational_roots(a, g, bounds, filters):
    lower, upper = bounds if bounds else cauchy(a)
    found = roots_in(square_free(a), F(lower), F(upper))
    roots = []
    for index, interval in enumerate(found):
        root = {'equation': a, 'interval': interval}
        if not passes(filters, g, root):
            continue
        root['order'] = multiplicity(a, interval)
        root['signs'] = side_signs((a, []), g, found, index, (F(lower), F(upper)), bool(bounds))
        roots.append(root)
    return roots


def square_free(poly):
    """The square-free part: Sturm counts at a multiple root need simple roots."""
    common = gcd(poly, derivative(poly))
    return quotient(poly, common) if len(common) > 1 else poly


def passes(filters, g, root):
    return all(surd_sign_root(value, g, root) == wanted for value, wanted in filters)


def order(value, g, root):
    """Multiplicity of a + b s: the norm's when b != 0, else split off gcd(a, b)."""
    a, b = value
    equation, interval = root['equation'], root['interval']
    if sign_at(equation, interval, b) != 0:
        return multiplicity(equation, interval)
    common = gcd(a, b)
    own = multiplicity(common, interval) if len(common) > 1 and roots_in(common, *interval) else 0
    reduced = (quotient(a, common), quotient(b, common))
    rest = norm(reduced, g)
    inner = {'equation': rest, 'interval': interval}
    if not roots_in(rest, *interval) or surd_sign_root(reduced, g, inner) != 0:
        return own
    return own + multiplicity(rest, interval)


def side_signs(value, g, found, index, bounds, bounded):
    """Exact signs just left and right, at rational points between neighbouring roots;
    None beyond a bounded piece end."""
    low, high = found[index]
    if low == high:
        before = found[index - 1][1] if index else bounds[0]
        after = found[index + 1][0] if index + 1 < len(found) else bounds[1]
        points = [(before + low) / 2, (low + after) / 2]
    else:
        points = [low, high]
    ends = [bounded and low == high == bounds[side] for side in (0, 1)]
    return [None if ends[side] else surd_sign_ratio(value, g, points[side]) for side in (0, 1)]


def refined(root):
    low, high = refine(root['equation'], root['interval'], WIDTH)
    return (low + high) / 2 if low != high else low, low == high


def merge(roots, cyclic):
    """Join a root at a piece end with the root at the next piece start (and, around
    a closed chain of pieces, the last end with the first start)."""
    merged = []
    for root in roots:
        previous = merged[-1] if merged else None
        if previous and root['signs'][0] is None and previous['signs'][1] is None:
            previous['signs'][1] = root['signs'][1]
            previous['right'] = root.get('right', root['order'])
            continue
        merged.append(root)
    if cyclic and len(merged) >= 2 and merged[0]['signs'][0] is None and merged[-1]['signs'][1] is None:
        last = merged.pop()
        merged[0]['signs'][0] = last['signs'][0]
        merged[0]['left'] = last.get('left', last['order'])
    return merged


def counts(root):
    left = root.get('left', root['order'])
    right = root.get('right', root['order'])
    return ['-' if root['signs'][0] is None else str(left),
            '-' if root['signs'][1] is None else str(right)]


def transition(root, missing):
    left, right = root['signs']
    if left is None or right is None:
        return missing
    return 'T' if left == right else 'C'
