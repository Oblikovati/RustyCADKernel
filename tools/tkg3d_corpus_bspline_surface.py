"""TKG3d corpus: B-spline surfaces (tools/oracle/tkg3d_probe_bspline_surface.cxx), and
B-spline specifications for the generic surface requests of tkg3d_probe_surfaces.cxx.

Surface kind 8 follows g3_surface: rational uperiodic vperiodic udeg vdeg nuk (k m)*
nvk (k m)* nu nv (x y z w)*(nu*nv), poles read row by row (U major).
"""
from __future__ import annotations
import random
from dataclasses import dataclass
from tkg3d_corpus import fmt, frame, moves, params, point, trsf
from tkg3d_corpus_surfaces import cylinder, descriptor, plane


@dataclass
class Spec:
    """A B-spline surface specification with the data the edit requests need."""

    values: list
    u_range: tuple
    v_range: tuple
    nu: int
    nv: int
    nuk: int
    nvk: int


def nb_poles(degree: int, periodic: bool, knots: list) -> int:
    mults = [m for _, m in knots]
    return sum(mults[:-1]) if periodic else sum(mults) - degree - 1


def grid(rng: random.Random, nu: int, nv: int, weights) -> list:
    """A wavy pole grid; `weights(i, j)` gives each weight."""
    poles = []
    for i in range(nu):
        for j in range(nv):
            poles.append([float(i) + rng.uniform(-0.2, 0.2), float(j) + rng.uniform(-0.2, 0.2),
                          rng.uniform(-1.0, 1.0), weights(i, j)])
    return poles


def bspline(rng: random.Random, degrees, u_knots, v_knots, periodic=(False, False), weights=None) -> Spec:
    nu = nb_poles(degrees[0], periodic[0], u_knots)
    nv = nb_poles(degrees[1], periodic[1], v_knots)
    rational = weights is not None
    poles = grid(rng, nu, nv, weights or (lambda i, j: 1.0))
    values = [8, int(rational), int(periodic[0]), int(periodic[1]), degrees[0], degrees[1],
              len(u_knots), *[c for k in u_knots for c in k], len(v_knots), *[c for k in v_knots for c in k],
              nu, nv, *[c for p in poles for c in p]]
    return Spec(values, (u_knots[0][0], u_knots[-1][0]), (v_knots[0][0], v_knots[-1][0]), nu, nv,
                len(u_knots), len(v_knots))


def valid_specs(rng: random.Random) -> list[Spec]:
    """Clamped, non-uniform, rational, periodic, uniform, C0 and linear B-spline surfaces."""
    varied = lambda i, j: 0.5 + 0.3 * i + 0.2 * j  # noqa: E731
    u_only = lambda i, j: 1.0 + 0.5 * j  # noqa: E731
    per = [[float(k), 1] for k in range(6)]
    return [
        bspline(rng, (3, 3), [[0.0, 4], [1.0, 4]], [[0.0, 4], [1.0, 4]]),
        bspline(rng, (3, 2), [[0.0, 4], [0.3, 1], [0.7, 2], [1.0, 4]], [[0.0, 3], [0.5, 1], [1.0, 3]]),
        bspline(rng, (3, 2), [[0.0, 4], [0.3, 1], [0.7, 2], [1.0, 4]], [[0.0, 3], [0.5, 1], [1.0, 3]], weights=varied),
        bspline(rng, (2, 2), [[0.0, 3], [0.5, 2], [1.0, 3]], [[0.0, 3], [0.4, 1], [1.0, 3]], weights=u_only),
        bspline(rng, (3, 2), per, [[0.0, 3], [1.0, 3]], periodic=(True, False)),
        bspline(rng, (2, 3), [[-1.0, 3], [2.0, 3]], per, periodic=(False, True), weights=varied),
        bspline(rng, (2, 2), per, [[0.0, 1], [0.5, 1], [1.5, 1], [2.0, 1], [3.0, 1]], periodic=(True, True)),
        bspline(rng, (2, 1), [[float(k), 1] for k in range(7)], [[0.0, 2], [1.0, 2]]),
        bspline(rng, (1, 1), [[0.0, 2], [0.5, 1], [1.0, 2]], [[0.0, 2], [1.0, 2]]),
        bspline(rng, (2, 2), [[0.0, 3], [0.5, 2], [1.0, 3]], [[0.0, 3], [0.25, 2], [0.75, 1], [1.0, 3]]),
    ]


def invalid_specs(rng: random.Random) -> list[list]:
    """Degrees, pole counts, knot spacing and weights that the constructors refuse."""
    good = valid_specs(rng)[1].values
    bad = []
    zero_degree = list(good)
    zero_degree[4] = 0
    bad.append(zero_degree)
    too_close = bspline(rng, (1, 1), [[0.0, 2], [1e-300, 1], [1.0, 2]], [[0.0, 2], [1.0, 2]]).values
    bad.append(too_close)
    mismatch = bspline(rng, (2, 2), [[0.0, 3], [1.0, 3]], [[0.0, 3], [1.0, 3]]).values
    mismatch[4] = 3
    bad.append(mismatch)
    zero_weight = bspline(rng, (1, 1), [[0.0, 2], [1.0, 2]], [[0.0, 2], [1.0, 2]], weights=lambda i, j: 0.0 if i == j == 1 else 1.0).values
    bad.append(zero_weight)
    return bad


def uv_list(rng: random.Random, spec: Spec) -> list[float]:
    """Pairs inside, on the bounds, on knots and outside."""
    (u0, u1), (v0, v1) = spec.u_range, spec.v_range
    pairs = [(u0, v0), (u1, v1), ((u0 + u1) / 2, (v0 + v1) / 3), (rng.uniform(u0, u1), rng.uniform(v0, v1)),
             (u0 - 0.7, v1 + 0.4), (u1 + 3.3, v0 - 2.1), (0.3, 0.5)]
    return [c for pair in pairs for c in pair]


def edit(rng: random.Random, spec: Spec) -> list:
    """One random edit request of g3_bss_edit, valid or not."""
    op = rng.choice([0, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27,
                     28, 29, 30, 31, 32, 34, 36, 37, 38, 39, 40, 42, 43, 44, 46])
    u = op % 2 == 0
    lo, hi = spec.u_range if u else spec.v_range
    nk = spec.nuk if u else spec.nvk
    across = spec.nv if u else spec.nu
    mid = lo + (hi - lo) * rng.uniform(0.1, 0.9)
    index = rng.randrange(0, nk + 2)
    args = {
        4: [index], 5: [index], 10: [rng.randrange(1, 5), rng.randrange(1, 5)],
        12: [1, 2, mid, lo + (hi - lo) * 0.5, 1, 2, rng.randrange(1, 3), rng.randrange(0, 4), rng.choice([0.0, 1e-3]), rng.randrange(2)],
        14: [index, rng.randrange(-1, 3), rng.choice([1e-7, 10.0])],
        # Natively the knot ranges of 18 and 20 view the knot array without a copy or a
        # bound check, so a range past the knots (earlier edits may remove some) is
        # undefined behaviour: keep them inside the two knots every surface has.
        16: [index, rng.randrange(1, 4)], 18: [1, 2, rng.randrange(1, 4)],
        20: [1, 2, rng.randrange(0, 2)],
        22: [rng.choice([mid, lo, hi + 1.0]), rng.randrange(1, 4), rng.choice([0.0, 1e-6]), rng.randrange(2)],
        24: [*sorted([rng.uniform(*spec.u_range), rng.uniform(*spec.u_range)]),
             *sorted([rng.uniform(*spec.v_range), rng.uniform(*spec.v_range)]), 1e-9, rng.choice([1e-9, 0.5])],
        26: [index, rng.choice([mid, lo - 1.0])],
        28: [rng.randrange(1, 3), 2, mid, mid + 1e-3],
        30: [index, mid, rng.randrange(1, 3)],
        32: [rng.randrange(0, spec.nu + 2), rng.randrange(1, spec.nv + 1), *point(rng, 3.0)],
        34: [rng.randrange(1, spec.nu + 1), rng.randrange(1, spec.nv + 1), *point(rng, 3.0), rng.choice([2.0, 1.0, 0.0])],
        36: [rng.randrange(0, (spec.nu if u else spec.nv) + 2), 1, 2, *point(rng), *point(rng)],
        38: [rng.randrange(1, (spec.nu if u else spec.nv) + 1), 1, 2, *point(rng), *point(rng), 1, 2, 1.5, rng.choice([2.0, 0.0])],
        40: [rng.randrange(0, spec.nu + 2), rng.randrange(1, spec.nv + 1), rng.choice([2.0, 1.0, -1.0])],
        42: [rng.randrange(1, (spec.nu if u else spec.nv) + 1), 1, min(3, across), *[rng.choice([1.0, 2.5]) for _ in range(min(3, across))]],
        44: [rng.uniform(*spec.u_range), rng.uniform(*spec.v_range), *point(rng, 3.0), 1, spec.nu, 1, spec.nv],
        46: trsf(rng),
    }
    key = op if op in args else op - 1
    return [op, *args.get(key, [])]


def bss_request(rng: random.Random, spec: Spec) -> list:
    i, j = rng.randrange(0, spec.nu + 2), rng.randrange(0, spec.nv + 2)
    # Knot ranges whose located flat index stays inside the flat knots: natively FlatIndex
    # and BSplSLib read past the arrays for wider ranges (undefined behaviour).
    knots = rng.choice([[1, 2, 1, 2], [2, 3, 1, 2], [1, 1, 2, 2]])
    if spec.nuk == 7:
        # The uniform knots are their own flat knots: start past the first Degree ones.
        knots = [3, 4, 1, 2]
    return [*spec.values, i, j, rng.choice([1e-7, 1.0]), *params(uv_list(rng, spec)), *knots, *params([0.0, 1e-9, 0.1])]


def append_bspline_surface_requests(requests: list[str]) -> None:
    """Every B-spline surface request family, with its own seed."""
    rng = random.Random(0xB55F)
    specs = valid_specs(rng)
    for spec in specs:
        isos = params([spec.u_range[0], 0.3, spec.v_range[1], -0.5])
        requests.append('g3_surface ' + fmt([*spec.values, *params(uv_list(rng, spec)), *isos, *params([0.0, 0.5, 1.0])]))
        requests.append('g3_surface_moves ' + fmt([*spec.values, *params(uv_list(rng, spec)[:6]), *trsf(rng), *moves(rng)]))
        for _ in range(2):
            requests.append('g3_bspline_surface ' + fmt(bss_request(rng, spec)))
        for count in (1, 2, 3, 3, 4):
            edits = [c for _ in range(count) for c in edit(rng, spec)]
            requests.append('g3_bspline_surface_edit ' + fmt([*spec.values, *params(uv_list(rng, spec)[:6]), count, *edits]))
    for values in invalid_specs(rng):
        requests.append('g3_surface ' + fmt([*values, *params([0.0, 0.0]), *params([]), *params([])]))
    reps = [cylinder(frame(rng), 1.0), plane(frame(rng)), specs[2].values]
    for index, spec in enumerate(specs[:6]):
        for kind, order in ((0, 0), (1, 1), (1, 3), (2, 1), (2, 3)):
            rep = reps[(index + kind + order) % len(reps)]
            requests.append('g3_surface_evalrep ' + fmt([*spec.values, *params(uv_list(rng, spec)[:8]), *rep,
                                                         *descriptor(rng, kind, order), *trsf(rng)]))
