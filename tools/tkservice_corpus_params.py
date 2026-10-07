"""Deterministic requests for Graphic3d_RenderingParams, Graphic3d_ZLayerSettings and
Graphic3d_PresentationAttributes (tools/oracle/tkservice_probe_params.cxx).

`sv_params` compares the default rendering parameters (with several resolutions), Z layer
settings (origins near and away from zero, finite/infinite/negative culling, depth offsets and
every option) and presentation attributes, through their JSON dumps.
"""
from __future__ import annotations
import random


def numbers(values: list[float]) -> str:
    """The fields of a request, with every double spelled exactly."""
    return ' '.join(repr(float(v)) for v in values)


def append_params_requests(requests: list[str]) -> None:
    """Append rendering parameters, then seeded Z layers and presentation attributes."""
    for resolution in (72, 96, 144, 0):
        requests.append('sv_params ' + numbers([0, resolution]))
    rng = random.Random(20261010)
    for _ in range(30):
        origin = rng.choice([[0, 0, 0], [1e6, 0, 0], [1e-18, 0, 0], [-3.5, 2.0, 1e5]])
        culling = [rng.choice([1e100, 2e100, 100.0, 0.0, -1.0]) for _ in range(2)]
        flags = [rng.randrange(2) for _ in range(7)]
        requests.append('sv_params ' + numbers([1, *origin, *culling, rng.randrange(3), *flags]))
    for _ in range(20):
        color = [rng.choice([0.0, 0.25, 1.0]) for _ in range(3)]
        requests.append('sv_params ' + numbers([2, rng.randrange(2), *color, rng.choice([0.0, 0.3, 1.0, 1.5]),
                                                rng.choice([0, -2, 3]), rng.choice([0, 1, 7]), rng.randrange(2)]))
