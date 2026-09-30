"""Deterministic requests for BRepLib_PointCloudShape (tools/oracle/tktopalgo_probe_breplib_pointcloud.cxx).

Every request carries its shape record, so it never relies on an earlier request. A
`blp_pointcloud` request builds one cloud (tolerance, distance, optional UV nodes on the
meshes of the plane, cylinder and sphere faces) and runs a list of operations on it:
the point count and the points by density (given, zero and negative densities, so the
default density with and without faces below the tolerance, counts kept between calls,
offsets along the normal), and the node count and the points of the triangulations
(meshes with and without UV nodes, located meshes, faces without mesh). The shapes:
boxes, a cylinder, a sphere, a torus, a sphere patch, a Bezier patch, a REVERSED face,
located and rotated boxes, a face without surface, an infinite plane, free edges and an
empty solid. Densities keep each request below a few hundred points.
"""
from __future__ import annotations
from tktopalgo_corpus_brepbndlib import (
    bezier_patch, face_located_box, free_edges, infinite_plane, located, meshed_box_face,
    mesh_only_face, numbers, sphere_patch)
from tktopalgo_corpus_brepbndlib_obb import meshed_cylinder, meshed_sphere
from tktopalgo_corpus_shapes import NO_SURFACE, REVERSED, ShapeSpec, rotation, translation
from tktopalgo_corpus_solids import box, cylinder, sphere, torus

# Operation codes of the probe.
NB_DENSITY, GEN_DENSITY, NB_MESH, GEN_MESH, SET_DIST, SET_TOL = range(6)


def reversed_face() -> ShapeSpec:
    """The top face of a meshed box, REVERSED."""
    spec = meshed_box_face()
    spec.faces[5].orientation = REVERSED
    return spec


def request(spec: ShapeSpec, tol: float, dist: float, uv: int, ops: list[tuple[int, float]]) -> str:
    """One request: the shape, the cloud settings and the operations."""
    fields = [*spec.numbers(), tol, dist, uv, len(ops)]
    for op, arg in ops:
        fields.extend([op, arg])
    return 'blp_pointcloud ' + numbers(fields)


# (shape, density giving a few dozen points) for the density families.
def density_shapes() -> list[tuple[ShapeSpec, float]]:
    return [
        (box([0.0, 0.0, 0.0], [4.0, 3.0, 2.0]), 1.0),
        (box([-1.0, 2.0, -3.0], [1.0, 2.5, 7.0], tol=0.01), 0.5),
        (cylinder([1.0, 2.0, 3.0], 1.0, 2.0), 0.5),
        (sphere([0.0, 0.0, 0.0], 2.0), 2.0),
        (torus([0.0, 0.0, 0.0], 5.0, 1.0), 5.0),
        (sphere_patch(0.2, 1.2, -0.3, 0.6), 0.1),
        (bezier_patch(True), 0.2),
        (bezier_patch(False), 0.2),
        (reversed_face(), 0.5),
        (face_located_box(), 0.5),
        (located(box([0.0, 0.0, 0.0], [1.0, 2.0, 3.0]), rotation([1.0, 0.0, 0.0], [0.0, 0.0, 1.0], 0.3)), 0.4),
        (located(cylinder([0.0, 0.0, 0.0], 1.0, 1.0), translation(2.0, -1.0, 0.5)), 0.4),
    ]


def density_requests() -> list[str]:
    """Counts and points by density, default density, offsets and kept counts."""
    out = []
    for spec, density in density_shapes():
        out.append(request(spec, 1e-7, 0.0, 0, [(NB_DENSITY, density)]))
        out.append(request(spec, 1e-7, 0.0, 0, [(GEN_DENSITY, density)]))
        out.append(request(spec, 1e-7, 0.25, 0, [(GEN_DENSITY, density)]))
        out.append(request(spec, 1e-7, 0.0, 0, [(NB_DENSITY, 0.0), (GEN_DENSITY, 0.0)]))
        out.append(request(spec, 1e-7, 0.0, 0, [(NB_DENSITY, -2.0), (NB_DENSITY, density * 2.0)]))
        out.append(request(spec, 1e-7, -1.0, 0, [(NB_DENSITY, density * 4.0), (GEN_DENSITY, density),
                                                 (SET_DIST, 0.1), (GEN_DENSITY, density)]))
    return out


def tolerance_requests() -> list[str]:
    """The default density with faces below the square of the tolerance."""
    spec = box([0.0, 0.0, 0.0], [4.0, 3.0, 0.5])
    return [request(spec, tol, 0.0, 0, [(NB_DENSITY, 0.0)]) for tol in (1e-7, 1.5, 2.5, 3.0, 10.0)] + [
        request(spec, 1e-7, 0.0, 0, [(SET_TOL, 2.5), (NB_DENSITY, 0.0), (SET_TOL, 1e-7), (NB_DENSITY, 0.0)])]


def mesh_requests() -> list[str]:
    """Node counts and points of the triangulations."""
    shapes = [
        box([0.0, 0.0, 0.0], [1.0, 2.0, 3.0], mesh=0.02),
        meshed_box_face(),
        reversed_face(),
        face_located_box(),
        meshed_sphere(2.0),
        meshed_cylinder(),
        located(meshed_cylinder(), translation(-3.0, 1.0, 2.0)),
        located(meshed_sphere(1.0), rotation([0.0, 0.0, 0.0], [1.0, 1.0, 0.0], 0.7)),
        mesh_only_face(),
        box([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]),
    ]
    out = []
    for spec in shapes:
        for uv in (0, 1):
            out.append(request(spec, 1e-7, 0.0, uv, [(NB_MESH, 0.0), (GEN_MESH, 0.0)]))
    out.append(request(shapes[0], 1e-7, 0.0, 1, [(NB_DENSITY, 1.0), (GEN_MESH, 0.0), (GEN_DENSITY, 1.0)]))
    return out


def edge_requests() -> list[str]:
    """Shapes without a usable face: free edges, an empty solid, a face without
    surface and the infinite plane."""
    shapes = [free_edges(), ShapeSpec(), mesh_only_face(), infinite_plane()]
    out = []
    for spec in shapes:
        out.append(request(spec, 1e-7, 0.0, 0, [(NB_DENSITY, 1.0), (NB_MESH, 0.0)]))
        out.append(request(spec, 1e-7, 0.0, 0, [(GEN_MESH, 0.0)]))
        if spec.faces and spec.faces[0].surface[0] == NO_SURFACE:
            # Density points on a face without surface crash natively (a null surface
            # is dereferenced before addDensityPoints checks it); left out.
            continue
        out.append(request(spec, 1e-7, 0.0, 0, [(GEN_DENSITY, 1.0)]))
        out.append(request(spec, 1e-7, 0.0, 0, [(GEN_DENSITY, 0.0)]))
    return out


def append_breplib_pointcloud_requests(requests: list[str]) -> None:
    """Append every `blp_pointcloud` request."""
    requests.extend(density_requests())
    requests.extend(tolerance_requests())
    requests.extend(mesh_requests())
    requests.extend(edge_requests())
