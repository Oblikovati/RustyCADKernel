// Rust translation of synthesizeFaceBoundaries of
// ModelingData/TKBRep/BRepGraphInc/BRepGraphInc_Populate.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The natural-restriction boundaries: a face without wires but with the flag gets
//! the vertices, edges, coedges and wire of its surface's parameter box.
use super::boundary_builder::{ParameterBox, SurfaceBoundary, build_surface_boundary};
use super::context::BuildContext;
use super::face::FaceBuildData;
use super::register::append_vertex_ref;
use crate::brep_graph::{EdgeId, VertexId};
use crate::brep_graph_inc::{ParityOrientation, Storage, std_max};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomPlane, GeomRectangularTrimmedSurface, GeomSurface};

// Plane surfaces (or trimmed planes) get no stored p-curves: they derive from the 3D
// curves.
fn is_plane(surface: &Handle<dyn GeomSurface>) -> bool {
    if surface.cast::<GeomPlane>().is_some() {
        return true;
    }
    let trimmed = surface.cast::<GeomRectangularTrimmedSurface>();
    trimmed
        .get()
        .is_some_and(|t| t.basis_surface().cast::<GeomPlane>().is_some())
}

/// `synthesizeFaceBoundaries(theBuild, theFaceData)`: the boundary of every face that
/// needs one, attached as a FORWARD outer wire after canonicalisation.
pub(super) fn synthesize_face_boundaries(
    build: &mut BuildContext<'_>,
    faces: &[FaceBuildData],
) -> Result<(), Failure> {
    for data in faces.iter().filter(|data| data.needs_synthesis) {
        let surface = data
            .surface
            .get()
            .expect("a face needing synthesis has a surface");
        let tolerance = std_max(data.tolerance, precision::confusion());
        let (u_min, u_max, v_min, v_max) = surface.bounds();
        let parameters = ParameterBox {
            u_min,
            u_max,
            v_min,
            v_max,
        };
        let mut boundary = SurfaceBoundary::default();
        if !build_surface_boundary(&mut boundary, &data.surface, parameters, tolerance)? {
            continue;
        }
        let vertices = append_boundary_vertices(build.storage, &boundary, tolerance);
        let edges = append_boundary_edges(build.storage, &boundary, &vertices, tolerance);
        let wire = build.storage.append_wire();
        append_boundary_co_edges(
            build.storage,
            &boundary,
            &edges,
            data,
            wire,
            is_plane(&data.surface),
        )?;
        let status = build.storage.canonicalize_wire_co_edge_order_status(wire);
        build.record_wire_order_status(status);
        build
            .storage
            .attach_wire_to_face(data.face_id, wire, ParityOrientation::FORWARD);
    }
    Ok(())
}

fn append_boundary_vertices(
    storage: &mut Storage,
    boundary: &SurfaceBoundary,
    tolerance: f64,
) -> Vec<VertexId> {
    boundary
        .vertices
        .iter()
        .map(|point| {
            let id = storage.append_vertex();
            let def = storage.change_vertex(id);
            def.point = *point;
            def.tolerance = tolerance;
            id
        })
        .collect()
}

// The boundary edges: a 3D curve use unless degenerate (so the derived state flags
// them degenerate), the vertex references and the vertex-edge relations.
fn append_boundary_edges(
    storage: &mut Storage,
    boundary: &SurfaceBoundary,
    vertices: &[VertexId],
    tolerance: f64,
) -> Vec<EdgeId> {
    let mut edges = Vec::with_capacity(boundary.edges.len());
    for item in &boundary.edges {
        let curve_rep = if item.curve3d.is_some() && !item.is_degenerate {
            let rep = storage.append_edge_curve3d_rep();
            storage.change_edge_curve3d_rep(rep).curve = item.curve3d.clone();
            rep
        } else {
            Default::default()
        };
        let (start, end) = (vertices[item.start_vertex], vertices[item.end_vertex]);
        let edge = storage.append_edge();
        storage.change_edge(edge).curve3d_rep_id = curve_rep;
        storage.change_edge(edge).tolerance = tolerance;
        if curve_rep.is_valid() {
            let record = storage.change_edge_curve3d_rep(curve_rep);
            record.parent_edge_id = edge;
            record.param_first = item.first;
            record.param_last = item.last;
        }
        let start_ref = append_vertex_ref(storage, start, ParityOrientation::FORWARD, edge);
        storage.change_edge(edge).start_vertex_ref_id = start_ref;
        let end_ref = append_vertex_ref(storage, end, ParityOrientation::REVERSED, edge);
        storage.change_edge(edge).end_vertex_ref_id = end_ref;
        storage.attach_edge_to_vertex(edge, start);
        if start != end {
            storage.attach_edge_to_vertex(edge, end);
        }
        edges.push(edge);
    }
    edges
}

// The coedges on the face, with the 2D line p-curves on non-plane surfaces.
fn append_boundary_co_edges(
    storage: &mut Storage,
    boundary: &SurfaceBoundary,
    edges: &[EdgeId],
    data: &FaceBuildData,
    wire: crate::brep_graph::WireId,
    plane: bool,
) -> Result<(), Failure> {
    for item in &boundary.co_edges {
        let orientation = ParityOrientation::new(item.orientation)?;
        let co_edge =
            storage.create_co_edge_use(wire, edges[item.edge_index], data.face_id, orientation);
        if plane || item.pcurve.is_null() {
            continue;
        }
        let rep = storage.append_co_edge_curve2d_rep();
        let record = storage.change_co_edge_curve2d_rep(rep);
        record.curve = item.pcurve.clone();
        record.parent_co_edge_id = co_edge;
        record.param_first = item.first;
        record.param_last = item.last;
        storage.change_co_edge(co_edge).curve2d_rep_id = rep;
    }
    Ok(())
}
