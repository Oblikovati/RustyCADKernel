"""Deterministic requests for ShapeUpgrade (tools/oracle/tkshhealing_probe_shapeupgrade.cxx).

`su_c0split` splits 3d and 2d B-splines at their C0 knots: polylines, quadratic and cubic
curves with one or several C0 knots, mixed with smooth knots, rational, periodic (made
non-periodic in place), C1 curves and a null handle.

`su_tool` sets the precision and the tolerances of a ShapeUpgrade_Tool in every order,
copies another tool (with and without a context) and limits tolerances.

`su_removeloc` builds one scene with locations on faces (in the TFace and on the shape),
edges (on the 3D curve), vertices, shells, solids and compounds, with a seam and an edge
shared by two faces, and runs ShapeUpgrade_RemoveLocations at every removal level on each
shape, then ModifiedShape and the map of new shapes.

`su_splitcurve` inits ShapeUpgrade_SplitCurve and sets split values (sorted, unsorted,
within the precision of the bounds, outside the range, empty, null, the sequence itself,
repeated), then runs Perform, Compute and Build.
"""
from __future__ import annotations
import math
from tkshhealing_corpus_shcommon import PLANE, CYLINDER, Prelude, rotation, translation
from tktopalgo_shape_program import COMPOUND, FORWARD, REVERSED, SHELL, SOLID, WIRE, num

PI = math.pi


def request(op: str, *fields: float | int | str) -> str:
    return ' '.join([op, *[f if isinstance(f, str) else num(f) for f in fields]])


# ------------------------------------------------------------------ C0 B-spline splits
def spline(degree: int, periodic: bool, poles, weights, knots) -> list[float | int]:
    """`degree periodic n (pole weight)... nk (knot mult)...`."""
    out: list[float | int] = [degree, periodic, len(poles)]
    for pole, weight in zip(poles, weights):
        out += [*pole, weight]
    out.append(len(knots))
    for knot in knots:
        out += list(knot)
    return out


def ones(n: int) -> list[float]:
    return [1.] * n


def c0_curves3d() -> list[list[float | int]]:
    p7 = [(0., 0., 0.), (1., 2., 0.), (2., 2., 1.), (3., 0., 0.), (4., -1., 1.), (5., 1., 2.), (6., 0., 0.)]
    p5 = p7[:5]
    polyline = [(float(i), (-1.) ** i, 0.5 * i) for i in range(6)]
    p10 = [(float(i), math.sin(i), math.cos(i)) for i in range(10)]
    return [
        spline(3, False, p7, ones(7), [(0., 4), (1., 3), (2., 4)]),
        spline(3, False, p7[:6], ones(6), [(0., 4), (1., 2), (2., 4)]),
        spline(2, False, p5, ones(5), [(0., 3), (1., 2), (2., 3)]),
        spline(1, False, polyline, ones(6), [(0., 2), (1., 1), (2., 1), (3., 1), (4., 1), (5., 2)]),
        spline(3, False, p7, [1., 2., 0.5, 1., 3., 1., 1.], [(0., 4), (1., 3), (2., 4)]),
        spline(3, False, p10, ones(10), [(0., 4), (0.5, 3), (1., 3), (2., 4)]),
        spline(3, False, p10[:8], ones(8), [(0., 4), (0.25, 1), (0.5, 3), (1.5, 4)]),
        spline(2, False, p10[:9], [1., 1.5, 1., 2., 1., 1., 0.7, 1., 1.], [(0., 3), (1., 2), (2., 2), (3., 2), (4., 3)]),
        spline(3, True, p7[:6], ones(6), [(0., 1), (1., 3), (2., 1), (3., 1), (4., 1)]),
        spline(3, True, p10[:6], ones(6), [(0., 1), (1., 1), (2., 1), (3., 1), (4., 1), (5., 1), (6., 1)]),
        spline(2, True, p10[:6], ones(6), [(0., 2), (1., 2), (2., 2), (3., 2)]),
        spline(3, False, p7[:4], ones(4), [(0., 4), (1., 4)]),
        spline(4, False, p10[:9], ones(9), [(0., 5), (1., 4), (2., 5)]),
        spline(4, False, p10[:8], ones(8), [(0., 5), (1., 3), (2., 5)]),
        spline(3, False, p7, ones(7), [(-1e3, 4), (0., 3), (1e3, 4)]),
    ]


def c0_curves2d() -> list[list[float | int]]:
    p7 = [(0., 0.), (1., 2.), (2., 2.), (3., 0.), (4., -1.), (5., 1.), (6., 0.)]
    p9 = [(float(i), math.sin(i)) for i in range(9)]
    return [
        spline(3, False, p7, ones(7), [(0., 4), (1., 3), (2., 4)]),
        spline(3, False, p7[:6], ones(6), [(0., 4), (1., 2), (2., 4)]),
        spline(1, False, p7[:4], ones(4), [(0., 2), (1., 1), (3., 1), (4., 2)]),
        spline(2, False, p9, [1., 2., 1., 1., 0.5, 1., 1., 3., 1.], [(0., 3), (1., 2), (2., 2), (3., 2), (4., 3)]),
        spline(3, True, p7[:6], ones(6), [(0., 1), (1., 3), (2., 1), (3., 1), (4., 1)]),
        spline(2, True, p9[:6], ones(6), [(0., 2), (1., 2), (2., 2), (3., 2)]),
        spline(3, False, p7[:4], ones(4), [(0., 4), (1., 4)]),
    ]


def c0split_requests() -> list[str]:
    out = [request('su_c0split', 3, 0, *c) for c in c0_curves3d()]
    out += [request('su_c0split', 2, 0, *c) for c in c0_curves2d()]
    out += [request('su_c0split', 3, 1), request('su_c0split', 2, 1)]
    return out


# ------------------------------------------------------------------ ShapeUpgrade_Tool
def tool_requests() -> list[str]:
    scripts = [
        [],
        [0, 1e-3], [0, 5.], [0, 1e-9], [0, -1.],
        [1, 1e-2, 0, 1e-3], [2, 1e-4, 0, 1e-3], [1, 3., 2, 2., 0, 2.5],
        [3, 0.5], [3, 2.], [3, -3.], [2, 1e-5, 3, 1e-3, 3, 1e-6],
        [4, 1e-3, 1e-5, 0.1, 0], [4, 1e-3, 1e-5, 0.1, 1], [5, 4, 2., 1., 3., 0],
        [5], [0, 0.], [2, 0., 3, 0.], [0, 1e300, 3, 1e301],
    ]
    out = []
    for script in scripts:
        count, k = 0, 0
        widths = {0: 1, 1: 1, 2: 1, 3: 1, 4: 4, 5: 0}
        while k < len(script):
            k += 1 + widths[int(script[k])]
            count += 1
        out.append(request('su_tool', count, *script))
    return out


# ------------------------------------------------------------------ RemoveLocations
def removeloc_scene() -> tuple[str, dict[str, int]]:
    """The prelude and program of the RemoveLocations requests, with the named shapes."""
    pre = Prelude()
    plane = pre.elementary(PLANE, (0., 0., 0.), (0., 0., 1.), (1., 0., 0.), False)
    cyl = pre.elementary(CYLINDER, (0., 0., 0.), (0., 0., 1.), (1., 0., 0.), False, 1.)
    trimmed_plane = pre.trimmed(plane, -1., 2., -1., 2.)
    move = pre.location(translation((1., -2., 3.)))
    turn = pre.location(rotation((0., 0., 0.), (0., 1., 0.), 0.5))
    both = pre.location([6, *rotation((1., 0., 0.), (0., 0., 1.), 0.3), *translation((0., 0., 2.))])
    p = pre.program()
    sphere = p.surface(2, 0., 0., 0., 0., 0., 1., 2.)
    n: dict[str, int] = {}
    v0, v1, v2 = p.vertex((1., 0., 0.)), p.vertex((1., 0., 1.)), p.vertex((0., 1., 0.), 1e-5)
    n['v_moved'] = p.located(v2, move)
    # A cylinder face (location in the TFace) with a seam and a shared line edge.
    fa = p.face(cyl, turn)
    seam = p.edge()
    p.edge_curve(seam, p.line((1., 0., 0.), (0., 0., 1.)), move)
    p.pcurves_on_face(seam, p.line2d((0., 0.), (0., 1.)), p.line2d((2 * PI, 0.), (0., 1.)), fa)
    p.range_on_face(seam, fa, 0., 1.)
    p.edge_range(seam, 0., 1.)
    p.add(seam, v0, FORWARD)
    p.add(seam, v1, REVERSED)
    shared = p.edge()
    p.edge_curve(shared, p.line((1., 0., 0.), (-1., 1., 0.)))
    p.pcurve_on_face(shared, p.line2d((0., 0.), (1., 0.)), fa)
    p.range_on_face(shared, fa, 0., 1.)
    p.edge_range(shared, 0., 1.)
    p.add(shared, v0, FORWARD)
    p.add(shared, v2, REVERSED)
    wa = p.container(WIRE)
    p.add(wa, seam, FORWARD)
    p.add(wa, shared, FORWARD)
    p.add(wa, seam, REVERSED)
    p.add(fa, wa)
    # A sphere face sharing the line edge, and a planar face located on the shape.
    fc = p.face(sphere)
    p.pcurve_on_face(shared, p.line2d((0., 0.5), (1., 0.)), fc)
    p.range_on_face(shared, fc, 0., 1.)
    wc = p.container(WIRE)
    p.add(wc, shared, REVERSED)
    p.add(fc, wc)
    fb = p.face(trimmed_plane)
    eb = p.edge()
    p.edge_curve(eb, p.line((0., 0., 0.), (1., 0., 0.)), turn)
    p.pcurve_on_face(eb, p.line2d((0., 0.), (1., 0.)), fb)
    p.edge_range(eb, 0., 1.)
    p.add(eb, v0, FORWARD)
    p.add(eb, v2, REVERSED)
    wb = p.container(WIRE)
    p.add(wb, eb)
    p.add(fb, wb)
    n['fb_moved'] = p.located(fb, both)
    n['fa'], n['fc'], n['fb'], n['shared'], n['seam'], n['eb'] = fa, fc, fb, shared, seam, eb
    shell = p.container(SHELL)
    p.add(shell, fa)
    p.add(shell, fc)
    n['shell'] = shell
    n['shell_moved'] = p.located(shell, move)
    solid = p.container(SOLID)
    p.add(solid, n['shell_moved'])
    n['solid'] = solid
    compound = p.container(COMPOUND)
    for name in ('shell_moved', 'fb_moved', 'v_moved', 'eb'):
        p.add(compound, n[name])
    n['compound'] = compound
    n['compound_moved'] = p.located(compound, turn)
    n['edge_moved'] = p.located(shared, both)
    n['plane_face_moved'] = p.located(p.face(plane), move)
    return pre.tokens()+' '+p.tokens(), n


def removeloc_requests() -> list[str]:
    scene, n = removeloc_scene()
    out = []
    targets = ['compound_moved', 'compound', 'solid', 'shell_moved', 'shell', 'fa', 'fb_moved', 'fc',
               'edge_moved', 'shared', 'v_moved', 'plane_face_moved']
    for level in (8, 0, 1, 2, 3, 4, 5, 6, 7):
        for target in targets:
            ops = [0, level, 1, n[target], 2, n['shared'], 2, n['fa'], 3]
            out.append(request('su_removeloc', scene, 5, *ops))
    twice = [1, n['shell_moved'], 1, n['compound_moved'], 2, n['seam'], 3, 4]
    out.append(request('su_removeloc', scene, 5, *twice))
    out.append(request('su_removeloc', scene, 3, 0, 4, 4, 2, n['fa']))
    return out


# ------------------------------------------------------------------ SplitCurve
def splitcurve_requests() -> list[str]:
    scripts = [
        [], [5], [4, 1], [0, 0., 10.], [0, 0., 10., 1, 3, 2., 5., 8.],
        [0, 0., 10., 1, 3, 5., 2., 8.], [0, 0., 10., 1, 2, 1e-10, 10. - 1e-10],
        [0, 0., 10., 1, 3, -1., 5., 11.], [0, 0., 10., 1, 0], [0, 0., 10., 2],
        [0, 0., 10., 1, 2, 3., 7., 3], [0, 0., 10., 1, 2, 3., 7., 1, 3, 1., 5., 9.],
        [0, 0., 10., 1, 3, 3., 3. + 1e-9, 3. + 1e-6], [0, 10., 0., 1, 2, 3., 7.],
        [0, 5., 5., 1, 1, 5.], [0, 0., 1., 1, 4, 0.25, 0.5, 0.75, 1.], [0, 0., 10., 4, 0],
        [0, 0., 10., 1, 1, 4., 6, 1, 5], [0, -PI, PI, 1, 5, -3., -1., 0., 1., 3., 3, 1, 2, -2., 2.],
        [0, 0., 10., 3],
    ]
    widths_fixed = {0: 2, 2: 0, 3: 0, 4: 1, 5: 0, 6: 1}
    out = []
    for script in scripts:
        count, k = 0, 0
        while k < len(script):
            op = int(script[k])
            k += 1 + (1 + int(script[k+1]) if op == 1 else widths_fixed[op])
            count += 1
        out.append(request('su_splitcurve', count, *script))
    return out


def append_shapeupgrade_requests(requests: list[str]) -> None:
    """Append every ShapeUpgrade request family in a fixed order."""
    requests += c0split_requests()
    requests += tool_requests()
    requests += removeloc_requests()
    requests += splitcurve_requests()
