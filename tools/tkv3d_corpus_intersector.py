"""Ray crossings, output preservation and the base selecting-volume defaults."""
from itertools import product


def append_intersector_requests(requests: list[str]) -> None:
    """Exercise miss/tangent/inside/behind rays and cylinder/cone caps; append to `requests`."""
    origins = ('-3 0 0', '3 0 0', '0 0 0', '-3 1 0', '-3 2 0', '.1 .2 .3')
    rays = ('1 0 0', '0 1 0', '0 0 1', '1 2 3', '-1 -2 -3')
    for center, radius, origin, ray in product(('0 0 0', '1 -2 .5'), (0, 1, 2), origins, rays):
        requests.append(f'v3_ray_sphere {center} {radius} {origin} {ray}')
    for radius, x, ray, filled in product((0, 1, 2), (0, 1, 1.00000005, 1.0000002, 3), rays, (0, 1)):
        requests.append(f'v3_ray_circle {radius} {x} 0 -3 {ray} {filled}')
    append_cylinder_requests(requests, rays)
    requests.extend(('v3_base_intersector 0', 'v3_base_intersector 1'))


def append_cylinder_requests(requests: list[str], rays: tuple[str, ...]) -> None:
    """Cover degenerate heights, taper direction, cap-only and single crossings."""
    origins = ('0 0 -1', '0 0 1', '-3 0 1', '-3 1 1', '-2 0 0', '.2 -.3 1.5')
    for radii, height, origin, ray, hollow in product(('1 1', '2 1', '1 2', '1 0', '0 1'), (0, 2), origins, rays, (0, 1)):
        requests.append(f'v3_ray_cylinder {radii} {height} {origin} {ray} {hollow}')
    requests.append('v3_ray_cylinder 1 1 2 -2 0 0 1 0 1 1')
