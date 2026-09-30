"""The BRepCheck_Shell requests of the TKTopAlgo corpus: closed and open cubes, reversed
and repeated faces, disconnected faces, an edge of three faces (NbConnectedSet), a
Moebius band built topologically, empty shells, and the solid contexts.

NbConnectedSet visits the edges of more than two faces in the order of their hash (the
TShape addresses), and the sets it finds depend on it when there are several such
edges; the shells have at most one of them (docs/tktopalgo-progress.md)."""
from __future__ import annotations
from tktopalgo_corpus_brepcheck import CONTEXTS, FLAGS, IN_CONTEXT, SHELL_CHECK, STATUS, check
from tktopalgo_corpus_brepcheck_shapes import CUBE_FACES, PlanarShapes, cube, solid_of
from tktopalgo_shape_program import FORWARD, INTERNAL, REVERSED, SHELL, SOLID, ShapeProgram

CLOSED, ORIENTATION, SET_UNORIENTABLE, UNORIENTABLE, SETS = range(1, 6)


def shell_requests(requests: list[str], p: ShapeProgram, shell: int, closed: bool = True) -> None:
    """Each check with and without update, NbConnectedSet, and the shell in a solid."""
    for update in (0, 1):
        requests.append(check(SHELL_CHECK, p, shell, (ORIENTATION, update), (CLOSED, update), UNORIENTABLE,
                              STATUS))
    requests.append(check(SHELL_CHECK, p, shell, UNORIENTABLE, (SET_UNORIENTABLE,), UNORIENTABLE, (SETS,),
                          (CLOSED, 1), STATUS))
    solid = solid_of(p, shell, closed)
    other = p.container(SOLID)
    requests.append(check(SHELL_CHECK, p, shell, (IN_CONTEXT, solid), (IN_CONTEXT, other), (IN_CONTEXT, shell),
                          FLAGS, CONTEXTS, (ORIENTATION, 1)))


def cube_requests(requests: list[str]) -> None:
    """The cube, open (five faces), with one face reversed, with a face twice, and with
    an INTERNAL face; closed flag on and off."""
    for closed in (True, False):
        p = ShapeProgram()
        shapes = cube(p)
        shell_requests(requests, p, shapes.shell(), closed)
    p = ShapeProgram()
    shapes = cube(p, faces=CUBE_FACES[:5])
    shell_requests(requests, p, shapes.shell())
    for orientations in ({0: REVERSED}, {0: REVERSED, 1: REVERSED}, {3: INTERNAL}, {2: REVERSED, 4: REVERSED}):
        p = ShapeProgram()
        shapes = cube(p)
        shell_requests(requests, p, shapes.shell(orientations=orientations))
    # A face met twice (Minimum finds the shell not connected, which hides the
    # redundant face from Orientation). Its edges then have three faces in a cube:
    # NbConnectedSet (and so Closed) would depend on the order of their hash, so the
    # face is repeated in shells of one and two faces only.
    for faces, repeated, orientation in ((CUBE_FACES[:1], 0, FORWARD), (CUBE_FACES[:1], 0, REVERSED),
                                         (CUBE_FACES[:2], 1, FORWARD), ([CUBE_FACES[0], CUBE_FACES[2]], 1, REVERSED)):
        p = ShapeProgram()
        shapes = cube(p, faces=faces)
        shell = shapes.shell()
        p.add(shell, shapes.faces[repeated], orientation)
        shell_requests(requests, p, shell)


def other_requests(requests: list[str]) -> None:
    """Disconnected faces, three faces on one edge, an empty shell, one face."""
    p = ShapeProgram()
    shapes = PlanarShapes(p, [(0, 0, 0), (1, 0, 0), (1, 1, 0), (0, 1, 0), (5, 0, 0), (6, 0, 0), (6, 1, 0),
                              (5, 1, 0)])
    shapes.face([[0, 1, 2, 3]], normal=(0, 0, 1))
    shapes.face([[4, 5, 6, 7]], normal=(0, 0, 1))
    shapes.finish()
    shell_requests(requests, p, shapes.shell())
    p = ShapeProgram()
    fan = PlanarShapes(p, [(0, 0, 0), (1, 0, 0), (1, 1, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (1, 0, 1),
                           (1, 0, -1), (0, 0, -1)])
    fan.face([[0, 1, 2, 3]])
    fan.face([[1, 0, 4]])
    fan.face([[0, 1, 6, 5]])
    fan.face([[1, 0, 8, 7]])
    fan.finish()
    shell_requests(requests, p, fan.shell())
    shell_requests(requests, p, fan.shell(faces=fan.faces[:3]))
    p = ShapeProgram()
    shell_requests(requests, p, p.container(SHELL))
    p = ShapeProgram()
    single = cube(p, faces=CUBE_FACES[:1])
    shell_requests(requests, p, single.shell())


def multi_edge_requests(requests: list[str]) -> None:
    """An edge of three and of four faces, the faces otherwise connected so that
    NbConnectedSet finds one set: Orientation counts the uses of the edge."""
    points = [(0, 0, 0), (1, 0, 0), (0, 1, 0), (0, 0, 1), (0, -1, 0)]
    for loops in ([[0, 1, 2], [1, 0, 2], [0, 1, 3]], [[0, 1, 2], [1, 0, 2], [0, 1, 3], [1, 0, 3]],
                  [[0, 1, 2], [1, 0, 2], [0, 1, 3], [0, 1, 4]]):
        p = ShapeProgram()
        shapes = PlanarShapes(p, points)
        for loop in loops:
            shapes.face([loop])
        shapes.finish()
        shell_requests(requests, p, shapes.shell())


def moebius_requests(requests: list[str]) -> None:
    """A band of three quads glued in a cycle, then with the last gluing twisted: its
    orientation cannot be made coherent (only the topology is read)."""
    for twisted in (False, True):
        p = ShapeProgram()
        points = [(0, 0, 0), (1, 0, 0), (2, 0, 0), (0, 1, 0), (1, 1, 0), (2, 1, 0)]
        band = PlanarShapes(p, points)
        band.face([[0, 1, 4, 3]], normal=(0, 0, 1))
        band.face([[1, 2, 5, 4]], normal=(0, 0, 1))
        # The third face closes the band on the edges 2-5 and 0-3; twisted, it uses
        # the edge 0-3 with the orientation of the first face.
        band.face([[2, 3, 0, 5]] if twisted else [[2, 0, 3, 5]], normal=(0, 0, 1))
        band.finish()
        shell_requests(requests, p, band.shell())


def append_shell_requests(requests: list[str]) -> None:
    cube_requests(requests)
    other_requests(requests)
    multi_edge_requests(requests)
    moebius_requests(requests)
