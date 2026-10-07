"""Deterministic requests for BRepFill_ShapeLaw and BRepFill_NSections
(tools/oracle/tkbool_probe_brepfill_section_law.cxx; the records are documented there).

`bf_shape_law` builds the law of a vertex, closed squares (also with REVERSED edges),
open polylines whose ends meet or not, circles and B-spline wires, built or not, constant
or scaled by a Law_Linear; `bf_nsections` skins squares, circles, polylines, sections
with punctual ends and looping sections, built or not.
"""
from __future__ import annotations
from tkbool_corpus_brepfill_compatible import polygon
from tkbool_corpus_brepfill_generator import (bspline_wire, circle_wire, point_wire,
                                              polyline_wire, square)
from tktopalgo_shape_program import ShapeProgram

LAW = [1, 0.0, 1.0, 1.0, 2.0]
NO_LAW = [0]


def numbers(values) -> str:
    """The fields of a request; integers stay integers."""
    return ' '.join(str(v) if isinstance(v, int) else repr(float(v)) for v in values)


SHAPE_LAWS = {
    'vertex': lambda p: p.vertex((1.0, 2.0, 3.0), 1e-3),
    'square': lambda p: polyline_wire(p, square(0.0, 2.0), True),
    'square_reversed': lambda p: polyline_wire(p, square(0.0, 2.0), True, True),
    'open_polyline': lambda p: polyline_wire(p, [(0, 0, 0), (1, 0, 0), (2, 1, 0)], False),
    'closing_polyline': lambda p: polyline_wire(p, [(0, 0, 0), (1, 0, 0), (1, 1, 0), (0, 0, 0)], False),
    'circle': lambda p: circle_wire(p, (0, 0, 0), 1.5),
    'bspline': lambda p: bspline_wire(p, [(0, 0, 0), (1, 1, 0), (2, -1, 0), (3, 0, 0)]),
}

NSECTIONS = {
    'squares': lambda p: [polyline_wire(p, square(0.0, 2.0), True), polyline_wire(p, square(1.0, 1.0, 0.2), True)],
    'three_squares': lambda p: [polyline_wire(p, square(0.0, 2.0), True), polyline_wire(p, square(1.0, 1.0), True),
                                polyline_wire(p, square(2.0, 2.0, 0.3), True)],
    'circles': lambda p: [circle_wire(p, (0, 0, 0), 1.0), circle_wire(p, (0, 0, 1.0), 2.0), circle_wire(p, (0.5, 0, 2.0), 1.0)],
    'polylines': lambda p: [polyline_wire(p, [(0, 0, 0), (1, 0, 0), (2, 1, 0)], False),
                            polyline_wire(p, [(0, 0, 1), (1, 0.5, 1), (2, 1, 1.5)], False)],
    'apexes': lambda p: [point_wire(p, (0, 0, -1.0)), polyline_wire(p, square(0.0, 2.0), True), point_wire(p, (0, 0, 1.0))],
    'triangles': lambda p: [polyline_wire(p, polygon(0.0, 1.0, 3), True), polyline_wire(p, polygon(1.0, 1.5, 3, 0.4), True)],
    'looping': lambda p: looping(p),
}


def looping(p: ShapeProgram) -> list[int]:
    """Looping sections: the last section is the first one."""
    first = polyline_wire(p, square(0.0, 2.0), True)
    return [first, polyline_wire(p, square(1.0, 1.0, 0.5, (0.2, 0.0)), True), first]


def append_section_law_requests(requests: list[str]) -> None:
    """Every shape law built or not, with and without a law; every NSections built or not."""
    for name, builder in sorted(SHAPE_LAWS.items()):
        p = ShapeProgram()
        index = builder(p)
        for build in (1, 0):
            for law in (NO_LAW, LAW):
                requests.append(f'bf_shape_law {p.tokens()} {index} {build} {numbers(law)} 0.4')
    for name, builder in sorted(NSECTIONS.items()):
        p = ShapeProgram()
        wires = builder(p)
        fields = ' '.join([p.tokens(), str(len(wires)), *map(str, wires)])
        # Not built, D0 reaches a null law natively (a crash): only built laws here.
        for build, param in ((1, 0.3), (1, 1.0)):
            requests.append(f'bf_nsections {fields} {build} {param!r}')
