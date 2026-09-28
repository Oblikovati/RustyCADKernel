"""TKG3d corpus: offset curves (g3_offset, and offset specs in the generic curve and
evaluation-representation requests of tkg3d_probe_curves.cxx and tkg3d_probe_bspl.cxx)."""
from __future__ import annotations
import random
from tkg3d_corpus import (bezier, bspline, circle, ellipse, fmt, frame, line, moves, offset, params, parabola,
                          point, trimmed, trsf, unit)
from tkg3d_corpus_bspl import bspline_random, descriptor, representation

Z = [0.0, 0.0, 1.0]


def c0_bsplines(rng: random.Random) -> list[list]:
    """C0 B-splines: one with a G1 corner (collinear poles around the double knot), one kinked."""
    knots = [[0.0, 3], [0.5, 2], [1.0, 3]]
    g1 = [[0, 0, 0, 1], [1, 1, 0, 1], [2, 1, 0, 1], [3, 1, 0, 1], [4, 0, 0, 1]]
    kink = [[0, 0, 0, 1], [1, 1, 0, 1], [2, 0, 0, 1], [3, 1, 0, 1], [4, 0, 0, 1]]
    return [bspline(2, knots, g1), bspline(2, knots, kink)]


def offset_specs(rng: random.Random) -> list[list]:
    """Offsets of every basis kind, nested and trimmed offsets, degenerate normals and cusps."""
    circle_z = circle([0.0, 0.0, 0.0, *Z, 1.0, 0.0, 0.0], 2.0)
    g1, kink = c0_bsplines(rng)
    return [
        offset(0.5, Z, circle_z), offset(-3.0, Z, circle_z), offset(1.0, unit(rng), ellipse(frame(rng), 3.0, 1.0)),
        offset(0.7, Z, line([0.0, 0.0, 0.0], [1.0, 0.0, 0.0])), offset(0.7, [1.0, 0.0, 0.0], line(point(rng), [1.0, 0.0, 0.0])),
        offset(0.25, unit(rng), parabola(frame(rng), 1.0)),
        offset(1.5, Z, offset(0.5, [0.0, 1.0, 0.0], circle_z)), offset(-1.0, Z, offset(2.0, Z, circle_z)),
        offset(1.0, Z, offset(-1.0, Z, circle_z)),
        offset(0.5, Z, trimmed(0.5, 2.0, circle_z)), trimmed(0.5, 2.0, offset(0.5, Z, circle_z)),
        offset(0.3, unit(rng), bspline_random(rng, 3, 3, False)), offset(0.3, Z, bspline_random(rng, 3, 2, True)),
        offset(0.4, Z, g1), offset(0.4, Z, kink), offset(0.4, Z, kink, no_check=True),
        offset(0.2, Z, bezier([[0, 0, 0, 1], [1, 1, 0, 1], [0, 0, 0, 1], [2, 0, 0, 1]])),
        offset(0.2, Z, bezier([[0, 0, 0, 1], [0, 0, 0, 1], [1, 1, 0, 1], [2, 0, 0, 1]])),
    ]


def append_offset_requests(requests: list[str]) -> None:
    """Offset curves with their own generator so the other families keep their inputs."""
    rng = random.Random(0x0FF5E7)
    u = [0.0, 0.25, 0.5, 1.0, 1.7, 3.0]
    for spec in offset_specs(rng):
        requests.append('g3_curve ' + fmt(spec + params(u)))
        requests.append('g3_transform ' + fmt(spec + params(u[:3]) + trsf(rng) + moves(rng)))
        if spec[0] != 8:
            continue
        basis = rng.choice([circle(frame(rng), 1.5), *c0_bsplines(rng), offset(0.5, Z, circle(frame(rng), 1.0))])
        direction = rng.choice([unit(rng), Z])
        requests.append('g3_offset ' + fmt(spec + params(u[:3]) + [rng.choice([2.0, -0.5, 0.0]), *direction,
                                                                   rng.randrange(2), *trsf(rng), *basis]))
    for kind in range(3):
        for order in (0, 2, 3):
            owner = offset(0.5, Z, circle([0.0, 0.0, 0.0, *Z, 1.0, 0.0, 0.0], 2.0))
            requests.append('g3_evalrep ' + fmt(owner + params([0.0, 0.5, 1.0, 3.0]) + representation(rng) +
                                                descriptor(rng, kind, order) + trsf(rng)))
