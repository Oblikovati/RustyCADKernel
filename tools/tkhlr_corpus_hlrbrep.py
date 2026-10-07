"""Deterministic requests for the HLRBRep curves and surfaces (tools/oracle/tkhlr_probe_hlrbrep_*.cxx).

The fixtures (tkhlr_probe_hlrbrep_fixtures.cxx) read a projector (`pk loc dir xdir
[focus]`: parallel, perspective or the default top view) and an edge without vertices
(`ck ...` geometry on a range): lines, circles and ellipses in every orientation to the
view (along the sight, edge-on, oblique), degree 1 and degree 3 Bezier curves and
degree 2 B-splines.

- `hb_curve`: HLRBRep_Curve Update with its 16-direction box, the 2D/3D parameters,
  depth, D0-D2 (with preset outputs), Tangent at both ends, the projected line,
  circle and ellipse, and the projected poles, weights, knots and multiplicities.
"""
from __future__ import annotations
import math
import random

VIEW_DIRS = [(0.0, 0.0, 1.0), (0.0, 0.0, -1.0), (1.0, 0.0, 0.0), (0.0, 1.0, 0.0),
             (1.0, 1.0, 1.0), (1.0, -1.0, 1.0), (0.3, -0.4, 0.5), (1.0, 1e-9, 0.0),
             (-0.2, 0.9, -0.1)]


def numbers(values: list[float]) -> str:
    """The fields of a request, with every double spelled exactly."""
    return ' '.join(repr(float(v)) for v in values)


def unit(v: tuple[float, float, float]) -> tuple[float, float, float]:
    n = math.sqrt(sum(x * x for x in v))
    return (v[0] / n, v[1] / n, v[2] / n)


def x_direction(n: tuple[float, float, float]) -> tuple[float, float, float]:
    """A direction not parallel to `n`, for a frame with normal `n`."""
    a = unit(n)
    return (1.0, 0.0, 0.0) if abs(a[0]) < 0.9 else (0.0, 1.0, 0.0)


def projector(rng: random.Random, kind: int | None = None) -> list[float]:
    """`pk loc dir xdir [focus]`."""
    kind = rng.choice((0, 0, 1, 2)) if kind is None else kind
    if kind == 2:
        return [2]
    d = rng.choice(VIEW_DIRS)
    loc = rng.choice(((0.0, 0.0, 0.0), (1.0, -2.0, 0.5), (10.0, 3.0, -4.0)))
    fields = [kind, *loc, *d, *x_direction(d)]
    return fields + ([rng.choice((50.0, 100.0, 7.5))] if kind == 1 else [])


def point(rng: random.Random, scale: float = 5.0) -> list[float]:
    return [round(rng.uniform(-scale, scale), 3) for _ in range(3)]


def poles(rng: random.Random, n: int) -> list[float]:
    fields: list[float] = [n]
    for _ in range(n):
        fields += point(rng)
    return fields


def geometry(rng: random.Random, kind: int, view: tuple[float, float, float]) -> tuple[list[float], float, float]:
    """`ck ...` and its parameter range."""
    if kind == 0:
        d = rng.choice(VIEW_DIRS + [view])
        u1 = rng.uniform(-3, 0)
        return [0, *point(rng), *d], u1, u1 + rng.choice((0.5, 2.0, 7.0))
    if kind in (1, 2):
        n = rng.choice(VIEW_DIRS + [view, view])
        fields = [kind, *point(rng), *n, *x_direction(n)]
        fields += [rng.choice((1.0, 2.5))] if kind == 1 else [3.0, rng.choice((1.0, 2.9))]
        u1 = rng.choice((0.0, 0.5, -1.0))
        return fields, u1, u1 + rng.choice((math.pi / 2, math.pi, 2 * math.pi))
    if kind == 3:
        u1 = rng.choice((0.0, 0.25))
        return [3, *poles(rng, rng.choice((2, 4)))], u1, rng.choice((0.75, 1.0))
    n = rng.choice((3, 4, 6))
    return [4, *poles(rng, n)], 0.0, float(n - 2) * rng.choice((0.5, 1.0))


def edge(rng: random.Random, view: tuple[float, float, float]) -> tuple[list[float], float, float]:
    fields, u1, u2 = geometry(rng, rng.choice((0, 1, 1, 2, 2, 3, 4)), view)
    return fields + [u1, u2], u1, u2


def curve_requests(rng: random.Random) -> list[str]:
    """hb_curve over random projectors and edges."""
    requests = []
    for _ in range(600):
        proj = projector(rng)
        view = tuple(proj[4:7]) if proj[0] != 2 else (0.0, 0.0, 1.0)
        fields, u1, u2 = edge(rng, view)
        ts = [u1, u2, u1 + (u2 - u1) * rng.random()]
        requests.append('hb_curve ' + numbers(proj + fields + ts))
    return requests


def append_hlrbrep_requests(requests: list[str]) -> None:
    """Append every HLRBRep request family in a fixed order."""
    rng = random.Random(0x41B12E9)
    requests.extend(curve_requests(rng))
