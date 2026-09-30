"""Deterministic requests for BRepPrim (tools/oracle/tkprim_probe_brepprim.cxx and
tkprim_probe_brepprim_oneaxis.cxx; the records are documented in the first file).

`bp_wedge` builds the default wedge, boxes, right angular wedges and general wedges
(through BRepPrim_Wedge and BRepPrim_GWedge) on several frames, including degenerated
ones, YMax faces flat in X2 or Z2 or reduced to a point, shifted YMax faces and open
directions; then it runs a call sequence: every existence test (with same-axis pairs
and triples, which raise), every plane, line and point, sub-shapes built in orders that
exercise the sharing of the flat YMax edges and vertices, and Open/Close after building.
`bp_builder` runs every BRepPrim_Builder member. `bp_oneaxis` builds cylinders, cones,
spheres, tori and revolutions through every constructor (with the invalid parameters
each one rejects), sets angles and meridian ranges (sides, inverted and equal heights,
closed meridians, infinite ranges), then calls the topology members in forward,
reverse and interleaved orders, the setters after building (which raise) and every
query.
"""
from __future__ import annotations
import math
import random

PI = math.pi
X_MIN, X_MAX, Y_MIN, Y_MAX, Z_MIN, Z_MAX = range(6)
REFERENCE = [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0]
FRAMES = [
    REFERENCE,
    [1.0, -2.0, 0.5, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0],
    [0.25, 0.5, -1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0],
    [-3.0, 0.1, 2.0, 0.0, -1.0, 0.0, 0.0, 0.0, 1.0],
]


def numbers(values: list[float]) -> str:
    """The fields of a request, with every double spelled exactly."""
    return ' '.join(repr(float(v)) for v in values)


def wedge_request(kind: int, frame: list[float] | None, values: list[float], calls: list[list[int]]) -> str:
    """One `bp_wedge` request."""
    fields = [kind] + (frame or []) + values + [len(calls)] + [v for call in calls for v in call]
    return 'bp_wedge ' + numbers(fields)


def wedge_queries() -> list[list[int]]:
    """Every existence test, plane, line and point, same-axis ones included."""
    calls = [[op, d, 0, 0] for op in (10, 11, 14, 7) for d in range(6)]
    calls += [[op, a, b, 0] for op in (12, 8) for a in range(6) for b in range(6)]
    calls += [[op, a, b, c] for op in (13, 9) for a in (0, 1, 2) for b in (2, 3, 4) for c in (4, 5, 0)]
    return calls


def wedge_builds() -> list[list[list[int]]]:
    """Call sequences that build sub-shapes in different orders."""
    return [
        [],
        [[0, 0, 0, 0]],
        [[4, X_MAX, Z_MIN, Y_MAX], [4, X_MIN, Z_MAX, Y_MAX], [4, X_MAX, Z_MAX, Y_MAX], [0, 0, 0, 0]],
        [[4, X_MIN, Z_MAX, Y_MAX], [4, X_MAX, Z_MIN, Y_MAX], [4, X_MIN, Z_MIN, Y_MAX]],
        [[3, Y_MAX, Z_MAX, 0], [3, Y_MAX, Z_MIN, 0], [3, X_MAX, Y_MAX, 0], [3, X_MIN, Y_MAX, 0]],
        [[2, Y_MAX, 0, 0], [1, Y_MAX, 0, 0], [1, X_MIN, 0, 0], [2, Z_MAX, 0, 0]],
        [[1, d, 0, 0] for d in reversed(range(6))],
        [[3, Z_MIN, X_MIN, 0], [5, Z_MAX, 0, 0], [6, Z_MAX, 0, 0], [0, 0, 0, 0], [5, X_MIN, 0, 0]],
        [[5, X_MIN, 0, 0], [5, Y_MAX, 0, 0], [0, 0, 0, 0], [6, X_MIN, 0, 0]],
        [[5, Z_MAX, 0, 0], [5, Z_MIN, 0, 0], [6, Z_MIN, 0, 0], [3, Z_MAX, X_MIN, 0], [4, X_MIN, Y_MIN, Z_MAX]],
        [[5, d, 0, 0] for d in range(6)] + [[1, X_MIN, 0, 0], [2, Y_MIN, 0, 0]],
    ]


def wedge_shapes() -> list[tuple[int, list[float]]]:
    """The wedges: (kind, values after the frame)."""
    shapes = [(1, [1.0, 2.0, 3.0]), (1, [10.0, 20.0, 30.0]), (1, [1e-8, 1.0, 1.0]), (1, [-1.0, 2.0, 3.0]),
              (1, [0.0, 1.0, 1.0]), (1, [1e-7, 1e-7, 1e-7]), (4, [3.0, 4.0, 5.0])]
    shapes += [(2, [10.0, 10.0, 10.0, ltx]) for ltx in (5.0, 0.0, 10.0, 15.0, -1.0, 1e-9)]
    shapes += [(3, v) for v in (
        [0, 0, 0, 5, 5, 20, 10, 20, 15, 15], [0, 0, 0, 0, 0, 1, 1, 1, 1, 1], [0, 0, 0, 2, 3, 4, 5, 6, 2, 7],
        [0, 0, 0, 1, 1, 4, 5, 6, 5, 1], [0, 0, 0, 2, 2, 4, 5, 6, 2, 2], [-1, -2, -3, -1, -1, 1, 2, 3, 1, 1],
        [0, 0, 0, -1, -2, 1, 1, 1, 2, 3], [0, 0, 0, 1, 1, 2, 2, 2, 0, 3], [0, 0, 0, 0.5, 0.25, 1, 3, 1, 0.75, 0.5])]
    return [(kind, [float(v) for v in values]) for kind, values in shapes]


def flat_on_one_side(kind: int, values: list[float]) -> bool:
    """Whether the YMax face is flat in exactly one of X2 and Z2 (a segment): the case
    where GWedge copies a vertex that is not built yet (docs/tkprim-progress.md)."""
    if kind == 2:
        return values[3] == 0.0
    if kind != 3:
        return False
    return (values[4] == values[9]) != (values[3] == values[8])


def poisoning_builds() -> list[list[list[int]]]:
    """Build orders of a segment YMax face that copy a vertex not built yet: its null copy
    is printed, then the line stops, because the native shell would crash on it."""
    stop = [15, 0, 0, 0]
    return [
        [[4, X_MAX, Z_MIN, Y_MAX], [4, X_MIN, Z_MAX, Y_MAX], [4, X_MAX, Z_MAX, Y_MAX], stop],
        [[3, Z_MIN, X_MIN, 0], [4, X_MAX, Z_MAX, Y_MAX], [4, X_MAX, Z_MIN, Y_MAX], stop],
        [[4, X_MIN, Z_MIN, Y_MAX], [4, X_MAX, Z_MIN, Y_MAX], [4, X_MIN, Z_MAX, Y_MAX], stop],
    ]


def append_wedge_requests(requests: list[str]) -> None:
    """The wedge requests: every shape on two frames with the query sequence, and every
    build order (a segment YMax face gets the orders without direct edge or vertex
    builds, and the poisoning orders that stop before the shell)."""
    requests.append(wedge_request(0, None, [], []))
    requests.append(wedge_request(0, None, [], wedge_queries()))
    for index, (kind, values) in enumerate(wedge_shapes()):
        for frame in (FRAMES[0], FRAMES[1 + index % 3]):
            requests.append(wedge_request(kind, frame, values, wedge_queries()))
        builds = wedge_builds()
        if flat_on_one_side(kind, values):
            builds = [calls for calls in builds if not any(call[0] in (3, 4) for call in calls)]
            builds += poisoning_builds()
        for calls in builds:
            requests.append(wedge_request(kind, FRAMES[index % 4], values, calls))


def append_builder_requests(requests: list[str]) -> None:
    """The BRepPrim_Builder requests on each frame and a few points and radii."""
    points = [([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]), ([1.0, 2.0, 3.0], [-1.0, 0.5, 2.0]), ([0.1, 0.2, 0.3], [0.3, 0.2, 0.1])]
    for frame in FRAMES:
        for (p1, p2), (radius, first, last) in zip(points, [(1.0, 0.0, 2 * PI), (2.5, 0.5, 4.0), (0.1, -1.0, 1.0)]):
            requests.append('bp_builder ' + numbers(frame + p1 + p2 + [radius, first, last]))


MEMBERS = list(range(30))
QUERIES = [[op, 0.0] for op in range(52, 62)]


def one_axis_request(kind: int, ctor: int, fields: list[float], calls: list[list[float]]) -> str:
    """One `bp_oneaxis` request."""
    values = [kind, ctor] + fields + [len(calls)] + [v for call in calls for v in call]
    return 'bp_oneaxis ' + numbers(values)


def one_axis_sequences(rng: random.Random) -> list[list[list[float]]]:
    """Call sequences: none, the queries, every member forward and backward, a seeded
    shuffle, and the setters after building."""
    # LateralStartWire (7) shares the wire slot of LateralWire: called before the lateral
    # face of a finite primitive, it makes that face read edges never built, which crashes
    # natively (even after a shell that raised). The member sequences leave it out; it runs
    # alone and after the shell, each time followed by a stop (docs/tkprim-progress.md).
    reordered = [op for op in MEMBERS if op != 7]
    shuffled = reordered[:]
    rng.shuffle(shuffled)
    return [
        [],
        QUERIES + [[50, 0.0], [50, 0.5], [51, 0.0], [30, 0.0], [31, 0.0], [31, 1.0], [32, 0.5]],
        [[op, 0.0] for op in reordered],
        [[op, 0.0] for op in reversed(reordered)],
        [[op, 0.0] for op in shuffled],
        [[26, 0.0], [40, 1.0], [41, 0.0], [42, 1.0], [44, 0.3], [0, 0.0]],
        [[7, 0.0], [8, 0.0], [99, 0.0]],
        [[0, 0.0], [7, 0.0], [6, 0.0], [99, 0.0]],
    ]


def one_axis_shapes() -> list[tuple[int, int, list[float], list[list[float]]]]:
    """The primitives (kind, ctor, fields, setup calls) before a call sequence."""
    f0, f1, f2 = FRAMES[0], FRAMES[1], FRAMES[2]
    shapes = [
        (0, 0, f1 + [2.0, 3.0], []), (0, 1, [1.5], []), (0, 2, [1.0, 2.0, 3.0, 0.5], []), (0, 3, f2 + [2.0], []),
        (0, 4, [5.0, 10.0], []), (0, 5, [1.0, -1.0, 0.5, 2.0, 1.0], []), (0, 4, [5.0, 10.0], [[40, PI]]),
        (0, 4, [1.0, 2.0], [[40, PI / 2]]), (0, 1, [1.0], [[40, 1.0]]), (0, 4, [1.0, 2.0], [[40, 3 * PI]]),
        (0, 4, [1.0, 2.0], [[40, 2 * PI - 1e-13]]), (0, 4, [-1.0, 2.0], []), (0, 4, [1.0, 0.0], []),
        (0, 4, [1.0, 2.0], [[41, 3.0], [42, 1.0], [40, 1.0]]), (0, 4, [0.0, 2.0], [[40, 1.0]]),
        (1, 0, [0.5] + f1 + [2.0, 1.0], []), (1, 0, [0.5] + f0 + [2.0, 0.0], [[40, PI]]),
        (1, 0, [0.5] + f0 + [1e-8, 1.0], []), (1, 0, [0.0] + f0 + [2.0, 1.0], []), (1, 0, [PI / 2] + f0 + [2.0, 1.0], []),
        (1, 1, [0.5], []), (1, 1, [0.5], [[40, PI]]), (1, 1, [-0.1], []), (1, 1, [2.0], []), (1, 2, [0.25, 1.0, 2.0, 3.0], []),
        (1, 4, [5.0, 2.0, 10.0], []), (1, 4, [5.0, 0.0, 10.0], []), (1, 4, [0.0, 3.0, 4.0], [[40, PI]]),
        (1, 4, [5.0, 2.0, 10.0], [[40, PI]]), (1, 4, [2.0, 2.0, 1.0], []), (1, 4, [-1.0, 2.0, 1.0], []),
        (1, 4, [1.0, 2.0, 0.0], []), (1, 5, [1.0, 1.0, 1.0, 3.0, 1.0, 2.0], []), (1, 6, f2 + [1.0, 0.0, 2.0], [[40, 1.0]]),
        (2, 0, [5.0], []), (2, 1, [1.0, 2.0, 3.0, 2.0], []), (2, 2, f1 + [1.5], []), (2, 0, [5.0], [[40, PI]]),
        (2, 0, [5.0], [[41, 0.0], [42, PI / 2], [40, PI / 2]]), (2, 0, [5.0], [[41, -PI / 4], [42, PI / 4]]),
        (2, 0, [2.0], [[41, 0.0], [42, 0.5]]), (2, 0, [2.0], [[41, -0.5], [42, 0.0], [40, 2.0]]), (2, 0, [-1.0], []),
        (3, 1, [10.0, 2.0], []), (3, 0, f1 + [3.0, 1.0], []), (3, 2, [1.0, 2.0, 3.0, 5.0, 1.0], []),
        (3, 1, [10.0, 2.0], [[40, PI]]), (3, 0, f0 + [1.0, 0.1], [[41, -PI / 2], [42, PI / 2], [40, 2.094]]),
        (3, 1, [10.0, 2.0], [[41, PI / 2], [42, 3 * PI / 2], [40, PI / 4]]),
        (3, 1, [10.0, 2.0], [[41, 0.0], [42, PI], [40, PI / 4]]), (3, 1, [10.0, 2.0], [[41, 0.0], [42, PI]]),
        (3, 1, [10.0, 2.0], [[41, PI / 4], [42, 3 * PI / 4], [40, PI / 4]]),
        (3, 1, [10.0, 2.0], [[41, -PI / 6], [42, PI / 3], [40, PI / 4]]), (3, 1, [10.0, -2.0], []),
        (4, 0, f0 + [0.0, 2.0] + [0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0] + [0, 1.0, 0.0, 0.0, 1.0], []),
        (4, 0, f0 + [0.0, 2.0] + [0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0] + [0, 1.0, 0.0, 0.0, 1.0], [[40, 2.0]]),
        (4, 0, f1 + [-1.0, 1.0] + [0, 0.0, 0.0, 0.0, 0.6, 0.0, 0.8] + [0, 0.0, 0.0, 0.6, 0.8], [[40, 1.0]]),
        (4, 0, f0 + [0.0, PI] + [1, 3.0, 0.0, 0.0, 0.0, -1.0, 0.0, 1.0] + [1, 3.0, 0.0, 1.0, 0.0, 1.0, 1], [[40, 1.5]]),
    ]
    return [(k, c, [float(v) for v in f], s) for k, c, f, s in shapes]


def append_one_axis_requests(requests: list[str]) -> None:
    """Every primitive with every call sequence."""
    rng = random.Random(20261002)
    for kind, ctor, fields, setup in one_axis_shapes():
        for sequence in one_axis_sequences(rng):
            requests.append(one_axis_request(kind, ctor, fields, setup + sequence))


def append_brepprim_requests(requests: list[str]) -> None:
    """Append the BRepPrim requests."""
    append_wedge_requests(requests)
    append_builder_requests(requests)
    append_one_axis_requests(requests)
