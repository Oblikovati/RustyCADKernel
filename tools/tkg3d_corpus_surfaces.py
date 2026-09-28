"""TKG3d corpus: surface evaluation, surface moves, elementary and swept surfaces
(tools/oracle/tkg3d_probe_surfaces.cxx). Surface specifications follow g3_surface."""
from __future__ import annotations
import math
import random
from tkg3d_corpus import (axis, circle, ellipse, fmt, frame, line, moves, parabola, params, point, trimmed,
                          trsf, unit)

HALF_PI = math.pi / 2


def plane(f):
    return [0, *f]


def cylinder(f, radius):
    return [1, *f, radius]


def cone(f, angle, radius):
    return [2, *f, angle, radius]


def sphere(f, radius):
    return [3, *f, radius]


def torus(f, major, minor):
    return [4, *f, major, minor]


def extrusion(direction, basis):
    return [5, *direction, *basis]


def revolution(ax, basis):
    return [6, *ax, *basis]


def elementary_specs(rng: random.Random) -> list[list]:
    """Every elementary surface with valid, degenerate and invalid data."""
    return [
        plane(frame(rng)), plane([0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0]),
        cylinder(frame(rng), 2.0), cylinder(frame(rng), 0.0), cylinder(frame(rng), -1.0),
        cone(frame(rng), 0.3, 2.0), cone(frame(rng), -0.4, 1.0), cone(frame(rng), 0.3, 0.0),
        cone(frame(rng), 0.0, 1.0), cone(frame(rng), HALF_PI, 1.0), cone(frame(rng), 0.3, -1.0),
        sphere(frame(rng), 2.0), sphere(frame(rng), 0.0), sphere(frame(rng), -1.0),
        torus(frame(rng), 5.0, 1.0), torus(frame(rng), 1.0, 1.0), torus(frame(rng), 1.0, 3.0),
        torus(frame(rng), 0.0, 0.0), torus(frame(rng), -1.0, 1.0),
    ]


def swept_specs(rng: random.Random) -> list[list]:
    """Extrusions and revolutions of lines, conics and trimmed curves, degenerate ones included."""
    z = [0.0, 0.0, 1.0]
    return [
        extrusion(unit(rng), line(point(rng), unit(rng))),
        extrusion(unit(rng), circle(frame(rng), 2.0)),
        extrusion(z, ellipse(frame(rng), 3.0, 1.0)),
        extrusion(unit(rng), parabola(frame(rng), 0.5)),
        extrusion(unit(rng), trimmed(0.5, 2.0, circle(frame(rng), 1.5))),
        extrusion(z, line([0.0, 0.0, 0.0], z)),
        revolution([0.0, 0.0, 0.0, *z], line([2.0, 0.0, 0.0], z)),
        revolution([0.0, 0.0, 0.0, *z], line([0.0, 0.0, 0.0], [1.0, 0.0, 0.0])),
        revolution(axis(rng), circle(frame(rng), 1.0)),
        revolution([0.0, 0.0, 0.0, *z], circle([3.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0], 1.0)),
        revolution([0.0, 0.0, 0.0, *z], trimmed(-1.0, 1.0, circle([0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0], 2.0))),
        revolution(axis(rng), ellipse(frame(rng), 2.0, 1.0)),
    ]


def uv_pairs(rng: random.Random) -> list[float]:
    """(u, v) pairs inside, on the bounds and poles, and outside the usual ranges."""
    pairs = [(0.0, 0.0), (1.0, 0.5), (HALF_PI, HALF_PI), (math.pi, -HALF_PI), (2 * math.pi, -1.0), (7.0, 8.0),
             (rng.uniform(-5, 5), rng.uniform(-5, 5)), (1e-9, 1e-9)]
    return [c for pair in pairs for c in pair]


def append_surface_family_requests(rng: random.Random, requests: list[str]) -> None:
    """Every request family of tools/oracle/tkg3d_probe_surfaces.cxx."""
    specs = elementary_specs(rng) + swept_specs(rng)
    isos = params([0.0, 1.0, HALF_PI, -0.5])
    for spec in specs:
        requests.append('g3_surface ' + fmt([*spec, *params(uv_pairs(rng)), *isos, *params([0.0, 0.5, 2.0])]))
        requests.append('g3_surface_moves ' + fmt([*spec, *params(uv_pairs(rng)[:6]), *trsf(rng), *moves(rng)]))
    append_elementary_requests(rng, requests)
    append_swept_requests(rng, requests)


def append_elementary_requests(rng: random.Random, requests: list[str]) -> None:
    """g3_elementary with valid and invalid setter values, then g3_elementary_build."""
    for spec in elementary_specs(rng):
        for a, b in ((3.0, 0.5), (0.0, 0.0), (-1.0, HALF_PI), (0.5, 7.0), (6.0, 5.9999999999999)):
            f = frame(rng)
            ax = rng.choice([axis(rng), [*f[:3], *f[6:9]]])
            requests.append('g3_elementary ' + fmt([*spec, *params(uv_pairs(rng)[:4]), a, b, *ax, *point(rng), *frame(rng)]))
    for kind in range(7):
        for a, b, coefficients in ((2.0, 0.3, [1.0, 2.0, 3.0, 4.0]), (-1.0, 0.0, [0.0, 0.0, 0.0, 1.0]),
                                   (1.0, 3.0, [0.0, 0.0, 2.0, -5.0]), (0.0, -0.4, [1e-300, 0.0, 0.0, 1.0])):
            requests.append('g3_elementary_build ' + fmt([kind, *frame(rng), a, b, *point(rng), *unit(rng), *coefficients]))


def descriptor(rng: random.Random, kind: int, order: int):
    """A g3_surface_desc spec: full, bounded or mapped with an optional domain."""
    if kind == 0:
        return [0]
    if kind == 1:
        return [1, order]
    domain = [rng.choice([0, 1]), -1.0, 2.0, -3.0, 3.0]
    scale = rng.choice([[1.0, 0.0, 1.0, 0.0, 0], [2.0, 0.5, -1.0, 0.25, 0], [0.5, 0.0, 3.0, 1.0, 1], [1e-9, 0.0, 1.0, 0.0, 0]])
    return [2, order, *domain, *scale]


def append_swept_requests(rng: random.Random, requests: list[str]) -> None:
    """g3_swept setters on every swept spec and g3_surface_evalrep on extrusions and revolutions."""
    for spec in swept_specs(rng):
        basis = rng.choice([circle(frame(rng), 1.0), line(point(rng), unit(rng)), ellipse(frame(rng), 2.0, 0.5)])
        requests.append('g3_swept ' + fmt([*spec, *params(uv_pairs(rng)[:6]), *params([0.0, 1.0, 2.5]),
                                           *unit(rng), *axis(rng), *point(rng), *basis]))
    reps = [cylinder(frame(rng), 1.0), plane(frame(rng)), extrusion(unit(rng), circle(frame(rng), 3.0))]
    for index, spec in enumerate(swept_specs(rng)):
        for kind, order in ((0, 0), (1, 0), (1, 2), (1, 3), (2, 1), (2, 3)):
            rep = reps[(index + kind + order) % len(reps)]
            requests.append('g3_surface_evalrep ' + fmt([*spec, *params(uv_pairs(rng)[:8]), *rep,
                                                         *descriptor(rng, kind, order), *trsf(rng)]))
