"""Deterministic requests for the placement of HelixGeom_BuilderHelix
(tools/oracle/tkhelix_probe_placement.cxx).

`hx_placement P N X` compares the displacement from the standard frame to
gp_Ax3(gp_Ax2(P, N, X)): the standard frame itself, axis-aligned and skew frames, a
reversed axis, huge and tiny locations, an X direction parallel to N (gp_Ax2 raises
Standard_ConstructionError) and seeded random frames.
"""
from __future__ import annotations
import random


def numbers(values: list[float]) -> str:
    """The fields of a request, with every double spelled exactly."""
    return ' '.join(repr(float(v)) for v in values)


def placement_request(location: list[float], normal: list[float], x: list[float]) -> str:
    """One `hx_placement P N X` request line."""
    return 'hx_placement ' + numbers(location + normal + x)


def fixed_frames() -> list[tuple[list[float], list[float], list[float]]]:
    """Frames chosen for the boundary cases of the displacement."""
    return [([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]),
            ([10.0, 20.0, 30.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
            ([-0.0, 0.0, -0.0], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0]),
            ([1e300, -1e-300, 5.0], [1.0, 1.0, 1.0], [1.0, -1.0, 0.0]),
            ([0.1, 0.2, 0.3], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]),
            ([1.0, 2.0, 3.0], [0.0, 0.0, 1.0], [0.0, 0.0, 2.0]),
            ([1.0, 2.0, 3.0], [0.3, -0.4, 0.5], [0.7, 0.1, -0.2])]


def append_placement_requests(requests: list[str]) -> None:
    """Append the placement requests: fixed frames, then 16 seeded random ones."""
    rng = random.Random(20261001)
    frames = fixed_frames()
    for _ in range(16):
        frames.append(([rng.uniform(-50.0, 50.0) for _ in range(3)],
                       [rng.uniform(-1.0, 1.0) for _ in range(3)],
                       [rng.uniform(-1.0, 1.0) for _ in range(3)]))
    requests.extend(placement_request(*frame) for frame in frames)
