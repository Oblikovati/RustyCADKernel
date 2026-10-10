"""Frustum state propagation and boundary/sphere native requests."""
from itertools import product


def append_base_frustum_requests(requests: list[str]) -> None:
    """Cover builder lifetimes, camera modes and both untouched/changed inside flags."""
    for tolerance, size, perspective, zero, depth in product((-2, 2, 7), ('800 600', '0 -1'), (0, 1), (0, 1), (-1, 0, 1)):
        requests.append(f'v3_base_frustum {tolerance} {size} {perspective} {zero} {depth}')
    polygons = ((), ((1, 0, 0),), ((1, 0, 0), (2, 0, 0)),
                ((-2, 0, 0), (2, 0, 0), (-2, 0, 0)),
                ((-2, 1, 0), (2, 1, 0), (-2, 1, 0)),
                ((0, 0, 4), (2, 0, 4), (0, 0, 4)),
                ((2, 0, -1), (2, 0, 1), (2, 0, -1)),
                ((-2, 3, 0), (2, 3, 0), (0, 0, 0)),
                ((1, 0, 0), (2, 0, 0), (1, 0, 0)),
                ((-2, -2, 0), (2, -2, 0), (2, 2, 0), (-2, 2, 0), (-2, -2, 0)))
    for reject, center, radius, normal, inside, lower, points in product((0, 1), ('0 0 0', '1 -2 0'), (0, 1, 4), ('0 0 1', '1 2 3'), (0, 1), (-3, 1), polygons):
        vertices = ' '.join(str(value) for point in points for value in point)
        requests.append(f'v3_boundary_sphere {reject} {center} {radius} {normal} {inside} {lower} {len(points)} {vertices}')
