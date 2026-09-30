"""Deterministic requests for BRepTopAdaptor (tools/oracle/tktopalgo_probe_breptopadaptor.cxx).

Every request carries its face record (tktopalgo_corpus_faces.py and
tktopalgo_corpus_adaptor_faces.py), so it never relies on an earlier request:
- `bta_fclass2d`: BRepTopAdaptor_FClass2d on grids, vertices, edge points, points just
  beside the boundary, periodic copies and seeded points of every face, with small and
  large UV tolerances;
- `bta_destroy`: the classifier before and after Destroy;
- `bta_topol`: the arcs, vertices (HVertex parameter and resolution), samples and domain
  of BRepTopAdaptor_TopolTool, on restricted and unrestricted adaptors (not on the face
  with an edge without pcurve, whose arc crashes natively);
- `bta_classify`: Classify and IsThePointOn of the tool;
- `bta_tool` and `bta_errors`: BRepTopAdaptor_Tool and the exceptions of the tool.
"""
from __future__ import annotations
import math
import random
from tktopalgo_corpus_adaptor_faces import ADAPTOR_WINDOWS, adaptor_faces
from tktopalgo_corpus_faces import FaceSpec

TWO_PI = 2.0 * math.pi


def numbers(values: list[float]) -> str:
    """The fields of a request, with every double spelled exactly."""
    return ' '.join(repr(float(v)) for v in values)


def request(op: str, face: FaceSpec, *fields: float) -> str:
    """One `op face fields...` request line."""
    return f'{op} ' + numbers([*face.numbers(), *fields])


def grid(window: tuple[float, float, float, float], steps: int) -> list[tuple[float, float]]:
    """A grid over the window grown by a tenth on each side."""
    u0, u1, v0, v1 = window
    du, dv = (u1 - u0) * 0.1, (v1 - v0) * 0.1
    us = [u0 - du + (u1 - u0 + 2 * du) * i / (steps - 1) for i in range(steps)]
    vs = [v0 - dv + (v1 - v0 + 2 * dv) * j / (steps - 1) for j in range(steps)]
    return [(u, v) for u in us for v in vs]


def boundary_points(face: FaceSpec) -> list[tuple[float, float]]:
    """The vertices, the middles of the segment edges and points 1e-9 and 1e-5 beside
    them (inside and outside)."""
    points = [(u, v) for u, v, _ in face.vertices]
    for edge in face.edges:
        if edge[0] not in (0, 6):
            continue
        u1, v1, u2, v2 = edge[5:9]
        u, v = 0.5 * (u1 + u2), 0.5 * (v1 + v2)
        length = math.hypot(u2 - u1, v2 - v1) or 1.0
        nu, nv = -(v2 - v1) / length, (u2 - u1) / length
        for offset in (0.0, 1e-9, -1e-9, 1e-5, -1e-5):
            points.append((u + offset * nu, v + offset * nv))
    return points


def periodic_points(window: tuple[float, float, float, float]) -> list[tuple[float, float]]:
    """Copies of an inner point shifted by whole periods (and a far one)."""
    u0, u1, v0, v1 = window
    u, v = u0 + 0.37 * (u1 - u0), v0 + 0.41 * (v1 - v0)
    return [(u + TWO_PI, v), (u - 2 * TWO_PI, v), (u, v + TWO_PI), (u + 3 * TWO_PI, v - TWO_PI), (u + 1000.0, v)]


def seeded_points(window: tuple[float, float, float, float], rng: random.Random, count: int) -> list[tuple[float, float]]:
    """Seeded points around the window."""
    u0, u1, v0, v1 = window
    du, dv = 0.2 * (u1 - u0), 0.2 * (v1 - v0)
    return [(rng.uniform(u0 - du, u1 + du), rng.uniform(v0 - dv, v1 + dv)) for _ in range(count)]


def chunks(points: list[tuple[float, float]], size: int) -> list[list[tuple[float, float]]]:
    """The points in consecutive groups of `size` (one request each)."""
    return [points[i:i + size] for i in range(0, len(points), size)]


def flat(points: list[tuple[float, float]]) -> list[float]:
    """`n (u v)...`."""
    return [len(points), *[c for point in points for c in point]]


def append_fclass2d_requests(requests: list[str], faces: dict[str, FaceSpec]) -> None:
    """`bta_fclass2d` and `bta_destroy` on every face."""
    rng = random.Random(4217)
    for name, face in faces.items():
        window = ADAPTOR_WINDOWS[name]
        points = grid(window, 9) + boundary_points(face) + periodic_points(window) + seeded_points(window, rng, 24)
        for index, group in enumerate(chunks(points, 12)):
            tol, ontol = ((1e-9, 1e-7), (1e-4, 1e-3), (0.05, 1e-6))[index % 3]
            requests.append(request('bta_fclass2d', face, tol, ontol, *flat(group)))
        u0, u1, v0, v1 = window
        requests.append(request('bta_destroy', face, 1e-7, 0.5 * (u0 + u1), 0.5 * (v0 + v1)))


def append_topol_requests(requests: list[str], faces: dict[str, FaceSpec]) -> None:
    """`bta_topol`, `bta_classify`, `bta_tool` and `bta_errors` on every face."""
    rng = random.Random(733)
    for name, face in faces.items():
        # The arc of an edge without pcurve has no curve: a crash natively.
        for restr in (1, 0) if name != 'no_pcurve' else ():
            requests.append(request('bta_topol', face, restr))
        window = ADAPTOR_WINDOWS[name]
        points = grid(window, 5) + boundary_points(face)[:10] + seeded_points(window, rng, 8)
        for index, group in enumerate(chunks(points, 10)):
            restr, tol = ((1, 1e-7), (0, 1e-3), (1, 0.0))[index % 3]
            requests.append(request('bta_classify', face, restr, tol, *flat(group)))
        requests.append(request('bta_tool', face))
        if face.edges:
            requests.append(request('bta_errors', face))


def append_breptopadaptor_requests(requests: list[str]) -> None:
    """Append every BRepTopAdaptor request family in a fixed order."""
    faces = adaptor_faces()
    append_fclass2d_requests(requests, faces)
    append_topol_requests(requests, faces)
