# SPDX-License-Identifier: GPL-2.0-only
"""Exact B-spline contacts for the incidence oracle.

Span polynomials are recovered by exact interpolation of de Boor points, a
different construction from the Cox-de Boor basis used by the kernel. Roots,
multiplicities and signs use the exact Fraction Sturm helpers.
"""
from fractions import Fraction as F
from oracle_exact import evaluate, multiplicity, position, refine, roots_in, sign_at, trim


def de_boor(degree, poles, knots, span, u):
    points = [tuple(map(F, poles[span - degree + j])) for j in range(degree + 1)]
    for level in range(1, degree + 1):
        for j in range(degree, level - 1, -1):
            index = span - degree + j
            width = F(knots[index + degree + 1 - level]) - F(knots[index])
            alpha = (u - F(knots[index])) / width
            points[j] = tuple((1 - alpha) * points[j - 1][a] + alpha * points[j][a]
                              for a in range(2))
    return points[degree]


def interpolate(nodes, values):
    total = [F(0)] * len(nodes)
    for i, (node, value) in enumerate(zip(nodes, values)):
        basis, scale = [F(1)], F(1)
        for j, other in enumerate(nodes):
            if j == i:
                continue
            basis = [F(0)] + basis
            for k in range(len(basis) - 1):
                basis[k] -= other * basis[k + 1]
            scale *= node - other
        for k, coefficient in enumerate(basis):
            total[k] += value * coefficient / scale
    return total


def span_polynomials(degree, poles, knots, span, lower, upper):
    nodes = [lower + (upper - lower) * F(k, degree) for k in range(degree + 1)]
    points = [de_boor(degree, poles, knots, span, u) for u in nodes]
    return [interpolate(nodes, [p[a] for p in points]) for a in range(2)]


def shifted(poly, constant):
    poly = list(poly) + [F(0)]
    poly[0] -= constant
    return trim(poly)


def spline_contacts(source, degree, poles, knots, bounds):
    start, end = [tuple(map(F, p)) for p in source]
    direction = (end[0] - start[0], end[1] - start[1])
    squared = direction[0] ** 2 + direction[1] ** 2
    contacts = []
    for span in range(degree, len(poles)):
        lower = max(F(knots[span]), F(bounds[0]))
        upper = min(F(knots[span + 1]), F(bounds[1]))
        if lower >= upper:
            continue
        x, y = span_polynomials(degree, poles, knots, span, lower, upper)
        rel_x, rel_y = shifted(x, start[0]), shifted(y, start[1])
        size = max(len(rel_x), len(rel_y))
        rel_x += [F(0)] * (size - len(rel_x))
        rel_y += [F(0)] * (size - len(rel_y))
        normal = trim([direction[0] * cy - direction[1] * cx for cx, cy in zip(rel_x, rel_y)])
        along = trim([direction[0] * cx + direction[1] * cy for cx, cy in zip(rel_x, rel_y)])
        if not normal:
            raise ValueError('generated spline overlaps the line')
        geometry = {'normal': normal, 'along': along, 'squared': squared,
                    'range': (lower, upper), 'xy': (x, y), 'bounds': bounds}
        for interval in roots_in(normal, lower, upper):
            contacts.append(span_contact(geometry, interval))
    return merge_knots(contacts)


def span_contact(geometry, interval):
    normal, along, squared = geometry['normal'], geometry['along'], geometry['squared']
    start = sign_at(normal, interval, along)
    end = sign_at(normal, interval, shifted(along, squared))
    count = multiplicity(normal, interval)
    exact = interval[0] == interval[1]
    low, high = refine(normal, interval, F(1, 2 ** 300))
    u = low if low == high else (low + high) / 2
    bounds = geometry['bounds']
    target = 'I'
    if low == high and u == F(bounds[0]):
        target = 'S'
    if low == high and u == F(bounds[1]):
        target = 'E'
    sides = [None if exact and u == geometry['range'][0] else count,
             None if exact and u == geometry['range'][1] else count]
    return {'u': u, 't': evaluate(along, u) / squared, 'exact': low == high,
            'line': position(start, end), 'target': target, 'multiplicity': sides,
            'point': [evaluate(geometry['xy'][a], u) for a in range(2)]}


def merge_knots(contacts):
    merged = []
    for contact in contacts:
        previous = merged[-1] if merged else None
        if (previous and contact['multiplicity'][0] is None
                and previous['multiplicity'][1] is None and previous['u'] == contact['u']):
            previous['multiplicity'][1] = contact['multiplicity'][1]
            continue
        merged.append(contact)
    return merged
