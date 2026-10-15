# SPDX-License-Identifier: GPL-2.0-only
"""Contacts of conic and spline sources with every admitted target kind.

Each function returns rows (order_key, fields) where fields are
target, left, right, source_position, target_position, transition, xlo, xhi, ylo, yhi.
"""
from fractions import Fraction as F
from oracle_curves_exact import (Conic, WIDTH, add, cauchy, coordinates, line_compose, merge,
                                 mul, piece_roots, point_at, position, refine, scale,
                                 sector_position, side_signs, sign_at, transition, trim)
from oracle_angles import TINY, frame_angle, sweep_offset
from oracle_curves_exact import AngleSector, decimal
from oracle_exact import evaluate, multiplicity, roots_in, sign


def counts(root):
    return ['-' if side is None else str(side) for side in root['sides']]


# ---------- conic sources ----------
class ConicSource:
    """A conic with its sweep frame: reference ray, orientation and end ray."""

    def __init__(self, conic, axis):
        self.conic = conic
        self.angles = isinstance(conic.sector, AngleSector)
        if conic.sector is None:
            self.reference, self.ccw, self.end = tuple(map(F, axis)), True, None
        elif self.angles:
            sector = conic.sector
            self.reference, self.ccw, self.end = tuple(map(F, sector.axis)), sector.ccw, None
        else:
            start, end, ccw = conic.sector
            self.reference, self.ccw, self.end = tuple(map(F, start)), ccw, tuple(map(F, end))

    def chart(self, root, x, y, w):
        """Exact chart 0..3 of the contact direction (reference, upper, opposite, lower)."""
        qx = add(x, scale(w, -self.conic.center[0]))
        qy = add(y, scale(w, -self.conic.center[1]))
        rx, ry = self.reference
        across = add(scale(qy, rx), scale(qx, -ry))
        if not self.ccw:
            across = scale(across, F(-1))
        side = sign_at(root['equation'], root['interval'], across)
        if side > 0:
            return 1
        if side < 0:
            return 3
        along = sign_at(root['equation'], root['interval'], add(scale(qx, rx), scale(qy, ry)))
        return 0 if along > 0 else 2

    def position(self, root, x, y, w):
        if self.conic.sector is None:
            return 'I'
        found = sector_position(self.conic, root, x, y, w)
        return 'A' if found is None else found

    def key(self, chart, point):
        """Sweep order key: chart, then the oriented direction (compared by cross products).

        Angle-bounded arcs order by the sweep offset from their start angle instead.
        """
        direction = (point[0] - self.conic.center[0], point[1] - self.conic.center[1])
        if self.angles:
            sector = self.conic.sector
            exact = tuple(decimal(value) for value in direction)
            angle = frame_angle(exact, sector.axis, sector.ccw, sector.radii)
            return ('offset', sweep_offset(angle, sector.bounds[0]), self.ccw)
        return (chart, direction, self.ccw)


def sweep_less(first, second):
    if first[0] == 'offset':
        if abs(first[1] - second[1]) < TINY:
            raise ValueError('unresolved sweep order')
        return first[1] < second[1]
    if first[0] != second[0]:
        return first[0] < second[0]
    if first[0] in (0, 2):
        raise ValueError('unresolved tie on a reference ray')
    (ax, ay), (bx, by) = first[1], second[1]
    cross = ax * by - ay * bx
    if not first[2]:
        cross = -cross
    size = (abs(ax) + abs(ay)) * (abs(bx) + abs(by))
    if abs(cross) <= size * F(1, 10 ** 200):
        raise ValueError('unresolved sweep order')
    return cross > 0


def conic_parametric(source, pieces, bounds, index):
    """Conic source against line or spline pieces: contacts in the target parameter."""
    roots = []
    for piece in pieces:
        roots.extend(piece_roots(source.conic.compose(piece['x'], piece['y'], piece['w']), piece))
    rows = []
    for root in merge(roots):
        piece = root['piece']
        x, y, w = piece['x'], piece['y'], piece['w']
        target = 'I' if bounds is None else position(root, *bounds)
        point, _ = point_at(piece, root)
        chart = source.chart(root, x, y, w)
        fields = [index] + counts(root) + [source.position(root, x, y, w), target,
                                           transition(side_signs(root), 'E')]
        rows.append((source.key(chart, point), fields + coordinates(point)))
    return rows


def cartesian_quadratic(conic, swap):
    """F as a v^2 + b(w) v + c(w), eliminating v = y (or v = x when swapped)."""
    t = conic.cartesian()
    if swap:
        t = {(j, i): value for (i, j), value in t.items()}
    return (t[(0, 2)], trim([t[(0, 1)], t[(1, 1)]]), trim([t[(0, 0)], t[(1, 0)], t[(2, 0)]]))


def conic_conic(source, target, index):
    """Eliminate y between the Cartesian equations (x when two contacts share x)."""
    try:
        return conic_conic_in(source, target, index, False)
    except SharedAbscissa:
        return conic_conic_in(source, target, index, True)


class SharedAbscissa(ValueError):
    pass


def conic_conic_in(source, target, index, swap):
    a1, b1, c1 = cartesian_quadratic(source.conic, swap)
    a2, b2, c2 = cartesian_quadratic(target, swap)
    numerator = add(scale(c2, a1), scale(c1, -a2))
    denominator = add(scale(b2, a1), scale(b1, -a2))
    third = add(mul(b1, c2), scale(mul(b2, c1), F(-1)))
    resultant = add(mul(numerator, numerator), scale(mul(denominator, third), F(-1)))
    if not resultant:
        raise ValueError('generated conics coincide')
    rows = []
    for interval in roots_in(resultant, *cauchy(resultant)):
        if sign_at(resultant, interval, denominator) == 0:
            raise SharedAbscissa('two contacts share one abscissa')
        count = multiplicity(resultant, interval)
        root = {'equation': resultant, 'interval': interval, 'sides': [count, count], 'piece': None}
        # The other coordinate is -numerator / denominator; weight denominator^2 > 0.
        own = mul([F(0), F(1)], mul(denominator, denominator))
        other = scale(mul(numerator, denominator), F(-1))
        w = mul(denominator, denominator)
        x, y = (other, own) if swap else (own, other)
        found = sector_position(target, root, x, y, w)
        if found is None:
            continue
        low, high = refine(resultant, interval, WIDTH)
        value = low if low == high else (low + high) / 2
        paired = -evaluate(numerator, value) / evaluate(denominator, value)
        point = [paired, value] if swap else [value, paired]
        chart = source.chart(root, x, y, w)
        kind = 'T' if count % 2 == 0 else 'C'
        fields = [index, str(count), str(count), source.position(root, x, y, w), found,
                  kind if found == 'I' else 'E']
        rows.append((source.key(chart, point), fields + coordinates(point)))
    return rows


# ---------- spline sources ----------
def spline_implicit(source_pieces, bounds, compose, classify, index):
    """Spline source pieces against an implicit target: contacts in the source parameter.

    Rows carry the 2^-300 point as a third element for spline-pair fibres.
    """
    roots = []
    for piece in source_pieces:
        roots.extend(piece_roots(compose(piece), piece))
    rows = []
    for root in merge(roots):
        piece = root['piece']
        target = classify(root, piece)
        if target is None:
            continue
        point, _ = point_at(piece, root)
        kind = transition(side_signs(root), 'O') if target == 'I' else 'E'
        fields = [index] + counts(root) + [position(root, *bounds), target, kind]
        rows.append((parameter_key(root), fields + coordinates(point), point))
    return rows


def parameter_key(root):
    low, high = refine(root['equation'], root['interval'], WIDTH)
    return ('parameter', (low + high) / 2, low == high)


def conic_classifier(conic):
    return lambda root, piece: sector_position(conic, root, piece['x'], piece['y'], piece['w'])


def line_classifier(line, bounded):
    def classify(root, piece):
        _, along = line_compose(line, piece)
        if not bounded:
            return 'I'
        dx, dy = line['direction']
        squared = dx * dx + dy * dy
        first = sign_at(root['equation'], root['interval'], along)
        second = sign_at(root['equation'], root['interval'], add(along, scale(piece['w'], -squared)))
        if first < 0 or second > 0:
            return None
        return 'S' if first == 0 else ('E' if second == 0 else 'I')
    return classify


def implicitize(piece):
    """F_T(x, y) = Res_u(X(u) - x, Y(u) - y) as {(i, j): coefficient}."""
    m, n = len(piece['x']) - 1, len(piece['y']) - 1
    size = m + n
    rows = []
    for shift in range(n):
        rows.append(shifted_row(piece['x'], m, shift, size, (1, 0)))
    for shift in range(m):
        rows.append(shifted_row(piece['y'], n, shift, size, (0, 1)))
    return determinant(rows)


def shifted_row(poly, degree, shift, size, variable):
    row = [{} for _ in range(size)]
    for power in range(degree + 1):
        entry = {(0, 0): poly[power]} if poly[power] != 0 else {}
        if power == 0:
            entry = dict(entry)
            entry[variable] = entry.get(variable, F(0)) - 1
        row[size - 1 - (power + shift)] = entry
    # Descending powers: column 0 is u^(size-1).
    return row


def bi_mul(first, second):
    product = {}
    for (i, j), a in first.items():
        for (k, l), b in second.items():
            product[(i + k, j + l)] = product.get((i + k, j + l), F(0)) + a * b
    return {key: value for key, value in product.items() if value != 0}


def bi_add(first, second, factor=F(1)):
    total = dict(first)
    for key, value in second.items():
        total[key] = total.get(key, F(0)) + factor * value
    return {key: value for key, value in total.items() if value != 0}


def determinant(rows):
    """Laplace expansion along the first row with memoized minors."""
    size = len(rows)
    memo = {}

    def minor(row, columns):
        if row == size:
            return {(0, 0): F(1)}
        key = (row, columns)
        if key in memo:
            return memo[key]
        total, sign_value = {}, 1
        for column in range(size):
            if columns & (1 << column):
                continue
            entry = rows[row][column]
            if entry:
                rest = minor(row + 1, columns | (1 << column))
                total = bi_add(total, bi_mul(entry, rest), F(sign_value))
            sign_value = -sign_value
        memo[key] = total
        return total
    return minor(0, 0)


def substitute(implicit, x, y):
    total = []
    powers_x, powers_y = {0: [F(1)]}, {0: [F(1)]}
    for (i, j), value in implicit.items():
        for k, base, cache in ((i, x, powers_x), (j, y, powers_y)):
            while k not in cache:
                last = max(cache)
                cache[last + 1] = mul(cache[last], base)
        total = add(total, scale(mul(powers_x[i], powers_y[j]), value))
    return total


def spline_spline(source_pieces, source_bounds, target_pieces, target_bounds, index):
    rows = []
    for target_piece in target_pieces:
        implicit = implicitize(target_piece)
        compose = lambda piece: substitute(implicit, piece['x'], piece['y'])
        found = spline_implicit(source_pieces, source_bounds, compose,
                                lambda root, piece: 'I', index)
        for key, fields, point in found:
            u = fibre(target_piece, point)
            lower, upper = target_piece['range']
            if not lower - F(1, 10 ** 200) <= u <= upper + F(1, 10 ** 200):
                continue
            if min(abs(u - lower), abs(u - upper)) < F(1, 10 ** 200):
                raise ValueError('generated spline contact at a target span end')
            fields[4] = position_value(u, target_bounds)
            rows.append((key, fields, point))
    return rows


def position_value(value, bounds):
    return 'I' if F(bounds[0]) < value < F(bounds[1]) else 'A'


def fibre(piece, point):
    """The target parameter of a contact, by high-precision inversion of one coordinate."""
    x, y = point
    axis, value, other, check = (piece['x'], x, piece['y'], y) if len(piece['x']) > 1 else \
        (piece['y'], y, piece['x'], x)
    candidates = []
    shifted = add(axis, [-value])
    for interval in roots_in(shifted, *cauchy(shifted)):
        low, high = refine(shifted, interval, F(1, 2 ** 280))
        u = (low + high) / 2
        if abs(evaluate(other, u) - check) < F(1, 10 ** 60):
            candidates.append(u)
    if len(candidates) != 1:
        raise ValueError('target fibre is not one point')
    return candidates[0]
