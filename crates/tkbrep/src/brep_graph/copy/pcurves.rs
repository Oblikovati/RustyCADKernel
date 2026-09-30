// Rust translation of `ensurePCurvesForFace` of
// ModelingData/TKBRep/BRepGraph/BRepGraph_Copy.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The face-bound data of the copied coedges: p-curve, face and mesh polygons.
use super::context::CopyContext;
use super::meshes::{ensure_polygon_on_tri_rep, ensure_polygon2d_rep};
use super::policy::{MeshPolicy, copy_geometry};
use crate::brep_graph::refs_iterator::{CoEdgesOfWire, RefsWireOfFace};
use crate::brep_graph::{CoEdgeId, FaceId};
use crate::brep_graph_inc::CoEdgeDef;
use rustycad_tkernel::standard::Failure;

/// `ensurePCurvesForFace(theCtx, theSrcFaceId, theNewFaceId)`: for every coedge of the
/// source face's wires that the face owns and that was copied: its p-curve (under the
/// geometry policy), the copied face, then its 2D polygon and polygon on triangulation.
pub(crate) fn ensure_pcurves_for_face(
    context: &mut CopyContext<'_>,
    source_face: FaceId,
    new_face: FaceId,
) -> Result<(), Failure> {
    for co_edge in face_co_edges(context, source_face) {
        let def = context
            .source()
            .topo()
            .co_edges()
            .definition(co_edge)
            .clone();
        if def.face_id != source_face {
            continue;
        }
        let Some(copied) = context.existing(co_edge) else {
            continue;
        };
        copy_co_edge_pcurve(context, &def, copied)?;
        // Set FaceId first so SetPersistentPolygonOnTri can resolve the triangulation.
        context
            .target_mut()
            .editor()
            .co_edges()
            .set_face_id(copied, new_face)?;
        if context.mesh != MeshPolicy::Drop && def.polygon2d_rep_id.is_valid() {
            ensure_polygon2d_rep(context, def.polygon2d_rep_id, copied)?;
        }
        if context.mesh != MeshPolicy::Drop && def.polygon_on_tri_rep_id.is_valid() {
            ensure_polygon_on_tri_rep(context, def.polygon_on_tri_rep_id, copied)?;
        }
    }
    Ok(())
}

// The coedges of every wire of the source face, in wire then coedge order.
fn face_co_edges(context: &CopyContext<'_>, face: FaceId) -> Vec<CoEdgeId> {
    let graph = context.source();
    RefsWireOfFace::new(graph, face)
        .flat_map(|reference| {
            let wire = graph.refs().wires().entry(reference).child_wire_id;
            CoEdgesOfWire::new(graph, wire)
        })
        .collect()
}

// The p-curve of a live source record, copied with its range.
fn copy_co_edge_pcurve(
    context: &mut CopyContext<'_>,
    source: &CoEdgeDef,
    copied: CoEdgeId,
) -> Result<(), Failure> {
    let rep = source.curve2d_rep_id;
    let storage = context.src_storage();
    if !rep.is_valid()
        || !rep.is_valid_for(storage.nb_co_edge_curves2d())
        || storage.is_removed(rep)
    {
        return Ok(());
    }
    let record = storage.co_edge_curve2d_rep(rep);
    let (first, last) = (record.param_first, record.param_last);
    let curve = copy_geometry(&record.curve, context.geom)?;
    context
        .target_mut()
        .editor()
        .co_edges()
        .set_pcurve(copied, &curve, first, last)
}
