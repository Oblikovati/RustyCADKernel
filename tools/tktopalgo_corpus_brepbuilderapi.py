"""Deterministic requests for BRepBuilderAPI (tools/oracle/tktopalgo_probe_brepbuilderapi*.cxx).

The maker and modification families live in tktopalgo_corpus_brepbuilderapi_makers.py and
tktopalgo_corpus_brepbuilderapi_modify.py. `tp_make_vertex` builds a vertex with
BRepBuilderAPI_MakeVertex and compares its point and its tolerance: the origin, signed
zeros, tiny, huge and non-representable decimal coordinates, and seeded random points.
"""
from __future__ import annotations
import random
from tktopalgo_corpus_brepbuilderapi_makers import append_brepbuilderapi_maker_requests
from tktopalgo_corpus_brepbuilderapi_modify import append_brepbuilderapi_modify_requests


def numbers(values: list[float]) -> str:
    """The fields of a request, with every double spelled exactly."""
    return ' '.join(repr(float(v)) for v in values)


def make_vertex_request(point: list[float]) -> str:
    """One `tp_make_vertex x y z` request line."""
    return 'tp_make_vertex ' + numbers(point)


def fixed_points() -> list[list[float]]:
    """Points chosen for the boundary spellings of the coordinates."""
    return [[0.0, 0.0, 0.0], [-0.0, 0.0, -0.0], [1.0, 2.0, 3.0], [0.1, 0.2, 0.3],
            [1e-300, -1e-300, 5e-324], [1e300, -1e300, 1e-7], [-2.5, 1e100, 7.0]]


def append_brepbuilderapi_requests(requests: list[str]) -> None:
    """Append the BRepBuilderAPI requests: the MakeVertex points (fixed, then 20 seeded
    random ones), the makers and the modifications."""
    rng = random.Random(20260930)
    points = fixed_points() + [[rng.uniform(-100.0, 100.0) for _ in range(3)] for _ in range(20)]
    requests.extend(make_vertex_request(point) for point in points)
    append_brepbuilderapi_maker_requests(requests)
    append_brepbuilderapi_modify_requests(requests)
