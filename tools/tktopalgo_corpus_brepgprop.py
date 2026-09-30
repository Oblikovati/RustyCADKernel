"""Deterministic requests for BRepGProp (tools/oracle/tktopalgo_probe_brepgprop.cxx and
tktopalgo_probe_brepgprop_tools.cxx).

Every request carries its shape record (tktopalgo_corpus_shapes.py; named shapes of
tktopalgo_corpus_solids.py, tktopalgo_corpus_brepbndlib.py, tktopalgo_corpus_brepclass3d.py
and tktopalgo_corpus_gprop_shapes.py), so it never relies on an earlier request:
- `bgp_package`: LinearProperties, SurfaceProperties, VolumeProperties (fixed and
  adaptive) and VolumePropertiesGK (by point and by plane) with their flags, on solids
  (boxes with tolerances, polygons and meshes, hollow, open, located and rotated boxes,
  cylinders, spheres, a torus, a cone frustum, a ring with holed end faces, cavities,
  compounds, a solid without shells, shared solids moved, reversed, mirrored and
  scaled), faces, free edges and meshes;
- `bgp_cinert`, `bgp_mesh_cinert`, `bgp_mesh_nodes`: the edge tool, Cinert and
  MeshCinert (PreparePolygon from 3d polygons, polygons on triangulations and none; node
  arrays indexed from 1 and from elsewhere);
- `bgp_mesh_props`, `bgp_mesh_calc`: MeshProps with identity, rigid, mirrored and
  scaled locations, both orientations and both types; CalculateProps with degenerate
  triangles and short rules;
- `bgp_sinert`, `bgp_vinert`, `bgp_vinert_gk`: every constructor and Perform, chained on
  one framework, on single faces of every surface kind;
- `bgp_face`, `bgp_knots`, `bgp_arrays`, `bgp_domain`, `bgp_gauss`, `bgp_functions`: the
  face tool, its knot spans on arbitrary ranges and short arrays, the domain past its
  end, each Gauss Compute overload (and the type assertion), the integrand functions.
"""
from __future__ import annotations
from tktopalgo_corpus_brepbndlib import (
    bezier_patch, empty_solid, face_located_box, free_edges, infinite_plane, located,
    mesh_edge_only, mesh_only_face, meshed_box_face, numbers, single, sphere_patch)
from tktopalgo_corpus_brepclass3d import (
    cylinder_in_box, flat_box, flipped_face_box, half_space, inner_first, internal_shell_first,
    spline_top_box, square_sheet, two_boxes, two_cavities)
from tktopalgo_corpus_gprop_shapes import (
    curve_edges, ellipse_disc, extrusion_face, frustum, holed_plate, meshed_reversed_box,
    offset_face, pcurve_only_edge, revolution_face, ring, shared_solids, side_face, spline_blob,
    spline_patch, wide_spline_face)
from tktopalgo_corpus_shapes import (
    BEZIER_SURFACE, FACE, FORWARD, IDENTITY, PLANE, REVERSED, SHELL, ShapeSpec, point_mirror,
    rotation, scaling, translation)
from tktopalgo_corpus_solids import box, cylinder, hollow_box, open_box, sphere, sphere_in_box, torus


# (Eps, OnlyClosed, IsUseSpan, CGFlag, IFlag, SkipShared) of the VolumePropertiesGK
# requests; the tight tolerances integrate the volume alone, so that each request stays
# fast in the debug build of the Rust twin.
GK_OPTIONS = [(0.001, 0, 0, 0, 0, 0), (1e-5, 0, 1, 0, 0, 0), (0.05, 0, 1, 1, 1, 0),
              (0.05, 1, 0, 1, 0, 1), (0.1, 1, 1, 0, 1, 1)]


def request(op: str, spec: ShapeSpec, *fields: float) -> str:
    """One request line: the operation, the shape record and the fields."""
    return f'{op} {numbers([*spec.numbers(), *fields])}'


def solids() -> list[ShapeSpec]:
    """The solids and compounds of the package requests."""
    return [
        box([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]), box([0.0, 0.0, 0.0], [10.0, 20.0, 30.0]),
        box([-1.0, 2.0, 0.5], [3.0, 2.5, 7.0], tol=1e-3),
        box([0.0, 0.0, 0.0], [4.0, 3.0, 2.0], polygon=0.02, mesh=0.1),
        hollow_box([0.0, 0.0, 0.0], [10.0, 10.0, 10.0], [2.0, 3.0, 4.0], [6.0, 7.0, 8.0]),
        open_box([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]), cylinder([0.0, 0.0, 0.0], 3.0, 7.0),
        cylinder([1.0, -2.0, 0.5], 10.0, 20.0), sphere([0.0, 0.0, 0.0], 5.0), sphere([1.0, 2.0, 3.0], 0.5),
        torus([0.0, 0.0, 0.0], 3.0, 1.0), sphere_in_box([0.0, 0.0, 0.0], [10.0, 10.0, 10.0], [5.0, 5.0, 5.0], 2.0),
        frustum(2.0, 1.0, 3.0), frustum(1.0, 2.5, 1.5), ring(3.0, 1.5, 2.0), flipped_face_box(), two_boxes(),
        inner_first(), internal_shell_first(), cylinder_in_box(), flat_box(), spline_top_box(), square_sheet(),
        two_cavities(), half_space(), empty_solid(), face_located_box(), meshed_reversed_box(),
        located(box([0.0, 0.0, 0.0], [2.0, 3.0, 4.0]), translation(5.0, -1.0, 2.0)),
        located(cylinder([0.0, 0.0, 0.0], 1.0, 3.0), rotation([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.7)),
        shared_solids(translation(4.0, 0.0, 0.0), FORWARD),
        shared_solids(rotation([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 0.5), REVERSED, extra=True),
        shared_solids(point_mirror([5.0, 5.0, 5.0]), FORWARD), shared_solids(scaling([1.0, 1.0, 1.0], 2.0), FORWARD),
        shared_solids(list(IDENTITY), REVERSED),
    ]


def faces() -> list[ShapeSpec]:
    """Single faces on every surface kind (and a mesh-only face)."""
    patch = [BEZIER_SURFACE, 3, 3, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 2.0, 0.0, 1.0, 0.0, 1.0, 1.0, 1.0, 3.0,
             1.0, 2.0, 1.0, 2.0, 0.0, 0.0, 2.0, 1.0, -1.0, 2.0, 2.0, 0.5]
    return [
        holed_plate(), spline_blob(), ellipse_disc(), ellipse_disc(tilted=True), sphere_patch(0.2, 1.4, -0.6, 0.9),
        bezier_patch(True), bezier_patch(False), spline_patch(True), spline_patch(False), revolution_face(),
        extrusion_face(False), extrusion_face(True), offset_face(patch, 0.25),
        offset_face([PLANE, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0], 1.5), side_face(), infinite_plane(),
        single(FACE, torus([0.0, 0.0, 0.0], 3.0, 1.0)), single(FACE, frustum(2.0, 1.0, 3.0)),
        single(FACE, sphere([0.0, 0.0, 0.0], 2.0)), meshed_box_face(), mesh_only_face(), wide_spline_face(),
    ]


def edges() -> list[ShapeSpec]:
    """Free edges and compounds of edges for the linear properties."""
    return [free_edges(), curve_edges(), pcurve_only_edge(), mesh_edge_only()]


def append_package(requests: list[str]) -> None:
    """`bgp_package`: every package function with its flags."""
    for spec in solids() + faces() + edges():
        for skip in (0, 1):
            requests.append(request('bgp_package', spec, 0, skip, 0))
            requests.append(request('bgp_package', spec, 0, skip, 1))
            requests.append(request('bgp_package', spec, 1, skip, 0))
            requests.append(request('bgp_package', spec, 1, skip, 1))
            requests.append(request('bgp_package', spec, 2, 1e-6, skip))
            requests.append(request('bgp_package', spec, 3, 0, skip, 0))
            requests.append(request('bgp_package', spec, 3, 1, skip, 1))
            requests.append(request('bgp_package', spec, 4, 1e-6, 0, skip))
        requests.append(request('bgp_package', spec, 2, 0.01, 0))
        requests.append(request('bgp_package', spec, 4, 0.5, 1, 0))
        for eps, closed, span, cg, inertia, skip in GK_OPTIONS:
            requests.append(request('bgp_package', spec, 5, eps, closed, span, cg, inertia, skip))
        requests.append(request('bgp_package', spec, 6, 0.0, 0.0, -1.0, 0.0, 0.0, 1.0, 0.05, 0, 0, 1, 1, 0))


def bad_polygon_edge() -> ShapeSpec:
    """A meshed plane face whose edge polygon names node 9 of a 4-node triangulation."""
    spec = box([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], mesh=0.1)
    edge, _, _ = spec.faces[5].mesh.polygons[0]
    spec.faces[5].mesh.polygons[0] = (edge, [1, 9], 0.1)
    spec.top = [(FACE, 5)]
    return spec


def append_edges(requests: list[str]) -> None:
    """`bgp_cinert`, `bgp_mesh_cinert` and `bgp_mesh_nodes`."""
    curves = [(free_edges(), range(5)), (curve_edges(), range(4)), (pcurve_only_edge(), range(1)),
              (box([0.0, 0.0, 0.0], [2.0, 3.0, 4.0]), (0, 5, 11)), (sphere([0.0, 0.0, 0.0], 2.0), (1,)),
              (spline_blob(), range(2))]
    for spec, indices in curves:
        for index in indices:
            for loc in ([0.0, 0.0, 0.0], [1.0, 2.0, -3.0]):
                requests.append(request('bgp_cinert', spec, index, *loc))
    polygons = [(free_edges(), range(7)), (mesh_edge_only(), range(12)), (face_located_box(), (0, 4, 11)),
                (box([0.0, 0.0, 0.0], [4.0, 3.0, 2.0], polygon=0.02), (0, 7)), (bad_polygon_edge(), range(12))]
    for spec, indices in polygons:
        for index in indices:
            requests.append(request('bgp_mesh_cinert', spec, index, 0.5, -0.5, 1.0))
    nodes = [[0.0, 0.0, 0.0], [3.0, 4.0, 0.0], [3.0, 4.0, 0.0], [3.0, 4.0, 12.0], [-1.0, 2.0, 5.0],
             [1e-17, 0.0, 0.0], [1e8, -1e8, 3.0]]
    for lower, count in [(1, 0), (1, 1), (1, 2), (1, 5), (1, 7), (0, 3), (2, 3), (-1, 1), (0, 1)]:
        points = [c for p in nodes[:count] for c in p]
        requests.append(f'bgp_mesh_nodes {numbers([lower, count, *points, 1.0, 1.0, 1.0])}')


def bad_triangle_mesh() -> ShapeSpec:
    """A face triangulation whose second triangle names node 7 of five."""
    spec = mesh_only_face()
    spec.faces[0].mesh.triangles[1] = (2, 7, 5)
    return spec


def empty_mesh() -> ShapeSpec:
    """A face with a triangulation of three nodes and no triangle."""
    spec = mesh_only_face()
    spec.faces[0].mesh.triangles = []
    return spec


# The transformations of tp_bgp_trsf: none, translation, rotation, scale, point mirror,
# scaled rotation, and the identity (a location that is not IsIdentity).
TRANSFORMS = [None, [0, 1.0, -2.0, 0.5], [1, 1.0, 0.0, 0.0, 0.0, 0.6, 0.8, 1.1], [2, 0.5, 0.5, 0.5, 2.0],
              [3, 1.0, 1.0, 1.0], [4, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.3, 0.5], [0, 0.0, 0.0, 0.0]]


def append_meshes(requests: list[str]) -> None:
    """`bgp_mesh_props` and `bgp_mesh_calc`."""
    meshed = box([0.0, 0.0, 0.0], [4.0, 3.0, 2.0], mesh=0.1)
    for face in range(6):
        for kind in (0, 1):
            for orientation in (FORWARD, REVERSED):
                for trsf in TRANSFORMS:
                    located = [0] if trsf is None else [1, *trsf]
                    fields = [face, kind, orientation, *located, 1.0, 0.5, -1.0]
                    requests.append(request('bgp_mesh_props', meshed, *fields))
    for spec in (mesh_only_face(), bad_triangle_mesh(), empty_mesh(), meshed_box_face()):
        for kind in (0, 1):
            for trsf in (None, TRANSFORMS[2], TRANSFORMS[3]):
                located = [0] if trsf is None else [1, *trsf]
                requests.append(request('bgp_mesh_props', spec, 0, kind, FORWARD, *located, 0.0, 0.0, 0.0))
    # A right triangle, a general one, collinear points, a tiny one and two equal points.
    triangles = [([0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 3.0, 0.0]),
                 ([1.0, 2.0, 3.0], [4.0, -1.0, 2.0], [0.5, 0.5, 7.0]),
                 ([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [2.0, 2.0, 2.0]),
                 ([0.0, 0.0, 0.0], [1e-9, 0.0, 0.0], [0.0, 1e-9, 0.0]),
                 ([5.0, 5.0, 5.0], [5.0, 5.0, 5.0], [6.0, 5.0, 5.0])]
    rules = [[1.0 / 6.0, 1.0 / 6.0, 1.0 / 6.0, 2.0 / 3.0, 1.0 / 6.0, 1.0 / 6.0, 1.0 / 6.0, 2.0 / 3.0, 1.0 / 6.0],
             [1.0 / 3.0, 1.0 / 3.0, 0.5], []]
    for a, b, c in triangles:
        for apex in ([0.0, 0.0, 0.0], [1.0, -2.0, 4.0]):
            for volume in (0, 1):
                for rule in rules:
                    requests.append(f'bgp_mesh_calc {numbers([*a, *b, *c, *apex, volume, len(rule) // 3, *rule])}')


def append_brepgprop_requests(requests: list[str]) -> None:
    """Append every BRepGProp request family."""
    append_package(requests)
    append_edges(requests)
    append_meshes(requests)
    __import__('tktopalgo_corpus_brepgprop_tools').append_brepgprop_tool_requests(requests)
