"""TKG3d corpus: rectangular trimmed surfaces and offset surfaces
(tools/oracle/tkg3d_probe_offset_surface.cxx), and both kinds in the generic surface
and evaluation-representation requests of tkg3d_probe_surfaces.cxx."""
from __future__ import annotations
import math
import random
from tkg3d_corpus import circle, fmt, frame, line, moves, params, trsf, unit
from tkg3d_corpus_bezier_surface import bezier_surface
from tkg3d_corpus_bspline_surface import bspline
from tkg3d_corpus_surfaces import cone, cylinder, descriptor, extrusion, plane, revolution, sphere, torus

HALF_PI = math.pi / 2
Z = [0.0, 0.0, 1.0]
ORIGIN_Z = [0.0, 0.0, 0.0, *Z, 1.0, 0.0, 0.0]


def trim(u1, u2, v1, v2, basis, u_sense=True, v_sense=True):
    return [9, u1, u2, v1, v2, int(u_sense), int(v_sense), *basis]


def trim_one(p1, p2, u_trim, basis, sense=True):
    return [10, p1, p2, int(u_trim), int(sense), *basis]


def offset(distance, basis, no_check=False):
    return [11, distance, int(no_check), *basis]


def collapsed_bspline(rng: random.Random) -> list:
    """A bicubic B-spline whose V = 0 row collapses to one point (a quasi-punctual boundary)."""
    spec = bspline(rng, (3, 3), [[0.0, 4], [0.5, 1], [1.0, 4]], [[0.0, 4], [1.0, 4]])
    values = list(spec.values)
    start = 7 + 2 * spec.nuk + 1 + 2 * spec.nvk + 2  # the first pole, after nu and nv
    for i in range(spec.nu):
        pole = start + (i * spec.nv) * 4
        values[pole:pole + 3] = [0.0, 0.0, 0.0]
    return values


def collapsed_bezier() -> list:
    """A bi-cubic Bezier patch whose U = 0 column collapses (the apex of a cone-like patch)."""
    net = []
    for i in range(4):
        for j in range(4):
            r = i / 3.0
            angle = j * math.pi / 6
            net.append([r * math.cos(angle), r * math.sin(angle), 1.0 - r, 1.0])
    return bezier_surface(4, 4, net)


def c0_bspline(rng: random.Random) -> list:
    """A B-spline with a C0 knot in U (kinked unless the poles line up)."""
    return bspline(rng, (2, 2), [[0.0, 3], [0.5, 2], [1.0, 3]], [[0.0, 3], [1.0, 3]]).values


def basis_specs(rng: random.Random) -> list[list]:
    """Every basis kind of an offset surface, degenerate ones included."""
    smooth = bspline(rng, (3, 2), [[0.0, 4], [0.3, 1], [1.0, 4]], [[0.0, 3], [0.5, 1], [1.0, 3]]).values
    return [
        plane(frame(rng)), cylinder(frame(rng), 2.0), cylinder(ORIGIN_Z, 0.5), cone(frame(rng), 0.4, 1.0),
        cone(ORIGIN_Z, -0.3, 0.2), sphere(frame(rng), 3.0), sphere(ORIGIN_Z, 0.5), torus(frame(rng), 4.0, 1.0),
        torus(ORIGIN_Z, 2.0, 0.5), extrusion(unit(rng), circle(frame(rng), 1.5)),
        extrusion(Z, line([0.0, 0.0, 0.0], [1.0, 0.0, 0.0])),
        revolution([0.0, 0.0, 0.0, *Z], line([2.0, 0.0, 0.0], [0.0, 0.5, 1.0])),
        smooth, collapsed_bspline(rng), collapsed_bezier(), c0_bspline(rng),
        trim(0.2, 1.4, -1.0, 2.0, cylinder(frame(rng), 1.0)),
    ]


def trimmed_specs(rng: random.Random) -> list[list]:
    """Two- and one-direction trims of periodic, bounded and offset bases, reversed senses included."""
    cyl, sph = cylinder(frame(rng), 1.5), sphere(frame(rng), 2.0)
    smooth = bspline(rng, (2, 2), [[0.0, 3], [0.5, 1], [1.0, 3]], [[0.0, 3], [1.0, 3]]).values
    return [
        trim(0.5, 2.0, -1.0, 3.0, cyl), trim(2.0, 0.5, 3.0, -1.0, cyl, False, True),
        trim(-1.0, 8.0, -0.5, 0.5, sph), trim(0.1, 0.9, 0.2, 0.8, smooth), trim(0.9, 0.1, 0.2, 0.8, smooth, True, False),
        trim_one(0.25, 1.5, True, cyl), trim_one(-2.0, 2.0, False, cyl, False), trim_one(0.2, 0.7, False, smooth),
        trim_one(0.1, 0.4, True, trim(0.0, 1.0, 0.0, 0.5, smooth)), trim_one(0.1, 0.4, False, trim_one(0.0, 1.0, True, smooth)),
        trim(0.0, 1.0, 0.0, 1.0, offset(0.3, plane(frame(rng)))), trim_one(0.5, 1.0, True, offset(0.2, cyl)),
        trim(0.0, 0.0, 0.0, 1.0, cyl), trim(-5.0, 5.0, 0.0, 1.0, smooth),
    ]


def offset_specs(rng: random.Random) -> list[list]:
    """Offsets of every basis with both signs, nested and trimmed offsets, unchecked C0 bases."""
    specs = []
    for basis in basis_specs(rng):
        for distance in (0.75, -0.5):
            specs.append(offset(distance, basis))
    specs += [offset(0.3, c0_bspline(rng), True), offset(-3.0, cylinder(ORIGIN_Z, 1.0)), offset(0.5, sphere(ORIGIN_Z, 0.5)),
              offset(0.0, sphere(frame(rng), 1.0)), offset(0.5, offset(0.25, cylinder(frame(rng), 1.0))),
              offset(-2.5, torus(ORIGIN_Z, 2.0, 0.5)), offset(0.2, cone(ORIGIN_Z, 0.4, 0.0))]
    return specs


def uv_pairs() -> list[float]:
    pairs = [(0.0, 0.0), (0.5, 0.5), (1.0, 0.25), (HALF_PI, 1.0), (0.2, 1.0), (math.pi, -0.5), (0.0, 0.7)]
    return [c for pair in pairs for c in pair]


def append_offset_surface_requests(requests: list[str]) -> None:
    """Trimmed and offset surfaces with their own generator."""
    rng = random.Random(0x0FF5)
    isos = params([0.0, 0.5, 1.0])
    for spec in trimmed_specs(rng) + offset_specs(rng):
        requests.append('g3_surface ' + fmt([*spec, *params(uv_pairs()), *isos, *params([0.0, 0.5])]))
        requests.append('g3_surface_moves ' + fmt([*spec, *params(uv_pairs()[:6]), *trsf(rng), *moves(rng)]))
    for spec in trimmed_specs(rng):
        if spec[0] not in (9, 10):
            continue
        requests.append('g3_rect_trim ' + fmt([*spec, *params(uv_pairs()), *isos, *params([0.1, 0.6]),
                                               0.2, 0.9, -0.5, 0.5, 1, 0, 0.1, 0.8, rng.randrange(2), rng.randrange(2),
                                               *trsf(rng)]))
    for spec in offset_specs(rng):
        if spec[0] != 11:
            continue
        basis = rng.choice(basis_specs(rng))
        requests.append('g3_offset_surface ' + fmt([*spec, *params(uv_pairs()), *isos, *params([0.1, 0.6]),
                                                    rng.choice([1.25, -0.4, 0.0]), rng.randrange(2), *trsf(rng), *basis]))
    reps = [cylinder(frame(rng), 1.0), plane(frame(rng))]
    for index, spec in enumerate(offset_specs(rng)[:8]):
        for kind, order in ((0, 0), (1, 2), (2, 3)):
            requests.append('g3_surface_evalrep ' + fmt([*spec, *params(uv_pairs()[:6]), *reps[index % 2],
                                                         *descriptor(rng, kind, order), *trsf(rng)]))
