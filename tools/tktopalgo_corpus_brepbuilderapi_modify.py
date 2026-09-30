"""Deterministic requests for the BRepBuilderAPI modifications
(tools/oracle/tktopalgo_probe_brepbuilderapi_modify.cxx).

The shapes are the shape records of the BRepLib probes: faces with only curves on surface,
BRepLib_MakeShell shells of every surface kind, edges with enlarged vertices, solids and
compounds, faces made on every surface2 kind (offsets, extrusions, revolutions) and
polygons. `bba_transform` runs every transformation kind (translation, rotation, scale,
point, axis and plane mirrors, frame change, products, identity) with and without the
geometry copy, meshed or not; `bba_gtransform` general transformations (affinities and
matrices); `bba_copy` the copy with and without geometry and mesh; `bba_nurbs` the
conversion of every surface kind; `bba_modify` the protected ModifyShape paths, including
the Standard_NullObject ones; `bba_findplane` planar and non-planar shapes of every curve
kind; `bba_collect` chains of modifications.
"""
from __future__ import annotations
import math
from tktopalgo_corpus_breplib_face import made_face, polygons, surfaces2
from tktopalgo_corpus_breplib_package import compound, edge, edges, pcurve_faces, shells, solids
from tktopalgo_corpus_breplib_records import geom_bezier, geom_bspline, geom_gp, geom_offset, geom_trimmed, numbers

UNIT = (0.1, 0.9, 0.1, 0.9)
PI = math.pi


def request(op: str, fields: list[float]) -> str:
    return f'bba_{op} ' + numbers(fields)


def transformations() -> list[list[float]]:
    """One `trsf` record of each kind."""
    return [[0, 1., -2., 0.5], [1, 0., 0., 0., 0., 0., 1., 0.7], [1, 1., 2., 3., 0.6, 0.8, 0., PI],
            [2, 1., 0., 0., 2.5], [2, 0., 0., 0., -1.], [3, 0.5, 0.5, 0.5], [4, 0., 0., 0., 1., 0., 0.],
            [5, 0., 0., 1., 0., 0., 1.], [6, 1., 1., 1., 0., 1., 0.],
            [7, 1, 0., 0., 0., 0., 0., 1., 0.3, 2, 0., 0., 0., 3.], [8], [2, 0., 0., 0., 1. + 1e-15]]


def general_transformations() -> list[list[float]]:
    """One `gtrsf` record of each kind."""
    return [[0, 0, 1., 2., 3.], [0, 1, 0., 0., 0., 1., 0., 0., 0.4], [1, 0., 0., 0., 0., 0., 1., 2.],
            [2, 0., 0., 0., 0., 0., 1., 0.5], [3, 1., 0.2, 0., 0., 1., 0.1, 0., 0., 2., 1., 1., 1.],
            [3, 2., 0., 0., 0., 2., 0., 0., 0., 2., 0., 0., 0.]]


def curve_edges() -> list[list[float]]:
    """Edges of every curve kind of the Geom curve record."""
    circle = geom_gp(1)
    return [edge(geom_gp(2), 0., 2.), edge(geom_gp(3), -1., 1.), edge(geom_gp(4), -1., 1.),
            edge(geom_offset(0.5, (0., 0., 1.), circle), 0., 3.),
            edge(geom_bspline(2, False, [(0., 0., 0.), (1., 1., 0.5), (2., 0., 0.), (3., 1., 0.)],
                              [(0., 3), (0.5, 1), (1., 3)]), 0., 1.),
            edge(geom_bezier([(0., 0., 0.), (1., 1., 0.), (2., 0., 0.)]), 0., 1.),
            edge(geom_trimmed(0., 2., geom_gp(1, p=(0., 0., 1.), n=(1., 0., 0.))), 0.5, 1.5)]


def made_faces() -> list[list[float]]:
    """Faces on every surface2 kind over a parameter box inside every domain."""
    return [made_face(s, UNIT) for s in surfaces2()]


def base_shapes() -> list[list[float]]:
    """The shapes most requests run on: faces, shells, edges, solids and compounds."""
    faces = pcurve_faces()
    return ([faces[0], faces[1], faces[3], faces[4], faces[12], faces[15], faces[17]] + shells()[:4]
            + [shells()[7], edges()[0], edges()[3], edges()[5], solids()[0], solids()[6],
               compound([faces[0], edges()[2]])])


def convertible_shapes() -> list[list[float]]:
    """The base shapes without the cylinder, sphere and cone faces of the face reader, whose
    edges have only curves on surface: BRepTools_NurbsConvertModification::NewCurve2d passes
    their null 3d curve to BRepTools::EvalAndUpdateTol, a segmentation fault natively, so
    NurbsConvert and GTransform (which runs it first) never get them."""
    shapes = base_shapes()
    return shapes[:2] + shapes[5:]


def transform_requests() -> list[str]:
    requests = []
    for shape in base_shapes():
        for trsf in transformations():
            requests += [request('transform', shape + [0] + trsf + [0, 0, 0]),
                         request('transform', shape + [0] + trsf + [1, 0, 0])]
    for shape in base_shapes()[:5]:
        for trsf in transformations()[:6]:
            requests += [request('transform', shape + [3] + trsf + [0, 1, 1]),
                         request('transform', shape + [3] + trsf + [1, 1, 2]),
                         request('transform', shape + [2] + trsf + [1, 0, 1])]
    return requests


def gtransform_requests() -> list[str]:
    requests = []
    for shape in convertible_shapes():
        requests += [request('gtransform', shape + g + [0, 0]) for g in general_transformations()]
    for shape in convertible_shapes()[:4]:
        requests += [request('gtransform', shape + general_transformations()[2] + [1, 1]),
                     request('gtransform', shape + general_transformations()[4] + [0, 2])]
    return requests


def copy_requests() -> list[str]:
    requests = []
    for shape in base_shapes() + made_faces()[:6] + curve_edges()[:3]:
        requests += [request('copy', shape + [0, 1, 0, 0]), request('copy', shape + [0, 0, 0, 0])]
    for shape in base_shapes()[:8]:
        requests += [request('copy', shape + [2, 1, 1, 0]), request('copy', shape + [2, 0, 1, 0]),
                     request('copy', shape + [2, 1, 0, 1]), request('copy', shape + [0, 0, 1, 2])]
    return requests


def nurbs_requests() -> list[str]:
    requests = [request('nurbs', shape + [0, 0]) for shape in convertible_shapes() + made_faces() + curve_edges()]
    for shape in convertible_shapes()[:5]:
        requests += [request('nurbs', shape + [1, 1]), request('nurbs', shape + [0, 2])]
    return requests


def modify_requests() -> list[str]:
    return [request('modify', shape + [mode]) for shape in base_shapes()[:6] for mode in range(8)]


def findplane_requests() -> list[str]:
    shapes = polygons() + base_shapes() + curve_edges() + made_faces()[:3] + [
        compound(curve_edges()[:3]), compound([edges()[0], edges()[1]]), compound([edges()[3], curve_edges()[0]])]
    return [request('findplane', shape + [tol]) for shape in shapes for tol in (-1., 0., 1e-3, 0.5)]


def collect_requests() -> list[str]:
    """Chains of steps: modifications (0 to 3), AddGenerated (4) of a sub-shape image, AddModif
    (5) of an image into itself (an image never lands in two lists of one map, whose
    iteration order follows the TShape addresses), and Filter (6)."""
    chains = [[[0]], [[1]], [[2]], [[3]], [[0], [1]], [[1], [2]], [[3], [0]], [[0], [1], [2]], [[0], [6]],
              [[1], [2], [6]], [[0], [4, 2, 3], [2]], [[3], [5, 2, 2], [3]], [[2], [5, 3, 3], [1], [6]],
              [[1], [4, 1, 2], [6]]]
    requests = []
    for shape in convertible_shapes()[:4] + [base_shapes()[12]]:
        for chain in chains:
            requests.append(request('collect', shape + [len(chain)] + [c for step in chain for c in step]))
    return requests


def append_brepbuilderapi_modify_requests(requests: list[str]) -> None:
    """Append the modification requests in a fixed order."""
    requests += transform_requests()
    requests += gtransform_requests()
    requests += copy_requests()
    requests += nurbs_requests()
    requests += modify_requests()
    requests += findplane_requests()
    requests += collect_requests()
