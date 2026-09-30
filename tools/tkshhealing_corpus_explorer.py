"""Deterministic requests for ShapeExtend_Explorer (tools/oracle/tkshhealing_probe_explorer.cxx).

One scene holds a shape of every type built with a TKTopAlgo shape program
(tools/tktopalgo_shape_program.py): vertices, edges, a wire, faces, a shell, a solid, a
composite solid and compounds (of edges, of an edge and a wire, of faces and a shell,
mixed, nested, empty, with an empty sub-compound, with one item, of solids). Each op of
the explorer runs on the shapes and lists of them; null shapes and null handles are
asked only where the native member does not dereference them.
"""
from __future__ import annotations
from dataclasses import dataclass
from tktopalgo_shape_program import (COMPOUND, COMPSOLID, FORWARD, REVERSED, SHELL, SOLID,
                                     WIRE, ShapeProgram)

COMPOUND_FROM_SEQ, SEQ_FROM_COMPOUND, LIST_FROM_SEQ, SEQ_FROM_LIST = range(4)
SHAPE_TYPE, SORTED_COMPOUND, DISPATCH_LIST = range(4, 7)
# TopAbs_ShapeEnum values.
T_COMPOUND, T_COMPSOLID, T_SOLID, T_SHELL, T_FACE, T_WIRE, T_EDGE, T_VERTEX, T_SHAPE = range(9)


@dataclass
class ExplorerScene:
    """The program and the named indices of its shapes."""
    program: ShapeProgram
    named: dict[str, int]


def container(p: ShapeProgram, kind: int, children: list[int], orientation: int = -1) -> int:
    """A new container of the children."""
    shape = p.container(kind)
    for child in children:
        p.add(shape, child, orientation)
    return shape


def base_shapes(p: ShapeProgram) -> dict[str, int]:
    """Vertices, edges, a wire, two faces, a shell, a solid and a composite solid."""
    n: dict[str, int] = {}
    plane = p.surface(0, 0., 0., 0., 0., 0., 1.)
    n['v0'], n['v1'] = p.vertex((0., 0., 0.)), p.vertex((1., 0., 0.))
    for name, (a, b) in (('e0', ('v0', 'v1')), ('e1', ('v1', 'v0'))):
        edge = p.edge()
        p.edge_curve(edge, p.line((0., 0., 0.), (1., 0., 0.)))
        p.add(edge, n[a], FORWARD)
        p.add(edge, n[b], REVERSED)
        n[name] = edge
    n['w0'] = container(p, WIRE, [n['e0'], n['e1']])
    n['f0'], n['f1'] = p.face(plane), p.face(plane)
    p.add(n['f0'], n['w0'])
    n['sh0'] = container(p, SHELL, [n['f0'], n['f1']])
    n['so0'] = container(p, SOLID, [n['sh0']])
    n['cs0'] = container(p, COMPSOLID, [n['so0']])
    n['e0r'] = p.oriented(n['e0'], REVERSED)
    return n


def compounds(p: ShapeProgram, n: dict[str, int]) -> None:
    """The compounds of the scene."""
    n['c_edges'] = container(p, COMPOUND, [n['e0'], n['e1']])
    n['c_edge_wire'] = container(p, COMPOUND, [n['e0'], n['w0']])
    n['c_wire_edge'] = container(p, COMPOUND, [n['w0'], n['e0']])
    n['c_face_shell'] = container(p, COMPOUND, [n['f0'], n['sh0']])
    n['c_shell_face'] = container(p, COMPOUND, [n['sh0'], n['f1']])
    n['c_mixed'] = container(p, COMPOUND, [n['e0'], n['f0']])
    n['c_empty'] = container(p, COMPOUND, [])
    n['c_with_empty'] = container(p, COMPOUND, [n['e0'], n['c_empty']])
    n['c_nested'] = container(p, COMPOUND, [n['c_edges'], n['w0']])
    n['c_deep'] = container(p, COMPOUND, [n['c_nested'], n['v0'], n['c_empty']])
    n['c_single'] = container(p, COMPOUND, [n['e1']])
    n['c_solids'] = container(p, COMPOUND, [n['so0'], n['cs0']])
    n['c_reversed'] = p.oriented(n['c_nested'], REVERSED)


def scene() -> ExplorerScene:
    """The shapes every explorer request works on."""
    p = ShapeProgram()
    n = base_shapes(p)
    compounds(p, n)
    return ExplorerScene(p, n)


def request(s: ExplorerScene, *fields: int) -> str:
    """One `sh_explorer program op ...` line."""
    return ' '.join(['sh_explorer', s.program.tokens(), *map(str, fields)])


def shape_list(s: ExplorerScene, names: list[str]) -> list[int]:
    """A list record of named shapes (`null` for a null shape)."""
    return [len(names), *[-1 if name == 'null' else s.named[name] for name in names]]


def sequence_requests(s: ExplorerScene) -> list[str]:
    """CompoundFromSeq, SeqFromCompound, ListFromSeq and SeqFromList."""
    lists = [[], ['e0'], ['e0', 'w0', 'f0'], ['c_edges', 'so0', 'e0r'], ['v0', 'v0']]
    out = [request(s, COMPOUND_FROM_SEQ, *shape_list(s, names)) for names in lists]
    out += [request(s, SEQ_FROM_LIST, *shape_list(s, names)) for names in lists + [['null', 'e0']]]
    for name in s.named:
        out += [request(s, SEQ_FROM_COMPOUND, s.named[name], exp) for exp in (0, 1)]
    out += [request(s, SEQ_FROM_COMPOUND, -1, 1)]
    for names in lists + [['null']]:
        for pre, clear in ((-2, 1), (-2, 0), (s.named['v1'], 1), (s.named['v1'], 0), (-1, 0)):
            out.append(request(s, LIST_FROM_SEQ, *shape_list(s, names), pre, clear))
    out += [request(s, LIST_FROM_SEQ, -1, s.named['v1'], clear) for clear in (0, 1)]
    return out


def type_requests(s: ExplorerScene) -> list[str]:
    """ShapeType of every shape, and SortedCompound for every requested type."""
    out = []
    for index in [*s.named.values(), -1]:
        out += [request(s, SHAPE_TYPE, index, compound) for compound in (0, 1)]
    for index in [*s.named.values(), -1]:
        for kind in (T_COMPOUND, T_SOLID, T_SHELL, T_FACE, T_WIRE, T_EDGE, T_VERTEX):
            for explore in (0, 1):
                out += [request(s, SORTED_COMPOUND, index, kind, explore, c) for c in (0, 1)]
    return out


def dispatch_requests(s: ExplorerScene) -> list[str]:
    """DispatchList over lists of every type, with null and pre-filled outputs."""
    everything = ['v0', 'e0', 'w0', 'f0', 'sh0', 'so0', 'cs0', 'c_edges', 'null', 'e1', 'c_empty']
    out = [request(s, DISPATCH_LIST, *shape_list(s, everything), mask) for mask in (0, 255, 0b10100101)]
    out += [request(s, DISPATCH_LIST, *shape_list(s, []), mask) for mask in (0, 3)]
    out += [request(s, DISPATCH_LIST, -1, mask) for mask in (0, 7)]
    return out


def append_explorer_requests(requests: list[str]) -> None:
    """Append the ShapeExtend_Explorer requests."""
    s = scene()
    requests.extend(sequence_requests(s) + type_requests(s) + dispatch_requests(s))
