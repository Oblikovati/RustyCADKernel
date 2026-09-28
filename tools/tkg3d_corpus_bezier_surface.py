"""TKG3d corpus: Geom_BezierSurface (tools/oracle/tkg3d_probe_bezier_surface.cxx).

Bezier surface specifications are g3_surface kind 7: `7 rational nu nv (x y z w)*(nu*nv)`.
"""
from __future__ import annotations
import random
from tkg3d_corpus import fmt, moves, params, point, trsf
from tkg3d_corpus_surfaces import descriptor, cylinder, plane
from tkg3d_corpus import frame


def bezier_surface(nu, nv, net, rational=False):
    """A kind 7 spec from a row-major net of [x, y, z, w]."""
    return [7, int(rational), nu, nv, *[c for pole in net for c in pole]]


def net(rng: random.Random, nu, nv, weight=lambda i, j, rng: 1.0):
    return [[i + rng.uniform(-0.3, 0.3), j + rng.uniform(-0.3, 0.3), rng.uniform(-1, 1), weight(i, j, rng)]
            for i in range(nu) for j in range(nv)]


def specs(rng: random.Random) -> list[list]:
    """Valid, rational in U, V or both, closed, degenerate and invalid surfaces."""
    random_weight = lambda i, j, r: r.uniform(0.5, 3.0)
    u_only = lambda i, j, r: 1.0 + 0.5 * j
    v_only = lambda i, j, r: 1.0 + 0.5 * i
    closed = net(rng, 3, 3)
    for j in range(3):
        closed[6 + j][:3] = closed[j][:3]
    return [
        bezier_surface(2, 2, net(rng, 2, 2)),
        bezier_surface(3, 3, net(rng, 3, 3)),
        bezier_surface(4, 2, net(rng, 4, 2)),
        bezier_surface(2, 5, net(rng, 2, 5)),
        bezier_surface(3, 4, net(rng, 3, 4, random_weight), True),
        bezier_surface(3, 3, net(rng, 3, 3, u_only), True),
        bezier_surface(3, 3, net(rng, 3, 3, v_only), True),
        bezier_surface(2, 3, net(rng, 2, 3, lambda i, j, r: 2.0), True),
        bezier_surface(3, 3, closed),
        bezier_surface(26, 2, net(rng, 26, 2)),
        bezier_surface(2, 2, [[0.0, 0.0, 0.0, 1.0]] * 4),
        bezier_surface(1, 3, net(rng, 1, 3)),
        bezier_surface(27, 2, net(rng, 27, 2)),
        bezier_surface(2, 2, net(rng, 2, 2, lambda i, j, r: 0.0 if i == j == 1 else 1.0), True),
    ]


def uv_pairs() -> list[float]:
    return [0.0, 0.0, 1.0, 1.0, 0.25, 0.75, 0.5, 0.5, -0.5, 1.5, 0.3, 1e-9]


def line(rng: random.Random, n, lower=1, shift=0, weights=None):
    """A line argument: lower, n, (x y z w)*n, weight lower shift."""
    weights = weights or [rng.uniform(0.5, 2.0) for _ in range(n)]
    return [lower, n, *[c for k in range(n) for c in (*point(rng, 3.0), weights[k])], shift]


def action(rng: random.Random, code, i=1, j=1, w=2.0, abcd=(0.0, 1.0, 0.0, 1.0), line_args=None):
    return [code, i, j, *point(rng, 3.0), w, *abcd, *(line_args or line(rng, 2)), *trsf(rng)]


def actions_for(rng: random.Random, nu, nv) -> list[list]:
    """Every action with valid and failing arguments, in an order that keeps the surface valid."""
    return [
        action(rng, 20, 1, 1, 3.0), action(rng, 20, 1, 1, 1.0), action(rng, 20, 2, 2, 0.0), action(rng, 20, 0, 1, 2.0),
        action(rng, 14, 2, 1), action(rng, 14, nu + 1, 1), action(rng, 15, 1, 2, 0.5), action(rng, 15, 1, 2, -1.0),
        action(rng, 16, 1, line_args=line(rng, nu)), action(rng, 17, 2, line_args=line(rng, nu)),
        action(rng, 17, 2, line_args=line(rng, nu, 1, 1)), action(rng, 16, 1, line_args=line(rng, nu, 0)),
        action(rng, 18, 1, line_args=line(rng, nv)), action(rng, 19, 2, line_args=line(rng, nv)),
        action(rng, 18, 1, line_args=line(rng, 1, 2)), action(rng, 19, 1, line_args=line(rng, 2, 1, 0, [1.0, 0.0])),
        action(rng, 21, 1, line_args=line(rng, nu)), action(rng, 21, 1, line_args=line(rng, nu, 2)),
        action(rng, 21, 2, line_args=line(rng, nu, 1, 0, [2.0] + [-1.0] * (nu - 1))),
        action(rng, 22, 1, line_args=line(rng, nv)), action(rng, 22, 2, line_args=line(rng, 1, 2)),
        action(rng, 22, 1, line_args=line(rng, nv, 1, 0, [1.0] * nv)),
        action(rng, 3, 1, line_args=line(rng, nu)), action(rng, 4, 2, line_args=line(rng, nu)),
        action(rng, 5, 1, line_args=line(rng, nu)), action(rng, 5, 2, line_args=line(rng, nu)),
        action(rng, 6, 2, line_args=line(rng, nu, 0)), action(rng, 4, 1, line_args=line(rng, nu, 1, 0, [1.0, -2.0, 1.0][:nu] + [1.0] * max(0, nu - 3))),
        action(rng, 7, 1, line_args=line(rng, nv + 4)), action(rng, 8, 1, line_args=line(rng, nv + 4)),
        action(rng, 9, 2, line_args=line(rng, nv + 4)), action(rng, 10, 3, line_args=line(rng, nv + 4, 3)),
        action(rng, 8, 9, line_args=line(rng, nv + 4)), action(rng, 3, 0, line_args=line(rng, nu)),
        action(rng, 11, 2), action(rng, 12, 1), action(rng, 11, 0), action(rng, 12, 30),
        action(rng, 13, abcd=(0.25, 0.75, 0.1, 0.6)), action(rng, 13, abcd=(0.8, 0.2, -0.5, 1.5)),
        action(rng, 13, abcd=(0.5, 0.5, 0.0, 1.0)),
        action(rng, 2, 5, 3), action(rng, 2, 1, 1), action(rng, 2, 26, 4), action(rng, 2, 5, 3),
        action(rng, 1), action(rng, 23), action(rng, 24), action(rng, 25), action(rng, 1),
        action(rng, 12, 1), action(rng, 11, 1), action(rng, 12, 1), action(rng, 11, 1),
    ]


def append_bezier_surface_requests(requests: list[str]) -> None:
    """g3_surface, g3_surface_moves, g3_bezier_surface and g3_bezier_surface_evalrep."""
    rng = random.Random(0xB5E1)
    isos = params([0.0, 1.0, 0.3, -0.5])
    for spec in specs(rng):
        requests.append('g3_surface ' + fmt([*spec, *params(uv_pairs()), *isos, *params([0.0, 0.5, 1.0])]))
        requests.append('g3_surface_moves ' + fmt([*spec, *params(uv_pairs()[:6]), *trsf(rng), *moves(rng)]))
    for spec in specs(rng)[:9]:
        nu, nv = spec[2], spec[3]
        chosen = actions_for(rng, nu, nv)
        requests.append('g3_bezier_surface ' + fmt([*spec, *params(uv_pairs()[:6]), len(chosen),
                                                    *[c for a in chosen for c in a]]))
    append_evalrep_requests(rng, requests)


def append_evalrep_requests(rng: random.Random, requests: list[str]) -> None:
    """Evaluation representations of every kind, and the edits that keep or clear them."""
    reps = [plane(frame(rng)), cylinder(frame(rng), 1.5), bezier_surface(3, 2, net(rng, 3, 2))]
    edits = [action(rng, code) for code in (1, 13, 20, 14, 23, 24, 25)] + [action(rng, 2, 4, 4)]
    for index, spec in enumerate(specs(rng)[:8]):
        for kind, order in ((0, 0), (1, 1), (1, 3), (2, 0), (2, 2), (2, 3)):
            rep = reps[(index + kind + order) % len(reps)]
            requests.append('g3_bezier_surface_evalrep ' + fmt([*spec, *params(uv_pairs()), *rep,
                                                                *descriptor(rng, kind, order), len(edits),
                                                                *[c for a in edits for c in a]]))
