"""Deterministic requests for TopOpeBRepTool_CurveTool
(tools/oracle/tkbool_probe_topopebreptool_curve_tool.cxx; the records are documented
there).

The square face [-2, 2]^2 on z = 1 cuts the lateral face of the cylinder of radius 1 on
the Z axis (z in [0, 2]) along the unit circle at z = 1. `tt_make_curves` approximates
polylines on that circle (with their exact UV on both faces): each plane case (the plane
first, second or not used), the 3d curve not asked, confused poles (the bad-poles
options), a p-curve off the face bounds, a partial index range, polylines of more than 100
poles whose close and aligned poles are dropped, and an edge given as face.
`tt_pcurve_on_face` projects circles and an ellipse (whole and on ranges) and lines (on
ranges) on both faces, and circle ranges across the cylinder seam.
"""
from __future__ import annotations
import math
from tktopalgo_corpus_brepcheck_shapes import PlanarShapes, cylinder_face
from tktopalgo_shape_program import ShapeProgram

TOL3D = 1.0e-6
TOL2D = 1.0e-8


def faces(p: ShapeProgram) -> dict:
    """The square on z = 1 (UV = (x + 2, y + 2)) and the cylinder (UV = (angle, z))."""
    shapes = PlanarShapes(p, [(-2.0, -2.0, 1.0), (2.0, -2.0, 1.0), (2.0, 2.0, 1.0), (-2.0, 2.0, 1.0)])
    square = shapes.face([[0, 1, 2, 3]], normal=(0.0, 0.0, 1.0))
    shapes.finish()
    cyl = cylinder_face(p, 1.0, 2.0)
    return {'plane': square, 'cyl': cyl['face'], 'edge': cyl['bottom']}


def circle_point(angle: float, v_cyl: float = 1.0) -> tuple:
    """The point of the circle at `angle` with its UV on the plane and the cylinder."""
    x, y = math.cos(angle), math.sin(angle)
    return (x, y, 1.0, x + 2.0, y + 2.0, angle, v_cyl)


def arc_points(n: int, a0: float = 0.3, a1: float = 2.5) -> list:
    """n points of the circle between the angles."""
    return [circle_point(a0 + (a1 - a0) * i / (n - 1)) for i in range(n)]


def pole_rich_points() -> list:
    """110 points of the circle with poles 2e-6 after some (dropped as too close), a spike
    whose next pole comes back 5e-10 near the kept one (kept off the line), and an exact
    return (the line cannot be made: the next pole is dropped)."""
    out = []
    for i, q in enumerate(arc_points(100)):
        out.append(q)
        a = 0.3 + 2.2 * i / 99
        if i % 10 == 3:
            out.append(circle_point(a + 2.0e-6))
        if i == 40:
            out.append(circle_point(a + 2.0e-5))
            out.append(circle_point(a + 5.0e-10))
        if i == 60:
            out.append(circle_point(a + 3.0e-5))
            out.append(q)
    return out


def make_curves_request(f1: str, f2: str, comps, points: list, rng=None, tols=(TOL3D, TOL2D, 30)) -> str:
    p = ShapeProgram()
    f = faces(p)
    parmin, parmax = rng or (1.0, float(len(points)))
    fields = ' '.join(repr(float(v)) for q in points for v in q)
    return (f"tt_make_curves {p.tokens()} {f[f1]} {f[f2]} {comps[0]} {comps[1]} {comps[2]} "
            f"{tols[0]!r} {tols[1]!r} {tols[2]} {parmin!r} {parmax!r} {len(points)} {fields}")


def pcurve_request(face: str, curve, rng=(0.0, 0.0)) -> str:
    p = ShapeProgram()
    f = faces(p)
    c = p.curve(*curve)
    return f"tt_pcurve_on_face {p.tokens()} {f[face]} {c} {rng[0]!r} {rng[1]!r}"


def append_make_curves_requests(requests: list[str]) -> None:
    pts = arc_points(12)
    for f1, f2 in (('plane', 'cyl'), ('cyl', 'plane')):
        for comps in ((1, 1, 1), (1, 1, 0), (1, 0, 1), (1, 0, 0), (0, 1, 1)):
            requests.append(make_curves_request(f1, f2, comps, pts))
    confused = pts[:5] + [pts[4]] + pts[5:]
    requests.append(make_curves_request('plane', 'cyl', (1, 1, 1), confused))
    off = [q[:6] + (5.0,) for q in pts]
    requests.append(make_curves_request('plane', 'cyl', (1, 1, 1), off))
    requests.append(make_curves_request('plane', 'cyl', (1, 1, 1), pts, (3.0, 9.0)))
    requests.append(make_curves_request('plane', 'cyl', (1, 1, 1), arc_points(40, 0.1, 6.0), None, (1.0e-4, 1.0e-6, 20)))
    rich = pole_rich_points()
    for f1, f2 in (('plane', 'cyl'), ('cyl', 'plane')):
        requests.append(make_curves_request(f1, f2, (1, 1, 1), rich))
    requests.append(make_curves_request('cyl', 'plane', (1, 1, 0), rich))
    requests.append(make_curves_request('plane', 'edge', (1, 1, 1), pts))
    requests.append(make_curves_request('plane', 'cyl', (1, 1, 1), arc_points(2)))
    requests.append(make_curves_request('plane', 'cyl', (1, 1, 1), arc_points(3)))


def append_pcurve_requests(requests: list[str]) -> None:
    circle = (1, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0)
    big = (1, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.5)
    line = (0, -1.0, 0.5, 1.0, 1.0, 0.0, 0.0)
    seam = (0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0)
    vertical = (0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0)
    ellipse = (2, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.5, 1.0)
    for face in ('plane', 'cyl'):
        for curve in (circle, big, ellipse):
            requests.append(pcurve_request(face, curve))
        # The lines only on ranges: the whole lines are infinite (the native step count
        # of the projection then converts an out-of-range real to int).
        for curve in (circle, big, line, seam, vertical, ellipse):
            requests.append(pcurve_request(face, curve, (0.5, 2.0)))
    for rng in ((math.pi + 0.5, 2.0 * math.pi + 1.0), (-3.0, -1.0), (7.0, 8.0), (2.0, 1.0)):
        requests.append(pcurve_request('cyl', circle, rng))
    requests.append(pcurve_request('edge', circle))


def append_curve_tool_requests(requests: list[str]) -> None:
    """The approximations of section curves and the p-curves on faces."""
    append_make_curves_requests(requests)
    append_pcurve_requests(requests)
