"""Face records of the TKTopAlgo oracle (tools/oracle/tktopalgo_probe_face.cxx, `tp_face`).

A face is built with BRep_Builder from its surface, vertices, pcurve edges and wires, so
both probes build the same shape without BRepBuilderAPI. `FaceSpec.numbers()` spells the
record; the named constructors below give the faces of the BRepClass corpus: rectangles
with and without holes, a disc, the lateral face of a cylinder (with its seam), a sphere
face (seam and degenerated poles), a polygon of Bezier edges, a rectangle with a gap at a
high-tolerance vertex, an infinite and a half-infinite strip, a trimmed plane, a face with
an internal edge, a triangle with a cusp Bezier edge, a rectangle with a nearly flat
Bezier edge, a cylinder face with a reversed pcurve, a square with an edge without pcurve
and a face without wires.
"""
from __future__ import annotations
import math
from dataclasses import dataclass, field

PLANE, CYLINDER, SPHERE, TRIMMED, CONE, TORUS, BEZIER_SURFACE, BSPLINE_SURFACE = range(8)
FORWARD, REVERSED, INTERNAL, EXTERNAL = range(4)
SEGMENT, ARC, BEZIER, SEAM, LINE, NO_PCURVE, SEGMENT_3D = range(7)
# Precision::Infinite(): a parameter at least half of it is infinite.
INFINITE = 2e100
Z_PLANE = [PLANE, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0]


@dataclass
class FaceSpec:
    """One `tp_face` record: surface fields, tolerance, vertices, edges, wires, orientation."""
    surface: list[float]
    tolerance: float = 1e-7
    vertices: list[list[float]] = field(default_factory=list)
    edges: list[list[float]] = field(default_factory=list)
    wires: list[tuple[int, list[tuple[int, int]]]] = field(default_factory=list)
    orientation: int = FORWARD

    def vertex(self, u: float, v: float, tol: float = 1e-7) -> int:
        """Add a vertex at the surface point of (u, v); its index."""
        self.vertices.append([u, v, tol])
        return len(self.vertices) - 1

    def edge(self, kind: int, pcurve: list[float], first: int = -1, last: int = -1,
             degenerated: bool = False, tol: float = 1e-7) -> int:
        """Add an edge with its pcurve fields and vertex indices; its index."""
        self.edges.append([kind, first, last, int(degenerated), tol, *pcurve])
        return len(self.edges) - 1

    def wire(self, edges: list[tuple[int, int]], orientation: int = FORWARD) -> None:
        """Add a wire of (edge index, orientation) pairs."""
        self.wires.append((orientation, edges))

    def numbers(self) -> list[float]:
        """The record fields in `tp_face` order."""
        values: list[float] = [*self.surface, self.tolerance, len(self.vertices)]
        for vertex in self.vertices:
            values.extend(vertex)
        values.append(len(self.edges))
        for edge in self.edges:
            values.extend(edge)
        values.append(len(self.wires))
        for orientation, edges in self.wires:
            values.extend([orientation, len(edges)])
            for index, edge_orientation in edges:
                values.extend([index, edge_orientation])
        values.append(self.orientation)
        return values


def polygon_wire(face: FaceSpec, points: list[tuple[float, float]], tol: float = 1e-7,
                 orientation: int = FORWARD) -> list[int]:
    """A closed polygon of segment edges through `points` (in order); the edge indices."""
    vertices = [face.vertex(u, v, tol) for u, v in points]
    edges = [face.edge(SEGMENT, [*points[i], *points[(i + 1) % len(points)]], vertices[i],
                       vertices[(i + 1) % len(points)]) for i in range(len(points))]
    face.wire([(edge, FORWARD) for edge in edges], orientation)
    return edges


def rectangle(x0: float, y0: float, x1: float, y1: float) -> list[tuple[float, float]]:
    """The corners of a rectangle, counterclockwise from (x0, y0)."""
    return [(x0, y0), (x1, y0), (x1, y1), (x0, y1)]


def square_face(size: float = 2.0, orientation: int = FORWARD) -> FaceSpec:
    """A square [0, size]^2 on the XY plane."""
    face = FaceSpec(list(Z_PLANE), orientation=orientation)
    polygon_wire(face, rectangle(0.0, 0.0, size, size))
    return face


def holed_face(hole_edges_reversed: bool = False) -> FaceSpec:
    """The square [0, 10]^2 with the hole [3, 7]^2: a reversed counterclockwise wire, or
    a forward wire of reversed edges."""
    face = FaceSpec(list(Z_PLANE))
    polygon_wire(face, rectangle(0.0, 0.0, 10.0, 10.0))
    if not hole_edges_reversed:
        polygon_wire(face, rectangle(3.0, 3.0, 7.0, 7.0), orientation=REVERSED)
        return face
    inner = polygon_wire(face, rectangle(3.0, 3.0, 7.0, 7.0))
    face.wires[-1] = (FORWARD, [(edge, REVERSED) for edge in reversed(inner)])
    return face


def disc_face(radius: float = 2.0) -> FaceSpec:
    """A disc: one closed circle edge with one vertex."""
    face = FaceSpec(list(Z_PLANE))
    vertex = face.vertex(radius, 0.0)
    edge = face.edge(ARC, [0.0, 0.0, radius, 0.0, 2.0 * math.pi, 1], vertex, vertex)
    face.wire([(edge, FORWARD)])
    return face


def cylinder_face(radius: float = 2.0, height: float = 5.0) -> FaceSpec:
    """The lateral face of a cylinder, u in [0, 2pi], v in [0, height], with its seam."""
    face = FaceSpec([CYLINDER, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, radius])
    bottom, top = face.vertex(0.0, 0.0), face.vertex(0.0, height)
    two_pi = 2.0 * math.pi
    e_bottom = face.edge(LINE, [0.0, 0.0, 1.0, 0.0, 0.0, two_pi], bottom, bottom)
    e_seam = face.edge(SEAM, [two_pi, 0.0, two_pi, height, -two_pi, 0.0], bottom, top)
    e_top = face.edge(LINE, [0.0, height, 1.0, 0.0, 0.0, two_pi], top, top)
    face.wire([(e_bottom, FORWARD), (e_seam, FORWARD), (e_top, REVERSED), (e_seam, REVERSED)])
    return face


def sphere_face(radius: float = 2.0) -> FaceSpec:
    """A whole sphere face: the seam and the two degenerated pole edges."""
    face = FaceSpec([SPHERE, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, radius])
    half, two_pi = 0.5 * math.pi, 2.0 * math.pi
    south, north = face.vertex(0.0, -half), face.vertex(0.0, half)
    e_south = face.edge(LINE, [0.0, -half, 1.0, 0.0, 0.0, two_pi], south, south, True)
    e_seam = face.edge(SEAM, [two_pi, -half, two_pi, half, -two_pi, 0.0], south, north)
    e_north = face.edge(LINE, [0.0, half, 1.0, 0.0, 0.0, two_pi], north, north, True)
    face.wire([(e_south, FORWARD), (e_seam, FORWARD), (e_north, REVERSED), (e_seam, REVERSED)])
    return face


def bezier_polygon_face(sides: int = 12, radius: float = 5.0, bulge: float = 0.2) -> FaceSpec:
    """A polygon of `sides` quadratic Bezier edges bulging outwards by `bulge`."""
    face = FaceSpec(list(Z_PLANE))
    corners = [(radius * math.cos(2 * math.pi * i / sides), radius * math.sin(2 * math.pi * i / sides))
               for i in range(sides)]
    vertices = [face.vertex(u, v) for u, v in corners]
    edges = []
    for i in range(sides):
        a, b = corners[i], corners[(i + 1) % sides]
        middle = ((a[0] + b[0]) * 0.5 * (1 + bulge), (a[1] + b[1]) * 0.5 * (1 + bulge))
        edges.append(face.edge(BEZIER, [3, *a, *middle, *b], vertices[i], vertices[(i + 1) % sides]))
    face.wire([(edge, FORWARD) for edge in edges])
    return face


def gap_face(gap: float = 0.2, vertex_tol: float = 0.5) -> FaceSpec:
    """The square [0, 4]^2 whose first corner edges stop `gap` short of the corner (4, 0),
    where their shared vertex has the tolerance `vertex_tol`."""
    face = FaceSpec(list(Z_PLANE))
    v0, v1, v2, v3 = face.vertex(0.0, 0.0), face.vertex(4.0, 0.0, vertex_tol), face.vertex(4.0, 4.0), face.vertex(0.0, 4.0)
    edges = [face.edge(SEGMENT, [0.0, 0.0, 4.0 - gap, 0.0], v0, v1),
             face.edge(SEGMENT, [4.0, gap, 4.0, 4.0], v1, v2),
             face.edge(SEGMENT, [4.0, 4.0, 0.0, 4.0], v2, v3),
             face.edge(SEGMENT, [0.0, 4.0, 0.0, 0.0], v3, v0)]
    face.wire([(edge, FORWARD) for edge in edges])
    return face


def strip_face() -> FaceSpec:
    """The strip 0 <= v <= 1 between two infinite lines without vertices."""
    face = FaceSpec(list(Z_PLANE))
    low = face.edge(LINE, [0.0, 0.0, 1.0, 0.0, -INFINITE, INFINITE])
    high = face.edge(LINE, [0.0, 1.0, -1.0, 0.0, -INFINITE, INFINITE])
    face.wire([(low, FORWARD), (high, FORWARD)])
    return face


def trimmed_face() -> FaceSpec:
    """The square [0, 10]^2 on a plane trimmed to [-1, 11]^2 (finite surface bounds)."""
    face = FaceSpec([TRIMMED, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, -1.0, 11.0, -1.0, 11.0])
    polygon_wire(face, rectangle(0.0, 0.0, 10.0, 10.0))
    return face


def internal_edge_face() -> FaceSpec:
    """The square [0, 4]^2 with an INTERNAL edge from (1, 1) to (3, 3) in a second wire."""
    face = FaceSpec(list(Z_PLANE))
    polygon_wire(face, rectangle(0.0, 0.0, 4.0, 4.0))
    a, b = face.vertex(1.0, 1.0), face.vertex(3.0, 3.0)
    inner = face.edge(SEGMENT, [1.0, 1.0, 3.0, 3.0], a, b)
    face.wire([(inner, INTERNAL)])
    return face


def cusp_face() -> FaceSpec:
    """The triangle (0, 0) (4, 0) (2, 3) whose closing edge is a Bezier curve of degree 8
    with four coincident poles at each end: its first three derivatives vanish there, so
    its tangent is undefined at both ends."""
    face = FaceSpec(list(Z_PLANE))
    a, b, c = face.vertex(0.0, 0.0), face.vertex(4.0, 0.0), face.vertex(2.0, 3.0)
    edges = [face.edge(SEGMENT, [0.0, 0.0, 4.0, 0.0], a, b), face.edge(SEGMENT, [4.0, 0.0, 2.0, 3.0], b, c),
             face.edge(BEZIER, [9, *[2.0, 3.0] * 4, 0.5, 1.0, *[0.0, 0.0] * 4], c, a)]
    face.wire([(edge, FORWARD) for edge in edges])
    return face


def flat_face() -> FaceSpec:
    """The rectangle [0, 4] x [0, 2] whose bottom edge is a nearly flat quadratic Bezier
    curve: segments from far along its axis are almost tangent to it everywhere."""
    face = FaceSpec(list(Z_PLANE))
    a, b, c, d = face.vertex(0.0, 0.0), face.vertex(4.0, 0.0), face.vertex(4.0, 2.0), face.vertex(0.0, 2.0)
    edges = [face.edge(BEZIER, [3, 0.0, 0.0, 2.0, -0.001, 4.0, 0.0], a, b),
             face.edge(SEGMENT, [4.0, 0.0, 4.0, 2.0], b, c), face.edge(SEGMENT, [4.0, 2.0, 0.0, 2.0], c, d),
             face.edge(SEGMENT, [0.0, 2.0, 0.0, 0.0], d, a)]
    face.wire([(edge, FORWARD) for edge in edges])
    return face


def flipped_cylinder_face(radius: float = 2.0, height: float = 5.0) -> FaceSpec:
    """The lateral cylinder face whose top edge has a pcurve running towards -u, used
    FORWARD (so the tangent of the pcurve has a negative u component)."""
    face = cylinder_face(radius, height)
    two_pi = 2.0 * math.pi
    face.edges[2] = [LINE, *face.edges[2][1:5], two_pi, height, -1.0, 0.0, 0.0, two_pi]
    orientation, edges = face.wires[0]
    face.wires[0] = (orientation, [edges[0], edges[1], (edges[2][0], FORWARD), edges[3]])
    return face


def half_strip_face() -> FaceSpec:
    """The half strip u >= 0, 0 <= v <= 1: two half-infinite lines and a segment."""
    face = FaceSpec(list(Z_PLANE))
    a, b = face.vertex(0.0, 0.0), face.vertex(0.0, 1.0)
    low = face.edge(LINE, [0.0, 0.0, 1.0, 0.0, 0.0, INFINITE], a)
    high = face.edge(LINE, [0.0, 1.0, -1.0, 0.0, -INFINITE, 0.0], -1, b)
    side = face.edge(SEGMENT, [0.0, 1.0, 0.0, 0.0], b, a)
    face.wire([(low, FORWARD), (high, FORWARD), (side, FORWARD)])
    return face


def no_pcurve_face() -> FaceSpec:
    """The square [0, 2]^2 whose last edge has no pcurve (and no 3d curve)."""
    face = FaceSpec(list(Z_PLANE))
    edges = polygon_wire(face, rectangle(0.0, 0.0, 2.0, 2.0))
    face.edges[edges[-1]] = [NO_PCURVE, *face.edges[edges[-1]][1:5]]
    return face


def empty_face() -> FaceSpec:
    """A plane face without wires."""
    return FaceSpec(list(Z_PLANE))


def named_faces() -> dict[str, FaceSpec]:
    """Every face of the corpus by name, in a fixed order."""
    return {'square': square_face(), 'square_reversed': square_face(orientation=REVERSED),
            'holed': holed_face(), 'holed_edges': holed_face(True), 'disc': disc_face(),
            'cylinder': cylinder_face(), 'sphere': sphere_face(), 'bezier': bezier_polygon_face(),
            'gap': gap_face(), 'strip': strip_face(), 'trimmed': trimmed_face(),
            'internal': internal_edge_face(), 'empty': empty_face(), 'cusp': cusp_face(),
            'half_strip': half_strip_face(), 'no_pcurve': no_pcurve_face(), 'flat': flat_face(),
            'cylinder_flipped': flipped_cylinder_face()}


# The number of pcurve fields of each edge kind (a Bezier curve counts its poles).
PCURVE_FIELDS = {SEGMENT: 4, ARC: 6, SEAM: 6, LINE: 6, NO_PCURVE: 0, SEGMENT_3D: 4}


def surface_length(values: list[float], start: int) -> int:
    """The number of fields of the surface record at `start` (kind, P, N and its own)."""
    kind, index = values[start], start + 7
    if kind in (CYLINDER, SPHERE):
        return 8
    if kind in (CONE, TORUS):
        return 9
    if kind == TRIMMED:
        return 11
    if kind == BEZIER_SURFACE:
        return 9 + 3 * int(values[index]) * int(values[index + 1])
    if kind == BSPLINE_SURFACE:
        index += 4 + 3 * int(values[index + 2]) * int(values[index + 3])
        index += 1 + 2 * int(values[index])
        index += 1 + 2 * int(values[index])
        return index - start
    return 7


def record_length(values: list[float], start: int = 0) -> int:
    """The number of fields of the face record at `start`, read as `tp_face` reads it;
    record_length(square_face().numbers()) == 70."""
    index = start + surface_length(values, start)
    index += 1  # the face tolerance
    index += 1 + 3 * int(values[index])
    nb_edges, index = int(values[index]), index + 1
    for _ in range(nb_edges):
        kind, index = int(values[index]), index + 5
        index += 1 + 2 * int(values[index]) if kind == BEZIER else PCURVE_FIELDS[kind]
    nb_wires, index = int(values[index]), index + 1
    for _ in range(nb_wires):
        index += 2 + 2 * int(values[index + 1])
    return index + 1 - start
