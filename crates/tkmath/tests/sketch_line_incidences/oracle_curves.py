# SPDX-License-Identifier: GPL-2.0-only
"""Generate near-degenerate conic- and spline-source incidence cases (ADR 0061).

Rows: id;kind;domain;source;targets;contacts. Numbers are binary64 bit patterns.
Curves use circle:cx,cy,r[,sx,sy,ex,ey,ccw], ellipse:cx,cy,ax,ay,ra,rb,
circlearc:cx,cy,r,ax,ay,t0,t1,ccw and ellipsearc:cx,cy,mx,my,ra,rb,ax,ay,t0,t1,ccw
(parameter-angle bounds in the frame of axis a),
segment:x0,y0,x1,y1, line:x0,y0,x1,y1 (unbounded) and
spline:degree:poles:knots:bounds. Contacts list every certified contact in exact
source order as target,left,right,source,target,transition,xlo,xhi,ylo,yhi with
positions B/S/I/E/A, transitions C (crossing), T (touching), E (target end),
O (source end), and '-' for a missing side.

Usage: python3 oracle_curves.py > oracle-curves.csv
"""
import functools
import math
import random
import sys
from fractions import Fraction as F
from oracle_curves_exact import AngleSector, Conic, line_implicit, line_pieces, spline_pieces
from oracle_curves_pairs import (ConicSource, conic_classifier, conic_conic, conic_parametric,
                                 line_classifier, spline_implicit, spline_spline, sweep_less)
from oracle_curves_exact import line_compose
from oracle_exact import hexbits

SEED = 6100
SCALES = [1e-6, 1e-3, 1.0, 1e3, 1e6]


def numbers(values):
    return ','.join(hexbits(float(value)) for value in values)


def shift(value, steps):
    for _ in range(abs(steps)):
        value = math.nextafter(value, math.inf if steps > 0 else -math.inf)
    return value


# ---------- curve specs and oracle objects ----------
def circle_spec(center, radius, sector=None):
    spec = 'circle:' + numbers([center[0], center[1], radius])
    if sector:
        start, end, ccw = sector
        spec += ',' + numbers([start[0], start[1], end[0], end[1]]) + ',' + ('1' if ccw else '0')
    return spec


def circle_arc_spec(center, radius, axis, bounds, ccw):
    return 'circlearc:' + numbers([center[0], center[1], radius, axis[0], axis[1], bounds[0],
                                   bounds[1]]) + (',1' if ccw else ',0')


def ellipse_arc_spec(center, major, radii, axis, bounds, ccw):
    return 'ellipsearc:' + numbers([center[0], center[1], major[0], major[1], radii[0], radii[1],
                                    axis[0], axis[1], bounds[0], bounds[1]]) + (',1' if ccw else ',0')


def ellipse_spec(center, axis, radii):
    return 'ellipse:' + numbers([center[0], center[1], axis[0], axis[1], radii[0], radii[1]])


def spline_spec(degree, poles, knots, bounds):
    return 'spline:%d:%s:%s:%s' % (degree, numbers(c for pole in poles for c in pole),
                                   numbers(knots), numbers(bounds))


def line_spec(start, end, bounded):
    return ('segment:' if bounded else 'line:') + numbers(start + end)


def target_rows(source, target, index):
    """Dispatch one oracle target against a conic or spline source."""
    kind = target[0]
    if source[0] == 'conic':
        conic_source = source[1]
        if kind == 'line':
            _, start, end, bounded = target
            return conic_parametric(conic_source, line_pieces((start, end), bounded),
                                    (0, 1) if bounded else None, index)
        if kind == 'conic':
            return conic_conic(conic_source, target[1], index)
        _, degree, poles, knots, bounds = target
        return conic_parametric(conic_source, spline_pieces(degree, poles, knots, bounds),
                                bounds, index)
    _, pieces, bounds = source
    if kind == 'line':
        _, start, end, bounded = target
        line = line_implicit((start, end))
        compose = lambda piece: line_compose(line, piece)[0]
        return strip(spline_implicit(pieces, bounds, compose, line_classifier(line, bounded),
                                     index))
    if kind == 'conic':
        return strip(spline_implicit(pieces, bounds, lambda piece: target[1].compose(
            piece['x'], piece['y'], piece['w']), conic_classifier(target[1]), index))
    _, degree, poles, knots, target_bounds = target
    return strip(spline_spline(pieces, bounds, spline_pieces(degree, poles, knots, target_bounds),
                               target_bounds, index))


def strip(rows):
    return [(key, fields) for key, fields, _ in rows]


def ordered(rows, conic):
    if conic:
        less = lambda a, b: -1 if sweep_less(a[0], b[0]) else (1 if sweep_less(b[0], a[0]) else 0)
        return sorted(rows, key=functools.cmp_to_key(less))
    rows = sorted(rows, key=lambda row: row[0][1])
    for left, right in zip(rows, rows[1:]):
        exact_tie = left[0][2] and right[0][2] and left[0][1] == right[0][1]
        if not exact_tie and right[0][1] - left[0][1] < F(1, 10 ** 200):
            raise ValueError('unresolved generated parameter order')
    return rows


def emit(case, kind, domain, source_spec, source, target_specs, targets):
    rows = []
    for index, target in enumerate(targets):
        rows.extend(target_rows(source, target, index))
    rows = ordered(rows, source[0] == 'conic')
    if domain == 'segment':
        rows = [row for row in rows if row[1][3] in 'SIE']
    contacts = '|'.join(','.join(str(value) for value in row[1]) for row in rows)
    print(';'.join([str(case), kind, domain, source_spec, '|'.join(target_specs), contacts]))


# ---------- families ----------
def unit(angle):
    return (math.cos(angle), math.sin(angle))


def circle_line_tangent(generator, cases):
    for scale in SCALES:
        center = (scale * generator.uniform(-1, 1), scale * generator.uniform(-1, 1))
        radius = scale * generator.uniform(0.5, 1.5)
        source = ('conic', ConicSource(Conic.circle(center, radius), (1.0, 0.0)))
        for angle in (generator.uniform(0, math.pi), generator.uniform(0, math.pi)):
            normal = (-math.sin(angle), math.cos(angle))
            touch = (center[0] + radius * normal[0], center[1] + radius * normal[1])
            direction = unit(angle)
            for steps in range(-2, 3):
                start = (touch[0] - radius * direction[0], shift(touch[1] - radius * direction[1], steps))
                end = (touch[0] + radius * direction[0], shift(touch[1] + radius * direction[1], steps))
                target = ('line', start, end, False)
                cases.append(('circle_source_line_tangent', 'segment', circle_spec(center, radius),
                              source, [line_spec(start, end, False)], [target]))


def arc_end_ray(generator, cases):
    # Centered at the origin with coarse dyadic rays, so a line through the center
    # along the end ray is exact and its perturbations are one ulp off the ray.
    for case in range(6):
        radius = generator.uniform(0.5, 2)
        first = (generator.randint(-64, 64) / 64, generator.randint(1, 64) / 64)
        second = (-first[1], first[0]) if case % 2 == 0 else (first[1], -first[0])
        ccw = case % 2 == 0
        sector = (first, second, ccw)
        source = ('conic', ConicSource(Conic.circle((0.0, 0.0), radius, sector), first))
        for steps in (-1, 0, 1):
            direction = (second[0], shift(second[1], steps))
            target = ('line', (0.0, 0.0), direction, False)
            cases.append(('arc_source_end_ray', 'support', circle_spec((0.0, 0.0), radius, sector),
                          source, [line_spec((0.0, 0.0), direction, False)], [target]))


def segment_end_on_circle(generator, cases):
    # Pythagorean ends lie exactly on the circle at zero steps.
    for exponent in (-10, 0, 10):
        scale = 2.0 ** exponent
        center = (generator.randint(-8, 8) * scale / 8, generator.randint(-8, 8) * scale / 8)
        radius = 5 * scale
        source = ('conic', ConicSource(Conic.circle(center, radius), (1.0, 0.0)))
        on_circle = (center[0] + 3 * scale, center[1] + 4 * scale)
        far = (center[0] - 2 * scale, center[1] - 9 * scale)
        for steps in range(-2, 3):
            end = (shift(on_circle[0], steps), on_circle[1])
            target = ('line', far, end, True)
            cases.append(('segment_end_on_circle_source', 'segment', circle_spec(center, radius),
                          source, [line_spec(far, end, True)], [target]))
    # Dyadic data: y = 1 is exactly tangent to the circle of radius 1/4 about (1/2, 3/4).
    source = ('conic', ConicSource(Conic.circle((0.5, 0.75), 0.25), (1.0, 0.0)))
    for steps in range(-2, 3):
        start, end = (-3.0, shift(1.0, steps)), (3.0, shift(1.0, steps))
        cases.append(('circle_source_exact_tangent', 'segment', circle_spec((0.5, 0.75), 0.25),
                      source, [line_spec(start, end, False)], [('line', start, end, False)]))


def circle_source_spline(generator, cases):
    for _ in range(3):
        poles = [(0.25 * i, generator.uniform(-1, 1)) for i in range(6)]
        pieces = spline_pieces(3, poles, KNOTS, (0.0, 1.5))
        samples = [(sampled_y(pieces, u / 4096), u / 4096) for u in range(6145)]
        top, at = max(samples)
        x = sampled_x(pieces, at)
        radius = 0.0625
        for steps in range(-2, 3):
            center = (x, shift(top + radius, steps * 4))
            source = ('conic', ConicSource(Conic.circle(center, radius), (1.0, 0.0)))
            target = ('spline', 3, poles, KNOTS, (0.0, 1.5))
            cases.append(('circle_source_spline', 'segment', circle_spec(center, radius), source,
                          [spline_spec(3, poles, KNOTS, (0.0, 1.5))], [target]))


def sampled_x(pieces, u):
    from oracle_exact import evaluate
    value = F(u)
    for piece in pieces:
        if piece['range'][0] <= value <= piece['range'][1]:
            return float(evaluate(piece['x'], value))
    raise ValueError('parameter outside the spline')


def spline_source_arc(generator, cases):
    # A circle arc whose end ray passes within an ulp of a polyline source crossing.
    poles = [(-2.0, -0.5), (2.0, 0.5)]
    knots = [0., 0., 1., 1.]
    pieces = spline_pieces(1, poles, knots, (0.0, 1.0))
    for steps in (-1, 0, 1):
        end = (4.0, shift(1.0, steps))
        sector = ((1.0, 0.0), end, True)
        target = ('conic', Conic.circle((0.0, 0.0), math.hypot(*end) and 1.25, sector))
        cases.append(('spline_source_arc_end', 'segment', spline_spec(1, poles, knots, (0., 1.)),
                      ('spline', pieces, (0.0, 1.0)), [circle_spec((0.0, 0.0), 1.25, sector)],
                      [target]))


def circle_pairs(generator, cases):
    for exponent in (-20, 0, 20):
        scale = 2.0 ** exponent
        center = (0.0, 0.0)
        radius = 0.75 * scale
        source = ('conic', ConicSource(Conic.circle(center, radius), (1.0, 0.0)))
        for internal in (False, True):
            other = (2.0 * scale, 0.0) if not internal else (0.25 * scale, 0.0)
            base = 1.25 * scale if not internal else 0.5 * scale
            for steps in range(-2, 3):
                other_radius = shift(base, steps)
                target = ('conic', Conic.circle(other, other_radius))
                cases.append(('circle_pair_tangent', 'segment', circle_spec(center, radius), source,
                              [circle_spec(other, other_radius)], [target]))
    for _ in range(3):
        center = (generator.uniform(-1, 1), generator.uniform(-1, 1))
        radius = generator.uniform(0.5, 1.5)
        source = ('conic', ConicSource(Conic.circle(center, radius), (1.0, 0.0)))
        angle = generator.uniform(0, 2 * math.pi)
        other_radius = generator.uniform(0.25, 0.75)
        distance = radius + other_radius
        other = (center[0] + distance * math.cos(angle), center[1] + distance * math.sin(angle))
        for steps in range(-2, 3):
            value = shift(other_radius, steps * 4)
            target = ('conic', Conic.circle(other, value))
            cases.append(('circle_pair_near_tangent', 'segment', circle_spec(center, radius),
                          source, [circle_spec(other, value)], [target]))


def ellipse_source(generator, cases):
    for _ in range(3):
        center = (generator.uniform(-1, 1), generator.uniform(-1, 1))
        axis = (generator.uniform(-1, 1), generator.uniform(-1, 1))
        radii = (generator.uniform(1, 2), generator.uniform(0.25, 1))
        conic = Conic.ellipse(center, axis, radii)
        source = ('conic', ConicSource(conic, axis))
        other = (center[0] + generator.uniform(2, 3), center[1] + generator.uniform(-1, 1))
        base = nearest_distance(center, axis, radii, other)
        for steps in range(-2, 3):
            value = shift(base, steps * 8)
            target = ('conic', Conic.circle(other, value))
            cases.append(('ellipse_source_circle', 'segment', ellipse_spec(center, axis, radii),
                          source, [circle_spec(other, value)], [target]))


def nearest_distance(center, axis, radii, point):
    length = math.hypot(*axis)
    major = (axis[0] / length, axis[1] / length)
    minor = (-major[1], major[0])

    def distance(angle):
        x = center[0] + radii[0] * math.cos(angle) * major[0] + radii[1] * math.sin(angle) * minor[0]
        y = center[1] + radii[0] * math.cos(angle) * major[1] + radii[1] * math.sin(angle) * minor[1]
        return math.hypot(x - point[0], y - point[1])
    best = min(range(4096), key=lambda step: distance(2 * math.pi * step / 4096))
    low, high = 2 * math.pi * (best - 1) / 4096, 2 * math.pi * (best + 1) / 4096
    for _ in range(200):
        first, second = low + (high - low) / 3, high - (high - low) / 3
        if distance(first) < distance(second):
            high = second
        else:
            low = first
    return distance((low + high) / 2)


KNOTS = [0., 0., 0., 0., 0.375, 1., 1.5, 1.5, 1.5, 1.5]


def spline_source_line(generator, cases):
    for _ in range(5):
        poles = [(0.25 * i, generator.uniform(-1, 1)) for i in range(6)]
        pieces = spline_pieces(3, poles, KNOTS, (0.0, 1.5))
        top = max(sampled_y(pieces, u / 512) for u in range(769))
        for steps in range(-2, 3):
            height = shift(top, steps)
            target = ('line', (-1.0, height), (3.0, height), False)
            cases.append(('spline_source_line_tangent', 'segment',
                          spline_spec(3, poles, KNOTS, (0.0, 1.5)), ('spline', pieces, (0.0, 1.5)),
                          [line_spec((-1.0, height), (3.0, height), False)], [target]))


def sampled_y(pieces, u):
    from oracle_exact import evaluate
    value = F(u)
    for piece in pieces:
        if piece['range'][0] <= value <= piece['range'][1]:
            return float(evaluate(piece['y'], value))
    return -math.inf


def spline_source_circle_knot(generator, cases):
    poles = [(0., 0.), (0.5, 1.), (1., -1.), (1.5, 0.5), (2., 0.25), (2.5, -0.5)]
    pieces = spline_pieces(3, poles, KNOTS, (0.0, 1.5))
    for knot in KNOTS[4:6]:
        from oracle_exact import evaluate
        piece = next(piece for piece in pieces if piece['range'][0] == F(knot))
        point = (float(evaluate(piece['x'], F(knot))), float(evaluate(piece['y'], F(knot))))
        center = (point[0] + 0.5, point[1] + 0.75)
        radius = math.hypot(0.5, 0.75)
        for steps in range(-2, 3):
            value = shift(radius, steps)
            target = ('conic', Conic.circle(center, value))
            cases.append(('spline_source_circle_knot', 'segment',
                          spline_spec(3, poles, KNOTS, (0.0, 1.5)), ('spline', pieces, (0.0, 1.5)),
                          [circle_spec(center, value)], [target]))


def spline_pairs(generator, cases):
    bezier = [0., 0., 0., 1., 1., 1.]
    for exponent in (-20, 0, 20):
        scale = 2.0 ** exponent
        poles = [(-scale, scale), (0.0, -scale), (scale, scale)]
        pieces = spline_pieces(2, poles, bezier, (0.0, 1.0))
        for steps in range(-2, 3):
            lift = steps * 2.0 ** -50 * scale
            other = [(-scale, lift - scale), (0.0, lift + scale), (scale, lift - scale)]
            target = ('spline', 2, other, bezier, (0.0, 1.0))
            cases.append(('spline_pair_tangent', 'segment', spline_spec(2, poles, bezier, (0., 1.)),
                          ('spline', pieces, (0.0, 1.0)), [spline_spec(2, other, bezier, (0., 1.))],
                          [target]))
    for _ in range(3):
        poles = [(0.0, generator.uniform(-1, 1)), (1.0, generator.uniform(-1, 1)),
                 (2.0, generator.uniform(-1, 1)), (3.0, generator.uniform(-1, 1))]
        knots = [0., 0., 0., 0., 1., 1., 1., 1.]
        pieces = spline_pieces(3, poles, knots, (0.0, 1.0))
        other = [(generator.uniform(0.5, 1), -2.0), (generator.uniform(1, 2), 0.0),
                 (generator.uniform(1.5, 2.5), 2.0)]
        target = ('spline', 2, other, bezier, (0.0, 1.0))
        cases.append(('spline_pair_crossing', 'segment', spline_spec(3, poles, knots, (0., 1.)),
                      ('spline', pieces, (0.0, 1.0)), [spline_spec(2, other, bezier, (0., 1.))],
                      [target]))


def spline_source_angle_arc(generator, cases):
    # y = x/4 meets circles about the origin along (4, 1): angle atan(1/4), never a
    # binary64 value, so end angles a few ulps away fall strictly inside or outside.
    poles = [(-2.0, -0.5), (2.0, 0.5)]
    knots = [0., 0., 1., 1.]
    pieces = spline_pieces(1, poles, knots, (0.0, 1.0))
    for radius in (0.75, 1.25, 1.75):
        for steps in range(-2, 3):
            end = shift(math.atan(0.25), steps)
            sector = AngleSector((1.0, 0.0), True, (0.0, end), (radius, radius))
            target = ('conic', Conic.circle((0.0, 0.0), radius, sector))
            cases.append(('spline_source_angle_arc', 'segment', spline_spec(1, poles, knots, (0., 1.)),
                          ('spline', pieces, (0.0, 1.0)),
                          [circle_arc_spec((0.0, 0.0), radius, (1.0, 0.0), (0.0, end), True)],
                          [target]))


def angle_arc_source(generator, cases):
    # x = 0.8 meets the unit circle (and x = 1.6 the ellipse x^2/4 + y^2 = 1) at
    # parameter angles +-atan(3/4); start angles a few ulps from -atan(3/4).
    for steps in range(-2, 3):
        start = shift(-math.atan(0.75), steps)
        bounds = (start, 1.0)
        sector = AngleSector((1.0, 0.0), True, bounds, (1.0, 1.0))
        source = ('conic', ConicSource(Conic.circle((0.0, 0.0), 1.0, sector), (1.0, 0.0)))
        target = ('line', (0.8, -2.0), (0.8, 2.0), True)
        cases.append(('angle_arc_source', 'support',
                      circle_arc_spec((0.0, 0.0), 1.0, (1.0, 0.0), bounds, True), source,
                      [line_spec((0.8, -2.0), (0.8, 2.0), True)], [target]))
        sector = AngleSector((1.0, 0.0), True, bounds, (2.0, 1.0))
        conic = Conic.ellipse((0.0, 0.0), (1.0, 0.0), (2.0, 1.0), sector)
        source = ('conic', ConicSource(conic, (1.0, 0.0)))
        target = ('line', (1.6, -2.0), (1.6, 2.0), True)
        cases.append(('angle_ellipse_source', 'support',
                      ellipse_arc_spec((0.0, 0.0), (1.0, 0.0), (2.0, 1.0), (1.0, 0.0), bounds, True),
                      source, [line_spec((1.6, -2.0), (1.6, 2.0), True)], [target]))


def main():
    generator = random.Random(SEED)
    cases = []
    for family in (circle_line_tangent, arc_end_ray, segment_end_on_circle, circle_pairs,
                   ellipse_source, circle_source_spline, spline_source_line,
                   spline_source_circle_knot, spline_source_arc, spline_pairs,
                   spline_source_angle_arc, angle_arc_source):
        family(generator, cases)
    for case, (kind, domain, source_spec, source, specs, targets) in enumerate(cases):
        emit(case, kind, domain, source_spec, source, specs, targets)
    print(f'{len(cases)} cases', file=sys.stderr)


if __name__ == '__main__':
    main()
