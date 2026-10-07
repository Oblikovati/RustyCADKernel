"""Deterministic requests for the Graphic3d_ShaderObject declaration generator
(tools/oracle/tkservice_probe_shader.cxx).

`sv_shader_decl` builds the uniform and stage in/out declarations of every stage type, with and
without interface block names and geometry input vertices, over fixed and seeded variable sets.
"""
from __future__ import annotations
import random

STAGES = [1, 2, 4, 8, 16, 32]


def request(kind: int, has_in: int, has_out: int, verts: int, uniforms: list[int], in_outs: list[int]) -> str:
    """One `sv_shader_decl` line."""
    fields = [kind, has_in, has_out, verts, len(uniforms), *uniforms, len(in_outs), *in_outs]
    return 'sv_shader_decl ' + ' '.join(str(field) for field in fields)


FIXED_UNIFORMS = [0x11, 0x10, 0x01, 0x3f, 0x08]
FIXED_IN_OUTS = [0x11, 0x19, 0x09, 0x18, 0x01, 0x30]


def append_shader_requests(requests: list[str]) -> None:
    """Append the declaration requests: every stage and naming case, then seeded sets."""
    for kind in STAGES:
        for has_in in (0, 1):
            for has_out in (0, 1):
                for verts in (0, 3):
                    requests.append(request(kind, has_in, has_out, verts, FIXED_UNIFORMS, FIXED_IN_OUTS))
    requests.append(request(1, 0, 0, 0, [], []))
    rng = random.Random(20261012)
    for _ in range(48):
        uniforms = [rng.randrange(64) for _ in range(rng.randrange(4))]
        in_outs = [rng.randrange(64) for _ in range(rng.randrange(5))]
        requests.append(request(rng.choice(STAGES), rng.randrange(2), rng.randrange(2),
                                rng.choice([0, 1, 3, 6]), uniforms, in_outs))
