# SPDX-License-Identifier: GPL-2.0-only
"""Ordinary-range offset and periodic-spline corpus shared by native OCCT and Rust.

Rows: id;kind;source;target as curve specs (see native_curve_cases.py), plus
offset:<distance>:<basis spec> (Geom2d_OffsetCurve of the basis, trimmed like it)
and periodic:degree:poles:knots:multiplicities:bounds (a periodic
Geom2d_BSplineCurve; multiplicities are decimal integers). Offsets cover
segments, a circle, counterclockwise and clockwise arcs,
axis-aligned and (3, 4) ellipses and the C1 Go fit recipes; contacts avoid
tangency and curve ends, so native solver output is comparable.

Usage: python3 native_offset_cases.py > native-offset-cases.csv
"""
from native_cases import go_recipes, numbers, spline_spec

LINES = [
    'segment:' + numbers([-4, 0.3, 4, 0.7]),
    'line:' + numbers([0.1, -3, 0.4, 3]),
    'segment:' + numbers([-3, -2.5, 3, 2]),
]
CIRCLE = 'circle:' + numbers([1, 0.5, 1.25])
ELLIPSE = 'ellipse:' + numbers([-0.25, 0.5, -1, 2, 2.2, 0.8])
ARC = 'circle:' + numbers([0, 0.75, 1.1, -1, 0.1, 0.5, 1]) + ',1'


def offset(distance, basis):
    return 'offset:' + numbers([distance]) + ':' + basis


OFFSET_LINE = offset(0.5, 'segment:' + numbers([-3, -1, 3, 1]))
OFFSET_CURVES = [
    ('offset_segment', OFFSET_LINE),
    ('offset_long_segment', offset(-0.75, 'segment:' + numbers([-1, -6, 2, 6]))),
    ('offset_circle', offset(0.5, 'circle:' + numbers([0.25, -0.5, 1.5]))),
    ('offset_ccw_arc', offset(0.25, 'circle:' + numbers([0, 0, 1.5, 1, 0.2, -0.3, 1]) + ',1')),
    ('offset_cw_arc', offset(0.25, 'circle:' + numbers([0.5, 0.5, 1, 0, 1, 1, 0]) + ',0')),
    ('offset_ellipse', offset(0.5, 'ellipse:' + numbers([0.5, 0.25, 1, 0, 2, 1]))),
    ('offset_rotated_ellipse', offset(-0.25, 'ellipse:' + numbers([0, 0, 3, 4, 2.5, 1.25]))),
]
PERIODIC = 'periodic:3:%s:%s:%s:%s' % (
    numbers([2, 0.25, 1, 2, -1, 1.75, -2, -0.25, -1, -2, 1, -1.75]),
    numbers([0, 1, 2, 3, 4, 5, 6]), ','.join(['1'] * 7), numbers([0, 6]))


def offset_targets(recipes):
    """Line and conic sources against every offset target."""
    sources = [('segment_source', LINES[0]), ('circle_source', 'circle:' + numbers([0.25, 0, 2])),
               ('ellipse_source', 'ellipse:' + numbers([0.5, 0.25, 3, 4, 2.5, 1.25]))]
    splines = [('offset_fit_%d' % index, offset(distance, spline_spec(recipe)))
               for index, recipe in enumerate(recipes) for distance in (0.125, -0.1)]
    # Conic sources meet only the open recipes at one distance: closed recipes give
    # degree-20 norms with 2700-bit coefficients, slow to isolate in debug builds.
    light = [pair for pair in splines[:6:2]]
    for source_kind, source in sources:
        for kind, target in OFFSET_CURVES + (splines if source_kind == 'segment_source' else light):
            yield source_kind + '_' + kind, source, target


def offset_sources(recipes):
    """Offset sources against lines, conics and a line offset."""
    targets = LINES + [CIRCLE, ELLIPSE, ARC, OFFSET_LINE]
    sources = [OFFSET_CURVES[0], OFFSET_CURVES[2], OFFSET_CURVES[3], OFFSET_CURVES[4],
               OFFSET_CURVES[5], OFFSET_CURVES[6]]
    sources += [('offset_fit_%d' % index, offset(0.125, spline_spec(recipe)))
                for index, recipe in enumerate(recipes[:3])]
    for kind, source in sources:
        # A coincident offset line is an uncertified overlap (a named failure).
        for target in (target for target in targets if target != source):
            yield kind + '_source', source, target


def periodic_cases():
    for target in LINES + [CIRCLE, ELLIPSE]:
        yield 'periodic_source', PERIODIC, target
    yield 'segment_periodic', LINES[0], PERIODIC
    yield 'circle_periodic', 'circle:' + numbers([0.25, 0, 1.5]), PERIODIC


def main():
    recipes = [recipe for recipe in go_recipes('go-fit-oracle.csv')]
    cases = list(offset_targets(recipes)) + list(offset_sources(recipes)) + list(periodic_cases())
    for case, (kind, source, target) in enumerate(cases):
        print(';'.join([str(case), kind, source, target]))


if __name__ == '__main__':
    main()
