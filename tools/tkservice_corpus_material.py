"""Deterministic requests for Graphic3d_MaterialAspect (tools/oracle/tkservice_probe_material.cxx).

`sv_material` compares every predefined material (colors, coefficients, BSDF, PBR material and
the DumpJson text); `sv_material_rank` the static catalogue, including the ranks that raise;
`sv_material_edit` the setters, with values that raise, that make a material user-defined,
and shine increases that leave [0, 1] and revert.
"""
from __future__ import annotations
import random

MATERIAL_COUNT = 26


def numbers(values: list[float]) -> str:
    """The fields of a request, with every double spelled exactly."""
    return ' '.join(repr(float(v)) for v in values)


EDITS = [
    [0, 0.5, 0.25, 1.0, 10.0, 0.5, 0.25, 1.5],
    [6, 0.5, 0.25, 1.0, 10.0, 0.5, 0.25, 1.5],
    [9, 1.0, 1.0, 1.0, -200.0, 1.5, -0.5, 0.5],
    [25, 0.0, 0.0, 0.0, 50.0, 0.0, 1.0, 1.0],
    [24, 0.2, 0.4, 0.6, 1000.0, 1.0, 0.0, 2.42],
    [17, 0.0, 0.787412, 0.142892, 0.0, 0.05, 2.0, 0.99],
]


def append_material_requests(requests: list[str]) -> None:
    """Append the material requests: every material and rank, fixed edits, seeded edits."""
    requests.extend(f'sv_material {name}' for name in range(MATERIAL_COUNT))
    requests.extend(f'sv_material_rank {rank}' for rank in range(-1, MATERIAL_COUNT + 1))
    requests.extend('sv_material_edit ' + numbers(edit) for edit in EDITS)
    rng = random.Random(20261011)
    for name in range(MATERIAL_COUNT):
        color = [rng.random() for _ in range(3)]
        requests.append('sv_material_edit ' + numbers(
            [name] + color + [rng.uniform(-150.0, 150.0), rng.uniform(-0.2, 1.2),
                              rng.uniform(-0.2, 1.2), rng.uniform(0.5, 3.0)]))
