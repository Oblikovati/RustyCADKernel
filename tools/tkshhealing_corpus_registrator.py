"""Deterministic requests for the ShapeExtend message registrators
(tools/oracle/tkshhealing_probe_registrator.cxx).

The scene holds a vertex, an edge, the edge reversed (the same key under
TopTools_ShapeMapHasher), the edge moved by a translation (another key) and a wire. Each
request sends messages to objects, to shapes and to nothing, through a
BasicMsgRegistrator (which drops them) or a MsgRegistrator (which keeps them in order).
"""
from __future__ import annotations
from tktopalgo_shape_program import FORWARD, REVERSED, WIRE, ShapeProgram

SEND_OBJECT, SEND_SHAPE, SEND = range(3)


def scene() -> ShapeProgram:
    """Shapes 0 vertex, 1 edge, 2 reversed edge, 3 translated edge, 4 wire."""
    p = ShapeProgram()
    vertex = p.vertex((0., 0., 0.))
    edge = p.edge()
    p.edge_curve(edge, p.line((0., 0., 0.), (1., 0., 0.)))
    p.add(edge, vertex, FORWARD)
    p.oriented(edge, REVERSED)
    p.located(edge, p.location(0, 1., 2., 3.))
    wire = p.container(WIRE)
    p.add(wire, edge)
    return p


def request(p: ShapeProgram, kind: int, nobj: int, ops: list[list[int]]) -> str:
    """One `sh_registrator program kind nobj nops (op)...` line."""
    fields = [kind, nobj, len(ops), *[v for op in ops for v in op]]
    return ' '.join(['sh_registrator', p.tokens(), *map(str, fields)])


def scripts() -> list[list[list[int]]]:
    """Message scripts: objects, shapes, null targets, repeated targets, all gravities."""
    return [
        [],
        [[SEND_OBJECT, 0, 1, 0], [SEND_OBJECT, 1, 2, 1], [SEND_OBJECT, 0, 3, 2], [SEND_OBJECT, -1, 4, 3]],
        [[SEND_SHAPE, 1, 1, 4], [SEND_SHAPE, 2, 2, 0], [SEND_SHAPE, 3, 3, 1], [SEND_SHAPE, -1, 4, 2],
         [SEND_SHAPE, 0, 5, 3], [SEND_SHAPE, 4, 6, 4], [SEND_SHAPE, 1, 7, 2]],
        [[SEND, 1, 2], [SEND, 2, 4], [SEND_OBJECT, 2, 3, 0], [SEND_SHAPE, 0, 4, 1]],
        [[SEND_OBJECT, k % 3, k, k % 5] for k in range(12)] + [[SEND_SHAPE, k % 5, k, 2] for k in range(10)],
    ]


def append_registrator_requests(requests: list[str]) -> None:
    """Append the registrator requests: every script through both registrators."""
    p = scene()
    for kind in (0, 1):
        requests.extend(request(p, kind, 3, ops) for ops in scripts())
    requests.append(request(p, 1, 0, [[SEND, 0, 0]]))
