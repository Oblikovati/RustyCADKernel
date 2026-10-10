"""Native frustum SAT/quadric requests with identical cached geometry on both sides."""
from itertools import product

Point = tuple[float, float, float]


def subtract(first: Point, last: Point) -> Point:
    """Coordinate displacement used only to construct oracle fixtures."""
    return tuple(a-b for a, b in zip(first, last))


def cross(first: Point, last: Point) -> Point:
    """Unnormalized fixture plane normal; both probes read the resulting numbers."""
    a, b, c = first
    x, y, z = last
    return b*z-c*y, c*x-a*z, a*y-b*x


def encode_numbers(values: list[float]) -> str:
    """Round-trip finite fixture values, including signed zero."""
    return ' '.join(format(value, '.17g') for value in values)


def frustum_state(n: int, ortho: int, variant: int) -> str:
    """A prism, tapered frustum, rotated/sheared frustum or fully degenerate cache."""
    outline = [(-1, 1), (-1, -1), (1, 1), (1, -1)] if n == 4 else [(-1, -1), (1, -1), (0, 1)]
    factor = 1 if ortho else 2
    vertices = [(x*scale, y*scale, z) for x, y in outline for scale, z in ((1, -2), (factor, 2))]
    if variant == 1:
        vertices = [(z + .375*x, y + .25*z, -x) for x, y, z in vertices]
    if variant == 2:
        vertices = [(0., 0., 0.) for _ in vertices]
    indices = [(0, 4, 1), (2, 3, 6), (0, 1, 2), (4, 6, 5), (0, 2, 4), (1, 5, 3)] if n == 4 else [(0, 1, 2), (2, 3, 4), (0, 4, 1), (0, 2, 4), (1, 5, 3)]
    planes = [cross(subtract(vertices[b], vertices[a]), subtract(vertices[c], vertices[a])) for a, b, c in indices]
    edges = [(4, 0), (2, 0), (3, 2), (7, 6), (1, 0), (5, 4)] if n == 4 else [(0, 1), (2, 3), (4, 5), (2, 0), (4, 2), (4, 0)]
    return encode_cache(planes, vertices, [subtract(vertices[a], vertices[b]) for a, b in edges])


def encode_cache(planes: list[Point], vertices: list[Point], edges: list[Point]) -> str:
    """Serialize each complete native protected array in the probe's fixed order."""
    projections = [[sum(a*b for a, b in zip(normal, point)) for point in vertices] for normal in planes]
    values = [value for group in (planes, vertices) for point in group for value in point]
    values += [max(row) for row in projections] + [min(row) for row in projections]
    values += [max(point[i] for point in vertices) for i in range(3)]
    values += [min(point[i] for point in vertices) for i in range(3)]
    values += [value for edge in edges for value in edge]
    return encode_numbers(values)


def sat_queries() -> list[str]:
    """Point/segment/AABB/polygon/triangle tests, preserving inside and normal outputs."""
    queries = [f'0 123 -456 789 {point}' for point in ('0 0 0', '5 0 0', '-1 1 -2', '0 0 10', '0 0 -3')]
    segments = ('-5 0 0 5 0 0', '3 3 3 4 4 4', '10 10 10 10 10 10', '0 0 0 0 0 1e-10', '-1 1 -2 1 -1 2')
    queries += [f'1 123 -456 789 {segment}' for segment in segments]
    boxes = ('-.5 -.5 -.5 .5 .5 .5', '-3 -3 -3 3 3 3', '5 5 5 6 6 6', '-1 -1 -2 1 1 2', '0 0 0 0 0 0', '1 1 1 -1 -1 -1')
    queries += [f'2 123 -456 789 {bounds} {mode}' for bounds, mode in product(boxes, (-1, 0, 1))]
    triangles = ('-2 0 0 2 0 0 0 2 0', '4 4 4 5 4 4 4 5 4', '0 0 0 0 0 0 0 0 0', '0 0 9 1 0 9 0 1 9', '-1 -1 -2 1 -1 -2 0 1 -2')
    queries += [f'4 123 -456 789 {triangle}' for triangle in triangles]
    queries += [f'3 123 -456 789 {lower} 3 {triangle}' for triangle, lower in product(triangles, (-3, 1))]
    queries.append('3 123 -456 789 1 4 -2 -2 0 2 -2 0 2 2 0 -2 2 0')
    return queries


def quadric_queries() -> list[str]:
    """Quadrics include side/cap intersections, containment, transformations and exceptions."""
    centers = ('0 0 0', '2 0 0', '4 4 4', '0 0 20', '-1 -1 -2')
    queries = [f'0 {mode} {center} {radius}' for center, radius, mode in product(centers, (0, .5, 1, 3), (-1, 0, 1))]
    transforms = ('0 0 1 0 0 0 0 1', '0 1 0 1.5707963267948966 0 0 0 1', '1 2 3 .4 2 -1 .5 .75', '1 0 0 -.3 -1 1 3 -2', '0 0 1 0 0 0 0 0')
    for radii, height, hollow, transform, mode in product(('1 1', '2 1', '1 0'), (0, 2), (0, 1), transforms, (-1, 0, 1)):
        queries.append(f'1 {mode} {radii} {height} {hollow} {transform}')
    queries += [f'2 {mode} {radius} {filled} {transform}' for radius, filled, transform, mode in product((0, 1, 3), (0, 1), transforms, (-1, 0, 1))]
    polygon = '0 4 -1 -1 0 1 -1 0 1 1 0 -1 1 0'
    queries += [f'3 0 {point} {polygon}' for point in ('0 0 0', '2 2 0', '-1 -1 0')]
    queries += [f'4 0 1 2 3 {transform} {polygon}' for transform in transforms]
    return queries


def append_frustum_requests(requests: list[str]) -> None:
    """Exercise N=3/N=4, both projection modes, raw degeneracy and all protected algorithms."""
    for n, ortho, variant in product((3, 4), (0, 1), (0, 1, 2)):
        state = frustum_state(n, ortho, variant)
        requests.extend(f'v3_frustum_core {n} {ortho} 0 {state} {query}' for query in sat_queries())
        requests.extend(f'v3_frustum_core {n} {ortho} 1 {state} {query}' for query in quadric_queries())
        requests.extend(f'v3_frustum_core {n} {ortho} 2 {state} {depth}' for depth in (-1, 0, 1))
