"""Deterministic requests for the TopOpeBRepBuild face chain: WireEdgeClassifier,
FaceBuilder, Builder::MakeFaces and WireToFace (tools/oracle/tkbool_probe_topopebrepbuild.cxx;
the records are documented there).

The shapes lie on the face of the square [0, 10]^2 of the plane z = 0: holes inside it,
squares outside it, a triangle sharing a corner with it, open chains and an edge given
twice with opposite orientations (a pseudo internal edge). Their edges have p-curves on
the plane of the face, or none (the FC2D functions then make them). The lateral face of
a cylinder with its seam adds a periodic surface.
"""
from __future__ import annotations
from tktopalgo_corpus_brepcheck_shapes import PlanarShapes, cylinder_face
from tktopalgo_shape_program import FORWARD, REVERSED, WIRE, ShapeProgram

POINTS = [(0, 0, 0), (10, 0, 0), (10, 10, 0), (0, 10, 0),
          (3, 3, 0), (5, 3, 0), (5, 5, 0), (3, 5, 0),
          (6, 6, 0), (8, 6, 0), (8, 8, 0), (6, 8, 0),
          (12, 0, 0), (14, 0, 0), (14, 2, 0), (12, 2, 0),
          (3, 1, 0), (1, 3, 0),
          (3.5, 3.5, 0), (4.5, 3.5, 0), (4.5, 4.5, 0), (3.5, 4.5, 0)]
LOOPS = {'outer': [0, 1, 2, 3], 'hole': [4, 7, 6, 5], 'hole_ccw': [4, 5, 6, 7], 'hole2': [8, 11, 10, 9],
         'away': [12, 13, 14, 15], 'island': [18, 19, 20, 21], 'corner': [0, 16, 17], 'chain': [4, 5, 6]}


class PlaneScene:
    """The face of the outer square and named wires and edges on its plane."""

    def __init__(self, pcurves: bool = True) -> None:
        self.p = ShapeProgram()
        self.pl = PlanarShapes(self.p, POINTS)
        self.face = self.pl.face([LOOPS['outer']])
        self.surface, self.frame = self.pl.surfaces[0], self.pl.frames[0]
        self.outer = self.pl.wires[0][0]
        self.pcurves = pcurves
        self.shapes: dict[str, int] = {'face': self.face, 'outer': self.outer}
        # The outer edges have their p-curves on the face.
        self.with_pcurve = set(self.pl.edges)

    def edge(self, a: int, b: int) -> int:
        """The edge from a to b, oriented along the walk, with its p-curve when asked."""
        key = self.pl.edge(a, b)
        if self.pcurves and key not in self.with_pcurve:
            self.pl.pcurve(key, self.surface, self.frame)
            self.with_pcurve.add(key)
        edge = self.pl.edges[key]
        return edge if a < b else self.p.oriented(edge, REVERSED)

    def wire(self, name: str, closed: bool = True) -> int:
        """The wire of a named loop (open for a chain)."""
        loop = LOOPS[name]
        pairs = list(zip(loop, loop[1:] + loop[:1])) if closed else list(zip(loop, loop[1:]))
        wire = self.p.container(WIRE)
        for a, b in pairs:
            self.p.add(wire, self.edge(a, b), -1)
        self.shapes[name] = wire
        return wire

    def tokens(self) -> str:
        self.pl.finish()
        return self.p.tokens()


def wire_to_face_request(names: list[str], pcurves: bool = True) -> str:
    s = PlaneScene(pcurves)
    wires = [s.outer if n == 'outer' else s.wire(n) for n in names]
    return f"tb_wire_to_face {s.tokens()} {s.face} {len(wires)} {' '.join(map(str, wires))}".rstrip()


def cylinder_wire_to_face_request() -> str:
    p = ShapeProgram()
    cyl = cylinder_face(p, 1.0, 2.0)
    return f"tb_wire_to_face {p.tokens()} {cyl['face']} 1 {cyl['wire']}"


def face_builder_request(items: list[tuple[int, str]], force: bool = True, pcurves: bool = True) -> str:
    """`items` are (kind, name): a named wire, `outer`, or an edge `e:a:b`."""
    s = PlaneScene(pcurves)
    fields = []
    for kind, name in items:
        if name == 'outer':
            index = s.outer
        elif name.startswith('e:'):
            a, b = (int(v) for v in name[2:].split(':'))
            index = s.edge(a, b)
        else:
            index = s.wire(name, name != 'chain')
        fields.append(f'{kind} {index}')
    return f"tb_face_builder {s.tokens()} {s.face} {int(force)} {len(items)} {' '.join(fields)}"


def wec_request(n1: str, n2: str, pcurves: bool = True) -> str:
    s = PlaneScene(pcurves)
    i1 = s.outer if n1 == 'outer' else s.wire(n1, n1 != 'chain')
    i2 = s.outer if n2 == 'outer' else s.wire(n2, n2 != 'chain')
    return f'tb_wec {s.tokens()} {s.face} {i1} {i2}'


def outer_edges() -> list[tuple[int, str]]:
    loop = LOOPS['outer']
    return [(1, f'e:{a}:{b}') for a, b in zip(loop, loop[1:] + loop[:1])]


def append_wire_to_face_requests(requests: list[str]) -> None:
    for names in (['outer'], ['outer', 'hole'], ['outer', 'hole', 'hole2'], ['outer', 'away'],
                  ['hole_ccw', 'hole2'], ['outer', 'hole_ccw'], ['outer', 'corner'], ['away']):
        for pcurves in (True, False):
            requests.append(wire_to_face_request(names, pcurves))
    requests.append(cylinder_wire_to_face_request())


def append_face_builder_requests(requests: list[str]) -> None:
    cases = [
        [(0, 'outer'), (0, 'hole')],
        [(0, 'outer'), (1, 'e:4:7'), (1, 'e:7:6'), (1, 'e:6:5'), (1, 'e:5:4')],
        outer_edges() + [(1, 'e:4:7'), (1, 'e:7:6'), (1, 'e:6:5'), (1, 'e:5:4')],
        outer_edges() + [(1, 'e:4:5'), (1, 'e:5:6')],
        outer_edges() + [(1, 'e:0:16'), (1, 'e:16:0')],
        outer_edges() + [(2, 'e:0:16'), (2, 'e:16:17'), (2, 'e:17:0')],
        [(0, 'outer'), (0, 'hole_ccw'), (0, 'away')],
        [(1, 'e:4:5'), (1, 'e:5:6'), (1, 'e:6:7')],
    ]
    for items in cases:
        for force in (True, False):
            requests.append(face_builder_request(items, force))
    requests.append(face_builder_request(cases[2], True, False))


def append_wec_requests(requests: list[str]) -> None:
    for n1, n2 in (('hole', 'outer'), ('outer', 'hole'), ('away', 'outer'), ('hole', 'hole2'),
                   ('corner', 'outer'), ('outer', 'corner'), ('hole_ccw', 'outer'), ('chain', 'outer')):
        for pcurves in (True, False):
            requests.append(wec_request(n1, n2, pcurves))


def append_topopebrepbuild_requests(requests: list[str]) -> None:
    """The wire edge classifications, face builders and wire to face requests."""
    append_wec_requests(requests)
    append_face_builder_requests(requests)
    append_wire_to_face_requests(requests)


def face_restrictor_request(names: list[str], proj: bool, control: bool, pcurves: bool) -> str:
    s = PlaneScene(pcurves)
    wires = [s.outer if n == 'outer' else s.wire(n) for n in names]
    return (f"ba_face_restrictor {s.tokens()} {s.face} {int(proj)} {int(control)} {len(wires)} "
            f"{' '.join(map(str, wires))}")


def cylinder_face_restrictor_request(control: bool) -> str:
    p = ShapeProgram()
    cyl = cylinder_face(p, 1.0, 2.0)
    return f"ba_face_restrictor {p.tokens()} {cyl['face']} 0 {int(control)} 1 {cyl['wire']}"


def append_face_restrictor_requests(requests: list[str]) -> None:
    """BRepAlgo_FaceRestrictor on the plane scene: plain (with and without p-curves,
    projected or not) and with orientation control (wrong orientations, three nesting
    levels, disjoint wires), and on the cylinder face."""
    for names in (['outer', 'hole'], ['outer', 'hole', 'hole2'], ['away'], ['outer', 'hole', 'island']):
        for pcurves in (True, False):
            requests.append(face_restrictor_request(names, False, False, pcurves))
        requests.append(face_restrictor_request(names, True, False, False))
    for names in (['outer', 'hole_ccw'], ['hole', 'outer'], ['outer', 'hole_ccw', 'island'], ['away', 'hole_ccw'],
                  ['outer', 'hole', 'hole2', 'island'], ['outer']):
        for pcurves in (True, False):
            requests.append(face_restrictor_request(names, False, True, pcurves))
    requests.append(cylinder_face_restrictor_request(False))
    requests.append(cylinder_face_restrictor_request(True))
