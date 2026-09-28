"""Deterministic requests for the Extrema surface-surface classes (tools/oracle/tkgeombase_probe_extrema_ss.cxx).

Extrema_ExtSS runs two planes (parallel, secant, trimmed) and the generic search of
Extrema_GenExtSS on pairs of every surface kind (quadrics, splines, Bezier, swept,
offset and trimmed surfaces), apart, touching, secant and nested, with bounds that
keep or drop the solutions and periodic bounds beyond one period. Extrema_GenExtSS
runs directly with few and many samples and sub-ranges; Extrema_FuncExtSS is
evaluated inside and outside the bounds.
"""
from __future__ import annotations
import math
import random
from tkgeombase_corpus_gce import add
from tkgeombase_corpus_extrema_ps import adaptor, surface_specs

PI = math.pi


def sub_patch(patch, a=0.1, b=0.3):
    u1, u2, v1, v2 = patch
    return (u1 + a * (u2 - u1), u2 - b * (u2 - u1), v1 + b * (v2 - v1), v2 - a * (v2 - v1))


def pairs(surfaces):
    """Pairs of surfaces: each with its neighbours in the list."""
    out = []
    for i, first in enumerate(surfaces):
        for step in (1, 5):
            out.append((first, surfaces[(i + step) % len(surfaces)]))
    return out


def plane_requests(requests):
    z0 = [0, 0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0]
    z2 = [0, 0, 1.0, 2.0, 2.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0]
    tilted = [0, 0, 0.0, 0.0, 1.0, 0.0, 0.6, 0.8, 1.0, 0.0, 0.0]
    patch = (-3.0, 3.0, -3.0, 3.0)
    for a, b in ((z0, z2), (z0, tilted), (tilted, z2), (z0, z0)):
        for p1, p2 in ((patch, patch), ((5.0, 6.0, 5.0, 6.0), patch)):
            add(requests, 'ex_ss', *a, 1, *patch, *b, 1, *patch, 1.0e-9, 1.0e-9, *p1, *p2)


def append_extrema_ss_requests(requests: list[str]) -> None:
    """Append the surface-surface requests (seed 20261004)."""
    rng = random.Random(20261004)
    surfaces = surface_specs(rng)
    for index, ((n1, s1, q1), (n2, s2, q2)) in enumerate(pairs(surfaces)):
        a, b = adaptor(n1, s1, q1, index % 3 == 0), adaptor(n2, s2, q2, index % 3 == 1)
        u1, u2, v1, v2 = q1
        s1_uv = [(u1, v1, 0.5 * (u1 + u2), 0.5 * (v1 + v2)), (u2 + 1.0, v1, u1, v2)]
        flat = [v for uv in s1_uv for v in (uv[0], uv[1], q2[0] + 0.3 * (q2[1] - q2[0]), q2[3])]
        if index % 3 == 0:
            add(requests, 'ex_fess', *a, *b, 2, *flat)
        if index % 2 == 0:
            nu, nv = (5, 7) if index % 4 == 0 else (12, 12)
            add(requests, 'ex_genss', *a, *b, nu, nv, 1.0e-9, 1.0e-9, *sub_patch(q1), *sub_patch(q2, 0.2, 0.0))
        add(requests, 'ex_ss', *a, *b, 1.0e-9, 1.0e-9, *sub_patch(q1), *sub_patch(q2, 0.0, 0.2))
    plane_requests(requests)
