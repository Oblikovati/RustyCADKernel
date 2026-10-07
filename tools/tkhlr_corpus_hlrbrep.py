"""Deterministic requests for the HLRBRep curves and surfaces (tools/oracle/tkhlr_probe_hlrbrep_*.cxx).

The fixtures (tkhlr_probe_hlrbrep_fixtures.cxx) read a projector (`pk loc dir xdir
[focus]`: parallel, perspective or the default top view) and an edge without vertices
(`ck ...` geometry on a range): lines, circles and ellipses in every orientation to the
view (along the sight, edge-on, oblique), degree 1 and degree 3 Bezier curves and
degree 2 B-splines.

- `hb_curve`: HLRBRep_Curve Update with its 16-direction box, the 2D/3D parameters,
  depth, D0-D2 (with preset outputs), Tangent at both ends, the projected line,
  circle and ellipse, and the projected poles, weights, knots and multiplicities.
- `hb_cinter`: HLRBRep_CurveTool (samples, C1 intervals, types), HLRBRep_TheProjPCurOfCInter
  and HLRBRep_CInter (every helper instance: conic-conic, conic-curve, polygon-polygon and
  self intersection) on pairs of projected edges.
- `hb_surface`: HLRBRep_Surface (type, IsSide, Plane, IsAbove), HLRBRep_SurfaceTool
  sample counts, HLRBRep_SLProps normals and curvatures, and HLRBRep_InterCSurf of sight
  lines (Perform(L, S) and the polygon/polyhedron form of HLRBRep_Intersector) on planes,
  cylinders, cones, spheres, tori, Bezier, B-spline, revolution and extrusion faces.
- `hb_edgedata`, `hb_facedata`, `hb_edgeface`: HLRBRep_EdgeData (Set, flags, status),
  HLRBRep::MakeEdge and MakeEdge3d, HLRBRep_FaceData with HLRBRep_FaceIterator, and
  HLRBRep_EdgeFaceTool (curvature, UVPoint by p-curve and by projection) with
  PolyHLRAngleAndDeflection.
- `ht_outliner`: HLRTopoBRep_OutLiner::Fill (DSFiller with Contap contours, FaceIsoLiner,
  Data, ProcessFace) on boxes, cylinders, cones, spheres, tori, faces and compounds:
  the outlined topology, every edge (vertices, range, type, points) and the data
  structure queries.
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


def cinter_requests(rng: random.Random) -> list[str]:
    """hb_cinter over random pairs of edges under one projector, in the four modes."""
    requests = []
    for index in range(800):
        proj = projector(rng)
        view = tuple(proj[4:7]) if proj[0] != 2 else (0.0, 0.0, 1.0)
        first, _, _ = edge(rng, view)
        second, _, _ = edge(rng, view)
        tol = rng.choice((1e-7, 1e-6, 1e-4))
        requests.append('hb_cinter ' + numbers(proj + first + second + [1e-9, tol, index % 4]))
    return requests


def grid(rng: random.Random, nu: int, nv: int, flat: bool) -> list[float]:
    """`nu nv` poles on a perturbed grid (in the plane z = 0 when `flat`)."""
    fields: list[float] = [nu, nv]
    for i in range(nu):
        for j in range(nv):
            z = 0.0 if flat else round(rng.uniform(-1, 1), 3)
            fields += [i * 2.0 - 2.0, j * 2.0 - 2.0, z]
    return fields


def surface(rng: random.Random, kind: int, view: tuple[float, float, float]) -> list[float]:
    """`sk ...` and its bounds `umin umax vmin vmax`."""
    two_pi = 2 * math.pi
    if kind == 5:
        nu, nv = rng.choice(((2, 2), (3, 3), (4, 3)))
        return [5, *grid(rng, nu, nv, rng.random() < 0.3), 0.0, 1.0, 0.0, 1.0]
    if kind == 6:
        nu, nv = rng.choice(((3, 3), (4, 4), (5, 3)))
        return [6, *grid(rng, nu, nv, rng.random() < 0.3), 0.0, nu - 2.0, 0.0, nv - 2.0]
    if kind == 7:
        return [7, *point(rng, 2.0), *rng.choice(VIEW_DIRS), *point(rng, 1.0), *rng.choice(VIEW_DIRS + [view]),
                0.0, rng.choice((math.pi, two_pi)), 0.0, 3.0]
    n = rng.choice(VIEW_DIRS + [view, view])
    head = [kind, *point(rng, 2.0), *n, *x_direction(n)]
    if kind == 0:
        return head + [-3.0, 3.0, -2.0, 2.5]
    if kind == 1:
        return head + [rng.choice((1.0, 2.0)), 0.0, rng.choice((math.pi, two_pi)), -2.0, 2.0]
    if kind == 2:
        return head + [rng.choice((0.3, 0.6)), 1.0, 0.0, two_pi, 0.0, 3.0]
    if kind == 3:
        return head + [2.0, 0.0, rng.choice((math.pi, two_pi)), -math.pi / 2, rng.choice((0.5, math.pi / 2))]
    if kind == 4:
        return head + [3.0, 1.0, 0.0, two_pi, 0.0, two_pi]
    return head + [1.5, *rng.choice(VIEW_DIRS + [view]), 0.0, two_pi, 0.0, 3.0]


def surface_requests(rng: random.Random) -> list[str]:
    """hb_surface over every surface kind, random projectors, curves and sight lines."""
    requests = []
    for index in range(900):
        proj = projector(rng)
        view = tuple(proj[4:7]) if proj[0] != 2 else (0.0, 0.0, 1.0)
        fields = surface(rng, index % 9, view)
        tols = list(rng.choice(((1e-7, 1e-7), (1e-3, 1e-3), (0.1, 1e-4))))
        params = [round(rng.uniform(0, 1), 3) for _ in range(4)]
        curve, _, _ = edge(rng, view)
        sights: list[float] = [3]
        for _ in range(3):
            sights += [round(rng.uniform(-4, 4), 3), round(rng.uniform(-4, 4), 3), rng.choice((0, 1)), -100.0, 100.0]
        requests.append('hb_surface ' + numbers(proj + fields + tols + params + curve
                                                 + [rng.choice((0, 1)), rng.choice((1e-7, 0.01))] + sights))
    return requests


def edgedata_requests(rng: random.Random) -> list[str]:
    """hb_edgedata: EdgeData::Set and the flags, MakeEdge and MakeEdge3d."""
    requests = []
    for _ in range(500):
        proj = projector(rng)
        view = tuple(proj[4:7]) if proj[0] != 2 else (0.0, 0.0, 1.0)
        curve, u1, u2 = edge(rng, view)
        flags = [rng.choice((0, 1)) for _ in range(9)]
        tols = [u1, rng.choice((1e-7, 1e-4)), u2, rng.choice((1e-7, 0.01))]
        requests.append('hb_edgedata ' + numbers(proj + curve + flags + tols))
    return requests


def facedata_requests(rng: random.Random) -> list[str]:
    """hb_facedata: FaceData::Set, SetWire, SetWEdge (some out of range) and the walk."""
    requests = []
    for index in range(300):
        fields = surface(rng, index % 9, (0.0, 0.0, 1.0))
        nw = rng.choice((1, 2, 3))
        counts = [rng.choice((1, 2, 4)) for _ in range(nw)]
        fields += [rng.randrange(4), rng.choice((0, 1)), nw, *counts]
        for wi, ne in enumerate(counts, 1):
            for ewi in range(1, ne + 1 + (1 if rng.random() < 0.05 else 0)):
                fields += [wi, ewi, rng.randrange(1, 20), rng.randrange(4)] + [rng.choice((0, 1)) for _ in range(4)]
        requests.append('hb_facedata ' + numbers(fields + [-1]))
    return requests


def edgeface_requests(rng: random.Random) -> list[str]:
    """hb_edgeface: the curvature along a direction, UVPoint of a face edge and of a free
    edge, and PolyHLRAngleAndDeflection."""
    requests = []
    for index in range(400):
        proj = projector(rng)
        fields = surface(rng, index % 9, (0.0, 0.0, 1.0))
        u1, u2, v1, v2 = fields[-4:]
        uv = [u1 + (u2 - u1) * rng.random(), v1 + (v2 - v1) * rng.random()]
        d = unit(rng.choice(VIEW_DIRS))
        par = rng.choice((0.0, 0.3, 0.5, 1.0, 0.001, 0.7))
        curve, _, _ = edge(rng, (0.0, 0.0, 1.0))
        requests.append('hb_edgeface ' + numbers(proj + fields + uv + list(d) + [par] + curve))
    return requests


def solid(rng: random.Random, kind: int) -> list[float]:
    """`hk ...` of a primitive: box, cylinder, cone, sphere or torus on a random frame."""
    n = unit(rng.choice(VIEW_DIRS))
    head = [kind, *point(rng, 2.0), *n, *x_direction(n)]
    sizes = {0: [rng.choice((1.0, 2.0)), rng.choice((1.5, 3.0)), rng.choice((1.0, 2.5))],
             1: [rng.choice((1.0, 2.0)), rng.choice((1.0, 3.0))],
             2: [rng.choice((1.0, 2.0)), rng.choice((0.0, 0.5)), rng.choice((1.0, 3.0))],
             3: [rng.choice((1.0, 2.5))],
             4: [3.0, rng.choice((0.5, 1.0))]}
    return head + sizes[kind]


def shape(rng: random.Random, kind: int) -> list[float]:
    """`hk ...`: a primitive, a face, or a compound of faces or primitives."""
    if kind < 5:
        return solid(rng, kind)
    if kind == 6:
        return [6, *surface(rng, rng.randrange(9), (0.0, 0.0, 1.0))]
    if kind == 5:
        fields: list[float] = [5, 2]
        for _ in range(2):
            fields += surface(rng, rng.randrange(9), (0.0, 0.0, 1.0))
        return fields
    return [7, 2, *solid(rng, rng.randrange(5)), *solid(rng, rng.randrange(5))]


def outliner_requests(rng: random.Random) -> list[str]:
    """ht_outliner: HLRTopoBRep_OutLiner::Fill on primitives, faces and compounds."""
    requests = []
    for index in range(240):
        proj = projector(rng, rng.choice((0, 0, 1)))
        if proj[0] == 1:
            proj[-1] = rng.choice((30.0, 60.0))
        requests.append('ht_outliner ' + numbers(proj + shape(rng, index % 8) + [rng.choice((0, 0, 2, 3))]))
    return requests


def intersector_requests(rng: random.Random) -> list[str]:
    """hb_intersector: self and pair intersections of edges (the second edge often the
    first one, so that the ends meet), a simulated point and sight lines on a face."""
    requests = []
    for index in range(600):
        proj = projector(rng)
        view = tuple(proj[4:7]) if proj[0] != 2 else (0.0, 0.0, 1.0)
        first, u1, u2 = edge(rng, view)
        second = first if index % 3 == 0 else edge(rng, view)[0]
        ends = [rng.choice((0.0, 0.0, 0.01, 0.1)) for _ in range(4)]
        params = [u1 + (u2 - u1) * rng.random(), u1 + (u2 - u1) * rng.random()]
        sights: list[float] = []
        for _ in range(3):
            sights += [round(rng.uniform(-4, 4), 3), round(rng.uniform(-4, 4), 3), rng.choice((-100.0, 0.0, 5.0, 100.0))]
        requests.append('hb_intersector ' + numbers(proj + first + second + ends + [rng.choice((0, 1))] + params
                                                     + surface(rng, index % 9, view) + sights))
    return requests


def data_requests(rng: random.Random) -> list[str]:
    """hb_data: ShapeToHLR::Load, Data::Update and the hiding walk of every hiding face
    on primitives, faces and compounds, parallel and perspective."""
    requests = []
    for index in range(320):
        proj = projector(rng, rng.choice((0, 0, 1)))
        if proj[0] == 1:
            proj[-1] = rng.choice((30.0, 60.0))
        requests.append('hb_data ' + numbers(proj + shape(rng, index % 8) + [rng.choice((0, 0, 2))]))
    return requests


def hider_requests(rng: random.Random) -> list[str]:
    """hb_hider: HLRBRep_Hider::Hide by every hiding face, then the visible parts."""
    requests = []
    for index in range(400):
        proj = projector(rng, rng.choice((0, 0, 1)))
        if proj[0] == 1:
            proj[-1] = rng.choice((30.0, 60.0))
        requests.append('hb_hider ' + numbers(proj + shape(rng, index % 8) + [rng.choice((0, 0, 2))]))
    return requests


def append_hlrbrep_requests(requests: list[str]) -> None:
    """Append every HLRBRep request family in a fixed order."""
    rng = random.Random(0x41B12E9)
    requests.extend(curve_requests(rng))
    requests.extend(cinter_requests(rng))
    requests.extend(surface_requests(rng))
    requests.extend(edgedata_requests(rng))
    requests.extend(facedata_requests(rng))
    requests.extend(edgeface_requests(rng))
    requests.extend(outliner_requests(rng))
    requests.extend(intersector_requests(rng))
    requests.extend(data_requests(rng))
    requests.extend(hider_requests(rng))
