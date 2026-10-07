"""Deterministic requests for BRepFill_MultiLine, BRepFill_ApproxSeewing and
BRepFill_TrimSurfaceTool (tools/oracle/tkbool_probe_brepfill_multi_line.cxx; the records
are documented there).

Two slanted planar faces rise at 45 degrees from the sides A-B and B-F of a square
corner and meet along B-C above its bisector: the multi-line along the bisector (a line,
the plane particular case, or a Bezier near it), straight or inverted, gives the
intersection line; the trim tool intersects the bisector with the upper edges, tests
points on the faces, projects on an edge and projects the faces along the bisector.
"""
from __future__ import annotations
from tktopalgo_corpus_brepcheck_shapes import PlanarShapes
from tktopalgo_shape_program import ShapeProgram

# A, B, C, D, F, G: the corner B, the sides A-B and B-F on z = 0 and their tops.
POINTS = [(0.0, 0.0, 0.0), (4.0, 0.0, 0.0), (3.0, 1.0, 1.0), (0.0, 1.0, 1.0), (4.0, 4.0, 0.0), (3.0, 4.0, 1.0)]
S = 0.7071067811865476


def corner(p: ShapeProgram) -> dict:
    """The two faces and their edges: e1 = A-B, e2 = B-F, top1 = C-D, top2 = C-G."""
    shapes = PlanarShapes(p, POINTS)
    f1 = shapes.face([[0, 1, 2, 3]])
    f2 = shapes.face([[1, 4, 5, 2]])
    shapes.finish()
    e = shapes.edges
    return {'f1': f1, 'f2': f2, 'e1': e[(0, 1)], 'e2': e[(1, 4)], 'top1': e[(2, 3)], 'top2': e[(2, 5)]}


def bisector(p: ShapeProgram, kind: str, last: float) -> int:
    """The bisector from B inward: a line trimmed to [0, last], or a Bezier near it."""
    if kind == 'line':
        line = p.pcurve(0, 4.0, 0.0, -S, S)
        return p.pcurve(4, line, 0.0, last)
    return p.pcurve(3, 3, 4.0, 0.0, 3.3, 0.75, 2.6 * last / 2.0, 1.4 * last / 2.0)


def multi_line_request(kind: str, inv: tuple[int, int], last: float) -> str:
    p = ShapeProgram()
    c = corner(p)
    bis = bisector(p, kind, last)
    return f"bf_multi_line {p.tokens()} {c['f1']} {c['f2']} {c['e1']} {c['e2']} {inv[0]} {inv[1]} {bis} 5"


def trim_request(kind: str, point: tuple[float, float], last: float, u: tuple[float, float]) -> str:
    p = ShapeProgram()
    c = corner(p)
    bis = bisector(p, kind, last)
    return (f"bf_trim_surface {p.tokens()} {bis} {c['f1']} {c['f2']} {c['e1']} {c['e2']} 0 0 "
            f"{c['top1']} {c['top2']} {point[0]} {point[1]} {c['top1']} {u[0]} {u[1]}")


def append_multi_line_requests(requests: list[str]) -> None:
    """The multi-lines and the trim tools of the corner."""
    for kind in ('line', 'bezier'):
        for inv in ((0, 0), (1, 0), (0, 1)):
            requests.append(multi_line_request(kind, inv, 1.0))
        requests.append(multi_line_request(kind, (0, 0), 2.0))
        for point in ((2.0, 0.5), (3.5, 3.0), (-1.0, -1.0)):
            requests.append(trim_request(kind, point, 2.0, (0.2, 1.2)))
        requests.append(trim_request(kind, (1.0, 1.0), 1.0, (0.0, 1.0)))
