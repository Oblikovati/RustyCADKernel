"""2D curve specifications for the TKTopAlgo Bisector, MAT and MAT2d requests.

Each helper returns the list of numbers that ga_curve2d (tools/oracle/tkgeomalgo_probe_geom2dint.cxx,
read again by tp_curve2d in tools/oracle/tktopalgo_probe_bisector.cxx) reads:
0 line P D | 1 circle P XD sense R | 2 ellipse P XD sense A B | 3 hyperbola P XD sense A B |
4 parabola P XD sense F | 5 bezier rational n (x y w)*n | 6 bspline rational periodic degree
nk (k m)*nk np (x y w)*np | 7 trimmed u1 u2 sense adjust <curve> | 8 offset d notcheck <curve>.
"""
from __future__ import annotations
import math

Point = tuple[float, float]


def numbers(values: list[float]) -> str:
    """The fields of a request, with every double spelled exactly."""
    return ' '.join(repr(float(v)) for v in values)


def line(p: Point, d: Point) -> list[float]:
    """An infinite line through `p` along `d`."""
    return [0, p[0], p[1], d[0], d[1]]


def circle(c: Point, r: float, sense: int = 1) -> list[float]:
    """A circle of centre `c`, X direction +X."""
    return [1, c[0], c[1], 1.0, 0.0, sense, r]


def ellipse(c: Point, xd: Point, a: float, b: float, sense: int = 1) -> list[float]:
    """An ellipse of major axis along `xd`."""
    return [2, c[0], c[1], xd[0], xd[1], sense, a, b]


def parabola(apex: Point, xd: Point, focal: float, sense: int = 1) -> list[float]:
    """A parabola of apex `apex` and axis `xd`."""
    return [4, apex[0], apex[1], xd[0], xd[1], sense, focal]


def bspline(poles: list[Point], degree: int) -> list[float]:
    """A clamped non-rational B-spline with uniform interior knots on [0, 1]."""
    n_knots = len(poles) - degree + 1
    knots: list[float] = []
    for i in range(n_knots):
        mult = degree + 1 if i in (0, n_knots - 1) else 1
        knots += [i / (n_knots - 1), mult]
    flat: list[float] = []
    for x, y in poles:
        flat += [x, y, 1.0]
    return [6, 0, 0, degree, n_knots] + knots + [len(poles)] + flat


def trimmed(u1: float, u2: float, basis: list[float], sense: int = 1) -> list[float]:
    """A trimmed curve (the periodic bounds adjusted)."""
    return [7, u1, u2, sense, 1] + basis


def segment(a: Point, b: Point) -> list[float]:
    """The segment from `a` to `b`: a line trimmed to [0, |ab|]."""
    dx, dy = b[0] - a[0], b[1] - a[1]
    length = math.hypot(dx, dy)
    return trimmed(0.0, length, line(a, (dx / length, dy / length)))


def arc(c: Point, r: float, a0: float, a1: float) -> list[float]:
    """The arc of the circle of centre `c` from the angle `a0` to `a1` (counterclockwise
    when a0 < a1, clockwise otherwise)."""
    if a0 < a1:
        return trimmed(a0, a1, circle(c, r))
    return trimmed(-a0, -a1, circle(c, r, 0))


def arc_point(c: Point, r: float, a: float) -> Point:
    """The point of angle `a` of a circle (approximately the one OCCT evaluates)."""
    return (c[0] + r * math.cos(a), c[1] + r * math.sin(a))
