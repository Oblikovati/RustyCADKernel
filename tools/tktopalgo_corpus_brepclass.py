"""Deterministic requests for BRepClass (tools/oracle/tktopalgo_probe_brepclass.cxx).

Every request carries its face record (tktopalgo_corpus_faces.py), so it builds its own
shape and never relies on an earlier request:
- `brc_defaults`: the default classifiers, intersector and edge;
- `brc_edge`: BRepClass_Edge scripts (geometry, boxes, edge and face changes);
- `brc_explorer`: the explorer's edges, next edges, pcurves and boxes per face;
- `brc_segment` and `brc_checkpoint`: the classification segments and the point check
  from points inside, outside, on edges, at vertices, far away and at infinity;
- `brc_intersect` and `brc_intersect2`: the intersector on every explorer edge, with and
  without boxes, in every box state, with finite and RealLast segments and small and
  large tolerances, and one intersector reused on two edges;
- `brc_classify`, `brc_classify3d` and `brc_passive`: the classifiers on grids and seeded
  random points of every face, with and without boxes.
"""
from __future__ import annotations
import math
import random
from tktopalgo_corpus_faces import CYLINDER, SPHERE, FaceSpec, named_faces

REAL_LAST = 1.7976931348623157e308
# The UV window of each face, around its boundary.
WINDOWS = {'square': (0.0, 2.0, 0.0, 2.0), 'square_reversed': (0.0, 2.0, 0.0, 2.0),
           'holed': (0.0, 10.0, 0.0, 10.0), 'holed_edges': (0.0, 10.0, 0.0, 10.0),
           'disc': (-2.0, 2.0, -2.0, 2.0), 'cylinder': (0.0, 2 * math.pi, 0.0, 5.0),
           'sphere': (0.0, 2 * math.pi, -0.5 * math.pi, 0.5 * math.pi),
           'bezier': (-6.0, 6.0, -6.0, 6.0), 'gap': (0.0, 4.0, 0.0, 4.0), 'strip': (-5.0, 5.0, 0.0, 1.0),
           'trimmed': (0.0, 10.0, 0.0, 10.0), 'internal': (0.0, 4.0, 0.0, 4.0), 'empty': (-1.0, 1.0, -1.0, 1.0),
           'cusp': (0.0, 4.0, 0.0, 3.0), 'half_strip': (0.0, 5.0, 0.0, 1.0), 'no_pcurve': (0.0, 2.0, 0.0, 2.0),
           'flat': (0.0, 4.0, 0.0, 2.0), 'cylinder_flipped': (0.0, 2 * math.pi, 0.0, 5.0)}


def numbers(values: list[float]) -> str:
    """The fields of a request, with every double spelled exactly."""
    return ' '.join(repr(float(v)) for v in values)


def request(op: str, face: FaceSpec, *fields: float) -> str:
    """One `op face fields...` request line."""
    return f'{op} ' + numbers([*face.numbers(), *fields])


def grid(window: tuple[float, float, float, float], steps: int = 7) -> list[tuple[float, float]]:
    """A grid over the window grown by a tenth on each side."""
    u0, u1, v0, v1 = window
    du, dv = (u1 - u0) * 0.1, (v1 - v0) * 0.1
    us = [u0 - du + (u1 - u0 + 2 * du) * i / (steps - 1) for i in range(steps)]
    vs = [v0 - dv + (v1 - v0 + 2 * dv) * j / (steps - 1) for j in range(steps)]
    return [(u, v) for u in us for v in vs]


def special_points(face: FaceSpec) -> list[tuple[float, float]]:
    """The vertices, points on the segment and line edges, just beside them and on the
    extensions of the segments (where a classification segment is tangent to an edge)."""
    points = [(u, v) for u, v, _ in face.vertices]
    for edge in face.edges:
        kind, pcurve = edge[0], edge[5:]
        if kind in (0, 3):
            (u1, v1, u2, v2) = pcurve[:4]
            u, v = 0.5 * (u1 + u2), 0.5 * (v1 + v2)
            points += [(u, v), (u + 1e-8, v + 1e-8), (u - 1e-4, v + 1e-4),
                       (u1 + 1.5 * (u2 - u1), v1 + 1.5 * (v2 - v1)), (u1 - 0.5 * (u2 - u1), v1 - 0.5 * (v2 - v1))]
        elif kind == 4 and abs(pcurve[4]) < 1e99 and abs(pcurve[5]) < 1e99:
            t = 0.5 * (pcurve[4] + pcurve[5])
            points += [(pcurve[0] + t * pcurve[2], pcurve[1] + t * pcurve[3])]
    return points


def through_vertex_points(face: FaceSpec) -> list[tuple[float, float]]:
    """Points whose first classification segment (to the point at 0.877 of the first
    segment edge) passes through a vertex: the segment ends there at an edge end."""
    first = next((edge for edge in face.edges if edge[0] == 0), None)
    if first is None:
        return []
    u1, v1, u2, v2 = first[5:9]
    target = (u1 + 0.877 * (u2 - u1), v1 + 0.877 * (v2 - v1))
    beyond = [(u + 0.5 * (u - target[0]), v + 0.5 * (v - target[1])) for u, v, _ in face.vertices]
    return [target, *beyond]


def along_axis_points(face: FaceSpec) -> list[tuple[float, float]]:
    """Far points along the first edge of a face whose first edge is a Bezier curve
    (the flat face): every probed segment is almost tangent to that edge."""
    if not face.edges or face.edges[0][0] != 2:
        return []
    return [(-100.0, 0.0), (200.0, -0.0002), (-50.0, 1e-5), (-3.0, 0.0)]


def far_points(window: tuple[float, float, float, float]) -> list[tuple[float, float]]:
    """Points whose distance to the face centre is huge, infinite, or at the centre."""
    u0, u1, v0, v1 = window
    centre = (0.5 * (u0 + u1), 0.5 * (v0 + v1))
    return [(1e20, 3.0), (-1e18, -1e18), (1e150, 0.0), (2.0, -1e120), centre]


def append_defaults_and_edges(requests: list[str], faces: dict[str, FaceSpec]) -> None:
    """`brc_defaults` and the BRepClass_Edge scripts."""
    requests.append('brc_defaults')
    scripts = [[0, 1, -1.0, -1.0, 3.0, 2.0, 6, 1, 4, 0, 1],
               [0, 8, 1, 7, 0.25, 2, 0, 5, 1, 0, 3, 4, 0, 0],
               [1, 0.0, 0.0, 2.0, 2.0, 0, 6, 2, 5, 0],
               [3, 0, 2, 1, 5.0, 5.0, 6.0, 6.0, 4, 1, 1]]
    for name in ('square', 'cylinder', 'holed'):
        face = faces[name]
        for k in range(min(3, len(face.edges))):
            for mode in (0, 1):
                for script in scripts:
                    requests.append(request('brc_edge', face, k, mode, count_commands(script), *script))


def count_commands(script: list[float]) -> int:
    """The number of commands in an edge script (codes 1 and 4 take extra fields)."""
    extra = {0: 0, 1: 4, 2: 0, 3: 0, 4: 2, 5: 1, 6: 1, 7: 1, 8: 1}
    count, index = 0, 0
    while index < len(script):
        index += 1 + extra[int(script[index])]
        count += 1
    return count


def append_explorer_requests(requests: list[str], faces: dict[str, FaceSpec]) -> None:
    """`brc_explorer`, `brc_segment` and `brc_checkpoint` on every face."""
    for name, face in faces.items():
        for gap, bounds in ((0.1, 0), (0.1, 1), (0.05, 1)):
            requests.append(request('brc_explorer', face, gap, bounds))
        window = WINDOWS[name]
        points = grid(window, 4) + special_points(face)[:12] + far_points(window) + along_axis_points(face)
        for u, v in points:
            requests.append(request('brc_segment', face, u, v, 6))
            requests.append(request('brc_checkpoint', face, u, v))


def edge_lines(window: tuple[float, float, float, float], rng: random.Random) -> list[list[float]]:
    """Segment origins inside and outside the window with seeded directions."""
    u0, u1, v0, v1 = window
    lines = []
    for _ in range(6):
        u, v = rng.uniform(u0 - 1, u1 + 1), rng.uniform(v0 - 1, v1 + 1)
        angle = rng.uniform(0.0, 2 * math.pi)
        lines.append([u, v, math.cos(angle), math.sin(angle)])
    centre = [0.5 * (u0 + u1), 0.5 * (v0 + v1)]
    lines += [[*centre, 1.0, 0.0], [*centre, 0.0, -1.0], [*centre, math.sqrt(0.5), -math.sqrt(0.5)]]
    return lines


def vertex_lines(face: FaceSpec, centre: tuple[float, float]) -> list[list[float]]:
    """Lines from the centre through each vertex (the vertex is an edge end on them)."""
    lines = []
    for u, v, _ in face.vertices:
        du, dv = u - centre[0], v - centre[1]
        norm = math.hypot(du, dv)
        if norm > 1e-9:
            lines.append([*centre, du / norm, dv / norm])
    return lines


def append_intersect_requests(requests: list[str], faces: dict[str, FaceSpec]) -> None:
    """`brc_intersect` on every explorer edge and `brc_intersect2` on edge pairs."""
    rng = random.Random(1992)
    settings = [(REAL_LAST, 1e-7, 0.1, 0, 0), (REAL_LAST, 1e-7, 0.1, 1, 0), (20.0, 1e-7, 0.1, 1, 1),
                (3.0, 0.3, 0.1, 1, 2), (REAL_LAST, 0.3, 0.1, 0, 3), (50.0, 1e-3, 0.05, 1, 0)]
    for name, face in faces.items():
        nb_edges = sum(len(edges) for _, edges in face.wires)
        u0, u1, v0, v1 = WINDOWS[name]
        lines = edge_lines(WINDOWS[name], rng) + vertex_lines(face, (0.5 * (u0 + u1), 0.5 * (v0 + v1)))
        for k in range(nb_edges):
            for index, line in enumerate(lines):
                par, tol, gap, bounds, mode = settings[(index + k) % len(settings)]
                requests.append(request('brc_intersect', face, k, *line, par, tol, gap, bounds, mode))
        for k in range(max(0, nb_edges - 1)):
            for line in lines[:3]:
                requests.append(request('brc_intersect2', face, k, k + 1, *line, REAL_LAST, 1e-7))


def append_classify_requests(requests: list[str], faces: dict[str, FaceSpec]) -> None:
    """`brc_classify` on grids, special and random points; `brc_passive` on lines."""
    rng = random.Random(2008)
    for name, face in faces.items():
        window = WINDOWS[name]
        u0, u1, v0, v1 = window
        randoms = [(rng.uniform(u0 - 1, u1 + 1), rng.uniform(v0 - 1, v1 + 1)) for _ in range(12)]
        points = grid(window) + special_points(face) + through_vertex_points(face) + far_points(window)
        points += along_axis_points(face) + randoms
        for index, (u, v) in enumerate(points):
            tol, bounds = ((1e-7, 0), (1e-7, 1), (1e-3, 0), (0.05, 1))[index % 4]
            requests.append(request('brc_classify', face, u, v, tol, bounds, 0.1))
        centre = (0.5 * (u0 + u1), 0.5 * (v0 + v1))
        for line in edge_lines(window, rng)[:6] + vertex_lines(face, centre):
            for mode in (0, 1):
                requests.append(request('brc_passive', face, mode, *line, 40.0, 1e-7))


def surface_point(face: FaceSpec, u: float, v: float, offset: float) -> list[float]:
    """The point of the face's surface at (u, v), moved by `offset` along the normal
    (the planes are z = 0, the cylinder and the sphere have the radius 2 about z)."""
    kind = face.surface[0]
    if kind == CYLINDER:
        return [(2.0 + offset) * math.cos(u), (2.0 + offset) * math.sin(u), v]
    if kind == SPHERE:
        r = 2.0 + offset
        return [r * math.cos(v) * math.cos(u), r * math.cos(v) * math.sin(u), r * math.sin(v)]
    return [u, v, offset]


def append_classify3d_requests(requests: list[str], faces: dict[str, FaceSpec]) -> None:
    """`brc_classify3d` on points near the surface of every face with a bounded surface
    window (the strip and the empty face are unbounded), and on seeded points around it."""
    rng = random.Random(1993)
    for name, face in faces.items():
        if name in ('strip', 'empty'):
            continue
        u0, u1, v0, v1 = WINDOWS[name]
        points = [surface_point(face, rng.uniform(u0, u1), rng.uniform(v0, v1), rng.uniform(-0.5, 0.5))
                  for _ in range(14)]
        points += [[rng.uniform(-6, 11), rng.uniform(-6, 11), rng.uniform(-3, 6)] for _ in range(4)]
        for index, point in enumerate(points):
            tol, bounds = ((1e-7, 0), (1e-4, 1))[index % 2]
            requests.append(request('brc_classify3d', face, *point, tol, bounds, 0.1))


def append_brepclass_requests(requests: list[str]) -> None:
    """Append every BRepClass request family in a fixed order."""
    faces = named_faces()
    append_defaults_and_edges(requests, faces)
    append_explorer_requests(requests, faces)
    append_intersect_requests(requests, faces)
    append_classify_requests(requests, faces)
    append_classify3d_requests(requests, faces)
