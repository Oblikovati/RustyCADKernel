"""The BRepCheck_Edge requests of the TKTopAlgo corpus: the reference curve of Minimum
(missing, multiple, degenerated, flags, ranges), the curves on the faces (shifted,
re-ranged, on a closed surface, projected on a plane), the solid contexts, Tolerance
and CheckPolygonOnTriangulation."""
from __future__ import annotations
import math
from tktopalgo_corpus_brepcheck import (CONTEXTS, EDGE, FAIL, FLAGS, IN_CONTEXT, PARALLEL, STATUS,
                                        check, contexts_script)
from tktopalgo_corpus_brepcheck_shapes import cube, cylinder_face, solid_of, square
from tktopalgo_shape_program import FORWARD, REVERSED, WIRE, ShapeProgram

TOLERANCE, SET_GCTRL, GCTRL, SET_EXACT, EXACT, SET_STATUS, POLYGON = range(1, 8)


def face_checks(requests: list[str], sq, edge_key=(0, 1), extra=()) -> None:
    """The edge of `sq` in its face, with the approximate and exact methods and without
    geometric controls, and its Tolerance."""
    e, face, wire = sq.edges[edge_key], sq.faces[0], sq.wires[0][0]
    for options in ([], [(SET_EXACT, 1)], [(SET_GCTRL, 0)], [(PARALLEL, 1), (SET_EXACT, 1)]):
        calls = [*options, *extra, *contexts_script(face, wire), GCTRL, EXACT, (TOLERANCE,)]
        requests.append(check(EDGE, sq.p, e, *calls))


def minimum_requests(requests: list[str]) -> None:
    """The reference curve: none, multiple, degenerated, flags and ranges."""
    p = ShapeProgram()
    empty = p.edge()
    requests.append(check(EDGE, p, empty, STATUS, (TOLERANCE,), FLAGS))
    sq = square(ShapeProgram())
    e = sq.edges[(0, 1)]
    sq.p.extra_curve3d(e, sq.p.line((0, 0, 0), (1, 0, 0)), sq.p.location(0, 0, 0, 1e-3))
    face_checks(requests, sq)
    for sp, sr, dg in ((True, False, False), (False, True, False), (False, False, False), (True, True, True)):
        sq = square(ShapeProgram())
        e = sq.edges[(1, 2)]
        sq.p.edge_flags(e, sp, sr, dg)
        if dg:
            # Degenerated clears the 3d curve: it is set again after the flag.
            sq.p.edge_curve(e, sq.p.line((1, 0, 0), (0, 1, 0)))
        face_checks(requests, sq, (1, 2))
    for first, last in ((1.0, 1.0), (0.5, 0.25), (-0.5, 1.0), (0.0, 1.5)):
        sq = square(ShapeProgram())
        e = sq.edges[(2, 3)]
        bezier = sq.p.curve(3, 2, 1, 1, 0, 0, 1, 0)
        sq.p.edge_curve(e, bezier)
        sq.p.edge_range(e, first, last, True)
        face_checks(requests, sq, (2, 3))


def periodic_requests(requests: list[str]) -> None:
    """Circles with ranges within and beyond their period, plain and trimmed."""
    for trimmed in (False, True):
        for first, last in ((0.0, 6.0), (-1.0, 5.0), (0.0, 7.0), (2.0, 2.0+2*math.pi+1e-10)):
            p = ShapeProgram()
            v = p.vertex((1, 0, 0), 2.0)
            circle = p.curve(1, 0, 0, 0, 0, 0, 1, 1.0)
            if trimmed:
                circle = p.curve(4, circle, 0.0, 3.0)
            e = p.edge()
            p.edge_curve(e, circle, 0, 1e-7)
            plane = p.surface(0, 0, 0, 0, 0, 0, 1)
            p.edge_pcurve(e, p.pcurve(1, 0.0, 0.0, 1.0, 1), plane, 0, 1e-7)
            p.edge_range(e, first, last)
            p.add(e, v, FORWARD)
            p.add(e, v, REVERSED)
            face = p.face(plane)
            wire = p.container(WIRE)
            p.add(wire, e, FORWARD)
            p.add(face, wire, FORWARD)
            requests.append(check(EDGE, p, e, *contexts_script(face), (TOLERANCE,)))


def pcurve_requests(requests: list[str]) -> None:
    """Curves on the face: shifted within and beyond the tolerance, on another range,
    missing (projected on the plane), on a located face."""
    for shift in (1e-9, 5e-8, 1e-6, 0.1):
        sq = square(ShapeProgram(), shifts={(0, 1): (0.0, shift)})
        face_checks(requests, sq)
    sq = square(ShapeProgram())
    e = sq.edges[(0, 1)]
    sq.p.edge_range_on(e, sq.surfaces[0], 0, 0.0, 1.0+1e-3)
    face_checks(requests, sq)
    sq = square(ShapeProgram(), skip_pcurves=[(0, 1)])
    face_checks(requests, sq)
    # The 3d line above the plane: the projection is 1e-3 away.
    sq = square(ShapeProgram(), skip_pcurves=[(0, 1)])
    sq.p.edge_curve(sq.edges[(0, 1)], sq.p.line((0, 0, 1e-3), (1, 0, 0)))
    face_checks(requests, sq)
    for loc_kind in (0, 1):
        sq = square(ShapeProgram())
        p = sq.p
        location = p.location(0, 0.0, 0.0, 5.0) if loc_kind == 0 else p.location(1, 0, 0, 1, 0.3)
        moved_face = p.located(sq.faces[0], location)
        moved_edge = p.located(sq.edges[(0, 1)], location)
        for e in (sq.edges[(0, 1)], moved_edge):
            requests.append(check(EDGE, p, e, *contexts_script(sq.faces[0], moved_face), (TOLERANCE,)))


def only_pcurve_requests(requests: list[str]) -> None:
    """Edges without 3d curve: the reference is their first curve on surface."""
    p = ShapeProgram()
    cylinder = p.surface(1, 0, 0, 0, 0, 0, 1, 2.0)
    plane = p.surface(0, 0, 0, 0, 0, 0, 1)
    e = p.edge()
    p.edge_pcurve(e, p.line2d((0.0, 0.0), (1.0, 0.0)), cylinder)
    p.edge_range(e, 0.0, 1.0)
    face_cyl = p.face(cylinder)
    e2 = p.edge()
    p.edge_pcurve(e2, p.line2d((0.0, 0.0), (0.0, 1.0)), cylinder)
    p.edge_pcurve(e2, p.line2d((2.0, 0.0), (0.0, 1.0)), plane)
    p.edge_range(e2, 0.0, 1.0)
    face_plane = p.face(plane)
    for edge, contexts in ((e, (face_cyl,)), (e2, (face_cyl, face_plane))):
        wire = p.container(WIRE)
        p.add(wire, edge, FORWARD)
        for face in contexts:
            requests.append(check(EDGE, p, edge, (IN_CONTEXT, face), FLAGS, CONTEXTS, (TOLERANCE,)))


def seam_requests(requests: list[str]) -> None:
    """The seam of a cylinder: both pcurves, the second one shifted."""
    for shift in (0.0, 1e-3, -0.5):
        p = ShapeProgram()
        cyl = cylinder_face(p, seam_shift=shift)
        for name in ('seam', 'bottom', 'top'):
            for options in ([], [(SET_EXACT, 1)]):
                requests.append(check(EDGE, p, cyl[name], *options, *contexts_script(cyl['face'], cyl['wire']),
                                      (TOLERANCE,)))


def solid_requests(requests: list[str]) -> None:
    """Edges of a cube, an open cube and a cube with a face met twice."""
    for faces in (None, [[0, 3, 2, 1], [0, 1, 5, 4]], None):
        p = ShapeProgram()
        shapes = cube(p, faces=faces)
        shell = shapes.shell()
        if faces is None and len(requests) % 2:
            p.add(shell, shapes.faces[0], REVERSED)
        solid = solid_of(p, shell)
        for key in ((0, 1), (0, 3), (4, 5)):
            if key in shapes.edges:
                requests.append(check(EDGE, p, shapes.edges[key], *contexts_script(solid, shell, shapes.faces[0])))


def polygon_requests(requests: list[str]) -> None:
    """Polygons on a triangulation of the square: with and without parameters, near and
    far from the edge, and on a closed triangulation (the second polygon is read)."""
    for dz in (0.0, 1e-3, 0.5):
        sq = square(ShapeProgram())
        p = sq.p
        tri = p.triangulation([(0, 0, 0), (1, 0, dz), (1, 1, 0), (0, 1, 0)], [(1, 2, 3), (1, 3, 4)])
        e = sq.edges[(0, 1)]
        p.polygon(e, tri, 0, ([1, 2], [0.0, 1.0], 1e-3))
        e2 = sq.edges[(1, 2)]
        p.polygon(e2, tri, 0, ([2, 3], None, 1e-3))
        e3 = sq.edges[(2, 3)]
        p.polygon(e3, tri, p.location(0, 0.0, 0.0, dz), ([3, 4], [0.0, 1.0], 0.0), ([1, 2], None, 0.0))
        for edge in (e, e2, e3, sq.edges[(0, 3)]):
            requests.append(check(EDGE, p, edge, (POLYGON, edge), (TOLERANCE,), (SET_STATUS, 33), STATUS,
                                  (FAIL, edge), STATUS))


def append_edge_requests(requests: list[str]) -> None:
    minimum_requests(requests)
    periodic_requests(requests)
    pcurve_requests(requests)
    only_pcurve_requests(requests)
    seam_requests(requests)
    solid_requests(requests)
    polygon_requests(requests)
