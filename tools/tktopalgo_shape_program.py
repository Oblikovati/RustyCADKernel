"""The shape programs of the TKTopAlgo probe (tools/oracle/tktopalgo_probe_shapes.cxx).

`ShapeProgram` records the ops that build a table of geometries and BRep_Builder shapes;
each method returns the index of what it adds, and `tokens()` gives the numbers of the
program. The op codes are documented in the native reader.
"""
from __future__ import annotations
import math

FORWARD, REVERSED, INTERNAL, EXTERNAL = 0, 1, 2, 3
WIRE, SHELL, SOLID, COMPOUND, COMPSOLID = 0, 1, 2, 3, 4


def num(value: float | int) -> str:
    """A number as the probes read it back exactly."""
    if isinstance(value, bool):
        return str(int(value))
    if isinstance(value, int):
        return str(value)
    return repr(float(value))


class ShapeProgram:
    """The ops of one shape program and the sizes of its tables."""

    def __init__(self) -> None:
        self.ops: list[list[float | int]] = []
        self.sizes = {'surface': 0, 'curve': 0, 'pcurve': 0, 'location': 1, 'triangulation': 0,
                      'shape': 0}

    def _add(self, table: str | None, code: int, *values: float | int) -> int:
        self.ops.append([code, *values])
        if table is None:
            return -1
        self.sizes[table] += 1
        return self.sizes[table]-1

    # Geometry tables.
    def surface(self, *record: float | int) -> int:
        """A tp_surface record (kind P N ...)."""
        return self._add('surface', 1, *record)

    def plane(self, origin, normal, xdir) -> int:
        """Geom_Plane(gp_Ax3(origin, normal, xdir))."""
        return self._add('surface', 5, *origin, *normal, *xdir)

    def curve(self, kind: int, *values: float | int) -> int:
        return self._add('curve', 2, kind, *values)

    def line(self, point, direction) -> int:
        return self.curve(0, *point, *direction)

    def pcurve(self, kind: int, *values: float | int) -> int:
        return self._add('pcurve', 3, kind, *values)

    def line2d(self, point, direction) -> int:
        return self.pcurve(0, *point, *direction)

    def location(self, kind: int, *values: float | int) -> int:
        return self._add('location', 4, kind, *values)

    def triangulation(self, nodes, triangles) -> int:
        flat = [len(nodes), *[c for n in nodes for c in n], len(triangles),
                *[i for t in triangles for i in t]]
        return self._add('triangulation', 28, *flat)

    # Shapes.
    def vertex(self, point, tol: float = 1e-7) -> int:
        return self._add('shape', 10, *point, tol)

    def edge(self) -> int:
        return self._add('shape', 11)

    def container(self, kind: int) -> int:
        return self._add('shape', 19, kind)

    def face(self, surface: int, location: int = 0, tol: float = 1e-7) -> int:
        return self._add('shape', 20, surface, location, tol)

    def oriented(self, shape: int, orientation: int) -> int:
        return self._add('shape', 21, shape, orientation)

    def located(self, shape: int, location: int) -> int:
        return self._add('shape', 22, shape, location)

    # Changes of existing shapes.
    def edge_curve(self, edge: int, curve: int, location: int = 0, tol: float = 1e-7) -> None:
        self._add(None, 12, edge, curve, location, tol)

    def edge_pcurve(self, edge: int, pcurve: int, surface: int, location: int = 0,
                    tol: float = 1e-7) -> None:
        self._add(None, 13, edge, pcurve, surface, location, tol)

    def edge_pcurves(self, edge: int, pc1: int, pc2: int, surface: int, location: int = 0,
                     tol: float = 1e-7) -> None:
        self._add(None, 14, edge, pc1, pc2, surface, location, tol)

    def edge_range(self, edge: int, first: float, last: float, only3d: bool = False) -> None:
        self._add(None, 15, edge, first, last, only3d)

    def edge_range_on(self, edge: int, surface: int, location: int, first: float, last: float) -> None:
        self._add(None, 16, edge, surface, location, first, last)

    def edge_flags(self, edge: int, same_parameter: bool, same_range: bool, degenerated: bool) -> None:
        self._add(None, 17, edge, same_parameter, same_range, degenerated)

    def add(self, parent: int, child: int, orientation: int = FORWARD) -> None:
        self._add(None, 18, parent, child, orientation)

    def vertex_on_curve(self, vertex: int, parameter: float, edge: int, tol: float = 1e-7) -> None:
        self._add(None, 23, vertex, parameter, edge, tol)

    def vertex_on_pcurve(self, vertex: int, parameter: float, edge: int, surface: int,
                         location: int = 0, tol: float = 1e-7) -> None:
        self._add(None, 24, vertex, parameter, edge, surface, location, tol)

    def vertex_on_face(self, vertex: int, u: float, v: float, face: int, tol: float = 1e-7) -> None:
        self._add(None, 25, vertex, u, v, face, tol)

    def flag(self, shape: int, which: int, value: bool) -> None:
        """Flag 0 Closed, 1 Infinite."""
        self._add(None, 26, shape, which, value)

    def tolerance(self, shape: int, tol: float) -> None:
        self._add(None, 27, shape, tol)

    def polygon(self, edge: int, triangulation: int, location: int, *polygons) -> None:
        """One polygon (op 29) or two (op 30): (nodes, parameters or None, deflection)."""
        flat: list[float | int] = []
        for nodes, parameters, deflection in polygons:
            flat += [len(nodes), *nodes, parameters is not None, *(parameters or []), deflection]
        self._add(None, 29 if len(polygons) == 1 else 30, edge, triangulation, location, *flat)

    def pcurve_on_face(self, edge: int, pcurve: int, face: int, tol: float = 1e-7) -> None:
        self._add(None, 33, edge, pcurve, face, tol)

    def pcurves_on_face(self, edge: int, pc1: int, pc2: int, face: int, tol: float = 1e-7) -> None:
        self._add(None, 34, edge, pc1, pc2, face, tol)

    def range_on_face(self, edge: int, face: int, first: float, last: float) -> None:
        self._add(None, 35, edge, face, first, last)

    def extra_curve3d(self, edge: int, curve: int, location: int = 0) -> None:
        self._add(None, 36, edge, curve, location)

    def tokens(self) -> str:
        """The program as request numbers."""
        return ' '.join([str(len(self.ops)), *[num(v) for op in self.ops for v in op]])


def sub(a, b):
    return tuple(x-y for x, y in zip(a, b))


def dot(a, b) -> float:
    return sum(x*y for x, y in zip(a, b))


def cross(a, b):
    return (a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0])


def unit(a):
    n = math.sqrt(dot(a, a))
    return tuple(x/n for x in a)
