"""Deterministic requests for Graphic3d_TransformPers and Graphic3d_TransformUtils
(tools/oracle/tkservice_probe_tpers.cxx).

`sv_tpers` builds every persistence kind (zoom, rotate, axial, trihedron, 2D, camera, ortho
and the scaled-above zoom, including the modes each constructor refuses) over random cameras
and compares the accessors, the persistent scale, ComputeApply, Apply in double and float,
Compute and the box Apply members; `sv_tutils` compares the rotate/scale/translate, ortho,
project/unproject, scale-factor and convert helpers in double and float.
"""
from __future__ import annotations
import random

from tkservice_corpus_camera import DEFAULT, camera, numbers

ANCHOR_MODES = [2, 8, 10, 512, 514, 2 | 256, 32]
CORNER_MODES = [32, 64, 64 | 256, 32 | 256, 2]
ONLY_MODES = [2, 8, 10, 512, 32, 64, 128, 256, 0, 2 | 256, 64 | 256]
CORNERS = [0, 1, 2, 4, 8, 5, 6, 9, 10]
BOXES = [[-1, -1, -1, 1, 1, 1], [0, 0, 0, 0, 0, 0], [10, 20, 30, 40, 50, 60]]


def persistence(rng: random.Random) -> list[float]:
    """`Kind Mode X Y Z Corner OffsetX OffsetY Scale`."""
    kind = rng.randrange(4)
    mode = rng.choice({0: ONLY_MODES, 1: ANCHOR_MODES, 2: CORNER_MODES, 3: [2]}[kind])
    point = [rng.choice([0.0, 10.0, -25.5, 3.25]) for _ in range(3)]
    return [kind, mode, *point, rng.choice(CORNERS), rng.choice([0, 15, -7]), rng.choice([0, 20, -3]),
            rng.choice([0.0, 0.5, 100.0])]


def utils_request(rng: random.Random) -> list[float]:
    """`Kind A X Y Z S(3) T(3) Ortho(6) Point(3) Viewport(4)`."""
    axis = rng.choice([[1, 0, 0], [0, -1, 0], [0, 0, 2], [-1, 0, 0], [0, 1, 0], [0, 0, -1], [1, 1, 0],
                       [0.3, -0.5, 0.8], [1e-5, 0, 1e-5], [0, 0, 0]])
    ortho = rng.choice([[-1, 1, -1, 1, 0.1, 10], [0, 800, 0, 600, -1, 1], [-5, 5, -2.5, 2.5, 1, 100], [1, 1, 0, 1, 0, 1]])
    return [rng.randrange(2), rng.choice([0.0, 30.0, 90.0, -45.0, 180.0, 12.5]), *axis,
            *[rng.choice([1.0, 2.0, 0.5, -1.0]) for _ in range(3)], *[rng.choice([0.0, 1.5, -3.0]) for _ in range(3)],
            *ortho, *[rng.choice([0.0, 0.25, -0.5, 2.0]) for _ in range(3)],
            *rng.choice([[0, 0, 800, 600], [10, 20, 640, 480], [0, 0, 1, 1]])]


def append_tpers_requests(requests: list[str]) -> None:
    """Append seeded persistences over cameras, then seeded utility requests."""
    rng = random.Random(20261009)
    for index in range(120):
        cam = DEFAULT if index < 10 else camera(rng)
        box = rng.choice(BOXES)
        size = rng.choice([[800, 600], [640, 480], [800, 0]])
        requests.append('sv_tpers ' + numbers(cam + persistence(rng) + size + box))
    for _ in range(60):
        requests.append('sv_tutils ' + numbers(utils_request(rng)))
