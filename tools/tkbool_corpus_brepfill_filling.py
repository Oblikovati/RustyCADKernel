"""Deterministic requests for BRepFill_Filling (tools/oracle/tkbool_probe_brepfill_filling.cxx;
the record is documented there).

A twisted quadrilateral of segments (sharing their vertices, or with their own vertices
at the same points) is filled in G0 with the default and other parameters, with a point,
an inner edge constraint and a point on a face; the square face's edges are filled on
the face (G1, C1 and the refused G2), on their first curve on surface, with the face as
initial surface, and with an edge left out and the face as free constraint closing the
hole; an empty boundary is not done.
"""
from __future__ import annotations
from tkprim_corpus_sweep_shapes import segment
from tktopalgo_corpus_brepcheck_shapes import square as square_face
from tktopalgo_shape_program import ShapeProgram

C0, G1, C1, G2 = 0, 1, 2, 3
TWISTED = [(0.0, 0.0, 0.0), (2.0, 0.0, 0.5), (2.0, 2.0, 0.0), (0.0, 2.0, 0.5)]
DEFAULTS = '3 15 2 0 1e-05 0.0001 0.01 0.1 8 9'


def twisted(p: ShapeProgram, shared: bool = True) -> list[int]:
    """The four segments of the twisted quadrilateral (sharing their vertices or not)."""
    vertices = [p.vertex(point) for point in TWISTED]
    edges = []
    for k in range(4):
        j = (k+1) % 4
        own = (vertices[k], vertices[j]) if shared else None
        edges.append(segment(p, TWISTED[k], TWISTED[j], own))
    return edges


def edges_record(edges: list[tuple[int, int, int, int]]) -> str:
    """`n (edge face order bound)...`."""
    return ' '.join([str(len(edges))]+[f'{e} {f} {o} {b}' for e, f, o, b in edges])


def request(p: ShapeProgram, edges, free=(), points=(), init: int = -1, params: str = DEFAULTS) -> str:
    """One filling of the program shapes."""
    free_part = ' '.join([str(len(free))]+[f'{f} {o}' for f, o in free])
    point_part = ' '.join([str(len(points))]+list(points))
    return f'bf_filling {p.tokens()} {params} {edges_record(edges)} {free_part} {point_part} {init}'


def twisted_requests() -> list[str]:
    """The twisted quadrilateral in G0 with its variants."""
    out = []
    for shared, params, extra in ((True, DEFAULTS, ''), (False, DEFAULTS, ''), (True, '2 8 1 1 1e-05 0.0001 0.01 0.1 6 4', ''),
                                  (True, DEFAULTS, 'point'), (True, DEFAULTS, 'inner'), (True, DEFAULTS, 'unbound')):
        p = ShapeProgram()
        edges = [(e, -1, C0, 1) for e in twisted(p, shared)]
        points = ['0 1.0 1.0 0.6'] if extra == 'point' else []
        if extra == 'inner':
            edges.append((segment(p, (0.5, 1.0, 0.3), (1.5, 1.0, 0.3)), -1, C0, 0))
        if extra == 'unbound':
            edges[1] = (edges[1][0], -1, C0, 0)
        out.append(request(p, edges, (), points, -1, params))
    return out


def square_requests() -> list[str]:
    """The square face's edges on the face, on their curves on surface, with the face
    as initial surface, as free constraint and with a point on it."""
    out = []
    cases = (('face', G1), ('face', C1), ('face', G2), ('face', C0), ('nonface', G1), ('init', C0), ('free', C0),
             ('free', G1), ('uv', C0))
    for kind, order in cases:
        p = ShapeProgram()
        shapes = square_face(p, 2.0)
        face = shapes.faces[0]
        keys = sorted(shapes.edges)
        support = face if kind in ('face', 'free') else -1
        edges = [(shapes.edges[k], support, order, 1) for k in keys]
        free, points, init = (), [], -1
        if kind == 'init':
            init = face
        if kind == 'free':
            edges = edges[:3]
            free = ((face, order),)
        if kind == 'uv':
            points = [f'1 0.5 0.7 {face} {G1}']
        out.append(request(p, edges, free, points, init))
    return out


def append_filling_requests(requests: list[str]) -> None:
    """The twisted, square and empty fillings."""
    requests.extend(twisted_requests())
    requests.extend(square_requests())
    p = ShapeProgram()
    p.vertex((0.0, 0.0, 0.0))
    requests.append(request(p, []))
