"""Deterministic requests for Graphic3d_PBRMaterial and Graphic3d_BSDF
(tools/oracle/tkservice_probe_pbr.cxx).

`sv_pbr_lut` generates environment lookup tables of several sizes and sample counts
(including one-pixel dimensions, where the native divisor wraps); `sv_pbr_factor` compares
the importance-sampling factor and the normalized roughness; `sv_pbr_bsdf` converts PBR
materials (metal, dielectric, transparent glass, emissive, out-of-range values that raise)
to BSDFs and back; `sv_fresnel` compares the conductor Fresnel models.
"""
from __future__ import annotations
import random


def numbers(values: list[float]) -> str:
    """The fields of a request, with every double spelled exactly."""
    return ' '.join(repr(float(v)) for v in values)


LUTS = [[4, 4, 16], [8, 2, 64], [1, 3, 8], [3, 1, 8], [16, 16, 32], [5, 7, 1], [2, 2, 0]]
MATERIALS = [
    [0.8, 0.6, 0.2, 1.0, 1.0, 0.3, 1.5, 0.0, 0.0, 0.0],
    [0.5, 0.5, 0.5, 1.0, 0.0, 1.0, 1.5, 0.0, 0.0, 0.0],
    [0.9, 0.95, 1.0, 0.3, 0.0, 0.0, 1.33, 0.0, 0.0, 0.0],
    [0.2, 0.7, 0.1, 0.6, 0.5, 0.5, 1.0, 0.0, 0.0, 0.0],
    [1.0, 1.0, 1.0, 1.0, 0.0, 0.2, 2.0, 1.0, 0.5, 0.25],
    [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 3.0, 0.0, 0.0, 0.0],
    [0.5, 0.5, 0.5, 1.0, 1.5, 0.5, 1.5, 0.0, 0.0, 0.0],
    [0.5, 0.5, 0.5, 1.0, 0.5, 0.5, 0.5, 0.0, 0.0, 0.0],
    [0.5, 0.5, 0.5, 1.0, 0.5, 0.5, 1.5, -1.0, 0.0, 0.0],
]


def append_pbr_requests(requests: list[str]) -> None:
    """Append the PBR and BSDF requests: fixed cases, then seeded random ones."""
    requests.extend('sv_pbr_lut ' + numbers(lut) for lut in LUTS)
    requests.extend('sv_pbr_bsdf ' + numbers(material) for material in MATERIALS)
    rng = random.Random(20261010)
    for p in [0.0, 0.25, 0.5, 0.999, 1.0]:
        for rough in [0.0, 0.01, 0.5, 1.0]:
            requests.append('sv_pbr_factor ' + numbers([p, rough]))
    for _ in range(32):
        requests.append('sv_pbr_factor ' + numbers([rng.random(), rng.random()]))
        material = [rng.random() for _ in range(4)] + [rng.choice([0.0, 1.0, rng.random()]), rng.random(),
                    rng.uniform(1.0, 3.0)] + [rng.choice([0.0, rng.random()]) for _ in range(3)]
        requests.append('sv_pbr_bsdf ' + numbers(material))
        requests.append('sv_fresnel ' + numbers([rng.uniform(0.1, 3.0) for _ in range(3)] +
                                                [rng.uniform(0.0, 5.0) for _ in range(3)]))
