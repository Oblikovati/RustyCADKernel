"""Deterministic requests for the shared shape digest (tools/oracle/tkprim_probe_shape.cxx).

`pr_digest_vertex` builds a vertex with BRep_Builder and prints its TKTopAlgo digest
(`tp_shape`), which every TKPrim result goes through: the origin, signed zeros, tiny,
huge and non-representable decimal coordinates and tolerances, and seeded random points.
"""
from __future__ import annotations
import random


def numbers(values: list[float]) -> str:
    """The fields of a request, with every double spelled exactly."""
    return ' '.join(repr(float(v)) for v in values)


def digest_vertex_request(point: list[float], tolerance: float) -> str:
    """One `pr_digest_vertex x y z tol` request line."""
    return 'pr_digest_vertex ' + numbers([*point, tolerance])


def fixed_vertices() -> list[tuple[list[float], float]]:
    """Vertices chosen for the boundary spellings of the coordinates and tolerances."""
    return [([0.0, 0.0, 0.0], 1e-7), ([-0.0, 0.0, -0.0], 0.0), ([1.0, 2.0, 3.0], 1e-3),
            ([0.1, 0.2, 0.3], 0.1), ([1e-300, -1e-300, 5e-324], 1e-300),
            ([1e300, -1e300, 1e-7], 1e10), ([-2.5, 1e100, 7.0], 2.5)]


def append_shape_requests(requests: list[str]) -> None:
    """Append the digest requests: fixed vertices, then 20 seeded random ones."""
    rng = random.Random(20261001)
    randoms = [([rng.uniform(-100.0, 100.0) for _ in range(3)], rng.uniform(0.0, 1.0))
               for _ in range(20)]
    requests.extend(digest_vertex_request(point, tolerance)
                    for point, tolerance in fixed_vertices() + randoms)
