"""Deterministic requests for BRepFill_Sweep and, through the corners of its right and
round transitions, BRepFill_TrimShellCorner (tools/oracle/tkbool_probe_brepfill_sweep.cxx;
the record is documented there).

The sections lie around the origin of the local frame (the plane normal to the path):
a closed square, a circle, an open polyline and a vertex. They are swept along the paths
of the location law requests (a segment, a bent polyline, a closed square, a tangent arc
after a segment, a B-spline) with each transition; the segment also runs with the bounds,
without the particular cases (KPart), with the other trihedrons and with a forced C1
approximation.

Left out: the round corners of the square along the bent path, whose filling faces modify
edges sharing vertices (the substitution follows the TShape address order of a hash map,
so the surviving vertex differs between runs natively), and the bent path with the last
bound (SetBounds 2 and 3), which crashes natively.
"""
from __future__ import annotations
from tkbool_corpus_brepfill_generator import circle_wire, polyline_wire, square
from tkbool_corpus_brepfill_location_law import PATHS
from tktopalgo_shape_program import ShapeProgram


SECTIONS = {
    'square': lambda p: polyline_wire(p, square(0.0, 0.2), True),
    'circle': lambda p: circle_wire(p, (0.0, 0.0, 0.0), 0.1),
    'open': lambda p: polyline_wire(p, [(-0.1, 0.0, 0.0), (0.0, 0.1, 0.0), (0.1, 0.0, 0.0)], False),
    'vertex': lambda p: p.vertex((0.1, 0.0, 0.0)),
}

# transition kpart bounds tol3d angmin angmax c1 (the native defaults otherwise).
DEFAULTS = '1 0 0.0001 0.01 6.0 0'


def request(path: str, section: str, trihedron: int, transition: int, settings: str = DEFAULTS) -> str:
    """One sweep of a section along a path with the corrected Frenet (0), Frenet (1) or
    constant binormal (2) trihedron."""
    p = ShapeProgram()
    path_index = PATHS[path](p)
    section_index = SECTIONS[section](p)
    return f'bf_sweep {p.tokens()} {path_index} {section_index} 0 {trihedron} {transition} {settings}'


def append_sweep_requests(requests: list[str]) -> None:
    """Every section along every path with each transition, then the variants."""
    for path in sorted(PATHS):
        if path == 'reversed':
            continue
        for section in sorted(SECTIONS):
            for transition in (0, 1, 2):
                if (path, section, transition) == ('bent', 'square', 2):
                    continue
                requests.append(request(path, section, 0, transition))
    for bounds in (1, 2, 3):
        requests.append(request('segment', 'square', 0, 0, f'1 {bounds} 0.0001 0.01 6.0 0'))
    requests.append(request('bent', 'square', 0, 1, '1 1 0.0001 0.01 6.0 0'))
    for path in ('segment', 'bspline', 'bent'):
        requests.append(request(path, 'circle', 0, 0, '0 0 0.0001 0.01 6.0 0'))
        requests.append(request(path, 'square', 2, 1, DEFAULTS))
        requests.append(request(path, 'circle', 0, 0, '1 0 0.0001 0.01 6.0 1'))
        if path != 'bent':
            # Round corners of the square along the bent path are left out (see above).
            requests.append(request(path, 'square', 1, 2, DEFAULTS))
            requests.append(request(path, 'square', 0, 1, '1 0 0.001 0.5 1.0 0'))
    requests.append(request('bent', 'circle', 1, 2, DEFAULTS))
    requests.append(request('bent', 'open', 0, 1, '1 0 0.001 0.5 1.0 0'))
