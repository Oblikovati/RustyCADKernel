"""Deterministic request corpus for the TKBRep oracle.

Each request is one `operation numbers...` line read by tools/oracle/tkbrep_probe.cxx
and crates/tkbrep/examples/numeric_probe.rs. A request starts with a graph script
(locations, then TopoDS commands, see tools/oracle/tkbrep_probe_graph.cxx) built with
`Script`; the scenarios below cover every shape type, located and oriented parents,
shared sub-shapes, the builder errors, the flags and the location algebra. Each
scenario feeds every report operation.
"""
from __future__ import annotations
from dataclasses import dataclass, field
from tkbrep_bintools_corpus import append_tkbrep_bintools_requests
from tkbrep_brep_corpus import append_tkbrep_brep_requests
from tkbrep_brepgraph_corpus import append_tkbrep_brepgraph_requests
from tkbrep_brepgraph_walk_corpus import append_tkbrep_brepgraph_walk_requests
from tkbrep_brepgraph_layers_corpus import append_tkbrep_brepgraph_layer_requests
from tkbrep_breptools_corpus import append_tkbrep_breptools_requests

COMPOUND, COMPSOLID, SOLID, SHELL, FACE, WIRE, EDGE, VERTEX, SHAPE = range(9)
FORWARD, REVERSED, INTERNAL, EXTERNAL = range(4)


@dataclass
class Script:
    """A graph script: location specs, then commands; indices follow the native registry."""

    locations: list[list[float]] = field(default_factory=list)
    commands: list[list[float]] = field(default_factory=list)
    shapes: int = 0

    def _loc(self, spec: list[float]) -> int:
        self.locations.append(spec)
        return len(self.locations)

    def translation(self, x: float, y: float, z: float) -> int:
        """A new datum translating by (x, y, z); returns its location index."""
        return self._loc([0, x, y, z])

    def rotation(self, x: float, y: float, z: float, angle: float) -> int:
        """A new datum rotating about the axis (x, y, z) through the origin."""
        return self._loc([1, x, y, z, angle])

    def product(self, i: int, j: int) -> int:
        return self._loc([2, i, j])

    def power(self, i: int, p: int) -> int:
        return self._loc([3, i, p])

    def scale(self, s: float) -> int:
        """A new datum scaling by s about the origin (not a rigid motion)."""
        return self._loc([4, s])

    def inverse(self, i: int) -> int:
        return self._loc([5, i])

    def _new(self, command: list[float]) -> int:
        self.commands.append(command)
        self.shapes += 1
        return self.shapes - 1

    def make(self, kind: int) -> int:
        return self._new([1, kind])

    def add(self, parent: int, child: int, orientation: int = FORWARD, loc: int = 0) -> None:
        self.commands.append([2, parent, child, orientation, loc])

    def remove(self, parent: int, child: int, orientation: int = FORWARD, loc: int = 0) -> None:
        self.commands.append([3, parent, child, orientation, loc])

    def place(self, shape: int, orientation: int, loc: int) -> None:
        """Sets the orientation and location of a registry shape in place."""
        self.commands.append([4, shape, orientation, loc])

    def moved(self, shape: int, orientation: int, loc: int) -> int:
        return self._new([5, shape, orientation, loc])

    def flag(self, shape: int, bit: int, value: bool) -> None:
        self.commands.append([6, shape, bit, int(value)])

    def empty_copy(self, shape: int) -> int:
        return self._new([7, shape])

    def locate(self, shape: int, loc: int, raise_exc: bool) -> None:
        self.commands.append([8, shape, loc, int(raise_exc)])

    def move(self, shape: int, loc: int, raise_exc: bool) -> None:
        self.commands.append([9, shape, loc, int(raise_exc)])

    def reversed(self, shape: int) -> int:
        return self._new([10, shape])

    def complemented(self, shape: int) -> int:
        return self._new([11, shape])

    def composed(self, shape: int, orientation: int) -> int:
        return self._new([12, shape, orientation])

    def nullify(self, shape: int) -> None:
        self.commands.append([13, shape])

    def null(self) -> int:
        return self._new([14])

    def numbers(self) -> list[float]:
        out: list[float] = [len(self.locations)]
        for spec in self.locations:
            out += spec
        out.append(len(self.commands))
        for command in self.commands:
            out += command
        return out

    def edge(self, first: int, last: int) -> int:
        """A new edge holding `first` FORWARD and `last` REVERSED."""
        e = self.make(EDGE)
        self.add(e, first, FORWARD)
        self.add(e, last, REVERSED)
        return e

    def wire(self, edges: list[tuple[int, int]]) -> int:
        """A new wire of (edge, orientation) pairs."""
        w = self.make(WIRE)
        for e, orientation in edges:
            self.add(w, e, orientation)
        return w


def text(values: list[float]) -> str:
    return ' '.join(repr(v) if isinstance(v, float) else str(v) for v in values)


def single_kinds() -> tuple[Script, int]:
    """One shape of every type, a compound holding the others, the flags and copies."""
    s = Script()
    shapes = [s.make(kind) for kind in range(8)]
    for kind in range(8):
        s.empty_copy(shapes[kind])
    s.flag(shapes[COMPOUND], 2, True)
    s.flag(shapes[COMPOUND], 1, True)
    s.flag(shapes[WIRE], 7, True)
    s.flag(shapes[FACE], 4, True)
    s.flag(shapes[FACE], 1, False)
    for kind in range(1, 8):
        s.add(shapes[COMPOUND], shapes[kind], kind % 4)
    s.null()
    return s, shapes[COMPOUND]


def located_triangle() -> tuple[Script, int]:
    """A closed and an open wire in a located, reversed face, with rotations and powers."""
    s = Script()
    t = s.translation(1.0, 2.0, 3.0)
    r = s.rotation(0.0, 0.0, 1.0, 0.5)
    tr = s.product(t, r)
    r2 = s.power(r, 2)
    rinv = s.inverse(r)
    back = s.product(r, rinv)
    v = [s.make(VERTEX) for _ in range(4)]
    e = [s.edge(v[0], v[1]), s.edge(v[1], v[2]), s.edge(v[2], v[0]), s.edge(v[2], v[3])]
    closed = s.wire([(e[0], FORWARD), (e[1], FORWARD), (e[2], FORWARD)])
    opened = s.wire([(e[0], REVERSED), (e[3], FORWARD)])
    face = s.make(FACE)
    s.place(face, REVERSED, tr)
    s.add(face, closed, FORWARD, r)
    s.add(face, opened, REVERSED, r2)
    s.add(face, v[3], INTERNAL, back)
    s.add(face, v[0], EXTERNAL, rinv)
    s.moved(face, FORWARD, t)
    s.moved(e[1], INTERNAL, tr)
    s.reversed(closed)
    s.complemented(e[3])
    s.composed(opened, REVERSED)
    return s, face


def box_like(dx: float, located: bool) -> tuple[Script, int]:
    """A solid with one shell of six four-edge faces sharing 12 edges and 8 vertices."""
    s = Script()
    t = s.translation(dx, 0.0, 0.0) if located else 0
    rot = s.rotation(1.0, 0.0, 0.0, 0.25) if located else 0
    v = [s.make(VERTEX) for _ in range(8)]
    pairs = [(0, 1), (2, 3), (4, 5), (6, 7), (0, 2), (1, 3), (4, 6), (5, 7), (0, 4), (1, 5), (2, 6), (3, 7)]
    e = [s.edge(v[a], v[b]) for a, b in pairs]
    faces = [[4, 6, 8, 10], [5, 7, 9, 11], [0, 2, 8, 9], [1, 3, 10, 11], [0, 1, 4, 5], [2, 3, 6, 7]]
    shell = s.make(SHELL)
    for number, edges in enumerate(faces):
        w = s.wire([(e[i], REVERSED if k == 2 else FORWARD) for k, i in enumerate(edges)])
        f = s.make(FACE)
        s.add(f, w, FORWARD, rot if number == 3 else 0)
        s.add(shell, f, REVERSED if number % 2 else FORWARD, t if number == 5 else 0)
    solid = s.make(SOLID)
    s.add(solid, shell, FORWARD, t)
    s.add(solid, e[0], INTERNAL)
    s.add(solid, v[7], EXTERNAL, rot)
    return s, solid


def nested_compounds() -> tuple[Script, int]:
    """Compounds and composite solids nested under located, reversed parents, with a
    sub-shape shared twice and removals."""
    s = Script()
    t = s.translation(0.0, 5.0, 0.0)
    u = s.translation(-1.0, 0.0, 2.5)
    tu = s.product(t, u)
    tt = s.power(t, 3)
    outer, inner = s.make(COMPOUND), s.make(COMPOUND)
    s.place(inner, REVERSED, t)
    s.place(outer, FORWARD, u)
    solids = [s.make(SOLID) for _ in range(2)]
    comp_solid = s.make(COMPSOLID)
    for solid in solids:
        s.add(comp_solid, solid, FORWARD, tt)
    v = s.make(VERTEX)
    s.add(inner, v, FORWARD, tu)
    s.add(inner, v, REVERSED, t)
    s.add(inner, comp_solid)
    s.add(outer, inner, REVERSED, u)
    s.add(outer, v, INTERNAL)
    s.add(outer, comp_solid, FORWARD, tu)
    s.remove(outer, v, FORWARD)
    s.remove(outer, v, INTERNAL)
    s.moved(inner, EXTERNAL, tt)
    return s, outer


def builder_errors() -> tuple[Script, int]:
    """Incompatible types, frozen parents, self insertion, removals from frozen shapes,
    raising location setters and flags."""
    s = Script()
    scale = s.scale(2.0)
    t = s.translation(1.0, 1.0, 1.0)
    shapes = [s.make(kind) for kind in range(8)]
    for parent in range(8):
        for child in range(8):
            s.add(shapes[parent], s.make(child))
    for kind in (VERTEX, WIRE, COMPOUND):
        s.add(shapes[kind], shapes[kind])
    frozen = s.make(WIRE)
    holder = s.make(COMPOUND)
    s.add(holder, frozen)
    s.add(frozen, s.edge(s.make(VERTEX), s.make(VERTEX)))
    s.remove(frozen, shapes[EDGE])
    s.flag(frozen, 0, True)
    s.add(frozen, shapes[EDGE])
    s.remove(frozen, shapes[EDGE])
    s.remove(frozen, shapes[EDGE])
    s.locate(shapes[VERTEX], scale, True)
    s.locate(shapes[VERTEX], scale, False)
    s.locate(shapes[EDGE], t, True)
    s.move(shapes[EDGE], scale, True)
    s.move(shapes[EDGE], t, True)
    s.move(shapes[WIRE], scale, False)
    s.flag(shapes[SOLID], 2, True)
    s.flag(shapes[SOLID], 1, True)
    s.flag(shapes[SHELL], 5, True)
    s.flag(shapes[SHELL], 6, True)
    victim = s.make(SHELL)
    s.nullify(victim)
    return s, holder


def scenarios() -> list[tuple[Script, int]]:
    return [single_kinds(), located_triangle(), box_like(10.0, False), box_like(2.5, True),
            nested_compounds(), builder_errors()]


def encoded(data: bytes) -> list[int]:
    return [len(data), *data]


LOCATION_TEXTS = [
    b'',
    b'Location 1\n',
    b'Locations 0\n',
    b'Locations 2\n1\n1 0 0 1 0 1 0 2 0 0 1 3\n2  1 -2 0\n',
    b'Locations 3\n1\n0 -1 0 0 1 0 0 0 0 0 1 0\n2  1 1 1 1 0\n3\n',
    b'Locations 1\n2  5 1 0\n',
    b'Locations 1\n1\n0 0 0 0 0 0 0 0 0 0 0 0\n',
    b'Locations 2\n1\n1 0 0 1e400 0 1 0 -0 0 0 1 abc\n',
    b'Locations 4\n1\n2 0 0 0 0 2 0 0 0 0 2 0\n2 1 2 1 -1 0\n',
    b'Locations -1\n',
]

V3 = b'CASCADE Topology V3, (c) Open Cascade'
V1 = b'CASCADE Topology V1, (c) Matra-Datavision'
SET_TEXTS = [
    (1, 3, b''),
    (1, 3, b'\n' + V1 + b'\nLocations 0\nTShapes 0\n'),
    (1, 2, b'garbage\n' + V3 + b'\r\r\nLocations 0\nTShape 0\n'),
    (1, 3, b'\nno version here\n'),
    (0, 3, b'\n' + V1 + b'\nLocations 0\nProbe 0\nTShapes 2\nVe\ng0\n0101101\n*\nEd\ng1\n1110000\n+2 0 -2 0 *\n'),
    (0, 3, b'\n' + V3 + b'\nLocations 1\n1\n1 0 0 1 0 1 0 0 0 0 1 0\nProbe 0\nTShapes 2\n'
           b'Wi\ng0\n1\n*\nCo\ng1\n11111111\ni2 1 e2 1 *\n'),
    (0, 3, b'\n' + V3 + b'\nLocations 0\nProbe 0\nTShapes 1\nSh\ng0\n1\n+9 0 *\n'),
    (0, 1, b'\n' + V3 + b'\nLocations 0\nProbe 0\nTShapes 2\nFa\ng0\n1111111\n*\nSo\ng0\n1\n+2 0 *\n'),
]


def append_tkbrep_requests(requests: list[str]) -> None:
    """Append every TKBRep request family in a fixed order."""
    for script, root in scenarios():
        numbers = text(script.numbers())
        requests.append(f'br_topods {numbers}')
        requests.append(f'br_dump {numbers}')
        requests.append(f'br_explore {numbers} {root}')
        requests.append(f'br_maps {numbers} {root}')
        requests.append(f'br_vertices {numbers}')
        for format_nb in (1, 2, 3, 0, 4):
            requests.append(f'br_shapeset {format_nb} {numbers} {root}')
        requests.append(f'br_locset {numbers}')
    for data in LOCATION_TEXTS:
        requests.append('br_locread ' + text(encoded(data)))
    for base, format_nb, data in SET_TEXTS:
        requests.append(f'br_setread {base} {format_nb} ' + text(encoded(data)))
    append_tkbrep_brep_requests(requests)
    append_tkbrep_bintools_requests(requests)
    append_tkbrep_breptools_requests(requests)
    append_tkbrep_brepgraph_requests(requests)
    append_tkbrep_brepgraph_walk_requests(requests)
    append_tkbrep_brepgraph_layer_requests(requests)
