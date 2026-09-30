"""The BRepGProp requests on single faces (tools/tktopalgo_corpus_brepgprop.py lists the
families): the Sinert, Vinert and VinertGK frameworks, the face tool, the domain, the
Gauss integrator and the integrand functions (tktopalgo_probe_brepgprop.cxx and
tktopalgo_probe_brepgprop_tools.cxx).
"""
from __future__ import annotations
from tktopalgo_corpus_brepbndlib import single, sphere_patch
from tktopalgo_corpus_brepgprop import faces, request
from tktopalgo_corpus_gprop_shapes import frustum, ring, spline_blob, spline_patch, extrusion_face
from tktopalgo_corpus_shapes import FACE, NO_SURFACE, ShapeSpec
from tktopalgo_corpus_solids import box, cylinder, sphere, torus

# The location, and a location away from the origin.
LOCATIONS = [[0.0, 0.0, 0.0], [1.0, -2.0, 3.0]]
# Tolerances of the adaptive integrations: tight (with the error estimate), loose
# (estimate, no refinement), above 0.001 (one pass), negative (estimate).
EPSILONS = [1e-6, 5e-4, 0.01, -1e-3]
# Bounds of the Vinert requests: the location, a point, a plane.
BOUNDS = [[0], [1, 1.0, 1.0, 1.0], [2, 0.0, 0.0, -1.0, 0.0, 0.6, 0.8]]


def patch_without_pcurve() -> ShapeSpec:
    """A spherical patch whose east edge has no pcurve (Load(E) fails on it)."""
    spec = sphere_patch(0.2, 1.4, -0.6, 0.9)
    orientation, uses = spec.faces[0].wires[0]
    edge, edge_orientation, _ = uses[1]
    uses[1] = (edge, edge_orientation, [0])
    return spec


def face_shapes() -> list[ShapeSpec]:
    """The single faces, the faces of a box, cylinder, sphere, torus, frustum and ring
    alone, and the patch without pcurve."""
    solids = [(box([0.0, 0.0, 0.0], [2.0, 3.0, 4.0]), (0, 5)), (cylinder([0.0, 0.0, 0.0], 1.5, 2.0), (1, 2)),
              (sphere([1.0, 0.0, 0.0], 2.0), (0,)), (torus([0.0, 0.0, 0.0], 3.0, 1.0), (0,)),
              (frustum(2.0, 1.0, 3.0), (1,)), (ring(3.0, 1.5, 2.0), (1, 2))]
    alone = [single(FACE, spec, k) for spec, indices in solids for k in indices]
    # A face without surface (the mesh-only face) crashes natively in every face tool
    # member (a null adaptor); only the package functions, which use its mesh, read it.
    with_surface = [spec for spec in faces() if spec.faces[0].surface != [NO_SURFACE]]
    return [*with_surface, *alone, patch_without_pcurve()]


def append_frames(requests: list[str], spec: ShapeSpec) -> None:
    """`bgp_sinert`, `bgp_vinert` and `bgp_vinert_gk` on face 0 of `spec`."""
    for loc in LOCATIONS:
        for mode in range(4):
            requests.append(request('bgp_sinert', spec, 1, 0, mode, *loc, EPSILONS[mode]))
        requests.append(request('bgp_sinert', spec, 4, 0, 4, 0, 5, 0, 6, 0, 7, *loc, 1e-3))
        for bound in BOUNDS:
            for mode in (4, 5, 6, 7):
                requests.append(request('bgp_vinert', spec, 1, 0, mode, *bound, *loc, EPSILONS[mode - 4]))
            chain = [c for mode in range(4) for c in (0, mode, *bound)]
            requests.append(request('bgp_vinert', spec, 4, *chain, *loc, 1e-3))
    # The Gauss-Kronrod options stay loose (the debug build of the Rust twin integrates
    # ten values per face with the inertia).
    for bound in BOUNDS:
        for mode, (tol, cg, inertia, span) in zip(range(4), [(1e-3, 0, 0, 0), (5e-2, 1, 0, 1), (5e-2, 0, 1, 0),
                                                             (0.1, 1, 1, 1)]):
            requests.append(request('bgp_vinert_gk', spec, 1, 0, mode, *bound, *LOCATIONS[1], tol, cg, inertia, span))
        chain = [c for mode in (0, 1) for c in (0, mode, *bound)]
        requests.append(request('bgp_vinert_gk', spec, 2, *chain, *LOCATIONS[0], 5e-2, 1, 0, 0))


def append_tools(requests: list[str], spec: ShapeSpec) -> None:
    """`bgp_face`, `bgp_domain` and `bgp_gauss` on face 0 of `spec`."""
    for span, eps in [(0, 1e-6), (1, 0.5), (0, 0.0)]:
        requests.append(request('bgp_face', spec, 0, span, eps))
    requests.append(request('bgp_domain', spec, 0))
    for kind in (0, 1):
        for overload in range(6):
            for by_point, coeff in [(1, [3, 0.5, -0.5, 1.0]), (0, [4, 0.0, 0.0, 1.0, -2.0])]:
                if by_point == 0 and (kind == 1 or overload in (1, 3, 5)):
                    continue
                eps = 1e-4 if overload >= 4 else 0.0
                requests.append(request('bgp_gauss', spec, 0, kind, overload, 1.0, 0.5, 0.0, eps, by_point, *coeff))


def append_knots(requests: list[str]) -> None:
    """`bgp_knots` and `bgp_arrays` on the spline faces, the extrusions and the blob."""
    ranges = [(0.0, 2.0, 0.0, 1.0), (0.3, 1.7, 0.2, 0.8), (1.0, 1.0, 0.5, 0.5), (-1.0, 3.0, -0.5, 2.0),
              (1.5, 0.5, 0.9, 0.1), (0.9999999, 1.0000001, 0.4999999, 0.5)]
    shapes = [(spline_patch(True), (-1,)), (spline_patch(False), (-1, 0, 1)), (extrusion_face(False), (-1, 0, 1)),
              (extrusion_face(True), (-1,)), (spline_blob(), (0, 1))]
    for spec, edges in shapes:
        for edge in edges:
            for span in (0, 1):
                for bounds in ranges:
                    requests.append(request('bgp_knots', spec, 0, span, edge, *bounds))
    arrays = [(spline_patch(False), -1, 0, [(1, 3), (1, 2), (1, 4), (0, 3), (2, 3)]),
              (spline_patch(False), -1, 1, [(1, 3), (1, 1), (0, 4)]),
              (spline_blob(), 0, 2, [(1, 3), (1, 2), (1, 5), (0, 3)]),
              (spline_blob(), 1, 2, [(1, 2), (1, 1), (3, 2)]),
              (single(FACE, sphere([0.0, 0.0, 0.0], 2.0)), -1, 1, [(1, 3), (1, 2), (0, 3)]),
              (single(FACE, box([0.0, 0.0, 0.0], [1.0, 1.0, 1.0])), -1, 0, [(1, 2), (1, 1), (-3, 5)])]
    for spec, edge, which, sizes in arrays:
        for lower, count in sizes:
            requests.append(request('bgp_arrays', spec, 0, edge, which, lower, count))


def append_functions(requests: list[str]) -> None:
    """`bgp_functions`: every value type by point and by plane."""
    shapes = [(spline_patch(False), (-1, 0, 1), (0.0, 2.0, 0.0, 1.0)), (sphere_patch(0.2, 1.4, -0.6, 0.9), (-1, 0, 2),
              (0.2, 1.4, -0.6, 0.9)), (ring(3.0, 1.5, 2.0), (-1, 2), (0.0, 6.283185307179586, 0.0, 2.0))]
    coefficients = [(1, [3, 0.0, 0.0, 0.0]), (1, [3, 0.3, -0.2, 0.1]), (0, [4, 0.0, 0.6, 0.8, 0.5])]
    for spec, edges, (u1, u2, v1, v2) in shapes:
        for edge in edges:
            for by_point, coeff in coefficients:
                for kind in range(11):
                    samples = [3, u1 + 0.3 * (u2 - u1), v1 + 0.6 * (v2 - v1), u2, v2, 0.5 * (u1 + u2), v1]
                    t_values = [3, 0.0, 0.4, 1.0]
                    requests.append(request('bgp_functions', spec, 0, edge, 1, by_point, 0.5, 0.5, 0.5, *coeff, kind,
                                            u1, 1e-3, *samples, *t_values))


def append_brepgprop_tool_requests(requests: list[str]) -> None:
    """Append the single-face families in a fixed order."""
    for spec in face_shapes():
        append_frames(requests, spec)
        append_tools(requests, spec)
    append_knots(requests)
    append_functions(requests)
