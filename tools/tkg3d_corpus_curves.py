"""TKG3d corpus: curve evaluation, curve moves, lines, conics and trimmed curves
(tools/oracle/tkg3d_probe_curves.cxx)."""
from __future__ import annotations
import math
import sys
import random
from tkg3d_corpus import (axis, circle, ellipse, fmt, frame, hyperbola, line, moves, parabola, params, point,
                          trimmed, trsf, unit)


def conic_specs(rng: random.Random) -> list[list]:
    """Every conic with valid, degenerate and invalid radii."""
    return [
        circle(frame(rng), 2.5), circle(frame(rng), 0.0), circle(frame(rng), -1.0),
        ellipse(frame(rng), 3.0, 2.0), ellipse(frame(rng), 2.0, 2.0), ellipse(frame(rng), 0.0, 0.0),
        ellipse(frame(rng), 2.0, 3.0), ellipse(frame(rng), 1.0, -0.5),
        hyperbola(frame(rng), 3.0, 2.0), hyperbola(frame(rng), 0.0, 1.5), hyperbola(frame(rng), 2.0, 0.0),
        hyperbola(frame(rng), -1.0, 2.0),
        parabola(frame(rng), 1.5), parabola(frame(rng), 0.0), parabola(frame(rng), -2.0),
    ]


def trimmed_specs(rng: random.Random) -> list[list]:
    """Trimmed conics and lines: senses, periodic adjustment, reversed and out of range bounds."""
    specs = []
    for basis in (circle(frame(rng), 2.0), ellipse(frame(rng), 3.0, 1.0)):
        for u1, u2, sense, adjust in ((0.5, 2.0, 1, 1), (2.0, 0.5, 1, 1), (-1.0, 8.0, 0, 1), (7.0, 9.0, 1, 0),
                                     (0.0, 2 * math.pi, 1, 1), (1.0, 1.0, 1, 1), (0.25, 0.25 + 4 * math.pi, 0, 0)):
            specs.append(trimmed(u1, u2, basis, bool(sense), bool(adjust)))
    for basis in (line(point(rng), unit(rng)), hyperbola(frame(rng), 2.0, 1.0), parabola(frame(rng), 0.5)):
        for u1, u2, sense in ((-1.0, 2.0, 1), (3.0, -2.0, 1), (3.0, -2.0, 0), (1e101, 2e101, 1)):
            specs.append(trimmed(u1, u2, basis, bool(sense)))
    specs.append(trimmed(0.5, 1.5, trimmed(0.0, 3.0, circle(frame(rng), 1.0))))
    return specs


def curve_specs(rng: random.Random) -> list[list]:
    """Every curve kind of this probe file, as g3_curve specs."""
    return [line(point(rng), unit(rng)), line([0.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
            *conic_specs(rng), *trimmed_specs(rng)]


def curve_params(rng: random.Random) -> list[float]:
    """Parameters inside, on and outside the usual ranges."""
    return [0.0, 1.0, -2.5, math.pi, 2 * math.pi, rng.uniform(-10, 10), 7.5, 1e-9]


def append_curve_family_requests(rng: random.Random, requests: list[str]) -> None:
    """Curve summaries and moves for every spec, then lines, conics and trimmed curves."""
    for spec in curve_specs(rng):
        requests.append('g3_curve ' + fmt([*spec, *params(curve_params(rng))]))
        requests.append('g3_transform ' + fmt([*spec, *params(curve_params(rng)[:3]), *trsf(rng), *moves(rng)]))
    for _ in range(6):
        requests.append('g3_line ' + fmt([*axis(rng), *unit(rng), *point(rng), *axis(rng), *params(curve_params(rng))]))
    valid_conics = [spec for spec in conic_specs(rng) if spec[-1] >= 0.0 and spec[-2] >= 0.0]
    append_conic_requests(rng, requests, valid_conics, [1, 2, 3, 4, 5])
    append_trimmed_requests(rng, requests)
    append_offset_utils_requests(rng, requests)


def append_conic_requests(rng: random.Random, requests: list[str], specs: list[list], build_kinds: list[int]) -> None:
    """g3_conic for each conic spec with valid and invalid setter values, and g3_conic_build.

    Half of the SetAxis inputs use the X direction of the conic's own frame, so the
    gp_Ax2::SetAxis parallel failure is exercised.
    """
    for spec in specs:
        own = spec[1:10]
        for value in (2.0, 0.0, -1.0, 0.5, 5.0):
            a = rng.choice([axis(rng), [*point(rng), *own[6:9]]])
            requests.append('g3_conic ' + fmt([*spec, *params(curve_params(rng)[:4]), value, *a, *point(rng), *frame(rng)]))
    for kind in build_kinds:
        for a, b in ((3.0, 2.0), (2.0, 3.0), (-1.0, 1.0), (0.0, 0.0), (1.0, -1.0)):
            f = frame(rng)
            directrix = rng.choice([axis(rng), [*f[:3], *f[3:6]]])
            focus = rng.choice([point(rng), f[:3]])
            requests.append('g3_conic_build ' + fmt([kind, *f, a, b, *directrix, *focus]))


def append_trimmed_requests(rng: random.Random, requests: list[str]) -> None:
    """g3_trimmed: state, copy, SetTrim (valid, equal, out of range), Reverse, Transform."""
    for spec in trimmed_specs(rng):
        for u1, u2, sense, adjust in ((0.1, 1.2, 1, 1), (1.2, 0.1, 0, 1), (5.0, 5.0, 1, 1), (-50.0, 50.0, 1, 0)):
            requests.append('g3_trimmed ' + fmt([*spec, *params(curve_params(rng)[:3]), u1, u2, sense, adjust,
                                                 *trsf(rng)]))


def append_offset_utils_requests(rng: random.Random, requests: list[str]) -> None:
    """Geom_OffsetCurveUtils::CalculateD0..D3 with D1 magnitudes that reach every
    stable and unstable branch (R^n compared with the tolerance) and null normals."""
    tiny = sys.float_info.min
    for scale in (1.0, 1e-3, 1e-40, 1e-50, 1e-70, 1e-105, 1e-160, 0.0):
        for tolerance in (tiny, 1e-12):
            d1 = [c * scale for c in point(rng, 1.0)]
            direction = rng.choice([unit(rng), [0.0, 0.0, 1.0], [c * 1e-60 for c in unit(rng)]])
            requests.append('g3_offset_utils ' + fmt([*point(rng), *d1, *point(rng, 2.0), *point(rng, 3.0),
                                                      *point(rng, 4.0), *direction, rng.choice([2.0, -0.5, 0.0]),
                                                      rng.randrange(2), tolerance]))
