"""Deterministic requests for IntCurvesFace (tools/oracle/tktopalgo_probe_intcurvesface.cxx).

Every request carries its face records (tktopalgo_corpus_adaptor_faces.py), so it never
relies on an earlier request:
- `icf_line`: one IntCurvesFace_Intersector per request on groups of lines through
  points of the face (inside, on edges, at vertices, just outside, at seams and poles),
  tangent lines, lines in or parallel to a plane, seeded lines, on infinite, finite and
  short ranges, with and without the boundary tolerances and the restriction;
- `icf_curve`: the same on circles, segments and Bezier curves;
- `icf_shape`: IntCurvesFace_ShapeIntersector on compounds of faces with Perform,
  PerformNearest (repeated, so the face order adapts) and Perform on a curve adaptor,
  after one or two Loads.
"""
from __future__ import annotations
import math
import random
from tktopalgo_corpus_adaptor_faces import ADAPTOR_WINDOWS, adaptor_faces
from tktopalgo_corpus_faces import CONE, CYLINDER, PLANE, SPHERE, TORUS, TRIMMED, FaceSpec

REAL_LAST = 1.7976931348623157e308
TWO_PI = 2.0 * math.pi
# The faces whose surface point of (u, v) this module can compute.
ANALYTIC = (PLANE, TRIMMED, CYLINDER, SPHERE, CONE, TORUS)


def numbers(values: list[float]) -> str:
    """The fields of a request, with every double spelled exactly."""
    return ' '.join(repr(float(v)) for v in values)


def surface_point(face: FaceSpec, u: float, v: float) -> list[float]:
    """The point of an analytic face surface at (u, v) (frames about the z axis)."""
    kind, fields = face.surface[0], face.surface[7:]
    if kind == CYLINDER:
        return [fields[0] * math.cos(u), fields[0] * math.sin(u), v]
    if kind == SPHERE:
        r = fields[0]
        return [r * math.cos(v) * math.cos(u), r * math.cos(v) * math.sin(u), r * math.sin(v)]
    if kind == CONE:
        radius, angle = fields
        rho = radius + v * math.sin(angle)
        return [rho * math.cos(u), rho * math.sin(u), v * math.cos(angle)]
    if kind == TORUS:
        major, minor = fields
        rho = major + minor * math.cos(v)
        return [rho * math.cos(u), rho * math.sin(u), minor * math.sin(v)]
    return [u, v, 0.0]


def unit(vector: list[float]) -> list[float]:
    """The vector scaled to length 1."""
    length = math.sqrt(sum(c * c for c in vector))
    return [c / length for c in vector]


def aimed_line(point: list[float], direction: list[float], reach: float = 6.0) -> list[float]:
    """`P D`: the line through `point` along `direction`, starting `reach` before it."""
    d = unit(direction)
    return [*(p - reach * c for p, c in zip(point, d)), *d]


def ranges(index: int, reach: float = 6.0) -> tuple[float, float]:
    """The line parameter range of the index-th line: infinite, finite around the
    aimed point, stopping just before it, or starting beyond it."""
    return ((-REAL_LAST, REAL_LAST), (0.0, 2 * reach), (0.0, reach - 1e-3), (reach + 1e-3, REAL_LAST),
            (-1e-8, 1e100))[index % 5]


def analytic_lines(face: FaceSpec, window: tuple[float, float, float, float], rng: random.Random) -> list[list[float]]:
    """Lines through points of an analytic face: a UV grid, the vertices, and seeded
    points, along seeded directions, plus the vertical and horizontal lines."""
    u0, u1, v0, v1 = window
    du, dv = 0.1 * (u1 - u0), 0.1 * (v1 - v0)
    uv = [(u0 + (u1 - u0) * i / 4, v0 + (v1 - v0) * j / 4) for i in range(5) for j in range(5)]
    uv += [(u, v) for u, v, _ in face.vertices]
    uv += [(rng.uniform(u0 - du, u1 + du), rng.uniform(v0 - dv, v1 + dv)) for _ in range(10)]
    lines = []
    for index, (u, v) in enumerate(uv):
        point = surface_point(face, u, v)
        direction = [(0.0, 0.0, 1.0), (1.0, 0.0, 0.0), (0.3, -0.5, 0.8), (0.0, 1.0, 0.0)][index % 4]
        if index % 3 == 2:
            direction = (rng.uniform(-1, 1), rng.uniform(-1, 1), rng.uniform(-1, 1))
        lines.append([*aimed_line(point, list(direction)), *ranges(index)])
    return lines


def special_lines(face: FaceSpec) -> list[list[float]]:
    """Tangent, in-plane and axial lines of each surface kind."""
    kind, fields = face.surface[0], face.surface[7:]
    full = [-REAL_LAST, REAL_LAST]
    if kind in (PLANE, TRIMMED):
        return [[-5.0, 1.0, 0.0, 1.0, 0.0, 0.0, *full], [-5.0, 1.0, 1.0, 1.0, 0.0, 0.0, *full],
                [0.5, -3.0, 0.0, 0.0, 1.0, 0.0, 0.0, 10.0], [1.0, 1.0, 0.0, 0.0, 0.0, 1.0, *full]]
    if kind == CYLINDER:
        r = fields[0]
        return [[r, -5.0, 2.0, 0.0, 1.0, 0.0, *full], [5.0, 0.0, 2.0, -1.0, 0.0, 0.0, *full],
                [r, 0.0, -1.0, 0.0, 0.0, 1.0, *full], [0.0, 0.0, -1.0, 0.0, 0.0, 1.0, *full]]
    if kind == SPHERE:
        r = fields[0]
        return [[0.0, 0.0, -5.0, 0.0, 0.0, 1.0, *full], [-5.0, 0.0, r, 1.0, 0.0, 0.0, *full],
                [-5.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 10.0], [-5.0, -5.0, 0.3, 1.0, 1.0, 0.0, *full]]
    if kind == TORUS:
        major, minor = fields
        return [[-6.0, 0.0, 0.0, 1.0, 0.0, 0.0, *full], [major, 0.0, -3.0, 0.0, 0.0, 1.0, *full],
                [-6.0, 0.0, minor, 1.0, 0.0, 0.0, *full], [0.0, 0.0, -3.0, 0.0, 0.0, 1.0, *full]]
    return [[0.0, 0.0, -5.0, 0.0, 0.0, 1.0, *full], [-6.0, 0.0, 1.0, 1.0, 0.0, 0.0, *full]]


def spline_lines(window: tuple[float, float, float, float], extent: tuple[float, float], rng: random.Random) -> list[list[float]]:
    """Vertical and tilted lines over the xy extent of a spline patch, and seeded ones."""
    lines = []
    for i in range(5):
        for j in range(5):
            x, y = extent[0] * (i + 0.5) / 5, extent[1] * (j + 0.5) / 5
            lines.append([x, y, -5.0, 0.0, 0.0, 1.0, *ranges(i + j)])
            lines.append([x - 1.0, y - 0.5, -3.0, 0.2, 0.1, 0.6, *ranges(i * j)])
    for _ in range(10):
        point = [rng.uniform(0, extent[0]), rng.uniform(0, extent[1]), rng.uniform(-1, 1)]
        direction = [rng.uniform(-1, 1), rng.uniform(-1, 1), rng.uniform(0.2, 1)]
        lines.append([*aimed_line(point, direction), *ranges(rng.randrange(5))])
    return lines


# The xy extent of the pole grids of the spline faces.
SPLINE_EXTENTS = {'bezier_surface': (3.0, 3.0), 'bspline': (6.0, 4.0), 'bspline_c1': (6.0, 4.0),
                  'bspline_holed': (6.0, 4.0)}


def boundary_lines() -> list[list[float]]:
    """Vertical lines just outside the square [0, 2]^2 (within and beyond 0.1), on its
    edges and at its corners, for the boundary tolerance of IntCurvesFace."""
    full = [-REAL_LAST, REAL_LAST]
    points = [(-0.05, 1.0), (2.05, 1.0), (1.0, -0.09), (1.0, 2.2), (0.0, 1.0), (2.0, 2.0), (-0.07, -0.07),
              (1.0, 1.0), (2.11, 0.5), (0.5, 1.95)]
    return [[u, v, -1.0, 0.0, 0.0, 1.0, *full] for u, v in points]


def face_lines(name: str, face: FaceSpec, rng: random.Random) -> list[list[float]]:
    """Every line request of one face."""
    window = ADAPTOR_WINDOWS[name]
    if name in SPLINE_EXTENTS:
        return spline_lines(window, SPLINE_EXTENTS[name], rng)
    lines = analytic_lines(face, window, rng) + special_lines(face)
    if name.startswith('toleranced'):
        lines += boundary_lines()
    return lines


def line_request(face: FaceSpec, tol: float, restr: int, usebtol: int, lines: list[list[float]]) -> str:
    """One `icf_line` request on a group of lines."""
    return 'icf_line ' + numbers([*face.numbers(), tol, restr, usebtol, len(lines), *[c for line in lines for c in line]])


def append_line_requests(requests: list[str], faces: dict[str, FaceSpec]) -> None:
    """`icf_line` on every face except the one without pcurve (a crash natively)."""
    rng = random.Random(8821)
    settings = ((1e-7, 1, 1), (1e-3, 1, 0), (1e-7, 0, 1), (0.0, 1, 1))
    for name, face in faces.items():
        if name == 'no_pcurve':
            continue
        lines = face_lines(name, face, rng)
        for index in range(0, len(lines), 6):
            tol, restr, usebtol = settings[(index // 6) % len(settings)]
            requests.append(line_request(face, tol, restr, usebtol, lines[index:index + 6]))


def curves(face: FaceSpec) -> list[list[float]]:
    """`kind fields... first last` curves crossing the face: circles, segments and a
    Bezier curve through its region."""
    kind = face.surface[0]
    if kind in (PLANE, TRIMMED):
        centre = [1.0, 1.0, 0.0]
    elif kind in ANALYTIC:
        centre = [0.0, 0.0, 1.0]
    else:
        centre = [1.5, 1.5, 0.0]
    x, y, z = centre
    return [[1, x, y, z, 1.0, 0.0, 0.0, 0.6, 0.0, TWO_PI],
            [1, x, y, z, 0.0, 1.0, 0.0, 2.5, 0.0, TWO_PI],
            [1, x, y, z, 0.3, 0.2, 0.9, 1.9, 0.5, 4.0],
            [0, x - 4.0, y, z - 0.5, 1.0, 0.0, 0.2, 0.0, 8.0],
            [2, 4, x - 3.0, y - 1.0, z - 2.0, x - 1.0, y + 2.0, z + 3.0, x + 1.0, y - 2.0, z - 3.0,
             x + 3.0, y + 1.0, z + 2.0, 0.0, 1.0]]


def append_curve_requests(requests: list[str], faces: dict[str, FaceSpec]) -> None:
    """`icf_curve` on every face except the one without pcurve."""
    for name, face in faces.items():
        if name == 'no_pcurve':
            continue
        for index, curve in enumerate(curves(face)):
            pmin, pmax = ((-REAL_LAST, REAL_LAST), (0.2, 3.0), (-1.0, 0.5))[index % 3]
            usebtol = index % 2
            requests.append('icf_curve ' + numbers([*face.numbers(), 1e-7, 1, usebtol, *curve, pmin, pmax]))


SHAPES = (('square', 'holed', 'disc'), ('cylinder', 'cone', 'sphere', 'torus'),
          ('bspline', 'bezier_surface', 'torus_patch'), ('toleranced_3d', 'square_reversed'),
          ('bspline_c1', 'bspline_holed', 'trimmed', 'internal'))


def append_shape_requests(requests: list[str], faces: dict[str, FaceSpec]) -> None:
    """`icf_shape` on compounds of faces, in every mode, after one or two Loads."""
    rng = random.Random(5150)
    for names in SHAPES:
        lines = []
        for name in names:
            lines += face_lines(name, faces[name], rng)[:8]
        rng.shuffle(lines)
        record = [len(names), *[v for name in names for v in faces[name].numbers()]]
        for mode in (0, 1, 2):
            for reload in (0, 1):
                group = lines[:10] if mode != 2 else [line for line in lines if max(abs(line[6]), abs(line[7])) < 1e100][:10]
                requests.append('icf_shape ' + numbers([*record, 1e-7, reload, mode, len(group),
                                                        *[c for line in group for c in line]]))


def append_intcurvesface_requests(requests: list[str]) -> None:
    """Append every IntCurvesFace request family in a fixed order."""
    faces = adaptor_faces()
    append_line_requests(requests, faces)
    append_curve_requests(requests, faces)
    append_shape_requests(requests, faces)
