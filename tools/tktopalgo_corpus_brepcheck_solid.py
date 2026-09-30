"""The BRepCheck_Solid and BRepCheck_Analyzer requests of the TKTopAlgo corpus: solids of
one, two and three shells (holes, enclosed regions, shells outside each other, shared
faces, detached and INTERNAL parts), and the analyzer on every kind of shape, compounds
and compsolids, with each option.

A solid of four or more closed shells where two are not holes would read an
uninitialized tool natively (a crash); the corpus keeps three shells at most."""
from __future__ import annotations
from tktopalgo_corpus_brepcheck import CONTEXTS, FLAGS, IN_CONTEXT, STATUS, check
from tktopalgo_corpus_brepcheck_shapes import CUBE_FACES, CUBE_POINTS, PlanarShapes, cube, square
from tktopalgo_shape_program import (COMPOUND, COMPSOLID, FORWARD, INTERNAL, REVERSED, SOLID,
                                     ShapeProgram)

SOLID_CHECK = 5


def box(p: ShapeProgram, origin, size: float, hole: bool = False) -> int:
    """A closed cube shell at `origin` (its faces reversed in the shell for a hole)."""
    points = [tuple(o+size*c for o, c in zip(origin, point)) for point in CUBE_POINTS]
    shapes = PlanarShapes(p, points)
    for loop in CUBE_FACES:
        shapes.face([loop])
    shapes.finish()
    shell = shapes.shell(orientations={k: REVERSED for k in range(6)} if hole else None)
    p.flag(shell, 0, True)
    return shell


def solid(p: ShapeProgram, *children) -> int:
    s = p.container(SOLID)
    for child, orientation in children:
        p.add(s, child, orientation)
    return s


def analyze(p: ShapeProgram, index: int, geom: bool = True, parallel: bool = False, exact: bool = False) -> str:
    return f'bck_analyze {p.tokens()} {index} {int(geom)} {int(parallel)} {int(exact)}'


def solid_requests(requests: list[str]) -> None:
    """The solid checker and the analyzer on solids of one to three shells."""
    cases = [
        [],
        [(0, 1.0, False)],
        [(0, 4.0, False), (1, 1.0, True)],
        [(0, 1.0, False), (3, 1.0, False)],
        [(0, 4.0, False), (1, 1.0, False)],
        [(0, 1.0, True)],
        [(0, 4.0, False), (1, 1.0, True), (2, 0.5, True)],
        [(0, 1.0, False), (3, 1.0, True)],
        [(0, 4.0, False), (1, 1.0, True), (5, 1.0, True)],
    ]
    for shells in cases:
        p = ShapeProgram()
        children = [(box(p, (offset, offset, offset), size, hole), FORWARD) for offset, size, hole in shells]
        s = solid(p, *children)
        requests.append(check(SOLID_CHECK, p, s, STATUS, (IN_CONTEXT, s), FLAGS, CONTEXTS))
        requests.append(analyze(p, s))
    p = ShapeProgram()
    outer = box(p, (0, 0, 0), 1.0)
    detached = p.vertex((5.0, 5.0, 5.0))
    for orientation in (FORWARD, INTERNAL):
        s = solid(p, (outer, FORWARD), (detached, orientation))
        requests.append(check(SOLID_CHECK, p, s, STATUS, FLAGS))
        requests.append(analyze(p, s))
    p = ShapeProgram()
    shapes = cube(p)
    inner = shapes.shell(orientations={2: INTERNAL})
    s = solid(p, (inner, FORWARD))
    requests.append(check(SOLID_CHECK, p, s, STATUS))
    requests.append(analyze(p, s))
    shared = shapes.shell()
    s = solid(p, (shared, FORWARD), (shapes.shell(faces=shapes.faces[:1]), FORWARD))
    requests.append(check(SOLID_CHECK, p, s, STATUS))
    requests.append(analyze(p, s))


def shape_requests(requests: list[str]) -> None:
    """The analyzer on each sub-shape kind of valid and invalid shapes, in compounds and
    compsolids, with the geometric controls off, in parallel and with the exact method."""
    sq = square(ShapeProgram(), 4.0, holes=((1, 1, 1),))
    p = sq.p
    targets = [sq.vertices[0], sq.edges[(0, 1)], sq.wires[0][0], sq.wires[0][1], sq.faces[0]]
    bad = square(p, 4.0, holes=((3, 1, 2),))
    shifted = square(p, 1.0, shifts={(0, 1): (0.0, 1e-3)})
    targets += [bad.faces[0], shifted.faces[0]]
    compound = p.container(COMPOUND)
    for face in (sq.faces[0], bad.faces[0]):
        p.add(compound, face, FORWARD)
    p.add(compound, p.oriented(shifted.faces[0], REVERSED))
    targets.append(compound)
    for target in targets:
        requests.append(analyze(p, target))
    for options in ((False, False, False), (True, True, False), (True, False, True), (True, True, True)):
        requests.append(analyze(p, compound, *options))
    for faces, reversed_face in ((CUBE_FACES, False), (CUBE_FACES, True), (CUBE_FACES[:5], False)):
        p = ShapeProgram()
        shapes = cube(p, faces=faces)
        shell = shapes.shell(orientations={0: REVERSED} if reversed_face else None)
        s = solid(p, (shell, FORWARD))
        compsolid = p.container(COMPSOLID)
        p.add(compsolid, s, FORWARD)
        outer = p.container(COMPOUND)
        p.add(outer, compsolid, FORWARD)
        p.add(outer, shapes.edges[(0, 1)], FORWARD)
        for target in (shell, s, compsolid, outer):
            requests.append(analyze(p, target))
        requests.append(analyze(p, outer, True, True, True))


def gtest_requests(requests: list[str]) -> None:
    """The face of BRepCheck_Face_Test's first case: 40 circular holes (IsValid)."""
    from tktopalgo_corpus_brepcheck_face import circle_hole
    sq = square(ShapeProgram(), 100.0)
    for row in range(5):
        for column in range(8):
            circle_hole(sq.p, sq.surfaces[0], sq.faces[0], (10.0+11.0*column, 10.0+11.0*row), 2.0)
    requests.append(analyze(sq.p, sq.faces[0]))


def append_solid_requests(requests: list[str]) -> None:
    solid_requests(requests)
    shape_requests(requests)
    gtest_requests(requests)
