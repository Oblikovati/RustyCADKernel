"""TKG3d corpus: the GeomEval elementary surfaces (tools/oracle/tkg3d_probe_eval_surfaces.cxx).

Surface kinds 20 to 24 of g3_surface: ellipsoid, hyperboloid, paraboloid, hyperbolic
paraboloid and circular helicoid. Each surface goes through the generic surface families
(summary and iso refusals, moves and reversals, the Geom_ElementarySurface members, use as
an evaluation representation) and g3_eval_surface (class getters, Coefficients, EvalDN of
orders 4 to 6, the setters with valid and invalid values, the copy and the dumps).
"""
from __future__ import annotations
import math
import random
from tkg3d_corpus import axis, circle, fmt, frame, moves, params, point, trsf, unit
from tkg3d_corpus_surfaces import descriptor, extrusion, revolution

HALF_PI = math.pi / 2
IDENTITY = [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0]


def ellipsoid(f, a, b, c):
    return [20, *f, a, b, c]


def hyperboloid(f, r1, r2, mode):
    return [21, *f, r1, r2, mode]


def paraboloid(f, focal):
    return [22, *f, focal]


def saddle(f, a, b):
    return [23, *f, a, b]


def helicoid(f, pitch):
    return [24, *f, pitch]


def valid_specs(rng: random.Random) -> list[list]:
    """Every class and mode in the identity frame and in random frames, near-degenerate sizes included."""
    return [
        ellipsoid(IDENTITY, 3.0, 2.0, 1.0), ellipsoid(frame(rng), 5.0, 5.0, 5.0), ellipsoid(frame(rng), 1e-7, 2.0, 1e7),
        hyperboloid(IDENTITY, 2.0, 3.0, 0), hyperboloid(IDENTITY, 2.0, 3.0, 1),
        hyperboloid(frame(rng), 0.5, 4.0, 0), hyperboloid(frame(rng), 1e-9, 1.0, 1),
        paraboloid(IDENTITY, 1.0), paraboloid(frame(rng), 2.5), paraboloid(frame(rng), 1e-12),
        saddle(IDENTITY, 2.0, 3.0), saddle(frame(rng), 0.3, 7.0), saddle(frame(rng), 1e8, 1e-8),
        helicoid(IDENTITY, 10.0), helicoid(frame(rng), -5.0), helicoid(frame(rng), 1e-300),
    ]


def invalid_specs(rng: random.Random) -> list[list]:
    """Each constructor refusal: zero, negative and non-positive parameters in native check order."""
    return [
        ellipsoid(frame(rng), 0.0, 1.0, 1.0), ellipsoid(frame(rng), 1.0, -1.0, 1.0), ellipsoid(frame(rng), 1.0, 1.0, 0.0),
        hyperboloid(frame(rng), 0.0, 1.0, 0), hyperboloid(frame(rng), 1.0, -1.0, 1), hyperboloid(frame(rng), -2.0, -3.0, 0),
        paraboloid(frame(rng), 0.0), paraboloid(frame(rng), -1.0),
        saddle(frame(rng), -1.0, 1.0), saddle(frame(rng), 1.0, 0.0), saddle(frame(rng), 0.0, 0.0),
        helicoid(frame(rng), 0.0), helicoid(frame(rng), -0.0),
    ]


def uv_pairs(rng: random.Random) -> list[float]:
    """Parameters inside, on the poles and seams, far outside the usual ranges and huge."""
    pairs = [(0.0, 0.0), (1.0, 0.5), (HALF_PI, HALF_PI), (math.pi, -HALF_PI), (2 * math.pi, -1.0), (-7.0, 8.0),
             (rng.uniform(-5, 5), rng.uniform(-5, 5)), (1e-9, -1e-9), (-0.0, -0.0), (300.0, 20.0)]
    return [c for pair in pairs for c in pair]


def setter_values() -> list[list[float]]:
    """Valid, zero, negative and tiny setter arguments. The third also picks the hyperboloid mode,
    so it stays 0 or 1: the native SetMode stores any integer cast to SheetMode, which the Rust
    enumeration cannot hold."""
    return [[4.0, 0.5, 1.0], [0.0, 2.0, 0.0], [-1.0, -2.0, 1.0], [1e-300, 1e300, 0.0], [2.0, 0.0, 1.0]]


def append_eval_surface_requests(requests: list[str]) -> None:
    """Every request family over the GeomEval elementary surfaces."""
    rng = random.Random(0x6E5A)
    isos = params([0.0, 1.0, -0.5])
    for spec in valid_specs(rng) + invalid_specs(rng):
        requests.append('g3_surface ' + fmt([*spec, *params(uv_pairs(rng)), *isos, *params([0.0, 0.5])]))
        requests.append('g3_surface_moves ' + fmt([*spec, *params(uv_pairs(rng)[:6]), *trsf(rng), *moves(rng)]))
        for values in setter_values():
            requests.append('g3_eval_surface ' + fmt([*spec, *params(uv_pairs(rng)[:8]), *values]))
    append_elementary_members(rng, requests)
    append_evaluation_representations(rng, requests)


def append_elementary_members(rng: random.Random, requests: list[str]) -> None:
    """The inherited Geom_ElementarySurface members: SetLocation, SetAxis (a parallel X raises), SetPosition."""
    for spec in valid_specs(rng):
        f = frame(rng)
        ax = rng.choice([axis(rng), [*f[:3], *f[6:9]]])
        requests.append('g3_elementary ' + fmt([*spec, *params(uv_pairs(rng)[:4]), 1.0, 2.0, *ax, *point(rng), *frame(rng)]))


def append_evaluation_representations(rng: random.Random, requests: list[str]) -> None:
    """Swept surfaces redirected to a GeomEval surface through every descriptor kind."""
    reps = valid_specs(rng)
    bases = [extrusion(unit(rng), circle(frame(rng), 2.0)), revolution(axis(rng), circle(frame(rng), 1.0))]
    for index, rep in enumerate(reps):
        for kind, order in ((0, 0), (1, 2), (2, 3)):
            spec = bases[(index + kind) % len(bases)]
            requests.append('g3_surface_evalrep ' + fmt([*spec, *params(uv_pairs(rng)[:8]), *rep,
                                                         *descriptor(rng, kind, order), *trsf(rng)]))
