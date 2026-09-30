"""The shapes of the BRepCheck corpus, built as shape programs (tktopalgo_shape_program.py).

`PlanarShapes` builds faces bounded by polygons on their planes: each edge has a 3d line
and one pcurve per face (a line in the plane frame, isometric to the 3d line), so the
shapes are valid until a family changes them. `cylinder_face` builds the lateral face
of a cylinder with its seam. Every builder returns the table indices it created.
"""
from __future__ import annotations
import math
from tktopalgo_shape_program import (FORWARD, REVERSED, SHELL, SOLID, WIRE, ShapeProgram, cross, dot,
                                     sub, unit)


class PlanarShapes:
    """Faces on planes bounded by vertex polygons, sharing their vertices and edges.

    `points` are the 3d vertex points; a face is given by its loops of point indices (the
    first loop outer, counter-clockwise seen from the normal side); an edge is shared by
    every loop that uses the same two points.
    """

    def __init__(self, program: ShapeProgram, points, tol: float = 1e-7) -> None:
        self.p = program
        self.points = [tuple(float(c) for c in point) for point in points]
        self.tol = tol
        self.vertices = [program.vertex(point, tol) for point in self.points]
        self.edges: dict[tuple[int, int], int] = {}
        self.lengths: dict[tuple[int, int], float] = {}
        self.faces: list[int] = []
        self.wires: list[list[int]] = []
        self.surfaces: list[int] = []
        self.frames: list[tuple] = []

    def frame(self, loop, normal=None):
        """The plane frame of a loop: origin at its first point, X along its first side."""
        origin = self.points[loop[0]]
        xdir = unit(sub(self.points[loop[1]], origin))
        if normal is None:
            normal = unit(cross(xdir, sub(self.points[loop[2]], origin)))
        return origin, unit(normal), xdir, cross(unit(normal), xdir)

    def uv(self, frame, index: int):
        origin, _, xdir, ydir = frame
        d = sub(self.points[index], origin)
        return (dot(d, xdir), dot(d, ydir))

    def edge(self, a: int, b: int) -> tuple[int, int]:
        """The edge between points a and b (created once, from the lower index)."""
        key = (min(a, b), max(a, b))
        if key not in self.edges:
            e = self.p.edge()
            start, end = self.points[key[0]], self.points[key[1]]
            length = math.sqrt(dot(sub(end, start), sub(end, start)))
            self.p.edge_curve(e, self.p.line(start, unit(sub(end, start))), 0, self.tol)
            self.p.add(e, self.vertices[key[0]], FORWARD)
            self.p.add(e, self.vertices[key[1]], REVERSED)
            self.edges[key] = e
            self.lengths[key] = length
        return key

    def pcurve(self, key, surface: int, frame, shift=(0.0, 0.0)) -> None:
        """The pcurve of edge `key` on the plane, isometric to its 3d line."""
        a, b = self.uv(frame, key[0]), self.uv(frame, key[1])
        start = (a[0]+shift[0], a[1]+shift[1])
        pc = self.p.line2d(start, unit(sub(b, a)))
        self.p.edge_pcurve(self.edges[key], pc, surface, 0, self.tol)

    def face(self, loops, normal=None, skip_pcurves=(), shifts=None, orientations=None) -> int:
        """A face bounded by `loops`; `skip_pcurves` names edges (point pairs) left
        without pcurve on this face, `shifts` moves some pcurves, `orientations` gives
        the wire orientations."""
        frame = self.frame(loops[0], normal)
        surface = self.p.plane(frame[0], frame[1], frame[2])
        face = self.p.face(surface, 0, self.tol)
        wires = []
        for k, loop in enumerate(loops):
            wire = self.p.container(WIRE)
            for a, b in zip(loop, loop[1:]+loop[:1]):
                key = self.edge(a, b)
                if key not in skip_pcurves:
                    self.pcurve(key, surface, frame, (shifts or {}).get(key, (0.0, 0.0)))
                self.p.add(wire, self.edges[key], FORWARD if a < b else REVERSED)
            wire_orientation = (orientations or [FORWARD]*len(loops))[k]
            if wire_orientation != FORWARD:
                wire = self.p.oriented(wire, wire_orientation)
            self.p.add(face, wire, FORWARD)
            wires.append(wire)
        self.faces.append(face)
        self.wires.append(wires)
        self.surfaces.append(surface)
        self.frames.append(frame)
        return face

    def finish(self) -> None:
        """The ranges of every edge, on all its representations."""
        for key, e in self.edges.items():
            self.p.edge_range(e, 0.0, self.lengths[key], False)

    def shell(self, faces=None, orientations=None) -> int:
        shell = self.p.container(SHELL)
        for k, face in enumerate(faces if faces is not None else self.faces):
            self.p.add(shell, face, (orientations or {}).get(k, FORWARD))
        return shell


CUBE_POINTS = [(0, 0, 0), (1, 0, 0), (1, 1, 0), (0, 1, 0), (0, 0, 1), (1, 0, 1), (1, 1, 1), (0, 1, 1)]
# The faces of the unit cube, counter-clockwise seen from outside.
CUBE_FACES = [[0, 3, 2, 1], [4, 5, 6, 7], [0, 1, 5, 4], [1, 2, 6, 5], [2, 3, 7, 6], [3, 0, 4, 7]]


def cube(program: ShapeProgram, scale: float = 1.0, faces=None, tol: float = 1e-7) -> PlanarShapes:
    """The faces of a cube (all six by default), its edges finished."""
    shapes = PlanarShapes(program, [tuple(scale*c for c in p) for p in CUBE_POINTS], tol)
    for loop in (faces if faces is not None else CUBE_FACES):
        shapes.face([loop])
    shapes.finish()
    return shapes


def solid_of(program: ShapeProgram, shell: int, closed: bool = True) -> int:
    """A solid holding `shell`, with the shell's Closed flag."""
    program.flag(shell, 0, closed)
    solid = program.container(SOLID)
    program.add(solid, shell, FORWARD)
    return solid


def square(program: ShapeProgram, size: float = 1.0, holes=(), tol: float = 1e-7, **face_options):
    """A square face on z = 0 with square holes (x0, y0, side) wound clockwise."""
    points = [(0, 0, 0), (size, 0, 0), (size, size, 0), (0, size, 0)]
    loops = [[0, 1, 2, 3]]
    for x0, y0, side in holes:
        base = len(points)
        points += [(x0, y0, 0), (x0, y0+side, 0), (x0+side, y0+side, 0), (x0+side, y0, 0)]
        loops.append([base, base+1, base+2, base+3])
    shapes = PlanarShapes(program, points, tol)
    shapes.face(loops, normal=(0.0, 0.0, 1.0), **face_options)
    shapes.finish()
    return shapes


def cylinder_face(program: ShapeProgram, radius: float = 1.0, height: float = 2.0,
                  seam_shift: float = 0.0, tol: float = 1e-7) -> dict:
    """The lateral face of the cylinder on the Z axis: the bottom and top circles and
    the seam (u = 2 Pi FORWARD, u = 0 REVERSED; `seam_shift` moves the first pcurve)."""
    p = program
    surface = p.surface(1, 0, 0, 0, 0, 0, 1, radius)
    v0 = p.vertex((radius, 0.0, 0.0), tol)
    v1 = p.vertex((radius, 0.0, height), tol)
    edges = {}
    for name, z, v in (('bottom', 0.0, v0), ('top', height, v1)):
        e = p.edge()
        p.edge_curve(e, p.curve(1, 0, 0, z, 0, 0, 1, radius), 0, tol)
        p.edge_pcurve(e, p.line2d((0.0, z), (1.0, 0.0)), surface, 0, tol)
        p.add(e, v, FORWARD)
        p.add(e, v, REVERSED)
        p.edge_range(e, 0.0, 2*math.pi)
        edges[name] = e
    seam = p.edge()
    p.edge_curve(seam, p.line((radius, 0.0, 0.0), (0.0, 0.0, 1.0)), 0, tol)
    p.edge_pcurves(seam, p.line2d((2*math.pi+seam_shift, 0.0), (0.0, 1.0)),
                   p.line2d((0.0, 0.0), (0.0, 1.0)), surface, 0, tol)
    p.add(seam, v0, FORWARD)
    p.add(seam, v1, REVERSED)
    p.edge_range(seam, 0.0, height)
    wire = p.container(WIRE)
    for e, o in ((edges['bottom'], FORWARD), (seam, FORWARD), (edges['top'], REVERSED),
                 (seam, REVERSED)):
        p.add(wire, e, o)
    face = p.face(surface, 0, tol)
    p.add(face, wire, FORWARD)
    return {'surface': surface, 'face': face, 'wire': wire, 'seam': seam, 'v0': v0, 'v1': v1, **edges}
