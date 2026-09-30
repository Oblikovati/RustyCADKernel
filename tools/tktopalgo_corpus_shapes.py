"""Shape records of the TKTopAlgo oracle (tools/oracle/tktopalgo_probe_shape.cxx, `tp_shape`).

A shape is built with BRep_Builder from its vertices, edges (3d curves, 3d polygons),
faces (surfaces, wires with pcurves, triangulations with polygons on them), shells and
top-level items, so both probes build the same shape without BRepBuilderAPI.
`ShapeSpec.numbers()` spells the record; tools/tktopalgo_corpus_solids.py holds the
named constructors (boxes, cylinders, spheres, tori, holed and open shells...).
"""
from __future__ import annotations
from dataclasses import dataclass, field

# Curve kinds, surface kinds, pcurve kinds and top-level item kinds of the record.
NO_CURVE, LINE, CIRCLE, BEZIER, BSPLINE, ELLIPSE, PARABOLA = range(7)
NO_SURFACE, PLANE, CYLINDER, SPHERE, CONE, TORUS, BEZIER_SURFACE, BSPLINE_SURFACE = -1, 0, 1, 2, 3, 4, 5, 6
EXTRUSION, REVOLUTION, OFFSET = 7, 8, 9
NO_PCURVE, PLINE, PCIRCLE, PBEZIER, PBSPLINE, PELLIPSE = range(6)
SOLID, SHELL, FACE, EDGE, VERTEX, AGAIN = range(6)
FORWARD, REVERSED, INTERNAL, EXTERNAL = range(4)
IDENTITY = [0]


def translation(dx: float, dy: float, dz: float) -> list[float]:
    """A location record: the translation by (dx, dy, dz)."""
    return [1, dx, dy, dz]


def rotation(point: list[float], axis: list[float], angle: float) -> list[float]:
    """A location record: the rotation about the axis through `point`."""
    return [2, *point, *axis, angle]


def point_mirror(point: list[float]) -> list[float]:
    """A location record: the symmetry about `point` (not rigid: it reverses)."""
    return [3, *point]


def scaling(point: list[float], factor: float) -> list[float]:
    """A location record: the scale about `point` (not rigid)."""
    return [4, *point, factor]


@dataclass
class Mesh:
    """A face triangulation and the polygons of some edges on it (1-based node indices)."""
    nodes: list[list[float]]
    triangles: list[tuple[int, int, int]]
    deflection: float
    polygons: list[tuple[int, list[int], float]] = field(default_factory=list)

    def numbers(self) -> list[float]:
        """`nt (x y z)... ntri (a b c)... defl npol (edge n (node)... defl)...`."""
        values: list[float] = [len(self.nodes)]
        for node in self.nodes:
            values.extend(node)
        values.append(len(self.triangles))
        for triangle in self.triangles:
            values.extend(triangle)
        values.extend([self.deflection, len(self.polygons)])
        for edge, indices, deflection in self.polygons:
            values.extend([edge, len(indices), *indices, deflection])
        return values


@dataclass
class FaceRecord:
    """One face: surface fields, tolerance, natural flag, wires, mesh, orientation."""
    surface: list[float]
    tolerance: float = 1e-7
    natural: bool = False
    wires: list[tuple[int, list[tuple[int, int, list[float]]]]] = field(default_factory=list)
    mesh: Mesh | None = None
    orientation: int = FORWARD

    def wire(self, uses: list[tuple[int, int, list[float]]], orientation: int = FORWARD) -> None:
        """Add a wire of (edge index, orientation, pcurve fields) uses."""
        self.wires.append((orientation, uses))

    def numbers(self) -> list[float]:
        """The face fields in `tp_shape_face` order."""
        values: list[float] = [*self.surface, self.tolerance, int(self.natural), len(self.wires)]
        for orientation, uses in self.wires:
            values.extend([orientation, len(uses)])
            for edge, edge_orientation, pcurve in uses:
                values.extend([edge, edge_orientation, *pcurve])
        values.extend(self.mesh.numbers() if self.mesh else [0])
        values.append(self.orientation)
        return values


@dataclass
class ShapeSpec:
    """One `tp_shape` record: vertices, edges, faces, shells, top items and location."""
    vertices: list[list[float]] = field(default_factory=list)
    edges: list[list[float]] = field(default_factory=list)
    faces: list[FaceRecord] = field(default_factory=list)
    shells: list[tuple[int, bool, list[tuple[int, int, list[float]]]]] = field(default_factory=list)
    # `(kind, index)` items, or `(AGAIN, item, *location, orientation)` for an earlier item
    # again with the same TShape.
    top: list[tuple[float, ...]] = field(default_factory=lambda: [(SOLID, 0)])
    location: list[float] = field(default_factory=lambda: list(IDENTITY))

    def vertex(self, point: list[float], tol: float = 1e-7) -> int:
        """Add a vertex; its index."""
        self.vertices.append([*point, tol])
        return len(self.vertices) - 1

    def edge(self, curve: list[float], first_vertex: int, last_vertex: int, span: tuple[float, float],
             tol: float = 1e-7, degenerated: bool = False,
             polygon: tuple[list[list[float]], float] | None = None) -> int:
        """Add an edge (curve fields, vertex indices, range, 3d polygon); its index."""
        values = [*curve, first_vertex, last_vertex, int(degenerated), tol, *span]
        if polygon is None:
            values.append(0)
        else:
            nodes, deflection = polygon
            values.append(len(nodes))
            for node in nodes:
                values.extend(node)
            values.append(deflection)
        self.edges.append(values)
        return len(self.edges) - 1

    def face(self, record: FaceRecord) -> int:
        """Add a face; its index."""
        self.faces.append(record)
        return len(self.faces) - 1

    def shell(self, faces: list[int], closed: bool = True, orientation: int = FORWARD,
              face_orientation: int = FORWARD) -> int:
        """Add a shell of faces (same orientation, no location); its index."""
        self.shells.append((orientation, closed, [(f, face_orientation, list(IDENTITY)) for f in faces]))
        return len(self.shells) - 1

    def numbers(self) -> list[float]:
        """The record fields in `tp_shape` order."""
        values: list[float] = [len(self.vertices)]
        for vertex in self.vertices:
            values.extend(vertex)
        values.append(len(self.edges))
        for edge in self.edges:
            values.extend(edge)
        values.append(len(self.faces))
        for face in self.faces:
            values.extend(face.numbers())
        values.append(len(self.shells))
        for orientation, closed, uses in self.shells:
            values.extend([orientation, int(closed), len(uses)])
            for face, face_orientation, location in uses:
                values.extend([face, face_orientation, *location])
        values.append(len(self.top))
        for item in self.top:
            values.extend(item)
        values.extend(self.location)
        return values
