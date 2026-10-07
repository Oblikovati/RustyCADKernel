"""Deterministic requests for the chamfer probes of BlendFunc
(tools/oracle/tkfillet_probe_blendfunc.cxx).

`fl_chamfer` evaluates BlendFunc_Chamfer (kind 0), BlendFunc_ConstThroat (1),
BlendFunc_ConstThroatWithPenetration (2) and BlendFunc_ChAsym (3); `fl_chamfer_inv` their
inverses. The right-angled planes give exact sections (two distances, a throat, a distance
and an angle), perturbed and trimmed; a sphere cut by a plane along a circle and a plane
with a cylinder give curved cases.
"""
from __future__ import annotations
import math
import random

from tkfillet_corpus_blendfunc import X, Y, Z, guide, numbers, surface


def chamfer(s1, s2, c, kind, d1, d2, choix, t, x4, tol=1e-7, trim=None) -> str:
    """One `fl_chamfer` request."""
    trim_fields = [1, *trim] if trim else [0, 0.0, 0.0]
    return 'fl_chamfer ' + numbers([*s1, *s2, *c, kind, d1, d2, choix, t, *trim_fields, *x4, tol])


def chamfer_inv(s1, s2, c, curve2d, kind, d1, d2, choix, first, x4, tol=1e-7) -> str:
    """One `fl_chamfer_inv` request; `curve2d` is (u, v, du, dv)."""
    return 'fl_chamfer_inv ' + numbers([*s1, *s2, *c, *curve2d, kind, d1, d2, choix, int(first), *x4, tol])


def right_angle():
    """The planes z = 0 (point (u, v, 0)) and y = 0 (point (u, 0, -v)) along the X axis."""
    return surface(0, n=Z, x=X), surface(0, n=Y, x=X), guide(0, n=X, x=Y)


def exact_sections(t: float) -> list[tuple[int, float, float, list[float]]]:
    """(kind, d1, d2, X) of exact sections at `t` for an odd choice: the distances 1 and
    2, the throat sqrt(1/2) (points at 1), and the distance 1 with the angle atan(2)."""
    return [
        (0, 1.0, 2.0, [t, 1.0, t, -2.0]),
        (1, math.sqrt(0.5), 0.0, [t, 1.0, t, -1.0]),
        (2, 1.0, 0.0, [t, 1.0, t, -0.5]),
        (3, 1.0, math.atan(2.0), [t, 1.0, t, -2.0]),
    ]


def right_angle_requests(requests: list[str]) -> None:
    s1, s2, c = right_angle()
    for t in (0.0, 0.75, -1.5):
        for kind, d1, d2, x4 in exact_sections(t):
            for choix in (1, 2, 3, 4, 5, 6, 7, 8):
                requests.append(chamfer(s1, s2, c, kind, d1, d2, choix, t, x4))
            requests.append(chamfer(s1, s2, c, kind, d1, d2, 1, t, [x4[0] + 0.1, x4[1] * 1.2, x4[2], x4[3] - 0.3]))
            requests.append(chamfer(s1, s2, c, kind, d1, d2, 1, t, x4, trim=(-2.0, 3.0)))


def curved_requests(requests: list[str]) -> None:
    sphere = surface(2, n=Z, x=X, a=2.0)
    cut = surface(0, p=(0.0, 0.0, 1.0), n=Z, x=X)
    circle = guide(1, p=(0.0, 0.0, 1.0), n=Z, x=X, a=math.sqrt(3.0))
    plane = surface(0, n=Z, x=X)
    cylinder = surface(1, n=X, x=Y, a=2.0)
    line = guide(0, p=(0.0, 2.0, 0.0), n=X, x=Y)
    for kind in range(4):
        for t in (0.0, 0.7, 2.0):
            x4 = [t, 0.4, 1.4 * math.cos(t), 1.4 * math.sin(t)]
            requests.append(chamfer(sphere, cut, circle, kind, 0.3, 0.4, 1 + kind, t, x4))
            requests.append(chamfer(plane, cylinder, line, kind, 0.5, 0.6, 3, t, [t, 1.6, t, 0.2]))


def random_requests(requests: list[str]) -> None:
    rng = random.Random(20261009)
    s1, s2, c = right_angle()
    for _ in range(16):
        t, kind = rng.uniform(-3.0, 3.0), rng.randint(0, 3)
        d1, d2 = rng.uniform(0.1, 3.0), rng.uniform(0.1, 1.4)
        x4 = [t + rng.uniform(-0.2, 0.2), d1 + rng.uniform(-0.3, 0.3), t + rng.uniform(-0.2, 0.2), -rng.uniform(0.1, 3.0)]
        requests.append(chamfer(s1, s2, c, kind, d1, d2, rng.randint(1, 8), t, x4))


def inverse_requests(requests: list[str]) -> None:
    """The inverses with the section ending on the iso line of either plane at the first
    distance, exact and perturbed; a sphere cut by a plane."""
    s1, s2, c = right_angle()
    for kind, d1, d2, _ in exact_sections(0.0):
        far = {0: 2.0, 1: 1.0, 2: 0.5, 3: 2.0}[kind]
        for first in (True, False):
            line = (0.0, 1.0, 1.0, 0.0) if first else (0.0, -far, 1.0, 0.0)
            for w in (0.0, 0.6, -1.1):
                exact = [w, w, w, -far] if first else [w, w, w, 1.0]
                for choix in (1, 2, 5, 8):
                    requests.append(chamfer_inv(s1, s2, c, line, kind, d1, d2, choix, first, exact))
                requests.append(chamfer_inv(s1, s2, c, line, kind, d1, d2, 1, first, [w + 0.1, w - 0.2, w, 0.3]))
    sphere = surface(2, n=Z, x=X, a=2.0)
    cut = surface(0, p=(0.0, 0.0, 1.0), n=Z, x=X)
    circle = guide(1, p=(0.0, 0.0, 1.0), n=Z, x=X, a=math.sqrt(3.0))
    for kind in range(4):
        for first in (True, False):
            requests.append(chamfer_inv(sphere, cut, circle, (0.0, 0.4, 1.0, 0.0), kind, 0.3, 0.4, 2, first, [0.5, 0.5, 1.2, 0.4]))


def append_chamfer_requests(requests: list[str]) -> None:
    """Append the chamfer request families."""
    right_angle_requests(requests)
    curved_requests(requests)
    random_requests(requests)
    inverse_requests(requests)
