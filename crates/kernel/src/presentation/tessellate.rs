// SPDX-License-Identifier: GPL-2.0-only
//! The tessellation pipeline: enumerate, copy and mesh, then read faces, edges and
//! vertices in stable order.
use super::detached::{meshed_copy, native};
use super::edge_polyline::edge_display;
use super::face_mesh::{MeshedFace, face_triangles, meshed_face};
use super::mesh::{FaceTriangles, PresentationMesh};
use super::options::DisplayOptions;
use crate::query_error::{BodyQuery, BodyQueryError, QueryEntity};
use crate::topology::{BodyTopology, FaceIndex, VertexIndex, enumerate_for};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Face, Shape};

/// Tessellates `body` for display within `options`, keyed by stable topology indices.
///
/// The body is copied and the copy is meshed sequentially, so the body, its history
/// and any mesh it already carries stay unchanged, and equal inputs give
/// byte-identical results on every run and thread. A mesh with defects, a face
/// without triangles or an edge without a polygon is refused, never returned partial.
///
/// ```
/// use rustycad_kernel::presentation::{DisplayOptions, tessellate};
/// use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
/// use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
/// let body = BRepPrimAPIMakeBox::from_sizes(1.0, 1.0, 1.0)?.shape()?.clone();
/// let mesh = tessellate(&body, &DisplayOptions::new(0.01, 0.5)?)?;
/// assert_eq!(mesh.faces.len(), 6);
/// assert_eq!(mesh.triangle_count(), 12);
/// assert_eq!((mesh.edges.len(), mesh.vertices.len()), (12, 8));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn tessellate(
    body: &Shape,
    options: &DisplayOptions,
) -> Result<PresentationMesh, BodyQueryError> {
    let topology = enumerate_for(body, BodyQuery::Tessellate)?;
    let detached = meshed_copy(body, &topology, options)?;
    let meshed = meshed_faces(&detached.faces)?;
    let faces = owned_faces(&meshed)?;
    let edges = edge_display(&detached.edges, &meshed)?;
    Ok(PresentationMesh {
        requested: *options,
        effective: detached.effective,
        faces,
        edges: edges.polylines,
        degenerated_edges: edges.degenerated,
        vertices: vertex_positions(&topology)?,
    })
}

fn meshed_faces(faces: &[Face]) -> Result<Vec<MeshedFace>, BodyQueryError> {
    faces
        .iter()
        .enumerate()
        .map(|(position, face)| meshed_face(face, FaceIndex(position as u32)))
        .collect()
}

fn owned_faces(meshed: &[MeshedFace]) -> Result<Vec<FaceTriangles>, BodyQueryError> {
    meshed
        .iter()
        .enumerate()
        .map(|(position, face)| face_triangles(face, FaceIndex(position as u32)))
        .collect()
}

fn vertex_positions(topology: &BodyTopology) -> Result<Vec<[f64; 3]>, BodyQueryError> {
    (0..topology.vertex_count() as u32)
        .map(|index| {
            let vertex = topology
                .vertex(VertexIndex(index))
                .expect("index below the count");
            let point = BRepTool::pnt(&vertex)
                .map_err(|failure| native(QueryEntity::Body, "vertex point", &failure))?;
            Ok([point.x(), point.y(), point.z()])
        })
        .collect()
}
