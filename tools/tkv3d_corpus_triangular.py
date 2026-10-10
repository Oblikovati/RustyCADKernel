"""Concrete triangular frustum construction, state transitions and overload contracts."""
from __future__ import annotations
from itertools import product

IDENTITY = '1 0 0 0 0 1 0 0 0 0 1 0'
SHEAR = '1 .25 0 3 0 -2 .125 -1 .1 0 .5 2'
COLLAPSE = '0 0 0 1 0 0 0 2 0 0 0 3'
TRIANGLES = ('200 100 600 100 400 500', '400 500 600 100 200 100',
             '100 100 300 300 500 500', '400 300 400 300 400 300')
QUADRIC_TRANSFORMS = ('0 0 1 0 0 0 0 1', '1 2 3 .7 2 -1 3 -2')


def triangular_queries() -> list[str]:
    """Exercise all overloads with retained pick results and optional inside flags."""
    requests: list[str] = []
    for mode, bounds in product((-1, 0, 1), ('-1 -1 -1 1 1 1', '50 50 50 51 51 51',
                                          '-100 -100 -100 100 100 100', '0 0 0 0 0 0')):
        requests.append(f'0 {mode} {bounds}')
    requests += [f'1 1 {p}' for p in ('0 0 0', '0 0 9.95', '0 0 -100', '30 30 0', '-1 2 -5')]
    requests += [f'2 1 {p}' for p in ('-20 0 0 20 0 0', '0 0 0 0 0 0', '50 50 50 50 50 50', '1 2 3 4 5 6')]
    polygons = ('-3 3 -1 -1 0 1 -1 0 0 1 0', '4 4 -20 -20 0 20 -20 0 20 20 0 -20 20 0',
                '2 2 0 0 0 1 0 0', '1 0')
    for sensitivity in (0, 1, 7):
        requests += [f'3 1 {sensitivity} {points}' for points in polygons]
        requests += [f'4 1 {sensitivity} {points}' for points in
                     ('-1 -1 0 1 -1 0 0 1 0', '50 50 50 51 50 50 50 51 50', '0 0 0 0 0 0 0 0 0')]
    for mode, center, radius in product((-1, 0, 1), ('0 0 0', '0 0 -100', '8 8 0', '50 50 0'), (0, 3)):
        requests.append(f'5 {mode} {center} {radius}')
    append_triangular_quadrics(requests)
    return requests


def append_triangular_quadrics(requests: list[str]) -> None:
    """Compare filled/hollow, affine orientation, cone and degenerate-radius queries."""
    for mode, radii, hollow, transform in product((-1, 0, 1), ('1 1 4', '3 0 2'), (0, 1), QUADRIC_TRANSFORMS):
        requests.append(f'6 {mode} {radii} {hollow} {transform}')
    for mode, radius, filled, transform in product((-1, 0, 1), (0, 5), (0, 1), QUADRIC_TRANSFORMS):
        requests.append(f'7 {mode} {radius} {filled} {transform}')


def append_triangular_requests(requests: list[str]) -> None:
    """Cross native builders, reversed/degenerate pixels and copy/transform transitions."""
    queries = triangular_queries()
    for perspective, zero, viewport, triangle in product((0, 1), (0, 1), (0, 1), TRIANGLES):
        prefix = f'v3_triangular {perspective} {zero} {viewport} {triangle}'
        variants = ((0, IDENTITY), (1, SHEAR), (1, COLLAPSE), (2, IDENTITY), (3, IDENTITY), (4, IDENTITY))
        for (variant, matrix), depth in product(variants, (-1, 0, 1, 2)):
            requests.append(f'{prefix} {variant} -1 {matrix} 0 0 {depth}')
            if variant != 4:
                requests.append(f'{prefix} {variant} 8 {matrix} 0 1 {depth}')
        for variant, matrix in ((0, IDENTITY), (1, SHEAR), (3, IDENTITY)):
            requests += [f'{prefix} {variant} 4 {matrix} 1 {query}' for query in queries]
