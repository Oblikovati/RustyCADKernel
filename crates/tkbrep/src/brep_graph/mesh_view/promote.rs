// Rust translation of BRepGraph::MeshView::EditorView::PromoteToPersistent of
// ModelingData/TKBRep/BRepGraph/BRepGraph_MeshView.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `MeshView::EditorView::PromoteToPersistent`: the fresh mesh cache entries become
//! persistent representations, written through the graph editor.
use super::editor_view::EditorView;
use crate::brep_graph::{BRepGraph, CoEdgeId, EdgeId, FaceId};
use rustycad_tkernel::standard::Failure;

impl EditorView<'_> {
    /// `PromoteToPersistent()`: every fresh, non-null default-slot cache entry of an
    /// active face, edge and coedge (in that order, by ascending id) is written to the
    /// definition through the graph editor's `SetPersistent*` setters, so the layer
    /// notifications and cache invalidation of each write fire as natively. A coedge
    /// promotes its 2D polygon, then its first polygon on triangulation. It takes the
    /// graph mutably because the view itself only borrows it; a lock or an active
    /// guard stops the promotion at the first refused write (Standard_ProgramError).
    ///
    /// ```
    /// use rustycad_tkbrep::brep_graph::BRepGraph;
    /// use rustycad_tkbrep::brep_graph::mesh_view::editor_view::EditorView;
    /// let mut graph = BRepGraph::new();
    /// EditorView::promote_to_persistent(&mut graph).unwrap();
    /// ```
    pub fn promote_to_persistent(graph: &mut BRepGraph) -> Result<(), Failure> {
        promote_faces(graph)?;
        promote_edges(graph)?;
        promote_co_edges(graph)
    }
}

// Each entry is re-read for its id, as natively: an earlier write notifies its node and
// can make later cache entries stale.
fn promote_faces(graph: &mut BRepGraph) -> Result<(), Failure> {
    for index in 0..graph.topo().faces().nb() {
        let face = FaceId::new(index);
        if face.is_removed(graph) {
            continue;
        }
        let entry = graph.mesh().cache().faces().entry(face);
        if let Some(entry) = entry.filter(|entry| entry.triangulation.is_some()) {
            graph
                .editor()
                .faces()
                .set_persistent_triangulation(face, &entry.triangulation)?;
        }
    }
    Ok(())
}

fn promote_edges(graph: &mut BRepGraph) -> Result<(), Failure> {
    for index in 0..graph.topo().edges().nb() {
        let edge = EdgeId::new(index);
        if edge.is_removed(graph) {
            continue;
        }
        let entry = graph.mesh().cache().edges().entry(edge);
        if let Some(entry) = entry.filter(|entry| entry.polygon3d.is_some()) {
            graph
                .editor()
                .edges()
                .set_persistent_polygon3d(edge, &entry.polygon3d)?;
        }
    }
    Ok(())
}

fn promote_co_edges(graph: &mut BRepGraph) -> Result<(), Failure> {
    for index in 0..graph.topo().co_edges().nb() {
        let co_edge = CoEdgeId::new(index);
        if !co_edge.is_removed(graph) {
            promote_co_edge(graph, co_edge)?;
        }
    }
    Ok(())
}

fn promote_co_edge(graph: &mut BRepGraph, co_edge: CoEdgeId) -> Result<(), Failure> {
    let polygon2d = graph.mesh().cache().co_edges().find_polygon2d(co_edge);
    if let Some(entry) = polygon2d.filter(|entry| entry.polygon2d.is_some()) {
        graph
            .editor()
            .co_edges()
            .set_persistent_polygon2d(co_edge, &entry.polygon2d)?;
    }
    let on_tri = graph.mesh().cache().co_edges().find_polygon_on_tri(co_edge);
    let first = on_tri.and_then(|entry| entry.polygons_on_tri.first().cloned());
    match first {
        Some(polygon) if polygon.is_some() => graph
            .editor()
            .co_edges()
            .set_persistent_polygon_on_tri(co_edge, &polygon),
        _ => Ok(()),
    }
}
