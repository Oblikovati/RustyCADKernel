# SPDX-License-Identifier: GPL-2.0-only
"""Ordinary-range conic- and spline-source corpus shared by native OCCT and Rust.

Rows: id;kind;source;target, both as curve specs: circle:cx,cy,r[,sx,sy,ex,ey,ccw],
ellipse:cx,cy,ax,ay,ra,rb, segment:x0,y0,x1,y1, line:x0,y0,x1,y1 (unbounded) and
spline:degree:poles:flat_knots:bounds. Spline recipes are the retained Go fit and
control captures of ../sketch_spline. Sources are bounded: full conics, arcs and
splines; arcs and contacts avoid their ends so native angle trims agree.

Usage: python3 native_curve_cases.py > native-curve-cases.csv
"""
from native_cases import go_recipes, numbers, spline_spec

CONIC_SOURCES = [
    ('circle_source', 'circle:' + numbers([0.25, -0.5, 2])),
    ('arc_source', 'circle:' + numbers([0, 0, 1.5, 1, 0.2, -0.3, 1]) + ',1'),
    ('clockwise_arc_source', 'circle:' + numbers([0.5, 0.5, 1, 0, 1, 1, 0]) + ',0'),
    ('ellipse_source', 'ellipse:' + numbers([0.5, 0.25, 2, 1, 3, 1])),
]
LINE_TARGETS = [
    'segment:' + numbers([-4, 0.3, 4, 0.7]),
    'line:' + numbers([0.1, -3, 0.4, 3]),
    'segment:' + numbers([-3, -2.5, 3, 2]),
]
CONIC_TARGETS = [
    'circle:' + numbers([1, 0.5, 1.25]),
    'circle:' + numbers([-1, -1, 1.75]),
    'circle:' + numbers([0, 0.75, 1.1, -1, 0.1, 0.5, 1]) + ',1',
    'ellipse:' + numbers([-0.25, 0.5, -1, 2, 2.2, 0.8]),
]


def conic_cases(recipes):
    for kind, source in CONIC_SOURCES:
        for target in LINE_TARGETS + CONIC_TARGETS:
            yield kind, source, target
        for recipe in recipes[:2]:
            yield kind + '_spline', source, spline_spec(recipe)


def spline_cases(recipes):
    others = [LINE_TARGETS[0], CONIC_TARGETS[0], CONIC_TARGETS[2], CONIC_TARGETS[3]]
    for index, recipe in enumerate(recipes):
        for target in others:
            yield 'spline_source_%d' % index, spline_spec(recipe), target
        # A dyadic shift keeps partners off the shared fit stations, whose exact
        # coincidence would depend on fit residuals of order 1e-16.
        partner = shifted(recipes[(index + 3) % len(recipes)], (0.3125, 0.1875))
        yield 'spline_pair_%d' % index, spline_spec(recipe), spline_spec(partner)


def shifted(recipe, offset):
    poles = [(x + offset[0], y + offset[1]) for x, y in recipe['poles']]
    return {'degree': recipe['degree'], 'poles': poles, 'knots': recipe['knots']}


def main():
    recipes = go_recipes('go-fit-oracle.csv') + go_recipes('go-control-oracle.csv')
    cases = list(conic_cases(recipes)) + list(spline_cases(recipes))
    for case, (kind, source, target) in enumerate(cases):
        print(';'.join([str(case), kind, source, target]))


if __name__ == '__main__':
    main()
