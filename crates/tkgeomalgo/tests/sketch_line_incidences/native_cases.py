# SPDX-License-Identifier: GPL-2.0-only
"""Ordinary-range sketch line incidence corpus shared by native OCCT and Rust.

Rows: id;kind;source;target. Numbers are binary64 bit patterns. Targets use
circle:cx,cy,r[,sx,sy,ex,ey,ccw], ellipse:cx,cy,ax,ay,ra,rb,
segment:x0,y0,x1,y1 and spline:degree:poles:flat_knots:bounds. Spline recipes
are the retained Go fit and control captures of ../sketch_spline.

Usage: python3 native_cases.py > native-cases.csv
"""
import os
import struct

HERE = os.path.dirname(os.path.abspath(__file__))


def hexbits(value):
    return format(struct.unpack('<Q', struct.pack('<d', float(value)))[0], '016x')


def numbers(values):
    return ','.join(hexbits(value) for value in values)


def go_recipes(name):
    recipes = {}
    with open(os.path.join(HERE, '..', 'sketch_spline', name)) as source:
        for row in source:
            case, kind, _, *values = row.strip().split(',')
            recipe = recipes.setdefault(int(case), {'poles': [], 'knots': []})
            if kind == 'shape':
                recipe['degree'] = int(values[0])
            elif kind == 'pole':
                recipe['poles'].append((float(values[0]), float(values[1])))
            elif kind == 'knot':
                recipe['knots'].append(float(values[0]))
    return [recipes[case] for case in sorted(recipes)]


def spline_spec(recipe):
    degree, poles, knots = recipe['degree'], recipe['poles'], recipe['knots']
    bounds = (knots[degree], knots[len(poles)])
    return 'spline:%d:%s:%s:%s' % (degree, numbers(c for p in poles for c in p),
                                   numbers(knots), numbers(bounds))


LINES = [((-3.0, 0.3), (4.0, 0.3)), ((1.1, -3.0), (1.1, 3.0)), ((-2.0, -2.5), (3.0, 2.5))]


def analytic_cases():
    unit = 'circle:' + numbers([0, 0, 1])
    arc = 'circle:' + numbers([0, 0, 1, 1, 0, 0, 1]) + ',1'
    ellipse = 'ellipse:' + numbers([0.5, 0.25, 2, 1, 3, 1])
    yield 'circle', ((-2.0, 0.0), (2.0, 0.0)), unit
    yield 'circle', ((-2.0, 0.5), (2.0, 0.5)), unit
    yield 'circle_tangent', ((-2.0, 1.0), (2.0, 1.0)), unit
    yield 'circle_miss', ((-2.0, 1.5), (2.0, 1.5)), unit
    yield 'circle', ((-3.0, -2.0), (3.0, 1.0)), 'circle:' + numbers([1.25, -0.5, 2])
    yield 'arc', ((-1.0, -1.0), (2.0, 2.0)), arc
    yield 'arc', ((-2.0, 0.5), (2.0, 0.5)), arc
    yield 'arc_miss', ((-2.0, -0.5), (2.0, -0.5)), arc
    for start, end in LINES:
        yield 'ellipse', (start, end), ellipse
    yield 'segment', ((0.0, 0.0), (4.0, 0.0)), 'segment:' + numbers([1, -1, 1.5, 1])
    yield 'segment_end', ((0.0, 0.0), (4.0, 0.0)), 'segment:' + numbers([2, 0, 2.5, 3])
    yield 'segment_parallel', ((0.0, 0.0), (4.0, 0.0)), 'segment:' + numbers([0, 1, 4, 1])


def spline_cases():
    for name, kind in (('go-fit-oracle.csv', 'fit'), ('go-control-oracle.csv', 'control')):
        for index, recipe in enumerate(go_recipes(name)):
            for start, end in LINES:
                yield '%s_%d' % (kind, index), (start, end), spline_spec(recipe)


def main():
    cases = list(analytic_cases()) + list(spline_cases())
    for case, (kind, (start, end), target) in enumerate(cases):
        print(';'.join([str(case), kind, numbers(start + end), target]))


if __name__ == '__main__':
    main()
