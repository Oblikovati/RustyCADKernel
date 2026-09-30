"""Deterministic requests for ShapeExtend_ComplexCurve (tools/oracle/tkshhealing_probe_complexcurve.cxx).

The segments are curves of a TKTopAlgo shape program (tools/tktopalgo_shape_program.py):
Bezier arcs, a trimmed line, a full and a trimmed circle and a B-spline that closes the
chain. Each request builds the probe's complex curve on some of them (connected, closed,
broken, single, repeated), checks its connectivity with a precision, evaluates it at
parameters inside, at the joints, at the ends and outside [0, 1], then transforms it.
"""
from __future__ import annotations
import math
from tktopalgo_shape_program import ShapeProgram, num

# Curve kinds of the shape program (op 2).
LINE, CIRCLE, BEZIER, TRIMMED, BSPLINE = 0, 1, 3, 4, 5


def curves() -> tuple[ShapeProgram, dict[str, int]]:
    """The segment curves, by name."""
    p = ShapeProgram()
    c = {}
    c['bez0'] = p.curve(BEZIER, 3, 0., 0., 0., 1., 1., 0., 2., 0., 0.)
    c['bez1'] = p.curve(BEZIER, 3, 2., 0., 0., 3., -1., 0., 4., 0., 1.)
    line = p.curve(LINE, 4., 0., 1., 1., 0., 0.)
    c['seg'] = p.curve(TRIMMED, line, 0., 2.)
    c['circle'] = p.curve(CIRCLE, 0., 0., 0., 0., 0., 1., 1.)
    c['arc'] = p.curve(TRIMMED, c['circle'], 0., math.pi)
    c['close'] = p.curve(BSPLINE, 2, 4, 6., 0., 1., 7., 2., 1., 3., 1., 0., 0., 0., 0.,
                         3, 0., 3, 0.5, 1, 1., 3)
    return p, c


# (segments, precision) of each complex curve.
CHAINS = [
    (['bez0', 'bez1'], 1e-7),
    (['bez0', 'bez1', 'seg', 'close'], 1e-7),
    (['bez0'], 1e-7),
    (['bez0', 'seg'], 1e-7),
    (['bez0', 'seg'], 3.),
    (['circle'], 1e-7),
    (['arc', 'arc'], 1e-7),
    (['bez1', 'bez0'], 0.),
]
PARAMETERS = [-0.5, 0., 0.1, 0.25, 0.5, 0.5 - 1e-17, 0.75, 1. / 3., 0.999, 1., 1.25]
TRANSFORMS = [[0], [1, 0.5, -1., 2.], [2, 1., 0., 0., 0., 0., 1., 0.7]]


def request(p: ShapeProgram, c: dict[str, int], chain: list[str], preci: float, trsf: list[float]) -> str:
    """One `sh_complex program n (curve)... preci nu (u)... trsf` line."""
    fields = [len(chain), *[c[name] for name in chain], preci, len(PARAMETERS), *PARAMETERS, *trsf]
    return ' '.join(['sh_complex', p.tokens(), *map(num, fields)])


def append_complexcurve_requests(requests: list[str]) -> None:
    """Append the ShapeExtend_ComplexCurve requests: every chain under every transform."""
    p, c = curves()
    for chain, preci in CHAINS:
        requests.extend(request(p, c, chain, preci, trsf) for trsf in TRANSFORMS)
