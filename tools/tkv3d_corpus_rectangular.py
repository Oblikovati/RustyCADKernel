"""Rectangular frustum construction, finite picking rays and clipping at OCCT 3d097a0."""
from __future__ import annotations
from itertools import product
from tkv3d_corpus_triangular import IDENTITY, SHEAR, COLLAPSE, QUADRIC_TRANSFORMS
from tkv3d_corpus_axis import CLIPS, axis_queries

RECTANGLES = ('1 400 300 0 0', '1 150 200 0 0', '2 200 100 600 500',
              '2 600 500 200 100', '2 400 300 400 300')
TRANSITIONS = ((0, 1, 0, 0, IDENTITY), (1, 1, 0, 1, IDENTITY),
               (1, 1, 1, 0, IDENTITY), (1, 4, 0, 0, IDENTITY),
               (1, 0, 1, 1, SHEAR), (1, 1, 1, 0, SHEAR),
               (1, 4, 1, 1, SHEAR), (1, 4, 1, 2, COLLAPSE),
               (1, -1, 0, 1, IDENTITY), (2, 1, 0, 1, IDENTITY),
               (2, 1, 0, 0, IDENTITY), (3, 1, 0, 1, IDENTITY))


def rectangle_prefix(camera: tuple[int, int, int], rectangle: str, tolerance: int = 9, built: int = 1) -> str:
    """Encode a camera plus stored and optionally built selection primitive."""
    return f'v3_rectangle {" ".join(map(str, camera))} {rectangle} {tolerance} {built}'


def rectangle_transition(transition: tuple[int, int, int, int, str]) -> str:
    """Keep transform form separate from its numeric identity matrix."""
    return ' '.join(map(str, transition))


def append_rectangle_states(requests: list[str]) -> None:
    """Cross camera depth modes, initialization and copies, including rejected transforms."""
    for camera, rectangle, transition, depth in product(product((0, 1), repeat=3), RECTANGLES,
                                                        TRANSITIONS, (-1, 0, 1, 2)):
        prefix = rectangle_prefix(camera, rectangle)
        requests.append(f'{prefix} {rectangle_transition(transition)} 0 0 {depth}')
    for selection, built, transition in product(('0 0 0 0 0', *RECTANGLES), (0, 1), TRANSITIONS):
        prefix = rectangle_prefix((0, 0, 0), selection, built=built)
        requests.append(f'{prefix} {rectangle_transition(transition)} 0 0 -1')
    requests.append(f'{rectangle_prefix((0, 0, 0), RECTANGLES[0])} 1 1 0 0 {IDENTITY} 0 0 0')
    for tolerance, rectangle in product((-5, 0, 1, 200), RECTANGLES):
        prefix = rectangle_prefix((0, 0, 0), rectangle, tolerance)
        requests.append(f'{prefix} 0 1 0 0 {IDENTITY} 0 1 -1')


def append_rectangle_helpers(requests: list[str]) -> None:
    """Check protected finite-segment picks and the public planar predicates."""
    helpers = [f'0 {points}' for points in ('-1 0 5 1 0 5', '0 0 -100 0 0 50',
                                           '1 1 0 1 1 0', '0 0 0 0 0 0', '50 50 0 51 51 0')]
    helpers += [f'1 {normal} {point}' for normal, point in product(('0 0 1', '0 1 0', '0 0 0', '1 2 3'),
                                                                ('0 0 0', '0 0 20', '0 0 -100'))]
    helpers += [f'2 {points}' for points in ('-1 0 0 1 0 0 0 -1 0 0 1 0',
                '0 0 0 1 1 0 0 1 0 1 0 0', '0 0 0 0 0 1 0 0 2 0 0 3',
                '-1 0 0 1 0 0 0 -1 1 0 1 1')]
    helpers += [f'3 {radius} 0 0 0 {transform} -2 4 -2 -2 0 2 -2 0 2 2 0 -2 2 0'
                for radius, transform in product((0, 1, 3, -1), QUADRIC_TRANSFORMS)]
    for camera, rectangle, built, query in product(((0, 0, 0), (1, 1, 1)), RECTANGLES, (0, 1), helpers):
        prefix = rectangle_prefix(camera, rectangle, built=built)
        requests.append(f'{prefix} 0 1 0 0 {IDENTITY} 2 {query}')
    requests.append(f'{rectangle_prefix((0, 0, 0), RECTANGLES[0])} 0 1 0 0 {IDENTITY} 3 1 2 3 4 5 6')


def append_rectangular_requests(requests: list[str]) -> None:
    """Compare all overlaps, optional inside outputs, pick mutations and local transforms."""
    append_rectangle_states(requests)
    append_rectangle_helpers(requests)
    variants = (TRANSITIONS[0], TRANSITIONS[5], TRANSITIONS[6], TRANSITIONS[-1])
    queries = axis_queries()
    for camera, rectangle, transition, clip in product(((0, 0, 0), (1, 1, 1)), RECTANGLES[:4], variants, CLIPS):
        prefix = rectangle_prefix(camera, rectangle)
        requests += [f'{prefix} {rectangle_transition(transition)} 1 {clip} {query}' for query in queries]
    prefix = rectangle_prefix((0, 0, 0), '0 0 0 0 0', built=0)
    requests += [f'{prefix} 0 1 0 0 {IDENTITY} 1 {CLIPS[0]} {query}' for query in queries]
