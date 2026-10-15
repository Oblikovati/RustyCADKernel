"""Deterministic requests for BRepOffsetAPI_ThruSections
(tools/oracle/tkoffset_probe_brepoffsetapi_thru_sections.cxx; the record is documented there).

The section families of the BRepFill_CompatibleWires corpus
(tools/tkbool_corpus_brepfill_compatible.py) and the GTest shapes of BRepOffset_MakeOffset_Test
(a circle to a rectangle, an ellipse-like B-spline, polygons; vertex apexes added by
AddVertex), lofted as shells and solids, ruled and smoothed; the squares also without the
compatibility check, with an immutable input, with smoothing, the other parametrizations,
C1 continuity, a lower maximum degree and a coarse precision (three iterations); wrong
punctual sections (an inner apex, two apexes) give WrongUsage.
"""
from __future__ import annotations
from tkbool_corpus_brepfill_compatible import FAMILIES, polygon
from tkbool_corpus_brepfill_generator import circle_wire, point_wire, polyline_wire, square
from tktopalgo_shape_program import ShapeProgram

WIRE, VERTEX = 0, 1
# check smoothing partype continuity maxdeg mutable
DEFAULTS = (1, 0, 0, 4, -1, 1)


def request(build, solid: int, ruled: int, pres3d: float = 1.0e-6, settings=DEFAULTS) -> str:
    """One loft of the `(kind, index)` sections that `build(p)` returns."""
    p = ShapeProgram()
    sections = build(p)
    body = ' '.join(f'{kind} {index}' for kind, index in sections)
    options = ' '.join(map(str, settings))
    return f'oa_thru_sections {p.tokens()} {solid} {ruled} {pres3d!r} {options} {len(sections)} {body}'


def wires(family):
    """A family of wires as AddWire sections."""
    return lambda p: [(WIRE, w) for w in family(p)]


def rectangle(z: float, width: float, height: float):
    """The corners of a rectangle centred on the Z axis at height `z`."""
    w, h = width/2, height/2
    return [(-w, -h, z), (w, -h, z), (w, h, z), (-w, h, z)]


GTEST_FAMILIES = {
    'circle_rectangle': lambda p: [circle_wire(p, (0, 0, 0), 5.0), polyline_wire(p, rectangle(10.0, 8.0, 6.0), True)],
    'circle_hexagon': lambda p: [circle_wire(p, (0, 0, 0), 5.0), polyline_wire(p, polygon(8.0, 4.0, 6), True)],
    'three_sections': lambda p: [circle_wire(p, (0, 0, 0), 5.0), polyline_wire(p, rectangle(5.0, 8.0, 6.0), True),
                                 circle_wire(p, (0, 0, 10.0), 3.0)],
    'circle_triangle': lambda p: [circle_wire(p, (0, 0, 0), 4.0), polyline_wire(p, polygon(6.0, 3.0, 3), True)],
}


def apex_vertex(p: ShapeProgram):
    """A square with an apex vertex above it (AddVertex)."""
    return [(WIRE, polyline_wire(p, square(0.0, 2.0), True)), (VERTEX, p.vertex((0.0, 0.0, 1.5)))]


def apex_both(p: ShapeProgram):
    """Apex vertices on both sides of a square."""
    return [(VERTEX, p.vertex((0.0, 0.0, -1.5))), (WIRE, polyline_wire(p, square(0.0, 2.0), True)),
            (VERTEX, p.vertex((0.0, 0.0, 1.5)))]


def inner_apex(p: ShapeProgram):
    """An inner punctual section: WrongUsage."""
    return [(WIRE, polyline_wire(p, square(0.0, 2.0), True)), (WIRE, point_wire(p, (0, 0, 1.0))),
            (WIRE, polyline_wire(p, square(2.0, 2.0), True))]


def two_apexes(p: ShapeProgram):
    """Two punctual sections only: WrongUsage."""
    return [(VERTEX, p.vertex((0.0, 0.0, 0.0))), (VERTEX, p.vertex((0.0, 0.0, 1.0)))]


def squares3(p: ShapeProgram):
    """Three turned squares."""
    return [(WIRE, polyline_wire(p, square(0.0, 2.0), True)), (WIRE, polyline_wire(p, square(1.0, 2.0, 0.3), True)),
            (WIRE, polyline_wire(p, square(2.5, 1.5, 0.6), True))]


SETTINGS = [
    (0, 0, 0, 4, -1, 1), (1, 0, 0, 4, -1, 0), (1, 1, 0, 4, -1, 1), (1, 0, 1, 4, -1, 1),
    (1, 0, 2, 4, -1, 1), (1, 0, 0, 2, -1, 1), (1, 0, 0, 4, 4, 1),
]


def append_thru_sections_requests(requests: list[str]) -> None:
    """Every family as shell and solid, ruled and smoothed, then the settings."""
    families = {**{name: wires(f) for name, f in FAMILIES.items()},
                **{name: wires(f) for name, f in GTEST_FAMILIES.items()},
                'apex_vertex': apex_vertex, 'apex_both': apex_both, 'inner_apex': inner_apex,
                'two_apexes': two_apexes, 'squares3': squares3}
    for name in sorted(families):
        for solid in (0, 1):
            for ruled in (0, 1):
                requests.append(request(families[name], solid, ruled))
    for settings in SETTINGS:
        for name in ('squares3', 'three_sections', 'apex_square_triangle'):
            requests.append(request(families[name], 1, 0, 1.0e-6, settings))
    for name in ('squares3', 'circle_rectangle'):
        requests.append(request(families[name], 1, 0, 1.0e-2))
        requests.append(request(families[name], 1, 1, 1.0e-2))
    requests.append(request(families['squares3'], 1, 0, 1.0e-6, (1, 0, 0, 4, 1, 1)))
