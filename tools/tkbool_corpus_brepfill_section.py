"""Deterministic requests for BRepFill_Section and BRepFill_SectionPlacement
(tools/oracle/tkbool_probe_brepfill_section.cxx; the records are documented there).

Sections of a wire, a located wire, a wire of REVERSED edges, a circle, a vertex and a
face (refused); placements of square, circle and point sections on straight, bent and
curved paths, located by the nearest point or by a path vertex, with and without
contact and correction.
"""
from __future__ import annotations
from tkbool_corpus_brepfill_generator import circle_wire, polyline_wire, square
from tkbool_corpus_brepfill_location_law import arc_after_segment
from tktopalgo_corpus_brepcheck_shapes import square as square_face
from tktopalgo_shape_program import ShapeProgram


def located_wire(p: ShapeProgram) -> int:
    """A square wire located by a translation."""
    return p.located(polyline_wire(p, square(0.0, 1.0), True), p.location(0, 0.5, 0.25, 1.0))


PROFILES = {
    'square': lambda p: polyline_wire(p, square(0.0, 1.0), True),
    'located': located_wire,
    'reversed': lambda p: polyline_wire(p, square(0.0, 1.0), True, True),
    'circle': lambda p: circle_wire(p, (0, 0, 0), 0.5),
    'vertex': lambda p: p.vertex((1.0, 2.0, 3.0)),
    'face': lambda p: square_face(p, 1.0).faces[0],
}


def append_section_requests(requests: list[str]) -> None:
    """Every profile with and without a location vertex and flags."""
    for name, builder in sorted(PROFILES.items()):
        p = ShapeProgram()
        profile = builder(p)
        v = p.vertex((0.0, 0.0, 0.0))
        requests.append(f'bf_section {p.tokens()} {profile} -1 0 0')
        requests.append(f'bf_section {p.tokens()} {profile} {v} 1 1')


PLACEMENTS = [
    # (path builder, section builder, located by the path vertex rank or -1)
    (lambda p: polyline_wire(p, [(0, 0, 0), (0, 0, 3)], False), lambda p: polyline_wire(p, square(0.0, 1.0), True)),
    (lambda p: polyline_wire(p, [(0, 0, 0), (0, 0, 3)], False), lambda p: polyline_wire(p, square(1.5, 1.0, 0.3, (0.2, 0.1)), True)),
    (lambda p: polyline_wire(p, [(0, 0, 0), (0, 0, 2), (1, 0, 3)], False), lambda p: circle_wire(p, (0.1, 0, 2.0), 0.3)),
    (arc_after_segment, lambda p: polyline_wire(p, [(1, -0.2, -0.2), (1, 0.2, -0.2), (1, 0.2, 0.2), (1, -0.2, 0.2)], True)),
    (lambda p: polyline_wire(p, [(0, 0, 0), (0, 0, 3)], False), lambda p: p.vertex((0.5, 0.0, 1.0))),
]


def append_placement_requests(requests: list[str]) -> None:
    """Every placement free, by the first path vertex, and with contact/correction."""
    for path_builder, section_builder in PLACEMENTS:
        p = ShapeProgram()
        path = path_builder(p)
        section = section_builder(p)
        first = 0
        for vertex, contact, correction in ((-1, 0, 0), (-1, 1, 0), (-1, 0, 1), (-1, 1, 1), (first, 0, 0), (first, 1, 1)):
            requests.append(f'bf_section_placement {p.tokens()} {path} {section} {vertex} {contact} {correction}')


def append_section_family_requests(requests: list[str]) -> None:
    """The section and placement families."""
    append_section_requests(requests)
    append_placement_requests(requests)
