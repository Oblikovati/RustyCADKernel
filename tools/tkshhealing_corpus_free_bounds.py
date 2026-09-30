"""Native boundary chaining, repeated status checks, and elementary loop splitting."""

from itertools import product, permutations
from tktopalgo_shape_program import ShapeProgram, WIRE, FORWARD, REVERSED, num
from tkshhealing_corpus_wiredata import scene, wire_of


def boundary_record(
    program: ShapeProgram,
    mode: int,
    tolerance: float,
    shared: bool,
    shapes: list[int],
    extra: list[float | int] = (),
) -> str:
    """Serialize one independent operation; boundary_record(program,0,1e-7,True,edges)."""
    values = [mode, tolerance, shared, len(shapes), *shapes, *extra]
    return "sh_free_bounds " + program.tokens() + " " + " ".join(map(num, values))


def append_free_bounds_requests(requests: list[str]) -> None:
    """Exercise chaining/splitting and the exact advanced status state; append_free_bounds_requests(rows)."""
    append_boundary_scene_requests(requests)
    append_boundary_gap_requests(requests)
    requests.append(boundary_record(ShapeProgram(), 12, 0.0, False, []))


def append_boundary_scene_requests(requests: list[str]) -> None:
    """Use real BRep builder topology with curves and pcurves; append_boundary_scene_requests(rows)."""
    fixture = scene()
    program = fixture.program
    wires = [wire_of(program, [(edge, FORWARD)]) for edge in fixture.edges]
    for order in permutations(wires):
        for mode, shared in product([1, 2, 3, 9, 10, 11], [False, True]):
            requests.append(boundary_record(program, mode, 1e-7, shared, list(order)))
    families = [
        [],
        [fixture.square],
        [fixture.square_reversed],
        [fixture.with_internal],
        [fixture.broken],
        [fixture.seam_wire],
        [fixture.square, fixture.seam_wire],
    ]
    for shapes, mode, shared in product(
        families, [1, 2, 3, 4, 9, 10, 11], [False, True]
    ):
        requests.append(boundary_record(program, mode, 1e-7, shared, shapes))
    for mode, face, closed, opened, shared in product(
        [5, 6], [fixture.plane_face, fixture.compound], [0, 1], [0, 1], [0, 1]
    ):
        requests.append(
            boundary_record(program, mode, 1e-6, shared, [face], [closed, opened])
        )
    for mode in [8, 9, 10, 11]:
        for shapes in [[fixture.vertex], [fixture.square, fixture.vertex], [-1]]:
            requests.append(boundary_record(program, mode, 1e-7, True, shapes))
    for shapes in [
        fixture.edges,
        [fixture.internal_edge, *fixture.edges],
        [*fixture.edges, fixture.external_edge],
    ]:
        for shared in [False, True]:
            requests.append(boundary_record(program, 0, 1e-7, shared, shapes))
            requests.append(boundary_record(program, 8, 1e-7, shared, shapes))


def gap_scene(
    gap: float, orientation: int, duplicate: bool
) -> tuple[ShapeProgram, list[int], int]:
    """Two genuine line edges, distinct or shared junction vertex; gap_scene(0.,0,False)."""
    program = ShapeProgram()
    points = [
        (0.0, 0.0, 0.0),
        (1.0, 0.0, 0.0),
        (1.0 + gap, 0.0, 0.0),
        (2.0 + gap, 0.0, 0.0),
    ]
    vertices = [program.vertex(point) for point in points]
    if not duplicate and gap == 0.0:
        vertices[2] = vertices[1]
    edges = []
    for first, last in [(0, 1), (2, 3)]:
        edge = program.edge()
        program.edge_curve(edge, program.line(points[first], (1.0, 0.0, 0.0)))
        program.edge_range(edge, 0.0, 1.0)
        program.add(edge, vertices[first], FORWARD)
        program.add(edge, vertices[last], REVERSED)
        program.vertex_on_curve(vertices[first], 0.0, edge)
        program.vertex_on_curve(vertices[last], 1.0, edge)
        edges.append(edge)
    wires = [
        wire_of(program, [(edges[0], FORWARD)]),
        wire_of(program, [(edges[1], orientation)]),
    ]
    wire = wire_of(program, [(edges[0], FORWARD), (edges[1], orientation)])
    return program, wires, wire


def append_boundary_gap_requests(requests: list[str]) -> None:
    """Check thresholds, reversed tails, and accumulated CheckConnected status; append_boundary_gap_requests(rows)."""
    for gap, orientation, duplicate in product(
        [0.0, 1e-15, 1e-8, 1e-6, 1e-3], [0, 1], [False, True]
    ):
        program, wires, wire = gap_scene(gap, orientation, duplicate)
        for tolerance, shared, mode in product(
            [-1.0, 0.0, 1e-7, 1e-4, 1.0], [False, True], [1, 2, 3, 9, 10, 11]
        ):
            requests.append(boundary_record(program, mode, tolerance, shared, wires))
        for tolerance in [-1.0, 0.0, 1e-7, 1e-4]:
            checks = [
                8,
                2,
                0.0,
                2,
                1e-2,
                -99,
                0.0,
                1,
                0.0,
                0,
                1.0,
                -1,
                0.0,
                -99,
                1.0,
                2,
                0.0,
            ]
            requests.append(
                boundary_record(program, 7, tolerance, False, [wire], checks)
            )
