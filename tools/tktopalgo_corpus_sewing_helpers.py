"""Deterministic requests for the sewing helpers (tools/oracle/tktopalgo_probe_sewing_helpers.cxx).

BRepBuilderAPI_VertexInspector: clouds of points around a current point, some within
the tolerance along every axis, some beyond it along one axis, some in neighbouring
cells, on cell boundaries and at negative coordinates. BRepBuilderAPI_BndBoxTreeSelector:
UB-trees of boxes (disjoint, nested, touching, a random field) selected by a box, with
direct Reject and Accept calls.
"""
from __future__ import annotations
import random
from tktopalgo_corpus_breplib_records import numbers


def inspector_request(tol: float, points: list[tuple[float, float, float]], current: tuple[float, float, float],
                      targets: list[int]) -> str:
    fields = [tol, len(points), *[c for p in points for c in p], *current, len(targets), *targets]
    return 'bba_vertexinspector ' + numbers(fields)


def box_request(boxes: list[list[float]], current: list[float], rejects: list[list[float]], accepts: list[int]) -> str:
    fields = [len(boxes), *[c for b in boxes for c in b], *current, len(rejects), *[c for b in rejects for c in b],
              len(accepts), *accepts]
    return 'bba_boxselector ' + numbers(fields)


def cloud(seed: int, count: int, spread: float) -> list[tuple[float, float, float]]:
    rng = random.Random(seed)
    return [(rng.uniform(-spread, spread), rng.uniform(-spread, spread), rng.uniform(-spread, spread))
            for _ in range(count)]


def inspector_requests() -> list[str]:
    tol = 0.1
    around = [(0.05, 0.0, 0.0), (0.1, 0.1, 0.1), (0.1000001, 0.0, 0.0), (-0.05, -0.05, 0.02), (0.3, 0.0, 0.0),
              (0.0, 0.15, 0.0), (0.0, 0.0, -0.1), (0.2, 0.2, 0.2), (0.0, 0.0, 0.0)]
    requests = [inspector_request(tol, around, (0.0, 0.0, 0.0), [1, 2, 3, 4, 5, 9]),
                inspector_request(tol, around, (0.1, 0.1, 0.1), list(range(1, len(around) + 1))),
                inspector_request(tol, around, (-1.0, -1.0, -1.0), [1]),
                inspector_request(1e-6, [(1.0, 2.0, 3.0), (1.0 + 1e-6, 2.0, 3.0), (1.0 + 2e-6, 2.0, 3.0)],
                                  (1.0, 2.0, 3.0), [1, 2, 3]),
                inspector_request(tol, [], (0.0, 0.0, 0.0), [])]
    for seed, spread in ((1, 0.3), (2, 1.0), (3, 0.15)):
        points = cloud(seed, 40, spread)
        requests.append(inspector_request(tol, points, points[0], [1, 7, 13, 40]))
        requests.append(inspector_request(tol * 2, points, (0.0, 0.0, 0.0), [2, 3]))
    return requests


def box(x: float, y: float, z: float, size: float) -> list[float]:
    return [x, y, z, x + size, y + size, z + size]


def box_requests() -> list[str]:
    row = [box(float(i), 0.0, 0.0, 0.5) for i in range(6)]
    nested = [box(-i, -i, -i, 2.0 * i + 1.0) for i in range(4)]
    touching = [box(float(i), 0.0, 0.0, 1.0) for i in range(5)]
    rng = random.Random(7)
    field = [box(rng.uniform(0, 10), rng.uniform(0, 10), rng.uniform(0, 10), rng.uniform(0.1, 2.0)) for _ in range(60)]
    probes = [box(2.2, -0.1, -0.1, 1.5), box(20.0, 20.0, 20.0, 1.0), box(-0.5, -0.5, -0.5, 1.0)]
    requests = []
    for boxes in (row, nested, touching, field):
        for current in ([*probes[0]], [*probes[1]], box(4.0, 4.0, 4.0, 3.0)):
            requests.append(box_request(boxes, current, probes, [5, 1, 5]))
    requests.append(box_request([], box(0.0, 0.0, 0.0, 1.0), [], []))
    return requests


def append_sewing_helpers_requests(requests: list[str]) -> None:
    """Append the VertexInspector and BndBoxTreeSelector requests in a fixed order."""
    requests += inspector_requests()
    requests += box_requests()
