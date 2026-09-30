"""Deterministic requests for Sweep (tools/oracle/tkprim_probe_sweep.cxx).

A directing shape is `index type closed begInf endInf` (type: TopAbs_ShapeEnum, 0..8).
`sw_numshape` prints the accessors of every shape of the grid below, its tool (the
indexation over 0..NbShapes+1, which reaches the indices past both ends, and the first
and last vertices, which raise on an infinite end) and its iteration: the edges of 0, 1
and 2 vertices with every closed and infinite flag, the vertices of index 1 and 2, the
dummy shape and every other shape type, and out-of-range indices (negative, 3, 7).
`sw_reinit` restarts an iteration on another shape after 0 to 3 steps (a non-edge keeps
the state of the first iteration); `sw_init` reinitializes a shape and prints the default
constructors. A first shape that is not an edge is never advanced, because the native
iterator would then read its uninitialized orientation (docs/tkprim-progress.md).
"""
from __future__ import annotations
import itertools

EDGE, VERTEX = 6, 7


def shape_fields(index: int, kind: int, closed: bool, beg: bool, end: bool) -> list[int]:
    """The five fields of a directing shape."""
    return [index, kind, int(closed), int(beg), int(end)]


def shapes() -> list[list[int]]:
    """Every edge of 0 to 2 vertices with every flag, then vertices, the other types and
    out-of-range indices."""
    grid = [shape_fields(index, EDGE, *flags)
            for index in (0, 1, 2) for flags in itertools.product((False, True), repeat=3)]
    grid += [shape_fields(index, VERTEX, closed, False, False)
             for index in (1, 2) for closed in (False, True)]
    grid += [shape_fields(0, kind, False, False, False) for kind in range(9) if kind != EDGE]
    grid += [shape_fields(index, EDGE, False, False, False) for index in (-1, 3, 7)]
    grid += [shape_fields(index, VERTEX, False, False, False) for index in (-1, 0, 3)]
    return grid


def request(op: str, fields: list[int]) -> str:
    """One request line of integer fields."""
    return op + ' ' + ' '.join(str(value) for value in fields)


def append_sweep_requests(requests: list[str]) -> None:
    """Append the Sweep requests."""
    grid = shapes()
    requests.extend(request('sw_numshape', fields) for fields in grid)
    edges = [fields for fields in grid if fields[1] == EDGE]
    others = [shape_fields(1, VERTEX, False, False, False), shape_fields(2, 3, True, False, False),
              shape_fields(1, EDGE, False, True, False), shape_fields(2, EDGE, True, False, False)]
    for first, second, steps in itertools.product(edges[::5], others, range(4)):
        requests.append(request('sw_reinit', first + second + [steps]))
    for first in grid[::4]:
        requests.append(request('sw_init', first + grid[-1 - grid.index(first) % 7]))
