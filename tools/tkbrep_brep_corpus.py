"""Deterministic BRep request corpus for the TKBRep oracle (`br_brep`, `br_brep_dump`).

A request is `nloc (location)... ngeom (geometry)... ncmd (command)...`, read by
tools/oracle/tkbrep_probe_brep.cxx and crates/tkbrep/examples/br_probe/brep.rs.
`BRepScript` numbers the registries as the probes do: locations from 1 (0 is the
identity), each geometry kind from 0, shapes from 0 in creation order. The scenarios
build every curve and point representation, located and reversed shapes, the builder
errors and the queries that reach the planar projection (printed `P` on both sides).
"""
from __future__ import annotations
from dataclasses import dataclass, field

FORWARD, REVERSED, INTERNAL, EXTERNAL = range(4)
C0, G1, C1, G2, C2, C3, CN = range(7)


@dataclass
class BRepScript:
    """Locations, geometries and commands of one BRep request."""

    locations: list[list[float]] = field(default_factory=list)
    geometries: list[list[float]] = field(default_factory=list)
    commands: list[list[float]] = field(default_factory=list)
    counts: dict[str, int] = field(default_factory=dict)
    shapes: int = 0

    def _count(self, kind: str) -> int:
        self.counts[kind] = self.counts.get(kind, 0) + 1
        return self.counts[kind] - 1

    # Locations (the tkbrep_probe_graph.cxx kinds); the result is the location index.
    def translation(self, x: float, y: float, z: float) -> int:
        self.locations.append([0, x, y, z])
        return len(self.locations)

    def rotation(self, x: float, y: float, z: float, angle: float) -> int:
        self.locations.append([1, x, y, z, angle])
        return len(self.locations)

    def product(self, i: int, j: int) -> int:
        self.locations.append([2, i, j])
        return len(self.locations)

    def scale(self, s: float) -> int:
        self.locations.append([4, s])
        return len(self.locations)

    def inverse(self, i: int) -> int:
        self.locations.append([5, i])
        return len(self.locations)

    # Geometries; the result is the index in the registry of the kind.
    def _geometry(self, registry: str, spec: list[float]) -> int:
        self.geometries.append(spec)
        return self._count(registry)

    def line(self, p, d) -> int:
        return self._geometry('curve', [0, *p, *d])

    def circle(self, p, n, r: float) -> int:
        return self._geometry('curve', [1, *p, *n, r])

    def bezier(self, poles) -> int:
        return self._geometry('curve', [2, len(poles), *[c for p in poles for c in p]])

    def trimmed(self, curve: int, u1: float, u2: float) -> int:
        return self._geometry('curve', [3, curve, u1, u2])

    def plane(self, p, n) -> int:
        return self._geometry('surface', [10, *p, *n])

    def cylinder(self, p, n, r: float) -> int:
        return self._geometry('surface', [11, *p, *n, r])

    def rtrim(self, surface: int, u1: float, u2: float, v1: float, v2: float) -> int:
        return self._geometry('surface', [12, surface, u1, u2, v1, v2])

    def offset(self, surface: int, d: float) -> int:
        return self._geometry('surface', [13, surface, d])

    def bezier_surface(self, poles) -> int:
        """A Bezier surface from rows of poles (U major)."""
        flat = [c for row in poles for p in row for c in p]
        return self._geometry('surface', [14, len(poles), len(poles[0]), *flat])

    def line2d(self, p, d) -> int:
        return self._geometry('pcurve', [20, *p, *d])

    def circle2d(self, c, r: float) -> int:
        return self._geometry('pcurve', [21, *c, r])

    def polygon3d(self, points) -> int:
        return self._geometry('poly3d', [30, len(points), *[c for p in points for c in p]])

    def polygon2d(self, points) -> int:
        return self._geometry('poly2d', [31, len(points), *[c for p in points for c in p]])

    def triangulation(self, nodes, triangles, purpose: int) -> int:
        spec = [32, len(nodes), *[c for p in nodes for c in p]]
        spec += [len(triangles), *[i for t in triangles for i in t], purpose]
        return self._geometry('triangulation', spec)

    def nodes(self, indices) -> int:
        return self._geometry('nodes', [33, len(indices), *indices])

    # Commands.
    def do(self, *command: float) -> None:
        """A command that makes no shape."""
        self.commands.append(list(command))

    def new(self, *command: float) -> int:
        """A command that makes one shape (it must not fail)."""
        self.commands.append(list(command))
        self.shapes += 1
        return self.shapes - 1

    def vertex(self, p, tol: float = 1e-7) -> int:
        return self.new(1, *p, tol)

    def edge(self, curve: int, loc: int = 0, tol: float = 1e-7) -> int:
        return self.new(4, curve, loc, tol)

    def face(self, surface: int, loc: int = 0, tol: float = 1e-7) -> int:
        return self.new(5, surface, loc, tol)

    def add(self, parent: int, child: int, orientation: int = FORWARD, loc: int = 0) -> None:
        self.do(102, parent, child, orientation, loc)

    def make(self, kind: int) -> int:
        """A TopoDS shape of `kind` (only compound 0, shell 3 and wire 5 are used)."""
        return self.new(101, kind)

    def moved(self, shape: int, orientation: int, loc: int) -> int:
        return self.new(105, shape, orientation, loc)

    def lock(self, shape: int) -> None:
        self.do(106, shape, 7, 1)

    def empty_copy(self, shape: int) -> int:
        return self.new(107, shape)

    def reversed(self, shape: int) -> int:
        return self.new(110, shape)

    def numbers(self) -> list[float]:
        out: list[float] = [len(self.locations)]
        for spec in self.locations:
            out += spec
        out.append(len(self.geometries))
        for spec in self.geometries:
            out += spec
        out.append(len(self.commands))
        for command in self.commands:
            out += command
        return out


def text(values: list[float]) -> str:
    return ' '.join(repr(v) if isinstance(v, float) else str(v) for v in values)


def line_edge(s: BRepScript, a, b, loc: int = 0) -> tuple[int, int, int]:
    """A line edge on [0, |ab|] between two new vertices (BRepLib_MakeEdge style)."""
    d = [b[i] - a[i] for i in range(3)]
    length = sum(c * c for c in d) ** 0.5
    e = s.edge(s.line(a, d), loc)
    v1, v2 = s.vertex(a), s.vertex(b)
    s.add(e, v1, FORWARD)
    s.add(e, v2, REVERSED)
    s.do(28, e, 0.0, length, 0)
    return e, v1, v2


def edge_scenario() -> BRepScript:
    """Edges with 3D curves, polygons, flags, vertex parameters, copies and transfers."""
    s = BRepScript()
    t = s.translation(1.0, 2.0, 3.0)
    r = s.rotation(0.0, 0.0, 1.0, 0.5)
    tr = s.product(t, r)
    sc = s.scale(2.0)
    e, v1, v2 = line_edge(s, (0.0, 0.0, 0.0), (3.0, 4.0, 0.0))
    s.do(34, v1, 0.0, e, 1e-5)
    s.do(34, v2, 5.0, e, 1e-6)
    inner = s.vertex((1.5, 2.0, 0.0), 1e-3)
    s.add(e, inner, INTERNAL)
    s.do(34, inner, 2.5, e, 1e-4)
    s.do(34, inner, 2.25, e, 1e-4)
    s.do(22, e, 1e-3)
    s.do(22, e, 1e-6)
    s.do(25, e, 0)
    s.do(26, e, 0)
    p3 = s.polygon3d([(0.0, 0.0, 0.0), (1.5, 2.0, 0.0), (3.0, 4.0, 0.0)])
    p3b = s.polygon3d([(0.0, 0.0, 0.0), (3.0, 4.0, 0.0)])
    s.do(15, e, p3, 0)
    s.do(15, e, p3b, t)
    circle = s.circle((0.0, 0.0, 0.0), (0.0, 0.0, 1.0), 2.0)
    ce = s.edge(circle, tr, 1e-4)
    cv = s.vertex((2.0, 0.0, 0.0))
    s.add(ce, cv, FORWARD)
    s.add(ce, cv, REVERSED)
    s.do(34, cv, 0.0, ce, 1e-7)
    s.moved(ce, REVERSED, t)
    bez = s.bezier([(0.0, 0.0, 0.0), (1.0, 2.0, 0.0), (3.0, 1.0, 1.0), (4.0, 0.0, 0.0)])
    be = s.edge(bez, sc)
    s.do(28, be, 0.1, 0.9, 1)
    s.edge(s.trimmed(circle, 0.5, 2.0), r)
    degenerate = s.new(3)
    dv = s.vertex((5.0, 5.0, 5.0))
    s.do(27, degenerate, 1)
    s.do(34, dv, 0.25, degenerate, 1e-7)
    polygon_edge = s.new(8, p3)
    s.do(15, polygon_edge, -1, 0)
    s.empty_copy(e)
    s.reversed(e)
    target = s.new(3)
    s.do(31, e, target)
    s.do(32, e, target, v1, v1)
    s.new(2)
    return s


def face_scenario() -> BRepScript:
    """Planar and cylindrical faces with p-curves, seams, polygons and triangulations."""
    s = BRepScript()
    t = s.translation(0.0, 0.0, 1.0)
    r = s.rotation(1.0, 0.0, 0.0, 0.25)
    plane = s.plane((0.0, 0.0, 0.0), (0.0, 0.0, 1.0))
    cylinder = s.cylinder((0.0, 0.0, 0.0), (0.0, 0.0, 1.0), 1.0)
    f = s.face(plane, 0, 1e-6)
    fl = s.face(plane, t, 1e-5)
    cf = s.face(cylinder, r)
    e, v1, v2 = line_edge(s, (0.0, 0.0, 0.0), (2.0, 0.0, 0.0))
    pc = s.line2d((0.0, 0.0), (1.0, 0.0))
    s.do(13, e, pc, f, 1e-5)
    s.do(11, e, pc, plane, t, 1e-6, 1, 0.0, 0.0, 2.0, 0.0)
    s.do(35, v1, 0.0, e, f, 1e-6)
    inner = s.vertex((1.0, 0.0, 0.0))
    s.add(e, inner, INTERNAL)
    s.do(35, inner, 1.0, e, f, 1e-6)
    s.do(36, inner, 1.0, e, plane, t, 1e-6)
    s.do(37, inner, 1.0, 0.0, f, 1e-6)
    s.do(37, inner, 1.5, 0.5, fl, 1e-6)
    s.do(30, e, f, 0.0, 2.0)
    s.do(29, e, plane, t, 0.25, 1.75)
    seam_line = s.line((1.0, 0.0, 0.0), (0.0, 0.0, 1.0))
    seam = s.edge(seam_line, r)
    s.do(28, seam, 0.0, 1.0, 0)
    c1 = s.line2d((0.0, 0.0), (0.0, 1.0))
    c2 = s.line2d((6.283185307179586, 0.0), (0.0, 1.0))
    s.do(14, seam, c1, c2, cf, 1e-7)
    s.do(12, seam, c1, c2, cylinder, 0, 1e-7, 1, 0.0, 0.0, 0.0, 1.0)
    s.do(49, seam, cylinder, 0, 0.5, 0.5, 1.5, 1.5)
    s.do(50, seam, cf, 0.25, 0.0, 0.25, 1.0)
    s.reversed(cf)
    s.reversed(seam)
    s.do(23, e, plane, cylinder, 0, r, G1)
    s.do(24, e, f, cf, C2)
    s.do(24, e, f, cf, C1)
    s.do(23, e, plane, plane, 0, 0, CN)
    p2 = s.polygon2d([(0.0, 0.0), (1.0, 0.0), (2.0, 0.0)])
    p2b = s.polygon2d([(0.0, 1.0), (2.0, 1.0)])
    s.do(20, e, p2, f)
    s.do(21, seam, p2, p2b, cf)
    s.do(18, e, p2b, plane, t)
    s.do(19, e, p2, p2b, plane, t)
    tri = s.triangulation([(0.0, 0.0, 0.0), (2.0, 0.0, 0.0), (0.0, 2.0, 0.0)], [(1, 2, 3)], 1)
    tri2 = s.triangulation([(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (0.0, 1.0, 0.0)], [(1, 2, 3)], 2)
    n1, n2 = s.nodes([1, 2]), s.nodes([2, 1])
    s.do(16, e, n1, tri, 0)
    s.do(17, seam, n1, n2, tri, r)
    s.do(16, e, -1, tri, t)
    s.new(9, n1, tri2, t)
    tf = s.new(7, tri)
    s.do(40, tf, tri2, 0)
    s.do(40, tf, tri, 0)
    s.do(40, tf, -1, 1)
    s.new(47, 2, tri, tri2, tri2)
    s.new(47, 2, tri, tri2, -1)
    s.do(41, f, 1e-4)
    s.do(42, f, 1)
    s.do(39, fl, cylinder, r, 1e-3)
    wire = s.make(5)
    s.add(wire, e, FORWARD)
    s.add(f, wire, FORWARD)
    s.empty_copy(f)
    s.empty_copy(seam)
    target = s.new(3)
    s.do(31, e, target)
    s.do(31, seam, target)
    s.do(32, e, target, inner, inner)
    # An edge with a p-curve only: the parameter of an INTERNAL vertex on the first p-curve.
    pe = s.new(3)
    s.do(13, pe, s.circle2d((0.0, 0.0), 1.0), f, 1e-7)
    pv = s.vertex((1.0, 0.0, 0.0))
    s.add(pe, pv, INTERNAL)
    s.do(35, pv, 0.0, pe, f, 1e-7)
    s.do(11, e, -1, plane, t, 1e-6, 0)
    return s


def plane_scenario() -> BRepScript:
    """Edges without p-curves on planes: the rejected ranges and the projection (`P`)."""
    s = BRepScript()
    t = s.translation(0.5, 0.0, 0.0)
    sc = s.scale(3.0)
    plane = s.plane((0.0, 0.0, 0.0), (0.0, 0.0, 1.0))
    s.rtrim(plane, -1.0, 1.0, -1.0, 1.0)
    s.offset(plane, 2.0)
    s.cylinder((0.0, 0.0, 0.0), (1.0, 0.0, 0.0), 2.0)
    bezier = s.bezier([(0.0, 0.0, 0.0), (0.5, 0.25, 0.0), (1.0, 0.0, 0.0)])
    circle = s.circle((0.0, 0.0, 0.0), (0.0, 0.0, 1.0), 1.0)
    outside = s.edge(bezier)
    s.do(28, outside, -0.000002673149646, 0.999997326850354, 1)
    inside = s.edge(bezier, t)
    s.do(28, inside, 0.25, 0.75, 1)
    flat = s.edge(circle)
    s.do(28, flat, 0.0, 0.0, 1)
    shifted = s.edge(circle, sc)
    s.do(28, shifted, 6.283185307179586, 12.566370614359172, 1)
    s.new(3)
    s.face(plane)
    s.face(plane, t)
    for vertex in [(0.0, 0.0, 0.0), (1.0, 0.0, 0.0)]:
        s.add(outside, s.vertex(vertex), FORWARD if vertex[0] == 0.0 else REVERSED)
    return s


def error_scenario() -> BRepScript:
    """Locked shapes, missing p-curves, infinite parameters and bad triangulation lists."""
    s = BRepScript()
    plane = s.plane((0.0, 0.0, 0.0), (0.0, 0.0, 1.0))
    other = s.plane((0.0, 0.0, 1.0), (0.0, 1.0, 0.0))
    line = s.line((0.0, 0.0, 0.0), (1.0, 0.0, 0.0))
    pc = s.line2d((0.0, 0.0), (1.0, 0.0))
    tri = s.triangulation([(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (0.0, 1.0, 0.0)], [(1, 2, 3)], 0)
    tri2 = s.triangulation([(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (0.0, 1.0, 0.0)], [(1, 2, 3)], 4)
    e = s.edge(line)
    f = s.face(plane)
    v = s.vertex((0.0, 0.0, 0.0))
    s.do(29, e, plane, 0, 0.0, 1.0)
    s.do(36, v, 0.5, e, plane, 0, 1e-7)
    s.do(34, v, 2e100, e, 1e-7)
    s.do(34, v, -3e100, e, 1e-7)
    s.do(47, 2, tri, -1, tri)
    s.do(47, 1, tri, tri2)
    s.do(13, e, pc, f, 1e-7)
    s.do(36, v, 0.5, e, other, 0, 1e-7)
    for shape in (e, f, v):
        s.lock(shape)
    for command in ([10, e, line, 0, 1e-7], [11, e, pc, plane, 0, 1e-7, 0], [22, e, 1e-3],
                    [25, e, 1], [26, e, 1], [27, e, 1], [28, e, 0.0, 1.0, 0],
                    [15, e, -1, 0], [16, e, -1, tri, 0], [18, e, -1, plane, 0],
                    [23, e, plane, other, 0, 0, C1], [33, v, 1.0, 1.0, 1.0, 1e-7],
                    [38, v, 1e-3], [34, v, 0.5, e, 1e-7], [37, v, 0.0, 0.0, f, 1e-7],
                    [39, f, plane, 0, 1e-7], [40, f, tri, 1], [41, f, 1e-3], [42, f, 1]):
        s.do(*command)
    s.new(1, 1.0, 2.0, 3.0, -1.0)
    return s


def append_tkbrep_brep_requests(requests: list[str]) -> None:
    """Append the BRep requests: every scenario through both operations."""
    for script in (edge_scenario(), face_scenario(), plane_scenario(), error_scenario()):
        numbers = text(script.numbers())
        requests.append(f'br_brep {numbers}')
        requests.append(f'br_brep_dump {numbers}')
