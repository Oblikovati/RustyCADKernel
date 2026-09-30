// Rust translation of the geometry loops of applyGeometryTransform and
// applyGeometryTransformInCopiedRange of ModelingData/TKBRep/BRepGraph/BRepGraph_Transform.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The geometry-level transform: the vertex points, surfaces and 3D curves of the range
//! are replaced by transformed copies through the editor (`BRepBuilderAPI_Transform` with
//! `theCopyGeom = true`). P-curves live in UV space and are untouched.
use super::{CopiedRange, ids_in_range};
use crate::brep_graph::tool;
use crate::brep_graph::*;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkmath::gp::Transform3;

/// The vertex, face and edge loops shared by `applyGeometryTransform` (the default,
/// whole-graph range) and `applyGeometryTransformInCopiedRange`. Without
/// `copy_mesh` the persistent triangulations and the mesh cache of the transformed
/// faces and edges are dropped (a mesh is stale after a geometry transform); with it,
/// the caller copies or transforms the mesh afterwards.
pub(crate) fn transform_geometry(
    graph: &mut BRepGraph,
    range: &CopiedRange,
    transform: &Transform3,
    copy_mesh: bool,
) -> Result<(), Failure> {
    for vertex in ids_in_range::<VertexTag>(graph, range) {
        let point = graph
            .topo()
            .vertices()
            .definition(vertex)
            .point
            .transformed(*transform);
        graph.editor().vertices().set_point(vertex, point)?;
    }
    for face in ids_in_range::<FaceTag>(graph, range) {
        transform_face(graph, face, transform, copy_mesh)?;
    }
    for edge in ids_in_range::<EdgeTag>(graph, range) {
        transform_edge(graph, edge, transform, copy_mesh)?;
    }
    Ok(())
}

fn transform_face(
    graph: &mut BRepGraph,
    face: FaceId,
    transform: &Transform3,
    copy_mesh: bool,
) -> Result<(), Failure> {
    if tool::face::has_surface(graph, face) {
        let surface = tool::face::surface(graph, face);
        if !surface.is_null() {
            let moved = surface.transformed(*transform)?.cast::<dyn GeomSurface>();
            graph.editor().faces().set_surface(face, &moved)?;
        }
    }
    if !copy_mesh {
        graph
            .editor()
            .faces()
            .clear_persistent_triangulation(face)?;
        graph.mesh().editor().faces().clear(face)?;
    }
    Ok(())
}

fn transform_edge(
    graph: &mut BRepGraph,
    edge: EdgeId,
    transform: &Transform3,
    copy_mesh: bool,
) -> Result<(), Failure> {
    if tool::edge::has_curve(graph, edge) {
        let curve = tool::edge::curve(graph, edge);
        if !curve.is_null() {
            let (first, last) = tool::edge::range(graph, edge);
            let moved = curve.transformed(*transform)?.cast::<dyn GeomCurve>();
            graph
                .editor()
                .edges()
                .set_curve(edge, &moved, first, last)?;
        }
    }
    if !copy_mesh {
        graph.mesh().editor().edges().clear(edge);
    }
    Ok(())
}
