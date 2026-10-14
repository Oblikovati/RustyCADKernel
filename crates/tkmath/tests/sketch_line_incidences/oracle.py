# SPDX-License-Identifier: GPL-2.0-only
"""Generate near-degenerate sketch line incidence cases with exact expectations.

Rows: id;kind;domain;source;targets;contacts. Numbers are binary64 bit patterns.
Targets are separated by '|': circle:cx,cy,r[,sx,sy,ex,ey,ccw],
ellipse:cx,cy,ax,ay,ra,rb, segment:x0,y0,x1,y1, spline:degree:poles:knots:bounds.
Contacts list every support-line contact in exact line order as
target,left,right,line,target_position,xlo,xhi,ylo,yhi ('-' marks no side).

Usage: python3 oracle.py > oracle.csv
"""
from decimal import Decimal
from fractions import Fraction as F
import math
import random
import sys
from oracle_exact import (bracket, circle_coefficients, conic_contacts, ellipse_coefficients,
                          hexbits)
from oracle_spline import spline_contacts

SEED = 3563
SCALES = [1e-9, 1e-3, 1.0, 1e3, 1e9]


def numbers(values):
    return ','.join(hexbits(float(value)) for value in values)


def shift(point, axis, steps):
    point = list(point)
    for _ in range(abs(steps)):
        point[axis] = math.nextafter(point[axis], math.inf if steps > 0 else -math.inf)
    return tuple(point)


def conic_expectations(source, index, coefficients, sector, center):
    rows = []
    for root, count, line, target, coordinate in conic_contacts(source, coefficients, sector,
                                                               center):
        rows.append((root.decimal(), [index, count, count, line, target] + coordinate))
    return rows


def circle_target(center, radius, sector=None):
    spec = 'circle:' + numbers([center[0], center[1], radius])
    if sector:
        start, end, ccw = sector
        spec += ',' + numbers([start[0], start[1], end[0], end[1]]) + ',' + ('1' if ccw else '0')
    return spec


def circle_rows(source, index, center, radius, sector=None):
    coefficients = circle_coefficients(source, center, radius)
    return conic_expectations(source, index, coefficients, sector, center)


def ellipse_rows(source, index, center, axis, radii):
    coefficients = ellipse_coefficients(source, center, axis, radii)
    return conic_expectations(source, index, coefficients, None, center)


def spline_rows(source, index, degree, poles, knots, bounds):
    rows = []
    for contact in spline_contacts(source, degree, poles, knots, bounds):
        left, right = contact['multiplicity']
        coordinate = [bracket(value) for value in contact['point']]
        decimal = Decimal(contact['t'].numerator) / Decimal(contact['t'].denominator)
        rows.append((decimal, [index, left, right, contact['line'], contact['target']]
                     + coordinate))
    return rows


def segment_rows(source, index, start, end):
    # A segment is a degree-one spline on [0, 1].
    return spline_rows(source, index, 1, [start, end], [0., 0., 1., 1.], [0., 1.])


def emit(case, kind, domain, source, targets, rows):
    rows.sort(key=lambda row: row[0])
    for left, right in zip(rows, rows[1:]):
        if right[0] - left[0] < Decimal('1e-200'):
            raise ValueError(f'case {case}: unresolved generated tie')
    contacts = '|'.join(','.join('-' if v is None else str(v) for v in row[1]) for row in rows)
    print(';'.join([str(case), kind, domain, numbers(source[0] + source[1]), '|'.join(targets),
                    contacts]))


def tangent_point(center, radius, angle):
    normal = (-math.sin(angle), math.cos(angle))
    return (center[0] + radius * normal[0], center[1] + radius * normal[1])


def line_through(point, angle, half_length):
    direction = (math.cos(angle), math.sin(angle))
    start = (point[0] - half_length * direction[0], point[1] - half_length * direction[1])
    end = (point[0] + half_length * direction[0], point[1] + half_length * direction[1])
    return start, end


def circle_families(generator, cases):
    for scale in SCALES:
        center = (scale * generator.uniform(-1, 1), scale * generator.uniform(-1, 1))
        radius = scale * generator.uniform(0.5, 1.5)
        for angle in [0.0, generator.uniform(0, math.pi), generator.uniform(0, math.pi)]:
            touch = tangent_point(center, radius, angle)
            for steps in range(-2, 3):
                start, end = line_through(touch, angle, 2 * radius)
                source = (shift(start, 1, steps), shift(end, 1, steps))
                rows = circle_rows(source, 0, center, radius)
                cases.append(('circle_near_tangent', 'support', source,
                              [circle_target(center, radius)], rows))
    # Dyadic data: y = 0.75 + 0.25 is an exact tangent line.
    for steps in range(-2, 3):
        source = (shift((-3.0, 1.0), 1, steps), shift((3.0, 1.0), 1, steps))
        cases.append(('circle_exact_tangent', 'support', source,
                      [circle_target((0.5, 0.75), 0.25)],
                      circle_rows(source, 0, (0.5, 0.75), 0.25)))


def ellipse_families(generator, cases):
    for scale in SCALES[1:4]:
        center = (scale * generator.uniform(-1, 1), scale * generator.uniform(-1, 1))
        radii = (scale * generator.uniform(1, 2), scale * generator.uniform(0.25, 1))
        for _ in range(3):
            axis = (generator.uniform(-1, 1), generator.uniform(-1, 1))
            angle = generator.uniform(0, math.pi)
            touch = ellipse_support_point(center, axis, radii, angle)
            for steps in range(-2, 3):
                start, end = line_through(touch, angle, 3 * radii[0])
                source = (shift(start, 1, steps), shift(end, 1, steps))
                target = 'ellipse:' + numbers([center[0], center[1], axis[0], axis[1],
                                               radii[0], radii[1]])
                cases.append(('ellipse_near_tangent', 'support', source, [target],
                              ellipse_rows(source, 0, center, axis, radii)))


def ellipse_support_point(center, axis, radii, angle):
    length = math.hypot(*axis)
    major = (axis[0] / length, axis[1] / length)
    minor = (-major[1], major[0])
    normal = (-math.sin(angle), math.cos(angle))
    along = radii[0] ** 2 * (normal[0] * major[0] + normal[1] * major[1])
    across = radii[1] ** 2 * (normal[0] * minor[0] + normal[1] * minor[1])
    scale = math.sqrt(along * (normal[0] * major[0] + normal[1] * major[1])
                      + across * (normal[0] * minor[0] + normal[1] * minor[1]))
    return (center[0] + (along * major[0] + across * minor[0]) / scale,
            center[1] + (along * major[1] + across * minor[1]) / scale)


def endpoint_families(generator, cases):
    for scale in SCALES[1:4]:
        center = (scale * generator.uniform(-1, 1), scale * generator.uniform(-1, 1))
        radius = scale * generator.uniform(0.5, 1.5)
        angle = generator.uniform(0, 2 * math.pi)
        on_circle = (center[0] + radius * math.cos(angle), center[1] + radius * math.sin(angle))
        far = (center[0] - 3 * radius * math.cos(angle + 0.3),
               center[1] - 3 * radius * math.sin(angle + 0.3))
        for steps in range(-2, 3):
            source = (shift(on_circle, 0, steps), far)
            cases.append(('endpoint_near_circle', 'segment', source,
                          [circle_target(center, radius)],
                          circle_rows(source, 0, center, radius)))


def arc_families(generator, cases):
    # Lines through the center meet the circle on the rays +d and -d exactly.
    for case in range(6):
        center = (generator.uniform(-2, 2), generator.uniform(-2, 2))
        radius = generator.uniform(0.5, 2)
        direction = (generator.uniform(-1, 1), generator.uniform(-1, 1))
        start = (center[0] - 3 * direction[0], center[1] - 3 * direction[1])
        end = (start[0] + 6 * direction[0], start[1] + 6 * direction[1])
        for steps in (-1, 0, 1):
            boundary = shift(direction, 1, steps)
            other = (-direction[1], direction[0])
            sector = (boundary, other, case % 2 == 0)
            rows = circle_rows((start, end), 0, center, radius, sector)
            cases.append(('arc_boundary', 'support', (start, end),
                          [circle_target(center, radius, sector)], rows))


def ordering_families(generator, cases):
    for steps in (1, 2, 5):
        center = (generator.uniform(-1, 1), generator.uniform(-1, 1))
        radius = generator.uniform(0.5, 2)
        larger = radius
        for _ in range(steps):
            larger = math.nextafter(larger, math.inf)
        offset = (center[0] + generator.uniform(-1e-12, 1e-12), center[1])
        angle = generator.uniform(0, math.pi)
        source = line_through(center, angle, 3 * radius)
        rows = circle_rows(source, 0, center, radius) + circle_rows(source, 1, offset, larger)
        cases.append(('close_circles', 'support', source,
                      [circle_target(center, radius), circle_target(offset, larger)], rows))


def line_families(generator, cases):
    # A target segment end within a few ulps of the source line, and nearly parallel lines.
    for steps in range(-2, 3):
        start, end = (generator.uniform(-1, 0), generator.uniform(-1, 0)), (1.0, 0.7)
        near = (0.3, start[1] + (0.3 - start[0]) * (0.7 - start[1]) / (1.0 - start[0]))
        near = shift(near, 1, steps)
        far = (near[0] + generator.uniform(-1, 1), near[1] + 2.0)
        add_segment_case(cases, 'segment_end_near_line', (start, end), near, far)
    for steps in (1, 3, 1000):
        start, end = (0.0, 0.0), (1.0, generator.uniform(-1, 1))
        other_start = shift((0.5, 0.5 * end[1]), 1, steps)
        other_end = (other_start[0] - 2.0, other_start[1] - 2.0 * end[1])
        add_segment_case(cases, 'near_parallel_segments', (start, end), other_start, other_end)


def add_segment_case(cases, kind, source, start, end):
    spec = 'segment:' + numbers(start + end)
    cases.append((kind, 'support', source, [spec], segment_rows(source, 0, start, end)))


def spline_families(generator, cases):
    knots = [0., 0., 0., 0., 0.375, 1., 1.5, 1.5, 1.5, 1.5]
    for _ in range(5):
        poles = [(0.25 * i, generator.uniform(-1, 1)) for i in range(6)]
        top = max(sampled_y(poles, knots, u / 512) for u in range(769))
        for steps in range(-2, 3):
            height = shift((0.0, top), 1, steps)[1]
            source = ((-1.0, height), (3.0, height))
            add_spline_case(cases, 'spline_near_tangent', source, poles, knots)
    # Horizontal lines through every exactly representable knot point.
    poles = [(0., 0.), (0.5, 1.), (1., -1.), (1.5, 0.5), (2., 0.25), (2.5, -0.5)]
    for u in knots[4:6]:
        y = exact_y(poles, knots, u)
        if float(y) == y:
            add_spline_case(cases, 'spline_knot_point', ((-1.0, float(y)), (3.0, float(y))),
                            poles, knots)


def add_spline_case(cases, kind, source, poles, knots):
    spec = 'spline:3:' + numbers([c for pole in poles for c in pole]) + ':' + numbers(knots) \
        + ':' + numbers([0.0, 1.5])
    rows = spline_rows(source, 0, 3, poles, knots, [0.0, 1.5])
    cases.append((kind, 'support', source, [spec], rows))


def sampled_y(poles, knots, u):
    return float(exact_y(poles, knots, F(u)))


def exact_y(poles, knots, u):
    from oracle_spline import de_boor
    span = max(index for index in range(3, 6) if F(knots[index]) <= u)
    return de_boor(3, poles, knots, span, F(u))[1]


def main():
    generator = random.Random(SEED)
    cases = []
    for family in (circle_families, ellipse_families, endpoint_families, arc_families,
                   ordering_families, line_families, spline_families):
        family(generator, cases)
    for case, (kind, domain, source, targets, rows) in enumerate(cases):
        emit(case, kind, domain, source, targets, rows)
    print(f'{len(cases)} cases', file=sys.stderr)


if __name__ == '__main__':
    main()
