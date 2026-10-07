"""Deterministic requests for Graphic3d_CullingTool (tools/oracle/tkservice_probe_culling.cxx).

`sv_culling` builds culling tools over random cameras (with and without a model translation),
viewports, resolution ratios, culling distances and sizes, and compares the culling context,
the model-space eye and direction, and the frustum, distance and size tests of boxes inside,
across and outside the view volume, with their "fully inside" flags.
"""
from __future__ import annotations
import random

from tkservice_corpus_camera import DEFAULT, camera, numbers

BOXES = [[-1, -1, -1, 1, 1, 1], [-10, -10, -10, 10, 10, 10], [1000, 1000, 1000, 1001, 1001, 1001],
         [-5000, -5000, -5000, 5000, 5000, 5000], [0, 0, 0, 0, 0, 0], [0.1, 0.1, 0.1, 0.11, 0.11, 0.11],
         [-100, 0, 0, -50, 1, 1], [200, -30, 40, 260, 30, 90]]


def append_culling_requests(requests: list[str]) -> None:
    """Append seeded culling tools over cameras with box lists."""
    rng = random.Random(20261012)
    for index in range(60):
        cam = DEFAULT if index < 4 else camera(rng)
        shift = rng.choice([[0, 0, 0], [0, 0, 0], [10, -20, 5]])
        viewport = rng.choice([[800, 600], [640, 480], [0, -5]])
        ratio, distance, size = rng.choice([1.0, 2.0]), rng.choice([0.0, 500.0, 1e100, -1.0]), rng.choice([0.0, 2.0, 50.0, 1e100])
        boxes = rng.sample(BOXES, rng.randrange(1, len(BOXES)))
        requests.append('sv_culling ' + numbers(cam + shift + viewport + [ratio, distance, size, len(boxes)]
                                                + [v for b in boxes for v in b]))
