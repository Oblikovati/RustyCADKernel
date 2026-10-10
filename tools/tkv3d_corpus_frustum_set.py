"""Polyline selection: native Delaunay child order, boundary containment and shared pixels."""
from __future__ import annotations
from itertools import product
from tkv3d_corpus_triangular import IDENTITY, SHEAR, COLLAPSE
from tkv3d_corpus_axis import CLIPS, axis_queries

POLYLINES = (
    '1 3 200 100 600 100 400 500',
    '1 4 200 100 600 100 600 500 200 500',
    '1 4 200 500 600 500 600 100 200 100',
    '1 6 200 100 600 100 600 300 400 300 400 500 200 500',
    '1 5 200 100 600 100 600 500 200 500 200 100',
    '1 3 100 100 300 300 500 500',
    '1 1 400 300',
    '1 2 200 100 600 500',
)
VARIANTS = ((0, 1, 0, IDENTITY), (1, -1, 0, SHEAR), (1, 4, 1, IDENTITY),
            (1, 1, 2, COLLAPSE), (2, 1, 1, IDENTITY), (2, 1, 0, IDENTITY),
            (3, 1, 1, IDENTITY), (4, 1, 2, IDENTITY), (5, 1, 2, IDENTITY))


def set_prefix(camera: tuple[int, int, int], allow: int, polyline: str, initialized: int = 1, built: int = 1) -> str:
    """Encode the native Init/Build state independently."""
    return f'v3_frustum_set {" ".join(map(str, camera))} {allow} {initialized} {built} {polyline}'


def set_variant(variant: tuple[int, int, int, str]) -> str:
    """Retain ignored scale and caller-provided builder as separate inputs."""
    return ' '.join(map(str, variant))


def append_frustum_set_states(requests: list[str]) -> None:
    """Compare empty/degenerate meshes, native order, array aliasing and all dump depths."""
    for camera, allow, polyline, variant, depth in product(((0, 0, 0), (1, 1, 1)), (0, 1),
                                                          POLYLINES, VARIANTS, (-1, 0, 1, 2, 3)):
        prefix = set_prefix(camera, allow, polyline)
        requests.append(f'{prefix} {set_variant(variant)} 0 0 {depth}')
    for initialized, built, variant in product((0, 1), (0, 1), VARIANTS):
        prefix = set_prefix((0, 0, 0), 0, POLYLINES[1], initialized, built)
        requests.append(f'{prefix} {set_variant(variant)} 0 0 -1')
    for lower in (-3, 0, 2, 5):
        polyline = f'{lower} 3 200 100 600 100 400 500'
        requests.append(f'{set_prefix((0, 0, 0), 0, polyline)} 0 1 0 {IDENTITY} 0 0 -1')


def append_frustum_set_requests(requests: list[str]) -> None:
    """Exercise every pick/inside overload against convex, concave and copied volumes."""
    append_frustum_set_states(requests)
    append_frustum_set_boundary_cases(requests)
    queries = axis_queries()
    for camera, allow, polyline, variant, clip in product(((0, 0, 0), (1, 1, 1)), (0, 1),
            POLYLINES[:4], (VARIANTS[0], VARIANTS[1], VARIANTS[6]), (CLIPS[0], CLIPS[2])):
        prefix = set_prefix(camera, allow, polyline)
        requests += [f'{prefix} {set_variant(variant)} 1 {clip} {query}' for query in queries]
    for initialized, built, polyline in ((0, 0, POLYLINES[1]), (1, 0, POLYLINES[1]), (1, 1, POLYLINES[6])):
        prefix = set_prefix((0, 0, 0), 0, polyline, initialized, built)
        requests += [f'{prefix} 0 1 0 {IDENTITY} 1 {CLIPS[0]} {query}' for query in queries]


def append_frustum_set_boundary_cases(requests: list[str]) -> None:
    """Probe copies before rebuild and circle-face crossings, including radius failures."""
    for camera, allow, variant in product(((0, 0, 0), (1, 1, 1)), (0, 1),
                                         (VARIANTS[4], VARIANTS[7], VARIANTS[8])):
        prefix = set_prefix(camera, allow, POLYLINES[1])
        requests += [f'{prefix} {set_variant(variant)} 1 {CLIPS[0]} {query}' for query in axis_queries()]
    transforms = ('0 0 1 0 4.9 0 0 1', '0 0 1 0 5 0 0 1',
                  '1 0 0 1.5707963267948966 0 0 0 1', '1 2 3 .7 0 0 0 -2')
    for allow, radius, filled, mode, transform in product((0, 1), (-1, 1, 6), (0, 1), (-1, 0, 1), transforms):
        prefix = set_prefix((0, 0, 0), allow, POLYLINES[1])
        requests.append(f'{prefix} 0 1 0 {IDENTITY} 1 {CLIPS[0]} 7 {mode} {radius} {filled} {transform}')
