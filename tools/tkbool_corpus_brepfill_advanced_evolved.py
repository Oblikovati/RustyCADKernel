"""Deterministic requests for BRepFill_AdvancedEvolved
(tools/oracle/tkbool_probe_brepfill_advanced_evolved.cxx; the record is documented there).

A square and an L-shaped spine on z = 0 swept by open profiles in the XZ plane at the
corner (0, 0, 0) (vertical, slanted outward or inward, bent) and by a closed triangle;
as solids and as shells, in parallel or not.
"""
from __future__ import annotations
from tkbool_corpus_brepfill_generator import polyline_wire
from tktopalgo_shape_program import ShapeProgram

SPINES = {
    'square': [(0.0, 0.0, 0.0), (4.0, 0.0, 0.0), (4.0, 4.0, 0.0), (0.0, 4.0, 0.0)],
    'l_shape': [(0.0, 0.0, 0.0), (6.0, 0.0, 0.0), (6.0, 2.0, 0.0), (2.0, 2.0, 0.0), (2.0, 5.0, 0.0), (0.0, 5.0, 0.0)],
}
PROFILES = {
    'vertical': ([(0.0, 0.0, 0.0), (0.0, 0.0, 1.0)], False),
    'outward': ([(0.0, 0.0, 0.0), (-0.5, 0.0, 1.0)], False),
    'inward': ([(0.0, 0.0, 0.0), (0.5, 0.0, 1.0)], False),
    'bent': ([(0.0, 0.0, 0.0), (0.0, 0.0, 1.0), (0.5, 0.0, 1.5)], False),
    'triangle': ([(0.0, 0.0, 0.0), (0.5, 0.0, 0.0), (0.0, 0.0, 1.0)], True),
}


def request(spine: str, profile: str, solid: int = 1, parallel: int = 0) -> str:
    """One advanced evolved of a spine and a profile."""
    p = ShapeProgram()
    s = polyline_wire(p, SPINES[spine], True)
    points, closed = PROFILES[profile]
    w = polyline_wire(p, points, closed)
    return f'bf_advanced_evolved {p.tokens()} {s} {w} 1e-07 {solid} {parallel}'


def append_advanced_evolved_requests(requests: list[str]) -> None:
    """Every spine and profile as a solid; the shells and the parallel mode on the
    square."""
    for spine in sorted(SPINES):
        for profile in sorted(PROFILES):
            requests.append(request(spine, profile))
    for profile in ('vertical', 'bent'):
        requests.append(request('square', profile, 0))
        requests.append(request('square', profile, 1, 1))
