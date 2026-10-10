"""AxisIntersector: assertions, native RTTI, ray picks, clipping and protected outputs."""
from __future__ import annotations
from itertools import product
from tkv3d_corpus_triangular import IDENTITY, SHEAR, COLLAPSE, triangular_queries

AXES = ('0 0 0 0 0 1', '0 0 10 0 0 -1', '0 0 -5 0 0 1', '1 2 3 1 2 3',
        '-5 0 0 1 0 0', '0 2 0 0 -1 0', '5 5 5 -1 -1 -1', '0 0 0 1e-8 0 1')
CLIPS = ('0 -10000 10000 0', '0 0 5 0', '0 5 10 0', '1 0 0 0',
         '0 -10 100 2 0 2 2 4', '0 -10 100 2 3 5 7 9')


def axis_queries() -> list[str]:
    """Reuse overload outputs and add triangle-plane/segment and quadric edge cases."""
    queries = triangular_queries()
    queries += [f'4 1 {sensitivity} {points}' for sensitivity, points in product((0, 1, 7),
                ('-2 -2 5 2 -2 5 0 2 5', '0 0 1 0 0 2 0 0 3', '0 -2 -2 0 2 -2 0 0 2',
                 '20 20 5 22 20 5 20 22 5', '0 0 0 0 0 0 0 0 0'))]
    queries += [f'2 1 {points}' for points in ('-1 0 5 1 0 5', '0 0 1 0 0 10', '-1 1 5 1 1 5',
                                              '-1 0 -1 1 0 -1', '0 0 0 1 0 0')]
    queries += [f'5 {mode} 0 0 0 {radius}' for mode, radius in product((-1, 0, 1), (1, -1, 10))]
    queries += [f'7 {mode} 5 {filled} 0 0 1 0 0 0 5 2' for mode, filled in product((-1, 0, 1), (0, 1))]
    return queries


def append_axis_states(requests: list[str]) -> None:
    """Check both transform forms, ignored scale/builder, initialization and dump depth."""
    transforms = ((0, 0, IDENTITY), (1, 0, IDENTITY), (1, 1, IDENTITY),
                  (1, 1, SHEAR), (1, 1, COLLAPSE), (2, 0, IDENTITY))
    for initialized, axis, (transition, mode, matrix), depth in product((0, 1), AXES, transforms, (-1, 0, 1, 2)):
        requests.append(f'v3_axis {initialized} {axis} {transition} {mode} -7 {matrix} 0 {depth} -3 1 2 3')


def append_axis_protected(requests: list[str]) -> None:
    """Protected calls work without Init and preserve output arguments after failure."""
    queries = ['0 -1 -1 -1 1 1 1', '0 50 50 50 51 51 51', '0 2 2 2 -2 -2 -2',
               '1 0 0 0', '1 0 0 5', '1 0 0 -5', '1 1 2 3',
               '2 -1 0 5 1 0 5', '2 0 0 1 0 0 5', '2 -1 1 5 1 1 5',
               '3 0 0 1 0 0 5', '3 0 1 0 0 5 0', '3 0 0 0 1 2 3', '3 0 0 1 0 0 -5']
    for initialized, axis, query in product((0, 1), AXES, queries):
        requests.append(f'v3_axis {initialized} {axis} 0 0 0 {IDENTITY} 2 {query}')


def append_axis_requests(requests: list[str]) -> None:
    """Exercise each overlap against multiple ray directions, transforms and clipping states."""
    append_axis_states(requests)
    append_axis_protected(requests)
    queries = axis_queries()
    transforms = ((0, 0, IDENTITY), (1, 1, SHEAR), (2, 0, IDENTITY))
    for axis, (transition, mode, matrix), clip in product(AXES, transforms, CLIPS):
        requests += [f'v3_axis 1 {axis} {transition} {mode} 5 {matrix} 1 {clip} {query}' for query in queries]
    requests += [f'v3_axis 0 {AXES[0]} 0 0 0 {IDENTITY} 1 {CLIPS[0]} {query}' for query in queries]
