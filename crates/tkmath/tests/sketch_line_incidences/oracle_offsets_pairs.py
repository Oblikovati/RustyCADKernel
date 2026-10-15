# SPDX-License-Identifier: GPL-2.0-only
"""Offset curves and their contact rows for the offset oracle (ADR 0080).

Offsets are built from basis pieces: a line piece, de Boor interpolated spline
spans, or an ellipse in two tangent-half-angle charts u = tan((t - t0) / 2),
t0 = 0 and pi, over [-1, 1] (counterclockwise; a clockwise arc flips the
distance). Each function returns rows (order_key, fields) with fields
target, left, right, source_position, target_position, transition, xlo..yhi.
"""
from fractions import Fraction as F
import math
from oracle_curves_exact import Conic, line_implicit, line_pieces, position, spline_pieces, trim
from oracle_curves_pairs import ConicSource
from oracle_offsets_exact import (OffsetPiece, compose_conic, compose_line, compose_offset_line,
                                  counts, merge, refined, surd_add, surd_roots, surd_scale,
                                  surd_sign_root, transition)
from oracle_exact import bracket, evaluate


# ---------- offset curves ----------
class Offset:
    """An offset: kind line/ellipse/spline, its pieces and its basis description."""

    def __init__(self, kind, pieces, distance, extra):
        self.kind, self.distance, self.extra = kind, F(distance), extra
        self.pieces = [OffsetPiece(piece, distance) for piece in pieces]
        self.cyclic = kind == 'ellipse'

    @staticmethod
    def line(start, end, bounded, distance):
        extra = {'line': line_implicit((start, end)), 'bounded': bounded}
        return Offset('line', line_pieces((start, end), bounded), distance, extra)

    @staticmethod
    def spline(degree, poles, knots, bounds, distance):
        return Offset('spline', spline_pieces(degree, poles, knots, bounds), distance,
                      {'bounds': tuple(map(F, bounds))})

    @staticmethod
    def ellipse(center, axis, radii, distance):
        ax, ay = F(axis[0]), F(axis[1])
        squared = ax * ax + ay * ay
        root = F(math.isqrt(squared.numerator), math.isqrt(squared.denominator))
        assert root * root == squared, 'ellipse offsets need a rational axis length'
        unit = (ax / root, ay / root)
        major = (unit[0] * F(radii[0]), unit[1] * F(radii[0]))
        minor = (-unit[1] * F(radii[1]), unit[0] * F(radii[1]))
        center = tuple(map(F, center))
        pieces = [chart(center, major, minor, side, index) for index, side in enumerate((1, -1))]
        return Offset('ellipse', pieces, distance, {'center': center})


def chart(center, major, minor, side, index):
    """C (1 + u^2) + side (major (1 - u^2) + minor 2u) over w = 1 + u^2."""
    coordinate = lambda axis: trim([center[axis] + side * major[axis], 2 * side * minor[axis],
                                    center[axis] - side * major[axis]])
    return {'x': coordinate(0), 'y': coordinate(1), 'w': [F(1), F(0), F(1)],
            'range': (F(-1), F(1)), 'index': index}


def offset_line_implicit(offset):
    return (offset.extra['line'], offset.distance)


# ---------- implicit sources against offset targets ----------
def implicit_offset_target(source, target, index):
    """A line, offset-line or conic source against offset pieces: roots in u."""
    roots = []
    for piece in target.pieces:
        equation, filters = source_equation(source, piece)
        for root in surd_roots(equation, piece.g, piece.range, filters):
            root['piece'] = piece
            roots.append(root)
    rows = []
    for root in merge(roots, target.cyclic):
        target_position = offset_position(target, root)
        if target_position is None:
            continue
        piece = root['piece']
        u, _ = refined(root)
        point = piece.point(u)
        key, source_position = source_place(source, piece, root, point)
        kind = transition(root, 'E') if target_position == 'I' else 'E'
        fields = [index] + counts(root) + [source_position, target_position, kind]
        rows.append((key, fields + coordinates(point)))
    return rows


def source_equation(source, piece):
    kind = source[0]
    if kind == 'line':
        return compose_line(line_implicit(source[1]), piece)[0], []
    if kind == 'offsetsource':
        return compose_offset_line(offset_line_implicit(source[1]), piece)
    return compose_conic(source[1].conic, piece), []


def offset_position(target, root):
    if target.kind == 'ellipse':
        return 'I'
    if target.kind == 'line' and not target.extra['bounded']:
        return 'I'
    bounds = (F(0), F(1)) if target.kind == 'line' else target.extra['bounds']
    return position(root, *bounds)


def source_place(source, piece, root, point):
    """Order key and exact source position of a contact point on an implicit source."""
    if source[0] == 'conic':
        conic = source[1]
        chart = sweep_chart(conic, piece, root)
        direction = tuple(F(point[axis]) - conic.conic.center[axis] for axis in (0, 1))
        return (chart, direction, conic.ccw), 'I'
    line = line_implicit(source[1]) if source[0] == 'line' else source[1].extra['line']
    _, along = compose_line(line, piece)
    dx, dy = line['direction']
    squared = dx * dx + dy * dy
    value = along_value(along, piece, root) / squared
    first = surd_sign_root(along, piece.g, root)
    second = surd_sign_root(surd_add(along, surd_scale(piece.w, -squared)), piece.g, root)
    place = 'B' if first < 0 else ('S' if first == 0 else
                                   ('I' if second < 0 else ('E' if second == 0 else 'A')))
    return ('parameter', value, False), place


def along_value(along, piece, root):
    from oracle_offsets_exact import surd_decimal
    u, _ = refined(root)
    return F(surd_decimal(along, piece.g, u) / surd_decimal(piece.w, piece.g, u))


def sweep_chart(source, piece, root):
    qx, qy = piece.relative(source.conic.center)
    rx, ry = source.reference
    across = surd_add(surd_scale(qy, rx), surd_scale(qx, -ry))
    side = surd_sign_root(across, piece.g, root) * (1 if source.ccw else -1)
    if side != 0:
        return 1 if side > 0 else 3
    along = surd_sign_root(surd_add(surd_scale(qx, rx), surd_scale(qy, ry)), piece.g, root)
    return 0 if along > 0 else 2


def coordinates(point):
    return [bracket(F(value)) for value in point]


# ---------- offset sources against implicit targets ----------
def offset_source_implicit(source, target, index):
    """Offset source pieces against a line, conic or offset-line target: roots in u."""
    roots = []
    for piece in source.pieces:
        equation, filters = target_equation(target, piece)
        for root in surd_roots(equation, piece.g, piece.range, filters):
            root['piece'] = piece
            roots.append(root)
    rows = []
    for root in merge(roots, source.cyclic):
        piece = root['piece']
        target_position = implicit_position(target, piece, root)
        if target_position is None:
            continue
        u, exact = refined(root)
        point = piece.point(u)
        key, source_position = offset_place(source, piece, root, u, exact)
        kind = transition(root, 'O') if target_position == 'I' else 'E'
        fields = [index] + counts(root) + [source_position, target_position, kind]
        rows.append((key, fields + coordinates(point)))
    return rows


def target_equation(target, piece):
    kind = target[0]
    if kind == 'line':
        return compose_line(line_implicit((target[1], target[2])), piece)[0], []
    if kind == 'offset':
        return compose_offset_line(offset_line_implicit(target[1]), piece)
    return compose_conic(target[1], piece), []


def implicit_position(target, piece, root):
    """Exact target position: a bounded line or offset line by its own parameter."""
    if target[0] == 'conic':
        return 'I'
    if target[0] == 'line':
        line, bounded = line_implicit((target[1], target[2])), target[3]
    else:
        line, bounded = target[1].extra['line'], target[1].extra['bounded']
    if not bounded:
        return 'I'
    _, along = compose_line(line, piece)
    dx, dy = line['direction']
    first = surd_sign_root(along, piece.g, root)
    second = surd_sign_root(surd_add(along, surd_scale(piece.w, -(dx * dx + dy * dy))), piece.g,
                            root)
    if first < 0 or second > 0:
        return None
    return 'S' if first == 0 else ('E' if second == 0 else 'I')


def offset_place(source, piece, root, u, exact):
    if source.kind == 'ellipse':
        basis = piece.basis
        conic = source.extra['frame']
        chart = conic.chart(root, basis['x'], basis['y'], basis['w'])
        weight = evaluate(basis['w'], u)
        point = (evaluate(basis['x'], u) / weight, evaluate(basis['y'], u) / weight)
        direction = (point[0] - conic.conic.center[0], point[1] - conic.conic.center[1])
        return (chart, direction, conic.ccw), 'I'
    bounds = (F(0), F(1)) if source.kind == 'line' else source.extra['bounds']
    return ('parameter', u, exact), position(root, *bounds)


def ellipse_frame(center, axis, radii):
    """The basis conic sweep frame of an ellipse offset source (full ellipse)."""
    return ConicSource(Conic.ellipse(center, axis, radii), axis)
