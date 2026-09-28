"""TKG3d corpus: GeomAdaptor_TransformedCurve, Adaptor3d_HVertex and Adaptor3d_TopolTool
(tools/oracle/tkg3d_probe_topology.cxx). Curve adaptors follow a3_geom_curve
(tkg3d_corpus_geom_adaptor.py); 2D adaptors and surface adaptors follow
tkg3d_corpus_adaptor3d.py."""
from __future__ import annotations
import math
import random
from tkg3d_corpus import bezier, bspline, circle, fmt, frame, line, offset, params, point, trsf, unit
from tkg3d_corpus_adaptor3d import (F0, adapted, bounded, bspline_surface, cylinder, curves2d, line2d, plane,
                                    ranged, surfaces, whole, conic2)
from tkg3d_corpus_geom_adaptor import curve_specs


def transformed_inputs(rng: random.Random) -> list[list]:
    """The curve-on-surface inputs: lines on a plane (a Geom2dAdaptor curve on a plane would
    make the native Bezier()/BSpline() dereference a null spline), any 2D curve elsewhere."""
    grid = bspline_surface([[0.0, 3], [0.5, 1], [1.0, 3]], [[0.0, 3], [0.25, 2], [1.0, 3]], (2, 2), rng)
    x, y = [1.0, 0.0], [0.0, 1.0]
    on_plane = [[*line2d([0.3, 0.2], x, 0.0, 0.4), *adapted(plane(F0))],
                [*line2d([0.0, 0.0], [0.6, 0.8], 0.0, 1.0), *adapted(plane(frame(rng)))]]
    elsewhere = [[*c, *adapted(cylinder(F0, 2.0))] for c in curves2d(rng)[::3]]
    elsewhere += [[*whole(conic2(1, [0.5, 0.5], x, True, 0.3)), *adapted(grid)],
                  [*ranged(0.0, 0.5, [0, 0.2, 0.2, *y]), *adapted(grid)]]
    return on_plane + elsewhere


def append_transformed_curve_requests(rng: random.Random, requests: list[str]) -> None:
    specs = curve_specs(rng)
    cos_inputs = transformed_inputs(rng)
    for index, (spec, u, (t1, t2)) in enumerate(specs):
        for kind, adaptor in ((0, [0, *spec]), (1, [1, 0.3, 0.6, *spec])):
            cos = cos_inputs[(2 * index + kind) % len(cos_inputs)]
            m = trsf(rng, [1, 3, 4, 5, 7][(index + kind) % 5])
            requests.append('a3_transformed_curve ' + fmt([*adaptor, *params(u), *m, t1, t2, 1, *cos]))
        requests.append('a3_transformed_curve ' + fmt([0, *spec, *params(u), *trsf(rng, 3), t1, t2, 0]))


def append_hvertex_requests(rng: random.Random, requests: list[str]) -> None:
    x = [1.0, 0.0]
    curves = [line2d([0.0, 1.0], x, -1.0, 2.0), line2d([1.0, 1.0], [0.6, 0.8], 0.0, 1.0),
              whole(conic2(1, [0.5, 0.5], x, True, 0.3))]
    for index, curve in enumerate(curves):
        for ori in range(4):
            p = [rng.uniform(-2, 2), rng.uniform(-2, 2)]
            q = [p[0] + [0.0, 5e-8, 1e-6, 1.0][ori], p[1]]
            requests.append('a3_hvertex ' + fmt([*p, ori, [1e-8, 0.0, 0.5, 1e-3][index % 4], *q, *curve]))


def classify_points(rng: random.Random) -> list[float]:
    """Points inside, on the sides and corners (and within the tolerance), outside, far."""
    pts = [(0.0, 0.0), (0.5, 0.5), (1.0, 1.0), (0.0, 0.5), (1.0, 0.25), (1e-8, 0.5), (0.5, 1.0 + 1e-8),
           (-0.25, 0.5), (0.5, 1.5), (2.0, 2.0), (-1e6, 0.3), (0.3, 1e6), (math.pi, 1.0), (2 * math.pi, -1.0),
           (rng.uniform(-3, 3), rng.uniform(-3, 3))]
    return [c for pt in pts for c in pt]


def append_topol_requests(rng: random.Random, requests: list[str]) -> None:
    grid = bspline_surface([[0.0, 3], [0.5, 1], [1.0, 3]], [[0.0, 3], [0.25, 2], [1.0, 3]], (2, 2), rng)
    fine = bspline_surface([[float(k) / 10, 4 if k in (0, 10) else 1] for k in range(11)],
                           [[0.0, 4], [0.5, 2], [1.0, 4]], (3, 3), rng)
    extra = [adapted(fine), bounded([0.1, 0.9, 0.3, 0.8], [0.0, 0.0], fine), bounded([0.0, 0.5, 0.25, 1.0], [0.0, 0.0], grid),
             adapted([2, *F0, 0.3, 2.0]), bounded([0.0, 2 * math.pi, -3.0, 1.0], [0.0, 0.0], [2, *F0, 0.3, 2.0]),
             adapted([2, *frame(rng), -0.4, 1.0])]
    # Minimum counts of at least 4: below, a direction of 3 candidates keeps 2 flags in an
    # array of 4, whose last values natively are uninitialised memory.
    samples = [(0.01, 10, 10), (1e-5, 4, 4), (0.5, 20, 4), (1.0, 4, 20)]
    for index, surface in enumerate(surfaces(rng) + extra):
        defl, numin, nvmin = samples[index % len(samples)]
        requests.append('a3_topol ' + fmt([*surface, *params(classify_points(rng)), 1e-7, defl, numin, nvmin]))


def append_cone_apex_requests(rng: random.Random, requests: list[str]) -> None:
    for radius, angle in ((2.0, 0.3), (0.0, 0.5), (1.0, -0.4), (1e-12, 1.2), (3.0, 1.5), (1.0, 0.0)):
        requests.append('a3_cone_apex ' + fmt([*frame(rng), radius, angle]))
    requests.append('a3_cone_apex ' + fmt([*F0, 2.0, 0.3]))


def append_topology_requests(requests: list[str]) -> None:
    """Append every request family of tkg3d_probe_topology.cxx in a fixed order."""
    rng = random.Random(0x70901)
    append_transformed_curve_requests(rng, requests)
    append_hvertex_requests(rng, requests)
    append_topol_requests(rng, requests)
    append_cone_apex_requests(rng, requests)
