"""Deterministic requests for TopOpeBRepTool_ShapeTool, PROJECT, GEOMETRY, FC2D, FTOL,
HBoxTool and TOOL::UVISO (tools/oracle/tkbool_probe_topopebreptool_shapes.cxx; the
records are documented there).

One program holds the faces of the cube [0, 2]^3 and its shell, the lateral face of the
cylinder of radius 1 and height 2 on the Z axis with its seam and circles, a vertical
segment lying on the cylinder at the angle 1 (no p-curve), a segment above the cube, a
sphere and a cone of radius 0 (natural faces). The shape tool compares orientations of
faces, edges and vertices, bounds and adjusts UV on periodic faces and periodizes seam
parameters; the projections and geometry helpers run on lines, circles, planes, the
cylinder, the sphere poles and the cone apex; FC2D finds stored p-curves, makes the
p-curves of 3d-only edges and translates them onto the seam; the box tool boxes faces,
edges, vertices and refuses wires.
"""
from __future__ import annotations
import math
from tkprim_corpus_sweep_shapes import segment
from tktopalgo_corpus_brepcheck_shapes import cube, cylinder_face
from tktopalgo_shape_program import REVERSED, ShapeProgram


def scene(p: ShapeProgram) -> dict:
    """The shapes of every request, by name."""
    c = cube(p, 2.0)
    cyl = cylinder_face(p, 1.0, 2.0)
    a = 1.0
    out = {
        'f0': c.faces[0], 'f1': c.faces[1], 'f0r': p.oriented(c.faces[0], REVERSED), 'shell': c.shell(),
        'e01': c.edges[(0, 1)], 'e12': c.edges[(1, 2)], 'v0': c.vertices[0], 'v1': c.vertices[1],
        'cyl': cyl['face'], 'cylr': p.oriented(cyl['face'], REVERSED), 'seam': cyl['seam'], 'bottom': cyl['bottom'],
        'onc': segment(p, (math.cos(a), math.sin(a), 0.2), (math.cos(a), math.sin(a), 1.8)),
        'above': segment(p, (0.5, 0.5, 3.0), (1.5, 1.0, 3.0)),
        'sphere': p.face(p.surface(2, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0)),
        'cone': p.face(p.surface(4, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.5)),
    }
    return out


def request(op: str, names: list[str], numbers=()) -> str:
    p = ShapeProgram()
    s = scene(p)
    fields = ' '.join(str(s[n]) for n in names)
    tail = ' '.join(repr(float(v)) for v in numbers)
    return f'{op} {p.tokens()} {fields} {tail}'.rstrip()


SHAPE_TOOL = [
    (['f0', 'f1', 'e01', 'f0'], (0.5, 0.3, 0.4)),
    (['f0', 'f0r', 'e01', 'f0'], (0.0, -0.3, 2.4)),
    (['v0', 'v1', 'seam', 'cyl'], (1.0, 7.0, -0.5)),
    (['e01', 'e12', 'bottom', 'cyl'], (1.0, -1.0, 1.0)),
    (['cyl', 'cylr', 'seam', 'cyl'], (0.0, 2 * math.pi, 0.5)),
    (['cyl', 'cyl', 'seam', 'cyl'], (2.0, 3.0, 0.5)),
    (['shell', 'shell', 'above', 'sphere'], (0.5, 1.0, 0.2)),
    (['cyl', 'f0', 'onc', 'cone'], (0.5, 0.2, -1.0)),
]

PROJECT = [
    (['e01', 'f0'], (0.5, -0.3, 0.2)),
    (['e01', 'f0'], (5.0, 0.0, 0.0)),
    (['seam', 'cyl'], (1.5, 0.2, 1.0)),
    (['bottom', 'cyl'], (0.0, 2.0, 0.5)),
    (['onc', 'cyl'], (0.0, 0.0, 1.0)),
    (['above', 'f1'], (1.0, 1.0, 2.5)),
]

GEOMETRY = [
    (['e01', 'f0'], (0.5, 0.5)),
    (['bottom', 'cyl'], (1.0, 0.5)),
    (['seam', 'sphere'], (0.3, math.pi / 2)),
    (['seam', 'sphere'], (0.3, -math.pi / 2)),
    (['e12', 'sphere'], (0.3, 0.2)),
    (['e12', 'cone'], (0.0, 0.0)),
    (['e12', 'cone'], (1.0, 0.0)),
    (['e12', 'cone'], (1.0, 0.5)),
]

FC2D = [
    ['shell', 'cyl', 'bottom', 'cyl', 'seam'],
    ['shell', 'cyl', 'onc', 'cyl', 'seam'],
    ['shell', 'cyl', 'onc', 'cylr', 'seam'],
    ['shell', 'cyl', 'above', 'f1', 'e01'],
    ['shell', 'cyl', 'e01', 'f0', 'e12'],
    ['cyl', 'shell', 'seam', 'cyl', 'bottom'],
]

HBOX = [('shell', 4), ('cyl', 6), ('shell', 7), ('shell', 5), ('e01', 7)]


def append_topopebreptool_shapes_requests(requests: list[str]) -> None:
    """The shape tool, projections, geometry, FC2D and box requests."""
    for names, numbers in SHAPE_TOOL:
        requests.append(request('tt_shape_tool', names, numbers))
    for names, numbers in PROJECT:
        requests.append(request('tt_project', names, numbers))
    for names, numbers in GEOMETRY:
        requests.append(request('tt_geometry', names, numbers))
    for names in FC2D:
        requests.append(request('tt_fc2d', names))
    for name, ts in HBOX:
        p = ShapeProgram()
        s = scene(p)
        requests.append(f'tt_hbox {p.tokens()} {s[name]} {ts}')
