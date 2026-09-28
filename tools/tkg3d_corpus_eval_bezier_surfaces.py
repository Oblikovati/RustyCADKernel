"""TKG3d corpus: the GeomEval Bezier surfaces (tools/oracle/tkg3d_probe_eval_bezier_surfaces.cxx).

Specifications are g3_surface kinds 30 (T-Bezier) and 31 (AHT-Bezier), documented in the
probe file. The family has its own seed so it never perturbs the other TKG3d requests.
"""
from __future__ import annotations
import math
import random
from tkg3d_corpus import fmt, params, trsf


def tbezier(net, nu, nv, alpha_u, alpha_v, weights=None, lower=(1, 1), weight_shape=None):
    """Kind 30 from a row-major list of [x, y, z]; weights row-major with their own shape."""
    # A non-rational spec still reads a 1x1 weight grid: the probe never builds an empty one.
    stored = weights or [1.0]
    wu, wv = weight_shape or ((nu, nv) if weights else (1, 1))
    return [30, int(weights is not None), *lower, nu, nv, *[c for p in net for c in p],
            wu, wv, *stored, alpha_u, alpha_v]


def aht(net, nu, nv, degrees, alphas, betas, weights=None, lower=(1, 1), weight_shape=None):
    """Kind 31 with degrees (U, V), alphas (U, V) and betas (U, V)."""
    # A non-rational spec still reads a 1x1 weight grid: the probe never builds an empty one.
    stored = weights or [1.0]
    wu, wv = weight_shape or ((nu, nv) if weights else (1, 1))
    return [31, int(weights is not None), *lower, *degrees, *alphas, *betas, nu, nv,
            *[c for p in net for c in p], wu, wv, *stored]


def net(rng: random.Random, nu, nv):
    return [[i + rng.uniform(-0.4, 0.4), j + rng.uniform(-0.4, 0.4), rng.uniform(-1.5, 1.5)]
            for i in range(nu) for j in range(nv)]


def positive(rng: random.Random, count):
    return [rng.uniform(0.3, 3.0) for _ in range(count)]


def dimension(degree, alpha, beta):
    return degree + 1 + (2 if alpha > 0 else 0) + (2 if beta > 0 else 0)


def closed_net(rng: random.Random, nu, nv, in_u):
    grid = net(rng, nu, nv)
    for k in range(nv if in_u else nu):
        first, last = (k, (nu - 1) * nv + k) if in_u else (k * nv, k * nv + nv - 1)
        grid[last] = list(grid[first])
    return grid


def sphere_patch():
    grid = [[0.0, 0.0, 0.0] for _ in range(9)]
    grid[1], grid[5], grid[8] = [0.0, 0.0, 1.0], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]
    return tbezier(grid, 3, 3, 1.0, 1.0)


def tbezier_specs(rng: random.Random):
    """Orders 1 to 3, rational or not, shifted bounds, closed grids and each refusal."""
    specs = [sphere_patch()]
    for nu, nv in ((3, 3), (3, 5), (5, 3), (5, 7)):
        au, av = rng.uniform(0.3, 2.5), rng.uniform(0.3, 2.5)
        lower = rng.choice([(1, 1), (0, -2), (5, 3)])
        specs.append(tbezier(net(rng, nu, nv), nu, nv, au, av, lower=lower))
        specs.append(tbezier(net(rng, nu, nv), nu, nv, au, av, positive(rng, nu * nv), lower=lower))
    specs += [tbezier(closed_net(rng, 3, 5, True), 3, 5, 1.0, 2.0),
              tbezier(closed_net(rng, 5, 3, False), 5, 3, 0.5, 1.0, positive(rng, 15)),
              tbezier(net(rng, 3, 3), 3, 3, 1.0, 1.0, [2.0] * 9),
              tbezier(net(rng, 3, 3), 3, 3, 1e-3, 40.0, positive(rng, 9)),
              tbezier(net(rng, 2, 3), 2, 3, 1.0, 1.0), tbezier(net(rng, 3, 4), 3, 4, 1.0, 1.0),
              tbezier(net(rng, 1, 3), 1, 3, 1.0, 1.0), tbezier(net(rng, 3, 3), 3, 3, 0.0, 1.0),
              tbezier(net(rng, 3, 3), 3, 3, 1.0, -1.0),
              tbezier(net(rng, 3, 3), 3, 3, 1.0, 1.0, [1.0] * 15, weight_shape=(3, 5)),
              tbezier(net(rng, 3, 3), 3, 3, 1.0, 1.0, [1, 1, 1, 1, 0, 1, 1, 1, 1]),
              tbezier(net(rng, 3, 3), 3, 3, 1.0, 1.0, [1, 1, 1, 1, 1, 1, 1, 1, -2])]
    return specs


def aht_specs(rng: random.Random):
    """Every basis shape per direction (the bicubic and general Horner paths included),
    rational or not, shifted bounds and each refusal."""
    specs = []
    shapes = [(0, 0.0, 0.0), (1, 0.0, 0.0), (2, 0.0, 0.0), (3, 0.0, 0.0), (4, 0.0, 0.0),
              (1, rng.uniform(0.2, 2), 0.0), (0, 0.0, rng.uniform(0.2, 4)),
              (2, rng.uniform(0.2, 2), rng.uniform(0.2, 4))]
    pairs = [(shapes[3], shapes[3]), (shapes[2], shapes[2]), (shapes[0], shapes[4]), (shapes[4], shapes[1]),
             (shapes[1], shapes[5]), (shapes[6], shapes[2]), (shapes[7], shapes[3]), (shapes[5], shapes[7]),
             (shapes[7], shapes[7])]
    for (du, au, bu), (dv, av, bv) in pairs:
        nu, nv = dimension(du, au, bu), dimension(dv, av, bv)
        lower = rng.choice([(1, 1), (0, -2), (4, 2)])
        args = ((du, dv), (au, av), (bu, bv))
        specs.append(aht(net(rng, nu, nv), nu, nv, *args, lower=lower))
        specs.append(aht(net(rng, nu, nv), nu, nv, *args, positive(rng, nu * nv), lower=lower))
    specs += [aht(net(rng, 3, 3), 3, 3, (2, 2), (0.0, 0.0), (0.0, 0.0), [5.0] * 9),
              aht(net(rng, 3, 3), 3, 3, (-1, 2), (0.0, 0.0), (0.0, 0.0)),
              aht(net(rng, 3, 3), 3, 3, (2, 2), (-1.0, 0.0), (0.0, 0.0)),
              aht(net(rng, 3, 3), 3, 3, (2, 2), (0.0, 0.0), (0.0, -2.0)),
              aht(net(rng, 3, 4), 3, 4, (2, 2), (0.0, 0.0), (0.0, 0.0)),
              aht(net(rng, 3, 3), 3, 3, (2, 2), (0.0, 0.0), (0.0, 0.0), [1.0] * 6, weight_shape=(3, 2)),
              aht(net(rng, 3, 3), 3, 3, (2, 2), (0.0, 0.0), (0.0, 0.0), [1, 1, 1, 1, 1, 1, 1, 0, 1])]
    return specs


def uv_list(rng: random.Random):
    """Corners, zero (the exact algebraic branch), interior, outside and large parameters."""
    fixed = [0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, math.pi / 3, math.pi / 4, -0.5, 2.0, 7.5, -3.25]
    return fixed + [rng.uniform(-0.2, 1.2) for _ in range(6)]


def append_eval_bezier_surface_requests(requests: list[str]) -> None:
    """One g3_eval_bezier_surface request per T-Bezier and AHT-Bezier specification."""
    rng = random.Random(0xBE25)
    for spec in tbezier_specs(rng) + aht_specs(rng):
        requests.append('g3_eval_bezier_surface ' + fmt([*spec, *params(uv_list(rng)), *trsf(rng)]))
