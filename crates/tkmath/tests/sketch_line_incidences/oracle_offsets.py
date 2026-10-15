# SPDX-License-Identifier: GPL-2.0-only
"""Generate near-degenerate offset and periodic-spline incidence cases (ADR 0080).

Rows follow oracle_curves.py: id;kind;domain;source;targets;contacts. New specs:
offset:<distance>:<basis spec> (a Geom2d_OffsetCurve of the basis) and
periodic:degree:poles:knots:multiplicities:bounds (a native periodic B-spline,
multiplicities as decimal integers). Contacts are certified by exact Fraction
algebra in oracle_offsets_exact.py and oracle_offsets_pairs.py.

Usage: python3 oracle_offsets.py > oracle-offsets.csv
"""
import functools
import math
import random
import sys
from fractions import Fraction as F
from oracle_curves import (KNOTS, circle_spec, ellipse_spec, line_spec, numbers, ordered, shift,
                           spline_spec, strip)
from oracle_curves_exact import Conic, line_implicit, line_compose, spline_pieces
from oracle_curves_pairs import (ConicSource, conic_parametric, line_classifier, spline_implicit)
from oracle_offsets_pairs import (Offset, ellipse_frame, implicit_offset_target,
                                  offset_source_implicit)
from oracle_offsets_exact import OffsetPiece, surd_decimal

SEED = 8080
EXPONENTS = (-10, 0, 10)


def offset_spec(distance, basis):
    return 'offset:' + numbers([distance]) + ':' + basis


def periodic_spec(degree, poles, knots, mults, bounds):
    return 'periodic:%d:%s:%s:%s:%s' % (degree, numbers(c for pole in poles for c in pole),
                                        numbers(knots), ','.join(map(str, mults)), numbers(bounds))


# ---------- dispatch ----------
def target_rows(source, target, index):
    if target[0] == 'offset' and target[1].kind != 'line':
        return implicit_offset_target(source, target[1], index)
    if source[0] == 'offsetsource':
        return offset_source_implicit(source[1], target, index)
    if target[0] == 'offset':
        return implicit_offset_target(source, target[1], index)
    if source[0] == 'spline':
        _, pieces, bounds = source
        line = line_implicit((target[1], target[2]))
        compose = lambda piece: line_compose(line, piece)[0]
        return strip(spline_implicit(pieces, bounds, compose, line_classifier(line, target[3]),
                                     index))
    _, degree, poles, knots, bounds = target
    return conic_parametric(source[1], spline_pieces(degree, poles, knots, bounds), bounds, index)


def emit(case, kind, domain, source_spec, source, target_specs, targets):
    rows = []
    for index, target in enumerate(targets):
        rows.extend(target_rows(source, target, index))
    sweep = source[0] == 'conic' or (source[0] == 'offsetsource' and source[1].kind == 'ellipse')
    rows = ordered(rows, sweep)
    if domain == 'segment':
        rows = [row for row in rows if row[1][3] in 'SIE']
    contacts = '|'.join(','.join(str(value) for value in row[1]) for row in rows)
    print(';'.join([str(case), kind, domain, source_spec, '|'.join(target_specs), contacts]))


# ---------- families ----------
def ellipse_offset_targets(cases):
    """Lines within +-2 ulps of the top of an ellipse grown by d (axis-aligned and (3, 4))."""
    for exponent in EXPONENTS:
        s = 2.0 ** exponent
        center, radii, distance = (0.25 * s, -0.5 * s), (2 * s, s), 0.5 * s
        target = ('offset', Offset.ellipse(center, (1.0, 0.0), radii, distance))
        spec = offset_spec(distance, ellipse_spec(center, (1.0, 0.0), radii))
        for steps in range(-2, 3):
            height = shift(s, steps)
            start, end = (center[0] - 4 * s, height), (center[0] + 4 * s, height)
            cases.append(('line_offset_ellipse_top', 'segment', line_spec(start, end, True),
                          ('line', (start, end)), [spec], [target]))
        # Axis (3, 4): the top along n = (-4, 3)/5 is (-4s, 3s) for rb + d = 5s.
        target = ('offset', Offset.ellipse((0.0, 0.0), (3.0, 4.0), (6 * s, 4 * s), s))
        spec = offset_spec(s, ellipse_spec((0.0, 0.0), (3.0, 4.0), (6 * s, 4 * s)))
        for steps in range(-2, 3):
            start, end = (-10 * s, shift(-5 * s, steps)), (2 * s, 11 * s)
            cases.append(('line_rotated_offset_ellipse_top', 'support', line_spec(start, end, True),
                          ('line', (start, end)), [spec], [target]))


def spline_offset_targets(generator, cases):
    """Lines within +-2 ulps of the sampled top of a cubic spline offset."""
    for _ in range(4):
        poles = [(0.25 * i, generator.uniform(-1, 1)) for i in range(6)]
        target = ('offset', Offset.spline(3, poles, KNOTS, (0.0, 1.5), 0.125))
        spec = offset_spec(0.125, spline_spec(3, poles, KNOTS, (0.0, 1.5)))
        top = offset_top(target[1])
        for steps in range(-2, 3):
            height = shift(top, steps)
            start, end = (-1.0, height), (3.0, height)
            cases.append(('line_offset_spline_top', 'segment', line_spec(start, end, True),
                          ('line', (start, end)), [spec], [target]))


def offset_top(offset):
    best = None
    for piece in offset.pieces:
        low, high = piece.range
        samples = [low + (high - low) * F(k, 64) for k in range(65)]
        for u in samples:
            y = float(piece.point(u)[1])
            best = y if best is None or y > best else best
    return refine_top(offset, best)


def refine_top(offset, guess):
    """Golden-section search on each piece; returns the largest binary64 maximum."""
    best = guess
    for piece in offset.pieces:
        low, high = float(piece.range[0]), float(piece.range[1])
        value = lambda u: float(piece.point(F(u))[1])
        for _ in range(80):
            first, second = low + (high - low) / 3, high - (high - low) / 3
            if value(first) < value(second):
                low = first
            else:
                high = second
        best = max(best, value((low + high) / 2))
    return best


def offset_line_targets(cases):
    """Circles about the origin within +-2 ulps of an offset line at distance |d|."""
    for exponent in EXPONENTS:
        s = 2.0 ** exponent
        for direction, distance in (((1.0, 2.0), 0.75 * s), ((2.0, -1.0), -1.25 * s),
                                    ((3.0, 4.0), 5 * s)):
            basis = ((0.0, 0.0), (direction[0] * s, direction[1] * s))
            target = ('offset', Offset.line(basis[0], basis[1], False, distance))
            spec = offset_spec(distance, line_spec(basis[0], basis[1], False))
            for steps in range(-2, 3):
                radius = shift(abs(distance), steps)
                source = ('conic', ConicSource(Conic.circle((0.0, 0.0), radius), (1.0, 0.0)))
                cases.append(('circle_offset_line_tangent', 'segment',
                              circle_spec((0.0, 0.0), radius), source, [spec], [target]))


def offset_line_sources(cases):
    """An offset line source within +-2 ulps of tangency to circles and an ellipse offset."""
    for exponent in EXPONENTS:
        s = 2.0 ** exponent
        basis, distance = ((-2 * s, -2 * s), (2 * s, 2 * s)), 0.5 * s
        offset = Offset.line(basis[0], basis[1], True, distance)
        spec = offset_spec(distance, line_spec(basis[0], basis[1], True))
        for steps in range(-2, 3):
            radius = shift(0.5 * s, steps)
            target = ('conic', Conic.circle((0.0, 0.0), radius))
            cases.append(('offset_line_source_circle', 'segment', spec, ('offsetsource', offset),
                          [circle_spec((0.0, 0.0), radius)], [target]))
        # The basis y = y0 offset by d (right of +x travel) is y = y0 - d; the grown
        # ellipse below has its top at y = s.
        ellipse = Offset.ellipse((0.25 * s, -0.5 * s), (1.0, 0.0), (2 * s, s), 0.5 * s)
        ellipse_target = offset_spec(0.5 * s, ellipse_spec((0.25 * s, -0.5 * s), (1.0, 0.0),
                                                           (2 * s, s)))
        for steps in range(-2, 3):
            height = shift(1.25 * s, steps)
            line = Offset.line((-4 * s, height), (4 * s, height), True, 0.25 * s)
            cases.append(('offset_line_source_ellipse_offset', 'segment',
                          offset_spec(0.25 * s, line_spec((-4 * s, height), (4 * s, height), True)),
                          ('offsetsource', line), [ellipse_target], [('offset', ellipse)]))


def offset_curve_sources(generator, cases):
    """Spline and ellipse offset sources within +-2 ulps of tangency to lines."""
    for _ in range(3):
        poles = [(0.25 * i, generator.uniform(-1, 1)) for i in range(6)]
        offset = Offset.spline(3, poles, KNOTS, (0.0, 1.5), 0.125)
        spec = offset_spec(0.125, spline_spec(3, poles, KNOTS, (0.0, 1.5)))
        top = offset_top(offset)
        for steps in range(-2, 3):
            height = shift(top, steps)
            target = ('line', (-1.0, height), (3.0, height), False)
            cases.append(('offset_spline_source_line_top', 'segment', spec,
                          ('offsetsource', offset), [line_spec((-1.0, height), (3.0, height),
                                                               False)], [target]))
    for exponent in EXPONENTS:
        s = 2.0 ** exponent
        offset = Offset.ellipse((0.0, 0.0), (1.0, 0.0), (2 * s, s), s)
        offset.extra['frame'] = ellipse_frame((0.0, 0.0), (1.0, 0.0), (2 * s, s))
        spec = offset_spec(s, ellipse_spec((0.0, 0.0), (1.0, 0.0), (2 * s, s)))
        lines = [((-4 * s, shift(2 * s, steps)), (4 * s, shift(2 * s, steps))) for steps in
                 range(-2, 3)] + [((-4 * s, 0.0), (4 * s, 0.0)), ((0.0, -4 * s), (0.0, 4 * s))]
        for start, end in lines:
            cases.append(('offset_ellipse_source_line', 'segment', spec, ('offsetsource', offset),
                          [line_spec(start, end, False)], [('line', start, end, False)]))


def circle_offset_targets(cases):
    """Circle offsets are concentric circles: lines within +-2 ulps of the grown top."""
    for exponent in EXPONENTS:
        s = 2.0 ** exponent
        # The clockwise arc from 135 to 45 degrees passes the top; its right normal
        # points to the center, so a negative distance grows it.
        for sector, distance, radius in ((None, 0.25 * s, 0.75 * s),
                                         (((-1.0, 1.0), (1.0, 1.0), False), -0.25 * s, 0.75 * s)):
            grown = F(radius) + (F(distance) if sector is None else -F(distance))
            target = ('conic', Conic.circle((0.0, 0.0), grown, sector))
            spec = offset_spec(distance, circle_spec((0.0, 0.0), radius, sector))
            for steps in range(-2, 3):
                height = shift(s, steps)
                start, end = (-2 * s, height), (2 * s, height)
                # The line source as a zero offset of itself: parameter t of A + t (B - A).
                source = ('offsetsource', Offset.line(start, end, True, 0.0))
                cases.append(('line_offset_circle_top', 'segment', line_spec(start, end, True),
                              source, [spec], [target]))


def periodic_splines(generator, cases):
    """Native periodic cubics: a circle near the top as a target, a line near the top
    against a periodic source."""
    for _ in range(2):
        poles = [(math.cos(k * math.pi / 3) * 2 + generator.uniform(-0.25, 0.25),
                  math.sin(k * math.pi / 3) * 2 + generator.uniform(-0.25, 0.25)) for k in range(6)]
        knots, mults = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0], [1] * 7
        flat, wrapped = unwrap(3, poles, knots, mults)
        pieces = spline_pieces(3, wrapped, flat, (0.0, 6.0))
        spec = periodic_spec(3, poles, knots, mults, (0.0, 6.0))
        _, sample = max((float(point_y(pieces, F(k, 256))), k) for k in range(6 * 256))
        top, at = spline_top(pieces, sample / 256)
        for steps in range(-2, 3):
            height = shift(top, steps)
            cases.append(('periodic_source_line_top', 'segment', spec, ('spline', pieces, (0, 6)),
                          [line_spec((-4.0, height), (4.0, height), False)],
                          [('line', (-4.0, height), (4.0, height), False)]))
        x = float(point_x(pieces, F(at)))
        for steps in range(-2, 3):
            center = (x, shift(top + 0.0625, steps * 4))
            source = ('conic', ConicSource(Conic.circle(center, 0.0625), (1.0, 0.0)))
            cases.append(('circle_source_periodic_target', 'segment', circle_spec(center, 0.0625),
                          source, [spec], [('spline', 3, wrapped, flat, (0.0, 6.0))]))


def spline_top(pieces, guess):
    """Golden-section refinement of a sampled maximum of y (binary64 height, parameter)."""
    low, high = guess - 1 / 256, guess + 1 / 256
    height = lambda u: float(point_y(pieces, F(u)))
    for _ in range(80):
        first, second = low + (high - low) / 3, high - (high - low) / 3
        if height(first) < height(second):
            low = first
        else:
            high = second
    middle = (low + high) / 2
    return height(middle), middle


def unwrap(degree, poles, knots, mults):
    """Periodic flat knots by periodic indexing: base[j mod N] + period floor(j / N)."""
    base = [F(k) for k, m in zip(knots[:-1], mults[:-1]) for _ in range(m)]
    count, period = len(base), F(knots[-1]) - F(knots[0])
    lead = degree + 1 - mults[0]
    indices = range(-lead, count + mults[-1] + lead)
    flat = [float(base[j % count] + period * (j // count)) for j in indices]
    wrapped = [poles[k % len(poles)] for k in range(len(flat) - degree - 1)]
    return flat, wrapped


def point_y(pieces, u):
    from oracle_exact import evaluate
    for piece in pieces:
        if piece['range'][0] <= u <= piece['range'][1]:
            return evaluate(piece['y'], u)
    return -math.inf


def point_x(pieces, u):
    from oracle_exact import evaluate
    for piece in pieces:
        if piece['range'][0] <= u <= piece['range'][1]:
            return evaluate(piece['x'], u)
    raise ValueError('parameter outside the spline')


def main():
    generator = random.Random(SEED)
    cases = []
    ellipse_offset_targets(cases)
    spline_offset_targets(generator, cases)
    offset_line_targets(cases)
    offset_line_sources(cases)
    offset_curve_sources(generator, cases)
    circle_offset_targets(cases)
    periodic_splines(generator, cases)
    for case, (kind, domain, source_spec, source, specs, targets) in enumerate(cases):
        emit(case, kind, domain, source_spec, source, specs, targets)
    print(f'{len(cases)} cases', file=sys.stderr)


if __name__ == '__main__':
    main()
