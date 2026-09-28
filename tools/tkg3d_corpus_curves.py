"""TKG3d corpus: curve evaluation, curve moves, lines and conics
(tools/oracle/tkg3d_probe_curves.cxx)."""
from __future__ import annotations
import math
import random
from tkg3d_corpus import axis, circle, fmt, frame, line, moves, params, point, trsf, unit


def curve_specs(rng: random.Random) -> list[list]:
    """Every curve kind with valid and invalid data, as g3_curve specs."""
    return [
        line(point(rng), unit(rng)),
        line([0.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
        circle(frame(rng), 2.5),
        circle(frame(rng), 0.0),
        circle(frame(rng), -1.0),
    ]


def curve_params(rng: random.Random) -> list[float]:
    """Parameters inside, on and outside the usual ranges."""
    return [0.0, 1.0, -2.5, math.pi, 2 * math.pi, rng.uniform(-10, 10), 7.5, 1e-9]


def append_curve_family_requests(rng: random.Random, requests: list[str]) -> None:
    """Curve summaries and moves for every spec, then lines and conics."""
    for spec in curve_specs(rng):
        requests.append('g3_curve ' + fmt([*spec, *params(curve_params(rng))]))
        requests.append('g3_transform ' + fmt([*spec, *params(curve_params(rng)[:3]), *trsf(rng), *moves(rng)]))
    for _ in range(6):
        requests.append('g3_line ' + fmt([*axis(rng), *unit(rng), *point(rng), *axis(rng), *params(curve_params(rng))]))
    append_conic_requests(rng, requests, [circle(frame(rng), r) for r in (1.0, 3.5, 0.0)], [1])


def append_conic_requests(rng: random.Random, requests: list[str], specs: list[list], build_kinds: list[int]) -> None:
    """g3_conic for each conic spec with valid and invalid setter values, and g3_conic_build."""
    for spec in specs:
        for value in (2.0, 0.0, -1.0, 0.5):
            f = frame(rng)
            a = rng.choice([axis(rng), [*f[:3], *f[6:9]]])
            requests.append('g3_conic ' + fmt([*spec, *params(curve_params(rng)[:4]), value, *a, *point(rng), *frame(rng)]))
    for kind in build_kinds:
        for a, b in ((3.0, 2.0), (2.0, 3.0), (-1.0, 1.0), (0.0, 0.0)):
            requests.append('g3_conic_build ' + fmt([kind, *frame(rng), a, b, *axis(rng), *point(rng)]))
